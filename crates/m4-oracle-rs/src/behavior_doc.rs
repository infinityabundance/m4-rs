// m4-oracle-rs: Doxygen-style behavior documentation extraction.
//
// WHO:   infinityabundance, inspired by Doxygen's structured doc comments
//        and the forensic-parity methodology from sibling projects.
// WHAT:  Defines a structured format for documenting oracle behavior in a
//        machine-parseable way. Each behavior claim is a "witness statement"
//        that can be cross-referenced between:
//        - The GNU m4 manual description
//        - The actual oracle behavior (black-box interrogation)
//        - The m4-rs Rust implementation (via @m4_behavior doc tags)
// WHEN:  Used by xtask `behaviors` scan and `ast-verify` commands.
// WHERE: m4-oracle-rs/src/behavior_doc.rs
// WHY:   Without structured behavior documentation, it's impossible to prove
//        that every line of Rust code corresponds to a verified oracle behavior.
//        This module bridges the gap between "we think it works" and "we proved
//        it works against oracle witness M4.X.Y.Z".
// HOW:   Rust doc comments tagged with @m4_behavior are parsed from source files.
//        Each witness has an ID, surface label, claim text, manual reference,
//        oracle invocation, expected output, non-claims, and divergence notes.
//        The scan_directory_for_behaviors function recursively finds all witnesses.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single behavior witness extracted from doc comments or a separate file.
///
/// This is the machine-parseable equivalent of a Doxygen `@param`/`@return` block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorWitness {
    pub id: String,
    pub surface: String,
    pub claim: String,
    pub manual_section: String,
    pub oracle_invoke: String,
    pub oracle_expect: Expectation,
    pub non_claims: Vec<String>,
    pub divergences: Vec<DivergenceNote>,
    pub oracle_interrogations: Vec<OracleInterrogation>,
    pub rust_status: WitnessStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expectation {
    pub match_type: String,
    pub stdout_base64: String,
    pub stderr_base64: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergenceNote {
    pub axis: String,
    pub oracle_behavior: String,
    pub rust_behavior: String,
    pub reason: String,
    pub intentional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleInterrogation {
    pub oracle_kind: String,
    pub oracle_sha256: String,
    pub timestamp: String,
    pub passed: bool,
    pub stdout_match: bool,
    pub stderr_match: bool,
    pub exit_code_match: bool,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WitnessStatus {
    Unverified,
    Passing,
    Failing(String),
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorDoc {
    pub schema: String,
    pub module_name: String,
    pub description: String,
    pub witnesses: Vec<BehaviorWitness>,
    pub cross_references: Vec<CrossReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    pub witness_id: String,
    pub rust_file: String,
    pub rust_lines: String,
    pub rust_function: String,
    pub implementation_note: String,
}

/// Parse a Rust source file and extract `@m4_behavior` tagged doc comments.
pub fn extract_behaviors_from_rust_source(source: &str, _file_path: &str) -> Vec<BehaviorWitness> {
    let mut witnesses = Vec::new();
    let mut current_witness: Option<BehaviorWitness> = None;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("/// @m4_behavior ") || trimmed.starts_with("//! @m4_behavior ") {
            if let Some(w) = current_witness.take() {
                witnesses.push(w);
            }

            let id = trimmed
                .trim_start_matches("/// @m4_behavior ")
                .trim_start_matches("//! @m4_behavior ")
                .trim()
                .to_string();

            current_witness = Some(BehaviorWitness {
                id,
                surface: String::new(),
                claim: String::new(),
                manual_section: String::new(),
                oracle_invoke: String::new(),
                oracle_expect: Expectation {
                    match_type: "byte_exact".to_string(),
                    stdout_base64: String::new(),
                    stderr_base64: String::new(),
                    exit_code: Some(0),
                },
                non_claims: Vec::new(),
                divergences: Vec::new(),
                oracle_interrogations: Vec::new(),
                rust_status: WitnessStatus::Unverified,
            });
            continue;
        }

        if let Some(ref mut w) = current_witness {
            if trimmed.starts_with("/// @surface ") || trimmed.starts_with("//! @surface ") {
                w.surface = extract_field_value(trimmed, "@surface ");
            } else if trimmed.starts_with("/// @claim ") || trimmed.starts_with("//! @claim ") {
                w.claim = extract_field_value(trimmed, "@claim ");
            } else if trimmed.starts_with("/// @manual_section ")
                || trimmed.starts_with("//! @manual_section ")
            {
                w.manual_section = extract_field_value(trimmed, "@manual_section ");
            } else if trimmed.starts_with("/// @oracle_invoke ")
                || trimmed.starts_with("//! @oracle_invoke ")
            {
                w.oracle_invoke = extract_field_value(trimmed, "@oracle_invoke ");
            } else if trimmed.starts_with("/// @oracle_expect ")
                || trimmed.starts_with("//! @oracle_expect ")
            {
                let value = extract_field_value(trimmed, "@oracle_expect ");
                parse_expectation(&mut w.oracle_expect, &value);
            } else if trimmed.starts_with("/// @non_claim ")
                || trimmed.starts_with("//! @non_claim ")
            {
                w.non_claims
                    .push(extract_field_value(trimmed, "@non_claim "));
            } else if trimmed.starts_with("/// @divergence ")
                || trimmed.starts_with("//! @divergence ")
            {
                let div = parse_divergence(extract_field_value(trimmed, "@divergence "));
                w.divergences.push(div);
            }
        }
    }

    if let Some(w) = current_witness {
        witnesses.push(w);
    }

    witnesses
}

fn extract_field_value(line: &str, prefix: &str) -> String {
    let start = line.find(prefix).unwrap_or(0) + prefix.len();
    line[start..].trim().to_string()
}

fn parse_expectation(expect: &mut Expectation, value: &str) {
    let parts: Vec<&str> = value.splitn(3, ' ').collect();
    if parts.len() >= 2 {
        let channel = parts[0];
        let match_type = parts[1];
        let data = parts.get(2).unwrap_or(&"");

        if channel == "stdout" {
            expect.match_type = match_type.to_string();
            expect.stdout_base64 = data.to_string();
        } else if channel == "stderr" {
            expect.stderr_base64 = data.to_string();
        }
    }
}

fn parse_divergence(value: String) -> DivergenceNote {
    let parts: Vec<&str> = value.split('|').collect();
    DivergenceNote {
        axis: parts.first().unwrap_or(&"").to_string(),
        oracle_behavior: parts.get(1).unwrap_or(&"").to_string(),
        rust_behavior: parts.get(2).unwrap_or(&"").to_string(),
        reason: parts.get(3).unwrap_or(&"").to_string(),
        intentional: *parts.get(4).unwrap_or(&"false") == "true",
    }
}

/// Scan a directory of Rust source files for behavior witnesses.
pub fn scan_directory_for_behaviors(dir: &Path) -> Vec<BehaviorWitness> {
    let mut witnesses = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                witnesses.extend(scan_directory_for_behaviors(&path));
            } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(source) = std::fs::read_to_string(&path) {
                    let file_path = path.to_string_lossy().to_string();
                    witnesses.extend(extract_behaviors_from_rust_source(&source, &file_path));
                }
            }
        }
    }
    witnesses
}

