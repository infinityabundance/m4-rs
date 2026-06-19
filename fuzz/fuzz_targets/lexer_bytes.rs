// Fuzz target: lexer panic-free on arbitrary byte input.
//
// Feeds random bytes to the lexer and verifies no panic occurs.
// Run with: cargo fuzz run lexer_bytes

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut lexer = m4_rs_core::Lexer::new();
    // The lexer must never panic, no matter what bytes are fed to it
    let _tokens = lexer.tokenize(data);
});
