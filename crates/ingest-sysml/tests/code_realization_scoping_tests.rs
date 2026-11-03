//! Integration tests for Issue #472: Deterministic realization package naming and architectural
//! `part def` scoping during Rust (`src/`) and Dart/Flutter (`lib/`) source code ingestion.
//!
//! Realises: [REQ-ISSUE-472-REALIZATION-SCOPING-TESTS]

use compile_sysml::docstring::run_extract_source;
use compile_sysml::parse_sysml;
use deap_core::source_ast::{ingest_source_directory, SourceLanguage};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// RAII temporary workspace guard for Issue #472 realization scoping tests. Realises: [REQ-ISSUE-472-TEMP-DIR-GUARD]
struct ScopedTempDir {
    root: PathBuf,
}

impl ScopedTempDir {
    /// Creates a new isolated temporary workspace directory with the given `label`. Realises: [REQ-ISSUE-472-TEMP-DIR-NEW]
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let root = std::env::temp_dir().join(format!("deap_issue472_{label}_{}_{nanos}_{seq}", std::process::id()));
        let _ = fs::create_dir_all(&root);
        Self { root }
    }
}

impl Drop for ScopedTempDir {
    /// Recursively removes the temporary workspace directory when the guard leaves scope. Realises: [REQ-ISSUE-472-TEMP-DIR-DROP]
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// Verifies parent-qualified `<Parent>_<src|lib>_Realization` package naming and `--package` override. Realises: [REQ-ISSUE-472-PACKAGE-QUALIFICATION]
#[test]
fn test_issue_472_parent_qualified_realization_package_naming_and_override() -> Result<(), Box<dyn std::error::Error>> {
    let ws = ScopedTempDir::new("pkg_naming");
    let solver_src = ws.root.join("crates").join("solver").join("src");
    let flutter_lib = ws.root.join("app_flutter").join("lib");
    let schema_dir = ws.root.join("schema");
    fs::create_dir_all(&solver_src)?;
    fs::create_dir_all(&flutter_lib)?;
    fs::create_dir_all(&schema_dir)?;

    fs::write(
        solver_src.join("lib.rs"),
        "/// Realises: [REQ-SOLVER-01]\npub struct TacticalSolver {\n    pub horizon_sec: u32,\n}\n",
    )?;
    fs::write(
        flutter_lib.join("cockpit_hud.dart"),
        "/// Realises: [REQ-HUD-01]\nclass CockpitHud {\n  final String mode;\n  CockpitHud(this.mode);\n}\n",
    )?;

    let bin = env!("CARGO_BIN_EXE_ingest-sysml");

    let solver_out = schema_dir.join("solver_realization.sysml");
    let solver_digest = schema_dir.join("solver_digest.json");
    let status_solver = Command::new(bin)
        .args(["--source", &solver_src.to_string_lossy(), "--out", &solver_out.to_string_lossy(), "--digest", &solver_digest.to_string_lossy()])
        .output()?;
    assert!(status_solver.status.success(), "{}", String::from_utf8_lossy(&status_solver.stderr));
    let solver_sysml = fs::read_to_string(&solver_out)?;
    let solver_pkg = parse_sysml(&solver_sysml)?;
    assert_eq!(solver_pkg.name, "solver_src_Realization");
    assert!(!solver_sysml.contains("package src "));

    let cockpit_out = schema_dir.join("cockpit_realization.sysml");
    let cockpit_digest = schema_dir.join("cockpit_digest.json");
    let status_cockpit = Command::new(bin)
        .args(["--source", &flutter_lib.to_string_lossy(), "--out", &cockpit_out.to_string_lossy(), "--digest", &cockpit_digest.to_string_lossy()])
        .output()?;
    assert!(status_cockpit.status.success(), "{}", String::from_utf8_lossy(&status_cockpit.stderr));
    let cockpit_sysml = fs::read_to_string(&cockpit_out)?;
    let cockpit_pkg = parse_sysml(&cockpit_sysml)?;
    assert_eq!(cockpit_pkg.name, "app_flutter_lib_Realization");
    assert!(!cockpit_sysml.contains("package lib "));

    let custom_out = schema_dir.join("custom_solver.sysml");
    let custom_digest = schema_dir.join("custom_digest.json");
    let status_custom = Command::new(bin)
        .args(["--source", &solver_src.to_string_lossy(), "--package", "CustomSolverPkg", "--out", &custom_out.to_string_lossy(), "--digest", &custom_digest.to_string_lossy()])
        .output()?;
    assert!(status_custom.status.success(), "{}", String::from_utf8_lossy(&status_custom.stderr));
    let custom_pkg = parse_sysml(&fs::read_to_string(&custom_out)?)?;
    assert_eq!(custom_pkg.name, "CustomSolverPkg");

    let extracted_pkg = run_extract_source(
        &solver_src,
        &schema_dir.join("extract.sysml"),
        &schema_dir.join("extract_digest.json"),
        true,
    )
    .map_err(|(_, msg)| msg)?;
    assert_eq!(extracted_pkg.name, "solver_src_Realization");

    let direct_pkg = ingest_source_directory(&flutter_lib, SourceLanguage::Dart, "lib")?;
    assert_eq!(direct_pkg.name, "app_flutter_lib_Realization");
    Ok(())
}

/// Verifies `--source <DIR> --schema <DOMAIN_SCHEMA>` scopes extracted `PartDef` items to architectural and realized parts. Realises: [REQ-ISSUE-472-SCHEMA-SCOPED-CODE-INGESTION]
#[test]
fn test_issue_472_source_with_reference_schema_scopes_to_architectural_and_realized_parts() -> Result<(), Box<dyn std::error::Error>> {
    let ws = ScopedTempDir::new("schema_scoping");
    let solver_src = ws.root.join("crates").join("solver").join("src");
    let schema_dir = ws.root.join("schema");
    fs::create_dir_all(&solver_src)?;
    fs::create_dir_all(&schema_dir)?;

    let domain_schema_path = schema_dir.join("domain.sysml");
    let domain_sysml = "package TacticalDomain {\n    requirement def REQ_SOLVER_01 {\n        doc /* Real-time track correlation */\n    }\n    part def TacticalSolver;\n    part def CockpitHud;\n}\n";
    fs::write(&domain_schema_path, domain_sysml)?;

    let rust_source = "pub struct TacticalSolver {\n    pub cycle_ms: u32,\n}\n\n/// Realises: [REQ-SOLVER-01]\npub struct RealizedTrackFilter {\n    pub alpha_gain: f64,\n}\n\npub struct InternalScratchBuffer {\n    pub capacity: usize,\n}\n\npub enum PrivateUiHelperEnum {\n    Hidden,\n    Visible,\n}\n";
    fs::write(solver_src.join("lib.rs"), rust_source)?;

    let out_sysml_path = schema_dir.join("solver_realization.sysml");
    let out_digest_path = schema_dir.join("solver_digest.json");

    let bin = env!("CARGO_BIN_EXE_ingest-sysml");
    let output = Command::new(bin)
        .args([
            "--source", &solver_src.to_string_lossy(),
            "--schema", &domain_schema_path.to_string_lossy(),
            "--out", &out_sysml_path.to_string_lossy(),
            "--digest", &out_digest_path.to_string_lossy(),
        ])
        .output()?;
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));

    let pkg = parse_sysml(&fs::read_to_string(&out_sysml_path)?)?;
    let part_names: Vec<&str> = pkg.part_defs.iter().map(|p| p.name.as_str()).collect();
    assert!(part_names.contains(&"TacticalSolver"));
    assert!(part_names.contains(&"RealizedTrackFilter"));
    assert!(!part_names.contains(&"InternalScratchBuffer"));
    assert!(!part_names.contains(&"PrivateUiHelperEnum"));
    Ok(())
}
