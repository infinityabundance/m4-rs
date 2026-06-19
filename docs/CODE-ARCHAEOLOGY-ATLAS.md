# M4 Code Archaeology Atlas — Esoteric Knowledge & Parity Differences

**Schema:** m4-rs-atlas-v1  
**Generated:** 2026-06-19  
**Status:** Archival reference for m4-rs forensic parity  

> This atlas documents less-common, esoteric, and historically-rooted m4
> behaviors, release archaeology, cross-implementation differences, and
> architectural decisions. It serves as the definitive reference for anyone
> full forensic parity with GNU m4 1.4.21.

---

## 1. M4 Release Archaeology

### 1.1 Lineage

| Era | Processor | Author(s) | Year | Notes |
|-----|-----------|-----------|------|-------|
| Precursor | GPM | C. Strachey | 1965 | 250 machine instructions, pure macro generator |
| Precursor | M6 | A.D. Hall (Bell Labs) | 1972 | 600 Fortran statements, first of m4 line |
| Precursor | m3 | D. Ritchie | ~1976 | For AP-3 minicomputer |
| Original | **m4** | B. Kernighan & D. Ritchie | 1977 | 21 builtin macros only |
| GNU | GNU m4 1.0 | R. Seindal | 1990 | Removed artificial limits |
| GNU | GNU m4 1.4 | F. Pinard | 1994 | 10-year stable release |
| GNU | GNU m4 1.4.1-1.4.2 | P. Eggert | 2004 | Long-standing bug fixes |
| GNU | GNU m4 1.4.3-1.4.4 | G.V. Vaughan | 2005 | Collected community patches |
| GNU | GNU m4 1.4.5-1.4.20 | E. Blake et al. | 2006-2025 | Active maintenance |
| Research | M5 | A. Dain Samples | 1992 | Further evolution, comp.compilers |
| Future | GNU m4 2.0 | — | Planned | Dynamic modules, `${11}` syntax |

### 1.2 Key Behavioral Changes Across Versions

| Version | Change | Impact |
|---------|--------|--------|
| 1.4.6 | Added `__program__` builtin | Diagnostic parity |
| 1.4.8 | Added `mkstemp` (secure temp files) | Security |
| 1.4.8 | Fixed `__file__`/`__line__` in `m4wrap` | Diagnostic parity |
| 1.4.9 | Fixed eval precedence to match C | Breaking: `!0*2` was 0, now 2 |
| 1.4.9 | `-E` supports multiple invocations (behavior levels) | New fatal-warning semantics |
| 1.4.11 | Secure `maketemp` quotes result (security fix) | Breaking: result now quoted |
| 1.4.20 | Current stable | Oracle target |
| 2.0 (planned) | `${11}` for >9 args, `$11` deprecated | Breaking for multi-digit |
| 2.0 (planned) | `POSIXLY_CORRECT` enables `--traditional` | Breaking for GNU-by-default |
| 2.0 (planned) | Dynamic module loading | Extension mechanism |

### 1.3 POSIX vs GNU Differences

| Feature | POSIX (IEEE 1003.1) | GNU m4 1.4.20 |
|---------|---------------------|---------------|
| Max diversions | 9 | Unlimited (>2^28) |
| Max args | 9 (`$1`-`$9`) | Unlimited (`$10`, `$11`, ...) |
| `$10` meaning | `$1` + "0" | Tenth argument (deprecated in 2.0) |
| `define` semantics | Unspecified (stack replace or top replace) | Replaces top of stack only |
| `m4wrap` order | FIFO | LIFO (changing to FIFO in future) |
| `syscmd` output | Rescanned for macros (POSIX 2001 bug) | Not rescanned (corrected in next POSIX) |
| `translit` ranges | Unspecified | Supported (`a-z`, `0-9`) |
| `regexp` | Not in POSIX | GNU extension (Emacs-style regex) |
| `patsubst` | Not in POSIX | GNU extension |
| `format` | Not in POSIX | GNU extension (C printf) |
| `esyscmd` | Not in POSIX | GNU extension |
| `builtin`/`indir` | Not in POSIX | GNU extensions |
| `debugmode`/`debugfile` | Not in POSIX | GNU extensions |
| `__file__`/`__line__`/`__program__` | Not in POSIX | GNU extensions |
| `$@` behavior | Required | Implemented |
| `$*` behavior | Required | Implemented |
| `maketemp` | OB (obsolescent) — PID-based | Secure (mkstemp-like) |

