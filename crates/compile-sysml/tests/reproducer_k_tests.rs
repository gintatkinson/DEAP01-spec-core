//! Reproducer test helpers for GitHub Issue #699 (`compile-sysml` Epic grounding and merge eviction).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_k_tests.rs` (refs #699).
//!
//! ## 2. Invariants & Mathematical Properties
//! - `EPIC-02-LedgerGovernance_SysML.md` grounds Sections 1, 3, 4, and 5 in `PackageDef.doc`,
//!   `SnapshotIntegrityValidator`, `ValidatorLifecycleState`, `REQ-0699`, and constraints without
//!   compiler self-description boilerplate.
//! - Under `force: false, dry_run: false`, stale `@container`, `viewport containment`,
//!   `AST references`, and `Deterministic compilation passes` sections on disk are evicted and
//!   replaced with AST-derived sections while `| **Issue ID** | #699 |`, `- [x] AC-01:`, and custom
//!   Layer 3 prose survive intact.
//!
//! ## 3. Preconditions
//! - `std::env::temp_dir()` is writable for `TempDirGuard` test fixtures.
//!
//! ## 4. Postconditions
//! - All fresh-sync and non-forced merge-eviction assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear pass over the synthetic `PackageDef` and Markdown buffers.
//! - Space Complexity: $O(N)$ bounded heap memory for AST fixtures and Markdown strings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates `forward_sync_sysml_to_specs` under both `force: true` and `force: false`.
//! - State: Isolated via RAII `TempDirGuard`.
//! - Action: Executes with zero panics or `.unwrap()` calls.
//! - Resource: Reclaims ephemeral files on drop.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYNC-FEAT-EPIC-AST-GROUNDED-699], [REQ-SYNC-MERGE-BOILERPLATE-EVICTION-699] | SysML v2: VerificationCaseDef(reproducer_k_tests)

use std::fs;
use std::path::{Path, PathBuf};

use compile_sysml::sync::forward::{forward_sync_sysml_to_specs, ForwardSyncOptions};

use crate::reproducer_j_tests::{
    assert_issue_699_acronyms, assert_issue_699_feat_sections_and_bdd,
    build_issue_699_ledger_governance_fixture,
};
use crate::reproducer_tests::{extract_markdown_h2_section, TempDirGuard};

fn assert_issue_699_feat_positive_and_negative_bdd(
    feat_md: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let feat_bdd = extract_markdown_h2_section(feat_md, "## Acceptance Criteria (BDD)")?;
    let ac_lines = feat_bdd
        .lines()
        .filter(|line| line.trim_start().starts_with("- [ ] AC-"))
        .collect::<Vec<_>>();
    let has_positive_bdd = ac_lines.iter().any(|line| {
        let lower = line.to_ascii_lowercase();
        let has_gwt = lower.contains("given") && lower.contains("when") && lower.contains("then");
        let is_positive = (lower.contains("satisf")
            || lower.contains("valid")
            || lower.contains("nominal")
            || lower.contains("holds")
            || lower.contains("conform")
            || lower.contains("verif"))
            && !lower.contains("violat");
        has_gwt && is_positive
    });
    let has_negative_bdd = ac_lines.iter().any(|line| {
        let lower = line.to_ascii_lowercase();
        let has_gwt = lower.contains("given") && lower.contains("when") && lower.contains("then");
        let cites_violation = lower.contains("violat")
            || lower.contains("invalid")
            || lower.contains("malformed")
            || lower.contains("unsatisfied")
            || lower.contains("fails");
        let cites_rejection = lower.contains("reject")
            || lower.contains("fault_out")
            || lower.contains("diagnostic")
            || lower.contains("abort")
            || lower.contains("refus")
            || lower.contains("fail");
        has_gwt && cites_violation && cites_rejection
    });
    assert!(
        has_positive_bdd && has_negative_bdd,
        "FEAT-01 BDD must include both positive satisfaction and negative violation/rejection criteria:\n{}",
        feat_bdd
    );
    Ok(())
}

