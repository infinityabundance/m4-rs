// m4-rs profile module — oracle profile tracking.
//
// Placeholder. Profile separation is a parity surface (M4.ORACLE.PROFILE.1).
// Currently not claimed.

/// Runtime profile of the current m4-rs build.
pub struct RuntimeProfile {
    pub crate_version: String,
    pub platform: String,
    pub builtins_enabled: Vec<String>,
    pub features_enabled: Vec<String>,
}

impl RuntimeProfile {
    pub fn current() -> Self {
        Self {
            crate_version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
            builtins_enabled: vec![],
            features_enabled: vec![],
        }
    }
}
