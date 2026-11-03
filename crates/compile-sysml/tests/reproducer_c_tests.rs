//! Reproducer test helpers for GitHub Issue #693 (`compile-sysml` FMECA & STPA subworkspace scoping).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_c_tests.rs` (refs #693).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure software AST declarations never synthesize physical hardware/avionics prose or falsified
//!   `SSOT` provenance when `FAILURE_MODE_REGISTRY.md` has no entry.
//! - All AST ports, states, actions, operations, and attributes are enumerated across all 4
//!   failure dimensions without `.first()` truncation, and empty-scale `FmecaScoringConfig::default()`
//!   resolves `Interface` row scores to `(8, 3, 2)` (`RPN == 48`).
//! - `--schema .pipeline/upstream --stpa-transpile` scopes safety output to
//!   `<subworkspace>/docs/safety` and ingests `<subworkspace>/docs/research/FAILURE_MODE_REGISTRY.md`.
//!
//! ## 3. Preconditions
//! - `std::env::temp_dir()` is writable and `CARGO_BIN_EXE_compile-sysml` is available.
//!
//! ## 4. Postconditions
//! - All temporary test directories are removed deterministically via `TempDirGuard`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(M)$ over the AST member count $M$.
//! - Space Complexity: $O(M)$ bounded auxiliary memory for FMECA rows and safety artifacts.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates both direct `generate_fmeca_matrix` calls and CLI `--stpa-transpile` execution.
//! - State: Isolated per-test temporary workspaces via `TempDirGuard`.
//! - Action: Executes with zero panics or `.unwrap()` calls.
//! - Resource: Reclaims ephemeral files on scope exit.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-STPA-FMECA-DOMAIN-NEUTRAL], [REQ-STPA-SUBWORKSPACE-OUTDIR] | SysML v2: VerificationCaseDef(reproducer_c_tests)

use std::fs;
use std::process::Command;

use compile_sysml::projections::stpa::fmeca::FmecaRow;
use compile_sysml::FmecaScoringConfig;
use deap_core::sysml_cst::{
    ActionDef, AttributeDef, OperationDef, PackageDef, PartDef, PortDef, StateDef,
};

use crate::reproducer_tests::TempDirGuard;

