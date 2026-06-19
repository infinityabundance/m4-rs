// m4-rs comment handling.
//
// GNU m4 comments are controlled by `changecom` and interact with quoting
// in subtle ways. Comments are NOT passed through to output by default —
// they are discarded. However, when comment delimiters are disabled via
// `changecom` with no arguments, `#` is treated as plain text.
//
// Key behaviors:
//
// 1. **Default comments**: `#` starts a comment, newline ends it.
//    Everything from `#` to end-of-line (including the newline) is discarded.
//    This means `#` comments consume the newline, so no blank line appears.
//
// 2. **Comments inside quotes**: Text that looks like a comment but appears
//    inside a quoted string is NOT treated as a comment. This is because
//    quote processing happens before comment processing.
//
// 3. **`changecom`**: Changes comment delimiters.
//    `changecom(/*, */)` sets C-style comments.
//    `changecom` with no arguments disables comments (everything passes
//    through, including `#` characters).
//
// 4. **Disabled comments**: When `changecom` disables comments, `#` is
//    treated as plain text and appears in output.
//
// 5. **Comment delimiters as text**: When comments are disabled, the
//    comment delimiters themselves become plain text (they are not
//    "special" anymore).
//
// 6. **Comments in definitions**: A comment inside a macro definition
//    is consumed at definition time (unless quoted). This means the
//    comment is NOT part of the macro body.
//
// 7. **`dnl`**: The `dnl` builtin is a special kind of comment — it
//    discards everything from the `dnl` token to the next newline,
//    including the newline. Unlike `#` comments, `dnl` is a macro
//    (it can be renamed, undefined, etc.). `dnl` is often used to
//    suppress unwanted blank lines in macro definitions.
//
// Reference: GNU M4 manual, Sections 3.3, 6.3 (changecom), 6.10 (dnl)

use crate::quote::QuoteConfig;

/// Consume a comment from input, given the current comment configuration.
///
/// Returns the number of bytes consumed (including the comment delimiters
/// themselves), and the comment text (without delimiters).
///
/// This is used by the lexer to skip comment regions without emitting
/// any output tokens.
///
/// Note: Comments are processed AFTER quote recognition. If we're inside
/// a quoted string, comment delimiters are NOT recognized.
pub fn consume_comment(input: &[u8], config: &QuoteConfig) -> Option<(usize, Vec<u8>)> {
    if !config.comments_enabled {
        return None;
    }

    let open_bytes = config.comment_open.as_bytes();
    let close_bytes = config.comment_close.as_bytes();

    // Check if input starts with comment open delimiter
    if !input.starts_with(open_bytes) {
        return None;
    }

    let start = open_bytes.len();
    let mut pos = start;

    // Search for the close delimiter
    while pos + close_bytes.len() <= input.len() {
        if &input[pos..pos + close_bytes.len()] == close_bytes {
            let comment_text = input[start..pos].to_vec();
            pos += close_bytes.len();
            return Some((pos, comment_text));
        }
        pos += 1;
    }

    // Comment extends to end of input (no close delimiter found)
    let comment_text = input[start..].to_vec();
    Some((input.len(), comment_text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_comment_basic() {
        let config = QuoteConfig::default();
        let input = b"# this is a comment\nafter comment";
        let (consumed, text) = consume_comment(input, &config).unwrap();
        assert_eq!(consumed, b"# this is a comment\n".len());
        assert_eq!(text, b" this is a comment");
    }

    #[test]
    fn test_no_comment_when_disabled() {
        let config = QuoteConfig {
            comments_enabled: false,
            ..Default::default()
        };
        let input = b"# this should not be a comment";
        assert!(consume_comment(input, &config).is_none());
    }

    #[test]
    fn test_comment_to_eof() {
        let config = QuoteConfig::default();
        let input = b"# no newline at end";
        let (consumed, text) = consume_comment(input, &config).unwrap();
        assert_eq!(consumed, input.len());
        assert_eq!(text, b" no newline at end");
    }
}
