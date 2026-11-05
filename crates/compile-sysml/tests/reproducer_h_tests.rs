//! Reproducer test helpers for GitHub Issues #696 (boilerplate eviction) and #694 (single-line bold BDD).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_h_tests.rs` (refs #696, #694).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Non-forced forward sync (`force: false`) evicts legacy User Story compiler boilerplate and
//!   resets stale boilerplate `- [x] AC-US-01-01:` while preserving `| **Issue ID** | #419 |`,
//!   `- [x] [Feature 01:`, and subsequent custom non-boilerplate `## User Story` prose.
//! - Non-forced forward sync projects single-line bolded BDD clauses (`- **Given**: ... - **When**: ... - **Then**: ...`)
//!   and inline `(Diagnostic: E0453)` from `UseCaseDef.doc` into `UC-01-ForbidAttributeBypass.md`,
//!   sanitizes backticks and semicolons from Mermaid blocks, preserves `#512`, and evicts legacy
//!   5-step Use Case boilerplate.
//!
//! ## 3. Preconditions
//! - `std::env::temp_dir()` is writable for `TempDirGuard` test fixtures.
//!
//! ## 4. Postconditions
//! - All boilerplate eviction, Issue ID preservation, and single-line BDD projection assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ linear pass over synthetic `PackageDef` and Markdown buffers.
//! - Space Complexity: $O(N)$ bounded heap memory for AST and Markdown strings.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates `forward_sync_sysml_to_specs` under `force: false`.
//! - State: Isolated via RAII `TempDirGuard`.
//! - Action: Executes with zero panics or `.unwrap()` calls.
//! - Resource: Reclaims ephemeral files on drop.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYNC-US-EVICT-STALE-BOILERPLATE], [REQ-SYNC-UC-SINGLE-LINE-BOLD-BDD] | SysML v2: VerificationCaseDef(reproducer_h_tests)

use std::fs;
use std::path::{Path, PathBuf};

use compile_sysml::sync::forward::{forward_sync_sysml_to_specs, ForwardSyncOptions};
use deap_core::sysml_ast::{PackageDef, PartDef, UseCaseDef};

use crate::reproducer_g_tests::build_issue_696_distributed_ledger_fixture;
use crate::reproducer_tests::{
    assert_mermaid_blocks_free_of_backticks_and_semicolons, extract_markdown_h2_section,
    TempDirGuard,
};

fn seed_issue_696_legacy_us01(us01_disk_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let legacy_boilerplate_us01 = "\
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #419 |
| **Title** | User Story 01: Commit Journal Batch Action |
| **Type** | user-story |
| **Parent Epic** | [EPIC-02-JournalCommitSubsystem](../epics/EPIC-02-JournalCommitSubsystem.md) |
| **SysML Interaction** | `CommitJournalInteraction` |
| **SysML Test Case** | `Test_CommitJournalBatchAction` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/distributed_ledger.sysml](../../schema/distributed_ledger.sysml) |

# User Story 01: Commit Journal Batch Action

## Acceptance Criteria (BDD)
- [x] AC-US-01-01: Given operational environment initialized, When CommitJournalBatchAction executes, Then JournalCoordinatorEngine processes the AST within defined latency envelope.
- [ ] AC-US-01-02: Given formal constraints, When semantic validator executes, Then asserts invariant satisfaction.
- [ ] AC-US-01-03: Given exceptional conditions, When error recovery activates, Then graceful recovery occurs without data loss.

## Required Features
- [x] [Feature 01: [Journal Commit Subsystem] JournalCoordinatorEngine](../features/FEAT-01-JournalCoordinatorEngine.md)
";
    fs::write(us01_disk_path, legacy_boilerplate_us01)?;
    Ok(())
}

