//! Unit tests for composite type normalization, generic argument splitting, builtin recognition, and scoped type resolution.
//!
//! Safety & Traceability: Realises: [REQ-SEMANTIC-VALIDATOR-TESTS]
//! Contract: Verifies `normalize_source_type`, `split_top_level_type_args`, `is_primitive_or_builtin_type`, and `SemanticValidator::is_resolvable_type` across SysML v2, Rust, and Dart/Flutter types.
//! Bounds & Faults: Executes deterministic in-memory type resolution assertions with zero panics.

use super::*;

#[test]
fn test_issue_461_normalize_source_type() {
    assert_eq!(normalize_source_type("&str"), "str");
    assert_eq!(normalize_source_type("&mut Vec<u8>"), "Vec<u8>");
    assert_eq!(normalize_source_type("&const i32"), "i32");
    assert_eq!(normalize_source_type("&&str"), "str");
    assert_eq!(normalize_source_type("&'a str"), "str");
    assert_eq!(normalize_source_type("&'static str"), "str");
    assert_eq!(normalize_source_type("&'a mut [u8]"), "u8");
    assert_eq!(normalize_source_type("*const u8"), "u8");
    assert_eq!(normalize_source_type("*mut u8"), "u8");
    assert_eq!(normalize_source_type("[u8; 32]"), "u8");
    assert_eq!(normalize_source_type("[u8]"), "u8");
    assert_eq!(normalize_source_type("[[u8; 4]; 8]"), "u8");
    assert_eq!(normalize_source_type("Sensor[1..*]"), "Sensor");
    assert_eq!(normalize_source_type("u8[]"), "u8");
    assert_eq!(normalize_source_type("Sensor[0..1]"), "Sensor");
    assert_eq!(normalize_source_type("String?"), "String");
    assert_eq!(normalize_source_type("Widget?"), "Widget");
    assert_eq!(normalize_source_type("BuildContext?"), "BuildContext");
    assert_eq!(normalize_source_type("&'a mut [Option<String>?; 10]"), "Option<String>");
}

#[test]
fn test_issue_461_split_top_level_type_args() {
    assert_eq!(split_top_level_type_args("u8"), vec!["u8"]);
    assert_eq!(split_top_level_type_args("String, i32"), vec!["String", "i32"]);
    assert_eq!(split_top_level_type_args("<String, i32>"), vec!["String", "i32"]);
    assert_eq!(
        split_top_level_type_args("HashMap<String, Vec<u8>>, Result<Option<T>, E>"),
        vec!["HashMap<String, Vec<u8>>", "Result<Option<T>, E>"]
    );
    assert_eq!(split_top_level_type_args("(A, B), [C; 2]"), vec!["(A, B)", "[C; 2]"]);
    assert!(split_top_level_type_args("").is_empty());
    assert!(split_top_level_type_args("< >").is_empty());
}

#[test]
fn test_issue_461_is_primitive_or_builtin_type_scalars() {
    let scalars = [
        "void", "unit", "()", "double", "num", "char", "dynamic", "Object", "bool", "Boolean",
        "int", "Integer", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
        "u128", "usize", "float", "Float", "f32", "f64", "String", "string", "str", "ScalarValues",
    ];
    for ty in scalars {
        assert!(
            is_primitive_or_builtin_type(ty),
            "Expected scalar '{}' to be recognized as primitive or builtin",
            ty
        );
    }
}

#[test]
fn test_issue_461_is_primitive_or_builtin_type_rust_containers() {
    let containers = [
        "Vec", "Option", "Result", "Box", "Arc", "Rc", "Cow", "Mutex", "RwLock", "BTreeMap",
        "BTreeSet", "HashMap", "HashSet", "Path", "PathBuf", "Duration", "Instant", "SystemTime",
        "DateTime", "Error",
    ];
    for ty in containers {
        assert!(
            is_primitive_or_builtin_type(ty),
            "Expected Rust container '{}' to be recognized as builtin",
            ty
        );
    }
}

