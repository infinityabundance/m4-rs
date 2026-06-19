//! Performance baseline benchmark — `cargo xtask bench`
//!
//! Measures m4-rs throughput vs GNU m4 oracle on standard workloads.
//! Not claiming parity — establishing a measured baseline.

use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::time::Instant;

pub fn run() -> ExitCode {
    println!("=== m4-rs Performance Baseline ===\n");

    let oracle = find_m4();
    let m4rs = find_m4rs();

    // Standard benchmark workloads (owned Strings to avoid temporary borrow issues)
    let d100 = "define(`x',`hello')x\n".repeat(100);
    let d1k = "define(`x',`hello')x\n".repeat(1000);
    let d10k = "define(`x',`hello')x\n".repeat(10000);
    let ev = "eval(1+2+3+4+5+6+7+8+9+10)\n".repeat(1000);
    let dv = "divert(1)xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\ndivert(0)\n".repeat(100);

    let workloads: Vec<(&str, &str)> = vec![
        ("tiny-define", "define(`x',`hello')x\n"),
        ("100-defines", &d100),
        ("1k-defines", &d1k),
        ("10k-defines", &d10k),
        ("nested-define", "define(`outer',`define(`inner',`nested')')outer inner\n"),
        ("ifelse-chain", "ifelse(`a',`a',`1',`b',`b',`2',`c',`c',`3',`d',`d',`4',`e',`e',`5',`default')\n"),
        ("eval-heavy", &ev),
        ("divert-fill", &dv),
        ("forloop-10", "define(`forloop',`pushdef(`$1',`$2')_forloop(`$1',`$2',`$3',`$4')popdef(`$1')')define(`_forloop',`$4`'ifelse($1,`$3',,`define(`$1',incr($1))_forloop(`$1',`$2',`$3',`$4')')')forloop(`i',1,10,`i ')\n"),
        ("tiny-copy", "define(`x',`ok')x\n"),
    ];

    println!(
        "{:<20} {:>10} {:>10} {:>10} {:>8}",
        "Workload", "Oracle", "m4-rs", "Ratio", "Bytes"
    );
    println!("{}", "-".repeat(62));

    let tmp = std::env::temp_dir().join("m4-rs-bench");
    std::fs::create_dir_all(&tmp).unwrap();

    let mut oracle_total = 0f64;
    let mut rust_total = 0f64;

    for (name, input) in &workloads {
        let path = tmp.join(format!("{}.m4", name));
        std::fs::write(&path, input).unwrap();

        // Benchmark GNU m4 (5 runs, take median)
        let oracle_time = bench_binary(&oracle, &path, 5);
        let rust_time = bench_binary(&m4rs, &path, 5);

        let ratio = if oracle_time > 0.0 {
            rust_time / oracle_time
        } else {
            0.0
        };
        let marker = if ratio < 5.0 {
            "✅"
        } else if ratio < 20.0 {
            "⚠️"
        } else {
            "❌"
        };

        println!(
            "{:<20} {:>8.1}ms {:>8.1}ms {:>8.1}x {:>6} {}",
            name,
            oracle_time * 1000.0,
            rust_time * 1000.0,
            ratio,
            input.len(),
            marker
        );

        oracle_total += oracle_time;
        rust_total += rust_time;
    }

    let overall = if oracle_total > 0.0 {
        rust_total / oracle_total
    } else {
        0.0
    };
    println!("\n=== Overall: {:.1}x slower than GNU m4 ===", overall);
    println!("Oracle total: {:.1}ms", oracle_total * 1000.0);
    println!("m4-rs total:  {:.1}ms", rust_total * 1000.0);

    // Save baseline
    let receipt = serde_json::json!({
        "schema": "m4-rs-perf-baseline-v1",
        "timestamp": chrono_now(),
        "overall_ratio": overall,
        "workloads": workloads.iter().map(|(name, input)| {
            serde_json::json!({
                "name": name,
                "input_bytes": input.len(),
            })
        }).collect::<Vec<_>>(),
    });
    let dir = std::path::Path::new("lab/corpus/receipts");
    std::fs::create_dir_all(dir).ok();
    std::fs::write(
        dir.join("perf-baseline.json"),
        serde_json::to_string_pretty(&receipt).unwrap(),
    )
    .ok();

    ExitCode::SUCCESS
}

fn bench_binary(binary: &std::path::Path, fixture: &std::path::Path, runs: u32) -> f64 {
    let mut times: Vec<f64> = Vec::new();
    for _ in 0..runs {
        let start = Instant::now();
        let _ = Command::new(binary)
            .arg(fixture)
            .env("LC_ALL", "C")
            .output();
        times.push(start.elapsed().as_secs_f64());
    }
    // Return median
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[times.len() / 2]
}

fn find_m4() -> PathBuf {
    for p in &["/usr/bin/m4", "/usr/local/bin/m4"] {
        if std::path::Path::new(p).exists() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from("/usr/bin/m4")
}

fn find_m4rs() -> PathBuf {
    for p in &["target/release/m4-rs", "target/debug/m4-rs"] {
        if std::path::Path::new(p).exists() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from("target/debug/m4-rs")
}

fn chrono_now() -> String {
    Command::new("date")
        .arg("-u")
        .arg("+%Y-%m-%dT%H:%M:%SZ")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}
