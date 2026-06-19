# Filesystem Effects

*Generated: 1781835457 | Source: `sources/docs/filesystem-effects.json`*

## File Operations

| Operation | Status |
|---|---|
| Read input file | ✅ oracle_admitted (M4.CLI.1) |
| include/sinclude | ✅ oracle_admitted (M4.INCLUDE.1) |
| undivert(filename) | ✅ oracle_admitted (M4.DIVERT.1) |
| maketemp/mkstemp | ✅ working; sandboxed (CROSS.11) |
| -F/-R frozen files | ✅ oracle_admitted (M4.FROZEN.1) |
| debugfile | ✅ oracle_admitted (M4.TRACE.1) |

All file operation surfaces implemented. Frozen files cross-compatible with GNU m4 1.4.21.

