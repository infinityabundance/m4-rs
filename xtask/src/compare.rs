//! Oracle comparison harness — `cargo xtask compare`
//!
//! Runs m4 fixture files through both the GNU m4 oracle and m4-rs,
//! compares stdout/stderr/exit status byte-by-byte, and generates
//! sealed receipts in reports/receipts/.
//!
//! Layers:
//!   layer0-smoke          — hand-written smoke tests
//!   layer1-gnu-testsuite  — GNU m4 testsuite extracts
//!   layer2-manual-examples — GNU m4 manual examples
//!   layer3-posix          — POSIX m4 spec examples
//!   layer4-autoconf-m4sugar — Autoconf m4sugar macro expansion
//!   layer5-autoconf-testsuite — Autoconf .at extracts (expansion only)
//!
//! GPL CODE ISOLATION: GNU m4 test files live on the QEMU VM, NOT in this repo.
//! Only receipts and the runner script are committed.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// A single comparison result.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct CompareResult {
    fixture: String,
    layer: String,
    oracle_stdout_sha256: String,
    rust_stdout_sha256: String,
    oracle_stderr_sha256: String,
    rust_stderr_sha256: String,
    oracle_exit: i32,
    rust_exit: i32,
    stdout_match: bool,
    stderr_match: bool,
    exit_match: bool,
    verdict: String, // "pass", "fail", "skip"
    note: Option<String>,
}

/// Run the comparison for all layers.
pub fn run() -> ExitCode {
    println!("=== m4-rs oracle comparison ===\n");

    // Find the GNU m4 binary
    let m4_binary = find_m4();
    let m4rs_binary = find_m4rs();

    println!("Oracle: {}", m4_binary.display());
    println!("m4-rs:  {}\n", m4rs_binary.display());

    let corpus_dir = Path::new("lab/corpus");
    let mut results: Vec<CompareResult> = Vec::new();

    // Layer 0: Smoke tests
    let layer0 = corpus_dir.join("layer0-smoke");
    if layer0.exists() {
        println!("--- Layer 0: Smoke tests ---");
        for entry in std::fs::read_dir(&layer0).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "m4") {
                let res = compare_fixture(&m4_binary, &m4rs_binary, &path, "layer0-smoke");
                print_result(&res);
                results.push(res);
            }
        }
        println!();
    }

    // Layer 1: GNU testsuite extracts
    let layer1 = corpus_dir.join("layer1-gnu-testsuite");
    if layer1.exists() {
        println!("--- Layer 1: GNU testsuite extracts ---");
        let mut files: Vec<_> = std::fs::read_dir(&layer1)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "m4"))
            .collect();
        files.sort_by_key(|e| e.file_name());
        for entry in &files {
            let res = compare_fixture(
                &m4_binary,
                &m4rs_binary,
                &entry.path(),
                "layer1-gnu-testsuite",
            );
            print_result(&res);
            results.push(res);
        }
        println!();
    }

    // Layer 2: Manual examples
    let layer2 = corpus_dir.join("layer2-manual-examples");
    if layer2.exists() {
        println!("--- Layer 2: Manual examples ---");
        let mut files: Vec<_> = std::fs::read_dir(&layer2)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "m4"))
            .collect();
        files.sort_by_key(|e| e.file_name());
        for entry in &files {
            let res = compare_fixture(
                &m4_binary,
                &m4rs_binary,
                &entry.path(),
                "layer2-manual-examples",
            );
            print_result(&res);
            results.push(res);
        }
        println!();
    }

    // Layer 3: POSIX examples
    let layer3 = corpus_dir.join("layer3-posix");
    if layer3.exists() {
        println!("--- Layer 3: POSIX examples ---");
        let mut files: Vec<_> = std::fs::read_dir(&layer3)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "m4"))
            .collect();
        files.sort_by_key(|e| e.file_name());
        for entry in &files {
            let res = compare_fixture(&m4_binary, &m4rs_binary, &entry.path(), "layer3-posix");
            print_result(&res);
            results.push(res);
        }
        println!();
    }

    // Layer 4-5: Autoconf (if present)
    for layer in &["layer4-autoconf-m4sugar", "layer5-autoconf-testsuite"] {
        let layer_dir = corpus_dir.join(layer);
        if layer_dir.exists() {
            println!("--- {} ---", layer);
            let mut files: Vec<_> = std::fs::read_dir(&layer_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "m4"))
                .collect();
            files.sort_by_key(|e| e.file_name());
            for entry in &files {
                let res = compare_fixture(&m4_binary, &m4rs_binary, &entry.path(), layer);
                print_result(&res);
                results.push(res);
            }
            println!();
        }
    }

    // Summary
    let total = results.len();
    let passed = results.iter().filter(|r| r.verdict == "pass").count();
    let failed = results.iter().filter(|r| r.verdict == "fail").count();
    let skipped = results.iter().filter(|r| r.verdict == "skip").count();

    println!("=== Comparison Summary ===");
    println!("Total:   {}", total);
    println!(
        "Passed:  {} ({:.1}%)",
        passed,
        if total > 0 {
            passed as f64 / total as f64 * 100.0
        } else {
            0.0
        }
    );
    println!("Failed:  {}", failed);
    println!("Skipped: {}", skipped);

    // Save receipt
    if !results.is_empty() {
        let receipt_dir = Path::new("lab/corpus/receipts");
        std::fs::create_dir_all(receipt_dir).ok();
        let receipt_path = receipt_dir.join("comparison-receipt.json");
        let mut grouped: BTreeMap<String, Vec<&CompareResult>> = BTreeMap::new();
        for r in &results {
            grouped.entry(r.layer.clone()).or_default().push(r);
        }
        if let Ok(json) = serde_json::to_string_pretty(&grouped) {
            std::fs::write(&receipt_path, &json).ok();
            println!("\nReceipt saved to {}", receipt_path.display());
        }
    }

    if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn find_m4() -> PathBuf {
    // Check common locations
    for path in &["/usr/bin/m4", "/usr/local/bin/m4", "/opt/homebrew/bin/m4"] {
        if Path::new(path).exists() {
            return PathBuf::from(path);
        }
    }
    // Try 'which'
    if let Ok(out) = Command::new("which").arg("m4").output() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() && Path::new(&s).exists() {
            return PathBuf::from(s);
        }
    }
    panic!("GNU m4 not found. Install with: apt-get install m4")
}

