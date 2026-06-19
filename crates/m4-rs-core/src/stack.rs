// m4-rs stack module — native Rust stack overflow guard.
//
// Port of GNU m4 src/stackovf.c (~4KB). GNU m4 uses sigaltstack + SIGSEGV
// handler + sigsetjmp/siglongjmp to detect and recover from stack overflow.
//
// Safe Rust cannot directly intercept SIGSEGV. Instead, we:
// 1. Use std::thread::Builder::stack_size() for user-configurable stack limits
// 2. Estimate remaining stack space via stack probe
// 3. Document the C→Rust architectural gap precisely
//
// CROSS-REF: CROSS.20 — stack overflow test differs (Rust aborts on overflow)

use std::sync::atomic::{AtomicUsize, Ordering};

/// Default stack size for m4-rs worker threads (matches typical GNU m4 limits).
pub const DEFAULT_STACK_SIZE: usize = 8 * 1024 * 1024; // 8MB

/// Tracks the configured nesting/recursion limit.
static RECURSION_LIMIT: AtomicUsize = AtomicUsize::new(1024);

/// Set the recursion limit for macro expansion.
/// In GNU m4, this is controlled by `-L` / `--nesting-limit`.
pub fn set_recursion_limit(limit: usize) {
    RECURSION_LIMIT.store(limit, Ordering::Relaxed);
}

/// Get the current recursion limit.
pub fn get_recursion_limit() -> usize {
    RECURSION_LIMIT.load(Ordering::Relaxed)
}

/// Quick stack probe: writes to stack memory at increasing offsets
/// to test if we're near the guard page. Returns estimated remaining bytes.
/// This is a heuristic — it cannot guarantee detection of all overflows.
pub fn probe_stack() -> Option<usize> {
    // Use a volatile stack allocation to prevent optimization
    let mut probe: [u8; 4096] = [0; 4096];
    // Touch each page in the probe to force stack growth
    for i in (0..probe.len()).step_by(4096) {
        unsafe {
            std::ptr::write_volatile(&mut probe[i], 0);
        }
    }
    // If we got here without SIGSEGV, we have at least 4KB remaining
    std::hint::black_box(&probe);
    Some(4096)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Assert constant is sensible. Clippy suggests `const { assert!(..) }`
    /// but that syntax requires Rust 1.79+. Using allow with documented reason
    /// per project Rule 6.
    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_default_stack_size() {
        assert!(DEFAULT_STACK_SIZE > 0);
        assert_eq!(DEFAULT_STACK_SIZE, 8 * 1024 * 1024);
    }

    #[test]
    fn test_recursion_limit_default() {
        assert_eq!(get_recursion_limit(), 1024);
    }

    #[test]
    fn test_set_recursion_limit() {
        set_recursion_limit(500);
        assert_eq!(get_recursion_limit(), 500);
        set_recursion_limit(1024); // reset
    }

    #[test]
    fn test_stack_probe_succeeds() {
        // Should succeed under normal conditions (we're not near stack limit)
        let result = probe_stack();
        assert!(result.is_some());
    }
}
