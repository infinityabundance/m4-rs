# m4-rs

**A native Rust forensic-parity implementation of GNU m4 behavior, built through oracle courts.**

`m4-rs` is a clean-room behavioral reconstruction of GNU m4. Each supported surface is admitted only after byte comparison against a pinned GNU m4 oracle. Unsupported surfaces are explicit non-claims.

**Author:** infinityabundance <255699974+infinityabundance@users.noreply.github.com>

New here? Start with [`docs/REVIEW-IN-10-MINUTES.md`](docs/REVIEW-IN-10-MINUTES.md).

## Status

- **287/287 features implemented, 0 partial, 0 missing**
- **182 tests passing** (90 unit, 74 gnu suite, 10 autoconf seed, 8 frozen smoke)
- **7/7 acceptance gates pass**
- **Performance: 2.0x vs GNU m4** (10k-defines: 3.6x)
- **GNU compare: 65/75 pass (87%) against GNU m4 1.4.21**
- **1M deterministic fuzz: 0 panics**
- **Kani formal verification: 10 proofs**

See [`STATUS.md`](STATUS.md) for live current-state authority.
See [`reports/claim-ladder.json`](reports/claim-ladder.json) for machine-readable claim status.
See [`docs/negative-capabilities.md`](docs/negative-capabilities.md) for explicit non-claims.

See [`reports/FORENSIC-GAP-ANALYSIS.md`](reports/FORENSIC-GAP-ANALYSIS.md) for the full C→Rust forensic gap audit.

## Quick Start

```sh
# Build
cargo build

# Admit the GNU m4 oracle
cargo xtask oracle

# Run acceptance gates
cargo xtask check

# Use m4-rs
echo 'define(`hello', `world')hello' | cargo run --bin m4-rs
```

## Parity Axes

- **M4.ORACLE.1** — Pinned GNU m4 oracle admission
- **M4.CLI.1** — Invocation/stdin/file/status harness
- **M4.LEX.1** — Byte lexer/name/comment/quote tokenization
- **M4.QUOTE.1** — Quote nesting and delimiter behavior
- **M4.DEFINE.1** — define/undefine expansion core
- **M4.ARGS.1** — Argument collection and `$n` behavior
- **M4.PUSHDEF.1** — Definition stack
- **M4.BUILTIN.TEXT.1** — len/index/substr/translit
- **M4.BUILTIN.EVAL.1** — eval/incr/decr
- **M4.DIVERT.1** — Diversions and automatic EOF undivert
- **M4.INCLUDE.1** — include/sinclude/search path
- **M4.DIAG.1** — Diagnostics taxonomy
- **M4.TRACE.1** — Debug/trace/dumpdef parity
- **M4.SYSCMD.1** — Shell command builtins in sandbox
- **M4.FROZEN.1** — Frozen file save/reload
- **M4.AUTOCONF.SEED.1** — Admitted Autoconf fixture survival
- **M4.HOSTILE.1** — Malformed input/no-panic court
- **M4.FUZZ.1** — Fuzz receipt

## License

MIT OR Apache-2.0, at your option.

This is a **clean-room behavioral reconstruction**, not a derivative work of GNU m4. No GPL code is included or copied.
