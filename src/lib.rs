//! `m4-rs` — forensic-parity GNU m4 reimplementation (workspace umbrella crate).
//!
//! Re-exports the workspace member crates: the macro engine [`m4_rs_core`], the GNU m4 oracle
//! admission crate [`m4_oracle_rs`], and receipt/casefile types [`m4_casefile_rs`]. The `m4-rs`
//! command-line binary lives in the `m4-rs-cli` member crate.
pub use m4_casefile_rs;
pub use m4_oracle_rs;
pub use m4_rs_core;
