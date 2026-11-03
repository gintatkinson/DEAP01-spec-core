//! Unit test suite B for `compile-sysml` crate root (`lib.rs`) deep flattening and MATLAB escaping.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Verifies DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D bounded-stack package flattening
//! (`ARCH-03`), MATLAB single-quote escaping (`ARCH-06`), and iterative pre-order part traversal.
//!
//! ## 2. Invariants & Mathematical Properties
//! - `flatten_package_to_model` preserves definitions at depth $d \ge 2$ and cross-package ports/items/requirements.
//! - `escape_matlab_single_quoted` doubles single quotes in `Simulink.BusElement` literals.
//! - `collect_model_parts_iterative` preserves exact pre-order traversal in $O(1)$ call-stack frames up to depth 4,000.
//!
//! ## 3. Preconditions
//! - Invoked via `cargo test` with scoped 256 KiB stack thread support.
//!
//! ## 4. Postconditions
//! - Deep `PartDef` trees are iteratively dismantled in $O(1)$ stack frames before drop.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(P + E)$ over packages and elements.
//! - Space Complexity: $O(P + E)$ heap memory for iterative work-stacks.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Propagates parse or thread join errors explicitly via `Result`.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-COMPILER-SYSML-TEST-ARCH-03-FLATTEN-DEEP], [REQ-COMPILER-SYSML-TEST-ARCH-06-MATLAB-ESCAPING]

use super::*;

/// Asserts that `model_all` and `model_suba` preserve all nested and cross-package AST nodes.
/// Realises: [REQ-COMPILER-SYSML-TEST-ARCH-03-FLATTEN-DEEP]
fn assert_arch_03_flattened_models(model_all: &SysmlModel, model_suba: &SysmlModel) {
    assert_eq!(model_all.package_name.as_deref(), Some("Root"));
    for part_name in ["RootPart", "PartA", "PartB", "PartC"] {
        assert!(model_all.parts.iter().any(|p| p.name == part_name), "missing {part_name}");
    }
    for port_name in ["RootPort", "PortA", "PortC"] {
        assert!(model_all.ports.iter().any(|p| p.name == port_name), "missing {port_name}");
    }
    for req_name in ["RootReq", "ReqB", "ReqC"] {
        assert!(model_all.requirements.iter().any(|r| r.name == req_name), "missing {req_name}");
    }
    for item_name in ["ItemC", "RootItem"] {
        assert!(model_all.parts.iter().flat_map(|p| &p.item_defs).any(|i| i.name == item_name));
    }

    assert_eq!(model_suba.package_name.as_deref(), Some("SubA"));
    assert!(!model_suba.parts.iter().any(|p| p.name == "RootPart"));
    for part_name in ["PartA", "PartB", "PartC"] {
        assert!(model_suba.parts.iter().any(|p| p.name == part_name), "SubA missing {part_name}");
    }
    for port_name in ["PortC", "RootPort"] {
        assert!(model_suba.ports.iter().any(|p| p.name == port_name), "SubA missing {port_name}");
    }
    for req_name in ["ReqB", "ReqC", "RootReq"] {
        assert!(model_suba.requirements.iter().any(|r| r.name == req_name), "SubA missing {req_name}");
    }
    for item_name in ["ItemC", "RootItem"] {
        assert!(model_suba.parts.iter().flat_map(|p| &p.item_defs).any(|i| i.name == item_name));
    }
}

#[test]
fn test_arch_03_flatten_package_to_model_deep_hierarchy(
) -> Result<(), Box<dyn std::error::Error>> {
    let src = r#"
        package Root {
            port def RootPort;
            item def RootItem { attribute version : Integer; }
            requirement def RootReq { id = "REQ-ROOT-01"; text = "Root level requirement."; }
            part def RootPart;
            package SubA {
                part def PartA;
                port def PortA;
                package SubB {
                    part def PartB;
                    requirement def ReqB { id = "REQ-B-01"; text = "Depth 2 requirement."; }
                    package SubC {
                        part def PartC { item def PartCInnerItem; }
                        port def PortC;
                        item def ItemC { attribute payload : Real; }
                        requirement def ReqC { id = "REQ-C-01"; text = "Depth 3 requirement."; }
                    }
                }
            }
        }
    "#;
    let pkg = parse_sysml(src)?;
    let model_all = flatten_package_to_model(pkg.clone(), None);
    let model_suba = flatten_package_to_model(pkg, Some("SubA"));
    assert_arch_03_flattened_models(&model_all, &model_suba);
    Ok(())
}

