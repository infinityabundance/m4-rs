// xtask — project maintenance tasks.
//
// Usage: cargo xtask <command>
//
// Commands:
//   check       Run all acceptance gate checks (fmt, clippy, test, freshness, oracle)
//   fmt         Run rustfmt
//   clippy      Run clippy with warnings denied
//   test        Run all tests
//   oracle      Run oracle admission
//   generate    (Re)generate all documents from JSON sources
//   receipts    Verify receipt freshness
//   claims      Verify claim ladder freshness
//   ast-verify  Run AST parity verification bridge against oracle
//   behaviors   Scan source for @m4_behavior witnesses
//   status      Print current status summary
//
// The check command runs all gates and is the standard CI entry point.

mod ast_verify;
mod bench;
mod cleanroom;
mod compare;
mod docgen;
mod fuzz;
mod gnu_compare;
mod smoke;

use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("check");

    match command {
        "check" => run_check(),
        "fmt" => run_fmt(),
        "clippy" => run_clippy(),
        "test" => run_test(),
        "oracle" => run_oracle_admission(),
        "compare" => compare::run(),
        "generate" => run_generate(),
        "receipts" => run_receipt_check(),
        "claims" => run_claim_check(),
        "ast-verify" => run_ast_verify(),
        "behaviors" => run_behaviors_scan(),
        "cleanroom" => run_cleanroom_scan(),
        "fuzz" => fuzz::run(),
        "smoke" => smoke::run(),
        "gnu-compare" => gnu_compare::run(),
        "bench" => bench::run(),
        "status" => run_status(),
        _ => {
            eprintln!("xtask: unknown command: {}", command);
            eprintln!("Available: check, fmt, clippy, test, oracle, compare, generate, receipts, claims, ast-verify, behaviors, cleanroom, fuzz, smoke, status");
            ExitCode::FAILURE
        }
    }
}

fn run_check() -> ExitCode {
    println!("=== m4-rs acceptance gate check ===\n");
    let mut failed = false;

    // 1. Format
    println!("[1/7] rustfmt...");
    let fmt = Command::new("cargo")
        .args(["fmt", "--", "--check"])
        .status();
    if fmt.map(|s| !s.success()).unwrap_or(true) {
        eprintln!("  FAIL: formatting issues");
        failed = true;
    } else {
        println!("  PASS");
    }

    // 2. Clippy
    println!("[2/7] clippy...");
    let clippy = Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "-D", "warnings"])
        .status();
    if clippy.map(|s| !s.success()).unwrap_or(true) {
        eprintln!("  FAIL: clippy issues");
        failed = true;
    } else {
        println!("  PASS");
    }

    // 3. Tests
    println!("[3/7] tests...");
    let test = Command::new("cargo").args(["test", "--all"]).status();
    if test.map(|s| !s.success()).unwrap_or(true) {
        eprintln!("  FAIL: tests failed");
        failed = true;
    } else {
        println!("  PASS");
    }

    // 4. Document freshness
    println!("[4/7] document freshness...");
    let registry_path = Path::new("reports/doc-registry.json");
    if registry_path.exists() {
        match std::fs::read_to_string(registry_path) {
            Ok(json) => match serde_json::from_str::<docgen::DocumentRegistry>(&json) {
                Ok(registry) => match registry.verify_freshness() {
                    Ok(msgs) => {
                        for m in &msgs {
                            println!("  {}", m);
                        }
                    }
                    Err(stale) => {
                        for s in &stale {
                            eprintln!("  {}", s);
                        }
                        eprintln!("  FAIL: stale documents. Run 'cargo xtask generate'.");
                        failed = true;
                    }
                },
                Err(e) => {
                    eprintln!("  WARN: invalid registry: {}", e);
                }
            },
            Err(e) => {
                eprintln!("  WARN: cannot read registry: {}", e);
            }
        }
    } else {
        println!("  INFO: no doc registry yet. Run 'cargo xtask generate'.");
    }

    // 5. Oracle
    println!("[5/7] oracle profile...");
    if Path::new("reports/oracle-profile.json").exists() {
        println!("  PASS: oracle profile present");
    } else {
        println!("  WARN: no oracle profile. Run 'cargo xtask oracle'.");
    }

    // 6. Claim ladder
    println!("[6/7] claim ladder...");
    if Path::new("reports/claim-ladder.json").exists() {
        println!("  PASS: claim ladder present");
    } else {
        println!("  WARN: no claim ladder.");
    }

    // 7. Clean-room contamination scan
    println!("[7/7] clean-room scan...");
    match cleanroom::scan_source_tree() {
        Ok(receipt) => {
            if receipt.verdict == "FAIL" {
                eprintln!(
                    "  FAIL: {} GPL contamination errors found",
                    receipt.errors.len()
                );
                for e in &receipt.errors {
                    eprintln!("    {}:{} — {}: {}", e.file, e.line, e.pattern, e.matched);
                }
                failed = true;
            } else {
                println!(
                    "  PASS: {} files scanned, {} warnings, {} info markers",
                    receipt.files_scanned,
                    receipt.warnings.len(),
                    receipt.infos.len()
                );
                if let Ok(json) = serde_json::to_string_pretty(&receipt) {
                    let _ = std::fs::create_dir_all("reports/receipts");
                    let _ = std::fs::write("reports/receipts/cleanroom-receipt.json", &json);
                }
            }
        }
        Err(e) => {
            eprintln!("  FAIL: scan error: {}", e);
            failed = true;
        }
    }

    println!();
    if failed {
        eprintln!("=== ACCEPTANCE GATE FAILED ===");
        ExitCode::FAILURE
    } else {
        println!("=== ACCEPTANCE GATE PASSED ===");
        ExitCode::SUCCESS
    }
}

