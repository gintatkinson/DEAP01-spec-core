//! Reproducer test helpers for GitHub Issue #694 (`compile-sysml` multi-line BDD & diagnostic Use Cases).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_i_tests.rs` (refs #694).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Multi-line unbolded/bulleted `Actor:`, `Given:`, `When:`, `Then:`, and `Diagnostic:` clauses
//!   in `UseCaseDef.doc` resolve the primary actor in `UC-01` `## 1. Actors`, propagate that actor
//!   to sibling `ActionDef` User Stories (`US-01-VerifyAuditDigestAction.md`), map clauses across
//!   sections 2 through 6, and strip `;` from Mermaid diagram blocks.
//! - Format-2 trailing diagnostic-only `doc: Some("Diagnostic: E0602).")` cleans `"E0602)."` to
//!   `"E0602"` in `## 5. Alternate and Exception Flows` and incorporates `UseCaseDef.objective`
//!   into the trigger/main scenario with zero legacy compiler boilerplate.
//!
//! ## 3. Preconditions
//! - `std::env::temp_dir()` is writable for `TempDirGuard` test fixtures.
//!
//! ## 4. Postconditions
//! - All multi-line BDD, actor inheritance, Mermaid sanitization, and diagnostic projection assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear pass over the synthetic `PackageDef` and generated Markdown artifacts.
//! - Space Complexity: $O(N)$ bounded heap memory for AST fixtures and Markdown buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates `forward_sync_sysml_to_specs` under `force: true, dry_run: false`.
//! - State: Isolated via RAII `TempDirGuard`.
//! - Action: Executes with zero panics or `.unwrap()` calls.
//! - Resource: Reclaims ephemeral files on drop.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYNC-UC-MULTILINE-ACTOR-INHERITANCE], [REQ-SYNC-UC-DIAGNOSTIC-ONLY-OBJECTIVE] | SysML v2: VerificationCaseDef(reproducer_i_tests)

use std::fs;
use std::path::PathBuf;

use compile_sysml::projections::forward::{forward_sync_sysml_to_specs, ForwardSyncOptions};
use deap_core::sysml_cst::{ActionDef, PackageDef, PartDef, UseCaseDef};

use crate::reproducer_tests::{
    assert_mermaid_blocks_free_of_backticks_and_semicolons, extract_markdown_h2_section,
    TempDirGuard,
};

