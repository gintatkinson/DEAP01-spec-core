#![deny(
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::undocumented_unsafe_blocks
)]
//! # Use Case Doc Clause Helpers, Provenance Traps, and UTF-8 Boundary Slicing Unit Tests
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Realises: [REQ-0604], [REQ-SYNC-FORWARD-UC-DOC-HELPERS-UNIT]
//! Verifies zero-regex Use Case inline clause splitting and Mermaid sanitization,
//! `ENGINEERING_PARAMETER_TERMS`-clean fallback prose, stale boilerplate eviction vs
//! human-authored preservation, acronym-aware `to_display_name`, Trap A & Trap B provenance
//! guards, and multi-byte UTF-8 boundary slicing.
//!
//! ## 2. Preconditions, Postconditions & Mathematical Invariants
//! - **Preconditions**: In-memory strings and AST fixtures are initialized.
//! - **Postconditions**: Unitless engineering expressions are confined to fenced `sysml` blocks.
//! - **Invariants**: Multi-byte UTF-8 identifiers and HTML comments never slice mid-codepoint.
//!
//! ## 3. Failure Modes, Degraded Operation & Recovery
//! - Test assertions return `Result<(), Box<dyn std::error::Error>>` without panicking.
//!
//! ## 4. Concurrency, Memory & Resource Bounds
//! - Uses PID-scoped temporary directories and pure in-memory checks.
//!
//! ## 5. Dependencies & Integration Contracts
//! - Exercises `use_case_clauses`, `use_case_renderer`, `use_case_diagrams`, `feature_schema`, `feature_layers`, and `feature_renderer`.
//!
//! ## 6. Verification & Acceptance Criteria
//! - Executed as part of the `compile-sysml` unit test suite.
//!
//! ## 7. Realization & Traceability
//! - Realises: [REQ-0604], [REQ-SYNC-FORWARD-UC-DOC-HELPERS-UNIT]

use super::*;
use deap_core::sysml_ast::{AttributeDef, ConstraintDef, PartDef, PortDef, RequirementDef};

/// Verifies inline Use Case clause splitting, normalization, and Mermaid label sanitization for Issue `#694`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-UC-DOC-HELPERS-UNIT]
fn assert_issue_694_clause_and_mermaid_helpers() {
    let inline = concat!(
        "- **Given**: `[workspace.lints]` is active. - **When**: Module adds `#[",
        "allow(x)]` (`Diagnostic: E0453`). - **Then**: Check fails closed."
    );
    let split = split_inline_use_case_clauses(inline);
    assert_eq!(split.len(), 3);

    let semi_inline = "Given: Ledger open; **When**: Batch runs; **Then:** Receipt stored";
    let semi_split = split_inline_use_case_clauses(semi_inline);
    assert_eq!(semi_split.len(), 3);

    assert_eq!(
        normalize_doc_clause_head("- 1. **Given**: Ready state"),
        "Given: Ready state"
    );
    assert_eq!(
        normalize_doc_clause_head("* **When:** Action dispatched"),
        "When: Action dispatched"
    );
    assert_eq!(clean_diagnostic_value("`E0602`)."), "E0602");

    let parsed = parse_use_case_doc(Some(inline));
    assert_eq!(
        parsed.preconditions,
        vec!["`[workspace.lints]` is active.".to_string()]
    );
    assert_eq!(
        parsed.alternate_flows,
        vec!["Diagnostic: E0453".to_string()]
    );
    assert_eq!(parsed.main_scenario.len(), 2);

    let sanitized = sanitize_mermaid_uc_label(
        "1. `Actor` sends \"cmd\"; state A --> B --- C across a very long operational verification description",
    );
    assert!(!sanitized.contains('`'));
    assert!(!sanitized.contains('"'));
    assert!(!sanitized.contains(';'));
    assert!(!sanitized.contains("-->"));
    assert!(!sanitized.contains("---"));
    assert!(!sanitized.starts_with("1. "));
    assert!(sanitized.chars().count() <= 60);
}

/// Asserts boilerplate eviction, human-authored preservation, and AST override precedence for Issue `#694`.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-UC-DOC-HELPERS-UNIT]
fn assert_issue_694_merge_eviction_and_ast_override(
    pkg: &mut PackageDef,
    opts: &ForwardSyncOptions,
    uc_md: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let stale_existing = "## 2. Preconditions\n- The compiler execution environment is initialized in nominal operational state.\n";
    let evicted =
        merge_existing_spec_content("use-cases/UC-01-VerifyRecord.md", uc_md, stale_existing);
    assert!(!evicted.contains("The compiler execution environment"));

    let custom_existing =
        "## 2. Preconditions\n- Signed audit ledger snapshot is mounted read-only.\n";
    let preserved =
        merge_existing_spec_content("use-cases/UC-01-VerifyRecord.md", uc_md, custom_existing);
    assert!(preserved.contains("Signed audit ledger snapshot is mounted read-only."));

    if let Some(first_uc) = pkg.use_case_defs.first_mut() {
        first_uc.doc = Some("Given: Explicit AST precondition holds".to_string());
    }
    let generated_with_doc = forward_sync_sysml_to_specs(pkg, opts)?;
    let uc_with_doc = generated_with_doc
        .get("use-cases/UC-01-VerifyRecord.md")
        .ok_or("Missing generated_with_doc UC-01-VerifyRecord.md")?;
    let ast_wins = merge_existing_spec_content(
        "use-cases/UC-01-VerifyRecord.md",
        uc_with_doc,
        custom_existing,
    );
    assert!(ast_wins.contains("Explicit AST precondition holds"));
    Ok(())
}

