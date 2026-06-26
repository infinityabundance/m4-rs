# autoconf-rs: Complete Forensic-Parity Build Prompt

**Target:** GNU Autoconf (https://github.com/autotools-mirror/autoconf)
**Strategy:** Clean-room, black-box, oracle-court behavioral reconstruction
**Methodology:** Identical to m4-rs — oracle admission → receipt-backed claims → negative capabilities → xtask automation → JSON-first documents → freshness gates → fuzzing → Kani formal verification
**Precedent:** m4-rs (287/287 features, 182 tests, 19 sealed courts, 65/75 oracle compare, 2.0x perf, 1M fuzz 0 panics, 10 Kani proofs)

---

## DOCTRINE (identical to m4-rs)

1. **GNU Autoconf is the behavioral oracle.** Correctness means "matches the pinned GNU Autoconf oracle for the admitted surface," not "matches the manual."
2. **Claims are typed, bounded, and receipt-backed.** Every sealed court must have fixture input, exact oracle command, exact Rust command, stdout bytes, stderr bytes, exit status, environment, locale, SHA256 hashes, JSON receipt, human-readable rendered receipt, and negative-capability statement.
3. **No global parity claim until every axis has a sealed receipt.** "autoconf-rs supports Autoconf" is forbidden until broad receipts exist.
4. **Every unimplemented surface is a typed non-claim,** not hidden as "future work."
5. **The oracle is hostile.** Do not assume consistent output across versions. Do not assume documented edge cases. Sandbox, version, hash, and receipt everything.

---

## CLEAN-ROOM POSTURE

This is a **clean-room behavioral reconstruction.** GNU Autoconf is treated as a black-box oracle.

- **Do NOT consult GNU Autoconf source code** (C, M4, Perl, shell scripts in the `autoconf` repository).
- **Do NOT copy or translate implementation code.**
- **DO consult:** The GNU Autoconf manual (GFDL licensed), POSIX shell specification, black-box oracle interrogation (running `autoconf`, `autom4te`, `autoheader`, etc. and observing output), published papers and tutorials.
- **DO use the GNU m4 executable** as a subordinate oracle (since Autoconf depends on GNU m4). This is permitted because m4 is an external oracle, not Autoconf source.
- **Run `cargo xtask cleanroom`** (GPL contamination scanner) as part of every acceptance gate. Verify 0 errors.

**Licensing:** MIT OR Apache-2.0. No GPL entanglement.

---

## REPOSITORY SHAPE

```
autoconf-rs/
  Cargo.toml                     # workspace
  crates/
    autoconf-rs-core/            # M4 macro engine (depends on m4-rs-core or embeds it)
      src/
        lib.rs
        m4_engine.rs             # wraps m4-rs-core for Autoconf-specific M4 processing
        autom4te.rs              # autom4te replacement
        aclocal.rs               # aclocal replacement
        autoheader.rs            # autoheader replacement
        autoreconf.rs            # autoreconf orchestration
        configure_ac.rs          # configure.ac parser/analyzer
        shell_gen.rs             # configure script generator
        m4sugar.rs               # m4sugar macro library (built-in, not loaded from files)
        m4sh.rs                  # m4sh macro library
        template.rs              # template substitution (AC_SUBST, AC_CONFIG_FILES)
        site_file.rs             # config.site / site-lisp handling
        cache.rs                 # config.cache / config.status handling
        diagnostics.rs           # warning/error emission matching autoconf
        profile.rs               # oracle profile management
    autoconf-rs-cli/             # CLI binaries
      src/
        main_autoconf.rs         # autoconf binary
        main_autoheader.rs       # autoheader binary
        main_autom4te.rs         # autom4te binary
        main_autoreconf.rs       # autoreconf binary
        main_autoscan.rs         # autoscan binary
        main_autoupdate.rs       # autoupdate binary
        main_ifnames.rs          # ifnames binary
    autoconf-oracle-rs/           # Oracle admission crate
      src/lib.rs
    autoconf-casefile-rs/         # Receipt schema
      src/lib.rs
  xtask/                          # Maintenance tasks (pure Rust, no Python)
    src/
      main.rs                     # check, fmt, clippy, test, oracle, compare, generate, receipts, claims, cleanroom, fuzz, smoke, bench, status, ast-verify
      docgen/                     # Document generation engine
        mod.rs
        generate.rs               # generates all docs from JSON sources
        dsse.rs                   # DSSE signing
        sync_metrics.rs           # auto-detect surfaces from source
      bench.rs                    # performance baseline
      cleanroom.rs               # GPL contamination scanner
      compare.rs                  # corpus comparison
      fuzz.rs                     # deterministic fuzz harness
      gnu_compare.rs              # compare against GNU Autoconf test suite
      smoke.rs                    # synthetic smoke tests
      ast_verify.rs               # AST parity bridge (Doxygen/Clang oracle vs Rust AST)
  sources/                         # JSON sources of truth
    docs/                          # JSON sources for generated markdown docs
      status.json
      compatibility.json
      parity-ladder.json
      survival-ladder.json        # real-project configure.ac survival
      diagnostics.json
      oracle-profile.json
    gaps/
      master-gap-analysis.json
      needle-metrics.json
    negcaps/
      structured-negative-capabilities.json
    claims/                        # (optional, if claims not in claim-ladder)
  reports/                         # Generated documents (freshness-gated)
    receipts/
      cleanroom-receipt.json
      fuzz-1M-receipt.json
    FORENSIC-GAP-ANALYSIS.md
    NEEDLE-REPORT.md
    claim-ladder.json
    parity-matrix.json
    oracle-profile.json
    doc-registry.json              # Freshness registry (SHA256 of all sources/outputs)
  docs/                            # Generated documents
    negative-capabilities.md
    compatibility.md
    parity-ladder.md
    REVIEW-IN-10-MINUTES.md
  fixtures/                        # Test fixtures
    configure_ac/                  # .ac files
    m4_macros/                     # .m4 files
    expected_output/               # expected configure scripts
  lab/
    corpus/
      layer0-smoke/                # Tiny hand-written .ac files
      layer1-gnu-testsuite/        # GNU Autoconf's own test suite (~500 tests)
      layer2-manual-examples/      # Every example from the Autoconf manual
      layer3-posix/                # POSIX shell behavior tests
      layer4-real-packages/        # Real configure.ac from FOSS projects
      layer5-large-projects/       # Linux kernel, GCC, etc.
    oracle/
  kani/                            # Kani formal verification proofs
  fuzz/                            # cargo-fuzz targets
```

---

## CORE PARITY AXES

### A. Autoconf Binary Parity (autoconf)

**Oracle:** `autoconf [options] [template-file]`

Must handle:
- `configure.ac` / `configure.in` input
- `-o FILE` output file
- `-I DIR` include path
- `-B DIR` prepend include path
- `-W CATEGORY` warnings
- `-f` / `--force`
- `--trace=MACRO[:FORMAT]`
- `--debug`
- `--prepend-include=DIR`
- `--warnings=CATEGORY`
- `--initialization` / `-i`
- stdin handling
- exit status (0 success, 1 warnings, 2 errors)
- `AUTOCONF` environment variable
- `AUTOM4TE` environment variable
- `M4` environment variable
- `WARNINGS` environment variable

**Receipt family:** `AC.CLI.AUTOCONF.1`

### B. autom4te Parity

**Oracle:** `autom4te [options] [files]`

autom4te is the caching M4 wrapper that autoconf, autoheader, automake, and autoscan all use internally. It:
- Caches M4 expansions using frozen files
- Handles `--language=LANG` (Autoconf, Autotest, M4sh, M4sugar)
- Manages `--include=DIR` paths
- Supports `--freeze` / `--reload`
- Emits `--trace=MACRO[:FORMAT]`

**Receipt family:** `AC.CLI.AUTOM4TE.1`

### C. autoheader Parity

**Oracle:** `autoheader [options] [template-file]`

Generates `config.h.in` from `configure.ac` / `configure.in`.

**Receipt family:** `AC.CLI.AUTOHEADER.1`

### D. autoreconf Parity

**Oracle:** `autoreconf [options] [directory]`

Orchestrates autoconf, autoheader, automake, aclocal, libtoolize.

**Receipt family:** `AC.CLI.AUTORECONF.1`

### E. aclocal Parity

**Oracle:** `aclocal [options]`

Generates `aclocal.m4` from `configure.ac` by scanning `m4/` directories.

**Receipt family:** `AC.CLI.ACLOCAL.1`

### F. M4 Engine Parity

Autoconf is fundamentally an M4 macro processor with a massive standard library. The M4 engine must handle:
- All GNU m4 behaviors (already proven in m4-rs)
- Autoconf's specific M4 dialect: `AC_DEFUN`, `AC_REQUIRE`, `AC_PROVIDE`, `AC_BEFORE`, diversion management
- m4sugar: `m4_define`, `m4_defun`, `m4_require`, `m4_provide`, `m4_if`, `m4_case`, `m4_foreach`, `m4_map`, `m4_join`, `m4_expand`, `m4_do`, `m4_dquote`, `m4_quote`, `m4_normalize`, `m4_text_wrap`, etc.
- m4sh: `AS_ECHO`, `AS_ESCAPE`, `AS_EXIT`, `AS_IF`, `AS_CASE`, `AS_FOR`, `AS_MKDIR_P`, `AS_TR_SH`, `AS_TR_CPP`, etc.
- Autoconf proper: `AC_INIT`, `AC_OUTPUT`, `AC_CONFIG_FILES`, `AC_CONFIG_HEADERS`, `AC_CONFIG_COMMANDS`, `AC_CONFIG_LINKS`, `AC_CONFIG_SUBDIRS`, `AC_SUBST`, `AC_DEFINE`, `AC_CHECK_FUNC`, `AC_CHECK_HEADER`, `AC_CHECK_LIB`, `AC_CHECK_PROG`, `AC_TRY_COMPILE`, `AC_TRY_LINK`, `AC_TRY_RUN`, `AC_PROG_CC`, `AC_PROG_CXX`, `AC_PROG_MAKE_SET`, `AC_PROG_INSTALL`, etc.

**Receipt families:**
- `AC.M4.M4SUGAR.1`
- `AC.M4.M4SH.1`
- `AC.M4.AUTOCONF.1`

### G. Shell Script Generation Parity

The output of autoconf is a POSIX-compliant shell script. Parity means:
- The generated `configure` script produces identical behavior when executed
- `config.status` produces identical `Makefile`, `config.h`, etc.
- Shell variable quoting, escaping, and here-documents match
- `AC_CONFIG_FILES` substitution produces identical output

**Receipt families:**
- `AC.SHELL.CONFIGURE.1`
- `AC.SHELL.STATUS.1`

### H. Macro Library Parity

Autoconf ships with a large standard library:
- `autoconf/lib/autoconf/general.m4` — core macros
- `autoconf/lib/autoconf/status.m4` — config.status generation
- `autoconf/lib/autoconf/autoheader.m4` — config.h.in generation
- `autoconf/lib/autoconf/autoupdate.m4` — macro deprecation
- `autoconf/lib/autoconf/autoscan.m4` — autoscan support
- `autoconf/lib/autoconf/autotest.m4` — Autotest framework
- `autoconf/lib/autoconf/fortran.m4` — Fortran support
- `autoconf/lib/autoconf/erlang.m4` — Erlang support
- `autoconf/lib/autoconf/functions.m4` — function checks
- `autoconf/lib/autoconf/headers.m4` — header checks
- `autoconf/lib/autoconf/libs.m4` — library checks
- `autoconf/lib/autoconf/programs.m4` — program checks
- `autoconf/lib/autoconf/types.m4` — type checks
- `autoconf/lib/autoconf/c.m4` — C/C++ language support
- `autoconf/lib/autoconf/go.m4` — Go language support
- `autoconf/lib/m4sugar/m4sugar.m4` — m4sugar convenience macros
- `autoconf/lib/m4sugar/m4sh.m4` — m4sh shell generation

Each macro file must be admitted as a separate court with byte-identical expansion.

**Receipt family:** `AC.LIBRARY.<FILE>.1`

### I. Real-Package Survival

The ultimate pressure test: run autoconf-rs on real `configure.ac` files and compare the generated `configure` scripts.

**Tier 1 packages** (simple, well-known): GNU hello, grep, sed, make, tar, gzip, diffutils, findutils, gawk, bison, flex, readline, coreutils, wget, patch, texinfo

**Tier 2 packages** (medium complexity): libtool, automake, autoconf itself (self-host), gnulib, gettext, pkg-config, libpng, zlib, curl, openssl, sqlite

**Tier 3 packages** (large): Linux kernel, GCC, glibc, binutils, GDB, LLVM, CPython, Perl, Ruby, PHP, Apache, nginx, PostgreSQL, MariaDB

Start with Tier 1. Do not claim Tier 2 or 3 until Tier 1 has sealed receipts.

**Receipt families:**
- `AC.SURVIVAL.TIER1.1`
- `AC.SURVIVAL.TIER2.1`
- `AC.SURVIVAL.TIER3.1`

### J. Diagnostics Parity

Autoconf warnings: `AC_DIAGNOSE`, `AC_WARNING`, `AC_FATAL`, `AU_DEFUN` deprecation warnings, `AC_OBSOLETE`, syntax warnings, `-W` categories (cross, gnu, obsolete, override, portability, syntax, unsupported, all, error, no-CATEGORY).

**Receipt family:** `AC.DIAG.1`

---

## IMPLEMENTATION PHASES

### Phase 0: Oracle Admission

- Write `autoconf-oracle-rs`
- Locate `autoconf`, `autom4te`, `autoheader`, `autoreconf`, `aclocal`, `autoscan`, `autoupdate`, `ifnames` on PATH
- Capture versions, paths, SHA256 hashes, locale, shell, OS
- Run smoke fixtures for each binary
- Emit `reports/oracle-profile.json` with all binary profiles
- Refuse to run any court without an admitted oracle profile

**Seal:** `AC.ORACLE.1`

### Phase 1: CLI Harness

- Implement CLI wrappers for all 8 binaries
- Implement stdin/files/stdout/stderr/exit status capture
- No M4 processing yet — just copy-through
- Prove invocation model matches oracle

**Seal:** `AC.CLI.1`

### Phase 2: M4 Engine Integration

- Integrate or embed `m4-rs-core` as the M4 expansion engine
- Build the m4sugar library as built-in Rust macros (no external .m4 files)
- Build the m4sh library
- Prove that `m4_define`, `m4_if`, `m4_foreach`, `m4_defun`, `m4_require`, `m4_provide` expand identically to GNU m4 + GNU Autoconf's m4sugar.m4

**Seal:** `AC.M4.M4SUGAR.1`, `AC.M4.M4SH.1`

### Phase 3: Autoconf Macro Library

- Implement the core Autoconf macro set as built-in Rust behavior
- Each macro family gets its own court
- Start with: `AC_INIT`, `AC_OUTPUT`, `AC_CONFIG_FILES`, `AC_SUBST`, `AC_DEFINE`
- Expand to: `AC_PROG_CC`, `AC_PROG_CXX`, `AC_CHECK_FUNC`, `AC_CHECK_HEADER`, `AC_CHECK_LIB`

**Seal per macro family**

### Phase 4: Shell Script Generation

- Generate `configure` shell scripts from macro expansions
- Handle shell quoting, escaping, variable substitution
- Produce `config.status`, `Makefile.in` → `Makefile` substitution
- Produce `config.h.in` → `config.h` template processing

**Seal:** `AC.SHELL.1`

### Phase 5: Complete Binary Parity

- Each of the 8 binaries sealed against the oracle
- `autom4te` caching with frozen files
- `autoreconf` orchestration

**Seal per binary**

### Phase 6: Real-Package Survival

- Tier 1 packages (hello, grep, sed, make, tar, etc.)
- Compare generated `configure` scripts byte-for-byte
- Execute generated `configure` scripts in sandbox and compare output

**Seal:** `AC.SURVIVAL.TIER1.1`

### Phase 7: Tier 2 and Tier 3 Pressure

- Expand to larger packages
- Keep claims narrow: "passes this specific package" not "supports all Autoconf projects"

### Phase 8: Hostile Input and Fuzzing

- Panic-free malformed `configure.ac`
- Deep macro nesting
- Huge project sizes
- Binary bytes in macro arguments
- Recursive `AC_REQUIRE` cycles
- Resource limits

**Seal:** `AC.HOSTILE.1`, `AC.FUZZ.1`

---

## RECEIPT SCHEMA (identical to m4-rs)

```json
{
  "schema": "autoconf-rs-receipt-v1",
  "court": "AC.EXAMPLE.1",
  "verdict": "admitted_match",
  "oracle": {
    "kind": "gnu_autoconf",
    "version_output": "autoconf (GNU Autoconf) 2.72",
    "path": "/usr/bin/autoconf",
    "sha256": "...",
    "profile": "gnu_autoconf_2_72"
  },
  "rust": {
    "crate_version": "0.1.0",
    "git_commit": "...",
    "binary_sha256": "..."
  },
  "environment": {
    "os": "Linux x86_64",
    "arch": "x86_64",
    "locale": "C",
    "shell": "/bin/sh",
    "cwd_policy": "tempdir",
    "timezone": "UTC"
  },
  "fixture": {
    "name": "simple-ac-init",
    "input_sha256": "...",
    "files_sha256": "...",
    "argv": ["autoconf", "configure.ac"]
  },
  "comparison": {
    "stdout": "byte_exact",
    "stderr": "byte_exact | class_location_match | not_applicable",
    "exit_status": "exact",
    "filesystem_outputs": "exact | not_applicable"
  },
  "positive_claim": "...",
  "non_claims": ["..."],
  "known_divergences": [],
  "replay_command": "..."
}
```

---

## XTASK COMMANDS (all Rust, no Python)

```
cargo xtask check         — Run all acceptance gates
cargo xtask fmt           — rustfmt
cargo xtask clippy        — clippy -D warnings
cargo xtask test          — Run all tests
cargo xtask oracle        — Admit GNU Autoconf oracle
cargo xtask compare       — Run layered corpus comparison (layer0-layer5)
cargo xtask gnu-compare   — Compare against GNU Autoconf test suite
cargo xtask generate      — Regenerate all documents from JSON sources
cargo xtask receipts      — Verify receipt freshness
cargo xtask claims        — Verify claim ladder freshness
cargo xtask ast-verify    — AST parity verification bridge
cargo xtask behaviors     — Scan source for @ac_behavior witnesses
cargo xtask cleanroom     — GPL contamination scan
cargo xtask fuzz          — Run deterministic fuzz harness
cargo xtask smoke         — Run synthetic smoke test harness
cargo xtask bench         — Performance baseline vs GNU Autoconf
cargo xtask status        — Print current project status
```

---

## ACCEPTANCE GATES

1. **rustfmt** — all code formatted
2. **clippy** — `-D warnings`
3. **tests** — all unit/integration/smoke/oracle tests pass
4. **document freshness** — all generated docs match JSON source SHA256
5. **oracle profile** — valid oracle profile present
6. **claim ladder** — present and consistent with receipts
7. **clean-room scan** — 0 GPL contamination detected

---

## NEGATIVE CAPABILITIES (initial set)

- Not a drop-in GNU Autoconf replacement
- Not a replacement for automake, libtool, or gettext
- Not claimed for non-Linux platforms until tested
- Not claimed for cross-compilation scenarios
- Not claimed for Fortran/Erlang/Go language support until specifically admitted
- Not claimed for `config.guess`/`config.sub` replacement
- Not claimed for `autoscan` parity until specifically admitted
- Not claimed for `autoupdate` parity until specifically admitted
- Not claimed for `ifnames` parity until specifically admitted
- Not claimed for `autom4te` caching strategy parity (may use different cache format)
- Not claimed for m4 frozen file roundtrip with GNU Autoconf's frozen files (initially)
- Performance parity not claimed until after semantic parity
- Unicode correctness not claimed
- Security sandbox not claimed
- Diagnostic byte parity not claimed until `AC.DIAG.1`
- Build-system compatibility not claimed from simple macro expansion

---

## FIRST 12 COURTS (initial public sequence)

1. **AC.ORACLE.1** — Pinned GNU Autoconf oracle admission
2. **AC.CLI.1** — CLI harness (all 8 binaries)
3. **AC.BYTE.1** — Byte model
4. **AC.M4.M4SUGAR.1** — m4sugar macro library
5. **AC.M4.M4SH.1** — m4sh macro library
6. **AC.M4.AUTOCONF.CORE.1** — Core Autoconf macros (AC_INIT, AC_OUTPUT, AC_CONFIG_FILES, AC_SUBST, AC_DEFINE)
7. **AC.SHELL.CONFIGURE.1** — configure script generation
8. **AC.SHELL.STATUS.1** — config.status generation
9. **AC.LIBRARY.GENERAL.1** — general.m4 macro library
10. **AC.LIBRARY.PROGRAMS.1** — program detection macros
11. **AC.SURVIVAL.TIER1.1** — GNU hello survival
12. **AC.DIAG.1** — Diagnostics parity

---

## CORPUS STACK (ordered by pressure)

### Layer 0 — Tiny smoke fixtures (hand-written)
- `AC_INIT([hello], [1.0]) AC_OUTPUT` → compare configure output
- Single `AC_CHECK_FUNC` invocation
- Single `AC_CHECK_HEADER` invocation

### Layer 1 — GNU Autoconf's own test suite (~500 tests)
- From `autoconf/tests/` directory
- Port as Rust integration tests
- Compare generated output against oracle

### Layer 2 — GNU Autoconf manual examples
- Every example from the manual extracted as a runnable fixture
- Human-readable parity suite for reviewers

### Layer 3 — POSIX shell behavior tests
- Shell quoting edge cases
- Here-document handling
- Variable substitution corner cases
- Signal and exit code behavior

### Layer 4 — Real FOSS package configure.ac files
- GNU hello, grep, sed, make, tar, gzip, diffutils, findutils, gawk, bison, flex, readline, coreutils, wget, patch, texinfo
- Compare generated configure scripts byte-for-byte
- Execute generated configure scripts in sandbox

### Layer 5 — Large project stress tests
- Linux kernel, GCC, glibc
- Run in disposable container/tempdir only
- No network access

---

## JSON-FIRST DOCUMENTATION (all documents generated, freshness-gated)

**JSON sources** (hand-edited, source of truth):
- `sources/gaps/master-gap-analysis.json`
- `sources/gaps/needle-metrics.json`
- `sources/negcaps/structured-negative-capabilities.json`
- `sources/docs/status.json`
- `sources/docs/compatibility.json`
- `sources/docs/parity-ladder.json`
- `sources/docs/diagnostics.json`
- `sources/docs/oracle-profile.json`
- `sources/docs/survival-ladder.json`

**Generated documents** (never hand-edit):
- `reports/FORENSIC-GAP-ANALYSIS.md`
- `reports/NEEDLE-REPORT.md`
- `docs/negative-capabilities.md`
- `STATUS.md`
- `docs/compatibility.md`
- `docs/parity-ladder.md`
- `docs/diagnostics.md`
- `docs/oracle-profile.md`
- `docs/autoconf-survival.md`
- `docs/REVIEW-IN-10-MINUTES.md`
- All crate READMEs

**Freshness:** Every generated document has SHA256 hashes of its JSON sources recorded in `reports/doc-registry.json`. `cargo xtask check` fails if any source has changed.

---

## CODE COMMENTARY STANDARD (identical to m4-rs)

Every non-obvious code path must answer:
1. **What** behavior is being implemented? (reference Autoconf manual section)
2. **Why** must it be done this way? (link to source archaeology atlas)
3. **Which receipt** admits this behavior?
4. **What would break** if this were changed?

---

## FUZZING AND FORMAL VERIFICATION

- **Deterministic fuzz:** 1M iterations of randomly generated `configure.ac` inputs, compared against oracle. Seed fixed for reproducibility.
- **libfuzzer targets:** `configure_ac_parser`, `m4_expansion`, `shell_gen`
- **Kani formal proofs:** For safety-critical paths (shell escaping, variable substitution, AC_REQUIRE cycle detection)

---

## NEEDLE REPORT (gap percentage tracking)

Generated from `sources/gaps/needle-metrics.json`. Tracks:
- Per-surface feature count (implemented/partial/missing)
- Overall completion percentage
- Biggest movers (highest-impact unsealed surfaces)
- History entries for each major milestone

The needle percentage must be freshness-gated — `cargo xtask check` must fail if metrics are stale.

---

## KEY DIFFERENCES FROM m4-rs

1. **Multiple oracles:** Autoconf has 8 binaries (autoconf, autoheader, autom4te, autoreconf, aclocal, autoscan, autoupdate, ifnames) plus the subordinate GNU m4 oracle. The oracle profile must track all of them.
2. **Two-phase output:** Autoconf produces shell scripts. Those shell scripts produce Makefiles, config.h, etc. Parity requires matching both phases.
3. **Massive dependency chain:** autoconf → m4 → shell → make → compiler. The test sandbox must include all of these.
4. **Frozen files:** autom4te's caching uses GNU m4 frozen files. Must be cross-compatible.
5. **Macro library size:** ~20 M4 files totaling thousands of macros. Each file is its own admission surface.
6. **Real-world corpus:** Tier 1 alone is 18 packages. Each package's configure.ac is a survival test.

---

## CRITICAL WARNINGS

- **Do NOT use Python** in the build pipeline. All tasks are Rust via xtask.
- **Do NOT claim "Autoconf support"** until broad Tier 1 survival receipts exist. "Passes this specific configure.ac" is the only allowed claim format early on.
- **Do NOT run generated configure scripts on the host.** Use tempdir + controlled PATH + no network.
- **Do NOT bundle surfaces.** Each binary, each macro file, each macro family gets its own sealed court.
- **Do NOT skip negative capabilities.** The build roadmap IS the negative capabilities document.
- **Keep the freshness gate honest.** If a document is stale, the gate must fail with an actionable message.

---

## INITIAL MILESTONE NAMES

- `AC.ORACLE.1` — pinned GNU Autoconf oracle admission (all 8 binaries + GNU m4)
- `AC.CLI.1` — CLI harness for all 8 binaries
- `AC.M4.M4SUGAR.1` — m4sugar macro library byte-identical expansion
- `AC.M4.M4SH.1` — m4sh macro library byte-identical expansion
- `AC.M4.AUTOCONF.CORE.1` — core Autoconf macros (AC_INIT through AC_OUTPUT)
- `AC.SHELL.CONFIGURE.1` — configure script generation byte-identical
- `AC.SHELL.STATUS.1` — config.status generation byte-identical
- `AC.LIBRARY.GENERAL.1` — general.m4 macro library admitted
- `AC.LIBRARY.PROGRAMS.1` — program detection macros admitted
- `AC.LIBRARY.FUNCTIONS.1` — function check macros admitted
- `AC.LIBRARY.HEADERS.1` — header check macros admitted
- `AC.LIBRARY.TYPES.1` — type check macros admitted
- `AC.SURVIVAL.TIER1.1` — Tier 1 package survival (18 packages)
- `AC.DIAG.1` — diagnostics taxonomy
- `AC.HOSTILE.1` — hostile input / no-panic court
- `AC.FUZZ.1` — fuzz receipt
- `AC.PERF.1` — performance baseline

---

*This prompt is a complete build strategy for autoconf-rs using the m4-rs forensic parity methodology. Give it to the agent as a single message to begin the project.*
