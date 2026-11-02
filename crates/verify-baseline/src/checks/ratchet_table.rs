#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Baseline line-count ratchet table locking legacy `.rs` files at exact HEAD ceilings.
//! Non-ratcheted and new modules must satisfy strict modularity bounds without exemption.
//! Realises: [REQ-0601], [REQ-0608]

use std::borrow::Cow;

/// Lexicographically sorted `(rel_path, max_lines)` ceilings for legacy workspace `.rs` files.
/// Any growth above the recorded HEAD line count fails verification (`[REQ-0601]`, `[REQ-0608]`).
pub const LEGACY_RATCHET_MAX_LINES: &[(&str, usize)] = &[
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
    ("./crates/compile-sysml/src/lib.rs", 1327),
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
    ("./crates/compile-sysml/src/modular_codegen/emitters/rust_emitter.rs", 395),
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
    ("./crates/compile-sysml/src/modular_codegen/mod.rs", 80),
    ("./crates/compile-sysml/src/parser/expressions.rs", 1010),
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
    ("./crates/deap-core/src/diagnostics.rs", 645),
    ("./crates/deap-core/src/digest.rs", 361),
    ("./crates/deap-core/src/error.rs", 232),
    ("./crates/deap-core/src/markdown.rs", 1253),
    ("./crates/deap-core/src/rules.rs", 1349),
    ("./crates/deap-core/src/source_ast.rs", 6074),
    ("./crates/deap-core/src/source_ast_tests.rs", 1522),
    ("./crates/deap-core/src/sysml_ast/actions.rs", 359),
    ("./crates/deap-core/src/sysml_ast/connections_and_flows.rs", 577),
    ("./crates/deap-core/src/sysml_ast/expressions.rs", 576),
    ("./crates/deap-core/src/sysml_ast/identifiers.rs", 520),
    ("./crates/deap-core/src/sysml_ast/interactions.rs", 128),
    ("./crates/deap-core/src/sysml_ast/interfaces_and_items.rs", 581),
    ("./crates/deap-core/src/sysml_ast/mod.rs", 3401),
    ("./crates/deap-core/src/sysml_ast/package_model.rs", 621),
    ("./crates/deap-core/src/sysml_ast/parts_and_ports.rs", 487),
    ("./crates/deap-core/src/sysml_ast/requirements.rs", 545),
    ("./crates/deap-core/src/sysml_ast/safety_and_fmeca.rs", 557),
    ("./crates/deap-core/src/sysml_ast/states.rs", 393),
    ("./crates/ingest-sysml/src/detectors.rs", 1279),
    ("./crates/ingest-sysml/src/digest.rs", 843),
    ("./crates/ingest-sysml/src/lib.rs", 1084),
    ("./crates/ingest-sysml/src/main.rs", 974),
    ("./crates/ingest-sysml/src/table.rs", 1451),
    ("./crates/ingest-sysml/src/translators/code.rs", 465),
    ("./crates/ingest-sysml/src/translators/idl.rs", 598),
    ("./crates/ingest-sysml/src/translators/markdown.rs", 6569),
    ("./crates/ingest-sysml/src/translators/openapi.rs", 394),
    ("./crates/ingest-sysml/src/translators/yang.rs", 2016),
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
    ("./crates/verify-baseline/src/checks/structural.rs", 5583),
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

/// Unit tests verifying ratchet ordering, key prefixing, and on-disk line parity (`[REQ-0601]`, `[REQ-0608]`).
#[cfg(test)]
mod tests {
    use super::{lookup_ratchet_ceiling, LEGACY_RATCHET_MAX_LINES};
    use std::fs;
    use std::path::{Path, PathBuf};
    use walkdir::WalkDir;

    /// Resolves the workspace root directory from `CARGO_MANIFEST_DIR` (`[REQ-0601]`).
    fn workspace_root() -> Result<PathBuf, String> {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .ok_or_else(|| String::from("unable to resolve workspace root"))
    }

    /// Verifies strict lexicographical sort, `./crates/` prefix, and positive ceilings (`[REQ-0601]`, `[REQ-0608]`).
    #[test]
    fn ratchet_table_is_strictly_sorted_and_prefixed() -> Result<(), String> {
        if LEGACY_RATCHET_MAX_LINES.is_empty() {
            return Err(String::from("LEGACY_RATCHET_MAX_LINES must not be empty"));
        }
        for &(key, ceiling) in LEGACY_RATCHET_MAX_LINES {
            if !key.starts_with("./crates/") || !key.ends_with(".rs") || key.contains('\\') || ceiling == 0 {
                return Err(format!("invalid ratchet entry: ({key}, {ceiling})"));
            }
        }
        for pair in LEGACY_RATCHET_MAX_LINES.windows(2) {
            if let [(prev, _), (next, _)] = pair {
                if prev >= next {
                    return Err(format!("ratchet keys not strictly sorted: {prev} >= {next}"));
                }
            }
        }
        Ok(())
    }

    /// Verifies every ratchet entry matches exact on-disk lines and covers all `> 500`-line files (`[REQ-0608]`).
    #[test]
    fn ratchet_table_matches_exact_on_disk_line_counts() -> Result<(), String> {
        let root = workspace_root()?;
        for &(rel_key, expected) in LEGACY_RATCHET_MAX_LINES {
            let stripped = rel_key.strip_prefix("./").unwrap_or(rel_key);
            let disk_path = stripped.split('/').fold(root.clone(), |acc, seg| acc.join(seg));
            let actual = fs::read_to_string(&disk_path).map_err(|e| format!("{rel_key}: {e}"))?.lines().count();
            if actual != expected || lookup_ratchet_ceiling(rel_key) != Some(expected) {
                return Err(format!("mismatch for {rel_key}: disk={actual}, table={expected}"));
            }
        }
        for entry in WalkDir::new(root.join("crates")).into_iter().flatten() {
            if !entry.file_type().is_file() || entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let rel = entry.path().strip_prefix(&root).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");
            let key = format!("./{rel}");
            let lines = fs::read_to_string(entry.path()).map_err(|e| e.to_string())?.lines().count();
            if lines > 500 && lookup_ratchet_ceiling(&key) != Some(lines) {
                return Err(format!(">500-line file missing from ratchet table: {key} ({lines})"));
            }
        }
        Ok(())
    }

    /// Verifies cross-platform path separator normalization and non-ratcheted misses (`[REQ-0601]`).
    #[test]
    fn lookup_ratchet_ceiling_handles_normalization_and_misses() -> Result<(), String> {
        if lookup_ratchet_ceiling(".\\crates\\deap-core\\src\\source_ast.rs") != Some(6074)
            || lookup_ratchet_ceiling("crates/deap-core/src/source_ast.rs") != Some(6074)
        {
            return Err(String::from("path normalization lookup failed"));
        }
        if lookup_ratchet_ceiling("./crates/verify-baseline/src/checks/ratchet_table.rs").is_some()
            || lookup_ratchet_ceiling("./crates/nonexistent.rs").is_some()
            || lookup_ratchet_ceiling("").is_some()
        {
            return Err(String::from("unexpected ceiling hit for non-ratcheted path"));
        }
        Ok(())
    }
}
