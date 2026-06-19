# Needle Report

**Generated:** 1781835457

## Overall: 100.0% complete

- 149 impl / 0 partial / 0 missing / 149 total

## Per-Surface

| Surface | Done | Part | Miss | % |
|---------|------|------|------|----|
| M4.CLI.1 | 16 | 0 | 0 | 100.0% |
| M4.BYTE.1 | 5 | 0 | 0 | 100.0% |
| M4.LEX.1 | 16 | 0 | 0 | 100.0% |
| M4.EXPAND.1 | 20 | 0 | 0 | 100.0% |
| M4.BUILTINS.1 | 46 | 0 | 0 | 100.0% |
| M4.TABLE.1 | 10 | 0 | 0 | 100.0% |
| M4.DIVERT.1 | 8 | 0 | 0 | 100.0% |
| M4.DIAG.1 | 4 | 0 | 0 | 100.0% |
| M4.HOSTILE.1 | 9 | 0 | 0 | 100.0% |
| M4.AUTOCONF.SEED.1 | 10 | 0 | 0 | 100.0% |
| M4.FROZEN.1 | 2 | 0 | 0 | 100.0% |
| M4.FUZZ.1 | 3 | 0 | 0 | 100.0% |

## Biggest Movers

1. **M4.QUOTE.DEEP.1** (w:0) — DONE: Same-pass changequote now takes effect immediately via token re-lex. After changequote modifies quote config, remaining tokens are re-lexed with new delimiters (matching GNU m4 streaming behavior). All 6 changequote/changecom tests pass. changecom is excluded from re-lex (matching GNU m4 — comments are consumed during lexing, not expansion).
2. **M4.DIAG.EXIT.1** (w:0) — FIXED: Unclosed quote output suppression now matches GNU m4. Lexer suppresses text flush when quote_depth > 0 at EOF. All 4 DIAG smoke tests pass.
3. **M4.ORACLE.LAYER1** (w:0) — DONE: cargo xtask gnu-compare — 65/75 pass (87%) byte-identical against GNU m4 1.4.21. 10 known divergences documented.
4. **M4.FROZEN.1** (w:0) — DONE: 8/8 oracle smoke tests pass. Frozen file -F/-R now cross-compatible with GNU m4 1.4.21. Two-line format matched, diversion save/reload, pushdef stack, quote config roundtrip, version mismatch exit 63.
5. **CROSS.18** (w:0) — RESOLVED: changeword — permanent non-claim. Requires --enable-changeword in GNU m4 build (rarely enabled). Not planned for m4-rs.
6. **M4.PERF.1** (w:0) — DONE: Performance optimized — 2.1x overall vs GNU m4 (was 2.8x). 10k-defines 3.9x (was 6.0x). LTO + fat codegen + pure-text body fast-path + reusable Lexer + pre-allocated buffers + tokenize_owned. All 174 tests pass, 7/7 acceptance gates.
7. **CROSS.38** (w:0) — FIXED: Recursive forloop/self-ref/mutual-recursion via ifelse/ifdef branch expansion with recursion_depth increment + nested quote preservation in lexer.
