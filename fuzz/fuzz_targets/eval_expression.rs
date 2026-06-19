// Fuzz target: eval engine correctness and panic-free.
//
// Generates random arithmetic expressions within a bounded grammar
// and verifies the eval engine produces a result without panicking.
// Also verifies that re-evaluating the same expression produces
// the same result (determinism check).
// Run with: cargo fuzz run eval_expression

#![no_main]

use libfuzzer_sys::fuzz_target;
use m4_rs_core::eval::eval_expression;

fuzz_target!(|data: &[u8]| {
    // Filter to printable ASCII to form valid expressions
    let expr: Vec<u8> = data
        .iter()
        .filter(|&&b| b.is_ascii_graphic() || b == b' ')
        .copied()
        .collect();

    if expr.is_empty() {
        return;
    }

    // First evaluation
    let result1 = eval_expression(&expr, 10, None);

    // Second evaluation (must be deterministic)
    let result2 = eval_expression(&expr, 10, None);

    // Must not panic, and must be deterministic
    match (result1, result2) {
        (Ok(r1), Ok(r2)) => {
            assert_eq!(r1, r2, "eval not deterministic for input");
        }
        (Err(_), Err(_)) => {
            // Both errored — acceptable (e.g., division by zero)
        }
        _ => {
            // One succeeded, one failed — non-deterministic!
            panic!(
                "eval non-deterministic: first={:?}, second={:?}",
                result1, result2
            );
        }
    }
});
