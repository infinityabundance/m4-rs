//! Oracle comparison for the 74 GNU m4 test suite inputs.
//! Run via: cargo run --bin xtask -- gnu-compare
//!
//! Extracts test inputs from crates/m4-rs-core/tests/gnu_test_suite.rs,
//! runs them through both GNU m4 and m4-rs, compares byte-by-byte.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

pub fn run() -> ExitCode {
    println!("=== GNU Test Suite Oracle Comparison ===\n");
    let oracle = find_m4();
    let m4rs = find_m4rs();
    let tmp = std::env::temp_dir().join("m4-rs-gnu-compare");
    std::fs::create_dir_all(&tmp).unwrap();

    let tests = generate_gnu_tests();
    let mut passed = 0u64;
    let mut failed = 0u64;

    for (idx, (name, input, _expected)) in tests.iter().enumerate() {
        let path = tmp.join(format!("{}.m4", name));
        File::create(&path).unwrap().write_all(input).unwrap();

        let oracle_out = run_m4(&oracle, &path);
        let rust_out = run_m4(&m4rs, &path);

        let stdout_match = oracle_out.stdout == rust_out.stdout;
        let exit_match = oracle_out.exit_code == rust_out.exit_code;
        let ok = stdout_match && exit_match;

        if ok {
            passed += 1;
        } else {
            failed += 1;
        }
        let icon = if ok { "✅" } else { "❌" };
        eprintln!(
            "\r  [{}/{}] {} {} (o={} r={})",
            idx + 1,
            tests.len(),
            icon,
            name,
            oracle_out.stdout.len(),
            rust_out.stdout.len()
        );
    }

    println!(
        "\n=== Results: {}/{} passed, {} failed ({:.0}%) ===",
        passed,
        tests.len(),
        failed,
        if !tests.is_empty() {
            passed as f64 / tests.len() as f64 * 100.0
        } else {
            0.0
        }
    );

    if failed > 0 {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

struct ProcOutput {
    stdout: Vec<u8>,
    #[allow(dead_code)]
    stderr: Vec<u8>,
    exit_code: i32,
}

fn run_m4(binary: &Path, fixture: &Path) -> ProcOutput {
    let mut cmd = Command::new(binary);
    cmd.arg(fixture)
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("TZ", "UTC")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    match cmd.spawn() {
        Ok(mut child) => {
            let mut waited = 0u64;
            loop {
                match child.try_wait() {
                    Ok(Some(s)) => {
                        let out = child.wait_with_output().unwrap_or(std::process::Output {
                            status: s,
                            stdout: vec![],
                            stderr: vec![],
                        });
                        return ProcOutput {
                            stdout: out.stdout,
                            stderr: out.stderr,
                            exit_code: out.status.code().unwrap_or(-1),
                        };
                    }
                    Ok(None) => {
                        if waited >= 50 {
                            let _ = child.kill();
                            let _ = child.wait();
                            return ProcOutput {
                                stdout: vec![],
                                stderr: b"TIMEOUT".to_vec(),
                                exit_code: -1,
                            };
                        }
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        waited += 1;
                    }
                    Err(e) => {
                        return ProcOutput {
                            stdout: vec![],
                            stderr: format!("ERROR: {}", e).into_bytes(),
                            exit_code: -1,
                        }
                    }
                }
            }
        }
        Err(e) => ProcOutput {
            stdout: vec![],
            stderr: format!("ERROR: {}", e).into_bytes(),
            exit_code: -1,
        },
    }
}

fn find_m4() -> PathBuf {
    for p in &["/usr/bin/m4", "/usr/local/bin/m4"] {
        if Path::new(p).exists() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from("/usr/bin/m4")
}

fn find_m4rs() -> PathBuf {
    for p in &["target/release/m4-rs", "target/debug/m4-rs"] {
        if Path::new(p).exists() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from("target/debug/m4-rs")
}

use std::fs::File;

/// Generate all 74 GNU test suite inputs as (name, input_bytes, expected_output).
fn generate_gnu_tests() -> Vec<(&'static str, &'static [u8], &'static [u8])> {
    vec![
        ("001-empty", b"", b""),
        ("002-plain-text", b"plain text\n", b"plain text\n"),
        ("003-simple-define", b"define(`x',`hello')x\n", b"hello\n"),
        ("004-define-no-args", b"define(`x')x\n", b"\n"),
        (
            "005-redefine",
            b"define(`x',`first')define(`x',`second')x\n",
            b"second\n",
        ),
        (
            "006-undefine",
            b"define(`x',`hello')x\nundefine(`x')x\n",
            b"hello\nx\n",
        ),
        ("007-dnl", b"line1 dnl line2\nline3\n", b"line1 line3\n"),
        ("008-dnl-no-newline", b"dnl no newline", b""),
        ("009-comment", b"# comment\nvisible\n", b"visible\n"),
        (
            "010-changecom",
            b"changecom(#)hello # comment\nworld\n",
            b"hello \nworld\n",
        ),
        (
            "011-nested-quote",
            b"`outer `inner' outer'\n",
            b"outer `inner' outer\n",
        ),
        (
            "012-changequote",
            b"changequote([,])define([x],[hello])x\n",
            b"hello\n",
        ),
        (
            "013-empty-quote-concat",
            b"define(`foo',`text`'macro')define(`macro',`EXPANDED')foo\n",
            b"textEXPANDED\n",
        ),
        (
            "014-quote-in-name",
            b"define(`foo`'bar',`ok')foo\n",
            b"foo\n",
        ),
        (
            "015-disable-quoting",
            b"changequote()define(x,hello)x\n",
            b"hello\n",
        ),
        (
            "016-quote-whitespace",
            b"define(`x', `hello' `world')x\n",
            b"hello world\n",
        ),
        (
            "020-dollar-at",
            b"define(`x',`$@')x(`a',`b')\n",
            b"`a',`b'\n",
        ),
        (
            "021-dollar-star",
            b"define(`x',`$*')x(`a',`b')\n",
            b"`a',`b'\n",
        ),
        (
            "022-dollar-hash",
            b"define(`x',`$#')x(`a',`b',`c')\n",
            b"3\n",
        ),
        ("023-dollar-zero", b"define(`x',`$0')x\n", b"x\n"),
        (
            "024-too-few-args",
            b"define(`x',`$1:$2:$3')x(`a')\n",
            b"a::\n",
        ),
        (
            "025-too-many-args",
            b"define(`x',`$1')x(`a',`b',`c')\n",
            b"a\n",
        ),
        (
            "030-ifdef-true",
            b"define(`x',`ok')ifdef(`x',`yes',`no')\n",
            b"yes\n",
        ),
        ("031-ifdef-false", b"ifdef(`x',`yes',`no')\n", b"no\n"),
        (
            "032-ifelse-match",
            b"ifelse(`a',`a',`match',`no')\n",
            b"match\n",
        ),
        (
            "033-ifelse-no-match",
            b"ifelse(`a',`b',`match',`no')\n",
            b"no\n",
        ),
        ("040-eval-simple", b"eval(1+2)\n", b"3\n"),
        ("041-eval-comparison", b"eval(3>5)\n", b"0\n"),
        ("042-eval-hex-octal", b"eval(0x10)\neval(010)\n", b"16\n8\n"),
        ("043-incr-decr", b"incr(5)\ndecr(5)\n", b"6\n4\n"),
        ("044-eval-radix", b"eval(255,16)\n", b"ff\n"),
        ("050-len", b"len(`hello')\n", b"5\n"),
        ("051-index", b"index(`hello',`ll')\n", b"2\n"),
        ("052-substr", b"substr(`hello',1,3)\n", b"ell\n"),
        ("053-translit", b"translit(`hello',`el',`ip')\n", b"hippo\n"),
        (
            "060-divert-basic",
            b"divert(1)hidden\ndivert(0)visible\n",
            b"visible\nhidden\n",
        ),
        (
            "061-divert-discard",
            b"divert(-1)discarded\ndivert(0)visible\n",
            b"visible\n",
        ),
        ("062-divnum", b"divnum\n", b"0\n"),
        ("063-undivert-file", b"undivert(`/dev/null')\n", b""),
        (
            "070-sinclude-missing",
            b"sinclude(`nonexistent_xyz')\n",
            b"",
        ),
        ("071-file-line", b"__file__\n__line__\n", b"stdin\n1\n"),
        ("080-sysval", b"sysval\n", b"0\n"),
        ("081-esyscmd", b"esyscmd(`echo -n hello')\n", b"hello"),
        ("090-errprint", b"errprint(`test error')\n", b""),
        ("091-m4exit", b"m4exit(0)\nhello\n", b""),
        (
            "092-m4wrap",
            b"m4wrap(`cleanup')hello\n",
            b"hello\ncleanup\n",
        ),
        (
            "100-pushdef-popdef",
            b"define(`x',`1')pushdef(`x',`2')x popdef(`x') x\n",
            b"2 1\n",
        ),
        (
            "101-defn",
            b"define(`x',`hello')define(`y',defn(`x'))y\n",
            b"hello\n",
        ),
        ("110-format-decimal", b"format(`%d',42)\n", b"42\n"),
        ("111-format-string", b"format(`%s',`hello')\n", b"hello\n"),
        ("120-empty-define", b"define(`x')x\n", b"\n"),
        (
            "121-define-with-dollar",
            b"define(`x',`$1')x(`arg')\n",
            b"arg\n",
        ),
        (
            "122-long-name",
            b"define(`a_very_long_macro_name_indeed',`ok')a_very_long_macro_name_indeed\n",
            b"ok\n",
        ),
        (
            "123-nested-macro-call",
            b"define(`f',`$1')define(`g',`hello')f(g)\n",
            b"hello\n",
        ),
        (
            "130-multiple-macros",
            b"define(`a',`A')define(`b',`B')a b\n",
            b"A B\n",
        ),
        (
            "131-define-within-define",
            b"define(`outer',`define(`inner',`nested')')outer inner\n",
            b" nested\n",
        ),
        (
            "132-shift",
            b"define(`args',`$1:$2:$#')shift(args(`a',`b',`c',`d'))\n",
            b"b:c:3\n",
        ),
        ("133-eval-ternary", b"eval(1?2:3)\n", b"2\n"),
        (
            "134-translit-range",
            b"translit(`abc',`a-c',`A-C')\n",
            b"ABC\n",
        ),
        (
            "135-ifdef-no-else",
            b"define(`x',`ok')ifdef(`x',`yes')\n",
            b"yes\n",
        ),
        ("136-ifelse-single", b"ifelse(`sole-arg')\n", b"\n"),
        (
            "137-undefine-undefined",
            b"undefine(`never_defined')\n",
            b"",
        ),
        ("138-popdef-undefined", b"popdef(`never_defined')\n", b""),
        (
            "139-eval-bitwise",
            b"eval(1|2)\neval(1&3)\neval(~0)\n",
            b"3\n1\n-1\n",
        ),
        ("140-len-empty", b"len()\n", b"0\n"),
        ("141-index-not-found", b"index(`hello',`xx')\n", b"-1\n"),
        ("142-substr-edge", b"substr(`hello',10,2)\n", b"\n"),
        ("143-eval-negative", b"eval(-5)\n", b"-5\n"),
        ("144-eval-division", b"eval(10/3)\n", b"3\n"),
        ("145-eval-div-zero", b"eval(1/0)\n", b""),
        (
            "146-changecom-disable",
            b"changecom()text # not a comment\n",
            b"text # not a comment\n",
        ),
        (
            "147-changequote-disable",
            b"changequote()define(x,hello)x\n",
            b"hello\n",
        ),
        (
            "148-undivert-multiple",
            b"divert(1)A\ndivert(2)B\ndivert(0)undivert(1,2)\n",
            b"AB\n",
        ),
        (
            "149-recursive-via-ifelse",
            b"define(`recurse',`ifelse($1,0,,`$1 recurse(decr($1))')')recurse(3)\n",
            b"3 2 1 \n",
        ),
        (
            "150-ifelse-multibranch",
            b"ifelse(`a',`b',`1',`c',`d',`2',`c',`c',`3',`default')\n",
            b"3\n",
        ),
    ]
}
