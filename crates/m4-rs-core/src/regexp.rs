// m4-rs regexp module — GNU m4 regexp and patsubst builtins.
//
// Placeholder. Regexp operations are a parity surface.
// Currently not claimed.

/// Match a regexp against a string.
///
/// GNU m4 uses a regex engine similar to Emacs regexps.
/// `regexp(string, regexp, [replacement])`
pub fn regexp_match(_string: &[u8], _regex: &[u8], _replacement: Option<&[u8]>) -> Option<Vec<u8>> {
    None
}

/// Perform regexp substitution.
///
/// `patsubst(string, regexp, [replacement])`
pub fn patsubst(_string: &[u8], _regex: &[u8], _replacement: Option<&[u8]>) -> Vec<u8> {
    Vec::new()
}
