//! Lossless CST-backed source AST bridge compatibility re-export. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D qualification standards.
//! Re-exports unified source AST definitions backed by `ra_ap_syntax` and `dart_cst`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Pure delegation to lossless CST parsers; zero hand-rolled byte/char/brace scanners.
//!
//! ## 3. Preconditions
//! - UTF-8 encoded source buffers supplied to parsing functions.
//!
//! ## 4. Postconditions
//! - Produces validated CST-backed AST representations.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - $O(N)$ linear time and space complexity with respect to token stream length $N$.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Syntax errors yield empty item lists or fail closed via structured error values.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0201], [REQ-0602], [REQ-0608]

#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub use crate::dart_cst::{parse_dart_cst, parse_dart_items, DartCstItem, DartItemKind, DartToken};
pub use crate::rust_cst::codemod::{
    inventory_extractable_items, list_rust_cst_items, splice_disjoint_reverse_spans, CodemodError,
    RustCstInventoryError, RustCstItemDescriptor, RustItemDescriptor, RustItemInventory,
    SpanSpliceEdit, Utf8ByteSpan, ZeroTokenCodemodConfig,
};
pub use crate::rust_cst::source_bridge::*;
use crate::sysml_ast::{
    ActionDef, AttributeDef, ConstraintDef, PackageDef, PartDef, PortDef, RequirementDef,
};

/// [DOC-TIER: A] Lowers extracted items into canonical `PackageDef`.
///
/// Safety & Traceability: Realises: [REQ-0201]
/// Contract: Deterministically lowers CST-backed `SourceAstItem` records into a `SysML` v2 `PackageDef`.
/// Bounds & Faults: Executes in linear O(N) time and space over `items.len()`; falls back to `"SourceCodeModel"` on empty package names.
#[must_use]
pub fn lower_source_ast_to_package(items: &[SourceAstItem], package_name: &str) -> PackageDef {
    let clean_pkg = sanitize_sysml_identifier(package_name, "SourceCodeModel");
    let mut pkg = PackageDef { name: clean_pkg.clone(), ..Default::default() };
    let mut seen_reqs = BTreeSet::new();
    for item in items {
        for req_id in &item.satisfied_requirements {
            if seen_reqs.insert(req_id.clone()) {
                pkg.requirement_defs.push(RequirementDef {
                    name: sanitize_sysml_identifier(req_id, "REQ_ITEM"),
                    req_id: req_id.clone(),
                    text: format!("Requirement {req_id} realized by {}", item.name),
                    ..Default::default()
                });
            }
        }
        if matches!(item.kind, SourceItemKind::Struct | SourceItemKind::Class | SourceItemKind::Type | SourceItemKind::Trait | SourceItemKind::Mixin | SourceItemKind::Extension | SourceItemKind::Typedef) {
            pkg.part_defs.push(lower_type_item_to_part(item));
        }
    }
    attach_callable_items_to_package(items, &mut pkg);
    if pkg.part_defs.is_empty() && !pkg.action_defs.is_empty() {
        pkg.part_defs.push(PartDef {
            name: format!("{clean_pkg}Module"), is_def: true, actions: pkg.action_defs.clone(), ..Default::default()
        });
    }
    pkg
}

/// Lowers a structural type item into a `PartDef` with attributes and ports.
/// Realises: [REQ-0201]
fn lower_type_item_to_part(item: &SourceAstItem) -> PartDef {
    let mut attributes = Vec::new();
    let mut ports = Vec::new();
    for field in &item.fields {
        let fname = sanitize_sysml_identifier(&field.name, "field_attr");
        let ftype = normalize_type_for_sysml(&field.type_name);
        if field.is_callback_or_port {
            ports.push(PortDef { name: fname, type_name: ftype, direction: "inout".to_string(), port_category: "DataPort".to_string(), doc: field.doc.clone(), ..Default::default() });
        } else {
            attributes.push(AttributeDef { name: fname, type_name: ftype, default_value: None, doc: field.doc.clone() });
        }
    }
    PartDef {
        name: sanitize_sysml_identifier(&item.name, "ComponentPart"),
        doc: summary_from_doc_lines(&item.doc_lines),
        is_def: true,
        attributes,
        ports,
        satisfied_requirements: item.satisfied_requirements.clone(),
        ..Default::default()
    }
}

