#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Baseline line-count ratchet table locking legacy `.rs` files at exact HEAD ceilings.
//!
//! Safety & Traceability: Realises: [REQ-0601], [REQ-0608]
//! Contract: Provides `LEGACY_RATCHET_MAX_LINES`, `active_greater_than_500_ratchet_count`, and `lookup_ratchet_ceiling` for exact legacy file line-count verification.
//! Bounds & Faults: O(log N) binary search over sorted static entries with zero panicking operations.

use std::borrow::Cow;

/// Canonical `(rel_path, max_lines)` entry in `LEGACY_RATCHET_MAX_LINES`.
/// Realises: [REQ-0601], [REQ-0608]
pub type RatchetPair = (&'static str, usize);

/// Lexicographically sorted `(rel_path, max_lines)` ceilings for legacy workspace `.rs` files.
/// Any growth above the recorded HEAD line count fails verification (`[REQ-0601]`, `[REQ-0608]`).
/// Realises: [REQ-0601], [REQ-0608]
pub const LEGACY_RATCHET_MAX_LINES: &[RatchetPair] = &[
    ("./crates/assemble-conops/src/assembler.rs", 2295),
    ("./crates/assemble-conops/src/main.rs", 339),
    ("./crates/assemble-conops/src/params.rs", 5750),
    ("./crates/assemble-conops/src/sanitize.rs", 1475),
    ("./crates/assemble-conops/src/toc.rs", 289),
    ("./crates/compile-sysml/src/cli.rs", 1040),
    ("./crates/compile-sysml/src/docstring.rs", 5630),
    ("./crates/compile-sysml/src/driver.rs", 2362),
    ("./crates/compile-sysml/src/lexer/scanner.rs", 1188),
    ("./crates/compile-sysml/src/lexer/token.rs", 453),
    ("./crates/compile-sysml/src/lib.rs", 1331),
    ("./crates/compile-sysml/src/main.rs", 45),
    ("./crates/compile-sysml/src/modular_codegen/domain/bounds.rs", 268),
    ("./crates/compile-sysml/src/modular_codegen/domain/layer_role.rs", 214),
    ("./crates/compile-sysml/src/modular_codegen/domain/module_graph.rs", 355),
    ("./crates/compile-sysml/src/modular_codegen/emitters/flutter/bridge_emitter.rs", 342),
    ("./crates/compile-sysml/src/modular_codegen/emitters/flutter/contracts.rs", 310),
    ("./crates/compile-sysml/src/modular_codegen/emitters/flutter/mod.rs", 279),
    ("./crates/compile-sysml/src/modular_codegen/emitters/flutter/models_emitter.rs", 322),
    ("./crates/compile-sysml/src/modular_codegen/emitters/flutter/naming.rs", 366),
    ("./crates/compile-sysml/src/modular_codegen/emitters/flutter/seed_emitter.rs", 274),
    ("./crates/compile-sysml/src/modular_codegen/emitters/flutter/topology.rs", 245),
    ("./crates/compile-sysml/src/modular_codegen/emitters/flutter/ui_test_emitter.rs", 271),
    ("./crates/compile-sysml/src/modular_codegen/generated/domain/compilation_unit_spec/invariants.rs", 25),
    ("./crates/compile-sysml/src/modular_codegen/generated/domain/compilation_unit_spec/model.rs", 50),
    ("./crates/compile-sysml/src/modular_codegen/generated/domain/compute_cyclomatic_complexity.rs", 24),
    ("./crates/compile-sysml/src/modular_codegen/generated/domain/layer_hierarchy_spec/invariants.rs", 25),
    ("./crates/compile-sysml/src/modular_codegen/generated/domain/layer_hierarchy_spec/model.rs", 46),
    ("./crates/compile-sysml/src/modular_codegen/generated/domain/modularity_bounds_spec/invariants.rs", 25),
    ("./crates/compile-sysml/src/modular_codegen/generated/domain/modularity_bounds_spec/model.rs", 50),
    ("./crates/compile-sysml/src/modular_codegen/generated/emitters/modular_emitter_suite_spec/invariants.rs", 25),
    ("./crates/compile-sysml/src/modular_codegen/generated/emitters/modular_emitter_suite_spec/model.rs", 46),
    ("./crates/compile-sysml/src/modular_codegen/generated/emitters/modular_emitter_suite_spec/ports.rs", 41),
    ("./crates/compile-sysml/src/modular_codegen/generated/emitters/modular_flutter_emit_port.rs", 26),
    ("./crates/compile-sysml/src/modular_codegen/generated/emitters/modular_rust_emit_port.rs", 26),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/execute_four_level_lowering_pipeline.rs", 67),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/four_level_lowering_engine_spec/invariants.rs", 25),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/four_level_lowering_engine_spec/model.rs", 46),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/four_level_lowering_engine_spec/ports.rs", 41),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/modular_codegen_lifecycle/handlers_01.rs", 95),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/modular_codegen_lifecycle/handlers_02.rs", 95),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/modular_codegen_lifecycle/handlers_03.rs", 98),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/modular_codegen_lifecycle/mod.rs", 32),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/modular_codegen_lifecycle/states.rs", 49),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/schema_package_ast_port.rs", 26),
    ("./crates/compile-sysml/src/modular_codegen/generated/lowering/verified_module_graph_port.rs", 26),
    ("./crates/compile-sysml/src/modular_codegen/lowering/dag_verifier.rs", 217),
    ("./crates/compile-sysml/src/modular_codegen/lowering/expr_anf.rs", 356),
    ("./crates/compile-sysml/src/modular_codegen/lowering/outliner.rs", 364),
    ("./crates/compile-sysml/src/modular_codegen/lowering/partition_facets.rs", 324),
    ("./crates/compile-sysml/src/modular_codegen/lowering/partitioner.rs", 240),
    ("./crates/compile-sysml/src/modular_codegen/mod.rs", 84),
    ("./crates/compile-sysml/src/parser/expressions.rs", 509),
    ("./crates/compile-sysml/src/parser/grammar/behavioral_transitions.rs", 406),
    ("./crates/compile-sysml/src/parser/grammar/mod.rs", 117),
    ("./crates/compile-sysml/src/parser/grammar/safety_and_reqs.rs", 368),
    ("./crates/compile-sysml/src/parser/grammar/sigils_and_cursor.rs", 610),
    ("./crates/compile-sysml/src/parser/grammar/structural_connections.rs", 216),
    ("./crates/compile-sysml/src/parser/grammar/structural_decls.rs", 485),
    ("./crates/compile-sysml/src/parser/grammar/structural_parts.rs", 427),
    ("./crates/compile-sysml/src/parser/grammar/tests.rs", 447),
    ("./crates/compile-sysml/src/parser/grammar/tests_composite_types.rs", 484),
    ("./crates/compile-sysml/src/parser/grammar/tests_inner_blocks.rs", 451),
    ("./crates/compile-sysml/src/parser/grammar/tests_satisfy.rs", 389),
    ("./crates/compile-sysml/src/parser/grammar/tests_sigils_and_transitions.rs", 489),
    ("./crates/compile-sysml/src/parser/grammar/tests_tree_loader.rs", 411),
    ("./crates/compile-sysml/src/parser/grammar/tree_loader.rs", 288),
    ("./crates/compile-sysml/src/semantic/digest.rs", 1789),
    ("./crates/compile-sysml/src/semantic/serializer.rs", 2697),
    ("./crates/compile-sysml/src/semantic/symbols.rs", 1237),
    ("./crates/compile-sysml/src/semantic/validator.rs", 3440),
    ("./crates/compile-sysml/src/stpa/fmeca.rs", 1826),
    ("./crates/compile-sysml/src/stpa/mod.rs", 93),
    ("./crates/compile-sysml/src/stpa/safety_suite/constraint_compiler.rs", 186),
    ("./crates/compile-sysml/src/stpa/safety_suite/expr_lowering.rs", 532),
    ("./crates/compile-sysml/src/stpa/safety_suite/expr_lowering_render.rs", 411),
    ("./crates/compile-sysml/src/stpa/safety_suite/mod.rs", 251),
    ("./crates/compile-sysml/src/stpa/safety_suite/pillars.rs", 513),
    ("./crates/compile-sysml/src/stpa/safety_suite/pillars_suite.rs", 516),
    ("./crates/compile-sysml/src/stpa/safety_suite/tokens.rs", 175),
    ("./crates/compile-sysml/src/stpa/safety_suite/topology_uca.rs", 541),
    ("./crates/compile-sysml/src/stpa/uca.rs", 1138),
    ("./crates/compile-sysml/src/sync/forward/epic_emitter.rs", 301),
    ("./crates/compile-sysml/src/sync/forward/epic_renderer.rs", 380),
    ("./crates/compile-sysml/src/sync/forward/feature_diagrams.rs", 229),
    ("./crates/compile-sysml/src/sync/forward/feature_emitter.rs", 212),
    ("./crates/compile-sysml/src/sync/forward/feature_layers.rs", 370),
    ("./crates/compile-sysml/src/sync/forward/icd_wbs_emitter.rs", 551),
    ("./crates/compile-sysml/src/sync/forward/merge_and_cleanup.rs", 585),
    ("./crates/compile-sysml/src/sync/forward/mod.rs", 196),
    ("./crates/compile-sysml/src/sync/forward/model_indexer.rs", 330),
    ("./crates/compile-sysml/src/sync/forward/naming_and_dates.rs", 475),
    ("./crates/compile-sysml/src/sync/forward/story_allocator.rs", 287),
    ("./crates/compile-sysml/src/sync/forward/story_emitter.rs", 441),
    ("./crates/compile-sysml/src/sync/forward/story_layers.rs", 340),
    ("./crates/compile-sysml/src/sync/forward/story_renderer.rs", 425),
    ("./crates/compile-sysml/src/sync/forward/tests.rs", 402),
    ("./crates/compile-sysml/src/sync/forward/tests_diagrams_and_stories.rs", 470),
    ("./crates/compile-sysml/src/sync/forward/tests_non_destructive_and_actions.rs", 475),
    ("./crates/compile-sysml/src/sync/forward/tests_provenance_and_utf8.rs", 351),
    ("./crates/compile-sysml/src/sync/forward/tests_roundtrip_and_icd_wbs.rs", 443),
    ("./crates/compile-sysml/src/sync/forward/use_case_clauses.rs", 304),
    ("./crates/compile-sysml/src/sync/forward/use_case_emitter.rs", 460),
    ("./crates/compile-sysml/src/sync/forward/wbs_emitter.rs", 427),
    ("./crates/compile-sysml/src/sync/reverse.rs", 1257),
    ("./crates/compile-sysml/tests/docstring_injector_tests.rs", 1294),
    ("./crates/compile-sysml/tests/flutter_codegen_tests.rs", 696),
    ("./crates/compile-sysml/tests/multi_file_compilation_tests.rs", 435),
    ("./crates/compile-sysml/tests/reproducer_tests.rs", 3804),
    ("./crates/compile-sysml/tests/self_hosting_bootstrap.rs", 563),
    ("./crates/compile-sysml/tests/upstream_projection_acceptance_test.rs", 466),
    ("./crates/deap-core/src/diagnostics/dossier.rs", 279),
    ("./crates/deap-core/src/diagnostics/item.rs", 272),
    ("./crates/deap-core/src/diagnostics/mod.rs", 45),
    ("./crates/deap-core/src/diagnostics/severity.rs", 112),
    ("./crates/deap-core/src/diagnostics/tests.rs", 233),
    ("./crates/deap-core/src/digest.rs", 361),
    ("./crates/deap-core/src/error.rs", 232),
    ("./crates/deap-core/src/markdown/alignment.rs", 292),
    ("./crates/deap-core/src/markdown/ast.rs", 177),
    ("./crates/deap-core/src/markdown/katex.rs", 229),
    ("./crates/deap-core/src/markdown/mermaid.rs", 260),
    ("./crates/deap-core/src/markdown/mermaid_catalog.rs", 297),
    ("./crates/deap-core/src/markdown/mod.rs", 50),
    ("./crates/deap-core/src/markdown/sanitization.rs", 214),
    ("./crates/deap-core/src/markdown/tests.rs", 339),
    ("./crates/deap-core/src/rules/canonical_blueprints.rs", 287),
    ("./crates/deap-core/src/rules/codebase.rs", 295),
    ("./crates/deap-core/src/rules/excluded_dirs.rs", 282),
    ("./crates/deap-core/src/rules/master_blueprints.rs", 224),
    ("./crates/deap-core/src/rules/mod.rs", 41),
    ("./crates/deap-core/src/rules/tests.rs", 299),
    ("./crates/deap-core/src/sysml_ast/actions.rs", 359),
    ("./crates/deap-core/src/sysml_ast/cache.rs", 93),
    ("./crates/deap-core/src/sysml_ast/connections_and_flows.rs", 457),
    ("./crates/deap-core/src/sysml_ast/cot_audit.rs", 63),
    ("./crates/deap-core/src/sysml_ast/expressions.rs", 449),
    ("./crates/deap-core/src/sysml_ast/identifiers.rs", 377),
    ("./crates/deap-core/src/sysml_ast/interactions.rs", 128),
    ("./crates/deap-core/src/sysml_ast/interfaces_and_items.rs", 445),
    ("./crates/deap-core/src/sysml_ast/mod.rs", 86),
    ("./crates/deap-core/src/sysml_ast/package_model.rs", 433),
    ("./crates/deap-core/src/sysml_ast/parser.rs", 221),
    ("./crates/deap-core/src/sysml_ast/parts_and_ports.rs", 487),
    ("./crates/deap-core/src/sysml_ast/persistence.rs", 105),
    ("./crates/deap-core/src/sysml_ast/requirements.rs", 453),
    ("./crates/deap-core/src/sysml_ast/safety_and_fmeca.rs", 458),
    ("./crates/deap-core/src/sysml_ast/schema_extractor.rs", 336),
    ("./crates/deap-core/src/sysml_ast/schema_extractor_data.rs", 152),
    ("./crates/deap-core/src/sysml_ast/slice.rs", 202),
    ("./crates/deap-core/src/sysml_ast/states.rs", 393),
    ("./crates/deap-core/src/sysml_ast/sysml_ast_tests.rs", 1535),
    ("./crates/deap-core/src/sysml_ast/test_cases.rs", 159),
    ("./crates/ingest-sysml/src/detectors.rs", 1279),
    ("./crates/ingest-sysml/src/digest.rs", 843),
    ("./crates/ingest-sysml/src/lib.rs", 1084),
    ("./crates/ingest-sysml/src/main.rs", 974),
    ("./crates/ingest-sysml/src/table.rs", 446),
    ("./crates/ingest-sysml/src/translators/code.rs", 231),
    ("./crates/ingest-sysml/src/translators/idl.rs", 358),
    ("./crates/ingest-sysml/src/translators/markdown.rs", 6569),
    ("./crates/ingest-sysml/src/translators/openapi.rs", 394),
    ("./crates/ingest-sysml/src/translators/yang.rs", 2002),
    ("./crates/ingest-sysml/tests/code_ingestion_tests.rs", 769),
    ("./crates/ingest-sysml/tests/code_realization_scoping_tests.rs", 413),
    ("./crates/ingest-sysml/tests/reproducer_tests.rs", 611),
    ("./crates/reconcile-backlog/src/checklist.rs", 2366),
    ("./crates/reconcile-backlog/src/create_issue.rs", 1068),
    ("./crates/reconcile-backlog/src/frontmatter.rs", 1356),
    ("./crates/reconcile-backlog/src/lib.rs", 2625),
    ("./crates/reconcile-backlog/src/main.rs", 405),
    ("./crates/reconcile-backlog/src/tracker.rs", 2414),
    ("./crates/reconcile-backlog/tests/reproducer_tests.rs", 829),
    ("./crates/verify-baseline/src/audit_code.rs", 3214),
    ("./crates/verify-baseline/src/blueprint_gate.rs", 4815),
    ("./crates/verify-baseline/src/check_secrets.rs", 297),
    ("./crates/verify-baseline/src/checks/codebase.rs", 2350),
    ("./crates/verify-baseline/src/checks/docstring_gate.rs", 3603),
    ("./crates/verify-baseline/src/checks/domain_cleanliness_gate.rs", 959),
    ("./crates/verify-baseline/src/checks/math_katex.rs", 206),
    ("./crates/verify-baseline/src/checks/mermaid.rs", 206),
    ("./crates/verify-baseline/src/checks/parity_gates.rs", 7416),
    ("./crates/verify-baseline/src/checks/safety_stpa.rs", 2055),
    ("./crates/verify-baseline/src/checks/structural.rs", 5589),
    ("./crates/verify-baseline/src/checks/wbs_integrity.rs", 407),
    ("./crates/verify-baseline/src/commit_msg.rs", 1043),
    ("./crates/verify-baseline/src/defect_dossier.rs", 2465),
    ("./crates/verify-baseline/src/main.rs", 791),
    ("./crates/verify-baseline/src/output_integrity.rs", 265),
    ("./crates/verify-baseline/src/runner.rs", 1313),
    ("./crates/verify-baseline/src/setup_hooks.rs", 312),
    ("./crates/verify-baseline/src/spec_audit/behavioral.rs", 1241),
    ("./crates/verify-baseline/src/spec_audit/conops_audit.rs", 3055),
    ("./crates/verify-baseline/src/spec_audit/diagram_ast.rs", 3060),
    ("./crates/verify-baseline/src/spec_audit/logical_ui.rs", 1172),
    ("./crates/verify-baseline/src/spec_audit/markdown_ast.rs", 1143),
    ("./crates/verify-baseline/src/spec_audit/metadata.rs", 1398),
    ("./crates/verify-baseline/src/spec_audit/mod.rs", 1605),
    ("./crates/verify-baseline/src/spec_audit/provenance.rs", 2780),
    ("./crates/verify-baseline/src/spec_audit/sections.rs", 1084),
    ("./crates/verify-baseline/src/spec_audit/uml_validator.rs", 3172),
    ("./crates/verify-baseline/tests/blueprint_gate_tests.rs", 2572),
    ("./crates/verify-baseline/tests/check_36_docstring_gate_tests.rs", 1260),
    ("./crates/verify-baseline/tests/codebase_checks_tests.rs", 1021),
    ("./crates/verify-baseline/tests/defect_dossier_tests.rs", 1129),
];

