# FORENSIC GAP ANALYSIS — GNU m4 → m4-rs

**Generated:** 1781835457
**Source:** `sources/gaps/master-gap-analysis.json`
**DSSE:** verified by xtask

---

## Summary

| Metric | Count |
|--------|-------|
| Source files mapped | 13 |
| Total features tracked | 287 |
| Implemented | 287 |
| Partial | 0 |
| Missing | 0 |
| Cross-cutting C→Rust gaps | 25 |

## Critical Gaps (Implementation Priority)

1. **CROSS.9**: SIGINT/SIGPIPE signal handlers — permanent non-claim in safe Rust (broken pipe handled via write error)
2. **CROSS.20**: Stack overflow detection — Rust aborts on overflow vs C sigaltstack recovery (port: stack.rs with probe_stack)
3. **CROSS.17**: Configure-time constants — autotools vs Cargo feature flags (permanent non-claim: Cargo.toml features)
4. **CROSS.22**: i18n translations (15+ languages) — permanent non-claim (LC_MESSAGES=C pinned)
5. **CROSS.18**: changeword support — permanent non-claim (requires --enable-changeword, rarely enabled)
6. **CROSS.19**: GNU m4 oracle comparison: 65/75 pass (87%). 10 known divergences remain (diagnostics wording, platform-specific, NUL handling)
7. **CROSS.11**: temp file creation (mkstemp) — sandbox path differs from C; mkstemp builtin uses tempdir for oracle tests
8. **CROSS.12**: fd inheritance in child processes — Rust Command closes fds by default; syscmd/esyscmd sandboxed

## Source File Map

| GNU m4 File | Size | m4-rs Module | Implemented | Partial | Missing |
|------------|------|-------------|-------------|---------|--------|
| `src/m4.c` | 22.5KB | crates/m4-rs-cli/src/main.rs | ✅ 37 | 0 | 0 |
| `src/m4.h` | 16.2KB | distributed across all modules | ✅ 0 | 0 | 0 |
| `src/builtin.c` | 69.4KB | crates/m4-rs-core/src/builtin.rs + expansion.rs | ✅ 42 | 0 | 0 |
| `src/debug.c` | 12.1KB | crates/m4-rs-core/src/trace.rs | ✅ 12 | 0 | 0 |
| `src/eval.c` | 13.5KB | crates/m4-rs-core/src/eval.rs | ✅ 16 | 0 | 0 |
| `src/format.c` | 10.9KB | crates/m4-rs-core/src/format.rs | ✅ 13 | 0 | 0 |
| `src/freeze.c` | 13.1KB | crates/m4-rs-core/src/frozen.rs | ✅ 10 | 0 | 0 |
| `src/input.c` | 36.3KB | crates/m4-rs-core/src/input.rs + lexer.rs | ✅ 20 | 0 | 0 |
| `src/macro.c` | 13.2KB | crates/m4-rs-core/src/expansion.rs + args.rs | ✅ 19 | 0 | 0 |
| `src/output.c` | 33.8KB | crates/m4-rs-core/src/diversion.rs | ✅ 20 | 0 | 0 |
| `src/path.c` | 4.7KB | crates/m4-rs-core/src/include_.rs | ✅ 8 | 0 | 0 |
| `src/symtab.c` | 14.2KB | crates/m4-rs-core/src/macro_table.rs | ✅ 18 | 0 | 0 |
| `src/stackovf.c` | 4.0KB | crates/m4-rs-core/src/stack.rs | ✅ 4 | 0 | 0 |

## Non-Source Directories

| Directory | Content | Status |
|-----------|---------|--------|
| `lib/` | gnulib compatibility | N/A (Rust std replaces) |
| `checks/` | test suite (~150 tests) | ported — 74 gnu_test_suite tests + 8 frozen_smoke tests + 10 autoconf_seed tests (182 total: 90 unit + 8 frozen + 10 autoconf + 74 gnu) |
| `doc/` | Texinfo manual | ported — docs/compatibility.md, docs/negative-capabilities.md, docs/parity-ladder.md, docs/CODE-ARCHAEOLOGY-ATLAS.md |
| `examples/` | example macro files | ported — 10 autoconf seed fixtures + oracle comparison tests in crates/m4-rs-core/tests/autoconf_seed.rs |
| `m4/` | gnulib-cache.m4 | N/A (no gnulib) |
| `po/` | translations (15+ languages) | permanent non-claim (LC_MESSAGES=C pinned; diagnostics in English only) |
| `tests/` | additional tests | ported — 74 gnu_test_suite + 8 frozen_smoke + 10 autoconf_seed = 92 integration tests |
| `build-aux/` | autotools helpers | N/A (cargo) |
| `gl-mod/` | gnulib module config | N/A (no gnulib) |

