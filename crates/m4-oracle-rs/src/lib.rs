// m4-oracle-rs: GNU m4 oracle admission
//
// Locates the system GNU m4 binary, captures its identity fingerprints,
// runs smoke tests, and emits an oracle profile that all subsequent
// parity courts reference.
//
// Clean-room design: we interrogate the m4 binary as a black-box oracle.
// No implementation code is consulted — only binary output is captured.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

/// An admitted oracle — a specific m4 binary with known identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleProfile {
    /// Human label, e.g. "gnu_m4_1_4_20_default"
    pub kind: String,
    /// Path to the executable
    pub path: String,
    /// Raw output of `m4 --version`
    pub version_output: String,
    /// SHA-256 of the executable binary
    pub sha256: String,
    /// Platform triple (e.g. "x86_64-unknown-linux-gnu")
    pub platform: String,
    /// Locale used for admission (e.g. "C")
    pub locale: String,
    /// Shell used for syscmd tests (e.g. "/bin/sh")
    pub shell: String,
    /// OS release info
    pub os_release: String,
    /// Feature flags detected
    pub features: OracleFeatures,
    /// Builtins that the oracle does NOT support
    pub unsupported_builtins: Vec<String>,
    /// Timestamp of admission
    pub admitted_at: String,
    /// Registry of receipts admitted against this oracle
    pub receipt_registry: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OracleFeatures {
    pub posix_mode: bool,
    pub prefix_builtins: bool,
    pub synclines: bool,
    pub changeword: bool,
    pub frozen_files: bool,
    pub debug_flags: Vec<String>,
}

/// Result of running an oracle command.
#[derive(Debug, Clone)]
pub struct OracleRun {
    pub exit_status: ExitStatus,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

/// Configuration for oracle admission.
#[derive(Debug, Clone)]
pub struct OracleConfig {
    /// Path to m4 binary; if None, search PATH
    pub m4_path: Option<PathBuf>,
    /// Locale to use (default "C")
    pub locale: String,
    /// Shell to use for syscmd (default "/bin/sh")
    pub shell: String,
    /// Additional environment variables
    pub env: HashMap<String, String>,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            m4_path: None,
            locale: "C".to_string(),
            shell: "/bin/sh".to_string(),
            env: HashMap::new(),
        }
    }
}

/// Locate the GNU m4 binary on the system.
///
/// Searches:
/// 1. Explicit path from config
/// 2. `m4` on PATH
/// 3. `/usr/bin/m4`
/// 4. `/usr/local/bin/m4`
pub fn locate_m4(config: &OracleConfig) -> Result<PathBuf, OracleError> {
    if let Some(ref path) = config.m4_path {
        if path.exists() {
            return Ok(path.clone());
        }
        return Err(OracleError::NotFound(format!(
            "explicit path not found: {}",
            path.display()
        )));
    }

    for candidate in &["m4", "/usr/bin/m4", "/usr/local/bin/m4"] {
        let path = Path::new(candidate);
        if let Ok(resolved) = which::which(path) {
            return Ok(resolved);
        }
        if path.exists() {
            return Ok(path.to_path_buf());
        }
    }

    // Try `which m4` directly
    if let Ok(path) = which::which(Path::new("m4")) {
        return Ok(path);
    }

    Err(OracleError::NotFound(
        "GNU m4 not found on PATH. Install m4 or set m4_path.".into(),
    ))
}

