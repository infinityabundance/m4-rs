// m4-casefile-rs: Receipt schema for m4-rs forensic parity courts.
//
// Every parity claim must be backed by a sealed receipt conforming to this schema.
// No positive claim is valid without a matching receipt on disk.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level receipt schema version marker.
pub const RECEIPT_SCHEMA: &str = "m4-rs-receipt-v1";

/// A sealed parity receipt — the atomic unit of a forensic claim.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// Schema version for format evolution
    pub schema: String,

    /// Court identifier, e.g. "M4.CLI.1", "M4.QUOTE.1"
    pub court: String,

    /// Verdict: "pass", "fail", "partial", "not_applicable"
    pub verdict: String,

    /// Information about the oracle binary used
    pub oracle: OracleInfo,

    /// Information about the Rust implementation
    pub rust: RustInfo,

    /// Environment in which the comparison was run
    pub environment: EnvironmentInfo,

    /// The test fixture
    pub fixture: FixtureInfo,

    /// The detailed comparison results
    pub comparison: ComparisonResult,

    /// What this receipt positively proves
    pub positive_claim: String,

    /// What this receipt explicitly does NOT prove
    pub non_claims: Vec<String>,

    /// Known divergences that are intentional or understood
    pub known_divergences: Vec<Divergence>,

    /// Command to replay this receipt
    pub replay_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleInfo {
    pub kind: String,
    pub version_output: String,
    pub path: String,
    pub sha256: String,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustInfo {
    pub crate_version: String,
    pub git_commit: String,
    pub binary_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub os: String,
    pub arch: String,
    pub locale: String,
    pub shell: String,
    pub cwd_policy: String,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureInfo {
    pub name: String,
    pub input_sha256: String,
    pub files_sha256: HashMap<String, String>,
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    /// "byte_exact", "class_location_match", "stable_substring_match",
    /// "known_version_divergence", "intentional_divergence", "not_applicable"
    pub stdout: String,
    pub stderr: String,
    pub exit_status: String,
    pub filesystem_outputs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Divergence {
    pub axis: String,
    pub oracle_behavior: String,
    pub rust_behavior: String,
    pub reason: String,
    pub is_intentional: bool,
}

/// A signed claim — one row in the claim ladder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub court: String,
    pub surface: String,
    pub status: String, // "sealed", "partial", "unclaimed"
    pub receipts: Vec<String>,
    pub description: String,
    pub since_version: String,
}

/// The full claim ladder — machine authority for what is claimed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimLadder {
    pub schema: String,
    pub generated_at: String,
    pub claims: Vec<Claim>,
    pub sealed_count: usize,
    pub partial_count: usize,
    pub unclaimed_count: usize,
}

impl Receipt {
    /// Create a new receipt with defaults filled from the court name.
    pub fn new(court: &str, positive_claim: &str) -> Self {
        Self {
            schema: RECEIPT_SCHEMA.to_string(),
            court: court.to_string(),
            verdict: "pass".to_string(),
            oracle: OracleInfo {
                kind: String::new(),
                version_output: String::new(),
                path: String::new(),
                sha256: String::new(),
                profile: String::new(),
            },
            rust: RustInfo {
                crate_version: String::new(),
                git_commit: String::new(),
                binary_sha256: String::new(),
            },
            environment: EnvironmentInfo {
                os: String::new(),
                arch: String::new(),
                locale: "C".to_string(),
                shell: "/bin/sh".to_string(),
                cwd_policy: "tempdir".to_string(),
                timezone: "UTC".to_string(),
            },
            fixture: FixtureInfo {
                name: String::new(),
                input_sha256: String::new(),
                files_sha256: HashMap::new(),
                argv: vec![],
            },
            comparison: ComparisonResult {
                stdout: String::new(),
                stderr: String::new(),
                exit_status: String::new(),
                filesystem_outputs: String::new(),
            },
            positive_claim: positive_claim.to_string(),
            non_claims: vec![],
            known_divergences: vec![],
            replay_command: String::new(),
        }
    }

    /// Verify internal consistency of this receipt.
    pub fn verify(&self) -> Result<(), Vec<String>> {
        let mut errors = vec![];

        if self.schema != RECEIPT_SCHEMA {
            errors.push(format!(
                "unknown schema: {} (expected {})",
                self.schema, RECEIPT_SCHEMA
            ));
        }

        if self.court.is_empty() {
            errors.push("court identifier is empty".into());
        }

        if self.positive_claim.is_empty() {
            errors.push("positive_claim is empty".into());
        }

        if self.verdict == "pass" && self.comparison.stdout.is_empty() {
            errors.push("pass verdict but no stdout comparison".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Render this receipt as a human-readable markdown string.
    pub fn render(&self) -> String {
        let mut md = String::new();
        md.push_str(&format!("# Receipt: {}\n\n", self.court));
        md.push_str(&format!("**Verdict:** {}\n\n", self.verdict));
        md.push_str(&format!("**Claim:** {}\n\n", self.positive_claim));

        md.push_str("## Oracle\n\n");
        md.push_str(&format!("- **Kind:** {}\n", self.oracle.kind));
        md.push_str(&format!("- **Path:** {}\n", self.oracle.path));
        md.push_str(&format!("- **SHA256:** {}\n", self.oracle.sha256));
        md.push_str(&format!(
            "- **Version:**\n```\n{}\n```\n\n",
            self.oracle.version_output
        ));

        md.push_str("## Comparison\n\n");
        md.push_str(&format!("- **stdout:** {}\n", self.comparison.stdout));
        md.push_str(&format!("- **stderr:** {}\n", self.comparison.stderr));
        md.push_str(&format!(
            "- **exit_status:** {}\n",
            self.comparison.exit_status
        ));
        md.push_str(&format!(
            "- **filesystem:** {}\n\n",
            self.comparison.filesystem_outputs
        ));

        if !self.non_claims.is_empty() {
            md.push_str("## Non-Claims\n\n");
            for nc in &self.non_claims {
                md.push_str(&format!("- {}\n", nc));
            }
            md.push('\n');
        }

        if !self.known_divergences.is_empty() {
            md.push_str("## Known Divergences\n\n");
            for d in &self.known_divergences {
                md.push_str(&format!(
                    "- **{}:** {} (intentional: {})\n",
                    d.axis, d.reason, d.is_intentional
                ));
            }
            md.push('\n');
        }

        md.push_str(&format!(
            "## Replay\n\n```sh\n{}\n```\n",
            self.replay_command
        ));

        md
    }
}

impl ClaimLadder {
    pub fn new() -> Self {
        Self {
            schema: RECEIPT_SCHEMA.to_string(),
            generated_at: String::new(),
            claims: vec![],
            sealed_count: 0,
            partial_count: 0,
            unclaimed_count: 0,
        }
    }

    pub fn recount(&mut self) {
        self.sealed_count = self.claims.iter().filter(|c| c.status == "sealed").count();
        self.partial_count = self.claims.iter().filter(|c| c.status == "partial").count();
        self.unclaimed_count = self
            .claims
            .iter()
            .filter(|c| c.status == "unclaimed")
            .count();
    }
}

impl Default for ClaimLadder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_new_and_verify() {
        let mut r = Receipt::new("M4.CLI.1", "m4-rs copies plain text byte-identically");
        r.comparison.stdout = "byte_exact".to_string();
        r.comparison.stderr = "not_applicable".to_string();
        r.comparison.exit_status = "exact".to_string();
        r.comparison.filesystem_outputs = "not_applicable".to_string();

        let result = r.verify();
        // Empty vec means no errors
        assert!(result.is_ok(), "{:?}", result);
    }

    #[test]
    fn test_receipt_render() {
        let r = Receipt::new("M4.CLI.1", "byte-identical copy-through");
        let md = r.render();
        assert!(md.contains("M4.CLI.1"));
        assert!(md.contains("byte-identical copy-through"));
    }
}
