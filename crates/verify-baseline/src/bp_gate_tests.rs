//! Unit tests for `blueprint_gate` AST normalization and contract line scanning.
//! Realises: [REQ-VERIFY-BASELINE-BLUEPRINT-GATE]

use crate::bp_gate_checks_c::{has_split_token, scan_rust_contract_line};
use crate::bp_gate_sysml::{
    normalize_package_for_bijective_roundtrip, normalize_part_for_bijective_roundtrip,
    NormalizeDepthError, MAX_NORMALIZE_DEPTH,
};

fn build_nested_part_chain(
    nesting_depth: usize,
) -> Result<deap_core::sysml_ast::PartDef, String> {
    let parsed = compile_sysml::parse_sysml(
        "package P { part def Node { attribute count : Integer; port link : TelemetryPort; } }",
    )
    .map_err(|e| format!("parse error: {e}"))?;
    let template = parsed
        .part_defs
        .first()
        .cloned()
        .ok_or_else(|| "missing template PartDef".to_string())?;
    let mut current = template.clone();
    if let Some(first_attr) = current.attributes.first_mut() {
        first_attr.doc = Some("leaf attribute doc".to_string());
    }
    if let Some(first_port) = current.ports.first_mut() {
        first_port.protocol_family = "ARINC429".to_string();
    }
    for _ in 0..nesting_depth {
        let mut parent = template.clone();
        if let Some(first_attr) = parent.attributes.first_mut() {
            first_attr.doc = Some("parent attribute doc".to_string());
        }
        parent.parts.push(current);
        current = parent;
    }
    Ok(current)
}

fn build_nested_package_chain(
    nesting_depth: usize,
) -> Result<deap_core::sysml_ast::PackageDef, String> {
    let template = compile_sysml::parse_sysml("package SubPkg { port def BusPort; }")
        .map_err(|e| format!("parse error: {e}"))?;
    let mut current = template.clone();
    if let Some(first_port) = current.port_defs.first_mut() {
        first_port.protocol_family = "SPACEWIRE".to_string();
    }
    for _ in 0..nesting_depth {
        let mut parent = template.clone();
        parent.packages.push(current);
        current = parent;
    }
    Ok(current)
}

#[test]
fn test_normalize_part_depth_at_max_and_plus_one() -> Result<(), String> {
    let mut part_at_max = build_nested_part_chain(MAX_NORMALIZE_DEPTH)?;
    let res_max = normalize_part_for_bijective_roundtrip(&mut part_at_max, 0);
    assert_eq!(res_max, Ok(()));

    let mut leaf_cursor = &part_at_max;
    for _ in 0..MAX_NORMALIZE_DEPTH {
        leaf_cursor = leaf_cursor
            .parts
            .first()
            .ok_or_else(|| "expected nested child part".to_string())?;
    }
    assert_eq!(leaf_cursor.doc.as_deref(), Some("leaf attribute doc"));
    let leaf_port = leaf_cursor
        .ports
        .first()
        .ok_or_else(|| "expected leaf port".to_string())?;
    assert!(leaf_port.protocol_family.is_empty());
    assert_eq!(leaf_port.doc.as_deref(), Some("[protocol: ARINC429]"));

    let over_depth = MAX_NORMALIZE_DEPTH.saturating_add(1);
    let mut part_over_max = build_nested_part_chain(over_depth)?;
    let res_over = normalize_part_for_bijective_roundtrip(&mut part_over_max, 0);
    assert_eq!(
        res_over,
        Err(NormalizeDepthError::MaxDepthExceeded {
            depth: over_depth,
            max_depth: MAX_NORMALIZE_DEPTH,
        })
    );
    assert_eq!(part_over_max.doc, None);
    Ok(())
}

#[test]
fn test_normalize_package_depth_at_max_and_plus_one() -> Result<(), String> {
    let mut pkg_at_max = build_nested_package_chain(MAX_NORMALIZE_DEPTH)?;
    let res_max = normalize_package_for_bijective_roundtrip(&mut pkg_at_max, 0);
    assert_eq!(res_max, Ok(()));

    let mut leaf_pkg = &pkg_at_max;
    for _ in 0..MAX_NORMALIZE_DEPTH {
        leaf_pkg = leaf_pkg
            .packages
            .first()
            .ok_or_else(|| "expected nested sub-package".to_string())?;
    }
    let leaf_port = leaf_pkg
        .port_defs
        .first()
        .ok_or_else(|| "expected leaf port_def".to_string())?;
    assert!(leaf_port.protocol_family.is_empty());

    let over_depth = MAX_NORMALIZE_DEPTH.saturating_add(1);
    let mut pkg_over_max = build_nested_package_chain(over_depth)?;
    let res_over = normalize_package_for_bijective_roundtrip(&mut pkg_over_max, 0);
    assert_eq!(
        res_over,
        Err(NormalizeDepthError::MaxDepthExceeded {
            depth: over_depth,
            max_depth: MAX_NORMALIZE_DEPTH,
        })
    );
    Ok(())
}

#[test]
fn test_has_split_token_and_contract_scanner() {
    assert!(has_split_token("alpha_beta_gamma", "alpha_", "beta"));
    assert!(has_split_token(
        "\u{1F680}prefix_suffix",
        "prefix_",
        "suffix"
    ));
    assert!(!has_split_token("prefix_gap_suffix", "prefix_", "suffix"));
    assert!(!has_split_token("short", "longer_prefix", "suffix"));

    let mut lex_state = deap_core::rust_cst::source_bridge::CstLexState::default();
    let mut errors = Vec::new();
    let bad_regex_line = format!("let r = {}{}(\"x\");", "Re", "gex::new");
    scan_rust_contract_line(&bad_regex_line, 1, &mut lex_state, &mut errors);
    assert_eq!(errors.len(), 1);

    let mut panic_errors = Vec::new();
    let bad_unwrap_line = format!("let v = opt{}{};", ".un", "wrap()");
    scan_rust_contract_line(&bad_unwrap_line, 2, &mut lex_state, &mut panic_errors);
    assert_eq!(panic_errors.len(), 1);
}
