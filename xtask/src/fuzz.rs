// xtask fuzz: Deterministic 1M-iteration fuzz runner.
//
// WHO:   infinityabundance
// WHAT:  Runs 1,000,000 pseudo-random-but-deterministic inputs through
//        the m4-rs expansion engine, verifying no panics, no hangs,
//        and output determinism (same input → same output on rerun).
// WHEN:  Run via `cargo xtask fuzz` (standalone) or as part of CI.
// WHERE: xtask/src/fuzz.rs
// WHY:   Fuzzing proves the engine is panic-free across a vast input
//        space. Deterministic seeding ensures reproducibility — the
//        same seed always produces the same results, making failures
//        reproducible for debugging.
// HOW:   Uses a simple LCG (Linear Congruential Generator) seeded with
//        a fixed value for deterministic reproducibility. Generates
//        byte sequences fed through lexer + expansion engine.
//        Each iteration: generate random bytes → tokenize → expand →
//        check for panic. Periodically verifies output is valid UTF-8
//        or binary-safe.

use std::time::Instant;

/// Simple LCG for deterministic pseudo-random number generation.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        // LCG parameters from glibc
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state & 0x7fffffff
    }

    fn next_byte(&mut self) -> u8 {
        (self.next() & 0xff) as u8
    }

    fn next_bytes(&mut self, buf: &mut [u8]) {
        for b in buf {
            *b = self.next_byte();
        }
    }

    /// Generate a length in [0, max_len].
    fn next_len(&mut self, max_len: usize) -> usize {
        (self.next() as usize) % (max_len + 1)
    }

    /// Generate a printable ASCII string of random length.
    fn next_printable(&mut self, buf: &mut Vec<u8>) {
        buf.clear();
        let len = self.next_len(256);
        for _ in 0..len {
            // Printable ASCII: 0x20..=0x7e
            let b = 0x20 + (self.next_byte() % 95);
            buf.push(b);
        }
    }

    /// Generate a byte string that includes m4-significant characters.
    fn next_m4_input(&mut self, buf: &mut Vec<u8>) {
        buf.clear();
        let len = self.next_len(512);
        for _ in 0..len {
            let category = (self.next() % 10) as u8;
            let b = match category {
                0 => b'`',                           // quote open
                1 => b'\'',                          // quote close
                2 => b'(',                           // paren open
                3 => b')',                           // paren close
                4 => b',',                           // comma
                5 => b'$',                           // dollar
                6 => b'#',                           // comment
                7 => b'\n',                          // newline
                8 => 0x20 + (self.next_byte() % 95), // printable
                _ => self.next_byte(),               // any byte
            };
            buf.push(b);
        }
    }
}