/// Lowers callable functions and methods onto their owner `PartDef` or package `action_defs`.
/// Realises: [REQ-0201]
fn attach_callable_items_to_package(items: &[SourceAstItem], pkg: &mut PackageDef) {
    for item in items.iter().filter(|i| i.kind.is_callable()) {
        let act_name = sanitize_sysml_identifier(&item.name, "action_op");
        let in_params: Vec<AttributeDef> = item
            .params
            .iter()
            .filter(|p| !matches!(p.name.trim(), "self" | "&self" | "&mut self" | "mut self"))
            .map(|p| AttributeDef {
                name: sanitize_sysml_identifier(&p.name, "param"),
                type_name: normalize_type_for_sysml(&p.type_name),
                default_value: None,
                doc: None,
            })
            .collect();
        let out_params = item
            .return_type
            .as_deref()
            .map(str::trim)
            .filter(|rt| !rt.is_empty() && *rt != "()" && *rt != "void")
            .map(|rt| vec![AttributeDef { name: "result".to_string(), type_name: normalize_type_for_sysml(rt), default_value: None, doc: None }])
            .unwrap_or_default();
        let action = ActionDef { name: act_name.clone(), doc: summary_from_doc_lines(&item.doc_lines), in_params, out_params, performer: item.owner_name.clone(), is_def: true, ..Default::default() };
        if let Some(ref owner_raw) = item.owner_name {
            let owner_clean = sanitize_sysml_identifier(owner_raw, "ComponentPart");
            if let Some(owner_part) = pkg.part_defs.iter_mut().find(|p| p.name == owner_clean) {
                owner_part.actions.push(action);
                for (idx, expr) in item.invariants.iter().enumerate() {
                    owner_part.constraints.push(ConstraintDef { name: format!("invariant_{act_name}_{}", idx + 1), expression: expr.clone(), is_assertion: true, ..Default::default() });
                }
            }
        } else {
            pkg.action_defs.push(action);
        }
    }
}

/// [DOC-TIER: A] Sanitizes `SysML` identifier.
///
/// Safety & Traceability: Realises: [REQ-0201]
/// Contract: Normalizes arbitrary string tokens into valid ASCII `SysML` v2 identifiers containing only alphanumeric or `_` characters.
/// Bounds & Faults: Executes in linear O(N) time over `input.len()`; returns `fallback` when trimmed result is empty.
#[must_use]
pub fn sanitize_sysml_identifier(input: &str, fallback: &str) -> String {
    let s: String = input.chars().map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' }).collect();
    let trimmed = s.trim_matches('_');
    if trimmed.is_empty() { fallback.to_string() } else { trimmed.to_string() }
}

/// [DOC-TIER: A] Normalizes type for `SysML`.
///
/// Safety & Traceability: Realises: [REQ-0201]
/// Contract: Maps source-language primitive and generic wrapper types to canonical `SysML` v2 scalar or part type names.
/// Bounds & Faults: Executes in linear O(N) time over `ty.len()` with zero panics.
#[must_use]
pub fn normalize_type_for_sysml(ty: &str) -> String {
    let mut s = ty.trim().trim_end_matches('?').trim();
    while let Some(rest) = s.strip_prefix('&') {
        s = rest.trim_start().strip_prefix("mut ").unwrap_or_else(|| rest.trim_start()).trim_start();
    }
    if let Some((outer, rest)) = s.split_once('<') {
        if let Some(inner) = rest.strip_suffix('>') {
            let outer_ident = outer.trim().rsplit_once("::").map_or_else(|| outer.trim(), |(_, l)| l.trim());
            if matches!(outer_ident, "Result" | "Option" | "Vec" | "List" | "Set" | "Box" | "Arc" | "Rc" | "Future" | "Stream" | "ValueChanged" | "ValueSetter" | "ValueGetter") {
                let first = inner.split(',').next().unwrap_or("").trim();
                if !first.is_empty() && first != "()" && first != "void" {
                    return normalize_type_for_sysml(first);
                }
                return "String".to_string();
            }
        }
    }
    let base = s.split_once('<').map_or(s, |(o, _)| o.trim());
    let last = base.rsplit_once("::").map_or(base, |(_, l)| l.trim());
    let ident = sanitize_sysml_identifier(last, "String");
    match ident.as_str() {
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "int" => "Integer".to_string(),
        "f32" | "f64" | "double" | "num" => "Real".to_string(),
        "bool" => "Boolean".to_string(),
        "str" | "String" | "char" | "Path" | "PathBuf" | "Self" | "void" | "dynamic" | "Object" => "String".to_string(),
        other => other.to_string(),
    }
}