#[test]
fn test_arch_06_escape_matlab_single_quoted_and_mbd_bus_literals() {
    assert_eq!(
        escape_matlab_single_quoted("Controller's_State"),
        "Controller''s_State"
    );

    let part = PartDef {
        name: "PrimaryController".to_string(),
        attributes: vec![AttributeDef {
            name: "Controller's_State".to_string(),
            type_name: "Real's_Type".to_string(),
            default_value: None,
            doc: None,
        }],
        ..Default::default()
    };
    let model = SysmlModel {
        package_name: Some("CtrlPkg".to_string()),
        parts: vec![part],
        ..Default::default()
    };

    let bundle = synthesize_mbd_bundle(&model);
    assert!(
        bundle.simulink_bus_m.contains(
            "Simulink.BusElement('Name', 'Controller''s_State', 'DataType', 'Real''s_Type');"
        ),
        "simulink_bus_m must escape single quotes in MATLAB string literals:\n{}",
        bundle.simulink_bus_m
    );
}

/// Verifies bounded-stack deep `PartDef` traversal inside a 256 KiB stack thread for `depth`.
/// Realises: [REQ-COMPILER-SYSML-COLLECT-PARTS]
fn verify_bounded_stack_part_depth(depth: usize) -> Result<(), String> {
    let mut curr = PartDef {
        name: format!("Part_{}", depth.saturating_sub(1)),
        ..Default::default()
    };
    for idx in (0..depth.saturating_sub(1)).rev() {
        curr = PartDef {
            name: format!("Part_{idx}"),
            parts: vec![curr],
            ..Default::default()
        };
    }
    let mut deep_model = SysmlModel {
        package_name: Some("DeepStackPkg".to_string()),
        parts: vec![curr],
        ..Default::default()
    };

    let count_and_order = std::thread::scope(|s| -> Result<(usize, String, String), String> {
        let handle = std::thread::Builder::new()
            .name(format!("bounded_lib_parts_{depth}"))
            .stack_size(256 * 1024)
            .spawn_scoped(s, || {
                let b = synthesize_mbd_bundle(&deep_model);
                let names: Vec<String> = b
                    .simulink_bus_m
                    .lines()
                    .filter_map(|l| l.strip_suffix("_Bus = Simulink.Bus;"))
                    .map(str::to_string)
                    .collect();
                let first = names.first().cloned().unwrap_or_default();
                let last = names.last().cloned().unwrap_or_default();
                (names.len(), first, last)
            })
            .map_err(|e| format!("failed to spawn scoped thread: {e}"))?;
        handle
            .join()
            .map_err(|_| "bounded-stack thread panicked".to_string())
    })?;

    assert_eq!(count_and_order.0, depth);
    assert_eq!(count_and_order.1, "Part_0");
    assert_eq!(count_and_order.2, format!("Part_{}", depth.saturating_sub(1)));

    while let Some(mut node) = deep_model.parts.pop() {
        deep_model.parts.append(&mut node.parts);
    }
    Ok(())
}

#[test]
fn test_collect_model_parts_preorder_and_bounded_stack_deep_nesting() -> Result<(), String> {
    let make_part = |name: &str, children: Vec<PartDef>| PartDef {
        name: name.to_string(),
        parts: children,
        ..Default::default()
    };
    let tree = vec![
        make_part(
            "A",
            vec![
                make_part("A1", vec![make_part("A1a", vec![]), make_part("A1b", vec![])]),
                make_part("A2", vec![]),
            ],
        ),
        make_part("B", vec![make_part("B1", vec![])]),
    ];
    let model = SysmlModel {
        package_name: Some("PreOrderPkg".to_string()),
        parts: tree,
        ..Default::default()
    };
    let mut collected: Vec<&PartDef> = Vec::new();
    collect_model_parts_iterative(&model.parts, &mut collected);
    let direct_names: Vec<&str> = collected.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(direct_names, vec!["A", "A1", "A1a", "A1b", "A2", "B", "B1"]);
    let bundle = synthesize_mbd_bundle(&model);
    let bus_names: Vec<&str> = bundle
        .simulink_bus_m
        .lines()
        .filter_map(|line| line.strip_suffix("_Bus = Simulink.Bus;"))
        .collect();
    assert_eq!(bus_names, vec!["A", "A1", "A1a", "A1b", "A2", "B", "B1"]);

    for depth in [64usize, 65, 4_000] {
        verify_bounded_stack_part_depth(depth)?;
    }
    Ok(())
}
