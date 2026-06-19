// m4-rs argument collection.
//
// When a macro is invoked with parentheses, m4 collects arguments
// separated by commas. Argument collection is one of the most subtle
// parts of GNU m4 behavior, because it interacts with quoting,
// nesting, and expansion.
//
// Key behaviors:
//
// 1. **Argument trigger**: A macro is invoked with arguments only if
//    a `(` immediately follows the macro name (no intervening space
//    or newline). Otherwise, the macro is called with no arguments.
//
// 2. **Argument separators**: Commas at the top level (not inside
//    quotes and not inside nested parentheses) separate arguments.
//
// 3. **Nested parentheses**: `()` pairs inside arguments are balanced.
//    `define(foo, (a(b, c), d))` has two arguments:
//    - Arg 1: `foo`
//    - Arg 2: `(a(b, c), d)` — the comma inside inner parens is NOT
//      an argument separator.
//
// 4. **Quoted commas/parens**: Commas and parentheses inside quotes
//    are NOT argument separators or nesting markers. They are part
//    of the argument text (after quote stripping).
//
// 5. **Whitespace in arguments**: Leading whitespace in each argument
//    is stripped (GNU m4 strips leading unquoted spaces/tabs).
//    Trailing whitespace is NOT stripped.
//
// 6. **Empty arguments**: Empty arguments (nothing between commas)
//    result in empty string arguments. `foo(,bar)` → arg1="", arg2="bar"
//    `foo(bar,)` → arg1="bar", arg2=""
//
// 7. **Missing arguments**: If fewer arguments are provided than the
//    macro references (e.g., `$3` when only 2 args given), the missing
//    argument is the empty string.
//
// 8. **Excess arguments**: If more arguments are provided than
//    referenced, excess arguments are silently ignored (but they ARE
//    still expanded during argument collection).
//
// 9. **`$0`**: Expands to the macro's own name.
// 10. **`$#`**: Expands to the number of arguments.
// 11. **`$@`**: Expands to all arguments, each individually quoted.
// 12. **`$*`**: Expands to all arguments, comma-separated, as a single string.
// 13. **`$n`**: Expands to the nth argument (1-based). `$10` is `$1` followed by `0`,
//      not argument 10. To access arguments beyond 9, use `shift`.
//
// Reference: GNU M4 manual, Sections 5.2–5.3

use crate::token::Token;
use crate::token::TokenKind;

/// Collected arguments for a macro invocation.
///
/// Each argument is stored as raw bytes (after quote stripping
/// but before rescansion).
#[derive(Debug, Clone)]
pub struct Args {
    /// The macro name (for `$0`).
    pub macro_name: Vec<u8>,
    /// Collected arguments, in order.
    pub args: Vec<Vec<u8>>,
}

impl Args {
    pub fn new(macro_name: &[u8]) -> Self {
        Self {
            macro_name: macro_name.to_vec(),
            args: Vec::new(),
        }
    }

    /// Get the number of arguments.
    pub fn len(&self) -> usize {
        self.args.len()
    }

    /// Returns true if there are no arguments.
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Get the nth argument (1-based), or empty bytes if out of range.
    pub fn get(&self, n: usize) -> &[u8] {
        if n == 0 {
            return &self.macro_name;
        }
        if n == 0 || n > self.args.len() {
            return b"";
        }
        &self.args[n - 1]
    }

    /// Get all arguments as a single comma-separated byte sequence.
    pub fn join_all(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                result.push(b',');
            }
            result.extend_from_slice(arg);
        }
        result
    }
}

/// Collect arguments from a token stream.
///
/// Given tokens starting just after the opening `(` of a macro call,
/// this function collects arguments up to the matching `)`.
///
/// The token stream should be at the position just after the `(`.
/// Returns the collected arguments and the position just after the `)`.
///
/// This is a non-expanding argument collector — it collects raw tokens
/// without expanding them. Expansion happens later during rescansion
/// of the macro body.
pub fn collect_args(tokens: &[Token], start_pos: usize) -> Option<(Args, usize)> {
    let mut args = Vec::new();
    let mut current_arg = Vec::new();
    let mut paren_depth: usize = 0;
    let mut i = start_pos;

    while i < tokens.len() {
        let token = &tokens[i];

        match token.kind {
            TokenKind::ParenOpen => {
                paren_depth += 1;
                current_arg.extend_from_slice(&token.text);
            }
            TokenKind::ParenClose => {
                if paren_depth == 0 {
                    // Closing paren of the argument list
                    // Strip leading whitespace from current arg
                    let trimmed = strip_leading_whitespace(&current_arg);
                    args.push(trimmed);
                    return Some((
                        Args {
                            macro_name: Vec::new(), // caller sets this
                            args,
                        },
                        i + 1, // position after )
                    ));
                } else {
                    paren_depth -= 1;
                    current_arg.extend_from_slice(&token.text);
                }
            }
            TokenKind::Comma if paren_depth == 0 => {
                // Argument separator at top level
                let trimmed = strip_leading_whitespace(&current_arg);
                args.push(trimmed);
                current_arg = Vec::new();
            }
            TokenKind::Comma => {
                current_arg.extend_from_slice(&token.text);
            }
            _ => {
                current_arg.extend_from_slice(&token.text);
            }
        }

        i += 1;
    }

    // No closing paren found — invalid input
    None
}

/// Strip leading whitespace from argument bytes.
///
/// GNU m4 strips unquoted leading spaces and tabs from each argument.
fn strip_leading_whitespace(bytes: &[u8]) -> Vec<u8> {
    let start = bytes
        .iter()
        .position(|&b| b != b' ' && b != b'\t')
        .unwrap_or(bytes.len());
    bytes[start..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_collect_simple_args() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"foo, bar, baz)");
        let (args, _pos) = collect_args(&tokens, 0).unwrap();

        assert_eq!(args.len(), 3);
        assert_eq!(args.get(1), b"foo");
        assert_eq!(args.get(2), b"bar");
        assert_eq!(args.get(3), b"baz");
    }

    #[test]
    fn test_collect_empty_args() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b",bar,)");
        let (args, _pos) = collect_args(&tokens, 0).unwrap();

        assert_eq!(args.len(), 3);
        assert_eq!(args.get(1), b"");
        assert_eq!(args.get(2), b"bar");
        assert_eq!(args.get(3), b"");
    }

    #[test]
    fn test_collect_nested_parens() {
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(b"foo, (a, b))");
        let (args, _pos) = collect_args(&tokens, 0).unwrap();

        assert_eq!(args.len(), 2);
        assert_eq!(args.get(1), b"foo");
        assert_eq!(args.get(2), b"(a, b)");
    }

    #[test]
    fn test_args_get() {
        let mut args = Args::new(b"test");
        args.args.push(b"first".to_vec());
        args.args.push(b"second".to_vec());

        assert_eq!(args.get(0), b"test"); // $0
        assert_eq!(args.get(1), b"first"); // $1
        assert_eq!(args.get(2), b"second"); // $2
        assert_eq!(args.get(3), b""); // $3 — out of range
        assert_eq!(args.get(99), b""); // far out of range
    }
}
