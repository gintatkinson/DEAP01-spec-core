//! Unit tests for `MAX_AST_DEPTH` (64) stack bounds and multi-byte UTF-8 annotation delimiter validation.
//!
//! Safety & Traceability: Realises: [REQ-SEMANTIC-VALIDATOR-TESTS]
//! Contract: Verifies bounded O(1) call-stack AST validation under 256 KiB thread stacks and fail-closed UTF-8 `@SafetyRealises` / `@TriggersHazard` delimiter validation.
//! Bounds & Faults: Iteratively drains deep `PackageDef` and `PartDef` trees in O(1) stack frames with zero panics.

use super::*;

/// Iteratively dismantles a `PackageDef` tree and all nested `PartDef` subtrees in O(1) stack frames.
fn drain_package_tree_iteratively(root: PackageDef) {
    let mut extracted_parts: Vec<PartDef> = Vec::new();
    let mut pkg_drain = vec![root];
    while let Some(mut p) = pkg_drain.pop() {
        extracted_parts.append(&mut p.part_defs);
        pkg_drain.append(&mut p.packages);
    }
    while let Some(mut pt) = extracted_parts.pop() {
        extracted_parts.append(&mut pt.parts);
    }
}

/// Verifies symbol indexing, semantic validation, serialization, and digest at exact depth `bound` (64).
fn assert_depth_64_valid_package_and_part(bound: usize) {
    use crate::semantic::digest::SchemaDigest;
    use crate::semantic::serializer::SysmlSerializable;

    let mut pkg_64 = PackageDef {
        name: format!("Pkg_{}", bound.saturating_sub(1)),
        requirement_defs: vec![RequirementDef {
            name: "Req_Leaf_63".to_string(),
            text: "Leaf requirement at depth 63.".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    for idx in (0..bound.saturating_sub(1)).rev() {
        pkg_64 = PackageDef { name: format!("Pkg_{idx}"), packages: vec![pkg_64], ..Default::default() };
    }
    let pkg_64_symbols = SymbolTable::build_from_package(&pkg_64);
    assert_eq!(pkg_64_symbols.all_of_kind(SymbolKind::Package).len(), bound);
    assert_eq!(pkg_64_symbols.all_of_kind(SymbolKind::Requirement).len(), 1);
    let mut pkg_64_validator = SemanticValidator::new(&pkg_64);
    pkg_64_validator.validate();
    assert!(!pkg_64_validator.has_errors());
    let pkg_64_sysml = pkg_64.to_sysml(0);
    assert!(pkg_64_sysml.contains("package Pkg_0 {") && pkg_64_sysml.contains("package Pkg_63 {"));
    let pkg_64_digest = SchemaDigest::compute(&pkg_64, &pkg_64_sysml);
    assert!(!pkg_64_digest.sha256.is_empty() && pkg_64_digest.total_lines > 0);
    assert_eq!(pkg_64_digest.node_counts.get("packages").copied(), Some(bound));
    drain_package_tree_iteratively(pkg_64);

    let mut part_chain_63 = PartDef {
        name: format!("Part_{}", bound.saturating_sub(1)),
        type_name: Some("String".to_string()),
        ports: vec![PortDef { name: "leaf_out".to_string(), direction: "out".to_string(), type_name: "Real".to_string(), ..Default::default() }],
        ..Default::default()
    };
    for idx in (1..bound.saturating_sub(1)).rev() {
        part_chain_63 = PartDef { name: format!("Part_{idx}"), type_name: Some("Integer".to_string()), parts: vec![part_chain_63], ..Default::default() };
    }
    let part_root_64 = PackageDef { name: "PartRoot".to_string(), part_defs: vec![part_chain_63], ..Default::default() };
    let part_64_symbols = SymbolTable::build_from_package(&part_root_64);
    assert_eq!(part_64_symbols.all_of_kind(SymbolKind::Package).len(), 1);
    assert_eq!(part_64_symbols.all_of_kind(SymbolKind::Part).len(), bound.saturating_sub(1));
    let mut part_64_validator = SemanticValidator::new(&part_root_64);
    part_64_validator.validate();
    assert!(!part_64_validator.has_errors());
    let part_64_sysml = part_root_64.to_sysml(0);
    assert!(part_64_sysml.contains("part def Part_1 : Integer {") && part_64_sysml.contains("part def Part_63 : String {"));
    let part_64_digest = SchemaDigest::compute(&part_root_64, &part_64_sysml);
    assert_eq!(part_64_digest.node_counts.get("part_defs").copied(), Some(bound.saturating_sub(1)));
    drain_package_tree_iteratively(part_root_64);

    let mut bad_part_63 = PartDef { name: "BadLeaf_63".to_string(), is_def: false, type_name: Some("UndefinedTypeAtDepth63".to_string()), ..Default::default() };
    for idx in (1..bound.saturating_sub(1)).rev() {
        bad_part_63 = PartDef { name: format!("Carrier_{idx}"), parts: vec![bad_part_63], ..Default::default() };
    }
    let bad_root_64 = PackageDef { name: "BadRoot64".to_string(), part_defs: vec![bad_part_63], ..Default::default() };
    let mut bad_64_validator = SemanticValidator::new(&bad_root_64);
    bad_64_validator.validate();
    assert!(bad_64_validator.has_errors());
    drain_package_tree_iteratively(bad_root_64);
}

/// Verifies depth-65 truncation and generic wrapper nesting bounds (`Vec<...>` at 64 vs 65).
fn assert_depth_65_truncation_and_generics(bound: usize) {
    use crate::semantic::digest::SchemaDigest;
    use crate::semantic::serializer::SysmlSerializable;

    let over_bound = bound.saturating_add(1);
    let dup_port = PortDef { name: "dup_port".to_string(), direction: "in".to_string(), type_name: "Real".to_string(), ..Default::default() };
    let mut part_chain_over = PartDef {
        name: format!("Part_{}", over_bound.saturating_sub(1)),
        is_def: false,
        type_name: Some("UnindexedBeyondDepth64Type".to_string()),
        ports: vec![dup_port.clone(), dup_port],
        connections: vec![ConnectionDef { name: "BeyondBoundConn".to_string(), source_port: "Missing.src".to_string(), target_port: "Missing.tgt".to_string(), ..Default::default() }],
        requirements: vec![RequirementDef {
            name: "BeyondBoundReq".to_string(),
            text: "Shall cap traversal @SafetyRealises(UCA-MISSING-99) @TriggersHazard(HZ-MISSING-99)".to_string(),
            satisfied_by: vec!["MissingSatisfierBeyond64".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };
    for idx in (1..over_bound.saturating_sub(1)).rev() {
        part_chain_over = PartDef { name: format!("Part_{idx}"), parts: vec![part_chain_over], ..Default::default() };
    }
    let over_root_65 = PackageDef {
        name: "OverRoot65".to_string(),
        constraint_defs: vec![ConstraintDef { name: "UCA_01".to_string(), expression: "x > 0".to_string(), ..Default::default() }],
        hazard_defs: vec![HazardDef { name: "HZ_01".to_string(), ..Default::default() }],
        part_defs: vec![part_chain_over],
        ..Default::default()
    };
    let over_symbols = SymbolTable::build_from_package(&over_root_65);
    assert_eq!(over_symbols.all_of_kind(SymbolKind::Part).len(), bound.saturating_sub(1));
    let mut over_validator = SemanticValidator::new(&over_root_65);
    over_validator.validate();
    assert!(!over_validator.has_errors());
    let over_sysml = over_root_65.to_sysml(0);
    assert!(over_sysml.contains("part def Part_63") && !over_sysml.contains("Part_64"));
    let over_digest = SchemaDigest::compute(&over_root_65, &over_sysml);
    assert_eq!(over_digest.node_counts.get("part_defs").copied(), Some(over_bound.saturating_sub(1)));

    let mut generic_64 = "u8".to_string();
    for _ in 0..bound.saturating_sub(1) {
        generic_64 = format!("Vec<{generic_64}>");
    }
    let generic_65 = format!("Vec<{generic_64}>");
    assert!(is_primitive_or_builtin_type(&generic_64) && !is_primitive_or_builtin_type(&generic_65));
    assert!(over_validator.is_resolvable_type("OverRoot65", &generic_64));
    assert!(!over_validator.is_resolvable_type("OverRoot65", &generic_65));
    drain_package_tree_iteratively(over_root_65);
}

type ScaleValidationSummary = (usize, usize, usize, usize, bool, bool, bool, bool, usize);

/// Executes 1,500-level hostile AST validation inside a 256 KiB stack thread without overflowing.
fn assert_hostile_1500_bounded_stack_thread(bound: usize) -> Result<(), String> {
    use crate::semantic::digest::count_structural_elements;
    use crate::semantic::serializer::SysmlSerializable;

    let hostile_levels: usize = 1_500;
    let mut hostile_pkg = PackageDef { name: format!("HostilePkg_{}", hostile_levels.saturating_sub(1)), ..Default::default() };
    for idx in (0..hostile_levels.saturating_sub(1)).rev() {
        hostile_pkg = PackageDef { name: format!("HostilePkg_{idx}"), packages: vec![hostile_pkg], ..Default::default() };
    }
    let mut hostile_part = PartDef { name: format!("HostilePart_{}", hostile_levels.saturating_sub(1)), type_name: Some("Real".to_string()), ..Default::default() };
    for idx in (1..hostile_levels.saturating_sub(1)).rev() {
        hostile_part = PartDef { name: format!("HostilePart_{idx}"), type_name: Some("Integer".to_string()), parts: vec![hostile_part], ..Default::default() };
    }
    let hostile_part_root = PackageDef { name: "HostilePartRoot".to_string(), part_defs: vec![hostile_part], ..Default::default() };
    let mut hostile_generic = "u8".to_string();
    for _ in 0..hostile_levels {
        hostile_generic = format!("Option<{hostile_generic}>");
    }

    let outcome = std::thread::scope(|s| -> Result<ScaleValidationSummary, String> {
        let handle = std::thread::Builder::new()
            .name("bounded_validator_1500".to_string())
            .stack_size(256 * 1024)
            .spawn_scoped(s, || {
                let sym_pkg = SymbolTable::build_from_package(&hostile_pkg);
                let sym_part = SymbolTable::build_from_package(&hostile_part_root);
                let mut v_pkg = SemanticValidator::new(&hostile_pkg);
                v_pkg.validate();
                let mut v_part = SemanticValidator::new(&hostile_part_root);
                v_part.validate();
                let generic_builtin = is_primitive_or_builtin_type(&hostile_generic);
                let generic_resolvable = v_part.is_resolvable_type("HostilePartRoot", &hostile_generic);
                let rendered_pkg = hostile_pkg.to_sysml(0);
                let _rendered_part = hostile_part_root.to_sysml(0);
                let hostile_elem_count = count_structural_elements(&hostile_part_root);
                (
                    sym_pkg.symbols.len(),
                    sym_part.symbols.len(),
                    v_pkg.errors().len(),
                    v_part.errors().len(),
                    generic_builtin,
                    generic_resolvable,
                    rendered_pkg.contains("package HostilePkg_63 {"),
                    rendered_pkg.contains("HostilePkg_64"),
                    hostile_elem_count,
                )
            })
            .map_err(|e| format!("failed to spawn bounded-stack validator thread: {e}"))?;
        handle.join().map_err(|_| "bounded-stack validator thread panicked or overflowed".to_string())
    })?;

    assert_eq!((outcome.0, outcome.1, outcome.2, outcome.3), (bound, bound, 0, 0));
    assert!(!outcome.4 && !outcome.5 && outcome.6 && !outcome.7);
    assert_eq!(outcome.8, hostile_levels.saturating_sub(1));
    drain_package_tree_iteratively(hostile_pkg);
    drain_package_tree_iteratively(hostile_part_root);
    Ok(())
}

#[test]
fn test_deep_nested_ast_hierarchy_bounded_without_stack_overflow() -> Result<(), String> {
    let bound: usize = 64;
    assert_depth_64_valid_package_and_part(bound);
    assert_depth_65_truncation_and_generics(bound);
    assert_hostile_1500_bounded_stack_thread(bound)
}

/// Builds a valid multi-byte UTF-8 `PackageDef` fixture with `@SafetyRealises` and `@TriggersHazard` annotations.
fn build_utf8_valid_package() -> PackageDef {
    PackageDef {
        name: "航空安全_Sys".to_string(),
        constraint_defs: vec![
            ConstraintDef { name: "UCA_01_Overspeed".to_string(), expression: "v <= 60.0".to_string(), ..Default::default() },
            ConstraintDef { name: "UCA_02_Thermal".to_string(), expression: "t <= 85.0".to_string(), ..Default::default() },
        ],
        hazard_defs: vec![HazardDef { name: "HZ_01_LossOfControl".to_string(), ..Default::default() }],
        requirement_defs: vec![RequirementDef {
            name: "Req_UTF8_Valid".to_string(),
            doc: Some("Δv ≤ 60.0 m/s 🛰 許容範囲 äöüß".to_string()),
            text: "飛行制御 Ω @SafetyRealises(UCA-01, UCA-02) および @triggershazard: hz-01 完了".to_string(),
            assumes: vec!["温度 ≥ -40°C @SAFETYREALISES: UCA-02".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn test_utf8_multibyte_annotations_and_unclosed_delimiters(
) -> Result<(), Box<dyn std::error::Error>> {
    let valid_pkg = build_utf8_valid_package();
    let mut valid_validator = SemanticValidator::new(&valid_pkg);
    valid_validator.validate();
    assert!(!valid_validator.has_errors());

    let dangling_pkg = PackageDef {
        name: "航空安全_Dangling".to_string(),
        constraint_defs: valid_pkg.constraint_defs.clone(),
        hazard_defs: valid_pkg.hazard_defs.clone(),
        requirement_defs: vec![RequirementDef {
            name: "Req_UTF8_Dangling".to_string(),
            text: "🛰ΔΩ@SafetyRealises(UCA-99)航空@TriggersHazard:HZ-99≤≥".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut dangling_validator = SemanticValidator::new(&dangling_pkg);
    dangling_validator.validate();
    assert!(dangling_validator.has_errors());
    let dangling_errors = dangling_validator.errors();
    assert!(dangling_errors.iter().any(|e| e.message.contains("UCA-99")));
    assert!(dangling_errors.iter().any(|e| e.message.contains("HZ-99")));

    let unclosed_pkg = PackageDef {
        name: "航空安全_Unclosed".to_string(),
        constraint_defs: valid_pkg.constraint_defs,
        hazard_defs: valid_pkg.hazard_defs,
        requirement_defs: vec![RequirementDef {
            name: "Req_UTF8_Unclosed".to_string(),
            text: "Δv ≤ 60.0 m/s 🛰 @SafetyRealises(UCA-01".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut unclosed_validator = SemanticValidator::new(&unclosed_pkg);
    unclosed_validator.validate();
    assert!(unclosed_validator.has_errors());
    assert!(unclosed_validator
        .errors()
        .iter()
        .any(|e| e.message.contains("unclosed '(' delimiter in @SafetyRealises")));

    let spans = extract_annotation_targets(
        "ΔΩ §Safety: UCA-01 および §Safety invalid §Safety(UCA-02, UCA-03)",
        "§Safety",
    );
    assert_eq!(
        spans,
        vec![
            AnnotationSpan::Targets(vec!["UCA-01".to_string()]),
            AnnotationSpan::Targets(vec!["UCA-02".to_string(), "UCA-03".to_string()]),
        ]
    );
    let unclosed_hazard_spans =
        extract_annotation_targets("航空安全 @TriggersHazard(HZ-01", "@TriggersHazard");
    assert_eq!(unclosed_hazard_spans, vec![AnnotationSpan::UnclosedDelimiter]);
    Ok(())
}
