# m4-rs Review in 10 Minutes

*Generated: 1781835457*

## What is this?

`m4-rs` is a native Rust implementation of GNU m4's macro-processing behavior. It reproduces GNU m4 output byte-for-byte for all admitted surfaces, proven through oracle comparison receipts.

## The strategy

**Oracle-first.** We don't guess what GNU m4 does. We run it, capture the output, and prove we match. Every claim is backed by a sealed receipt. Same forensic-parity methodology as `gnucobol-rs`, `zic-rs`, `chrony-rs`, `ncurses-native`.

## Current Status

| Metric | Value |
|--------|-------|
| Features | 149/149 implemented |
| Smoke tests | 134/134 pass |
| Acceptance gates | 7/7 pass |
| Oracle comparison | 65/75 pass (87%) vs GNU m4 1.4.21 |
| Performance | 2.0x overall vs GNU m4 |
| Fuzzing | 1M deterministic: 0 panics |
| Clean-room | 44 files, 0 GPL contamination |

## How to run

```sh
cargo build --release
cargo xtask oracle       # Admit the GNU m4 binary
cargo xtask check        # Run all 7 acceptance gates
cargo xtask bench        # Performance baseline
echo 'define(`hello', `world')hello' | cargo run --release --bin m4-rs
```

## The doctrine

1. GNU m4 is the behavioral oracle.
2. Correct means matches the pinned GNU m4 oracle.
3. Every admitted behavior must have a sealed receipt.
4. No global parity claim until every axis has a sealed receipt.
5. Every unimplemented surface is a typed non-claim.
