//! Adversarial edge-case regression tests for composite type normalization, tuples, bare modifiers, and trait objects.
//!
//! Safety & Traceability: Realises: [REQ-SEMANTIC-VALIDATOR-TESTS]
//! Contract: Verifies fail-closed rejection of trailing tokens, bare modifiers, and undefined tuple/trait inner types alongside positive resolution of pointer arrays, slice multiplicities, and `dyn` trait objects.
//! Bounds & Faults: Executes deterministic in-memory semantic validation assertions with zero panics.

use super::*;

#[test]
fn test_issue_461_iter2_edge_1_trailing_tokens_after_generics_fail_closed() {
    let pkg = PackageDef {
        name: "TestScope".to_string(),
        part_defs: vec![
            PartDef { name: "ValidPart".to_string(), is_def: true, ..Default::default() },
            PartDef {
                name: "bad_part_trailing_undefined".to_string(),
                is_def: false,
                type_name: Some("Option<String> UndefinedType".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(validator.has_errors());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("references undefined type 'Option<String> UndefinedType'")));

    let validator = SemanticValidator::new(&pkg);
    assert!(!validator.is_resolvable_type("TestScope", "Option<String> UndefinedType"));
    assert!(!validator.is_resolvable_type("TestScope", "Vec<u8> junk"));
    assert!(!validator.is_resolvable_type("TestScope", "HashMap<String, i32> trailing"));
    assert!(!validator.is_resolvable_type("TestScope", "Vec<ValidPart> GhostPart"));

    assert!(!is_primitive_or_builtin_type("Option<String> UndefinedType"));
    assert!(!is_primitive_or_builtin_type("Vec<u8> junk"));
    assert!(!is_primitive_or_builtin_type("HashMap<String, i32> trailing"));
}

#[test]
fn test_issue_461_edge_1_trailing_tokens_after_generics_fail_closed() {
    test_issue_461_iter2_edge_1_trailing_tokens_after_generics_fail_closed();
}

#[test]
fn test_issue_461_iter2_edge_2_tuples_with_generics_resolve() {
    let pkg = PackageDef {
        name: "TestScope".to_string(),
        part_defs: vec![
            PartDef { name: "ValidPart".to_string(), is_def: true, ..Default::default() },
            PartDef {
                name: "tuple_part".to_string(),
                is_def: false,
                type_name: Some("(ValidPart, Vec<String>)".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(!validator.has_errors(), "Expected zero validation errors, got: {:?}", validator.errors());

    let validator = SemanticValidator::new(&pkg);
    assert!(validator.is_resolvable_type("TestScope", "(ValidPart, Vec<String>)"));
    assert!(validator.is_resolvable_type("TestScope", "(i32, Option<ValidPart>)"));
    assert!(validator.is_resolvable_type(
        "TestScope",
        "(Vec<ValidPart>, HashMap<String, Option<i32>>)"
    ));

    assert!(!validator.is_resolvable_type("TestScope", "(ValidPart, Vec<UndefinedPart>)"));
    assert!(!validator.is_resolvable_type("TestScope", "(UndefinedPart, Vec<String>)"));
    assert!(!validator.is_resolvable_type("TestScope", "(i32, Option<GhostSymbol>)"));
}

#[test]
fn test_issue_461_edge_2_tuples_with_generics_resolve() {
    test_issue_461_iter2_edge_2_tuples_with_generics_resolve();
}

#[test]
fn test_issue_461_iter2_edge_3_bare_modifiers_and_empty_types_fail_closed() {
    let pkg = PackageDef::default();
    let validator = SemanticValidator::new(&pkg);

    for bare in ["&", "*const", "*mut", "&mut", "&const", "[]", "?", "", "   "] {
        assert!(!validator.is_resolvable_type("", bare));
        assert!(!is_primitive_or_builtin_type(bare));
    }

    assert!(!validator.is_resolvable_type("", "Option<&>"));
    assert!(!validator.is_resolvable_type("", "Option<[]>"));
    assert!(!validator.is_resolvable_type("", "Vec<?>"));
    assert!(!validator.is_resolvable_type("", "Result<&, []>"));

    let invalid_pkg = PackageDef {
        name: "InvalidPkg".to_string(),
        part_defs: vec![PartDef {
            name: "bad_mod_part".to_string(),
            is_def: false,
            type_name: Some("&".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut invalid_validator = SemanticValidator::new(&invalid_pkg);
    invalid_validator.validate();
    assert!(invalid_validator.has_errors());
}

#[test]
fn test_issue_461_edge_3_bare_modifiers_and_empty_types_fail_closed() {
    test_issue_461_iter2_edge_3_bare_modifiers_and_empty_types_fail_closed();
}

#[test]
fn test_issue_461_iter2_edge_4_pointer_array_wrappers_normalize_and_resolve() {
    let pkg = PackageDef {
        name: "TestScope".to_string(),
        part_defs: vec![PartDef { name: "ValidPart".to_string(), is_def: true, ..Default::default() }],
        ..Default::default()
    };

    assert_eq!(normalize_source_type("*const[ValidPart; 10]"), "ValidPart");
    assert_eq!(normalize_source_type("*mut[u8; 16]"), "u8");
    assert_eq!(normalize_source_type("&mut[ValidPart; 4]"), "ValidPart");
    assert_eq!(normalize_source_type("&const[i32; 8]"), "i32");
    assert_eq!(normalize_source_type("*const[TotallyUndefined; 10]"), "TotallyUndefined");

    let validator = SemanticValidator::new(&pkg);
    assert!(validator.is_resolvable_type("TestScope", "*const[ValidPart; 10]"));
    assert!(validator.is_resolvable_type("TestScope", "*mut[ValidPart; 10]"));
    assert!(validator.is_resolvable_type("TestScope", "&mut[ValidPart; 10]"));
    assert!(validator.is_resolvable_type("TestScope", "&const[ValidPart; 10]"));
    assert!(validator.is_resolvable_type("TestScope", "*const[u8; 32]"));

    assert!(!validator.is_resolvable_type("TestScope", "*const[TotallyUndefined; 10]"));
    assert!(!validator.is_resolvable_type("TestScope", "*mut[TotallyUndefined; 10]"));
    assert!(!validator.is_resolvable_type("TestScope", "&mut[TotallyUndefined; 10]"));
    assert!(!validator.is_resolvable_type("TestScope", "&const[TotallyUndefined; 10]"));
}

#[test]
fn test_issue_461_edge_4_pointer_array_wrappers_normalize_and_resolve() {
    test_issue_461_iter2_edge_4_pointer_array_wrappers_normalize_and_resolve();
}

#[test]
fn test_issue_461_iter2_edge_5_slice_multiplicity_normalization_and_resolution() {
    let pkg = PackageDef {
        name: "TestScope".to_string(),
        part_defs: vec![PartDef { name: "SensorNode".to_string(), is_def: true, ..Default::default() }],
        ..Default::default()
    };

    assert_eq!(normalize_source_type("[u8][0..1]"), "u8");
    assert_eq!(normalize_source_type("[u8][1..*]"), "u8");
    assert_eq!(normalize_source_type("[u8][]"), "u8");
    assert_eq!(normalize_source_type("[SensorNode][0..1]"), "SensorNode");
    assert_eq!(normalize_source_type("[SensorNode][1..*]"), "SensorNode");
    assert_eq!(normalize_source_type("&[u8][0..1]"), "u8");

    let validator = SemanticValidator::new(&pkg);
    assert!(validator.is_resolvable_type("TestScope", "[u8][0..1]"));
    assert!(validator.is_resolvable_type("TestScope", "[u8][1..*]"));
    assert!(validator.is_resolvable_type("TestScope", "[SensorNode][0..1]"));
    assert!(validator.is_resolvable_type("TestScope", "[SensorNode][1..*]"));

    assert!(!validator.is_resolvable_type("TestScope", "[UndefinedPart][0..1]"));
    assert!(!validator.is_resolvable_type("TestScope", "[UndefinedPart][1..*]"));
}

#[test]
fn test_issue_461_edge_5_slice_multiplicity_normalization_and_resolution() {
    test_issue_461_iter2_edge_5_slice_multiplicity_normalization_and_resolution();
}

#[test]
fn test_issue_461_iter2_edge_6_split_top_level_type_args_multiple_bracket_groups() {
    assert_eq!(split_top_level_type_args("<A>, <B>"), vec!["<A>", "<B>"]);
    assert_eq!(split_top_level_type_args("<A, B>, <C, D>"), vec!["<A, B>", "<C, D>"]);
    assert_eq!(
        split_top_level_type_args("<A>, <B, <C, D>>, <E>"),
        vec!["<A>", "<B, <C, D>>", "<E>"]
    );
    assert_eq!(split_top_level_type_args("<A, B>"), vec!["A", "B"]);
    assert_eq!(split_top_level_type_args("<<A, B>>"), vec!["<A, B>"]);
    assert_eq!(split_top_level_type_args("A, B, C"), vec!["A", "B", "C"]);
    assert_eq!(
        split_top_level_type_args("HashMap<String, Vec<u8>>, Result<Option<T>, E>"),
        vec!["HashMap<String, Vec<u8>>", "Result<Option<T>, E>"]
    );
}

#[test]
fn test_issue_461_edge_6_split_top_level_type_args_multiple_bracket_groups() {
    test_issue_461_iter2_edge_6_split_top_level_type_args_multiple_bracket_groups();
}

#[test]
fn test_issue_461_iter2_edge_7_trait_objects_with_dyn_resolve() {
    let pkg = PackageDef {
        name: "TestScope".to_string(),
        part_defs: vec![PartDef { name: "ValidPart".to_string(), is_def: true, ..Default::default() }],
        ..Default::default()
    };

    assert_eq!(normalize_source_type("dyn Error"), "Error");
    assert_eq!(normalize_source_type("&dyn Error"), "Error");
    assert_eq!(normalize_source_type("Box<dyn Error>"), "Box<dyn Error>");
    assert_eq!(
        normalize_source_type("Result<Option<HashMap<String, Vec<u8>>>, Box<dyn Error>>"),
        "Result<Option<HashMap<String, Vec<u8>>>, Box<dyn Error>>"
    );

    let validator = SemanticValidator::new(&pkg);
    assert!(validator.is_resolvable_type("", "Box<dyn Error>"));
    assert!(validator.is_resolvable_type("", "Arc<dyn Error>"));
    assert!(validator.is_resolvable_type(
        "",
        "Result<Option<HashMap<String, Vec<u8>>>, Box<dyn Error>>"
    ));
    assert!(validator.is_resolvable_type("TestScope", "Result<ValidPart, Box<dyn Error>>"));

    assert!(!validator.is_resolvable_type("", "Box<dyn UndefinedTrait>"));
    assert!(!validator.is_resolvable_type("TestScope", "Result<ValidPart, Box<dyn UndefinedTrait>>"));
}

#[test]
fn test_issue_461_edge_7_trait_objects_with_dyn_resolve() {
    test_issue_461_iter2_edge_7_trait_objects_with_dyn_resolve();
}

#[test]
fn test_issue_461_iter2_edge_8_tuples_in_primitive_and_generic_containers() {
    let pkg = PackageDef {
        name: "TestScope".to_string(),
        part_defs: vec![PartDef { name: "ValidPart".to_string(), is_def: true, ..Default::default() }],
        ..Default::default()
    };

    assert!(is_primitive_or_builtin_type("(u32, bool)"));
    assert!(is_primitive_or_builtin_type("(i32, f64, String)"));
    assert!(is_primitive_or_builtin_type("Vec<(u32, bool)>"));
    assert!(is_primitive_or_builtin_type("Option<(i32, f64)>"));
    assert!(is_primitive_or_builtin_type("HashMap<String, (u8, u16)>"));

    assert!(!is_primitive_or_builtin_type("(u32, UnknownType)"));
    assert!(!is_primitive_or_builtin_type("Vec<(u32, UnknownType)>"));

    let validator = SemanticValidator::new(&pkg);
    assert!(validator.is_resolvable_type("", "Vec<(u32, bool)>"));
    assert!(validator.is_resolvable_type("TestScope", "Vec<(ValidPart, bool)>"));
    assert!(validator.is_resolvable_type("TestScope", "Option<(ValidPart, String)>"));
    assert!(!validator.is_resolvable_type("TestScope", "Vec<(UndefinedPart, bool)>"));
}

#[test]
fn test_issue_461_edge_8_tuples_in_primitive_and_generic_containers() {
    test_issue_461_iter2_edge_8_tuples_in_primitive_and_generic_containers();
}

#[test]
fn test_issue_461_iter2_all_iteration_1_edge_cases_comprehensive_matrix() {
    let pkg = PackageDef {
        name: "MatrixScope".to_string(),
        part_defs: vec![
            PartDef { name: "DeclaredSensor".to_string(), is_def: true, ..Default::default() },
            PartDef { name: "DeclaredActuator".to_string(), is_def: true, ..Default::default() },
            PartDef {
                name: "sensor_tuple".to_string(),
                is_def: false,
                type_name: Some("(DeclaredSensor, Vec<String>)".to_string()),
                ..Default::default()
            },
            PartDef {
                name: "pointer_array".to_string(),
                is_def: false,
                type_name: Some("*const[DeclaredActuator; 8]".to_string()),
                ..Default::default()
            },
            PartDef {
                name: "slice_multiplicity".to_string(),
                is_def: false,
                type_name: Some("[DeclaredSensor][1..*]".to_string()),
                ..Default::default()
            },
            PartDef {
                name: "error_handler".to_string(),
                is_def: false,
                type_name: Some("Box<dyn Error>".to_string()),
                ..Default::default()
            },
            PartDef {
                name: "coordinate_stream".to_string(),
                is_def: false,
                type_name: Some("Stream<(f64, f64)>".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(
        !validator.has_errors(),
        "Expected zero validation errors for comprehensive edge cases matrix, got: {:?}",
        validator.errors()
    );
}

#[test]
fn test_issue_461_all_iteration_1_edge_cases_comprehensive_matrix() {
    test_issue_461_iter2_all_iteration_1_edge_cases_comprehensive_matrix();
}