/// Generate a behavior document from a set of witnesses.
pub fn build_behavior_doc(
    module_name: &str,
    description: &str,
    witnesses: Vec<BehaviorWitness>,
) -> BehaviorDoc {
    BehaviorDoc {
        schema: "m4-rs-behavior-doc-v1".to_string(),
        module_name: module_name.to_string(),
        description: description.to_string(),
        witnesses,
        cross_references: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_behavior_witness() {
        let source = r#"
/// @m4_behavior M4.LEX.1.001
/// @surface lexer/name-recognition
/// @claim A name is the longest sequence of [a-zA-Z_][a-zA-Z0-9_]*
/// @manual_section 3.1
/// @oracle_invoke echo 'define foo' | m4
/// @oracle_expect stdout byte_exact ZGVmaW5lIGZvbwo=
/// @non_claim Names starting with digits are not names
fn lex_name_recognition() {}
"#;
        let witnesses = extract_behaviors_from_rust_source(source, "test.rs");
        assert_eq!(witnesses.len(), 1);
        assert_eq!(witnesses[0].id, "M4.LEX.1.001");
        assert_eq!(witnesses[0].surface, "lexer/name-recognition");
        assert_eq!(witnesses[0].manual_section, "3.1");
        assert_eq!(witnesses[0].non_claims.len(), 1);
    }

    #[test]
    fn test_multiple_witnesses() {
        let source = r#"
/// @m4_behavior M4.LEX.1.001
/// @surface lexer/names
/// @claim First witness
fn foo() {}

/// @m4_behavior M4.LEX.1.002
/// @surface lexer/names
/// @claim Second witness
fn bar() {}
"#;
        let witnesses = extract_behaviors_from_rust_source(source, "test.rs");
        assert_eq!(witnesses.len(), 2);
    }
}
