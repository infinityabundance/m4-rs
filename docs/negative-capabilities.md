# Negative Capabilities — Build Roadmap

**Generated:** 1781835457
**Source:** `sources/negcaps/structured-negative-capabilities.json`
**Purpose:** Knowing exactly what doesn't work is how we plan what to build next.

## PERMANENT: Permanent Non-Claims

_These will never be claimed. They are design boundaries, not gaps._

### NC.PERM.1

**Non-claim:** Not a GNU m4 replacement until all courts sealed

**Justification:** Replacement claim requires 100% surface parity. This is the terminal goal, not a deferral.

**Dependencies:** ALL_COURTS_SEALED

### NC.PERM.2

**Non-claim:** Not a security sandbox

**Justification:** m4 is a macro processor, not a sandbox. syscmd/esyscmd execute arbitrary commands by design. This matches GNU m4 behavior.

### NC.PERM.3

**Non-claim:** Unicode correctness not claimed

**Justification:** GNU m4 is byte-oriented (eight-bit-clean). m4-rs follows this design: core operates on bytes, not Unicode.

### NC.PERM.4

**Non-claim:** Performance parity not claimed

**Justification:** Byte-exact behavior is the target, not wall-clock performance. Performance may be better or worse.

### NC.PERM.5

**Non-claim:** Build-system compatibility not claimed

**Justification:** Different build systems (autotools vs cargo). Output behavior is the only comparandum.

## PROFILE: Profile-Bound Non-Claims

_True for some m4 variants, not portable claims._

### NC.PROF.1

**Non-claim:** Vendor m4 parity not claimed from GNU m4 parity

**Justification:** A receipt against gnu_m4_1_4_20_default does not imply busybox_m4, bsd_m4, or system_m4 parity. Each is a separate profile.

### NC.PROF.2

**Non-claim:** POSIX conformance not claimed

**Justification:** GNU m4 extends POSIX significantly. Posix-mode requires separate admission.

### NC.PROF.3

**Non-claim:** Not an Autoconf replacement

**Justification:** Autoconf is a framework built on m4, not just macro expansion. Autoconf survival is a separate parity ladder.

**Dependencies:** M4.AUTOCONF.SEED.1

## DEFERRED: Deferred Non-Claims

_Will be claimed when dependencies are sealed. Each has a specific dependency chain._

### NC.DEF.1

**Non-claim:** Frozen file parity not claimed

**Justification:** Depends on: macro table (M4.DEFINE.1), diversion system (M4.DIVERT.1), quote/comment config serialization, pushdef stack serialization, version mismatch handling. Cannot be sealed before Phase 10.

**Dependencies:** M4.DEFINE.1, M4.DIVERT.1, M4.QUOTE.1

**Target Phase:** 10

### NC.DEF.2

**Non-claim:** Shell command parity not claimed

**Justification:** Depends on: sandboxed test infrastructure, child process management, exit code capture, stderr passthrough semantics. syscmd/esyscmd/sysval all depend on each other.

**Dependencies:** M4.SYSCMD.1

**Target Phase:** 9

### NC.DEF.3

**Non-claim:** Diagnostic byte parity not claimed

**Justification:** Depends on: i18n decisions (15 language translations), oracle version stability, errprint/__file__/__line__/m4exit implementation, full diagnostic taxonomy. Byte-exact diagnostic matching is the most version-sensitive surface.

**Dependencies:** M4.DIAG.1

**Target Phase:** 8

### NC.DEF.4

**Non-claim:** Resource-exhaustion resistance not claimed

**Justification:** Will be tested in hostile/fuzz phases. Requires: recursion limit, input size limits, diversion buffer limits, panic-free error paths.

**Dependencies:** M4.HOSTILE.1, M4.FUZZ.1

**Target Phase:** 12

### NC.DEF.5

**Non-claim:** Autoconf survival not claimed

**Justification:** Multi-stage ladder: seed macros → macro files → autom4te invocation → configure generation → configure execution. Depends on virtually all builtins being sealed first.

**Dependencies:** ALL_BUILTINS_SEALED, M4.FROZEN.1, M4.DIVERT.1

**Target Phase:** 11

### NC.DEF.6

**Non-claim:** Trace/debug parity not claimed

**Justification:** Depends on: full expansion engine, line tracking, debug flags, dumpdef, output redirection.

**Dependencies:** M4.EXPAND.1, M4.INPUT.1

**Target Phase:** 8

## UNIMPLEMENTED: Unimplemented Surfaces

_Not yet started. These are the active implementation targets._

### NC.UNIMPL.1

**Non-claim:** eval/incr/decr arithmetic not implemented

**Justification:** 

**Complexity:** high

### NC.UNIMPL.2

**Non-claim:** format builtin not implemented

**Justification:** 

**Complexity:** medium

### NC.UNIMPL.3

**Non-claim:** Text builtins (len/index/substr/translit/regexp/patsubst) not implemented

**Justification:** 

**Complexity:** medium

### NC.UNIMPL.4

**Non-claim:** Diversion system not implemented

**Justification:** 

**Complexity:** high

### NC.UNIMPL.5

**Non-claim:** File inclusion (include/sinclude) not implemented

**Justification:** 

**Complexity:** medium

### NC.UNIMPL.6

**Non-claim:** Conditional builtins (ifdef/ifelse/shift) not implemented

**Justification:** 

**Complexity:** medium

### NC.UNIMPL.7

**Non-claim:** Indirect calls (builtin/indir) not implemented

**Justification:** 

**Complexity:** medium

### NC.UNIMPL.8

**Non-claim:** defn not implemented

**Justification:** 

**Complexity:** low

### NC.UNIMPL.9

**Non-claim:** Diagnostic builtins (errprint/__file__/__line__/m4exit/m4wrap) not implemented

**Justification:** 

**Complexity:** low

### NC.UNIMPL.10

**Non-claim:** Shell builtins (syscmd/esyscmd/sysval/maketemp/mkstemp) not implemented

**Justification:** 

**Complexity:** medium

### NC.UNIMPL.11

**Non-claim:** Syncline output (-s) not implemented

**Justification:** 

**Complexity:** low

### NC.UNIMPL.12

**Non-claim:** POSIX traditional mode (-G) not implemented

**Justification:** 

**Complexity:** high

## Critical Implementation Sequence

1. Fix lexer multi-byte delimiter scanning
2. Implement $0-$9, $#, $@, $* substitution
3. Implement proper rescanning
4. Implement quote stripping during expansion
5. Implement expansion during argument collection
6. Wire input stack, lexer, engine together
7. Implement diversion system (output.c parity)
8. Implement line number tracking
9. Implement eval arithmetic engine
10. Implement format builtin

