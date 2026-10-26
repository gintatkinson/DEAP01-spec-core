//! Regression and reproduction test suite for `ingest-sysml` Markdown and SysML translators.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/ingest-sysml/tests/reproducer_tests.rs`. Provides deterministic
//! regression tests for adversarial audit findings in `ingest-sysml`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every architecture blueprint document with a distinct `Blueprint ID` (`BP-*` / `BP_*`)
//!   translated via `MarkdownTranslator::translate_files` preserves a 1-to-1 bijection with a
//!   distinct top-level `PackageDef` rather than colliding on shared `Target Crates` metadata.
//!
//! ## 3. Preconditions
//! - `ingest_sysml::translators::markdown::MarkdownTranslator` is accessible and `std::env::temp_dir()`
//!   is writable for ephemeral test fixtures.
//!
//! ## 4. Postconditions
//! - All temporary test fixtures are removed prior to test completion and all assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over the byte length $N$ of synthetic Markdown fixtures.
//! - Space Complexity: $O(N)$ bounded auxiliary memory for parsed `PackageDef` ASTs.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates boundary inputs and file paths passed to `MarkdownTranslator::translate_files`.
//! - State: Ensures zero cross-test state contamination via process- and timestamp-scoped temp directories.
//! - Action: Executes deterministically with zero unbounded recursion, panics, or `.unwrap()` calls.
//! - Resource: Cleans up temporary filesystem directories deterministically before assertion checks.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-SYSML-INGEST-MD/reproducer_tests] | SysML v2: VerificationCaseDef(reproducer_tests)

use std::time::{SystemTime, UNIX_EPOCH};
use ingest_sysml::translators::markdown::MarkdownTranslator;

/// Verifies that `MarkdownTranslator::translate_files` preserves distinct top-level packages for
/// distinct blueprints (`BP-01`, `BP-02`) that share the same multi-crate `Target Crates` list.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_548_blueprint_target_crates_do_not_collapse_packages` in
/// `crates/ingest-sysml/tests/reproducer_tests.rs` (refs #548).
///
/// ## 2. Invariants & Mathematical Properties
/// - Bijective Blueprint-to-Package Traceability Invariant: $N$ distinct blueprint documents
///   with distinct `Blueprint ID` values (`BP-01`, `BP-02`) yield $N$ distinct top-level
///   `PackageDef` entries (`BP_01_*`, `BP_02_*`) even when `Target Crates` is identical.
///
/// ## 3. Preconditions
/// - Two synthetic blueprint Markdown files (`blueprint-01-alpha.md` and `blueprint-02-beta.md`)
///   sharing `| **Target Crates** | crates/a, crates/b |` are written to an isolated temp directory.
///
/// ## 4. Postconditions
/// - `MarkdownTranslator::translate_files` returns `Ok(PackageDef)` with `packages.len() == 2`,
///   containing `BP_01_Alpha_Architecture_Blueprint` and `BP_02_Beta_Architecture_Blueprint`,
///   and zero mangled `crates_a_crates_b` packages.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ over the two synthetic blueprint documents.
/// - Space Complexity: $O(N)$ for the translated `PackageDef` AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Fails closed if file writing or `translate_files` returns an `Err`.
/// - State: Removes the temporary directory prior to evaluating assertions.
/// - Action: Zero `.unwrap()`, `.expect()`, or slice indexing calls.
/// - Resource: Bounded two-file ephemeral fixture under `std::env::temp_dir()`.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-MD/test_issue_548_blueprint_target_crates_do_not_collapse_packages] | SysML v2: VerificationCaseDef(test_issue_548_blueprint_target_crates_do_not_collapse_packages)
#[test]
fn test_issue_548_blueprint_target_crates_do_not_collapse_packages() -> Result<(), String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!(
        "deap_issue_548_reproducer_{}_{}",
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("failed to create temp dir: {e}"))?;

    let bp1_path = temp_dir.join("blueprint-01-alpha.md");
    let bp2_path = temp_dir.join("blueprint-02-beta.md");

    let bp1_content = "\
# Alpha Architecture Blueprint

| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-01` |
| **Title** | Alpha Architecture Blueprint |
| **Target Crates** | `crates/a, crates/b` |

## 1. Normative Statement

The system shall enforce alpha architectural boundaries across workspace crates.
";

    let bp2_content = "\
# Beta Architecture Blueprint

| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-02` |
| **Title** | Beta Architecture Blueprint |
| **Target Crates** | `crates/a, crates/b` |

## 1. Normative Statement

