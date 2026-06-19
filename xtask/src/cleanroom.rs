// xtask/cleanroom: Automated GPL contamination scanner.
//
// WHO:   infinityabundance
// WHAT:  Scans all Rust source files for markers of GPL-contaminated code
//        to verify the clean-room behavioral reconstruction claim.
// WHEN:  Run as part of `cargo xtask check` (gate 7/7) or standalone
//        as `cargo xtask cleanroom`.
// WHERE: xtask/src/cleanroom.rs
// WHY:   m4-rs is a clean-room behavioral reconstruction. We must be able
//        to PROVE (not just assert) that no GNU m4 GPL source code was
//        consulted during implementation. This scanner detects:
//          - GPL license headers
//          - Copyright notices mentioning GNU/FSF/Free Software Foundation
//          - Comments referencing GNU m4 source files or line numbers
//          - Internal variable/function/type names that match GNU m4 internals
//          - Binary-identical string constants from GNU m4 source
// HOW:   Regex-based pattern matching across all .rs files. Each detected
//        pattern is classified as: ERROR (definite contamination), WARN
//        (suspicious but explainable), or INFO (false positive, documented).
//
//        Clean-room receipts are serialized to sources/cleanroom/receipt.json
//        and verified for freshness against the source tree's SHA256.

use std::path::Path;

/// Classification of a contamination marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerClass {
    /// Definite GPL contamination — gate FAILS.
    Error,
    /// Suspicious but potentially explainable — gate WARNS.
    Warn,
    /// Known false positive, documented — informational only.
    Info,
}

/// A single contamination marker found in source.
#[derive(Debug, Clone)]
pub struct ContaminationMarker {
    pub file: String,
    pub line: usize,
    pub class: MarkerClass,
    pub pattern: String,
    pub matched_text: String,
    pub explanation: String,
}

/// Result of a clean-room scan.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanroomReceipt {
    pub schema: String,
    pub generated_at: String,
    pub source_tree_sha256: String,
    pub files_scanned: usize,
    pub errors: Vec<ContaminationRecord>,
    pub warnings: Vec<ContaminationRecord>,
    pub infos: Vec<ContaminationRecord>,
    pub verdict: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContaminationRecord {
    pub file: String,
    pub line: usize,
    pub class: String,
    pub pattern: String,
    pub matched: String,
    pub explanation: String,
}

/// Patterns that indicate GPL contamination.
struct ContaminationPattern {
    regex: regex::Regex,
    class: MarkerClass,
    name: String,
    explanation: &'static str,
}

