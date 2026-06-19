# Process Effects

*Generated: 1781835457 | Source: `sources/docs/process-effects.json`*

## Shell Execution

| Builtin | Behavior | Status |
|---|---|---|
| syscmd(cmd) | Executes via /bin/sh -c, output to stdout | ✅ Working |
| esyscmd(cmd) | Captures output as string | ✅ Working |
| sysval | Returns last exit status | ✅ Working |

## Known Divergences

| Divergence | Status |
|---|---|
| fd inheritance (CROSS.12) | 🔍 monitored |
| sysval signal encoding (CROSS.34) | 🔴 not implemented |
| Shell path | 🔍 monitored |

M4.SYSCMD.1 is monitored. Sandboxed in tests. Full oracle parity not yet claimed.