The system shall enforce beta architectural boundaries across workspace crates.
";

    std::fs::write(&bp1_path, bp1_content)
        .map_err(|e| format!("failed to write bp1: {e}"))?;
    std::fs::write(&bp2_path, bp2_content)
        .map_err(|e| format!("failed to write bp2: {e}"))?;

    let translator = MarkdownTranslator::new();
    let files = vec![
        bp1_path.to_string_lossy().to_string(),
        bp2_path.to_string_lossy().to_string(),
    ];
    let result = translator.translate_files(&files, "Blueprint_Suite");

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)
            .map_err(|e| format!("failed to remove temp dir: {e}"))?;
    }

    let pkg = result?;
    let pkg_names: Vec<String> = pkg.packages.iter().map(|p| p.name.clone()).collect();

    assert_eq!(
        pkg.packages.len(),
        2,
        "Expected 2 distinct top-level blueprint packages, got: {pkg_names:?}"
    );
    assert!(
        pkg.packages.iter().any(|p| p.name.starts_with("BP_01")),
        "Expected a package starting with BP_01, got: {pkg_names:?}"
    );
    assert!(
        pkg.packages.iter().any(|p| p.name.starts_with("BP_02")),
        "Expected a package starting with BP_02, got: {pkg_names:?}"
    );
    assert!(
        !pkg.packages.iter().any(|p| p.name.contains("crates_a")),
        "Blueprint packages must not collapse into mangled target_crates name: {pkg_names:?}"
    );

    Ok(())
}

