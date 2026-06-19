# Parity Ladder

*Generated: 1781835457 | Source: `sources/docs/parity-ladder.json`*

Tracks which surfaces `m4-rs` claims to match against GNU m4 1.4.21. Generated from `reports/claim-ladder.json`.

## Legend

| Status | Meaning |
|---|---|
| 🟢 sealed | Oracle-admitted with byte-comparison receipts |
| 🟡 partial | Implementation exists but divergences remain |
| 🔴 unclaimed | Not yet implemented or compared |
| ⛔ permanent | Will never be claimed |

## Phase 0: Oracle

🟢 M4.ORACLE.1 — GNU m4 1.4.21 admitted (65/75 pass)

## Phase 1–4: Core

🟢 M4.CLI.1, M4.BYTE.1, M4.LEX.1, M4.QUOTE.1, M4.COMMENT.1, M4.DEFINE.1, M4.EXPAND.1, M4.ARGS.1, M4.PUSHDEF.1 — all sealed

## Phase 5–7: Builtins

🟢 M4.BUILTIN.TEXT.1, M4.BUILTIN.EVAL.1, M4.BUILTIN.COND.1, M4.DIVERT.1, M4.INCLUDE.1 — all sealed

## Phase 8–12: Extended

🟡 M4.DIAG.1 (wording differs), 🟢 M4.TRACE.1, 🟡 M4.SYSCMD.1 (sandboxed), 🟢 M4.FROZEN.1, 🟢 M4.AUTOCONF.SEED.1, 🟢 M4.HOSTILE.1, 🟢 M4.FUZZ.1

## Permanent Non-Claims

⛔ POSIX signals (CROSS.9), ⛔ Stack overflow (CROSS.20), ⛔ i18n (CROSS.22), ⛔ changeword (CROSS.18)

