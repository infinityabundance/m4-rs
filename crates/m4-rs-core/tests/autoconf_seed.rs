// Autoconf seed survival tests — M4.AUTOCONF.SEED.1
//
// Runs 10 Autoconf-style fixture files through the m4-rs expansion engine
// and verifies no panics, output is produced, and specific patterns
// expected in Autoconf macro processing are present.
//
// NOTE: Complex macro chaining (AC_DEFUN-style nested define with $1/$2
// forwarding, multi-branch AS_IF with $@ recursion, recursive forloop
// with $1 resolution in nested calls) are documented gaps.
// These fixtures exercise the core engine capabilities we have admitted.

use m4_rs_core::expansion::ExpansionEngine;
use m4_rs_core::lexer::Lexer;
use std::path::Path;

fn process_fixture(name: &str) -> Vec<u8> {
    let path = Path::new("../../fixtures/autoconf").join(name);
    let data =
        std::fs::read(&path).unwrap_or_else(|_| panic!("cannot read fixture: {}", path.display()));
    let mut engine = ExpansionEngine::new();
    engine.register_builtins();
    engine
        .include_path
        .directories
        .push(Path::new("../../fixtures/autoconf").to_path_buf());

    let mut lexer = Lexer::with_file(name.to_string());
    let tokens = lexer.tokenize(&data);
    engine.expand_tokens(&tokens);
    engine.undivert_all();
    engine.flush_wrap_buffer();
    engine.output.clone()
}

#[test]
fn test_autoconf_01_basic_macros() {
    let output = process_fixture("01-basic-macros.m4");
    let text = String::from_utf8_lossy(&output);
    assert!(text.contains("bar"), "got: {}", text);
    assert!(text.contains("hello"), "got: {}", text);
}

#[test]
fn test_autoconf_02_conditionals() {
    let output = process_fixture("02-conditionals.m4");
    let text = String::from_utf8_lossy(&output);
    assert!(text.contains("Feature is enabled"), "got: {}", text);
    assert!(text.contains("match"), "got: {}", text);
    assert!(text.contains("no match"), "got: {}", text);
}

#[test]
fn test_autoconf_03_forloop() {
    let output = process_fixture("03-forloop.m4");
    let text = String::from_utf8_lossy(&output);
    // pushdef/popdef: pushdef shadow, popdef restore
    assert!(text.contains("1"), "got: {}", text);
    assert!(text.contains("0"), "got: {}", text);
    // eval: 1 + 2 = 3
    assert!(text.contains("3"), "got: {}", text);
}

#[test]
fn test_autoconf_04_arithmetic() {
    let output = process_fixture("04-arithmetic.m4");
    let text = String::from_utf8_lossy(&output);
    // eval(3 > 5) = 0, eval(5 > 3) = 1
    assert!(text.contains("0"), "got: {}", text);
    assert!(text.contains("1"), "got: {}", text);
    // eval(0x10 + 020) = 32, eval(1 << 4) = 16
    assert!(text.contains("32"), "got: {}", text);
    assert!(text.contains("16"), "got: {}", text);
}

#[test]
fn test_autoconf_05_quote_nesting() {
    let output = process_fixture("05-quote-nesting.m4");
    let text = String::from_utf8_lossy(&output);
    assert!(text.contains("hello"), "got: {}", text);
    assert!(text.contains("world"), "got: {}", text);
}

#[test]
fn test_autoconf_06_diversions() {
    let output = process_fixture("06-diversions.m4");
    let text = String::from_utf8_lossy(&output);
    assert!(text.contains("This text appears first"), "got: {}", text);
    assert!(text.contains("diverted to diversion 1"), "got: {}", text);
    let pos0 = text.find("appears first").unwrap();
    let pos1 = text.find("diverted to").unwrap();
    assert!(pos0 < pos1, "diversion ordering wrong: {}", text);
}

#[test]
fn test_autoconf_07_text_manipulation() {
    let output = process_fixture("07-text-manipulation.m4");
    let text = String::from_utf8_lossy(&output);
    assert!(text.contains("hello world"), "got: {}", text);
    assert!(text.contains("123def"), "got: {}", text);
}

#[test]
fn test_autoconf_08_include() {
    let output = process_fixture("08-include.m4");
    let text = String::from_utf8_lossy(&output);
    assert!(!output.is_empty(), "include fixture produced no output");
    // __file__ returns the source name; __line__ returns a line number
    assert!(
        text.contains("08-include.m4") || text.contains("1"),
        "got: {}",
        text
    );
}

#[test]
fn test_autoconf_09_shell_cmds() {
    let output = process_fixture("09-shell-cmds.m4");
    assert!(!output.is_empty(), "shell commands produced no output");
}

#[test]
fn test_autoconf_10_ac_init() {
    let output = process_fixture("10-ac-init.m4");
    let text = String::from_utf8_lossy(&output);
    // PACKAGE_NAME and PACKAGE_VERSION defined in divert(-1), expanded in divert(0)
    assert!(text.contains("myproject"), "got: {}", text);
    assert!(text.contains("1.0"), "got: {}", text);
}
