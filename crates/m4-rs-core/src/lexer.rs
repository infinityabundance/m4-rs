// m4-rs lexer — byte-oriented tokenizer for GNU m4 input.
//
// WHO:   infinityabundance. Original m4 tokenizer design by Kernighan & Ritchie (1977).
// WHAT:  Reads raw input bytes and produces a stream of Tokens matching GNU m4's
//        lexical rules: names, text, quote delimiters, comment delimiters, and
//        punctuation for argument lists.
// WHEN:  Invoked by the expansion engine whenever new input arrives (files, stdin,
//        macro expansion rescan). Tokenization is the first stage of m4 processing.
// WHERE: crates/m4-rs-core/src/lexer.rs — consumed by expansion.rs.
// WHY:   GNU m4 has configurable multi-byte quote and comment delimiters via
//        changequote/changecom. A byte-at-a-time lexer without lookahead cannot
//        correctly match multi-byte delimiters like `[` + `]` or `/*` + `*/`.
//        This rewrite adds a lookahead buffer for delimiter matching.
// HOW:   The lexer maintains:
//        - A text accumulator (buf) for plain text being collected
//        - A name accumulator (name_buf) for macro names being built
//        - A quote depth counter for nested quote tracking
//        - A lookahead window for multi-byte delimiter matching
//        - Source location tracking for diagnostics
//
//        The tokenize() method processes all bytes at once, using the internal
//        position pointer and lookahead window to handle multi-byte sequences.

use crate::quote::QuoteConfig;
use crate::token::{SourceLocation, Token, TokenKind};

/// The lexer state machine with multi-byte delimiter support.
pub struct Lexer {
    /// Current quote/comment configuration (mutable via changequote/changecom).
    pub quote_config: QuoteConfig,
    /// Current source location for diagnostics.
    pub location: SourceLocation,
    /// Quote nesting depth — how many open-quote levels we're inside.
    quote_depth: usize,
    /// Accumulator for plain text being collected.
    text_buf: Vec<u8>,
    /// Accumulator for a macro name being built.
    name_buf: Vec<u8>,
    /// Current position in the input slice during tokenize().
    pos: usize,
    /// The input being tokenized (set by tokenize()).
    input: Vec<u8>,
    /// Whether we're inside a quoted string collecting quoted text.
    in_quoted: bool,
    /// Pending token that was deferred because another token took priority.
    /// Used when punctuation causes both text and name to flush.
    pending_token: Option<Token>,
}

impl Lexer {
    /// Create a new lexer with default GNU m4 delimiters.
    ///
    /// Default quote: `` `' '' (backtick 0x60 opens, apostrophe 0x27 closes)
    /// Default comment: `#` opens, `\n` (newline) closes
    ///
    /// Buffers are pre-allocated with conservative sizes to avoid
    /// reallocation in the common case, reducing allocator pressure.
    pub fn new() -> Self {
        Self {
            quote_config: QuoteConfig::default(),
            location: SourceLocation {
                file: "<stdin>".to_string(),
                line: 1,
                column: 1,
            },
            quote_depth: 0,
            text_buf: Vec::with_capacity(256),
            name_buf: Vec::with_capacity(64),
            pos: 0,
            input: Vec::new(),
            in_quoted: false,
            pending_token: None,
        }
    }

    /// Create a lexer with a specific file name for diagnostics.
    pub fn with_file(file: String) -> Self {
        let mut lex = Self::new();
        lex.location.file = file;
        lex
    }

    /// Tokenize an entire input buffer at once, returning all tokens.
    ///
    /// This is the main entry point. It resets internal state and processes
    /// all input bytes, emitting a complete token stream.
    ///
    /// The input is copied into the lexer's internal buffer (`input.to_vec()`).
    /// For owned data, prefer `tokenize_owned()` to avoid the copy.
    ///
    /// If an unclosed quote is detected (quote_depth > 0 at EOF), the
    /// accumulated text is NOT flushed — matching GNU m4 behavior which
    /// suppresses output on error.
    pub fn tokenize(&mut self, input: &[u8]) -> Vec<Token> {
        self.tokenize_owned(input.to_vec())
    }

