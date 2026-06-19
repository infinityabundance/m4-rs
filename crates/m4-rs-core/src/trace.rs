// m4-rs trace module — debug tracing and dumpdef.
//
// Placeholder. Tracing is a parity surface (M4.TRACE.1).
// Currently not claimed.

/// Debug flags for tracing.
pub struct DebugFlags {
    pub flags: Vec<String>,
}

impl DebugFlags {
    pub fn from_string(s: &str) -> Self {
        Self {
            flags: s.chars().map(|c| c.to_string()).collect(),
        }
    }
}

/// Trace output for a macro call.
pub fn trace_macro_call(_name: &str, _args: &[Vec<u8>], _expansion: &[u8]) -> String {
    String::new()
}