fn assert_issue_699_epic_sections(epic_md: &str) -> Result<(), Box<dyn std::error::Error>> {
    for forbidden_epic_boilerplate in [
        "Deterministic compilation passes, error containment",
        "zero hardcoded domain semantics",
    ] {
        assert!(
            !epic_md.contains(forbidden_epic_boilerplate),
            "EPIC-02 must contain zero compiler boilerplate substring {:?}:\n{}",
            forbidden_epic_boilerplate,
            epic_md
        );
    }

    let epic_sec1 = extract_markdown_h2_section(epic_md, "## 1. Context")?;
    assert!(
        epic_sec1.contains("Authoritative distributed ledger replication governance subsystem."),
        "EPIC-02 '## 1. Context' must project PackageDef.doc:\n{}",
        epic_sec1
    );

    let epic_sec3 = extract_markdown_h2_section(epic_md, "## 3. Architecture")?;
    assert!(
        epic_sec3.contains("SnapshotIntegrityValidator") && epic_sec3.contains("snapshot_in"),
        "EPIC-02 '## 3. Architecture' must reference 'SnapshotIntegrityValidator' and 'snapshot_in':\n{}",
        epic_sec3
    );

    let epic_sec4 = extract_markdown_h2_section(epic_md, "## 4. Operational Considerations")?;
    assert!(
        epic_sec4.contains("ValidatorLifecycleState")
            || epic_sec4.contains("Idle")
            || epic_sec4.contains("Verifying"),
        "EPIC-02 '## 4. Operational Considerations' must reference AST lifecycle state:\n{}",
        epic_sec4
    );

    let epic_sec5 = extract_markdown_h2_section(epic_md, "## 5. Security & Governance")?;
    assert!(
        epic_sec5.contains("REQ-0699")
            && (epic_sec5.contains("C09_NinthInvariant")
                || epic_sec5.contains("C10_TenthReqInvariant")),
        "EPIC-02 '## 5. Security & Governance' must reference 'REQ-0699' and C09/C10:\n{}",
        epic_sec5
    );
    Ok(())
}

fn seed_issue_699_stale_feat(feat_merge_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let seeded_stale_feat = "\
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #699 |
| **Title** | Feature 01: [Ledger Governance Sys ML] SnapshotIntegrityValidator |
| **Type** | feature |
| **Parent Epic** | [EPIC-02-LedgerGovernance_SysML](../epics/EPIC-02-LedgerGovernance_SysML.md) |
| **Interface Type** | Physical / Structural / Internal Bus |
| **Schema Containers** | `LedgerGovernance_SysML::SnapshotIntegrityValidator` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/ledger_governance.sysml](../../schema/ledger_governance.sysml) |

# Feature 01: [Ledger Governance Sys ML] SnapshotIntegrityValidator

## Interface Requirements
### 1. Test Data Shape
```json
{ \"type\": \"object\", \"properties\": { \"operational_invariants\": { \"type\": \"string\" } } }
```

### 2. Validation & Constraints
Formal constraints and invariant guarantees enforced by SnapshotIntegrityValidator.

### 3. Visual Layout & Arrangement
Logical layout, viewport containment, and container structure for SnapshotIntegrityValidator conforming to platform design tokens. Container boundaries enforce @container isolation rules.

### 4. Interactive Flow & States
Operational lifecycle events and discrete state transitions for SnapshotIntegrityValidator.

### Layer 1: Domain State & Signal Model
Domain state vector representing SnapshotIntegrityValidator status, AST references, and typed signal models.

### Layer 2: Logic & Safety State Management
Deterministic state machine solver and safety monitor logic for SnapshotIntegrityValidator.

### Layer 3: Presentation & Actuator Interface Binding
Custom authored quorum transport diagnostic routing narrative.

## Acceptance Criteria (BDD)
- [x] AC-01: Given valid ledger snapshot input, When SnapshotIntegrityValidator evaluates C01_Bound, Then verification succeeds.
- [ ] AC-02: Given operational state, When SnapshotIntegrityValidator evaluates constraint 'C02_EpochFloor', Then asserts invariant satisfaction without violation.

## Source References
- System Architecture Model: `schema/ledger_governance.sysml`
";
    fs::write(feat_merge_path, seeded_stale_feat)?;
    Ok(())
}

fn seed_issue_699_stale_epic(epic_merge_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let seeded_stale_epic = "\
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #699 |
| **Title** | Epic 02: Ledger Governance Sys ML |
| **Type** | epic |
| **Package** | LedgerGovernance_SysML |
| **Subsystem** | Ledger Governance Sys ML |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/ledger_governance.sysml](../../schema/ledger_governance.sysml) |

# Epic 02: Ledger Governance Sys ML

## 1. Context
Authoritative distributed ledger replication governance subsystem.

## 2. Requirements & Checklist
- [ ] #699 - [Feature 01: [Ledger Governance Sys ML] SnapshotIntegrityValidator](../features/FEAT-01-SnapshotIntegrityValidator.md)

## 3. Architecture
Subsystem structural composition, port allocations, and directional data connectors realized by SnapshotIntegrityValidator.

## 4. Operational Considerations
Deterministic compilation passes, error containment, and provable polynomial complexity execution.

## 5. Security & Governance
Safety-critical invariant satisfaction, formal trace matrix closure, and zero hardcoded domain semantics.

## 6. Source References
- System Architecture Model: `schema/ledger_governance.sysml`
";
    fs::write(epic_merge_path, seeded_stale_epic)?;
    Ok(())
}

