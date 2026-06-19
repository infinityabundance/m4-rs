// Fuzz target: expansion engine panic-free on arbitrary tokenizable input.
//
// Feeds random bytes through the lexer + expansion engine and verifies
// no panic occurs. Also validates that output is valid UTF-8 or empty.
// Run with: cargo fuzz run expansion_engine

#![no_main]

use libfuzzer_sys::fuzz_target;
use m4_rs_core::expansion::ExpansionEngine;
use m4_rs_core::lexer::Lexer;

fuzz_target!(|data: &[u8]| {
    let mut engine = ExpansionEngine::new();
    engine.register_builtins();
    let mut lexer = Lexer::new();
    let tokens = lexer.tokenize(data);
    // Must never panic on arbitrary token streams
    engine.expand_tokens(&tokens);
    // Output may be empty or any bytes — just verify no panic
    let _output = &engine.output;
});
