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

/// Verifies that `MarkdownTranslator::translate` preserves compound solid-arrow commands across
/// Rust `::` paths and CLI `--` flags while excluding dashed reply arrows (`-->>`, `-->`, `--x`,
/// `--)`) from `PackageDef::action_defs` and synthesized `PackageDef::state_defs` transition
/// triggers in `crates/ingest-sysml/tests/reproducer_tests.rs`.
///
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C safety objectives
/// for fn `test_issue_692_sequence_diagram_excludes_reply_arrows_and_preserves_compound_actions` in
/// `crates/ingest-sysml/tests/reproducer_tests.rs` (refs #692).
///
/// ## 2. Invariants & Mathematical Properties
/// - Compound Solid-Arrow Action Preservation: Solid Mermaid `sequenceDiagram` command arrows
///   preserve multi-token compound action semantics across Rust `::` module/function paths and
///   CLI `--` option flags without single-token prefix truncation.
/// - Dashed Reply Exclusion Invariant: All four Mermaid dashed reply arrow forms (`-->>`, `-->`,
///   `--x`, `--)`) are excluded from `PackageDef::action_defs` and synthesized
///   `PackageDef::state_defs` transition triggers, while remaining recorded in
///   `PackageDef::interaction_defs` as `reply_<Ident> from <Sender> to <Receiver>` messages with
///   a single valid SysML identifier token preceding ` from `.
///
/// ## 3. Preconditions
/// - Invoked within a valid test execution context for
///   `test_issue_692_sequence_diagram_excludes_reply_arrows_and_preserves_compound_actions` with
///   `MarkdownTranslator::new()` available.
///
/// ## 4. Postconditions
/// - Returns `Result<(), String>` confirming `pkg.action_defs` contains the exact solid-command
///   compound actions and zero dashed-reply or truncated tokens, `pkg.interaction_defs` contains
///   one interaction with valid SysML identifier message tokens for both commands and `reply_*`
///   responses, and `pkg.state_defs` transition triggers exclude dashed reply tokens.
///
/// ## 5. Algorithmic Complexity Bounds
/// - Time Complexity: $O(N)$ bounded over the synthetic Markdown blueprint byte length $N$.
/// - Space Complexity: $O(N)$ bounded auxiliary memory for the translated `PackageDef` AST.
///
/// ## 6. Failure Modes & Fault Tolerance
/// - Interface: Validates boundary inputs and `Result` return values from `MarkdownTranslator::translate`.
/// - State: Operates purely in-memory on an immutable synthetic Markdown fixture with zero shared mutable state.
/// - Action: Executes deterministically with zero `.unwrap()`, `.expect()`, `panic!`, or direct slice indexing.
/// - Resource: Bounded stack and heap allocations proportional to the fixed fixture payload.
///
/// ## 7. Realization & Traceability
/// Realises: [REQ-0603] [REQ-SYSML-INGEST-MD/test_issue_692_sequence_diagram_excludes_reply_arrows_and_preserves_compound_actions] | SysML v2: VerificationCaseDef(test_issue_692_sequence_diagram_excludes_reply_arrows_and_preserves_compound_actions)
#[test]
fn test_issue_692_sequence_diagram_excludes_reply_arrows_and_preserves_compound_actions(
) -> Result<(), String> {
    let md = "\
# Blueprint 692: Sequence Diagram Compound Action and Reply Filtering

| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-692` |
| **Title** | Sequence Diagram Compound Action and Reply Filtering |

## Part 1: Purpose & Architectural Scope
Verifies deterministic SysML v2 action and interaction extraction from Mermaid sequence diagrams.

## Part 2: Normative Requirements

### REQ-0603: Deterministic Sequence Action Extraction
The translator shall extract compound solid-arrow commands into `ActionDef` nodes and exclude dashed reply arrows from `ActionDef` and synthesized `StateDef` transition triggers.

## Part 3: Structural Decomposition (SysML v2 Alignment)

### Component: Coord

| Port Name | Direction | Type | Description |
| :--- | :--- | :--- | :--- |
| `cmd_out` | `out` | `CommandPayload` | Outbound command port |

## Part 4: Behavioral & Data Modeling

```mermaid
sequenceDiagram
    participant Coord
    participant Parser
    participant Ast
    participant Tracker
    participant Compiler
    Coord->>Parser: Parse deap_core::source_ast(schema_path)
    Parser->>Ast: deap_core::source_ast::lower_unit(ast)
    Ast-->>Parser: WBS_01_03
    Parser-->>Coord: Ok(Vec<PackageDef>)
    Coord->>Tracker: Execute gh issue edit --body-file
    Tracker-->Coord: [CLOSED-LOOP VERIFIED]
    Coord->>Compiler: compile-sysml --forward-sync docs
    Compiler--xCoord: Exit 0
    Coord->>Compiler: compile-sysml --reverse-sync docs
    Compiler--)Coord: C-PURITY-03