    /// Tokenize an owned input buffer, moving it into the lexer.
    ///
    /// This avoids the `to_vec()` copy that `tokenize()` performs.
    /// Use this when the caller already owns the input (e.g., from
    /// `std::fs::read()` or `read_to_end()`).
    pub fn tokenize_owned(&mut self, input: Vec<u8>) -> Vec<Token> {
        // Pre-allocate: each input byte typically produces fewer than
        // 1/4 token on average. Worst-case pathological input with
        // alternating single-byte names/text could produce 1 token/byte,
        // but 1/4 is a good conservative heuristic that avoids doubling.
        let mut tokens = Vec::with_capacity(input.len() / 4);
        self.pos = 0;
        self.input = input;
        self.text_buf.clear();
        self.name_buf.clear();
        self.quote_depth = 0;
        self.in_quoted = false;
        self.location.line = 1;
        self.location.column = 1;

        while self.pos < self.input.len() {
            if let Some(token) = self.next_token() {
                tokens.push(token);
            }
        }

        // If we ended inside an unclosed quote, suppress the remaining text.
        // GNU m4 outputs nothing for unclosed quotes — it only reports an error.
        // Only flush if quote_depth is 0 (all quotes properly closed).
        if self.quote_depth == 0 {
            if let Some(token) = self.flush() {
                tokens.push(token);
            }
        }

        tokens
    }

    /// Extract the next token from the current position.
    /// Returns None if more input is needed (shouldn't happen in batch mode).
    fn next_token(&mut self) -> Option<Token> {
        // Return any pending deferred token first
        if let Some(t) = self.pending_token.take() {
            return Some(t);
        }
        let remaining = self.input.len() - self.pos;
        if remaining == 0 {
            return None;
        }

        let byte = self.input[self.pos];

        // End of input — flush any pending name
        // NUL byte: GNU m4 warns and treats as end-of-string.
        if byte == 0 {
            eprintln!(
                "{}:{}: warning: NUL character in input",
                self.location.file, self.location.line
            );
            self.pos += 1;
            return self.flush();
        }

        // Token-boundary marker (0x01): flush pending text and continue.
        // Inserted by nested empty-quote handling. When re-lexing a macro
        // body, this marker splits text into separate tokens so subsequent
        // names can be recognized (e.g., `text``'``macro` → "text" + Name("macro")).
        if byte == 0x01 {
            let flushed = std::mem::take(&mut self.text_buf);
            self.pos += 1;
            // Also flush any pending name
            if let Some(name_tok) = self.flush_name() {
                self.pending_token = if !flushed.is_empty() {
                    Some(Token::new(TokenKind::Text, flushed, self.location.clone()))
                } else {
                    None
                };
                return Some(name_tok);
            }
            if !flushed.is_empty() {
                let loc = self.location.clone();
                return Some(Token::new(TokenKind::Text, flushed, loc));
            }
            return None;
        }

        // ---- Inside a quoted string ----
        if self.quote_depth > 0 {
            return self.lex_inside_quotes();
        }

        // ---- Comment detection (before quote, per GNU m4 order) ----
        if self.quote_config.comments_enabled {
            if let Some(token) = self.try_consume_comment() {
                return Some(token);
            }
        }

        // ---- Quote open detection ----
        if self.quote_config.quoting_enabled && self.try_match(&self.quote_config.open.clone()) {
            // Flush any pending NAME *and* text before entering quotes. Without the name flush,
            // `name[]macro` left `name` in name_buf and the following macro's chars appended to it
            // (`z[]patsubst(...)` -> one Name `zpatsubst`, never expanded). A quote ends the current
            // name just like `(` does, so `z[]patsubst` must lex as Name(z) + empty-quote + Name(patsubst).
            // (postgres pgac_arg_to_variable = `$1[]_[]patsubst($2, -, _)`.)
            let maybe_name = self.flush_name();
            let maybe_text = self.flush_text();
            self.quote_depth = 1;
            self.in_quoted = true;
            self.text_buf.clear();
            let open_len = self.quote_config.open.len();
            self.pos += open_len;
            self.advance(open_len);
            // At most one of name/text is normally pending (a starting name flushes text first);
            // if both somehow are, return the name now and defer the text.
            if let Some(nt) = maybe_name {
                if let Some(tt) = maybe_text {
                    self.pending_token = Some(tt);
                }
                return Some(nt);
            }
            if let Some(t) = maybe_text {
                return Some(t);
            }
            return None; // quoted text will be collected later
        }

        // ---- Parentheses and comma (always single-byte) ----
        if byte == b'(' {
            let loc = self.location.clone();
            // Flush pending name/text before emitting paren
            if let Some(t) = self.flush_name() {
                if let Some(tt) = self.flush_text() {
                    self.pending_token = Some(tt);
                }
                return Some(t);
            }
            if let Some(t) = self.flush_text() {
                return Some(t);
            }
            self.pos += 1;
            self.advance(1);
            return Some(Token::new(TokenKind::ParenOpen, vec![b'('], loc));
        }
        if byte == b')' {
            let loc = self.location.clone();
            if let Some(t) = self.flush_name() {
                if let Some(tt) = self.flush_text() {
                    self.pending_token = Some(tt);
                }
                return Some(t);
            }
            if let Some(t) = self.flush_text() {
                return Some(t);
            }
            self.pos += 1;
            self.advance(1);
            return Some(Token::new(TokenKind::ParenClose, vec![b')'], loc));
        }
        if byte == b',' {
            let loc = self.location.clone();
            if let Some(t) = self.flush_name() {
                if let Some(tt) = self.flush_text() {
                    self.pending_token = Some(tt);
                }
                return Some(t);
            }
            if let Some(t) = self.flush_text() {
                return Some(t);
            }
            self.pos += 1;
            self.advance(1);
            return Some(Token::new(TokenKind::Comma, vec![b','], loc));
        }

        // ---- Name detection ----
        if is_name_start(byte) {
            // Flush pending text before starting a name
            if let Some(text_token) = self.flush_text() {
                return Some(text_token);
            }
            self.name_buf.push(byte);
            self.pos += 1;
            self.advance(1);
            return None;
        }

        // ---- Continuing a name ----
        if !self.name_buf.is_empty() {
            if is_name_continuation(byte) {
                self.name_buf.push(byte);
                self.pos += 1;
                self.advance(1);
                return None;
            } else {
                // Name ended — the current byte will be handled next iteration
                return self.flush_name();
            }
        }

        // ---- Plain text byte ----
        self.text_buf.push(byte);
        self.pos += 1;
        self.advance(1);
        None
    }