---

## 2. Esoteric M4 Surfaces

### 2.1 Rarely-Known CLI Options

| Option | Description | Our Status |
|--------|-------------|------------|
| `--warn-macro-sequence[=REGEXP]` | Warn about `$11`/`${1}` in definitions | 🔴 NOT IMPLEMENTED |
| `--word-regexp=REGEXP` / `-W` | Alternative macro name syntax | 🔴 NOT IMPLEMENTED (requires changeword) |
| `--prepend-include=DIR` / `-B` | Search dirs BEFORE cwd (reverse order) | 🔴 NOT IMPLEMENTED |
| `--hashsize=N` / `-H` | Symbol table hash size (default 65537) | 🔴 NOT IMPLEMENTED |
| `--diversions=N` / `-N` | Deprecated, does nothing, warns | 🔴 NOT IMPLEMENTED |
| `-S`, `-T` | System V compat, do nothing, warn | 🔴 NOT IMPLEMENTED |
| `-e` | Deprecated alias for `-i` (interactive), warns | 🟡 Recognized but no deprecation warning |
| `-o`, `--error-output` | Deprecated aliases for `--debugfile` | 🔴 NOT IMPLEMENTED |
| `--debugfile` without `=FILE` | Discards debug output (empty string = discard) | 🟡 `debugfile` builtin handles this |

### 2.2 Rarely-Known Builtins & Behaviors

| Surface | Details | Our Status |
|---------|---------|------------|
| `__program__` | Expands to `argv[0]` of m4 invocation (1.4.6+) | ✅ IMPLEMENTED (via `env!("CARGO_PKG_NAME")`) |
| `__gnu__`, `__unix__`, `__windows__`, `__os2__` | Platform detection macros (empty expansion) | 🟡 `__gnu__`/`__unix__` implemented; `__windows__`/`__os2__` not |
| `unix`, `windows`, `os2` | Platform macros without `__` prefix (-G mode) | 🟡 `unix` implemented; others not |
| `dnl` with args | `dnl(foo, bar)` — collects args, side effects occur, output discarded | 🟡 PARTIAL (dnl is blind; arg side effects not collected) |
| `= ` in eval | Treated as `==` with deprecation warning | 🟡 PARTIAL (eval.rs uses `==`, no `=` alias) |
| `0b` binary prefix in eval | `0b1010` = 10 (GNU extension) | 🔴 NOT IMPLEMENTED |
| `0r` radix prefix in eval | `0r2:1010` = 10, `0r1:111` = 3 (GNU extension) | 🔴 NOT IMPLEMENTED |
| Emacs-style regex | `\(...\)` grouping, `\|` alternation, NOT `(...)` or `|` | 🔴 MISMATCH (Rust regex crate uses PCRE) |
| `sysval` signal encoding | Signal number << 8 on UNIX (e.g., SIGKILL=9 → 2304) | 🔴 NOT IMPLEMENTED |
| `divert` memory spill | 512K limit before temp file (`TMPDIR` or `/tmp`) | 🔴 NOT IMPLEMENTED |
| `undivert` of diversion 0 | Does nothing (already flushed to output) | ✅ IMPLEMENTED |
| `undivert` of current diversion | Silently ignored | 🟡 PARTIAL (diversion.rs checks this) |
| `changequote` with same start/end | Disables quote nesting | ✅ IMPLEMENTED |
| `changequote` with multi-byte delimiters | No limit on length (BSD caps at 5) | ✅ IMPLEMENTED |
| `frozen file V1` format | GNU m4 manual says version is `1`, not `2` | ✅ IMPLEMENTED (V1 format, cross-compatible with GNU m4 1.4.21) |
| Frozen `F` directive | Builtin renaming in frozen files | 🟡 PARTIAL — `F` entries are saved and loaded for builtin recognition |
| `defn` of multiple builtins | Warning issued, builtin omitted | ✅ IMPLEMENTED |
| `defn` token concat in frozen | Special tokens for builtins | 🔴 NOT IMPLEMENTED |
| `$#` in comments | `#` is comment, so `define(foo, $#)` defines as `$` | ✅ IMPLEMENTED |
| `translit` reverse ranges | `9-0` → `9876543210` | 🟡 PARTIAL (basic ranges work) |
| `patsubst` with empty regex | Matches before each character, produces `\-a\-b\-c\-` | ✅ IMPLEMENTED |

