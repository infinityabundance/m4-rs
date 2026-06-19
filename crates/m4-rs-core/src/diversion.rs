// m4-rs diversion module — output diversion buffers.
//
// Placeholder. Diversions are a first-class parity surface (M4.DIVERT.1).
// Currently not claimed.

/// Diversion buffers for the m4 processor.
pub struct Diversions {
    pub buffers: std::collections::BTreeMap<i32, Vec<u8>>,
    pub current: i32,
}

impl Diversions {
    pub fn new() -> Self {
        let mut buffers = std::collections::BTreeMap::new();
        buffers.insert(0, Vec::new()); // Default diversion
        Self {
            buffers,
            current: 0,
        }
    }
}

impl Default for Diversions {
    fn default() -> Self {
        Self::new()
    }
}