    fn lex_inside_quotes(&mut self) -> Option<Token> {
        // Check for close-quote delimiter
        if self.try_match(&self.quote_config.close.clone()) {
            self.quote_depth -= 1;
            if self.quote_depth == 0 {
                // End of quoted text — emit it as a Text token
                self.in_quoted = false;
                let close_len = self.quote_config.close.len();
                self.pos += close_len;
                self.advance(close_len);
                let loc = self.location.clone();
                let text = std::mem::take(&mut self.text_buf);
                let mut t = Token::new(TokenKind::Text, text, loc);
                t.from_quote = true; // interior of a quoted region — protect from arg-start ws-strip
                return Some(t);
            } else {
                // Nested close-quote — preserve close-quote chars in text.
                // GNU m4 preserves one level of inner quoting in output.
                let close_bytes = self.quote_config.close.clone().into_bytes();
                self.text_buf.extend_from_slice(&close_bytes);
                let close_len = close_bytes.len();
                self.pos += close_len;
                self.advance(close_len);
                return None;
            }
        }

        // Check for open-quote inside quotes (nesting)
        if self.try_match(&self.quote_config.open.clone()) {
            // Nested open-quote — preserve open-quote chars in text.
            // GNU m4 preserves one level of inner quoting in output.
            let open_bytes = self.quote_config.open.clone().into_bytes();
            self.text_buf.extend_from_slice(&open_bytes);
            self.quote_depth += 1;
            let open_len = open_bytes.len();
            self.pos += open_len;
            self.advance(open_len);
            return None;
        }

        let byte = self.input[self.pos];
        if byte == 0x01 {
            let flushed = std::mem::take(&mut self.text_buf);
            self.pos += 1;
            if !flushed.is_empty() {
                let loc = self.location.clone();
                return Some(Token::new(TokenKind::Text, flushed, loc));
            }
            return None;
        }
        self.text_buf.push(byte);
        self.pos += 1;
        self.advance(1);
        None
    }

