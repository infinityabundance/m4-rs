# Diagnostics Parity

*Generated: 1781835457 | Source: `sources/docs/diagnostics.json`*

Diagnostics are a parity surface. `m4-rs` implements all diagnostic builtins and targets class/location matching.

## Implemented

| Feature | Status |
|---|---|
| errprint(message) | ✅ Working |
| __file__ | ✅ Working |
| __line__ | ✅ Working |
| m4exit(code) | ✅ Working |
| Unclosed quote error | ✅ Class/location match |
| Excess arguments warning | ✅ Class/location match |

## Comparison Strategy

1. Class + source location match. 2. Stable substring. 3. Byte-exact where stable.

M4.DIAG.1 is `known_failure`. Wording differs from GNU m4 (CROSS.7 — errno vs io::Error, no gettext). Permanent non-claim for byte-exact diagnostic parity.