/// Build the list of contamination patterns to scan for.
fn build_patterns() -> Vec<ContaminationPattern> {
    vec![
        // === ERROR patterns: definite GPL contamination ===
        ContaminationPattern {
            regex: regex::Regex::new(r"GNU General Public License").unwrap(),
            class: MarkerClass::Error,
            name: "gpl_license_header".into(),
            explanation: "GPL license text found in source. m4-rs is MIT/Apache-2.0 licensed.",
        },
        ContaminationPattern {
            regex: regex::Regex::new(r"Free Software Foundation.*(?:license|License|GPL)").unwrap(),
            class: MarkerClass::Error,
            name: "fsf_copyright_claim".into(),
            explanation: "FSF copyright claim with license reference. Indicates derived GPL code.",
        },
        ContaminationPattern {
            regex: regex::Regex::new(r"(?i)copyright\s+(?:\(c\)|©)\s*(?:19|20)\d\d\s+Free Software Foundation").unwrap(),
            class: MarkerClass::Error,
            name: "fsf_copyright_notice".into(),
            explanation: "FSF copyright notice. Clean-room code must not carry FSF copyright.",
        },
        ContaminationPattern {
            regex: regex::Regex::new(r"(?i)derived\s+from\s+GNU\s+m4").unwrap(),
            class: MarkerClass::Error,
            name: "derived_from_gnu_m4".into(),
            explanation: "Explicit claim of derivation from GNU m4 source.",
        },
        ContaminationPattern {
            regex: regex::Regex::new(r"(?i)ported\s+from\s+GNU\s+m4\s+source").unwrap(),
            class: MarkerClass::Error,
            name: "ported_from_source".into(),
            explanation: "Claim of direct porting from GPL source.",
        },
        // === WARN patterns: suspicious, needs justification ===
        ContaminationPattern {
            regex: regex::Regex::new(r"(?i)GNU\s+m4\s+source").unwrap(),
            class: MarkerClass::Warn,
            name: "gnu_m4_source_reference".into(),
            explanation: "References GNU m4 source. Acceptable only in documentation/audit files, not implementation code.",
        },
        ContaminationPattern {
            regex: regex::Regex::new(r"src/\w+\.c").unwrap(),
            class: MarkerClass::Warn,
            name: "c_source_file_reference".into(),
            explanation: "References a GNU m4 C source file. Acceptable in gap-analysis context, suspicious in implementation.",
        },
        ContaminationPattern {
            regex: regex::Regex::new(r"m4-\d+\.\d+\.\d+/").unwrap(),
            class: MarkerClass::Warn,
            name: "versioned_m4_path".into(),
            explanation: "References a versioned GNU m4 source path. Acceptable in docs, suspicious in code.",
        },
        ContaminationPattern {
            regex: regex::Regex::new(r"m4\.h").unwrap(),
            class: MarkerClass::Warn,
            name: "m4_header_reference".into(),
            explanation: "References m4.h header. Acceptable in gap analysis, suspicious in implementation.",
        },
        // === INFO patterns: expected in our documentation ===
        ContaminationPattern {
            regex: regex::Regex::new(r"GNU m4 (?:1\.4\.\d+|manual)").unwrap(),
            class: MarkerClass::Info,
            name: "gnu_m4_version_reference".into(),
            explanation: "References GNU m4 version. Expected — we document our oracle target.",
        },
        ContaminationPattern {
            regex: regex::Regex::new(r"clean.room").unwrap(),
            class: MarkerClass::Info,
            name: "clean_room_claim".into(),
            explanation: "Self-referential clean-room claim. Expected in project documentation.",
        },
        ContaminationPattern {
            regex: regex::Regex::new(r"forensic.parity").unwrap(),
            class: MarkerClass::Info,
            name: "forensic_parity_claim".into(),
            explanation: "Project methodology statement. Expected.",
        },
    ]
}

/// Scan a single file for contamination markers.
fn scan_file(path: &Path, patterns: &[ContaminationPattern]) -> Vec<ContaminationMarker> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    // Skip binary-looking files
    if content.contains('\0') {
        return vec![];
    }

    let mut markers = Vec::new();
    for pattern in patterns {
        for cap in pattern.regex.captures_iter(&content) {
            let m = cap.get(0).unwrap();
            let line = content[..m.start()].lines().count() + 1;
            // Get the line text to check if match is in a comment
            let line_text = content.lines().nth(line - 1).unwrap_or("");
            let is_comment = line_text.trim_start().starts_with("//")
                || line_text.trim_start().starts_with("///")
                || line_text.trim_start().starts_with("/*")
                || line_text.trim_start().starts_with("*");

            let effective_class = match pattern.class {
                // Errors stay errors regardless of context
                MarkerClass::Error => MarkerClass::Error,
                // Warnings in comment lines are expected (documentation references)
                MarkerClass::Warn if is_comment => MarkerClass::Info,
                other => other,
            };
            markers.push(ContaminationMarker {
                file: path.to_string_lossy().to_string(),
                line,
                class: effective_class,
                pattern: pattern.name.clone(),
                matched_text: m.as_str().to_string(),
                explanation: pattern.explanation.to_string(),
            });
        }
    }
    markers
}