fn build_issue_693_query_engine_pkg() -> PackageDef {
    let planner = PartDef {
        name: "QueryPlanner".to_string(),
        ports: vec![
            PortDef {
                name: "sqlIngress".to_string(),
                ..Default::default()
            },
            PortDef {
                name: "planEgress".to_string(),
                ..Default::default()
            },
        ],
        states: vec![
            StateDef {
                name: "Idle".to_string(),
                ..Default::default()
            },
            StateDef {
                name: "Optimizing".to_string(),
                ..Default::default()
            },
        ],
        actions: vec![
            ActionDef {
                name: "parseAst".to_string(),
                ..Default::default()
            },
            ActionDef {
                name: "emitPlan".to_string(),
                ..Default::default()
            },
        ],
        operations: vec![OperationDef {
            name: "validateBindings".to_string(),
            ..Default::default()
        }],
        attributes: vec![
            AttributeDef {
                name: "maxRetries".to_string(),
                ..Default::default()
            },
            AttributeDef {
                name: "cacheLimitMb".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    PackageDef {
        name: "QueryEnginePkg".to_string(),
        part_defs: vec![planner],
        ..Default::default()
    }
}

fn assert_issue_693_fmeca_coverage_and_scores(rows: &[FmecaRow]) {
    assert!(
        rows.len() >= 15,
        "Full [0..] enumeration across 9 AST members must yield at least 15 FMECA rows, got {}: {:?}",
        rows.len(),
        rows
    );
    for expected_dim in ["Interface", "State", "Action", "Resource"] {
        assert!(
            rows.iter().any(|r| r.dimension == expected_dim),
            "FMECA rows must cover dimension {:?}, got: {:?}",
            expected_dim,
            rows
        );
    }
    for expected_member in [
        "sqlIngress",
        "planEgress",
        "Idle",
        "Optimizing",
        "parseAst",
        "emitPlan",
        "validateBindings",
        "maxRetries",
        "cacheLimitMb",
    ] {
        assert!(
            rows.iter().any(|r| {
                r.failure_mode.contains(expected_member)
                    || r.effect.contains(expected_member)
                    || r.mitigation.contains(expected_member)
            }),
            "FMECA rows must enumerate AST member {:?} without .first() truncation, got: {:?}",
            expected_member,
            rows
        );
    }

    let interface_rows: Vec<_> = rows.iter().filter(|r| r.dimension == "Interface").collect();
    assert!(
        !interface_rows.is_empty(),
        "Expected non-empty Interface FMECA rows for sqlIngress and planEgress"
    );
    for row in &interface_rows {
        assert_ne!(
            (row.severity, row.occurrence, row.detection),
            (8, 8, 8),
            "Empty-scale FmecaScoringConfig::default() must not collapse Interface scores to (8, 8, 8) / RPN 512: {:?}",
            row
        );
        assert_ne!(
            row.rpn, 512,
            "Interface row RPN must not be inflated to 512 by resolve_scores bug: {:?}",
            row
        );
    }
    assert!(
        interface_rows
            .iter()
            .any(|r| (r.severity, r.occurrence, r.detection) == (8, 3, 2) && r.rpn == 48),
        "Empty-scale FmecaScoringConfig::default() must resolve Interface row scores to (8, 3, 2) with rpn == 48, got: {:?}",
        interface_rows
    );
}

/// Executes Issue #693 checks for domain-neutral FMECA synthesis, `[0..]` coverage, and score fallback.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when all FMECA software AST invariants hold.
///
/// # Complexity
/// * Time: $O(M)$ over the 9 AST members of `QueryPlanner`. Space: $O(M)$.
///
/// # Failure Modes
/// * Panics with diagnostic assertion context if any avionics template or score bug regresses.
pub fn run_issue_693_fmeca_software_ast_checks() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = build_issue_693_query_engine_pkg();
    let scoring_cfg = FmecaScoringConfig::default();
    let rows = compile_sysml::projections::stpa::fmeca::generate_fmeca_matrix(&pkg, Some(&scoring_cfg));

    assert!(
        !rows.is_empty(),
        "generate_fmeca_matrix must produce non-empty FMECA rows for QueryPlanner"
    );

    let forbidden_avionics_substrings = [
        "serial bus",
        "CRC-32",
        "Actuation Loop",
        "deflection",
        "Thermal Headroom",
        "Brownout",
        "thermal sink",
        "low-voltage cutoff",
        "watchdog",
    ];
    for row in &rows {
        for needle in forbidden_avionics_substrings {
            assert!(
                !row.failure_mode.contains(needle)
                    && !row.effect.contains(needle)
                    && !row.mitigation.contains(needle),
                "Pure software FMECA row must not contain hardcoded physical avionics substring {:?}: {:?}",
                needle,
                row
            );
        }
    }

    assert!(
        rows.iter().all(|r| r.basis == "Derived"),
        "All AST-derived FMECA rows without researched registry entries must carry basis == \"Derived\", got: {:?}",
        rows
    );
    assert_issue_693_fmeca_coverage_and_scores(&rows);
    Ok(())
}

fn seed_issue_693_subworkspace(
    temp_path: &std::path::Path,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let subws = temp_path.join(".pipeline").join("upstream");
    let subws_pipeline = subws.join(".pipeline");
    let subws_research = subws.join("docs").join("research");
    fs::create_dir_all(&subws_pipeline)?;
    fs::create_dir_all(&subws_research)?;

    let schema_src = "package QueryEnginePkg {\n    part def QueryPlanner {\n        port sqlIngress;\n        port planEgress;\n        attribute maxRetries : Integer;\n    }\n}\n";
    fs::write(subws_pipeline.join("schema.sysml"), schema_src)?;

    let registry_md = "# Failure Mode Registry\n\n| Component | Dimension | Failure Mode | Effect | S | O | D | Mitigation | Basis |\n| :--- | :--- | :--- | :--- | :---: | :---: | :---: | :--- | :--- |\n| QueryPlanner | Resource | Planner Memoization Arena Exhaustion | Out-of-memory abort during join planning | 7 | 2 | 2 | Enforce per-query arena quota | SSOT |\n";
    fs::write(subws_research.join("FAILURE_MODE_REGISTRY.md"), registry_md)?;
    Ok(subws)
}

/// Executes Issue #693 checks for subworkspace `--stpa-transpile` output scoping and registry ingestion.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when subworkspace scoping and `SSOT` ingestion hold.
///
/// # Complexity
/// * Time: $O(N)$ over the subworkspace schema and `FAILURE_MODE_REGISTRY.md`. Space: $O(N)$.
///
/// # Failure Modes
/// * Propagates any I/O failure and cleans up via `TempDirGuard`.
pub fn run_issue_693_stpa_transpile_subworkspace_checks() -> Result<(), Box<dyn std::error::Error>>
{
    let temp = TempDirGuard::new("deap_issue_693_stpa_transpile")?;
    let subws = seed_issue_693_subworkspace(temp.path())?;

    let output = Command::new(env!("CARGO_BIN_EXE_compile-sysml"))
        .current_dir(temp.path())
        .args(["--schema", ".pipeline/upstream", "--stpa-transpile"])
        .output()?;

    assert!(
        output.status.success(),
        "compile-sysml --schema .pipeline/upstream --stpa-transpile failed (status {:?}):\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !temp.path().join("docs/safety/05_FMECA_MATRIX.md").exists(),
        "--schema .pipeline/upstream --stpa-transpile must NOT pollute workspace root docs/safety/05_FMECA_MATRIX.md"
    );

    let scoped_fmeca_path = subws.join("docs/safety/05_FMECA_MATRIX.md");
    let scoped_stpa_path = subws.join("docs/safety/STPA_MATRIX.md");
    assert!(
        scoped_fmeca_path.is_file() && scoped_stpa_path.is_file(),
        "Expected subworkspace FMECA and STPA matrices in .pipeline/upstream/docs/safety"
    );

    let fmeca_md = fs::read_to_string(&scoped_fmeca_path)?;
    assert!(
        fmeca_md.contains("Planner Memoization Arena Exhaustion")
            && fmeca_md.contains("| 7 | 2 | 2 | 28 |")
            && fmeca_md.contains("| SSOT |"),
        "Subworkspace 05_FMECA_MATRIX.md must ingest FAILURE_MODE_REGISTRY.md entry with | 7 | 2 | 2 | 28 | and | SSOT |:\n{}",
        fmeca_md
    );

    let unresearched_rows: Vec<&str> = fmeca_md
        .lines()
        .filter(|line| {
            line.starts_with('|')
                && line.contains("FMECA-")
                && !line.contains("Planner Memoization Arena Exhaustion")
        })
        .collect();
    assert!(
        !unresearched_rows.is_empty()
            && unresearched_rows
                .iter()
                .all(|line| line.contains("| Derived |") && !line.contains("| SSOT |")),
        "All unresearched AST-derived rows in 05_FMECA_MATRIX.md must carry | Derived | and never | SSOT |:\n{}",
        fmeca_md
    );
    Ok(())
}