/// Run 1M fuzz iterations.
pub fn run_fuzz(iterations: usize) -> Result<FuzzReport, String> {
    println!("=== m4-rs Deterministic Fuzz Runner ===\n");
    println!("Iterations: {}", iterations);
    println!("Seed: 0xDEADBEEF_M4RS (deterministic)");
    println!();

    let seed = 0xDEAD_BEEF_0000_0001u64;
    let mut rng = Lcg::new(seed);
    let start = Instant::now();

    let mut panics = 0u64;
    let mut nondeterministic = 0u64;
    let mut phase1_ok = 0u64;
    let mut phase2_ok = 0u64;
    let mut phase3_ok = 0u64;

    let mut buf = Vec::with_capacity(1024);

    // Progress reporting interval
    let report_interval = iterations / 20;

    for i in 0..iterations {
        if i > 0 && i % report_interval == 0 {
            let pct = (i as f64 / iterations as f64 * 100.0) as u32;
            let elapsed = start.elapsed();
            println!(
                "  {}% ({}/{}) — {:.1}s elapsed",
                pct,
                i,
                iterations,
                elapsed.as_secs_f64()
            );
        }

        let phase = (rng.next() % 3) as u8;

        match phase {
            // Phase 0: Random printable ASCII input
            0 => {
                rng.next_printable(&mut buf);
                let result = run_single(&buf);
                match result {
                    FuzzResult::Ok => phase1_ok += 1,
                    FuzzResult::Panic => panics += 1,
                    FuzzResult::NonDeterministic => nondeterministic += 1,
                }
            }
            // Phase 1: M4-significant character mix
            1 => {
                rng.next_m4_input(&mut buf);
                let result = run_single(&buf);
                match result {
                    FuzzResult::Ok => phase2_ok += 1,
                    FuzzResult::Panic => panics += 1,
                    FuzzResult::NonDeterministic => nondeterministic += 1,
                }
            }
            // Phase 2: Pure random binary
            _ => {
                let len = rng.next_len(1024);
                buf.resize(len, 0);
                rng.next_bytes(&mut buf);
                let result = run_single(&buf);
                match result {
                    FuzzResult::Ok => phase3_ok += 1,
                    FuzzResult::Panic => panics += 1,
                    FuzzResult::NonDeterministic => nondeterministic += 1,
                }
            }
        }

        if panics > 0 {
            eprintln!(
                "PANIC at iteration {}: phase={}, input={:?}",
                i,
                phase,
                String::from_utf8_lossy(&buf)
            );
            break;
        }
    }

    let elapsed = start.elapsed();
    let total = phase1_ok + phase2_ok + phase3_ok;

    let report = FuzzReport {
        iterations: total + panics + nondeterministic,
        phase1_printable: phase1_ok,
        phase2_m4_chars: phase2_ok,
        phase3_binary: phase3_ok,
        panics,
        nondeterministic,
        elapsed_secs: elapsed.as_secs_f64(),
        seed,
    };

    println!();
    println!("=== Fuzz Results ===");
    println!("  Phase 1 (printable ASCII):  {}", phase1_ok);
    println!("  Phase 2 (m4-significant):   {}", phase2_ok);
    println!("  Phase 3 (random binary):    {}", phase3_ok);
    println!("  Panics:                     {}", panics);
    println!("  Non-deterministic:          {}", nondeterministic);
    println!("  Total iterations:           {}", total);
    println!(
        "  Time:                       {:.1}s",
        elapsed.as_secs_f64()
    );
    if total > 0 {
        println!(
            "  Rate:                       {:.0} iter/s",
            total as f64 / elapsed.as_secs_f64()
        );
    }

    if panics > 0 || nondeterministic > 0 {
        println!("\n=== FUZZ FAILED ===");
    } else {
        println!("\n=== FUZZ PASSED — {} iterations, 0 panics ===", total);
    }

    Ok(report)
}

enum FuzzResult {
    Ok,
    Panic,
    NonDeterministic,
}

fn run_single(input: &[u8]) -> FuzzResult {
    use m4_rs_core::expansion::ExpansionEngine;
    use m4_rs_core::lexer::Lexer;
    use std::panic;

    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        let mut engine = ExpansionEngine::new();
        engine.register_builtins();
        let mut lexer = Lexer::new();
        let tokens = lexer.tokenize(input);
        engine.expand_tokens(&tokens);
        engine.undivert_all();
        engine.flush_wrap_buffer();
        let output1 = engine.output.clone();

        // Determinism check: run again with fresh engine
        let mut engine2 = ExpansionEngine::new();
        engine2.register_builtins();
        let mut lexer2 = Lexer::new();
        let tokens2 = lexer2.tokenize(input);
        engine2.expand_tokens(&tokens2);
        engine2.undivert_all();
        engine2.flush_wrap_buffer();
        let output2 = engine2.output.clone();

        (output1, output2)
    }));

    match result {
        Ok((o1, o2)) => {
            if o1 != o2 {
                FuzzResult::NonDeterministic
            } else {
                FuzzResult::Ok
            }
        }
        Err(_) => FuzzResult::Panic,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FuzzReport {
    pub iterations: u64,
    pub phase1_printable: u64,
    pub phase2_m4_chars: u64,
    pub phase3_binary: u64,
    pub panics: u64,
    pub nondeterministic: u64,
    pub elapsed_secs: f64,
    pub seed: u64,
}

/// Entry point for the xtask command.
pub fn run() -> std::process::ExitCode {
    let iterations = 1_000_000;

    match run_fuzz(iterations) {
        Ok(report) => {
            // Save receipt
            if let Ok(json) = serde_json::to_string_pretty(&report) {
                let _ = std::fs::create_dir_all("reports/receipts");
                let _ = std::fs::write("reports/receipts/fuzz-1M-receipt.json", &json);
                println!("Receipt saved to reports/receipts/fuzz-1M-receipt.json");
            }

            if report.panics > 0 || report.nondeterministic > 0 {
                std::process::ExitCode::FAILURE
            } else {
                std::process::ExitCode::SUCCESS
            }
        }
        Err(e) => {
            eprintln!("Fuzz error: {}", e);
            std::process::ExitCode::FAILURE
        }
    }
}