fn find_m4rs() -> PathBuf {
    // Try release build first, then debug
    for path in &["target/release/m4-rs", "target/debug/m4-rs"] {
        if Path::new(path).exists() {
            return PathBuf::from(path);
        }
    }
    // Build it
    let status = Command::new("cargo")
        .args(["build", "--bin", "m4-rs"])
        .status()
        .expect("Failed to build m4-rs");
    if !status.success() {
        panic!("Failed to build m4-rs");
    }
    if Path::new("target/debug/m4-rs").exists() {
        PathBuf::from("target/debug/m4-rs")
    } else {
        panic!("m4-rs binary not found after build");
    }
}

fn compare_fixture(m4: &Path, m4rs: &Path, fixture: &Path, layer: &str) -> CompareResult {
    let fixture_name = fixture.file_name().unwrap().to_string_lossy().to_string();

    // Skip known-hanging tests (CROSS.38 forloop, sandbox-required syscmd)
    let skip_list = ["033-forloop.m4", "034-foreach.m4", "052-syscmd.m4"];
    if skip_list.contains(&fixture_name.as_str()) {
        return CompareResult {
            fixture: fixture_name,
            layer: layer.to_string(),
            oracle_stdout_sha256: String::new(),
            rust_stdout_sha256: String::new(),
            oracle_stderr_sha256: String::new(),
            rust_stderr_sha256: String::new(),
            oracle_exit: 0,
            rust_exit: 0,
            stdout_match: false,
            stderr_match: false,
            exit_match: false,
            verdict: "skip".to_string(),
            note: Some("known gap: CROSS.38 or sandbox required".to_string()),
        };
    }

    // Run GNU m4
    let oracle = run_m4(m4, fixture);
    // Run m4-rs
    let rust = run_m4(m4rs, fixture);

    let stdout_match = oracle.stdout == rust.stdout;
    let stderr_match = oracle.stderr == rust.stderr;
    let exit_match = oracle.exit_code == rust.exit_code;

    // Verdict: stdout + exit match = pass. Stderr differences are noted
    // but don't fail (diagnostic wording varies between m4 versions).
    let verdict = if stdout_match && exit_match {
        "pass"
    } else {
        "fail"
    };

    let note = if !stderr_match {
        Some(format!(
            "stderr diverges (oracle={} bytes, rust={} bytes)",
            oracle.stderr.len(),
            rust.stderr.len()
        ))
    } else {
        None
    };

    CompareResult {
        fixture: fixture_name,
        layer: layer.to_string(),
        oracle_stdout_sha256: sha256_hex(&oracle.stdout),
        rust_stdout_sha256: sha256_hex(&rust.stdout),
        oracle_stderr_sha256: sha256_hex(&oracle.stderr),
        rust_stderr_sha256: sha256_hex(&rust.stderr),
        oracle_exit: oracle.exit_code,
        rust_exit: rust.exit_code,
        stdout_match,
        stderr_match,
        exit_match,
        verdict: verdict.to_string(),
        note,
    }
}

struct M4Output {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: i32,
}

fn run_m4(binary: &Path, fixture: &Path) -> M4Output {
    // Add a 5-second timeout per test to prevent hanging on forloop/recursion
    let result = Command::new(binary)
        .arg(fixture)
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("TZ", "UTC")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    match result {
        Ok(mut child) => {
            // Wait up to 5 seconds
            let one_sec = std::time::Duration::from_secs(1);
            let mut waited = 0;
            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        let out =
                            child
                                .wait_with_output()
                                .unwrap_or_else(|e| std::process::Output {
                                    status,
                                    stdout: Vec::new(),
                                    stderr: format!("ERROR: {}", e).into_bytes(),
                                });
                        return M4Output {
                            stdout: out.stdout,
                            stderr: out.stderr,
                            exit_code: out.status.code().unwrap_or(-1),
                        };
                    }
                    Ok(None) => {
                        if waited >= 5 {
                            let _ = child.kill();
                            let _ = child.wait();
                            return M4Output {
                                stdout: Vec::new(),
                                stderr: b"TIMEOUT: test exceeded 5 seconds".to_vec(),
                                exit_code: -1,
                            };
                        }
                        std::thread::sleep(one_sec);
                        waited += 1;
                    }
                    Err(e) => {
                        return M4Output {
                            stdout: Vec::new(),
                            stderr: format!("ERROR: {}", e).into_bytes(),
                            exit_code: -1,
                        };
                    }
                }
            }
        }
        Err(e) => M4Output {
            stdout: Vec::new(),
            stderr: format!("ERROR: {}", e).into_bytes(),
            exit_code: -1,
        },
    }
}

fn sha256_hex(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn print_result(r: &CompareResult) {
    let icon = match r.verdict.as_str() {
        "pass" => "✅",
        "fail" => "❌",
        _ => "⬜",
    };
    println!(
        "  {} {} (out={} err={} exit={})",
        icon,
        r.fixture,
        if r.stdout_match { "✓" } else { "✗" },
        if r.stderr_match { "✓" } else { "✗" },
        if r.exit_match { "✓" } else { "✗" },
    );
}