#[test]
fn test_issue_461_is_primitive_or_builtin_type_dart_collections_and_async() {
    for ty in ["List", "Map", "Set", "Iterable", "Future", "Stream"] {
        assert!(
            is_primitive_or_builtin_type(ty),
            "Expected Dart/Async type '{}' to be recognized as builtin",
            ty
        );
    }
}

#[test]
fn test_issue_461_is_primitive_or_builtin_type_flutter_framework() {
    let flutter_types = [
        "Widget", "StatefulWidget", "StatelessWidget", "State", "BuildContext", "Key", "GlobalKey",
        "LocalKey", "ValueKey", "VoidCallback", "ValueChanged", "ValueGetter", "ValueSetter",
        "WidgetBuilder", "IndexedWidgetBuilder", "TransitionBuilder", "Color", "IconData",
        "EdgeInsets", "EdgeInsetsGeometry", "BoxConstraints", "Size", "Offset", "Rect",
        "TextStyle", "ThemeData", "TextEditingController", "ScrollController", "FocusNode",
        "AnimationController",
    ];
    for ty in flutter_types {
        assert!(
            is_primitive_or_builtin_type(ty),
            "Expected Flutter UI type '{}' to be recognized as builtin",
            ty
        );
    }
}

#[test]
fn test_issue_461_is_primitive_or_builtin_type_module_prefixes() {
    let prefixed_types = [
        "ScalarValues::Boolean",
        "ISQ::LengthValue",
        "SI::m",
        "std::path::PathBuf",
        "std::collections::HashMap",
        "std::sync::Arc",
        "std::time::Duration",
        "core::option::Option",
        "core::result::Result",
        "alloc::vec::Vec",
        "alloc::string::String",
    ];
    for ty in prefixed_types {
        assert!(
            is_primitive_or_builtin_type(ty),
            "Expected prefixed type '{}' to be recognized as builtin",
            ty
        );
    }
}

#[test]
fn test_issue_461_is_primitive_or_builtin_type_composite_generics() {
    assert!(is_primitive_or_builtin_type("Vec<u8>"));
    assert!(is_primitive_or_builtin_type("Option<String>"));
    assert!(is_primitive_or_builtin_type("Result<i32, Error>"));
    assert!(is_primitive_or_builtin_type("HashMap<String, Vec<u8>>"));
    assert!(is_primitive_or_builtin_type("List<Widget>"));
    assert!(is_primitive_or_builtin_type("Future<void>"));
    assert!(is_primitive_or_builtin_type("Stream<String>"));
}

#[test]
fn test_issue_461_is_primitive_or_builtin_type_negative_unknown_types() {
    assert!(!is_primitive_or_builtin_type("UnknownCustomSensor"));
    assert!(!is_primitive_or_builtin_type("Vec<UnknownCustomSensor>"));
    assert!(!is_primitive_or_builtin_type("HashMap<String, UnknownCustomSensor>"));
    assert!(!is_primitive_or_builtin_type("Future<UnknownCustomSensor>"));
}

#[test]
fn test_issue_461_is_resolvable_type_composite_and_generic_builtins() {
    let pkg = PackageDef::default();
    let validator = SemanticValidator::new(&pkg);

    for ty in [
        "i32", "f64", "String", "Widget", "BuildContext", "StatefulWidget", "()", "&Widget",
        "&mut String", "*const i32", "Widget?", "[u8; 32]", "&'a str", "Option<String>",
        "Vec<u8>", "Result<i32, String>", "HashMap<String, Vec<u8>>", "Option<Widget>?",
    ] {
        assert!(validator.is_resolvable_type("", ty), "Expected '{ty}' to resolve");
    }
}

