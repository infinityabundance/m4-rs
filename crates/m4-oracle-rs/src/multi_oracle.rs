// m4-oracle-rs: Multi-oracle support.
//
// WHO:   infinityabundance.
// WHAT:  Supports interrogating multiple m4 implementations:
//        - GNU m4 (primary oracle, behavioral authority)
//        - BusyBox m4 (secondary, POSIX-like)
//        - System m4 (whatever /usr/bin/m4 resolves to)
//        - BSD m4 (FreeBSD/OpenBSD variant, if available)
//        - Custom oracles at user-specified paths
// WHEN:  Used by xtask `ast-verify` and during multi-oracle cross-checks.
// WHERE: m4-oracle-rs/src/multi_oracle.rs
// WHY:   Multiple oracles provide:
//        1. Cross-verification — does behavior hold across implementations?
//        2. Profile separation — which behaviors are GNU-specific vs POSIX?
//        3. Oracle fallback — if the primary oracle is unavailable
//        4. Divergence detection — identify genuine GNU m4 extensions
// HOW:   OracleKind enum lists known variants. Each variant knows how to
//        locate itself on the system. MultiOracle manages a primary oracle
//        and a list of secondaries, providing interrogate_all() and
//        all_agree() methods for cross-oracle comparison.

use crate::{locate_m4, run_oracle_text, OracleConfig, OracleProfile};
use std::collections::HashMap;
use std::path::PathBuf;

/// Known m4 oracle kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OracleKind {
    /// GNU m4 (the primary oracle for m4-rs)
    GnuM4,
    /// BusyBox m4 (minimal POSIX-like implementation)
    BusyboxM4,
    /// System m4 (whatever /usr/bin/m4 resolves to)
    SystemM4,
    /// BSD m4 (FreeBSD/OpenBSD-style)
    BsdM4,
    /// Custom oracle at a specific path
    Custom(String),
}

impl OracleKind {
    /// Human-readable label for this oracle kind.
    pub fn label(&self) -> String {
        match self {
            OracleKind::GnuM4 => "gnu_m4".to_string(),
            OracleKind::BusyboxM4 => "busybox_m4".to_string(),
            OracleKind::SystemM4 => "system_m4".to_string(),
            OracleKind::BsdM4 => "bsd_m4".to_string(),
            OracleKind::Custom(s) => format!("custom_{}", s),
        }
    }

    /// Build an OracleConfig that can locate this oracle on the system.
    fn config(&self) -> Result<OracleConfig, crate::OracleError> {
        match self {
            OracleKind::GnuM4 => Ok(OracleConfig {
                m4_path: None,
                ..Default::default()
            }),
            OracleKind::BusyboxM4 => {
                // BusyBox m4 is typically invoked as `busybox m4`.
                // We search for the busybox binary; BusyBox m4-specific
                // invocation handling is deferred.
                let busybox_path = which_binary("busybox")?;
                Ok(OracleConfig {
                    m4_path: Some(busybox_path),
                    ..Default::default()
                })
            }
            OracleKind::SystemM4 => Ok(OracleConfig {
                m4_path: Some(PathBuf::from("/usr/bin/m4")),
                ..Default::default()
            }),
            OracleKind::BsdM4 => {
                for candidate in &["/usr/bin/m4", "/usr/local/bin/m4", "/usr/bin/bsdm4"] {
                    let p = PathBuf::from(candidate);
                    if p.exists() {
                        return Ok(OracleConfig {
                            m4_path: Some(p),
                            ..Default::default()
                        });
                    }
                }
                Err(crate::OracleError::NotFound("BSD m4 not found".into()))
            }
            OracleKind::Custom(s) => {
                let p = PathBuf::from(s);
                if p.exists() {
                    Ok(OracleConfig {
                        m4_path: Some(p),
                        ..Default::default()
                    })
                } else {
                    Err(crate::OracleError::NotFound(format!(
                        "custom oracle not found: {}",
                        s
                    )))
                }
            }
        }
    }

    /// Try to locate this oracle on the system and return the path.
    pub fn locate(&self) -> Result<PathBuf, crate::OracleError> {
        let config = self.config()?;
        locate_m4(&config)
    }
}

/// Find a binary by searching PATH.
fn which_binary(name: &str) -> Result<PathBuf, crate::OracleError> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in path_var.split(':') {
        let candidate = std::path::Path::new(dir).join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(crate::OracleError::NotFound(format!(
        "binary not found: {}",
        name
    )))
}