fn run_fmt() -> ExitCode {
    let s = Command::new("cargo")
        .args(["fmt"])
        .status()
        .unwrap_or_else(|_| std::process::exit(1));
    if s.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_clippy() -> ExitCode {
    let s = Command::new("cargo")
        .args(["clippy", "--all-targets", "--", "-D", "warnings"])
        .status()
        .unwrap_or_else(|_| std::process::exit(1));
    if s.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_test() -> ExitCode {
    let s = Command::new("cargo")
        .args(["test", "--all"])
        .status()
        .unwrap_or_else(|_| std::process::exit(1));
    if s.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn run_oracle_admission() -> ExitCode {
    println!("=== m4-rs oracle admission ===\n");
    match m4_oracle_rs::admit_oracle(&m4_oracle_rs::OracleConfig::default()) {
        Ok(profile) => {
            println!(
                "Oracle: {} (sha256: {})",
                profile.kind,
                &profile.sha256[..16]
            );
            if let Err(e) =
                m4_oracle_rs::save_profile(&profile, Path::new("reports/oracle-profile.json"))
            {
                eprintln!("Error saving: {}", e);
                return ExitCode::FAILURE;
            }
            println!("Saved to reports/oracle-profile.json");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Oracle admission failed: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_generate() -> ExitCode {
    println!("=== Document Generation ===\n");

    let key = b"m4-rs-forensic-key-2026";
    let mut registry = docgen::DocumentRegistry::new();

    match docgen::generate::generate_all(&mut registry, key) {
        Ok(results) => {
            for r in &results {
                println!("  {}", r);
            }
            // Save registry
            let json = serde_json::to_string_pretty(&registry).unwrap_or_default();
            if let Err(e) = std::fs::write("reports/doc-registry.json", &json) {
                eprintln!("Error saving registry: {}", e);
                return ExitCode::FAILURE;
            }
            println!("\nRegistry saved to reports/doc-registry.json");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for e in &errors {
                eprintln!("  {}", e);
            }
            ExitCode::FAILURE
        }
    }
}

fn run_receipt_check() -> ExitCode {
    println!("=== Receipt check ===\n");
    let dir = Path::new("reports/receipts");
    if !dir.exists() {
        println!("No receipts directory. Expected before courts are sealed.");
        return ExitCode::SUCCESS;
    }
    match std::fs::read_dir(dir) {
        Ok(entries) => {
            let count = entries
                .flatten()
                .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
                .count();
            println!("Receipts: {}", count);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_claim_check() -> ExitCode {
    println!("=== Claim ladder check ===\n");
    let path = Path::new("reports/claim-ladder.json");
    if !path.exists() {
        println!("No claim-ladder.json. Expected before courts are sealed.");
        return ExitCode::SUCCESS;
    }
    match std::fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<m4_casefile_rs::ClaimLadder>(&contents) {
            Ok(ladder) => {
                println!(
                    "Sealed: {}, Partial: {}, Unclaimed: {}",
                    ladder.sealed_count, ladder.partial_count, ladder.unclaimed_count
                );
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Parse error: {}", e);
                ExitCode::FAILURE
            }
        },
        Err(e) => {
            eprintln!("Read error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_ast_verify() -> ExitCode {
    println!("=== AST Parity Verification ===\n");
    let profile_path = Path::new("reports/oracle-profile.json");
    if !profile_path.exists() {
        eprintln!("No oracle profile. Run 'cargo xtask oracle' first.");
        return ExitCode::FAILURE;
    }
    match ast_verify::AstParityBridge::new(profile_path) {
        Ok(bridge) => {
            let report = bridge.verify_all();
            report.print();
            if report.failed > 0 {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

fn run_behaviors_scan() -> ExitCode {
    println!("=== @m4_behavior Witness Scan ===\n");
    use m4_oracle_rs::behavior_doc;
    let src_dirs = &[
        "crates/m4-rs-core/src",
        "crates/m4-rs-cli/src",
        "crates/m4-oracle-rs/src",
    ];
    let mut total = 0;
    for dir in src_dirs {
        let path = Path::new(dir);
        if path.exists() {
            let witnesses = behavior_doc::scan_directory_for_behaviors(path);
            total += witnesses.len();
            println!("{}: {} witness(es)", dir, witnesses.len());
            for w in &witnesses {
                println!(
                    "  - {} (surface: {}, manual: §{})",
                    w.id, w.surface, w.manual_section
                );
            }
        }
    }
    println!("\nTotal witnesses: {}", total);
    if total == 0 {
        println!("No @m4_behavior tags found. Add structured behavior docs to source files.");
    }
    ExitCode::SUCCESS
}

fn run_cleanroom_scan() -> ExitCode {
    cleanroom::run_scan()
}

fn run_status() -> ExitCode {
    println!("=== m4-rs project status ===\n");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Strategy: Clean-room behavioral reconstruction");
    println!("License: MIT OR Apache-2.0");
    println!();

    if Path::new("reports/oracle-profile.json").exists() {
        println!("Oracle: admitted ✓");
    } else {
        println!("Oracle: NOT YET ADMITTED — run 'cargo xtask oracle'");
    }

    let test_output = Command::new("cargo")
        .args(["test", "--all", "--", "--list"])
        .output();
    if let Ok(output) = test_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let test_count = stdout.lines().filter(|l| l.contains("test")).count();
        println!("Tests: {} found", test_count);
    }

    let receipts_dir = Path::new("reports/receipts");
    if receipts_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(receipts_dir) {
            let count = entries
                .flatten()
                .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
                .count();
            println!("Receipts: {}", count);
        }
    } else {
        println!("Receipts: none yet");
    }

    // Check gap analysis freshness
    if Path::new("reports/doc-registry.json").exists() {
        println!("\nDocument freshness: run 'cargo xtask check' for details.");
    }

    println!("\nIMPORTANT: m4-rs is NOT a GNU m4 replacement.");
    println!("See reports/FORENSIC-GAP-ANALYSIS.md for full gap details.");
    println!("See docs/negative-capabilities.md for the build roadmap.");
    ExitCode::SUCCESS
}
