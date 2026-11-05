//! Reproducer test helpers for GitHub Issue #696 (`compile-sysml` User Story AST derivation).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_g_tests.rs` (refs #696).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every emitted `US-XX-*.md` artifact contains `"## User Story"` with `"As a "`, `", I want to "`,
//!   and `", so that "`, resolving explicit `doc` actor/outcome metadata or falling back to AST
//!   requirement/constraint bindings (`REQ-JRN-01`, `DurableQuorumRequirement`, `QuorumWatermarkBound`).
//! - Both `US-01` and `US-02` declare all 3 architectural layers and cite AST state, ports,
//!   constraints, transition guards, and negative boundary rejection criteria with zero compiler
//!   self-description boilerplate.
//!
//! ## 3. Preconditions
//! - `std::env::temp_dir()` is writable for `TempDirGuard` test fixtures.
//!
//! ## 4. Postconditions
//! - All User Story clause, 3-layer binding, and BDD assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear scan over the generated Markdown specification buffers.
//! - Space Complexity: $O(N)$ bounded heap storage for the returned `BTreeMap<String, String>`.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates `forward_sync_sysml_to_specs` User Story generation.
//! - State: Isolated via RAII `TempDirGuard`.
//! - Action: Executes with zero panics or `.unwrap()` calls.
//! - Resource: Reclaims ephemeral files on drop.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYNC-US-AST-CLAUSE-AND-3LAYER], [REQ-SYNC-US-BDD-ZERO-BOILERPLATE] | SysML v2: VerificationCaseDef(reproducer_g_tests)

use std::fs;
use std::path::PathBuf;

use compile_sysml::sync::forward::{forward_sync_sysml_to_specs, ForwardSyncOptions};
use deap_core::sysml_ast::{
    ActionDef, AttributeDef, ConstraintDef, InteractionDef, PackageDef, PartDef, PortDef,
    RequirementDef, StateDef, TransitionDef,
};

use crate::reproducer_tests::TempDirGuard;

