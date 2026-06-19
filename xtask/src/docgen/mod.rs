// xtask/docgen: Document generation engine.
//
// All project documents are generated from JSON sources by xtask.
// No document is hand-authored; the JSON is the machine source of truth.
// Every generated document is freshness-gated and DSSE-signed.
//
// Architecture:
//   sources/*.json  ──[generate]──>  reports/  +  docs/
//   (source of truth)               (generated, freshness-gated)
//
// The check command verifies that generated documents are fresh
// (not stale relative to their JSON sources).

pub mod dsse;
pub mod generate;
pub mod sync_metrics;

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Metadata for a generated document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMeta {
    /// Source JSON file(s) that this document was generated from.
    pub sources: Vec<String>,
    /// SHA256 of each source file at generation time (same order as sources).
    pub source_sha256s: Vec<String>,
    /// Output file path.
    pub output: String,
    /// Generation timestamp (epoch seconds).
    pub generated_at: u64,
    /// DSSE signature envelope (base64-encoded).
    pub dsse_signature: Option<String>,
    /// Schema version of the source.
    pub source_schema: String,
}

/// Registry of all generated documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRegistry {
    pub schema: String,
    pub documents: Vec<DocumentMeta>,
}

impl DocumentRegistry {
    pub fn new() -> Self {
        Self {
            schema: "m4-rs-doc-registry-v1".to_string(),
            documents: Vec::new(),
        }
    }

    pub fn register(&mut self, meta: DocumentMeta) {
        self.documents.push(meta);
    }

    /// Check if a document is stale by comparing current source SHA256
    /// against the SHA256 recorded at generation time.
    /// SHA256 is more reliable than mtime (unaffected by git checkout, fs quirks).
    pub fn is_stale(&self, meta: &DocumentMeta) -> Result<bool, String> {
        if meta.source_sha256s.is_empty() {
            // Legacy registry entry without SHA256 — fall back to mtime
            return self.is_stale_mtime(meta);
        }
        for (i, source) in meta.sources.iter().enumerate() {
            let source_path = Path::new(source);
            if !source_path.exists() {
                return Err(format!("source not found: {}", source));
            }
            let current_sha = crate::docgen::sha256_file(source_path)?;
            let recorded_sha = meta.source_sha256s.get(i).cloned().unwrap_or_default();
            if current_sha != recorded_sha {
                return Ok(true); // stale — source content changed
            }
        }
        Ok(false) // fresh
    }

    /// Legacy mtime-based staleness check (fallback for old registries).
    fn is_stale_mtime(&self, meta: &DocumentMeta) -> Result<bool, String> {
        for source in &meta.sources {
            let source_path = Path::new(source);
            if !source_path.exists() {
                return Err(format!("source not found: {}", source));
            }
            let source_mtime = source_path
                .metadata()
                .map_err(|e| format!("cannot stat {}: {}", source, e))?
                .modified()
                .map_err(|e| format!("cannot get mtime for {}: {}", source, e))?;
            let source_epoch = source_mtime
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| format!("time error: {}", e))?
                .as_secs();
            if source_epoch > meta.generated_at {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Verify freshness of all registered documents using SHA256 comparison.
    pub fn verify_freshness(&self) -> Result<Vec<String>, Vec<String>> {
        let mut stale = Vec::new();
        for doc in &self.documents {
            match self.is_stale(doc) {
                Ok(true) => stale.push(format!(
                    "STALE: {} — source content changed since generation. Run 'cargo xtask generate'.",
                    doc.output
                )),
                Ok(false) => {} // fresh
                Err(e) => stale.push(format!("ERROR: {} — {}", doc.output, e)),
            }
        }
        if stale.is_empty() {
            Ok(vec!["All documents fresh (SHA256 verified).".to_string()])
        } else {
            Err(stale)
        }
    }
}

/// Generate a timestamp for now.
pub fn now_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Compute SHA256 of file contents.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let data = std::fs::read(path).map_err(|e| format!("cannot read {}: {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute SHA256 of a string (reserved for future DSSE payload signing).
#[allow(dead_code)]
pub fn sha256_str(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}
