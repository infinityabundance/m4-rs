# Compatibility

*Generated: 1781835457 | Source: `sources/docs/compatibility.json`*

`m4-rs` aims for behavioral compatibility with GNU m4, not API or ABI compatibility.

## Oracle Profile

| Profile | Status |
|---|---|
| GNU m4 1.4.21 (Linux x86_64) | Admitted — 65/75 pass (87%) |
| GNU m4 1.4.21 POSIX mode | Not tested |
| BusyBox/BSD m4 | Not tested |

## Surface Coverage

| Surface | Status |
|---|---|
| 41+ builtins | oracle_admitted |
| 17 CLI flags | oracle_admitted |
| Frozen files | oracle_admitted — cross-compatible |
| Diversions | oracle_admitted |
| Diagnostics | known_failure — wording differs (CROSS.7) |
| Process builtins | monitored — sandboxed |
| Performance | 2.0x overall |
| Autoconf | 10 seed fixtures pass |

See `docs/negative-capabilities.md` for explicit non-claims.