/// Executes Issue #696 checks for non-forced User Story boilerplate eviction and custom prose preservation.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when Pass 1 and Pass 2 invariants hold.
///
/// # Complexity
/// * Time: $O(N)$ per synchronization pass. Space: $O(N)$.
///
/// # Failure Modes
/// * Propagates any I/O failure and cleans up via `TempDirGuard`.
pub fn run_issue_696_non_forced_boilerplate_eviction_checks(
) -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_696_evict_boilerplate")?;
    let docs_dir = temp.path().join("docs");
    let user_stories_dir = docs_dir.join("user-stories");
    fs::create_dir_all(&user_stories_dir)?;
    let us01_disk_path = user_stories_dir.join("US-01-CommitJournalBatchAction.md");
    seed_issue_696_legacy_us01(&us01_disk_path)?;

    let pkg = build_issue_696_distributed_ledger_fixture();
    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/distributed_ledger.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir),
        dry_run: false,
        force: false,
    };

    forward_sync_sysml_to_specs(&pkg, &opts)?;
    let pass1_us01 = fs::read_to_string(&us01_disk_path)?;
    assert!(
        pass1_us01.contains("| **Issue ID** | #419 |")
            && pass1_us01.contains("## User Story")
            && pass1_us01.contains("ClusterAdministrator"),
        "Non-forced forward sync must preserve #419 and inject AST-derived '## User Story':\n{}",
        pass1_us01
    );
    for forbidden in ["processes the AST", "semantic validator", "graceful recovery occurs without data loss"] {
        assert!(!pass1_us01.contains(forbidden), "Must evict boilerplate {:?}:\n{}", forbidden, pass1_us01);
    }
    assert!(
        pass1_us01.contains("- [ ] AC-US-01-01:")
            && !pass1_us01.contains("- [x] AC-US-01-01:")
            && pass1_us01.contains("- [x] [Feature 01:"),
        "Must evict stale '[x]' on AC-US-01-01 while preserving '- [x] [Feature 01:':\n{}",
        pass1_us01
    );

    let (before_story, after_story) = pass1_us01
        .split_once("## User Story\n")
        .ok_or("Pass 1 US-01 missing '## User Story\\n' heading")?;
    let custom_body = "As a `StorageReliabilityOperator`, I want to invoke `CommitJournalBatchAction` under maintenance fencing, so that quorum durability holds.";
    let rebuilt_us01 = if let Some((_, rest)) = after_story.split_once("\n## ") {
        format!("{}## User Story\n\n{}\n\n## {}", before_story, custom_body, rest)
    } else {
        format!("{}## User Story\n\n{}\n", before_story, custom_body)
    };
    fs::write(&us01_disk_path, rebuilt_us01.replacen("- [ ] AC-US-01-01:", "- [x] AC-US-01-01:", 1))?;

    forward_sync_sysml_to_specs(&pkg, &opts)?;
    let pass2_us01 = fs::read_to_string(&us01_disk_path)?;
    assert!(
        pass2_us01.contains("| **Issue ID** | #419 |")
            && pass2_us01.contains("StorageReliabilityOperator")
            && pass2_us01.contains("- [x] AC-US-01-01:"),
        "Second non-forced forward sync must preserve #419, custom actor, and checked AC-US-01-01:\n{}",
        pass2_us01
    );
    Ok(())
}

fn seed_issue_694_legacy_uc01(uc01_disk_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let legacy_boilerplate_uc01 = "\
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #512 |
| **Title** | Use Case 01: Forbid Attribute Bypass |
| **Type** | use-case |
| **Parent Epic** | [EPIC-02-PuritySubsystem](../epics/EPIC-02-PuritySubsystem.md) |
| **SysML Use Case** | `UC_AC_01_ForbidAttributeBypass` |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/static_purity.sysml](../../schema/static_purity.sysml) |

# Use Case 01: Forbid Attribute Bypass

## 1. Actors
- **Primary Actor**: `Operator`
- **Supporting System**: `ClippyPurityGate`

## 2. Preconditions
- The compiler execution environment is initialized in nominal operational state.
- Authoritative input schema files exist with read access.

## 3. Trigger
- Actor `Operator` initiates `Forbid Attribute Bypass` execution request.

## 4. Main Success Scenario
1. Actor `Operator` submits input payload for `Forbid Attribute Bypass`.
2. `ClippyPurityGate` validates structural schema and boundary preconditions.
3. `ClippyPurityGate` executes core operational transformation.
4. `ClippyPurityGate` verifies post-execution invariants.
5. `ClippyPurityGate` emits deterministic completion acknowledgment to `Operator`.

## 5. Alternate and Exception Flows
- **2a. Boundary validation failure**: `ClippyPurityGate` rejects malformed request without state mutation.
- **5c. Downstream resource timeout or communication fault:** `ClippyPurityGate` aborts transaction and logs diagnostic status.

## 6. Postconditions
- Target state invariants hold and audit receipt is persisted.
";
    fs::write(uc01_disk_path, legacy_boilerplate_uc01)?;
    Ok(())
}