#[test]
fn test_issue_461_is_resolvable_type_user_defined_symbols_in_scope() {
    let pkg = PackageDef {
        name: "Subsystem".to_string(),
        part_defs: vec![
            PartDef { name: "SensorNode".to_string(), is_def: true, ..Default::default() },
            PartDef { name: "SafetyController".to_string(), is_def: true, ..Default::default() },
        ],
        ..Default::default()
    };
    let validator = SemanticValidator::new(&pkg);

    for ty in [
        "SensorNode",
        "Subsystem::SensorNode",
        "Vec<SensorNode>",
        "Option<SafetyController>",
        "Result<SensorNode, String>",
        "HashMap<String, Vec<SensorNode>>",
        "&Vec<SensorNode>",
        "Option<SensorNode>?",
        "[SensorNode; 4]",
    ] {
        assert!(validator.is_resolvable_type("Subsystem", ty), "Expected '{ty}' to resolve");
    }
}

#[test]
fn test_issue_461_is_resolvable_type_fail_closed_negative_mutations() {
    let pkg = PackageDef {
        name: "Subsystem".to_string(),
        part_defs: vec![PartDef {
            name: "SensorNode".to_string(),
            is_def: true,
            ..Default::default()
        }],
        ..Default::default()
    };
    let validator = SemanticValidator::new(&pkg);

    for bad_ty in [
        "UndefinedType",
        "&UndefinedType",
        "UndefinedType?",
        "Vec<UndefinedType>",
        "Option<UndefinedType>",
        "Result<SensorNode, UndefinedError>",
        "Result<UndefinedNode, String>",
        "HashMap<String, Vec<UndefinedType>>",
        "UndefinedContainer<String>",
        "UndefinedContainer<SensorNode>",
        "Vec<>",
    ] {
        assert!(
            !validator.is_resolvable_type("Subsystem", bad_ty),
            "Expected '{bad_ty}' to fail closed"
        );
    }
}

#[test]
fn test_issue_461_validate_part_defs_with_composite_generic_types_passes() {
    let pkg = PackageDef {
        name: "Platform".to_string(),
        part_defs: vec![
            PartDef { name: "SensorNode".to_string(), is_def: true, ..Default::default() },
            PartDef {
                name: "nodes".to_string(),
                is_def: false,
                type_name: Some("Vec<SensorNode>".to_string()),
                ..Default::default()
            },
            PartDef {
                name: "active_node".to_string(),
                is_def: false,
                type_name: Some("Option<SensorNode>".to_string()),
                ..Default::default()
            },
            PartDef {
                name: "view".to_string(),
                is_def: false,
                type_name: Some("Widget".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(!validator.has_errors(), "Expected zero validation errors, got: {:?}", validator.errors());
}

#[test]
fn test_issue_461_validate_part_defs_with_undefined_generic_inner_fails() {
    let pkg = PackageDef {
        name: "Platform".to_string(),
        part_defs: vec![PartDef {
            name: "bad_nodes".to_string(),
            is_def: false,
            type_name: Some("Vec<GhostType>".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(validator.has_errors());
    assert!(validator
        .errors()
        .iter()
        .any(|e| e.message.contains("Part 'bad_nodes' references undefined type 'Vec<GhostType>'")));
}

#[test]
fn test_issue_461_validate_requirements_with_composite_generic_attributes_passes() {
    let pkg = PackageDef {
        name: "Safety".to_string(),
        part_defs: vec![PartDef { name: "Guard".to_string(), is_def: true, ..Default::default() }],
        requirement_defs: vec![RequirementDef {
            name: "Req_Safety".to_string(),
            text: "Shall maintain logs.".to_string(),
            attributes: vec![
                AttributeDef {
                    name: "log_entries".to_string(),
                    type_name: "Vec<String>".to_string(),
                    ..Default::default()
                },
                AttributeDef {
                    name: "guard_ref".to_string(),
                    type_name: "Option<Guard>".to_string(),
                    ..Default::default()
                },
            ],
            satisfied_by: vec!["Guard".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let mut validator = SemanticValidator::new(&pkg);
    validator.validate();
    assert!(!validator.has_errors(), "Expected zero validation errors, got: {:?}", validator.errors());
}
