//! STPA Safety Transpilation and Hazard Analysis Module.

pub mod fmeca;
pub mod safety_suite;
pub mod uca;

pub use fmeca::*;
pub use safety_suite::*;
pub use uca::*;

use std::fs;
use std::path::Path;

/// High-level entrypoint to transpile a SysML v2 model file into the 10-pillar safety artifact suite.
pub fn transpile_stpa(
    schema_path: &Path,
    out_dir: &Path,
    scoring_config_path: Option<&Path>,
) -> Result<(), String> {
    if !schema_path.exists() {
        return Err(format!("Schema file does not exist: {}", schema_path.display()));
    }

    let content = fs::read_to_string(schema_path)
        .map_err(|e| format!("Failed to read schema file '{}': {}", schema_path.display(), e))?;

    if content.trim().is_empty() {
        return Err(format!("Schema file '{}' is empty.", schema_path.display()));
    }

    let pkg = crate::parse_sysml(&content)
        .map_err(|e| format!("Failed to parse schema '{}': {}", schema_path.display(), e))?;

    let scoring_config = if let Some(cfg_path) = scoring_config_path {
        if !cfg_path.exists() {
            return Err(format!(
                "FMECA scoring config does not exist: {}",
                cfg_path.display()
            ));
        }
        let cfg_text = fs::read_to_string(cfg_path)
            .map_err(|e| format!("Failed to read scoring config: {}", e))?;
        let parsed: FmecaScoringConfig = serde_json::from_str(&cfg_text)
            .map_err(|e| format!("Failed to parse scoring config JSON: {}", e))?;
        Some(parsed)
    } else {
        None
    };

    emit_safety_suite(&pkg, out_dir, scoring_config.as_ref())?;

    println!(
        "[STPA Transpile] Emitted safety artifact suite to '{}'",
        out_dir.display()
    );

    Ok(())
}
