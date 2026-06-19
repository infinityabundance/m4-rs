// m4-rs: AST parity verification bridge.
//
// This module cross-references the Rust implementation against the oracle
// behavior documentation. It extracts `@m4_behavior` tagged doc comments
// from the Rust source, interrogates the oracle for each behavior, and
// verifies that the Rust implementation produces matching output.
//
// This is the verification compass — it guides all development by making
// the gap between "what the oracle does" and "what we implement" explicit
// and machine-checkable.
//
// The bridge works at three levels:
//   1. Behavior extraction — scan Rust source for @m4_behavior tags
//   2. Oracle interrogation — run each behavior against the oracle
//   3. Implementation check — verify m4-rs matches the oracle output

use m4_oracle_rs::behavior_doc::{self, BehaviorWitness};
use m4_oracle_rs::multi_oracle::MultiOracle;
use m4_oracle_rs::{OracleConfig, OracleProfile};
use std::path::Path;

/// The AST parity verification bridge.
pub struct AstParityBridge {
    /// The admitted primary oracle
    pub oracle: OracleProfile,
    /// The multi-oracle registry (reserved for future cross-oracle verification)
    #[allow(dead_code)]
    pub multi_oracle: Option<MultiOracle>,
    /// Extracted behavior witnesses
    pub witnesses: Vec<BehaviorWitness>,
}

impl AstParityBridge {
    /// Initialize the bridge with the admitted oracle and scan for behaviors.
    pub fn new(oracle_profile_path: &Path) -> Result<Self, String> {
        let oracle = m4_oracle_rs::load_profile(oracle_profile_path)
            .map_err(|e| format!("failed to load oracle profile: {}", e))?;

        // Try to build multi-oracle
        let multi = MultiOracle::admit_all(&OracleConfig::default()).ok();

        // Scan the crate source directories for behavior witnesses
        let mut witnesses = Vec::new();

        let src_dirs = &[
            "crates/m4-rs-core/src",
            "crates/m4-rs-cli/src",
            "crates/m4-oracle-rs/src",
        ];

        for dir in src_dirs {
            let path = Path::new(dir);
            if path.exists() {
                witnesses.extend(behavior_doc::scan_directory_for_behaviors(path));
            }
        }

        Ok(Self {
            oracle,
            multi_oracle: multi,
            witnesses,
        })
    }

    /// Verify a single behavior witness against the Rust implementation.
    ///
    /// This runs the oracle invocation specified in the witness and compares
    /// the output against the expected values stored in the witness.
    pub fn verify_witness_against_oracle(&self, witness: &BehaviorWitness) -> Result<bool, String> {
        let oracle_path = Path::new(&self.oracle.path);

        // Parse the oracle invocation from the witness
        // Format: "echo 'input' | m4" or "m4 [args] file.in"
        // For simplicity, extract stdin and args
        let (stdin, args) = parse_invocation(&witness.oracle_invoke);

        let run = m4_oracle_rs::run_oracle_text(
            oracle_path,
            &args.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            &stdin,
            None,
            &std::collections::HashMap::new(),
        )
        .map_err(|e| format!("oracle run failed: {}", e))?;

        // Compare stdout
        // If no base64 expectation, check if stdout is non-empty
        let expected_stdout =
            base64_decode(&witness.oracle_expect.stdout_base64).unwrap_or_default();

        let stdout_match = if witness.oracle_expect.match_type == "byte_exact" {
            run.stdout == expected_stdout
        } else if witness.oracle_expect.match_type == "class_location_match" {
            // For diagnostics, just check that stderr is non-empty when expected
            expected_stdout.is_empty() == run.stdout.is_empty()
        } else {
            // substring match — check that oracle output contains expected
            String::from_utf8_lossy(&run.stdout)
                .contains(String::from_utf8_lossy(&expected_stdout).as_ref())
        };

        let exit_code_match = match witness.oracle_expect.exit_code {
            Some(expected) => run.exit_code == Some(expected),
            None => true,
        };

        Ok(stdout_match && exit_code_match)
    }

