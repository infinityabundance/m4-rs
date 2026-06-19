// m4-rs diagnostics module — errors, warnings, and source location tracking.
//
// Placeholder. Diagnostics are a parity surface (M4.DIAG.1).
// Currently not claimed.

pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub location: Option<crate::token::SourceLocation>,
    pub macro_stack: Vec<String>,
}

pub enum DiagnosticLevel {
    Warning,
    Error,
    Fatal,
}

impl Diagnostic {
    pub fn warn(msg: &str) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: msg.to_string(),
            location: None,
            macro_stack: Vec::new(),
        }
    }

    pub fn error(msg: &str) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: msg.to_string(),
            location: None,
            macro_stack: Vec::new(),
        }
    }
}