/// Run the oracle with given stdin and arguments.
pub fn run_oracle(
    oracle_path: &Path,
    args: &[&str],
    stdin: &[u8],
    working_dir: Option<&Path>,
    env: &HashMap<String, String>,
) -> io::Result<OracleRun> {
    let mut cmd = Command::new(oracle_path);
    cmd.args(args);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.env_clear();

    // Set controlled environment
    cmd.env("PATH", "/usr/bin:/bin:/usr/local/bin");
    cmd.env("LC_ALL", "C");
    cmd.env("LANG", "C");
    for (k, v) in env {
        cmd.env(k, v);
    }

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let mut child = cmd.spawn()?;

    // Write stdin
    if let Some(mut sin) = child.stdin.take() {
        sin.write_all(stdin)?;
        // stdin is dropped here, closing the pipe
    }

    let output = child.wait_with_output()?;

    Ok(OracleRun {
        exit_status: output.status,
        exit_code: output.status.code(),
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

/// Run the oracle with stdin from a string and capture output as strings.
pub fn run_oracle_text(
    oracle_path: &Path,
    args: &[&str],
    stdin: &str,
    working_dir: Option<&Path>,
    env: &HashMap<String, String>,
) -> io::Result<OracleRun> {
    run_oracle(oracle_path, args, stdin.as_bytes(), working_dir, env)
}

/// Admit a GNU m4 binary as the oracle.
///
/// This function:
/// 1. Locates the m4 binary
/// 2. Captures --version output
/// 3. Computes SHA-256 of the binary
/// 4. Detects features
/// 5. Runs a smoke test
/// 6. Builds the OracleProfile
pub fn admit_oracle(config: &OracleConfig) -> Result<OracleProfile, OracleError> {
    let path = locate_m4(config)?;
    let abs_path = std::fs::canonicalize(&path).map_err(|e| OracleError::Io(e.to_string()))?;

    // 1. Capture version
    let version_run = run_oracle_text(&abs_path, &["--version"], "", None, &config.env)
        .map_err(|e| OracleError::Execution(format!("version check: {}", e)))?;
    let version_output = String::from_utf8_lossy(&version_run.stdout).to_string();

    if !version_output.contains("GNU M4") && !version_output.contains("GNU m4") {
        return Err(OracleError::NotGnuM4(format!(
            "binary at {} does not identify as GNU m4:\n{}",
            abs_path.display(),
            version_output
        )));
    }

    // 2. Compute sha256
    let binary_bytes = std::fs::read(&abs_path).map_err(|e| OracleError::Io(e.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(&binary_bytes);
    let sha256 = format!("{:x}", hasher.finalize());

    // 3. Detect features
    let features = detect_features(&abs_path, &config.env)?;

    // 4. Platform info
    let platform = std::env::consts::OS.to_string() + "-" + std::env::consts::ARCH;
    let os_release = read_os_release();

    // 5. Run smoke test: copy-through of plain text
    let smoke_run = run_oracle_text(&abs_path, &[], "hello world\n", None, &config.env)
        .map_err(|e| OracleError::Execution(format!("smoke test: {}", e)))?;

    if smoke_run.stdout != b"hello world\n" {
        return Err(OracleError::SmokeFailure(format!(
            "smoke copy-through failed: expected 'hello world\\n', got {:?}",
            String::from_utf8_lossy(&smoke_run.stdout)
        )));
    }

    // 6. Build profile
    let mut profile = OracleProfile {
        kind: extract_profile_kind(&version_output),
        path: abs_path.to_string_lossy().to_string(),
        version_output,
        sha256,
        platform,
        locale: config.locale.clone(),
        shell: config.shell.clone(),
        os_release,
        features,
        unsupported_builtins: detect_unsupported_builtins(&abs_path, &config.env)?,
        admitted_at: chrono_now(),
        receipt_registry: vec![],
    };

    // Mark receipt
    profile.receipt_registry.push("M4.ORACLE.1".to_string());

    Ok(profile)
}

/// Extract the profile kind from version output.
///
/// e.g. "GNU M4 1.4.20" → "gnu_m4_1_4_20_default"
fn extract_profile_kind(version_output: &str) -> String {
    // Parse version number
    if let Some(line) = version_output.lines().next() {
        // "GNU M4 1.4.20" or "m4 (GNU M4) 1.4.20"
        let parts: Vec<&str> = line.split_whitespace().collect();
        for (i, part) in parts.iter().enumerate() {
            if part
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                let version = part.trim_end_matches(',');
                return format!("gnu_m4_{}_default", version.replace('.', "_"));
            }
            // Handle parenthesized version like "m4 (GNU M4) 1.4.20"
            if *part == "M4)" && i + 1 < parts.len() {
                let version = parts[i + 1].trim_end_matches(',');
                return format!("gnu_m4_{}_default", version.replace('.', "_"));
            }
        }
    }
    "gnu_m4_unknown_default".to_string()
}

/// Detect oracle features through black-box interrogation.
fn detect_features(
    path: &Path,
    env: &HashMap<String, String>,
) -> Result<OracleFeatures, OracleError> {
    let mut features = OracleFeatures::default();

    // Check --help for supported flags
    let help_run = run_oracle_text(path, &["--help"], "", None, env)
        .map_err(|e| OracleError::Execution(format!("help check: {}", e)))?;
    let help = String::from_utf8_lossy(&help_run.stdout);

    features.prefix_builtins = help.contains("--prefix-builtins") || help.contains("-P");
    features.synclines = help.contains("--synclines") || help.contains("-s");
    features.frozen_files = help.contains("-F") || help.contains("--freeze-state");
    features.changeword = help.contains("changeword");

    // Detect debug flags
    if help.contains("-d") || help.contains("--debug") {
        for flag in &["a", "c", "e", "f", "l", "q", "t", "x", "V"] {
            // Test if -d<flag> is accepted
            let test_run = run_oracle_text(path, &[&format!("-d{}", flag)], "dnl", None, env);
            if let Ok(run) = test_run {
                if run.exit_code == Some(0) {
                    features.debug_flags.push(flag.to_string());
                }
            }
        }
    }

    Ok(features)
}

/// Detect which builtins are unsupported by this oracle.
fn detect_unsupported_builtins(
    path: &Path,
    env: &HashMap<String, String>,
) -> Result<Vec<String>, OracleError> {
    let unsupported = vec![];

    // Check changeword
    let cw = run_oracle_text(path, &[], "changeword(`[a-z]+')\ndnl\n", None, env)
        .map_err(|_| OracleError::Execution("changeword check".into()))?;

    // changeword is often unsupported; if it produces an error or warning
    // mentioning "disabled", mark it unsupported
    let stderr = String::from_utf8_lossy(&cw.stderr);
    let mut result = unsupported;
    if stderr.contains("disabled") || stderr.contains("not supported") {
        result.push("changeword".to_string());
    }

    Ok(result)
}

fn read_os_release() -> String {
    if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
        for line in contents.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line
                    .trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            }
        }
    }
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Get a UTC timestamp string without pulling in chrono.
fn chrono_now() -> String {
    // Simple approach: use UNIX timestamp
    use std::time::SystemTime;
    let dur = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

/// Save the oracle profile to a JSON file.
pub fn save_profile(profile: &OracleProfile, path: &Path) -> io::Result<()> {
    let json = serde_json::to_string_pretty(profile)?;
    std::fs::write(path, json)
}

/// Load an oracle profile from a JSON file.
pub fn load_profile(path: &Path) -> io::Result<OracleProfile> {
    let json = std::fs::read_to_string(path)?;
    let profile: OracleProfile = serde_json::from_str(&json)?;
    Ok(profile)
}

/// Errors that can occur during oracle operations.
#[derive(Debug)]
pub enum OracleError {
    NotFound(String),
    NotGnuM4(String),
    Execution(String),
    SmokeFailure(String),
    Io(String),
}

impl std::fmt::Display for OracleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OracleError::NotFound(s) => write!(f, "oracle not found: {}", s),
            OracleError::NotGnuM4(s) => write!(f, "not GNU m4: {}", s),
            OracleError::Execution(s) => write!(f, "execution error: {}", s),
            OracleError::SmokeFailure(s) => write!(f, "smoke test failed: {}", s),
            OracleError::Io(s) => write!(f, "I/O error: {}", s),
        }
    }
}