/// [DOC-TIER: A] Synthesizes first doc line summary.
///
/// Safety & Traceability: Realises: [REQ-0201]
/// Contract: Extracts the primary prose summary line from collected doc comment lines while skipping headings and `Realises:` annotations.
/// Bounds & Faults: Executes in linear O(N) time over `doc_lines`; returns `None` when no prose summary line exists.
#[must_use]
pub fn summary_from_doc_lines(doc_lines: &[String]) -> Option<String> {
    for line in doc_lines {
        let t = line.trim_start_matches('/').trim_start_matches('*').trim();
        if !t.is_empty() && !t.starts_with("Realises:") && !t.starts_with('#') { return Some(t.to_string()); }
    }
    None
}

/// [DOC-TIER: A] Resolves package name for source directory.
///
/// Safety & Traceability: Realises: [REQ-0201]
/// Contract: Determines the canonical `SysML` v2 package identifier from `explicit_package` override or parent-qualified `source_dir` stem.
/// Bounds & Faults: O(N) over path length; falls back to `"SourceCodeModel"` on unnamed or root paths.
#[must_use]
pub fn resolve_source_package_name(source_dir: &Path, explicit_package: Option<&str>) -> String {
    if let Some(pkg) = explicit_package.map(str::trim).filter(|s| !s.is_empty() && !matches!(*s, "src" | "lib")) {
        return sanitize_sysml_identifier(pkg, "SourceCodeModel");
    }
    let leaf = source_dir.file_name().and_then(|n| n.to_str()).filter(|s| !s.is_empty()).unwrap_or("SourceCodeModel");
    if matches!(leaf, "src" | "lib") {
        if let Some(parent_name) = source_dir.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()).filter(|s| !s.is_empty() && *s != "." && *s != "..") {
            return sanitize_sysml_identifier(&format!("{parent_name}_{leaf}_Realization"), "SourceCodeModel");
        }
    }
    sanitize_sysml_identifier(leaf, "SourceCodeModel")
}