## C→Rust Cross-Cutting Gaps

### build config

- **CROSS.16**: autotools vs cargo _(impact: Feature detection differs — permanent non-claim (Cargo.toml features))_ — ⛔ permanent non-claim
- **CROSS.17**: configure-time constants _(impact: Must map to Cargo features — permanent non-claim)_ — ⛔ permanent non-claim
- **CROSS.18**: --enable-changeword compile flag _(impact: Permanent non-claim (requires --enable-changeword, rarely enabled in GNU m4))_ — ⛔ permanent non-claim

### control flow

- **CROSS.6**: setjmp/longjmp vs Result/panic _(impact: Error recovery paths differ — m4exit unwinds via Result<usize::MAX>. Verified.)_ — ✅ resolved
- **CROSS.7**: errno vs io::Error _(impact: Error message formatting differs — known divergence in diagnostic wording)_ — ⚠️ known divergence
- **CROSS.8**: exit() from deep stack vs unwind _(impact: m4exit unwinds via usize::MAX sentinel — implemented and tested)_ — ✅ resolved
- **CROSS.9**: SIGINT/SIGPIPE handlers _(impact: Permanent non-claim in safe Rust (broken pipe handled via write error))_ — ⛔ permanent non-claim

### i18n

- **CROSS.22**: gettext translations (15+ languages) _(impact: Permanent non-claim (LC_MESSAGES=C pinned; diagnostics in English))_ — ⛔ permanent non-claim
- **CROSS.23**: locale-aware diagnostic formatting _(impact: LC_MESSAGES=C pinned for parity — no behavioral gap)_ — ✅ resolved

### io filesystem

- **CROSS.10**: fopen rb vs File::open _(impact: Binary mode is default in Rust — no behavioral gap)_ — ✅ resolved
- **CROSS.11**: temp file creation (mkstemp vs tempdir) _(impact: Sandbox path differs from C; mkstemp builtin uses tempdir for oracle tests)_ — 🔍 monitored
- **CROSS.12**: fd inheritance in child processes _(impact: Rust Command closes fds by default; syscmd/esyscmd sandboxed in tests)_ — 🔍 monitored

### memory model

- **CROSS.1**: obstack vs Vec<u8> _(impact: Different growth characteristics for large expansions — no behavioral gap)_ — ✅ resolved
- **CROSS.2**: null-terminated strings vs length-prefixed _(impact: NUL byte handling verified byte-exact in M4.BYTE.1 smoke tests)_ — ✅ resolved
- **CROSS.3**: FILE* buffering vs Read/Write traits _(impact: Output flushing verified byte-exact in all smoke tests)_ — ✅ resolved
- **CROSS.4**: alloca() vs heap _(impact: No behavioral difference — all stack-allocated equivalent data is heap-backed)_ — ✅ resolved
- **CROSS.5**: platform-dependent int sizes vs fixed-size _(impact: eval uses 32-bit i32 matching GNU m4; arithmetic courts pass byte-exact)_ — ✅ resolved

### string byte

- **CROSS.13**: locale-dependent isalpha/isdigit _(impact: LC_CTYPE=C pinned — no locale-dependent behavior. Verified.)_ — ✅ resolved
- **CROSS.14**: UTF-8 passthrough vs byte orientation _(impact: All core processing uses &[u8], not String. Byte-exact output verified.)_ — ✅ resolved
- **CROSS.15**: wide character support _(impact: GNU m4 is byte-oriented — no wide char gap exists)_ — N/A

### testing

- **CROSS.19**: GNU m4 tests oracle-compared: 65/75 pass (87%) _(impact: 74 gnu_test_suite + 8 frozen_smoke + 10 autoconf_seed tests. 10 known divergences documented (diagnostics wording, platform-specific, NUL).)_
- **CROSS.20**: stack overflow test _(impact: Permanent non-claim — Rust aborts on stack overflow vs C sigaltstack recovery (stack.rs with probe_stack for detection))_ — ⛔ permanent non-claim
- **CROSS.21**: cargo-fuzz targets (3) require nightly; 1M deterministic fuzz with 0 panics _(impact: 1M iter deterministic fuzz: 0 panics, 0 crashes. 3 libfuzzer targets scaffolded. Kani formal verification (10 proofs).)_ — ✅ resolved
- **CROSS.37**: Empty-quote token boundary inside nested quotes _(impact: FIXED: 0x01 token-boundary marker inserted at nested empty-quote positions; lexer splits on marker during re-lexing)_ — ✅ FIXED
- **CROSS.38**: Recursive forloop $1 resolution _(impact: FIXED: recursion_depth increment in ifelse/ifdef branch expansion. M4.QUOTE.DEEP.1 fix adds same-pass changequote re-lex.)_ — ✅ FIXED

