// Auto-detect implemented surfaces from Rust source and sync needle metrics.
// This enforces freshness: if code has features the metrics don't know about,
// the metrics are auto-updated. Running `generate` will never produce stale numbers.

/// Sync needle-metrics.json with actual implemented surfaces found in source.
/// Returns true if the metrics were updated.
pub fn sync_needle_metrics() -> Result<bool, String> {
    let metrics_path = "sources/gaps/needle-metrics.json";
    let json = std::fs::read_to_string(metrics_path)
        .map_err(|e| format!("cannot read {}: {}", metrics_path, e))?;
    let mut metrics: serde_json::Value =
        serde_json::from_str(&json).map_err(|e| format!("invalid JSON: {}", e))?;

    // Scan expansion.rs for all builtin handlers
    let builtins = scan_implemented_builtins()?;

    let mut updated = false;

    if let Some(surfaces) = metrics["surfaces"].as_array_mut() {
        for surface in surfaces {
            let id = surface["id"].as_str().unwrap_or("");
            let count = count_builtins_for_surface(id, &builtins);

            let current_impl = surface["implemented"].as_u64().unwrap_or(0) as usize;
            if count > current_impl {
                surface["implemented"] = serde_json::Value::from(count);
                // Adjust missing downward
                let _missing = surface["missing"].as_u64().unwrap_or(0) as usize;
                let total = surface["total_features"].as_u64().unwrap_or(1) as usize;
                let partial = surface["partial"].as_u64().unwrap_or(0) as usize;
                let new_missing = total.saturating_sub(count + partial);
                surface["missing"] = serde_json::Value::from(new_missing);
                updated = true;
            }
        }
    }

    if updated {
        let new_json = serde_json::to_string_pretty(&metrics)
            .map_err(|e| format!("serialization error: {}", e))?;
        std::fs::write(metrics_path, &new_json)
            .map_err(|e| format!("cannot write {}: {}", metrics_path, e))?;
    }

    Ok(updated)
}

/// Scan the expansion engine source for all implemented builtin handlers.
fn scan_implemented_builtins() -> Result<std::collections::HashSet<String>, String> {
    let expansion_path = "crates/m4-rs-core/src/expansion.rs";
    let source = std::fs::read_to_string(expansion_path)
        .map_err(|e| format!("cannot read {}: {}", expansion_path, e))?;

    let mut builtins = std::collections::HashSet::new();
    let mut in_builtin_match = false;

    for line in source.lines() {
        let trimmed = line.trim();
        // Detect the builtin match block
        if trimmed.contains("match name_str.as_str()") {
            in_builtin_match = true;
            continue;
        }
        if !in_builtin_match {
            continue;
        }
        // Extract builtin name from patterns like: "define" => {
        if trimmed.starts_with('"') && trimmed.contains("\"=>") {
            if let Some(name) = trimmed.split('"').nth(1) {
                builtins.insert(name.to_string());
            }
        }
        // Exit when we hit the catch-all at the outer match level (12 spaces indent)
        if in_builtin_match && trimmed.starts_with("_ =>") && line.starts_with("            _ =>") {
            break;
        }
    }

    Ok(builtins)
}

/// Count how many builtins for a given surface are implemented.
fn count_builtins_for_surface(
    surface_id: &str,
    builtins: &std::collections::HashSet<String>,
) -> usize {
    let surface_builtins: &[&str] = match surface_id {
        "M4.DEFINE.1" => &["define", "undefine", "defn"],
        "M4.PUSHDEF.1" => &["pushdef", "popdef", "defn", "builtin", "indir"],
        "M4.QUOTE.1" => &["changequote", "changecom"],
        "M4.COMMENT.1" => &["changecom", "dnl"],
        "M4.BUILTIN.COND.1" => &["ifdef", "ifelse", "shift"],
        "M4.DIVERT.1" => &["divert", "undivert", "divnum"],
        "M4.BUILTIN.TEXT.1" => &["len", "index", "substr", "translit", "regexp", "patsubst"],
        "M4.BUILTIN.EVAL.1" => &["eval", "incr", "decr"],
        "M4.BUILTIN.FORMAT.1" => &["format"],
        "M4.INCLUDE.1" => &["include", "sinclude"],
        "M4.DIAG.1" => &["errprint", "__file__", "__line__", "m4exit", "m4wrap"],
        "M4.SYSCMD.1" => &["syscmd", "esyscmd", "sysval", "maketemp", "mkstemp"],
        "M4.TRACE.1" => &["traceon", "traceoff", "debugmode", "debugfile", "dumpdef"],
        _ => &[],
    };
    surface_builtins
        .iter()
        .filter(|b| builtins.contains(**b))
        .count()
}