/// Executes deterministic fn `test_issue_550_bom_bullets_and_behavioral_blocks_extracted` within `crates/ingest-sysml/tests/reproducer_tests.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_550_bom_bullets_and_behavioral_blocks_extracted` in
/// `crates/ingest-sysml/tests/reproducer_tests.rs` (refs #550).
///
/// ## 2. Invariants & Mathematical Properties
/// - Verifies that `MarkdownTranslator::translate` and `MarkdownTranslator::translate_files` extract
///   BOM bullet subcomponents (`PartDef`), Mermaid `sequenceDiagram` blocks (`ActionDef` and `InteractionDef`),
///   Mermaid `stateDiagram-v2` blocks (`StateDef`), and `### AC-XX` scenarios (`UseCaseDef`) without silent omission.
///
/// ## 3. Preconditions
/// - Invoked within a valid test execution context for `test_issue_550_bom_bullets_and_behavioral_blocks_extracted`
///   in `REPRODUCER-TESTS`.
///
/// ## 4. Postconditions
/// - Returns `Result<(), String>` confirming all BOM subcomponent `PartDef` nodes and behavioral AST nodes
///   are populated and preserved across `translate` and `translate_files`.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: O(N) bounded over the fixture input lines in `test_issue_550_bom_bullets_and_behavioral_blocks_extracted`.
/// - Space Complexity: O(N) bounded auxiliary memory for `test_issue_550_bom_bullets_and_behavioral_blocks_extracted`.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and types for `test_issue_550_bom_bullets_and_behavioral_blocks_extracted` (`Result<(), String>`).
/// - State: Maintains consistent state invariants across `test_issue_550_bom_bullets_and_behavioral_blocks_extracted` without partial mutation on error.
/// - Action: Executes `test_issue_550_bom_bullets_and_behavioral_blocks_extracted` deterministically with zero unbounded recursion or blocking.
/// - Resource: Bounds stack and heap allocations within `test_issue_550_bom_bullets_and_behavioral_blocks_extracted` (`REPRODUCER-TESTS`).
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-SYSML-INGEST-MD/test_issue_550_bom_bullets_and_behavioral_blocks_extracted] | SysML v2: VerificationCaseDef(test_issue_550_bom_bullets_and_behavioral_blocks_extracted)
#[test]
fn test_issue_550_bom_bullets_and_behavioral_blocks_extracted() -> Result<(), String> {
    let md = "\
# Blueprint 01: Subsystem Verification Gate

| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-01` |
| **Title** | Subsystem Verification Gate |

## Part 1: Purpose & Architectural Scope
Defines the subsystem verification gate and its BOM subcomponents.

## Part 2: Normative Requirements

### REQ-SUBSYS-001: Mandatory Subcomponent Gate
The subsystem shall enforce all BOM subcomponent gates.

## Part 3: Structural Decomposition (SysML v2 Alignment)

### Component: SubsystemRoot

Bill of Materials (BOM):
- SubcomponentAlpha (Quantity 1): First BOM component enforcing alpha boundary invariants.
- **SubcomponentBeta** (Quantity 2): Second BOM component enforcing beta state transitions.
- `SubcomponentGamma` (Quantity 1): Third backtick-quoted BOM component enforcing gamma telemetry.

| Port Name | Direction | Type | Description |
| :--- | :--- | :--- | :--- |
| `cmd_in` | `in` | `CommandPayload` | Input command port |
| `status_out` | `out` | `StatusPayload` | Output status port |

## Part 4: Behavioral & Data Modeling

```mermaid
sequenceDiagram
    participant Op as Operator
    participant Gate as SubsystemRoot
    Op->>Gate: DispatchGateCheck(cmd_in)
    Gate-->>Op: EmitGateStatus(status_out)
```

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Verifying : DispatchGateCheck
    Verifying --> Verified : ValidationPassed
    Verified --> [*]
```

## Part 8: Verification & Acceptance Criteria

### AC-01: Deterministic Verification Scenario
- **Given**: A configured `SubsystemRoot` instance in `Idle` state.
- **When**: `DispatchGateCheck` is received on `cmd_in`.
- **Then**: `SubsystemRoot` transitions to `Verified` and emits `status_out`.
";

    let translator = MarkdownTranslator::new();
    let pkg = translator
        .translate(md, "SubsystemPkg")
        .map_err(|e| format!("translate failed: {e}"))?;
    let part_names: Vec<&str> = pkg.part_defs.iter().map(|p| p.name.as_str()).collect();

    assert!(
        part_names.contains(&"SubsystemRoot"),
        "Expected SubsystemRoot PartDef, found: {part_names:?}"
    );
    assert!(
        part_names.contains(&"SubcomponentAlpha"),
        "Expected BOM bullet SubcomponentAlpha PartDef, found: {part_names:?}"
    );
    assert!(
        part_names.contains(&"SubcomponentBeta"),
        "Expected BOM bullet SubcomponentBeta PartDef, found: {part_names:?}"
    );
    assert!(
        part_names.contains(&"SubcomponentGamma"),
        "Expected BOM bullet SubcomponentGamma PartDef, found: {part_names:?}"
    );

    for bom_name in ["SubcomponentAlpha", "SubcomponentBeta", "SubcomponentGamma"] {
        let bom_part = pkg
            .part_defs
            .iter()
            .find(|p| p.name == bom_name)
            .ok_or_else(|| format!("missing expected BOM PartDef {bom_name}"))?;
        assert!(
            !bom_part.ports.is_empty(),
            "BOM PartDef {bom_name} must have synthesized ports for forward-sync feature projection"
        );
        assert!(
            !bom_part.attributes.is_empty(),
            "BOM PartDef {bom_name} must have quantity attribute"
        );
    }

    assert!(
        !pkg.action_defs.is_empty(),
        "Expected non-empty pkg.action_defs extracted from sequenceDiagram, found empty"
    );
    assert!(
        !pkg.interaction_defs.is_empty(),
        "Expected non-empty pkg.interaction_defs extracted from sequenceDiagram, found empty"
    );
    assert!(
        !pkg.state_defs.is_empty(),
        "Expected non-empty pkg.state_defs extracted from stateDiagram-v2, found empty"
    );
    assert!(
        !pkg.use_case_defs.is_empty(),
        "Expected non-empty pkg.use_case_defs extracted from ### AC-01 heading, found empty"
    );

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_dir = std::env::temp_dir().join(format!(
        "deap_issue_550_reproducer_{}_{}",
        std::process::id(),
        nanos
    ));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("failed to create temp dir: {e}"))?;
    let bp_path = temp_dir.join("blueprint-01-subsystem.md");
    std::fs::write(&bp_path, md)
        .map_err(|e| format!("failed to write bp_path: {e}"))?;

    let files = vec![bp_path.to_string_lossy().to_string()];
    let consolidated_res = translator.translate_files(&files, "ConsolidatedPkg");

    if temp_dir.exists() {
        std::fs::remove_dir_all(&temp_dir)
            .map_err(|e| format!("failed to remove temp dir: {e}"))?;
    }

    let consolidated = consolidated_res.map_err(|e| format!("translate_files failed: {e}"))?;
    let child_pkg = consolidated
        .packages
        .first()
        .unwrap_or(&consolidated);

    assert!(
        !child_pkg.action_defs.is_empty(),
        "translate_files must preserve action_defs"
    );
    assert!(
        !child_pkg.interaction_defs.is_empty(),
        "translate_files must preserve interaction_defs"
    );
    assert!(
        !child_pkg.state_defs.is_empty(),
        "translate_files must preserve state_defs"
    );
    assert!(
        !child_pkg.use_case_defs.is_empty(),
        "translate_files must preserve use_case_defs"
    );

    Ok(())
}