fn build_issue_696_coordinator_part() -> PartDef {
    PartDef {
        name: "JournalCoordinatorEngine".to_string(),
        attributes: vec![AttributeDef { name: "active_epoch_seq".to_string(), type_name: "Integer".to_string(), default_value: None, doc: None }],
        ports: vec![
            PortDef { name: "batch_req_in".to_string(), type_name: "BatchRequestPort".to_string(), direction: "in".to_string(), ..Default::default() },
            PortDef { name: "commit_ack_out".to_string(), type_name: "CommitReceiptPort".to_string(), direction: "out".to_string(), ..Default::default() },
        ],
        constraints: vec![ConstraintDef { name: "QuorumWatermarkBound".to_string(), expression: "replica_ack_count >= 3".to_string(), ..Default::default() }],
        actions: vec![ActionDef {
            name: "VerifyReplicaQuorumAction".to_string(),
            in_params: vec![AttributeDef { name: "candidate_batch".to_string(), type_name: "JournalBatch".to_string(), default_value: None, doc: None }],
            out_params: vec![AttributeDef { name: "quorum_verified".to_string(), type_name: "Boolean".to_string(), default_value: None, doc: None }],
            doc: None,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Constructs a domain-neutral distributed transaction ledger `PackageDef` for Issue #696 tests.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `PackageDef` - Deterministic `DistributedLedgerSystem` package fixture.
///
/// # Complexity
/// * Time: $O(1)$ bounded AST struct construction. Space: $O(1)$.
///
/// # Failure Modes
/// * Infallible pure constructor.
pub fn build_issue_696_distributed_ledger_fixture() -> PackageDef {
    let subpkg = PackageDef {
        name: "JournalCommitSubsystem".to_string(),
        part_defs: vec![build_issue_696_coordinator_part()],
        action_defs: vec![ActionDef {
            name: "CommitJournalBatchAction".to_string(),
            in_params: vec![
                AttributeDef { name: "epoch_id".to_string(), type_name: "EpochNumber".to_string(), default_value: None, doc: None },
                AttributeDef { name: "entries".to_string(), type_name: "JournalEntryList".to_string(), default_value: None, doc: None },
            ],
            out_params: vec![AttributeDef { name: "commit_receipt".to_string(), type_name: "CommitToken".to_string(), default_value: None, doc: None }],
            doc: Some("Actor: ClusterAdministrator\nPreconditions: Quorum lease is active and write-ahead log is open\nOutcome: journal batch is durably committed across quorum replicas".to_string()),
            ..Default::default()
        }],
        interaction_defs: vec![InteractionDef {
            name: "CommitJournalInteraction".to_string(),
            lifelines: vec!["admin : ClusterAdministrator".to_string(), "coord : JournalCoordinatorEngine".to_string()],
            messages: vec!["DispatchBatch from admin to coord".to_string(), "CommitReceiptReady from coord to admin".to_string()],
            ..Default::default()
        }],
        state_defs: vec![StateDef {
            name: "JournalEpochState".to_string(),
            transitions: vec![TransitionDef {
                source: Some("EpochIdle".to_string()),
                target: Some("EpochCommitted".to_string()),
                trigger: Some("CommitJournalBatchAction".to_string()),
                guard: Some("quorum_lease_valid == true".to_string()),
                effect: Some("EmitCommitReceipt".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        }],
        requirement_defs: vec![RequirementDef {
            name: "DurableQuorumRequirement".to_string(),
            req_id: "REQ-JRN-01".to_string(),
            text: "Durable quorum commit".to_string(),
            satisfied_by: vec!["JournalCoordinatorEngine".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    PackageDef { name: "DistributedLedgerSystem".to_string(), packages: vec![subpkg], ..Default::default() }
}

fn assert_issue_696_us_clauses_and_layers(
    us01: &str,
    us02: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    for (us_name, us_md) in [("US-01-CommitJournalBatchAction.md", us01), ("US-02-VerifyReplicaQuorumAction.md", us02)] {
        for tok in ["## User Story", "As a ", ", I want to ", ", so that "] {
            assert!(us_md.contains(tok), "{} missing {:?}:\n{}", us_name, tok, us_md);
        }
        for heading in [
            "Layer 1: Domain State & Data Model",
            "Layer 2: Logic & State Management",
            "Layer 3: Presentation & Actuator Interface Binding",
        ] {
            assert!(us_md.contains(heading), "{} missing {:?}:\n{}", us_name, heading, us_md);
        }
        for ast_binding in ["active_epoch_seq", "batch_req_in", "commit_ack_out", "QuorumWatermarkBound"] {
            assert!(us_md.contains(ast_binding), "{} missing {:?}:\n{}", us_name, ast_binding, us_md);
        }
    }

    for tok in [
        "ClusterAdministrator",
        "CommitJournalBatchAction",
        "epoch_id: EpochNumber",
        "JournalCoordinatorEngine",
        "journal batch is durably committed across quorum replicas",
    ] {
        assert!(us01.contains(tok), "US-01 missing {:?}:\n{}", tok, us01);
    }

    for tok in ["VerifyReplicaQuorumAction", "candidate_batch: JournalBatch"] {
        assert!(us02.contains(tok), "US-02 missing {:?}:\n{}", tok, us02);
    }
    let (_, after_story) = us02.split_once("## User Story").ok_or("US-02 missing '## User Story'")?;
    let story_sec = after_story.split_once("\n## ").map_or(after_story, |(body, _)| body);
    let (_, so_that) = story_sec.split_once(", so that ").ok_or("US-02 missing ', so that '")?;
    assert!(
        so_that.contains("REQ-JRN-01")
            || so_that.contains("DurableQuorumRequirement")
            || so_that.contains("QuorumWatermarkBound"),
        "US-02 'so that' outcome must cite AST requirement/constraint, got:\n{}",
        so_that
    );
    Ok(())
}

fn assert_issue_696_us01_bdd_and_zero_boilerplate(
    us01: &str,
    us02: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let (_, us01_after_ac_heading) = us01
        .split_once("## Acceptance Criteria (BDD)")
        .ok_or("US-01 missing '## Acceptance Criteria (BDD)' section")?;
    let us01_ac_section = us01_after_ac_heading
        .split_once("\n## ")
        .map_or(us01_after_ac_heading, |(body, _)| body);

    for expected_bdd_token in [
        "Quorum lease is active and write-ahead log is open",
        "epoch_id: EpochNumber",
        "quorum_lease_valid == true",
        "EpochCommitted",
        "QuorumWatermarkBound",
        "commit_ack_out",
        "CommitReceiptReady",
    ] {
        assert!(
            us01_ac_section.contains(expected_bdd_token),
            "US-01 '## Acceptance Criteria (BDD)' must cite AST token {:?}:\n{}",
            expected_bdd_token,
            us01_ac_section
        );
    }

    let has_negative_boundary_criterion = us01_ac_section.lines().any(|line| {
        let lower = line.to_ascii_lowercase();
        let cites_guard_or_constraint =
            line.contains("quorum_lease_valid == true") || line.contains("QuorumWatermarkBound");
        let cites_violation = lower.contains("violat")
            || lower.contains("false")
            || lower.contains("fails")
            || lower.contains("unsatisfied")
            || lower.contains("not satisfied");
        let cites_rejection_without_partial_mutation = lower.contains("reject")
            && (lower.contains("without partial state mutation")
                || (lower.contains("without") && lower.contains("state mutation"))
                || lower.contains("no partial state mutation"));
        cites_guard_or_constraint && cites_violation && cites_rejection_without_partial_mutation
    });
    assert!(
        has_negative_boundary_criterion,
        "US-01 '## Acceptance Criteria (BDD)' must include a negative boundary criterion asserting rejection without partial state mutation:\n{}",
        us01_ac_section
    );

    for (us_name, us_md) in [
        ("US-01-CommitJournalBatchAction.md", us01),
        ("US-02-VerifyReplicaQuorumAction.md", us02),
    ] {
        for forbidden_boilerplate in [
            "processes the AST",
            "semantic validator",
            "graceful recovery occurs without data loss",
        ] {
            assert!(
                !us_md.contains(forbidden_boilerplate),
                "{} must contain zero compiler boilerplate substring {:?}:\n{}",
                us_name,
                forbidden_boilerplate,
                us_md
            );
        }
    }
    Ok(())
}

/// Executes Issue #696 checks for AST-derived User Story clauses, 3-layer DoD, and BDD criteria.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when `US-01` and `US-02` pass all checks.
///
/// # Complexity
/// * Time: $O(N)$ over generated User Story buffers. Space: $O(N)$.
///
/// # Failure Modes
/// * Propagates any I/O or missing-section error and cleans up via `TempDirGuard`.
pub fn run_issue_696_user_story_ast_clause_checks() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_696_us_ast_derivation")?;
    let docs_dir = temp.path().join("docs");
    fs::create_dir_all(&docs_dir)?;

    let pkg = build_issue_696_distributed_ledger_fixture();
    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/distributed_ledger.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir),
        dry_run: true,
        force: true,
    };

    let generated = forward_sync_sysml_to_specs(&pkg, &opts)?;
    let us01 = generated
        .get("user-stories/US-01-CommitJournalBatchAction.md")
        .ok_or("Missing user-stories/US-01-CommitJournalBatchAction.md in forward sync output")?;
    let us02 = generated
        .get("user-stories/US-02-VerifyReplicaQuorumAction.md")
        .ok_or("Missing user-stories/US-02-VerifyReplicaQuorumAction.md in forward sync output")?;

    assert_issue_696_us_clauses_and_layers(us01, us02)?;
    assert_issue_696_us01_bdd_and_zero_boilerplate(us01, us02)?;
    Ok(())
}