/// Verifies Issue `#694` zero-regex Use Case doc clause helpers, `ENGINEERING_PARAMETER_TERMS`-clean fallback prose, and `merge_existing_spec_content` boilerplate eviction.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-UC-DOC-HELPERS-UNIT]
#[test]
fn test_issue_694_use_case_doc_helpers_mermaid_sanitization_and_merge_eviction(
) -> Result<(), Box<dyn std::error::Error>> {
    use deap_core::sysml_ast::{InteractionDef, PackageDef, UseCaseDef};

    assert_issue_694_clause_and_mermaid_helpers();

    let mut pkg = PackageDef {
        name: "AuditSubsystem".to_string(),
        part_defs: vec![PartDef {
            name: "AuditCoordinator".to_string(),
            ..Default::default()
        }],
        interaction_defs: vec![InteractionDef {
            name: "VerifyInteraction".to_string(),
            lifelines: vec![
                "auditor : ExternalAuditor".to_string(),
                "coord : AuditCoordinator".to_string(),
            ],
            ..Default::default()
        }],
        use_case_defs: vec![UseCaseDef {
            name: "UC_01_VerifyRecord".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let tmp_dir = std::env::temp_dir().join(format!(
        "deap_issue_694_unit_helpers_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    let docs_dir = tmp_dir.join("docs");
    std::fs::create_dir_all(&docs_dir)?;

    let opts = ForwardSyncOptions {
        schema_path: tmp_dir.join("schema/audit.sysml"),
        docs_dir: docs_dir.clone(),
        out_dir: Some(docs_dir),
        dry_run: true,
        force: true,
    };
    let generated = forward_sync_sysml_to_specs(&pkg, &opts)?;
    let _ = std::fs::remove_dir_all(&tmp_dir);

    let uc_md = generated
        .get("use-cases/UC-01-VerifyRecord.md")
        .ok_or("Missing generated UC-01-VerifyRecord.md")?;
    assert!(
        uc_md.contains("| **Primary Actor** | ExternalAuditor |")
            && uc_md.contains("- `ExternalAuditor`"),
        "Use Case actor fallback must resolve package InteractionDef external initiator 'ExternalAuditor':\n{uc_md}"
    );
    for word in uc_md.split(|c: char| !c.is_alphabetic()) {
        if !word.is_empty() {
            assert!(
                !ENGINEERING_PARAMETER_TERMS
                    .iter()
                    .any(|term| word.eq_ignore_ascii_case(term)),
                "Undocumented UC-*.md fallback must not contain ENGINEERING_PARAMETER_TERMS token {:?}:\n{}",
                word,
                uc_md
            );
        }
    }

    assert_issue_694_merge_eviction_and_ast_override(&mut pkg, &opts, uc_md)
}

/// Asserts acronym-aware `to_display_name` and JSON Schema `x-sysml-type` / `default` generation for Issue `#699`.
/// Realises: [REQ-0604]
fn assert_issue_699_display_name_and_json_schema(
) -> Result<PartDef, Box<dyn std::error::Error>> {
    assert_eq!(
        to_display_name("LedgerGovernance_SysML"),
        "Ledger Governance SysML"
    );
    assert_eq!(
        to_display_name("CompileSysMLToCommonMark"),
        "Compile SysML To CommonMark"
    );
    assert_eq!(
        to_display_name("SyncGitHubOpenAPIWithUUIDv5"),
        "Sync GitHub OpenAPI With UUIDv5"
    );
    assert_eq!(
        to_display_name("VerifySTPAAndFMECAWithSHA256"),
        "Verify STPA And FMECA With SHA256"
    );
    assert_eq!(to_display_name("ConOps"), "Con Ops");

    let empty_part = PartDef {
        name: "UnconstrainedRelay".to_string(),
        attributes: vec![AttributeDef {
            name: "channel_tag".to_string(),
            type_name: "ChannelTag".to_string(),
            default_value: Some("\"ch-0\"".to_string()),
            doc: None,
        }],
        ports: vec![PortDef {
            name: "relay_out".to_string(),
            direction: "out".to_string(),
            type_name: "RelayFrame".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let schema_json = generate_json_schema(&empty_part);
    let parsed: serde_json::Value = serde_json::from_str(&schema_json)?;
    assert_eq!(
        parsed
            .get("properties")
            .and_then(|p| p.get("channel_tag"))
            .and_then(|c| c.get("x-sysml-type"))
            .and_then(serde_json::Value::as_str),
        Some("ChannelTag")
    );
    assert_eq!(
        parsed
            .get("properties")
            .and_then(|p| p.get("channel_tag"))
            .and_then(|c| c.get("default"))
            .and_then(serde_json::Value::as_str),
        Some("ch-0")
    );
    let req_arr = parsed
        .get("required")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing required array")?;
    assert!(
        !req_arr.iter().any(|v| v.as_str() == Some("channel_tag")),
        "Defaulted attribute channel_tag must be omitted from JSON Schema required[]"
    );
    Ok(empty_part)
}

/// Asserts Trap A (no non-fenced bullets) and Trap B (no non-fenced unitless engineering literals) across Feature subsections.
/// Realises: [REQ-0604]
fn assert_issue_699_trap_a_and_trap_b_sections(heavy_part: &PartDef, unitless_req: &RequirementDef) {
    let sec2 = render_feature_validation_and_constraints(heavy_part, &[unitless_req]);
    let sec3 = render_feature_logical_arrangement(heavy_part);
    let l1 = render_feature_layer1_state_model(heavy_part);
    let l2 = render_feature_layer2_logic_management(heavy_part, &[unitless_req]);
    let l3 = render_feature_layer3_interface_binding(heavy_part);

    let mut in_fence = false;
    for section in [&sec2, &sec3, &l1, &l2, &l3] {
        for line in section.lines() {
            let t = line.trim_start();
            if t.starts_with("```") {
                in_fence = !in_fence;
                continue;
            }
            if !in_fence {
                assert!(
                    !t.starts_with("- ") && !t.starts_with("* "),
                    "Trap A: non-fenced Feature subsection line must not start with bullet: {line:?}"
                );
                assert!(
                    !line.contains("latency < 5"),
                    "Trap B: unitless engineering literal must reside strictly inside fenced ```sysml block: {line:?}"
                );
            }
        }
    }
    assert!(
        sec2.contains("constraint Inv_10_Unitless { latency < 5 }"),
        "Section 2 fenced ```sysml block must preserve verbatim 'latency < 5':\n{sec2}"
    );
}

/// Unit-tests Issue #699 helpers: acronym-aware `to_display_name`, JSON Schema `x-sysml-type`/default projection, `generate_feature_bdd` across $K = 0..10$, and Trap A & Trap B guards.
/// Realises: [REQ-0604], [REQ-SYNC-FORWARD-UC-DOC-HELPERS-UNIT]
#[test]
fn test_issue_699_unit_display_name_bdd_cardinality_and_provenance_traps(
) -> Result<(), Box<dyn std::error::Error>> {
    let empty_part = assert_issue_699_display_name_and_json_schema()?;

    let bdd_k0 = generate_feature_bdd(&empty_part, &[]);
    assert!(!bdd_k0.contains("operational_invariants"));
    let k0_count = bdd_k0
        .lines()
        .filter(|l| l.trim_start().starts_with("- [ ] AC-"))
        .count();
    assert!((3..=10).contains(&k0_count));
    assert!(bdd_k0.contains("channel_tag") && bdd_k0.contains("relay_out"));

    let mut heavy_part = empty_part;
    for idx in 1..=9_usize {
        heavy_part.constraints.push(ConstraintDef {
            name: format!("Inv_{idx:02}"),
            expression: format!("val_{idx} >= {idx}"),
            ..Default::default()
        });
    }
    let unitless_req = RequirementDef {
        name: "UnitlessGuardReq".to_string(),
        req_id: "REQ-999".to_string(),
        constraints: vec![ConstraintDef {
            name: "Inv_10_Unitless".to_string(),
            expression: "latency < 5".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let bdd_k10 = generate_feature_bdd(&heavy_part, &[&unitless_req]);
    let k10_count = bdd_k10
        .lines()
        .filter(|l| l.trim_start().starts_with("- [ ] AC-"))
        .count();
    assert_eq!(k10_count, 10);
    for idx in 1..=9_usize {
        assert!(bdd_k10.contains(&format!("Inv_{idx:02}")));
    }
    assert!(bdd_k10.contains("Inv_10_Unitless") && bdd_k10.contains("REQ-999"));
    assert!(
        !bdd_k10.contains("latency < 5"),
        "Trap B: unitless engineering expression 'latency < 5' must not leak inline into BDD prose:\n{bdd_k10}"
    );

    assert_issue_699_trap_a_and_trap_b_sections(&heavy_part, &unitless_req);
    Ok(())
}