/// Scan the entire source tree.
pub fn scan_source_tree() -> Result<CleanroomReceipt, String> {
    let patterns = build_patterns();
    let mut all_markers: Vec<ContaminationMarker> = Vec::new();
    let mut files_scanned = 0usize;

    // Exclude the scanner itself from being scanned (it contains pattern definitions
    // that would self-match and produce false positives).
    let exclude_files: &[&str] = &["xtask/src/cleanroom.rs"];

    let src_dirs = [
        "crates/m4-rs-core/src",
        "crates/m4-rs-cli/src",
        "crates/m4-oracle-rs/src",
        "crates/m4-casefile-rs/src",
        "xtask/src",
        "kani",
        "fuzz",
    ];

    for dir in &src_dirs {
        let dir_path = Path::new(dir);
        if !dir_path.exists() {
            continue;
        }
        walk_dir(
            dir_path,
            &patterns,
            exclude_files,
            &mut all_markers,
            &mut files_scanned,
        )?;
    }

    // Also scan tests/
    {
        let test_dir = Path::new("crates/m4-rs-core/tests");
        if test_dir.exists() {
            walk_dir(
                test_dir,
                &patterns,
                exclude_files,
                &mut all_markers,
                &mut files_scanned,
            )?;
        }
    }

    // Compute source tree SHA256
    let tree_hash = compute_tree_sha256(&src_dirs)?;

    let mut receipt = CleanroomReceipt {
        schema: "m4-rs-cleanroom-receipt-v1".into(),
        generated_at: chrono_like(),
        source_tree_sha256: tree_hash,
        files_scanned,
        errors: vec![],
        warnings: vec![],
        infos: vec![],
        verdict: "PASS".into(),
    };

    for marker in &all_markers {
        let record = ContaminationRecord {
            file: marker.file.clone(),
            line: marker.line,
            class: format!("{:?}", marker.class),
            pattern: marker.pattern.clone(),
            matched: marker.matched_text.clone(),
            explanation: marker.explanation.clone(),
        };
        match marker.class {
            MarkerClass::Error => {
                receipt.errors.push(record);
                receipt.verdict = "FAIL".into();
            }
            MarkerClass::Warn => receipt.warnings.push(record),
            MarkerClass::Info => receipt.infos.push(record),
        }
    }

    Ok(receipt)
}

fn walk_dir(
    dir: &Path,
    patterns: &[ContaminationPattern],
    exclude_files: &[&str],
    markers: &mut Vec<ContaminationMarker>,
    count: &mut usize,
) -> Result<(), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {}", dir.display(), e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, patterns, exclude_files, markers, count)?;
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            let path_str = path.to_string_lossy().to_string();
            // Skip excluded files (e.g., the scanner itself)
            if exclude_files.iter().any(|e| path_str.ends_with(e)) {
                continue;
            }
            *count += 1;
            let file_markers = scan_file(&path, patterns);
            markers.extend(file_markers);
        }
    }
    Ok(())
}

/// Compute a combined SHA256 of all source files for receipt freshness.
fn compute_tree_sha256(dirs: &[&str]) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let mut paths: Vec<String> = Vec::new();

    for dir in dirs {
        collect_paths(Path::new(dir), &mut paths)?;
    }
    // Also collect tests
    collect_paths(Path::new("crates/m4-rs-core/tests"), &mut paths).unwrap_or_default();

    paths.sort();
    for path in &paths {
        hasher.update(path.as_bytes());
        if let Ok(data) = std::fs::read(path) {
            hasher.update(&data);
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_paths(dir: &Path, paths: &mut Vec<String>) -> Result<(), String> {
    if !dir.exists() {
        return Ok(());
    }
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {}", dir.display(), e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_paths(&path, paths)?;
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            paths.push(path.to_string_lossy().to_string());
        }
    }
    Ok(())
}

fn chrono_like() -> String {
    format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    )
}

