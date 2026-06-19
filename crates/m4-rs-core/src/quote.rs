// m4-rs quote state management.
//
// GNU m4's quoting system is one of its most powerful and subtle features.
// Understanding quote behavior is essential for forensic parity.
//
// Key behaviors:
//
// 1. **Default delimiters**: `` `' '' (ASCII backtick 0x60 and apostrophe 0x27).
//    These were chosen in the 1970s to be unusual characters that rarely
//    appear in normal text, minimizing conflicts.
//
// 2. **Nesting**: Quotes nest. `` ```'`' '' produces `'` as output.
//    Each `` `' '' opens a level; each `'' closes a level.
//    The outer quotes are stripped; inner ones remain in output.
//
// 3. **`changequote`**: Changes quote delimiters at runtime.
//    `changequote([, ])` sets quotes to `[` and `]`.
//    `changequote` with no arguments disables quoting entirely.
//    `changequote(`,')` is tricky — the comma separates arguments,
//    so you need to quote it: `changequote(``,'')` doesn't work either.
//    Use: `changequote(`[', `]')` → sets open to `[`, close to `]`.
//
// 4. **Quote stripping**: One level of quotes is always stripped during
//    expansion. Quoted text inside a macro definition is protected from
//    expansion at definition time, and the quotes are stripped at
//    expansion time.
//
// 5. **Empty quotes**: `'` produces empty output (one layer stripped).
//
// 6. **Quotes in macro arguments**: Commas and parentheses inside quotes
//    do NOT separate arguments or nest calls. This is how macros pass
//    complex arguments.
//
// Reference: GNU M4 manual, Sections 3.2–3.3, 6.1 (changequote)

/// Configuration for quote and comment delimiters.
///
/// This is mutable state that the lexer and expansion engine consult.
/// When `changequote` or `changecom` macros are expanded, they modify
/// this configuration.
#[derive(Debug, Clone)]
pub struct QuoteConfig {
    /// The opening quote delimiter string (default: "`").
    pub open: String,
    /// The closing quote delimiter string (default: "'").
    pub close: String,
    /// The opening comment delimiter string (default: "#").
    pub comment_open: String,
    /// The closing comment delimiter string (default: "\n").
    pub comment_close: String,
    /// Whether quoting is currently active.
    /// `changequote` with no args disables quoting.
    pub quoting_enabled: bool,
    /// Whether comments are currently active.
    /// `changecom` with no args disables comments.
    pub comments_enabled: bool,
}

impl Default for QuoteConfig {
    fn default() -> Self {
        Self {
            open: "`".to_string(),
            close: "'".to_string(),
            comment_open: "#".to_string(),
            comment_close: "\n".to_string(),
            quoting_enabled: true,
            comments_enabled: true,
        }
    }
}

impl QuoteConfig {
    /// Change the quote delimiters.
    /// Called by the `changequote` builtin.
    ///
    /// Behavior per GNU m4:
    /// - `changequote` with no args: disable quoting
    /// - `changequote(open)` with one arg: set open delimiter, close unchanged
    /// - `changequote(open, close)`: set both
    /// - Empty string for open or close is valid (but unusual)
    pub fn change_quote(&mut self, open: Option<&str>, close: Option<&str>) {
        if open.is_none() && close.is_none() {
            self.quoting_enabled = false;
            return;
        }
        self.quoting_enabled = true;
        if let Some(o) = open {
            self.open = o.to_string();
        }
        if let Some(c) = close {
            self.close = c.to_string();
        }
    }

    /// Change the comment delimiters.
    /// Called by the `changecom` builtin.
    ///
    /// Behavior per GNU m4:
    /// - `changecom` with no args: disable comments (everything passes through)
    /// - `changecom(start)`: set start delimiter, end unchanged
    /// - `changecom(start, end)`: set both
    pub fn change_comment(&mut self, start: Option<&str>, end: Option<&str>) {
        if start.is_none() && end.is_none() {
            self.comments_enabled = false;
            return;
        }
        self.comments_enabled = true;
        if let Some(s) = start {
            self.comment_open = s.to_string();
        }
        if let Some(e) = end {
            self.comment_close = e.to_string();
        }
    }
}

/// The quote state tracker for the expansion engine.
///
/// During expansion, we need to track whether we're inside quotes
/// because:
/// - Commas inside quotes don't separate arguments
/// - Parentheses inside quotes don't nest calls
/// - Quoted text in definitions is expanded once (not twice)
#[derive(Debug, Clone)]
pub struct QuoteState {
    /// Current nesting depth.
    pub depth: usize,
    /// Reference to the current quote delimiters.
    pub config: QuoteConfig,
}

impl QuoteState {
    pub fn new(config: QuoteConfig) -> Self {
        Self { depth: 0, config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QuoteConfig::default();
        assert_eq!(config.open, "`");
        assert_eq!(config.close, "'");
        assert!(config.quoting_enabled);
    }

    #[test]
    fn test_change_quote() {
        let mut config = QuoteConfig::default();
        config.change_quote(Some("["), Some("]"));
        assert_eq!(config.open, "[");
        assert_eq!(config.close, "]");
        assert!(config.quoting_enabled);
    }

    #[test]
    fn test_disable_quoting() {
        let mut config = QuoteConfig::default();
        config.change_quote(None, None);
        assert!(!config.quoting_enabled);
        // Open/close remain at their old values
        assert_eq!(config.open, "`");
    }

    #[test]
    fn test_disable_comments() {
        let mut config = QuoteConfig::default();
        config.change_comment(None, None);
        assert!(!config.comments_enabled);
    }
}
