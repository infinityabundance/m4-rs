//! `m4-rs` — forensic-parity GNU m4 reimplementation. The workspace crate: it ships the `m4-rs`
//! CLI binary and re-exports the member crates so `cargo install m4-rs` gives the tool and
//! `cargo add m4-rs` gives the engine.
pub use m4_casefile_rs;
pub use m4_oracle_rs;
pub use m4_rs_cli;
pub use m4_rs_core;