fn assert_issue_699_merged_feat_and_epic(merged_feat: &str, merged_epic: &str) {
    assert!(
        merged_feat.contains("| **Issue ID** | #699 |")
            && merged_epic.contains("| **Issue ID** | #699 |")
            && merged_feat.contains("- [x] AC-01:")
            && merged_feat
                .contains("Custom authored quorum transport diagnostic routing narrative."),
        "Non-forced sync must preserve Issue ID #699, checked AC-01, and custom Layer 3 narrative"
    );

    for evicted_token in [
        "@container",
        "viewport containment",
        "AST references",
        "Deterministic compilation passes",
    ] {
        assert!(
            !merged_feat.contains(evicted_token) && !merged_epic.contains(evicted_token),
            "Non-forced sync must evict stale boilerplate {:?}:\n{}\n{}",
            evicted_token,
            merged_feat,
            merged_epic
        );
    }

    assert!(
        merged_feat.contains("C09_NinthInvariant")
            && merged_feat.contains("C10_TenthReqInvariant")
            && merged_feat.contains("CheckpointDigestVector"),
        "Non-forced FEAT-01 sync must replace evicted boilerplate with AST-derived constraints and types:\n{}",
        merged_feat
    );
    assert!(
        merged_epic.contains("Authoritative distributed ledger replication governance subsystem.")
            && merged_epic.contains("SnapshotIntegrityValidator")
            && merged_epic.contains("REQ-0699"),
        "Non-forced EPIC-02 sync must replace evicted boilerplate with AST-derived sections:\n{}",
        merged_epic
    );
}

/// Executes Issue #699 checks for acronym preservation, Feature/Epic grounding, and merge eviction.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when Pass A (fresh) and Pass B (merge) succeed.
///
/// # Complexity
/// * Time: $O(N)$ over the synthetic `PackageDef` and Markdown buffers. Space: $O(N)$.
///
/// # Failure Modes
/// * Propagates any I/O or missing-section error and cleans up via `TempDirGuard`.
pub fn run_issue_699_features_epics_acronyms_and_merge_checks(
) -> Result<(), Box<dyn std::error::Error>> {
    let pkg = build_issue_699_ledger_governance_fixture();

    let temp_fresh = TempDirGuard::new("deap_issue_699_fresh_sync")?;
    let docs_fresh = temp_fresh.path().join("docs");
    fs::create_dir_all(&docs_fresh)?;

    let opts_fresh = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/ledger_governance.sysml"),
        docs_dir: docs_fresh.clone(),
        out_dir: Some(docs_fresh),
        dry_run: false,
        force: true,
    };
    let generated = forward_sync_sysml_to_specs(&pkg, &opts_fresh)?;

    let epic_md = generated
        .get("epics/EPIC-02-LedgerGovernance_SysML.md")
        .ok_or("Missing epics/EPIC-02-LedgerGovernance_SysML.md")?;
    let feat_md = generated
        .get("features/FEAT-01-SnapshotIntegrityValidator.md")
        .ok_or("Missing features/FEAT-01-SnapshotIntegrityValidator.md")?;
    let us01_md = generated
        .get("user-stories/US-01-CompileSysMLToCommonMark.md")
        .ok_or("Missing user-stories/US-01-CompileSysMLToCommonMark.md")?;
    let us02_md = generated
        .get("user-stories/US-02-SyncGitHubOpenAPIWithUUIDv5.md")
        .ok_or("Missing user-stories/US-02-SyncGitHubOpenAPIWithUUIDv5.md")?;
    let uc01_md = generated
        .get("use-cases/UC-01-VerifySTPAAndFMECAWithSHA256.md")
        .ok_or("Missing use-cases/UC-01-VerifySTPAAndFMECAWithSHA256.md")?;

    assert_issue_699_acronyms(epic_md, us01_md, us02_md, uc01_md)?;
    assert_issue_699_feat_sections_and_bdd(feat_md)?;
    assert_issue_699_feat_positive_and_negative_bdd(feat_md)?;
    assert_issue_699_epic_sections(epic_md)?;

    let temp_merge = TempDirGuard::new("deap_issue_699_non_forced_merge")?;
    let docs_merge = temp_merge.path().join("docs");
    let features_merge_dir = docs_merge.join("features");
    let epics_merge_dir = docs_merge.join("epics");
    fs::create_dir_all(&features_merge_dir)?;
    fs::create_dir_all(&epics_merge_dir)?;

    let feat_merge_path = features_merge_dir.join("FEAT-01-SnapshotIntegrityValidator.md");
    let epic_merge_path = epics_merge_dir.join("EPIC-02-LedgerGovernance_SysML.md");
    seed_issue_699_stale_feat(&feat_merge_path)?;
    seed_issue_699_stale_epic(&epic_merge_path)?;

    let opts_merge = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/ledger_governance.sysml"),
        docs_dir: docs_merge.clone(),
        out_dir: Some(docs_merge),
        dry_run: false,
        force: false,
    };
    forward_sync_sysml_to_specs(&pkg, &opts_merge)?;

    let merged_feat = fs::read_to_string(&feat_merge_path)?;
    let merged_epic = fs::read_to_string(&epic_merge_path)?;
    assert_issue_699_merged_feat_and_epic(&merged_feat, &merged_epic);
    Ok(())
}