/// Indicates that zero new unratcheted `> 500`-line modules are permitted in the workspace (`[REQ-0601]`, `[REQ-0801]`).
pub const ZERO_NEW_RATCHET_ENFORCED: bool = true;

/// Counts entries in [`LEGACY_RATCHET_MAX_LINES`] whose line ceiling exceeds `500` (`[REQ-0601]`, `[REQ-0801]`).
#[must_use]
pub const fn active_greater_than_500_ratchet_count() -> usize {
    let mut rest = LEGACY_RATCHET_MAX_LINES;
    let mut count = 0;
    while let Some((&(_, ceiling), tail)) = rest.split_first() {
        if ceiling > 500 {
            count += 1;
        }
        rest = tail;
    }
    count
}

/// Returns the maximum allowed line count for `rel_path` via `O(log N)` binary search.
/// Normalizes Windows `\` separators and bare `crates/...` prefixes (`[REQ-0601]`, `[REQ-0608]`).
#[must_use]
pub fn lookup_ratchet_ceiling(rel_path: &str) -> Option<usize> {
    let slash_norm: Cow<'_, str> = if rel_path.contains('\\') {
        Cow::Owned(rel_path.replace('\\', "/"))
    } else {
        Cow::Borrowed(rel_path)
    };
    let canonical: Cow<'_, str> = if slash_norm.starts_with("crates/") {
        Cow::Owned(format!("./{slash_norm}"))
    } else {
        slash_norm
    };
    let target = canonical.as_ref();
    LEGACY_RATCHET_MAX_LINES
        .binary_search_by_key(&target, |&(path, _)| path)
        .ok()
        .and_then(|idx| LEGACY_RATCHET_MAX_LINES.get(idx).map(|&(_, ceiling)| ceiling))
}