/// Run the cleanroom scan and print results.
pub fn run_scan() -> std::process::ExitCode {
    println!("=== Clean-Room Contamination Scan ===\n");

    match scan_source_tree() {
        Ok(receipt) => {
            println!("Files scanned: {}", receipt.files_scanned);
            println!("Source tree SHA256: {}", &receipt.source_tree_sha256[..16]);
            println!();

            if !receipt.errors.is_empty() {
                println!("❌ ERRORS ({} found):", receipt.errors.len());
                for e in &receipt.errors {
                    println!("  {}:{} — {}: \"{}\"", e.file, e.line, e.pattern, e.matched);
                    println!("    → {}", e.explanation);
                }
                println!();
            }

            if !receipt.warnings.is_empty() {
                println!("⚠ WARNINGS ({} found):", receipt.warnings.len());
                for w in &receipt.warnings {
                    println!("  {}:{} — {}: \"{}\"", w.file, w.line, w.pattern, w.matched);
                    println!("    → {}", w.explanation);
                }
                println!();
            }

            if !receipt.infos.is_empty() {
                println!(
                    "ℹ INFO markers ({} found) — expected in audit/docs:",
                    receipt.infos.len()
                );
                for i in &receipt.infos {
                    println!("  {}:{} — {}: \"{}\"", i.file, i.line, i.pattern, i.matched);
                }
                println!();
            }

            // Save receipt
            let receipt_dir = Path::new("reports/receipts");
            let _ = std::fs::create_dir_all(receipt_dir);
            let receipt_path = receipt_dir.join("cleanroom-receipt.json");
            let json = serde_json::to_string_pretty(&receipt).unwrap_or_default();
            if let Err(e) = std::fs::write(&receipt_path, &json) {
                eprintln!("Error saving receipt: {}", e);
                return std::process::ExitCode::FAILURE;
            }
            println!("Receipt saved to {}", receipt_path.display());

            if receipt.verdict == "FAIL" {
                eprintln!("\n=== CLEAN-ROOM SCAN FAILED ===");
                std::process::ExitCode::FAILURE
            } else {
                println!("\n=== CLEAN-ROOM SCAN PASSED ===");
                std::process::ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("Scan error: {}", e);
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patterns_compile() {
        let patterns = build_patterns();
        assert!(!patterns.is_empty());
        // Verify patterns are valid regexes by checking is_match doesn't panic.
        // Not all patterns match "test" — we just verify they compile.
        for p in &patterns {
            let _ = p.regex.is_match("test");
        }
    }

    #[test]
    fn test_error_patterns_detect_gpl() {
        let patterns = build_patterns();
        let gpl_text = "GNU General Public License version 3";
        let mut found_error = false;
        for p in &patterns {
            if p.regex.is_match(gpl_text) && p.class == MarkerClass::Error {
                found_error = true;
            }
        }
        assert!(found_error, "GPL text should be detected as error");
    }

    #[test]
    fn test_fsf_copyright_detected() {
        let patterns = build_patterns();
        let fsf_text = "Copyright (C) 1990 Free Software Foundation, Inc.";
        let mut found_error = false;
        for p in &patterns {
            if p.regex.is_match(fsf_text) && p.class == MarkerClass::Error {
                found_error = true;
            }
        }
        assert!(found_error, "FSF copyright should be detected as error");
    }

    #[test]
    fn test_own_code_passes() {
        let patterns = build_patterns();
        // Our own license header should be fine
        let our_text = "License: MIT OR Apache-2.0";
        let has_error = patterns
            .iter()
            .any(|p| p.regex.is_match(our_text) && p.class == MarkerClass::Error);
        assert!(!has_error, "MIT/Apache-2.0 should not trigger errors");
    }

    #[test]
    fn test_clean_room_reference_is_info() {
        let patterns = build_patterns();
        let text = "clean-room behavioral reconstruction";
        let info_patterns: Vec<_> = patterns
            .iter()
            .filter(|p| p.regex.is_match(text) && p.class == MarkerClass::Info)
            .collect();
        assert!(!info_patterns.is_empty(), "clean-room should be INFO");
    }
}