    /// Try to consume a comment at the current position.
    /// Returns a Text token (empty, discarded) if a comment was consumed,
    /// or None if no comment delimiter at this position.
    fn try_consume_comment(&mut self) -> Option<Token> {
        if !self.try_match(&self.quote_config.comment_open.clone()) {
            return None;
        }

        // Flush any pending text before the comment
        let maybe_text = self.flush_text();
        let maybe_name = self.flush_name();

        // Consume the comment open delimiter
        let open_len = self.quote_config.comment_open.len();
        self.pos += open_len;
        self.advance(open_len);

        // Find the comment close delimiter
        let close = &self.quote_config.comment_close;
        let close_bytes = close.as_bytes();
        let _search_start = self.pos;

        while self.pos + close_bytes.len() <= self.input.len() {
            if &self.input[self.pos..self.pos + close_bytes.len()] == close_bytes {
                // Found close delimiter — consume it too (GNU m4 consumes
                // the newline for # comments, so no blank line appears)
                self.pos += close_bytes.len();
                // Update line count if close is newline
                if close_bytes == b"\n" {
                    self.location.line += 1;
                    self.location.column = 1;
                } else {
                    self.advance(close_bytes.len());
                }
                // Comments produce no output token. Return any pending text/name.
                if let Some(t) = maybe_name {
                    return Some(t);
                }
                return maybe_text;
            }
            self.pos += 1;
        }

        // Comment extends to EOF — consume all remaining input
        self.pos = self.input.len();
        if let Some(t) = maybe_name {
            return Some(t);
        }
        maybe_text
    }

    /// Try to match a delimiter string at the current position.
    /// Returns true if the full delimiter matches, false otherwise.
    /// Does NOT advance the position.
    fn try_match(&self, delimiter: &str) -> bool {
        let delim_bytes = delimiter.as_bytes();
        if delim_bytes.is_empty() {
            return false;
        }
        if self.pos + delim_bytes.len() > self.input.len() {
            return false;
        }
        &self.input[self.pos..self.pos + delim_bytes.len()] == delim_bytes
    }

    /// Flush accumulated plain text as a Text token.
    fn flush_text(&mut self) -> Option<Token> {
        if self.text_buf.is_empty() {
            return None;
        }
        let loc = self.location.clone();
        let text = std::mem::take(&mut self.text_buf);
        Some(Token::new(TokenKind::Text, text, loc))
    }

    /// Flush accumulated name as a Name token.
    fn flush_name(&mut self) -> Option<Token> {
        if self.name_buf.is_empty() {
            return None;
        }
        let loc = self.location.clone();
        let name = std::mem::take(&mut self.name_buf);
        Some(Token::new(TokenKind::Name, name, loc))
    }

    /// Flush any remaining accumulated text or name at EOF.
    pub fn flush(&mut self) -> Option<Token> {
        if let Some(t) = self.flush_name() {
            return Some(t);
        }
        self.flush_text()
    }

