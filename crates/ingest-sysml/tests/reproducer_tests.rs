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