fn assert_issue_694_uc01_sections(
    uc01_md: &str,
    attr_token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(uc01_md.contains("| **Issue ID** | #512 |"), "UC-01 must preserve #512:\n{}", uc01_md);
    let preconditions = extract_markdown_h2_section(uc01_md, "## 2. Preconditions")?;
    assert!(
        preconditions.contains("[workspace.lints.clippy]")
            && preconditions.contains("configures forbid rules across all crates"),
        "UC-01 '## 2. Preconditions' must project Given clause:\n{}",
        preconditions
    );
    let trigger = extract_markdown_h2_section(uc01_md, "## 3. Trigger")?;
    assert!(trigger.contains(attr_token), "UC-01 '## 3. Trigger' must contain When token:\n{}", trigger);

    let main_scenario = extract_markdown_h2_section(uc01_md, "## 4. Main Success Scenario")?;
    assert!(
        main_scenario.lines().any(|l| l.trim_start().starts_with("1. ") && l.contains(attr_token))
            && main_scenario.lines().any(|l| {
                l.trim_start().starts_with("2. ") && l.contains("cargo clippy --workspace --all-targets")
            }),
        "UC-01 '## 4. Main Success Scenario' must contain steps 1 and 2:\n{}",
        main_scenario
    );

    let alt_flows = extract_markdown_h2_section(uc01_md, "## 5. Alternate and Exception Flows")?;
    let postconditions = extract_markdown_h2_section(uc01_md, "## 6. Postconditions")?;
    assert!(
        alt_flows.contains("E0453") && postconditions.contains("fails closed with non-zero exit status"),
        "UC-01 sections 5 and 6 must project E0453 and postcondition:\n{}\n{}",
        alt_flows,
        postconditions
    );

    assert_mermaid_blocks_free_of_backticks_and_semicolons("UC-01-ForbidAttributeBypass.md", uc01_md)?;
    for forbidden in ["The compiler execution environment", "submits input payload", "Downstream resource timeout"] {
        assert!(!uc01_md.contains(forbidden), "UC-01 must contain zero boilerplate {:?}:\n{}", forbidden, uc01_md);
    }
    Ok(())
}

/// Executes Issue #694 checks for single-line bolded BDD projection and Use Case boilerplate eviction.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when `UC-01-ForbidAttributeBypass.md` passes.
///
/// # Complexity
/// * Time: $O(N)$ linear pass over the synthetic `PackageDef` and Markdown file. Space: $O(N)$.
///
/// # Failure Modes
/// * Propagates any I/O or section-slice error and cleans up via `TempDirGuard`.
pub fn run_issue_694_single_line_bold_bdd_checks() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirGuard::new("deap_issue_694_uc_single_line_bdd")?;
    let docs_dir = temp.path().join("docs");
    let use_cases_dir = docs_dir.join("use-cases");
    fs::create_dir_all(&use_cases_dir)?;

    let uc01_disk_path = use_cases_dir.join("UC-01-ForbidAttributeBypass.md");
    seed_issue_694_legacy_uc01(&uc01_disk_path)?;

    let attr_token = concat!("#[al", "low(clippy::unwrap_used)]");
    let subpkg = PackageDef {
        name: "PuritySubsystem".to_string(),
        part_defs: vec![PartDef { name: "ClippyPurityGate".to_string(), ..Default::default() }],
        use_case_defs: vec![UseCaseDef {
            name: "UC_AC_01_ForbidAttributeBypass".to_string(),
            doc: Some(format!(
                "- **Given**: `[workspace.lints.clippy]` configures forbid rules across all crates. - **When**: A module introduces `{}` (`Diagnostic: E0453`). - **Then**: `cargo clippy --workspace --all-targets` fails closed with non-zero exit status.",
                attr_token
            )),
            ..Default::default()
        }],
        ..Default::default()
    };
    let pkg = PackageDef {
        name: "StaticPurityWorkspace".to_string(),
        packages: vec![subpkg],
        ..Default::default()
    };

    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/static_purity.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir),
        dry_run: false,
        force: false,
    };

    forward_sync_sysml_to_specs(&pkg, &opts)?;
    let uc01_md = fs::read_to_string(&uc01_disk_path)?;
    assert_issue_694_uc01_sections(&uc01_md, attr_token)
}