    /// Advance source location by N bytes, tracking newlines.
    fn advance(&mut self, n: usize) {
        // Count newlines in the consumed bytes for accurate line tracking.
        // Bound pos to input length to prevent out-of-bounds when pos
        // has been advanced past the end of input.
        let end = std::cmp::min(self.pos, self.input.len());
        let start = end.saturating_sub(n);
        let consumed = &self.input[start..end];
        for &b in consumed {
            if b == b'\n' {
                self.location.line += 1;
                self.location.column = 1;
            } else {
                self.location.column += 1;
            }
        }
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a byte can start a macro name.
///
/// GNU m4: a name starts with an ASCII letter or underscore.
/// Digits are NOT valid as the first character.
/// This matches `isalpha()` in the C locale.
fn is_name_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

/// Check if a byte can continue a macro name.
///
/// GNU m4: name characters are ASCII letters, digits, and underscore.
/// This matches `isalnum()` + `_` in the C locale.
fn is_name_continuation(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_plain_text() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"hello world\n");
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Name);
        assert_eq!(tokens[0].text, b"hello");
        assert_eq!(tokens[1].kind, TokenKind::Text);
        assert_eq!(tokens[1].text, b" ");
        assert_eq!(tokens[2].kind, TokenKind::Name);
        assert_eq!(tokens[2].text, b"world");
        assert_eq!(tokens[3].kind, TokenKind::Text);
        assert_eq!(tokens[3].text, b"\n");
    }

    #[test]
    fn test_name_recognition() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"define");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Name);
        assert_eq!(tokens[0].text, b"define");
    }

    #[test]
    fn test_mixed_text_and_name() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"abc define xyz");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].text, b"abc");
        assert!(tokens[1].is_text());
        assert_eq!(tokens[2].text, b"define");
        assert!(tokens[3].is_text());
        assert_eq!(tokens[4].text, b"xyz");
    }

    #[test]
    fn test_punctuation_tokens() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"foo(bar, baz)");
        assert_eq!(tokens.len(), 7);
        assert_eq!(tokens[0].text, b"foo");
        assert!(tokens[1].is_paren_open());
        assert_eq!(tokens[2].text, b"bar");
        assert!(tokens[3].is_comma());
        assert!(tokens[4].is_text());
        assert_eq!(tokens[5].text, b"baz");
        assert!(tokens[6].is_paren_close());
    }

    #[test]
    fn test_default_quotes() {
        let mut lexer = Lexer::new();
        // `` `hello' '' — backtick opens, apostrophe closes
        let tokens = lexer.tokenize(b"`hello'");
        // Should produce one Text token "hello" (quotes stripped)
        assert_eq!(tokens.len(), 1, "expected 1 token, got: {:?}", tokens);
        assert_eq!(tokens[0].kind, TokenKind::Text);
        assert_eq!(tokens[0].text, b"hello");
    }

    #[test]
    fn test_nested_quotes() {
        let mut lexer = Lexer::new();
        // Input: \x60\x60\x27\x60\x27 (outer quote never closed)
        // With unclosed quote fix: text inside unclosed outer quote is suppressed.
        // GNU m4 outputs nothing for unclosed quotes (error only).
        // Our lexer also suppresses: merged text is empty.
        let tokens = lexer.tokenize(b"``'`'");
        let combined: Vec<u8> = tokens.iter().flat_map(|t| t.text.clone()).collect();
        assert!(
            combined.is_empty(),
            "unclosed outer quote should suppress output, got: {:?}",
            String::from_utf8_lossy(&combined)
        );
    }

    #[test]
    fn test_paren_after_quote() {
        let mut lexer = Lexer::new();
        // Verify that ) after a quoted string is recognized as ParenClose
        let tokens = lexer.tokenize(b"eval(`***')");
        // DEBUG:
        // eprintln!("LEXER TEST: {} tokens", tokens.len());
        // for (i, t) in tokens.iter().enumerate() {
        //     eprintln!("  [{}]: {:?} = {:?}", i, t.kind, String::from_utf8_lossy(&t.text));
        // }
        assert!(
            tokens.len() >= 4,
            "expected at least 4 tokens, got {}",
            tokens.len()
        );
        // tokens[3] should be ParenClose since ) follows the quoted text
        assert!(
            tokens[3].is_paren_close(),
            "token[3] should be ParenClose, got {:?}",
            tokens[3].kind
        );
    }

    #[test]
    fn test_name_not_starting_with_digit() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"123abc");
        assert!(tokens.len() >= 2);
        assert!(tokens[0].is_text());
        assert_eq!(tokens[0].text, b"123");
        assert!(tokens[1].is_name());
        assert_eq!(tokens[1].text, b"abc");
    }

    #[test]
    fn test_multi_byte_delimiter_quotes() {
        let mut lexer = Lexer::new();
        lexer.quote_config.change_quote(Some("[["), Some("]]"));
        let tokens = lexer.tokenize(b"[[hello]]");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Text);
        assert_eq!(tokens[0].text, b"hello");
    }

    #[test]
    fn test_comment_consumed() {
        let mut lexer = Lexer::new();
        // # comment\n — consumed entirely, no blank line
        let tokens = lexer.tokenize(b"before# comment\nafter");
        // Should produce: "before" (text), "after" (text)
        // The comment and its newline are discarded
        let text: Vec<u8> = tokens.iter().flat_map(|t| t.text.clone()).collect();
        assert_eq!(
            text,
            b"beforeafter",
            "got: {:?}",
            String::from_utf8_lossy(&text)
        );
    }
}