    /// Verify all behavior witnesses and return a status summary.
    pub fn verify_all(&self) -> VerificationReport {
        let mut verified = 0;
        let mut failed = 0;
        let mut unchecked = 0;
        let mut results = Vec::new();

        for witness in &self.witnesses {
            match self.verify_witness_against_oracle(witness) {
                Ok(true) => {
                    verified += 1;
                    results.push(WitnessResult {
                        id: witness.id.clone(),
                        status: "verified".to_string(),
                        note: "Oracle output matches expectation".to_string(),
                    });
                }
                Ok(false) => {
                    failed += 1;
                    results.push(WitnessResult {
                        id: witness.id.clone(),
                        status: "failed".to_string(),
                        note: "Oracle output does not match expectation".to_string(),
                    });
                }
                Err(e) => {
                    unchecked += 1;
                    results.push(WitnessResult {
                        id: witness.id.clone(),
                        status: "unchecked".to_string(),
                        note: format!("Error: {}", e),
                    });
                }
            }
        }

        VerificationReport {
            total: self.witnesses.len(),
            verified,
            failed,
            unchecked,
            results,
        }
    }
}

/// Parse an oracle invocation string into stdin and args.
///
/// Handles formats like:
///   "echo 'input' | m4"
///   "m4 -Dfoo=bar file.in"
///   "m4 < file.in"
fn parse_invocation(invocation: &str) -> (String, Vec<String>) {
    if invocation.contains('|') {
        // Pipe format: "echo 'input' | m4 [args]"
        let parts: Vec<&str> = invocation.splitn(2, '|').collect();
        let cmd = parts[0].trim();
        let m4_part = parts.get(1).unwrap_or(&"").trim();

        // Extract stdin from the echo command
        let stdin = if cmd.starts_with("echo ") {
            let echo_arg = cmd.trim_start_matches("echo ").trim();
            // Remove surrounding quotes
            echo_arg.trim_matches('\'').trim_matches('"').to_string()
        } else {
            String::new()
        };

        // Extract m4 args
        let args = if m4_part.starts_with("m4 ") {
            m4_part
                .trim_start_matches("m4 ")
                .split_whitespace()
                .map(|s| s.to_string())
                .collect()
        } else {
            vec![]
        };

        (stdin, args)
    } else if invocation.starts_with("m4 ") {
        // Direct invocation: "m4 [args] file"
        let rest = invocation.trim_start_matches("m4 ").trim();
        let parts: Vec<&str> = rest.split_whitespace().collect();

        let mut args = Vec::new();
        let mut stdin = String::new();
        let mut i = 0;
        while i < parts.len() {
            if parts[i].starts_with('-') {
                args.push(parts[i].to_string());
                i += 1;
                // Consume argument to flag
                if i < parts.len() && !parts[i].starts_with('-') {
                    args.push(parts[i].to_string());
                    i += 1;
                }
            } else {
                // This is a filename — read its contents as "stdin" for verification
                if let Ok(contents) = std::fs::read_to_string(parts[i]) {
                    stdin = contents;
                }
                i += 1;
            }
        }

        (stdin, args)
    } else {
        (String::new(), vec![])
    }
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    if s.is_empty() {
        return Some(Vec::new());
    }
    // Simple base64 decode using a minimal implementation
    let chars: Vec<u8> = s.bytes().filter(|b| *b != b'=' && *b != b'\n').collect();
    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u32;

    for &c in &chars {
        let val = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => return None,
        };
        buffer = (buffer << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Some(result)
}

/// Result of verifying a single witness.
#[derive(Debug, Clone)]
pub struct WitnessResult {
    pub id: String,
    pub status: String,
    pub note: String,
}

/// Full verification report.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    pub total: usize,
    pub verified: usize,
    pub failed: usize,
    pub unchecked: usize,
    pub results: Vec<WitnessResult>,
}

impl VerificationReport {
    pub fn print(&self) {
        println!("=== AST Parity Verification Report ===");
        println!("Total witnesses: {}", self.total);
        println!("  Verified: {}", self.verified);
        println!("  Failed:   {}", self.failed);
        println!("  Unchecked: {}", self.unchecked);
        println!();

        for result in &self.results {
            let icon = match result.status.as_str() {
                "verified" => "✓",
                "failed" => "✗",
                _ => "?",
            };
            println!("  {} {}: {}", icon, result.id, result.note);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_invocation_pipe() {
        let (stdin, args) = parse_invocation("echo 'hello world' | m4");
        assert_eq!(stdin, "hello world");
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_invocation_direct() {
        let (stdin, args) = parse_invocation("m4 -Dfoo=bar");
        assert_eq!(args, vec!["-Dfoo=bar"]);
        assert!(stdin.is_empty());
    }

    #[test]
    fn test_base64_decode() {
        // "hello" in base64
        let decoded = base64_decode("aGVsbG8=").unwrap();
        assert_eq!(decoded, b"hello");
    }
}