### 2.3 Cross-Implementation Behavioral Differences

| Behavior | GNU m4 | System V | BSD | POSIX |
|----------|--------|----------|-----|-------|
| `changequote(,) `→ close quote | Uses `'` (default) | Reuses start quote | Preserves previous | Unspecified |
| `changecom(,) `→ end comment | Uses newline | Unspecified | Preserves previous | Unspecified |
| `define` replaces | Top of stack | Entire stack | Entire stack | Unspecified |
| Comment precedence | Comments > macros > quotes | Macros > comments | Macros > comments | Unspecified |
| `traceon()` | Only currently-defined macros | Global (also future defs) | Global | Unspecified |
| Trace persistence after undefine | Persists | Lost | Lost | Not specified |
| File boundary in arg collection | Error | Allowed (spans files) | Error | Not specified |
| `syncline` in diversions | At divert time | At undivert time | N/A | Not specified |

### 2.4 M4 2.0 Migration Warnings (Future-Proofing)

| Change | m4-rs Status |
|--------|-------------|
| `${11}` replaces `$11` | 🟡 We support `$11` (GNU 1.4.x behavior). Will need `${11}` for 2.0 |
| `POSIXLY_CORRECT` enables `--traditional` | 🔴 Not yet honored |
| `changeword` removed | ✅ We don't implement changeword |
| Dynamic module loading | 🔴 Not implemented |

---

## 3. Environment Variables

| Variable | Purpose | Our Status |
|----------|---------|------------|
| `M4PATH` | Colon-separated include search path (after `-I` dirs) | 🔴 NOT IMPLEMENTED |
| `TMPDIR` | Directory for diversion temp files | 🔴 NOT IMPLEMENTED (all in-memory) |
| `POSIXLY_CORRECT` | Should enable `--traditional` (2.0+) | 🔴 NOT IMPLEMENTED |
| `LC_ALL` | Locale override (we set to `C`) | ✅ IMPLEMENTED |
| `LC_CTYPE` | Character classification locale | 🟡 PINNED TO C |
| `LC_MESSAGES` | Diagnostic message locale | 🟡 PINNED TO C |

### 3.1 Obsolete / Removed CLI Options

| Option | Description | Status |
|--------|-------------|--------|
| `--warn-macro-sequence[=REGEXP]` | Warn about `$11`/`${1}` in definitions | 🔴 NOT IMPLEMENTED |
| `--word-regexp=REGEXP` / `-W` | Alternative macro name syntax | 🔴 NOT IMPLEMENTED (requires changeword) |
| `--prepend-include=DIR` / `-B` | Search dirs BEFORE cwd (reverse order) | 🔴 NOT IMPLEMENTED |
| `--hashsize=N` / `-H` | Symbol table hash size (default 65537) | 🔴 NOT IMPLEMENTED |
| `--diversions=N` / `-N` | Deprecated, does nothing, warns | 🔴 NOT IMPLEMENTED |
| `-S`, `-T` | System V compat, do nothing, warn | 🔴 NOT IMPLEMENTED |
| `-e` | Deprecated alias for `-i` (interactive), warns | 🟡 Recognized; no deprecation warning |
| `-o`, `--error-output` | Deprecated aliases for `--debugfile` | 🔴 NOT IMPLEMENTED |

---

## 4. Clean-Room Verification

### 4.1 No GPL Contamination Audit

**Methodology:** This implementation treats GNU m4 as a black-box behavioral oracle.
No GNU m4 source code was consulted. All behavior was derived from the GNU m4 manual
(GFDL licensed), POSIX specification, black-box oracle testing, and historical papers.

**Verification:** `cargo xtask cleanroom` scans all 44 source files. 0 warnings, 0 errors.
Receipt saved to `reports/receipts/cleanroom-receipt.json`.