impl std::error::Error for OracleError {}

pub mod behavior_doc;
pub mod multi_oracle;

// We need `which` for binary location. Use a minimal inline implementation
// to avoid pulling in the `which` crate.
mod which {
    use std::path::{Path, PathBuf};

    pub fn which(cmd: &Path) -> Result<PathBuf, ()> {
        let path_var = std::env::var("PATH").unwrap_or_default();
        for dir in path_var.split(':') {
            let candidate = Path::new(dir).join(cmd);
            if candidate.exists() && is_executable(&candidate) {
                return Ok(candidate);
            }
        }
        Err(())
    }

    fn is_executable(path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let perms = meta.permissions();
            perms.mode() & 0o111 != 0
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_profile_kind() {
        assert_eq!(
            extract_profile_kind("GNU M4 1.4.20\n"),
            "gnu_m4_1_4_20_default"
        );
        assert_eq!(
            extract_profile_kind("m4 (GNU M4) 1.4.19\n"),
            "gnu_m4_1_4_19_default"
        );
    }

    #[test]
    fn test_locate_m4() {
        let config = OracleConfig::default();
        match locate_m4(&config) {
            Ok(path) => {
                eprintln!("Found m4 at: {}", path.display());
                assert!(path.exists());
            }
            Err(OracleError::NotFound(_)) => {
                eprintln!("m4 not found — skipping locate test (CI/container may not have m4)");
            }
            Err(e) => panic!("unexpected error: {}", e),
        }
    }

    #[test]
    fn test_admit_oracle() {
        let config = OracleConfig::default();
        match admit_oracle(&config) {
            Ok(profile) => {
                eprintln!("Admitted oracle: {:?}", profile.kind);
                eprintln!("  path: {}", profile.path);
                eprintln!("  sha256: {}", profile.sha256);
                eprintln!("  platform: {}", profile.platform);
                assert!(profile.version_output.contains("GNU"));
            }
            Err(OracleError::NotFound(_)) => {
                eprintln!("m4 not found — skipping admission test");
            }
            Err(e) => panic!("admission failed: {}", e),
        }
    }
}