fn build_issue_694_ledger_audit_pkg() -> PackageDef {
    let subpkg = PackageDef {
        name: "AuditGateSubsystem".to_string(),
        part_defs: vec![PartDef {
            name: "AuditVerifierEngine".to_string(),
            ..Default::default()
        }],
        action_defs: vec![ActionDef {
            name: "VerifyAuditDigestAction".to_string(),
            ..Default::default()
        }],
        use_case_defs: vec![
            UseCaseDef {
                name: "UC_01_VerifyAuditDigest".to_string(),
                doc: Some(
                    "- Actor: AuditComplianceOfficer\n\
                     - Given: Audit journal digest manifest is loaded on disk\n\
                     - When: AuditComplianceOfficer invokes VerifyAuditDigestAction with statement first S1 accept trig then S2;\n\
                     - Then: AuditVerifierEngine confirms digest parity across all committed records\n\
                     - Diagnostic: E0601"
                        .to_string(),
                ),
                ..Default::default()
            },
            UseCaseDef {
                name: "UC_02_EnforceSchemaParity".to_string(),
                objective: Some("Deterministic Dual Schema Digest Parity Enforcement".to_string()),
                doc: Some("Diagnostic: E0602).".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    PackageDef {
        name: "LedgerAuditSystem".to_string(),
        packages: vec![subpkg],
        ..Default::default()
    }
}

fn assert_issue_694_uc01_and_us01(
    uc01_md: &str,
    us01_md: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let uc01_actors = extract_markdown_h2_section(uc01_md, "## 1. Actors")?;
    assert!(
        uc01_actors.contains("AuditComplianceOfficer"),
        "UC-01 '## 1. Actors' must resolve actor 'AuditComplianceOfficer':\n{}",
        uc01_actors
    );

    let uc01_preconditions = extract_markdown_h2_section(uc01_md, "## 2. Preconditions")?;
    assert!(
        uc01_preconditions.contains("Audit journal digest manifest is loaded on disk"),
        "UC-01 '## 2. Preconditions' must project Given clause:\n{}",
        uc01_preconditions
    );

    let uc01_trigger = extract_markdown_h2_section(uc01_md, "## 3. Trigger")?;
    let expected_when =
        "AuditComplianceOfficer invokes VerifyAuditDigestAction with statement first S1 accept trig then S2";
    assert!(
        uc01_trigger.contains(expected_when),
        "UC-01 '## 3. Trigger' must project When clause:\n{}",
        uc01_trigger
    );

    let uc01_mss = extract_markdown_h2_section(uc01_md, "## 4. Main Success Scenario")?;
    let expected_then = "AuditVerifierEngine confirms digest parity across all committed records";
    assert!(
        uc01_mss.contains(expected_when) && uc01_mss.contains(expected_then),
        "UC-01 '## 4. Main Success Scenario' must project When and Then clauses:\n{}",
        uc01_mss
    );

    let uc01_alt = extract_markdown_h2_section(uc01_md, "## 5. Alternate and Exception Flows")?;
    let uc01_post = extract_markdown_h2_section(uc01_md, "## 6. Postconditions")?;
    assert!(
        uc01_alt.contains("E0601") && uc01_post.contains(expected_then),
        "UC-01 sections 5 and 6 must project E0601 and Then postcondition:\n{}\n{}",
        uc01_alt,
        uc01_post
    );

    assert_mermaid_blocks_free_of_backticks_and_semicolons("UC-01-VerifyAuditDigest.md", uc01_md)?;

    let us01_story = extract_markdown_h2_section(us01_md, "## User Story")?;
    assert!(
        us01_story.contains("AuditComplianceOfficer"),
        "US-01-VerifyAuditDigestAction.md '## User Story' must inherit actor 'AuditComplianceOfficer':\n{}",
        us01_story
    );
    Ok(())
}

fn assert_issue_694_uc02_and_zero_boilerplate(
    uc01_md: &str,
    uc02_md: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let uc02_alt = extract_markdown_h2_section(uc02_md, "## 5. Alternate and Exception Flows")?;
    assert!(
        uc02_alt.contains("E0602") && !uc02_alt.contains("E0602).") && !uc02_alt.contains("E0602)"),
        "UC-02 '## 5. Alternate and Exception Flows' must clean 'E0602).' to 'E0602':\n{}",
        uc02_alt
    );

    let uc02_trigger = extract_markdown_h2_section(uc02_md, "## 3. Trigger")?;
    let uc02_mss = extract_markdown_h2_section(uc02_md, "## 4. Main Success Scenario")?;
    let expected_obj = "Deterministic Dual Schema Digest Parity Enforcement";
    assert!(
        uc02_trigger.contains(expected_obj) || uc02_mss.contains(expected_obj),
        "UC-02 must incorporate objective {:?} in its scenario/trigger:\n{}\n{}",
        expected_obj,
        uc02_trigger,
        uc02_mss
    );

    assert_mermaid_blocks_free_of_backticks_and_semicolons(
        "UC-02-EnforceSchemaParity.md",
        uc02_md,
    )?;

    for (uc_name, uc_content) in [
        ("UC-01-VerifyAuditDigest.md", uc01_md),
        ("UC-02-EnforceSchemaParity.md", uc02_md),
    ] {
        for forbidden_boilerplate in [
            "The compiler execution environment",
            "submits input payload",
            "Downstream resource timeout",
        ] {
            assert!(
                !uc_content.contains(forbidden_boilerplate),
                "{} must contain zero legacy boilerplate substring {:?}:\n{}",
                uc_name,
                forbidden_boilerplate,
                uc_content
            );
        }
    }
    Ok(())
}

/// Executes Issue #694 checks for multi-line BDD extraction, actor inheritance, and Format-2 diagnostics.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when `UC-01`, `US-01`, and `UC-02` pass.
///
/// # Complexity
/// * Time: $O(N)$ over generated Markdown specifications. Space: $O(N)$.
///
/// # Failure Modes
/// * Propagates any I/O or missing-section error and cleans up via `TempDirGuard`.
pub fn run_issue_694_multiline_bdd_and_diag_checks() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_694_uc_multiline_and_diag")?;
    let docs_dir = temp.path().join("docs");
    fs::create_dir_all(&docs_dir)?;

    let pkg = build_issue_694_ledger_audit_pkg();
    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/ledger_audit.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir.clone()),
        dry_run: false,
        force: true,
    };

    forward_sync_sysml_to_specs(&pkg, &opts)?;

    let uc01_md = fs::read_to_string(docs_dir.join("use-cases/UC-01-VerifyAuditDigest.md"))?;
    let us01_md =
        fs::read_to_string(docs_dir.join("user-stories/US-01-VerifyAuditDigestAction.md"))?;
    let uc02_md = fs::read_to_string(docs_dir.join("use-cases/UC-02-EnforceSchemaParity.md"))?;

    assert_issue_694_uc01_and_us01(&uc01_md, &us01_md)?;
    assert_issue_694_uc02_and_zero_boilerplate(&uc01_md, &uc02_md)?;
    Ok(())
}