```
";

    let translator = MarkdownTranslator::new();
    let pkg = translator
        .translate(md, "Issue692Pkg")
        .map_err(|e| format!("translate failed: {e}"))?;

    let action_names: Vec<&str> = pkg.action_defs.iter().map(|a| a.name.as_str()).collect();

    let forbidden_reply_tokens = [
        "Ok",
        "Vec",
        "CLOSED_LOOP",
        "CLOSED_LOOP_VERIFIED",
        "ClosedLoopVerified",
        "Exit_item",
        "Exit",
        "Exit0",
        "C_PURITY_03",
        "CPurity03",
        "WBS_01_03",
    ];
    for reply_tok in forbidden_reply_tokens {
        assert!(
            !action_names.contains(&reply_tok),
            "pkg.action_defs must NOT contain dashed-reply token '{reply_tok}', found action_defs: {action_names:?}"
        );
    }
    assert!(
        !action_names.iter().any(|name| name.starts_with("reply_")),
        "pkg.action_defs must NOT contain reply_* tokens, found action_defs: {action_names:?}"
    );

    let forbidden_truncated_prefixes = [
        "Parse",
        "deap_core",
        "Execute",
        "gh",
        "compile_sysml",
        "CompileSysml",
    ];
    for truncated in forbidden_truncated_prefixes {
        assert!(
            !action_names.contains(&truncated),
            "pkg.action_defs must NOT contain truncated single-word prefix '{truncated}', found action_defs: {action_names:?}"
        );
    }

    let expected_actions = [
        "ParseDeapCoreSourceAst",
        "deap_core_source_ast_lower_unit",
        "ExecuteGhIssueEditBodyFile",
        "CompileSysmlForwardSyncDocs",
        "CompileSysmlReverseSyncDocs",
    ];
    for expected in expected_actions {
        assert!(
            action_names.contains(&expected),
            "Expected compound solid-command action '{expected}' in pkg.action_defs, found: {action_names:?}"
        );
    }
    assert_eq!(
        action_names.len(),
        expected_actions.len(),
        "Expected exact solid-command actions {expected_actions:?}, found: {action_names:?}"
    );

    assert_eq!(
        pkg.interaction_defs.len(),
        1,
        "Expected exactly 1 InteractionDef for sequenceDiagram, found: {}",
        pkg.interaction_defs.len()
    );
    let interaction = pkg
        .interaction_defs
        .first()
        .ok_or_else(|| "missing interaction_def".to_string())?;

    for msg in &interaction.messages {
        let (msg_token, route_tail) = msg.split_once(" from ").ok_or_else(|| {
            format!("interaction message missing ' from ' delimiter: '{msg}'")
        })?;
        let is_valid_sysml_ident = !msg_token.is_empty()
            && msg_token
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && msg_token
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_');
        assert!(
            is_valid_sysml_ident,
            "Interaction message token before ' from ' must be a single valid SysML identifier, got '{msg_token}' in message '{msg}'"
        );
        let (sender, receiver) = route_tail.split_once(" to ").ok_or_else(|| {
            format!("interaction message missing ' to ' delimiter: '{msg}'")
        })?;
        assert!(
            !sender.is_empty() && !receiver.is_empty(),
            "Interaction message must have non-empty sender and receiver in '{msg}'"
        );
    }

    let expected_solid_messages = [
        "ParseDeapCoreSourceAst from Coord to Parser",
        "deap_core_source_ast_lower_unit from Parser to Ast",
        "ExecuteGhIssueEditBodyFile from Coord to Tracker",
        "CompileSysmlForwardSyncDocs from Coord to Compiler",
        "CompileSysmlReverseSyncDocs from Coord to Compiler",
    ];
    for expected_msg in expected_solid_messages {
        assert!(
            interaction.messages.iter().any(|m| m == expected_msg),
            "Expected solid command message '{expected_msg}' in interaction.messages, found: {:?}",
            interaction.messages
        );
    }

    let reply_messages: Vec<&str> = interaction
        .messages
        .iter()
        .map(String::as_str)
        .filter(|m| m.starts_with("reply_"))
        .collect();
    assert_eq!(
        reply_messages.len(),
        5,
        "Expected 5 reply_<Ident> messages for the 5 dashed reply arrows, found: {:?}",
        interaction.messages
    );
    for (sender, receiver, expected_min_count) in [
        ("Ast", "Parser", 1usize),
        ("Parser", "Coord", 1usize),
        ("Tracker", "Coord", 1usize),
        ("Compiler", "Coord", 2usize),
    ] {
        let route_suffix = format!(" from {sender} to {receiver}");
        let matching_count = reply_messages
            .iter()
            .filter(|m| {
                m.strip_suffix(&route_suffix)
                    .and_then(|ident| ident.strip_prefix("reply_"))
                    .is_some_and(|suffix| !suffix.is_empty())
            })
            .count();
        assert!(
            matching_count >= expected_min_count,
            "Expected at least {expected_min_count} 'reply_<Ident>{route_suffix}' message(s), found {matching_count} in {:?}",
            interaction.messages
        );
    }
    assert_eq!(
        interaction.messages.len(),
        10,
        "Expected 10 total interaction messages (5 solid commands + 5 dashed replies), found: {:?}",
        interaction.messages
    );

    for state_def in &pkg.state_defs {
        for transition in &state_def.transitions {
            if let Some(trigger_str) = transition.trigger.as_deref() {
                for reply_tok in forbidden_reply_tokens {
                    assert!(
                        trigger_str != reply_tok,
                        "Synthesized StateDef '{}' transition trigger must NOT reference dashed reply token '{reply_tok}'",
                        state_def.name
                    );
                }
                assert!(
                    !trigger_str.starts_with("reply_"),
                    "Synthesized StateDef '{}' transition trigger must NOT reference reply_* token '{trigger_str}'",
                    state_def.name
                );
            }
        }
    }

    Ok(())
}
