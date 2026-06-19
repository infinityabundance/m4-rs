// m4-rs frozen file oracle smoke tests.
//
// These tests verify that m4-rs's frozen file (-F/-R) behavior matches
// GNU m4's behavior byte-for-byte. Each test:
// 1. Creates an input file
// 2. Runs GNU m4 with -F to freeze state
// 3. Runs GNU m4 with -R to reload and expand a test expression
// 4. Runs m4-rs with the same -F/-R pipeline
// 5. Compares stdout and exit codes
//
// M4.FROZEN.1 — Frozen file save/reload parity court.

use std::path::PathBuf;
use std::process::Command;

/// Locate the GNU m4 oracle binary.
fn oracle_m4() -> PathBuf {
    for p in &["/usr/bin/m4", "/usr/local/bin/m4"] {
        if std::path::Path::new(p).exists() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from("/usr/bin/m4")
}

/// Locate the m4-rs release binary (workspace root relative).
fn m4rs_binary() -> PathBuf {
    for p in &[
        "target/release/m4-rs",
        "target/debug/m4-rs",
        "../../target/release/m4-rs",
        "../../target/debug/m4-rs",
    ] {
        if std::path::Path::new(p).exists() {
            return PathBuf::from(p);
        }
    }
    PathBuf::from("../../target/debug/m4-rs")
}

// ════════════════════════════════════════════════════════════════════
// M4.FROZEN.1.01 — Basic save/reload: define then reload
// ════════════════════════════════════════════════════════════════════
#[test]
fn frozen_01_basic_save_reload() {
    let m4 = oracle_m4();
    let m4rs = m4rs_binary();
    let dir = std::env::temp_dir().join("m4_frozen_test");
    std::fs::create_dir_all(&dir).ok();

    let input_path = dir.join("frozen_01_input.m4");
    let freeze_path = dir.join("frozen_01_state.m4f");
    std::fs::write(&input_path, b"define(`hello', `world')dnl\n").unwrap();

    // Freeze with GNU m4
    let _ = Command::new(&m4)
        .args([
            "-F",
            &format!("{}", freeze_path.display()),
            &format!("{}", input_path.display()),
        ])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    // Reload and expand `hello` with both oracles
    let gnu = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo hello | {} -R {}",
            m4.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    let rs = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo hello | {} -R {}",
            m4rs.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert_eq!(
        gnu.stdout,
        rs.stdout,
        "stdout mismatch:\nGNU: {:?}\nm4-rs: {:?}",
        String::from_utf8_lossy(&gnu.stdout),
        String::from_utf8_lossy(&rs.stdout)
    );
    assert_eq!(
        gnu.status.code(),
        rs.status.code(),
        "exit code: gnu={:?} rs={:?}",
        gnu.status.code(),
        rs.status.code()
    );

    let _ = std::fs::remove_file(&freeze_path);
    let _ = std::fs::remove_file(&input_path);
}

// ════════════════════════════════════════════════════════════════════
// M4.FROZEN.1.02 — Freeze multiple macros, reload and expand all
// ════════════════════════════════════════════════════════════════════
#[test]
fn frozen_02_multiple_macros() {
    let m4 = oracle_m4();
    let m4rs = m4rs_binary();
    let dir = std::env::temp_dir().join("m4_frozen_test");
    std::fs::create_dir_all(&dir).ok();

    let input_path = dir.join("frozen_02_input.m4");
    let freeze_path = dir.join("frozen_02_state.m4f");
    std::fs::write(
        &input_path,
        b"define(`a', `1')define(`b', `2')define(`c', `3')dnl\n",
    )
    .unwrap();

    let _ = Command::new(&m4)
        .args([
            "-F",
            &format!("{}", freeze_path.display()),
            &format!("{}", input_path.display()),
        ])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    let gnu = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "printf 'a b c\\n' | {} -R {}",
            m4.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    let rs = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "printf 'a b c\\n' | {} -R {}",
            m4rs.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert_eq!(
        gnu.stdout,
        rs.stdout,
        "stdout: gnu={:?} rs={:?}",
        String::from_utf8_lossy(&gnu.stdout),
        String::from_utf8_lossy(&rs.stdout)
    );

    let _ = std::fs::remove_file(&freeze_path);
    let _ = std::fs::remove_file(&input_path);
}

// ════════════════════════════════════════════════════════════════════
// M4.FROZEN.1.03 — pushdef stack roundtrip
// ════════════════════════════════════════════════════════════════════
#[test]
fn frozen_03_pushdef_stack() {
    let m4 = oracle_m4();
    let m4rs = m4rs_binary();
    let dir = std::env::temp_dir().join("m4_frozen_test");
    std::fs::create_dir_all(&dir).ok();

    let input_path = dir.join("frozen_03_input.m4");
    let freeze_path = dir.join("frozen_03_state.m4f");
    std::fs::write(
        &input_path,
        b"pushdef(`x', `outer')pushdef(`x', `inner')dnl\n",
    )
    .unwrap();

    let _ = Command::new(&m4)
        .args([
            "-F",
            &format!("{}", freeze_path.display()),
            &format!("{}", input_path.display()),
        ])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    // Reload: expand x ("inner"), popdef, expand x again ("outer")
    let gnu = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "printf 'x popdef(`x') x\\n' | {} -R {}",
            m4.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    let rs = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "printf 'x popdef(`x') x\\n' | {} -R {}",
            m4rs.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert_eq!(
        gnu.stdout,
        rs.stdout,
        "pushdef stack: gnu={:?} rs={:?}",
        String::from_utf8_lossy(&gnu.stdout),
        String::from_utf8_lossy(&rs.stdout)
    );

    let _ = std::fs::remove_file(&freeze_path);
    let _ = std::fs::remove_file(&input_path);
}

// ════════════════════════════════════════════════════════════════════
// M4.FROZEN.1.04 — Quote change roundtrip
// ════════════════════════════════════════════════════════════════════
#[test]
fn frozen_04_quote_change() {
    let m4 = oracle_m4();
    let m4rs = m4rs_binary();
    let dir = std::env::temp_dir().join("m4_frozen_test");
    std::fs::create_dir_all(&dir).ok();

    let input_path = dir.join("frozen_04_input.m4");
    let freeze_path = dir.join("frozen_04_state.m4f");
    std::fs::write(
        &input_path,
        b"changequote([, ])define([hello], [world])dnl\n",
    )
    .unwrap();

    let _ = Command::new(&m4)
        .args([
            "-F",
            &format!("{}", freeze_path.display()),
            &format!("{}", input_path.display()),
        ])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    let gnu = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "printf '[hello]\\n' | {} -R {}",
            m4.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    let rs = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "printf '[hello]\\n' | {} -R {}",
            m4rs.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert_eq!(
        gnu.stdout,
        rs.stdout,
        "quote change: gnu={:?} rs={:?}",
        String::from_utf8_lossy(&gnu.stdout),
        String::from_utf8_lossy(&rs.stdout)
    );

    let _ = std::fs::remove_file(&freeze_path);
    let _ = std::fs::remove_file(&input_path);
}

// ════════════════════════════════════════════════════════════════════
// M4.FROZEN.1.05 — Diversion save/reload
// ════════════════════════════════════════════════════════════════════
#[test]
fn frozen_05_diversion_save() {
    let m4 = oracle_m4();
    let m4rs = m4rs_binary();
    let dir = std::env::temp_dir().join("m4_frozen_test");
    std::fs::create_dir_all(&dir).ok();

    let input_path = dir.join("frozen_05_input.m4");
    let freeze_path = dir.join("frozen_05_state.m4f");
    // divert(0) must be on its own line to take effect (not inside diversion scope)
    std::fs::write(
        &input_path,
        b"divert(2)hidden data here\ndivert(0)visible\n",
    )
    .unwrap();

    let _ = Command::new(&m4)
        .args([
            "-F",
            &format!("{}", freeze_path.display()),
            &format!("{}", input_path.display()),
        ])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    // Reload — diversion data preserved, undiverted at EOF
    let gnu = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "{} -R {} < /dev/null",
            m4.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    let rs = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "{} -R {} < /dev/null",
            m4rs.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert_eq!(
        gnu.stdout,
        rs.stdout,
        "diversion reload: gnu={:?} rs={:?} rs.stderr={:?}",
        String::from_utf8_lossy(&gnu.stdout),
        String::from_utf8_lossy(&rs.stdout),
        String::from_utf8_lossy(&rs.stderr)
    );

    let _ = std::fs::remove_file(&freeze_path);
    let _ = std::fs::remove_file(&input_path);
}

// ════════════════════════════════════════════════════════════════════
// M4.FROZEN.1.06 — Version mismatch exit code (63)
// ════════════════════════════════════════════════════════════════════
#[test]
fn frozen_06_version_mismatch_exit_63() {
    let m4rs = m4rs_binary();
    let dir = std::env::temp_dir().join("m4_frozen_test");
    std::fs::create_dir_all(&dir).ok();

    let bad_freeze = dir.join("frozen_06_badver.m4f");
    std::fs::write(&bad_freeze, b"V9\n").unwrap();

    let output = Command::new(&m4rs)
        .args(["-R", &format!("{}", bad_freeze.display())])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .unwrap();

    assert_eq!(
        output.status.code(),
        Some(63),
        "m4-rs exit 63 on version mismatch, got {:?}. stderr: {:?}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify GNU m4 also exits 63
    let m4 = oracle_m4();
    let gnu = Command::new(&m4)
        .args(["-R", &format!("{}", bad_freeze.display())])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert_eq!(
        gnu.status.code(),
        Some(63),
        "GNU m4 exit 63, got {:?}",
        gnu.status.code()
    );

    let _ = std::fs::remove_file(&bad_freeze);
}

// ════════════════════════════════════════════════════════════════════
// M4.FROZEN.1.07 — -F flag writes valid freeze file readable by GNU m4
// ════════════════════════════════════════════════════════════════════
#[test]
fn frozen_07_freeze_file_readable_by_gnu() {
    let m4rs = m4rs_binary();
    let m4 = oracle_m4();
    let dir = std::env::temp_dir().join("m4_frozen_test");
    std::fs::create_dir_all(&dir).ok();

    let input_path = dir.join("frozen_07_input.m4");
    let freeze_path = dir.join("frozen_07_state.m4f");
    std::fs::write(&input_path, b"define(`x', `ok')dnl\n").unwrap();

    // Freeze with m4-rs
    let output = Command::new(&m4rs)
        .args([
            "-F",
            &format!("{}", freeze_path.display()),
            &format!("{}", input_path.display()),
        ])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "m4-rs freeze failed: {:?}",
        output.status
    );
    assert!(freeze_path.exists(), "freeze file should exist");

    // Now load it with GNU m4 — should accept the frozen file
    let gnu = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo x | {} -R {}",
            m4.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert!(
        gnu.status.success(),
        "GNU m4 should accept m4-rs frozen file. stderr: {:?}",
        String::from_utf8_lossy(&gnu.stderr)
    );
    assert_eq!(
        gnu.stdout,
        b"ok\n",
        "GNU m4 should expand x=ok from m4-rs frozen file. Got: {:?}",
        String::from_utf8_lossy(&gnu.stdout)
    );

    let _ = std::fs::remove_file(&freeze_path);
    let _ = std::fs::remove_file(&input_path);
}

// ════════════════════════════════════════════════════════════════════
// M4.FROZEN.1.08 — m4-rs loads GNU m4 frozen file
// ════════════════════════════════════════════════════════════════════
#[test]
fn frozen_08_load_gnu_frozen_file() {
    let m4rs = m4rs_binary();
    let m4 = oracle_m4();
    let dir = std::env::temp_dir().join("m4_frozen_test");
    std::fs::create_dir_all(&dir).ok();

    let input_path = dir.join("frozen_08_input.m4");
    let freeze_path = dir.join("frozen_08_state.m4f");
    std::fs::write(&input_path, b"define(`hello', `gnu_world')dnl\n").unwrap();

    // Freeze with GNU m4
    let _ = Command::new(&m4)
        .args([
            "-F",
            &format!("{}", freeze_path.display()),
            &format!("{}", input_path.display()),
        ])
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    // Load with m4-rs
    let rs = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "echo hello | {} -R {}",
            m4rs.display(),
            freeze_path.display()
        ))
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null())
        .output()
        .unwrap();

    assert!(
        rs.status.success(),
        "m4-rs should accept GNU frozen file. stderr: {:?}",
        String::from_utf8_lossy(&rs.stderr)
    );
    assert_eq!(
        rs.stdout,
        b"gnu_world\n",
        "m4-rs should expand hello=gnu_world from GNU frozen file. Got: {:?}",
        String::from_utf8_lossy(&rs.stdout)
    );

    let _ = std::fs::remove_file(&freeze_path);
    let _ = std::fs::remove_file(&input_path);
}