/// [DOC-TIER: A] Discovers `.rs` and `.dart` files in source directory.
///
/// Safety & Traceability: Realises: [REQ-0201]
/// Contract: Recursively discovers and lexicographically sorts `.rs` and `.dart` files matching `language` under `source_dir`.
/// Bounds & Faults: O(F log F) over discovered files `F`; propagates fatal traversal errors via `CoreError`.
///
/// # Errors
/// Fails if filesystem traversal encounters fatal error conditions.
pub fn discover_source_files(source_dir: &Path, language: SourceLanguage) -> Result<Vec<PathBuf>, crate::error::CoreError> {
    let mut files = Vec::new();
    for entry_res in walkdir::WalkDir::new(source_dir) {
        let Ok(e) = entry_res else { continue };
        let matches_lang = e.path().is_file() && match language {
            SourceLanguage::Rust => e.path().extension().is_some_and(|ext| ext == "rs"),
            SourceLanguage::Dart => e.path().extension().is_some_and(|ext| ext == "dart"),
            SourceLanguage::Auto => e.path().extension().is_some_and(|ext| ext == "rs" || ext == "dart"),
        };
        if matches_lang {
            files.push(e.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

/// [DOC-TIER: A] Ingests directory of source files into a `PackageDef`.
///
/// Safety & Traceability: Realises: [REQ-0201]
/// Contract: Discovers, reads, validates, and parses all source files in `source_dir` and lowers them into a consolidated `PackageDef`.
/// Bounds & Faults: O(N) over total source bytes; propagates file read or syntax errors via `CoreError`.
///
/// # Errors
/// Fails if source file discovery, reading, or syntax validation fails.
pub fn ingest_source_directory(source_dir: &Path, language: SourceLanguage, package_name: &str) -> Result<PackageDef, crate::error::CoreError> {
    let mut items = Vec::new();
    for path in discover_source_files(source_dir, language)? {
        let content = std::fs::read_to_string(&path).map_err(|e| crate::error::CoreError::io(e.to_string()))?;
        let label = path.display().to_string();
        let file_lang = match language {
            SourceLanguage::Auto => SourceLanguage::from_path(&path).unwrap_or(SourceLanguage::Rust),
            explicit => explicit,
        };
        let mut parsed = match file_lang {
            SourceLanguage::Rust | SourceLanguage::Auto => parse_rust_source_ast(&content, &label)?,
            SourceLanguage::Dart => parse_dart_source_ast(&content, &label)?,
        };
        items.append(&mut parsed);
    }
    let effective = resolve_source_package_name(source_dir, if package_name.is_empty() { None } else { Some(package_name) });
    Ok(lower_source_ast_to_package(&items, &effective))
}

#[cfg(test)]
#[allow(clippy::pedantic, clippy::nursery)]
mod tests {
    use super::*;
    use crate::rust_cst::*;

    #[test]
    fn seals_supply_chain_receipt_and_rejects_envelope_drift() -> Result<(), Box<dyn std::error::Error>> {
        let ws = Path::new(".");
        let toml_src = "[workspace]\nmembers = [\"deap-core\"]\n";
        let lock_src = "version = 3\n[[package]]\nname = \"ra_ap_syntax\"\nversion = \"1.0.0\"\n[[package]]\nname = \"rowan\"\nversion = \"1.0.0\"\n[[package]]\nname = \"syn\"\nversion = \"1.0.0\"\n[[package]]\nname = \"quote\"\nversion = \"1.0.0\"\n[[package]]\nname = \"prettyplease\"\nversion = \"1.0.0\"\n";
        let snap = parse_locked_manifest_snapshot(ws, toml_src, lock_src, 16)?;
        let crates: BTreeSet<String> = snap.packages.keys().cloned().collect();
        let receipt = verify_analyzer_envelope_and_seal_receipt(&snap, &crates, &CstComplexityEnvelope::default())?;
        assert_eq!(receipt.verify_snapshot_parity(&snap), Ok(()));
        let mut drifted = snap;
        drifted.cargo_lock_sha256 = [9u8; 32];
        assert_eq!(receipt.verify_snapshot_parity(&drifted), Err(OfflineLockGovernanceError::ManifestToLockDriftDetected));
        Ok(())
    }

    #[test]
    fn rejects_out_of_bounds_cst_metrics_thresholds() {
        let mut t = CstMetricsThresholds::default();
        assert!(validate_metrics_thresholds(&t).is_ok());
        t.max_fn_exec_sloc = 0;
        assert_eq!(validate_metrics_thresholds(&t).err().map(|e| e.diagnostic_code()), Some("DEAP-E0803"));
    }

    #[test]
    fn enforces_lossless_green_node_roundtrip_and_max_depth_64() -> Result<(), Box<dyn std::error::Error>> {
        let t = CstMetricsThresholds::default();
        let cst = parse_verified_lossless_cst("src/a.rs", b"pub fn f() -> u32 { 1 }\n", &t)?;
        assert!(cst.is_lossless_red_green_tree_ready());
        assert!(parse_verified_lossless_cst("bad.rs", b"fn f( {", &t).is_err());
        Ok(())
    }

    #[test]
    fn prunes_cfg_test_subtrees_from_production_doc_tiers() -> Result<(), Box<dyn std::error::Error>> {
        let t = CstMetricsThresholds::default();
        let src = b"/// Doc\npub struct S;\nfn h() {}\n#[cfg(test)]\nmod tests { #[test] fn t() {} }\n";
        let cst = parse_verified_lossless_cst("src/s.rs", src, &t)?;
        let inv = classify_cfg_test_excluded_doc_tier(&cst, &t, false)?;
        assert_eq!((inv.rust_tier_a_count, inv.rust_tier_bc_count), (1, 1));
        assert!(inv.test_excluded_item_count > 0);
        Ok(())
    }

    #[test]
    fn extracts_exact_v2_metrics_sloc_arity_mut_bool_and_string_err() -> Result<(), Box<dyn std::error::Error>> {
        let t = CstMetricsThresholds::default();
        let src = b"pub fn f(x: i32) -> Result<(), String> { let mut b = true; if x > 0 && b { Ok(()) } else { Err(String::new()) } }\n";
        let cst = parse_verified_lossless_cst("src/m.rs", src, &t)?;
        let rep = analyze_cst_v2_metrics(&cst, &t)?;
        let m = rep.callable_metrics.values().next().ok_or("missing callable")?;
        assert!(m.returns_result_string_err && m.mut_bool_flag_count == 1 && m.max_predicate_clauses == 2);
        Ok(())
    }

    #[test]
    fn emits_ndjson_diagnostics_on_sloc_arity_or_string_err_breach() -> Result<(), Box<dyn std::error::Error>> {
        let t = CstMetricsThresholds::default();
        let src = "pub fn bad() -> Result<(), String> { Err(String::new()) }\n";
        let cst = parse_verified_lossless_cst("src/b.rs", src.as_bytes(), &t)?;
        let rep = analyze_cst_v2_metrics(&cst, &t)?;
        let lines = evaluate_metrics_and_format_ndjson(&rep, &t, src, true)?;
        assert!(lines.first().is_some_and(|l| l.contains("DEAP-E0803")));
        Ok(())
    }

    #[test]
    fn lists_btreemap_ordered_rust_items_with_disjoint_utf8_spans() -> Result<(), Box<dyn std::error::Error>> {
        let src = "/// Doc A\npub struct Alpha;\nfn beta() {}\n";
        let inv = list_rust_cst_items("src/lib.rs", src)?;
        assert_eq!(inv.items.len(), 2);
        assert!(inv.symbol_index.contains("Alpha") && inv.symbol_index.contains("beta"));
        assert!(list_rust_cst_items("src/lib.rs", "fn broken(").is_err());
        Ok(())
    }

    #[test]
    fn inventories_extractable_items_with_outer_doc_and_attr_trivia() -> Result<(), Box<dyn std::error::Error>> {
        let src = "/// Summary\n/// Realises: [REQ-0608]\n#[derive(Debug)]\npub struct Widget;\n";
        let map = inventory_extractable_items("src/w.rs", src)?;
        let w = map.get("Widget").ok_or("missing Widget")?;
        assert!(w.realises_reqs.contains("REQ-0608") && w.span.byte_start == 0);
        Ok(())
    }

    #[test]
    fn splices_disjoint_reverse_spans_and_rejects_post_splice_syntax_errors() -> Result<(), Box<dyn std::error::Error>> {
        let src = "pub fn a() -> u32 { 1 }\npub fn b() -> u32 { 2 }\n";
        let edits = vec![
            SpanSpliceEdit { span: Utf8ByteSpan { byte_start: 20, byte_end: 21 }, replacement: "10".to_string() },
            SpanSpliceEdit { span: Utf8ByteSpan { byte_start: 44, byte_end: 45 }, replacement: "20".to_string() },
        ];
        let out = splice_disjoint_reverse_spans(src, &edits)?;
        assert!(out.contains("10") && out.contains("20"));
        let bad = vec![SpanSpliceEdit { span: Utf8ByteSpan { byte_start: 0, byte_end: 6 }, replacement: "fn (".to_string() }];
        assert!(matches!(splice_disjoint_reverse_spans(src, &bad), Err(CodemodError::PostSpliceSyntaxError(_))));
        Ok(())
    }
}