/// A multi-oracle registry that manages multiple m4 implementations.
///
/// The primary oracle (GNU m4) is the behavioral authority.
/// Secondary oracles provide cross-verification.
pub struct MultiOracle {
    /// Primary oracle (GNU m4) — the behavioral authority
    pub primary: OracleProfile,
    /// Secondary oracles for cross-verification
    pub secondaries: Vec<OracleProfile>,
}

impl MultiOracle {
    /// Admit the primary GNU m4 oracle and attempt to locate secondaries.
    pub fn admit_all(config: &OracleConfig) -> Result<Self, crate::OracleError> {
        let primary = crate::admit_oracle(config)?;

        let mut secondaries = Vec::new();

        // Try BusyBox m4
        if let Ok(path) = OracleKind::BusyboxM4.locate() {
            if path.to_string_lossy().contains("busybox") {
                // BusyBox m4 is a special case — needs `busybox m4` invocation.
                // Admitted as "available but not yet admissible" for now.
                let _busybox_config = OracleConfig {
                    m4_path: Some(path),
                    locale: config.locale.clone(),
                    shell: config.shell.clone(),
                    env: config.env.clone(),
                };
            }
        }

        // Try system m4 at /usr/bin/m4 (only if different from primary)
        let sys_path = PathBuf::from("/usr/bin/m4");
        if sys_path.exists() && sys_path.to_string_lossy().as_ref() != primary.path.as_str() {
            let sys_config = OracleConfig {
                m4_path: Some(sys_path),
                locale: config.locale.clone(),
                shell: config.shell.clone(),
                env: config.env.clone(),
            };
            if let Ok(profile) = crate::admit_oracle(&sys_config) {
                if profile.kind != primary.kind {
                    secondaries.push(profile);
                }
            }
        }

        Ok(Self {
            primary,
            secondaries,
        })
    }

    /// Run a fixture against all admitted oracles and compare results.
    pub fn interrogate_all(
        &self,
        args: &[&str],
        stdin: &[u8],
    ) -> Result<Vec<OracleInterrogationResult>, crate::OracleError> {
        let mut results = Vec::new();

        // Primary oracle
        let primary_path = std::path::Path::new(&self.primary.path);
        let run = run_oracle_text(
            primary_path,
            args,
            &String::from_utf8_lossy(stdin),
            None,
            &HashMap::new(),
        )
        .map_err(|e| crate::OracleError::Execution(format!("primary: {}", e)))?;
        results.push(OracleInterrogationResult {
            oracle_kind: self.primary.kind.clone(),
            stdout: run.stdout,
            stderr: run.stderr,
            exit_code: run.exit_code,
        });

        // Secondaries
        for sec in &self.secondaries {
            let sec_path = std::path::Path::new(&sec.path);
            match run_oracle_text(
                sec_path,
                args,
                &String::from_utf8_lossy(stdin),
                None,
                &HashMap::new(),
            ) {
                Ok(run) => {
                    results.push(OracleInterrogationResult {
                        oracle_kind: sec.kind.clone(),
                        stdout: run.stdout,
                        stderr: run.stderr,
                        exit_code: run.exit_code,
                    });
                }
                Err(e) => {
                    results.push(OracleInterrogationResult {
                        oracle_kind: sec.kind.clone(),
                        stdout: Vec::new(),
                        stderr: format!("error: {}", e).into_bytes(),
                        exit_code: Some(1),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Check if all oracles agree on a given fixture.
    pub fn all_agree(&self, args: &[&str], stdin: &[u8]) -> Result<bool, crate::OracleError> {
        let results = self.interrogate_all(args, stdin)?;
        if results.len() < 2 {
            return Ok(true); // Only one oracle — trivially agrees
        }

        let first = &results[0];
        for other in &results[1..] {
            if first.stdout != other.stdout || first.exit_code != other.exit_code {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Result of interrogating a single oracle.
#[derive(Debug, Clone)]
pub struct OracleInterrogationResult {
    pub oracle_kind: String,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_kind_labels() {
        assert_eq!(OracleKind::GnuM4.label(), "gnu_m4");
        assert_eq!(OracleKind::BusyboxM4.label(), "busybox_m4");
        assert_eq!(OracleKind::SystemM4.label(), "system_m4");
        assert_eq!(OracleKind::BsdM4.label(), "bsd_m4");
        assert_eq!(OracleKind::Custom("test".into()).label(), "custom_test");
    }

    #[test]
    fn test_locate_gnu_m4() {
        let kind = OracleKind::GnuM4;
        match kind.locate() {
            Ok(path) => {
                eprintln!("Found GNU m4 at: {}", path.display());
            }
            Err(crate::OracleError::NotFound(_)) => {
                eprintln!("GNU m4 not found — skipping (expected in minimal containers)");
            }
            Err(e) => panic!("unexpected error: {}", e),
        }
    }
}
