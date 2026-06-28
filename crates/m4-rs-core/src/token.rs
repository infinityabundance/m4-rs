// m4-rs token definitions.
//
// m4 tokens are not the same as programming-language tokens.
// The m4 tokenizer produces three kinds of output:
//   1. Plain text (bytes copied directly to output)
//   2. Macro names (potential macro invocations)
//   3. Quote/comment delimiters (control the tokenizer itself)
//
// Reference: GNU M4 manual, Section 3 "Lexical and syntactic conventions"
// https://www.gnu.org/software/m4/manual/m4.html#Syntax
//
// Important: m4 is byte-oriented, not character-oriented. See Section 1.1
// "History of m4" for the eight-bit-clean guarantee (except NUL).
//
// Key behavioral notes:
//   - A name is the longest sequence of letters, digits, and `_` that does
//     not start with a digit.
//   - Names are case-sensitive in GNU m4.
//   - Everything that is not a name, quote delimiter, comment delimiter,
//     or macro argument separator is plain text and passed through to output.
//   - The default quote delimiters are `` `' '' (backtick and apostrophe).
//   - The default comment delimiters are `#' and newline.

use std::fmt;

/// The three fundamental token kinds in m4's lexical model.
///
/// Note: Unlike many tokenizers, m4 does not have whitespace tokens.
/// Whitespace is part of the plain text that surrounds other tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    /// Plain text to be copied directly to output.
    /// This includes whitespace, punctuation, and anything not recognized
    /// as a name or delimiter.
    Text,

    /// A potential macro name.
    /// Recognition as a macro happens during expansion, not during lexing.
    /// A Name token may turn out to be plain text if no macro is defined.
    Name,

    /// An opening quote delimiter. Default: `` `' ''
    /// Everything between QuoteOpen and QuoteClose is quoted text,
    /// protected from immediate macro expansion.
    QuoteOpen,

    /// A closing quote delimiter. Default: `''
    QuoteClose,

    /// An opening parenthesis `(` used for macro argument collection.
    /// Must appear immediately after a macro name (no intervening space)
    /// to trigger argument collection.
    ParenOpen,

    /// A closing parenthesis `)` that ends argument collection.
    ParenClose,

    /// A comma `,` used to separate macro arguments.
    /// Only meaningful inside a macro argument list.
    Comma,
}

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    /// The raw bytes of this token.
    pub text: Vec<u8>,
    /// Source location for diagnostics.
    pub location: SourceLocation,
    /// True if this Text token is the interior of a quoted region (`[...]`). Argument collection uses
    /// this to strip leading whitespace BEFORE a quote without touching a quoted body that begins
    /// with whitespace (e.g. a Perl one-liner) — m4's "strip leading unquoted whitespace" rule.
    pub from_quote: bool,
}

/// Source location tracking for diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceLocation {
    /// File name or "<stdin>"
    pub file: String,
    /// 1-based line number
    pub line: usize,
    /// 1-based column (byte offset from start of line)
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, text: Vec<u8>, location: SourceLocation) -> Self {
        Self {
            kind,
            text,
            location,
            from_quote: false,
        }
    }

    /// Create a new token at a given location.
    pub fn text(bytes: &[u8], location: SourceLocation) -> Self {
        Self {
            kind: TokenKind::Text,
            text: bytes.to_vec(),
            location,
            from_quote: false,
        }
    }

    /// Create a name token.
    pub fn name(bytes: &[u8], location: SourceLocation) -> Self {
        Self {
            kind: TokenKind::Name,
            text: bytes.to_vec(),
            location,
            from_quote: false,
        }
    }

    /// Returns the text as a &str if it's valid UTF-8, otherwise the raw bytes as a debug string.
    pub fn text_str(&self) -> String {
        String::from_utf8_lossy(&self.text).to_string()
    }

    /// True if this token is plain text.
    pub fn is_text(&self) -> bool {
        self.kind == TokenKind::Text
    }

    /// True if this token is a macro name.
    pub fn is_name(&self) -> bool {
        self.kind == TokenKind::Name
    }

    /// True if this token is an opening quote delimiter.
    pub fn is_quote_open(&self) -> bool {
        self.kind == TokenKind::QuoteOpen
    }

    /// True if this token is a closing quote delimiter.
    pub fn is_quote_close(&self) -> bool {
        self.kind == TokenKind::QuoteClose
    }

    /// True if this token is a `(`.
    pub fn is_paren_open(&self) -> bool {
        self.kind == TokenKind::ParenOpen
    }

    /// True if this token is a `)`.
    pub fn is_paren_close(&self) -> bool {
        self.kind == TokenKind::ParenClose
    }

    /// True if this token is a comma.
    pub fn is_comma(&self) -> bool {
        self.kind == TokenKind::Comma
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Text => write!(f, "Text"),
            TokenKind::Name => write!(f, "Name"),
            TokenKind::QuoteOpen => write!(f, "QuoteOpen"),
            TokenKind::QuoteClose => write!(f, "QuoteClose"),
            TokenKind::ParenOpen => write!(f, "ParenOpen"),
            TokenKind::ParenClose => write!(f, "ParenClose"),
            TokenKind::Comma => write!(f, "Comma"),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}<{}>", self.kind, String::from_utf8_lossy(&self.text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_basics() {
        let t = Token::text(b"hello", SourceLocation::default());
        assert!(t.is_text());
        assert!(!t.is_name());
        assert_eq!(t.text_str(), "hello");
    }

    #[test]
    fn test_name_token() {
        let t = Token::name(b"define", SourceLocation::default());
        assert!(t.is_name());
        assert!(!t.is_text());
    }
}
