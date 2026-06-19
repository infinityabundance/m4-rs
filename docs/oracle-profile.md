# Oracle Profile

*Generated: 1781835457 | Source: `sources/docs/oracle-profile.json`*

**GNU m4 1.4.21** — admitted via `cargo xtask oracle`. Profile: `reports/oracle-profile.json`.

## Admission

`cargo xtask oracle` locates the m4 binary, captures --version, computes SHA256, runs smoke fixtures, emits profile.

## Feature Detection

| Feature | Method |
|---|---|
| --prefix-builtins | --help mentions -P |
| --synclines | --help mentions -s |
| --freeze-state | --help mentions -F |
| --debug flags | Each flag tested individually |

Oracle compare: 65/75 pass (87%). 10 known divergences documented in FORENSIC-GAP-ANALYSIS.md.