**Key verification points:**

| Concern | Status |
|---------|--------|
| C source code (`src/*.c`) consulted? | ❌ NO — clean-room |
| Test suite (`checks/`, `tests/`) copied? | ❌ NO — original Rust tests written |
| GPL headers (`m4.h`) used as reference? | ❌ NO — distributed Rust types |
| gnulib code (`lib/`) referenced? | ❌ NO — Rust std replaces |
| Manual examples used as test fixtures? | ✅ YES — manual is GFDL, examples are behavioral specs |
| Frozen file format from manual? | ✅ YES — behavioral specification |
| `format` specifiers from manual? | ✅ YES — behavioral specification |

**Resolved concerns:** Frozen file format verified and fixed as V1 (was V2). Cross-compatible with GNU m4 1.4.21 frozen output. 8 oracle smoke tests pass.

**Remaining concerns:**

| Concern | Status | Resolution |
|---------|--------|------------|
| Emacs regex vs PCRE (CROSS.24) | 🔍 monitored | Basic patterns match; edge cases (`\(`, `\|`, `\{`) differ |
| eval overflow beyond 32-bit | 🔍 monitored | i32 range matches GNU m4; overflow behavior differs for >32-bit |

### 4.2 Frozen File Format — RESOLVED

The GNU m4 manual §15.2 states the version directive is `V number NL` where
"m4 1.4.20 only creates and understands frozen files where number is 1."

**Resolution:** `m4-rs` now uses V1 format, two-line length-prefixed encoding, matching
GNU m4 1.4.21 exactly. Cross-compatible both ways: m4-rs can load GNU m4 frozen files
and GNU m4 can load m4-rs frozen files. Verified by 8 oracle smoke tests.

---

## 5. Recommended Parity Improvements

### Priority 1 (Behavioral)
1. ~~Fix frozen file version~~ — **DONE** (V1 format, cross-compatible with GNU m4 1.4.21)
2. ~~Add `__gnu__`, `__unix__` platform macros~~ — **DONE**
3. Add `0b`/`0r` prefix support in eval — GNU extensions
4. Add `=` alias for `==` in eval — with deprecation warning
5. Add `M4PATH` environment variable support

### Priority 2 (Esoteric)
6. **Emacs-style regex wrapper** — for `regexp`/`patsubst` parity
7. **`dnl` with args** — collect but discard
8. **`sysval` signal encoding** — `signal << 8`
9. **Frozen `F` directive** — builtin renaming in frozen files
10. **`--warn-macro-sequence`, `--prepend-include`, `--hashsize`** CLI options

### Priority 3 (Documentation)
1. ~~Document all permanent non-claims in negative-capabilities~~ — **DONE**
2. Cross-reference CROSS gaps to specific GNU m4 manual sections
3. ~~Create oracle admission receipt set~~ — **DONE** (GNU m4 1.4.21 admitted, 65/75 compare)

---

## 6. Known Gaps Not Yet in Surface Tracking

| ID | Surface | Impact |
|----|---------|--------|
| CROSS.24 | Emacs regex vs PCRE regex in regexp/patsubst | Edge cases with `\(`, `\|`, `\{` |
| CROSS.25 | eval `0b`/`0r` literal prefixes | GNU extension not implemented |
| CROSS.26 | Frozen file V1 vs V2 format | Version mismatch with oracle |
| CROSS.27 | `dnl(foo)` arg collection side effects | dnl treated as blind, no arg processing |
| CROSS.28 | Platform macros (`__gnu__`, etc.) | Not registered as builtins |
| CROSS.29 | `M4PATH` environment variable | Not implemented |
| CROSS.30 | Frozen `F` directive (builtin renaming) | Not implemented |
| CROSS.31 | `--warn-macro-sequence` CLI option | Not implemented |
| CROSS.32 | `--prepend-include` / `-B` CLI option | Not implemented |
| CROSS.33 | eval `=` as `==` alias | Not implemented |
| CROSS.34 | `sysval` signal encoding (signal<<8) | Not implemented |
| CROSS.35 | `divert` memory spill to temp files | Not implemented (all in-memory) |
| CROSS.36 | `translit` reverse ranges (`9-0`) | Not implemented |
