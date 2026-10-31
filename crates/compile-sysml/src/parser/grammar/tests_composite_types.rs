//! Composite type-expression, fail-closed container keyword, and `satisfy ... by` round-trip grammar tests.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-TESTS]
//! Contract: Verifies comment-interleaved `satisfy ... by <Part>` clauses, composite/generic/pointer/reference/tuple/slice type expressions across all declaration kinds, and fail-closed `Err(ParseError)` rejection on unknown container keywords.
//! Bounds & Faults: Executes deterministic in-memory parsing and serialization round-trip assertions with zero panics.

#[cfg(test)]
mod composite_and_container_tests {
    use super::super::*;

    #[test]
    fn test_issue_460_comments_surrounding_by_clause() -> Result<(), Box<dyn std::error::Error>> {
        let test_cases = [
            (
                "satisfy requirement REQ-01 /* comment before by */ by NavCtrl;",
                "REQ-01",
            ),
            (
                "satisfy requirement REQ-02 // line before by\n by NavCtrl;",
                "REQ-02",
            ),
            (
                "satisfy requirement REQ-03 /* multi\nline */ by NavCtrl;",
                "REQ-03",
            ),
            (
                "satisfy requirement REQ-04 by /* comment between by and part */ NavCtrl;",
                "REQ-04",
            ),
            (
                "satisfy requirement REQ-05 by // line between by and part\n NavCtrl;",
                "REQ-05",
            ),
            (
                "satisfy requirement REQ-06 by /* multi\nline */ NavCtrl;",
                "REQ-06",
            ),
            (
                "satisfy requirement REQ-07 by NavCtrl /* comment after part before semi */;",
                "REQ-07",
            ),
            (
                "satisfy requirement REQ-08 by NavCtrl // line after part before semi\n ;",
                "REQ-08",
            ),
            (
                "satisfy requirement REQ-09 by NavCtrl /* multi\nline */;",
                "REQ-09",
            ),
            (
                "satisfy requirement REQ-10 /* c1 */ by /* c2 */ NavCtrl /* c3 */;",
                "REQ-10",
            ),
            (
                "satisfy requirement REQ-11 // l1\n by // l2\n NavCtrl // l3\n ;",
                "REQ-11",
            ),
            (
                "satisfy requirement REQ-12 /* by */ by /* satisfy */ NavCtrl /* ; */;",
                "REQ-12",
            ),
            (
                "satisfy requirement \"REQ-UI-01\" /* c1 */ by /* c2 */ NavCtrl /* c3 */;",
                "REQ-UI-01",
            ),
        ];

        for (stmt, expected_id) in test_cases {
            let src = format!("package P {{ part def C {{ {} }} }}", stmt);
            let pkg = SysmlParser::parse_source(&src, "P")?;
            assert_eq!(
                pkg.part_defs.first().ok_or("missing part")?.satisfied_requirements,
                vec![expected_id.to_string()],
                "Failed to parse satisfy statement with comments surrounding 'by <Part>' clause: {}",
                stmt
            );
        }
        Ok(())
    }

    #[test]
    fn test_issue_460_comments_package_and_part_def_roundtrip_stress(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::semantic::serializer::SysmlSerializable;

        let source = r#"
package AdversarialCommentsPackage {
    // Package-level satisfy statement with inline block comment
    satisfy /* pkg comment 1 */ requirement REQ-PKG-01 /* pkg comment 2 */;

    // Package-level satisfy statement with 'by' clause and comments
    satisfy requirement REQ-PKG-02 /* before by */ by CorePlatform /* after by */;

    part def CockpitSubsystem {
        // Position 1: between satisfy and requirement
        satisfy /* block comment */ requirement REQ-UI-01;
        satisfy // line comment
            requirement REQ-UI-02;
        satisfy /* multi
            line
            comment */ requirement REQ-UI-03;

        // Position 2: between requirement and ID
        satisfy requirement /* block comment */ REQ-UI-04;
        satisfy /* without req keyword */ REQ-UI-05;

        // Position 3: between ID tokens and hyphens
        satisfy requirement REQ /* c1 */ - /* c2 */ UI /* c3 */ - /* c4 */ 06;
        satisfy requirement Core /* c1 */ :: /* c2 */ UI /* c3 */ :: /* c4 */ REQ-UI-07;

        // Position 4: before terminating semicolon
        satisfy requirement REQ-UI-08 /* comment before semi */;

        // Position 5: before and after 'by <Part>'
        satisfy requirement REQ-UI-09 /* c1 */ by /* c2 */ TouchDisplay /* c3 */;

        // Deduplication: statement with comments matches clean statement
        satisfy requirement REQ-UI-01;
        satisfy /* dupe comment */ requirement REQ-UI-01;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "AdversarialCommentsPackage")?;

        assert_eq!(
            pkg.satisfied_requirements,
            vec!["REQ-PKG-01".to_string(), "REQ-PKG-02".to_string()],
            "package-level satisfy requirements must be extracted cleanly without comment artifacts"
        );

        let cockpit = pkg
            .part_defs
            .iter()
            .find(|p| p.name == "CockpitSubsystem")
            .ok_or("missing CockpitSubsystem")?;

        let expected_part_reqs = vec![
            "REQ-UI-01".to_string(),
            "REQ-UI-02".to_string(),
            "REQ-UI-03".to_string(),
            "REQ-UI-04".to_string(),
            "REQ-UI-05".to_string(),
            "REQ-UI-06".to_string(),
            "Core::UI::REQ-UI-07".to_string(),
            "REQ-UI-08".to_string(),
            "REQ-UI-09".to_string(),
        ];
        assert_eq!(
            cockpit.satisfied_requirements, expected_part_reqs,
            "cockpit satisfied_requirements must match expected list with deduplication"
        );

        let serialized = pkg.to_sysml(0);
        assert!(serialized.contains("satisfy requirement REQ-PKG-01;"));
        assert!(serialized.contains("satisfy requirement REQ-PKG-02;"));
        assert!(serialized.contains("satisfy requirement REQ-UI-01;"));
        assert!(serialized.contains("satisfy requirement REQ-UI-06;"));
        assert!(serialized.contains("satisfy requirement Core::UI::REQ-UI-07;"));

        let reparsed = SysmlParser::parse_source(&serialized, "AdversarialCommentsPackage")?;
        assert_eq!(reparsed.satisfied_requirements, pkg.satisfied_requirements);
        let reparsed_cockpit = reparsed
            .part_defs
            .iter()
            .find(|p| p.name == "CockpitSubsystem")
            .ok_or("missing reparsed CockpitSubsystem")?;
        assert_eq!(
            reparsed_cockpit.satisfied_requirements,
            cockpit.satisfied_requirements
        );

        Ok(())
    }

    /// Verifies that `SysmlParser` deterministically parses generic, pointer, reference, slice, array, tuple, `dyn`, path-qualified, and optional `?` type expressions across declarations.
    #[test]
    fn test_defect_05_parse_composite_and_generic_types_across_declarations(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package CompositeTypesPkg {
    attribute items : Vec<T>;
    attribute raw_slice_ptr : *const [T; 10];
    attribute byte_slice : &[u8];
    attribute outcome : Result<T, E>;
    attribute opt_label : Option<String> = "none";
    attribute mut_ptr : *mut Core::Buffer<u8>;
    attribute tuple_pair : (u32, Option<String>);
    attribute unit_type : ();
    attribute dyn_ref : &dyn Handler;
    attribute nullable_type : String?;

    part def SubsystemNode {
        part child_vec : Vec<ChildPart>;
        part ptr_array : *const [SensorNode; 10];
        port in frame_in : ~Channel<Result<T, E>>;
        port def DataPort {
            in flow payload : &[u8];
            out flow status : Result<T, E>;
            inout flow bidir : (u32, Option<String>);
            flow inout raw_bus : *mut [u8; 4];
        }
        action def ProcessBuffer(in buf : &[u8], out res : Result<Vec<u8>, E>, inout state_buf : (u32, &'a mut [u8])) : Result<(), E> {
            in ptr : *const [T; 10];
            out summary : Option<String>;
            inout ctx : &mut Context<T>;
        }
        constraint def CheckEnvelope(in samples : &[f64; 4], out bounds : (f64, f64), inout state_ref : &'a mut Result<T, E>) {
            samples != bounds;
        }
        calc def ComputeTrajectory(in vec_in : Vec<f64>, out mat_out : [f64; 16], inout state_io : (f64, f64)) {
            in tol : Option<f64>;
            out diag : Result<f64, E>;
            inout scratch : &[u8; 64];
            return res : Result<(f64, [f64; 3]), ErrorCode> = integrate(vec_in, tol);
        }
        operation def ExecuteOp(in slice_in : &[u8], out result_out : Result<T, E>) : Option<String>;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "CompositeTypesPkg")?;
        assert_eq!(pkg.name, "CompositeTypesPkg");
        assert_eq!(pkg.attribute_defs.len(), 10);
        assert_eq!(
            pkg.attribute_defs
                .first()
                .ok_or("missing attr 0")?
                .type_name,
            "Vec<T>"
        );
        assert_eq!(
            pkg.attribute_defs.get(1).ok_or("missing attr 1")?.type_name,
            "*const [T; 10]"
        );
        assert_eq!(
            pkg.attribute_defs.get(2).ok_or("missing attr 2")?.type_name,
            "&[u8]"
        );
        assert_eq!(
            pkg.attribute_defs.get(3).ok_or("missing attr 3")?.type_name,
            "Result<T, E>"
        );
        assert_eq!(
            pkg.attribute_defs.get(4).ok_or("missing attr 4")?.type_name,
            "Option<String>"
        );
        assert_eq!(
            pkg.attribute_defs.get(5).ok_or("missing attr 5")?.type_name,
            "*mut Core::Buffer<u8>"
        );
        assert_eq!(
            pkg.attribute_defs.get(6).ok_or("missing attr 6")?.type_name,
            "(u32, Option<String>)"
        );
        assert_eq!(
            pkg.attribute_defs.get(7).ok_or("missing attr 7")?.type_name,
            "()"
        );
        assert_eq!(
            pkg.attribute_defs.get(8).ok_or("missing attr 8")?.type_name,
            "&dyn Handler"
        );
        assert_eq!(
            pkg.attribute_defs.get(9).ok_or("missing attr 9")?.type_name,
            "String?"
        );

        let node = pkg.part_defs.first().ok_or("missing SubsystemNode")?;
        assert_eq!(node.parts.len(), 2);
        assert_eq!(
            node.parts
                .first()
                .ok_or("missing child_vec")?
                .type_name
                .as_deref(),
            Some("Vec<ChildPart>")
        );
        assert_eq!(
            node.parts
                .get(1)
                .ok_or("missing ptr_array")?
                .type_name
                .as_deref(),
            Some("*const [SensorNode; 10]")
        );

        assert_eq!(node.ports.len(), 2);
        let frame_in = node.ports.first().ok_or("missing frame_in")?;
        assert!(frame_in.is_conjugated);
        assert_eq!(frame_in.type_name, "Channel<Result<T, E>>");
        let data_port = node.ports.get(1).ok_or("missing DataPort")?;
        assert_eq!(data_port.item_flows.len(), 4);
        assert_eq!(
            data_port
                .item_flows
                .first()
                .ok_or("missing flow 0")?
                .item_type,
            "&[u8]"
        );
        assert_eq!(
            data_port
                .item_flows
                .get(1)
                .ok_or("missing flow 1")?
                .item_type,
            "Result<T, E>"
        );
        assert_eq!(
            data_port
                .item_flows
                .get(2)
                .ok_or("missing flow 2")?
                .direction,
            "inout"
        );
        assert_eq!(
            data_port
                .item_flows
                .get(2)
                .ok_or("missing flow 2")?
                .item_type,
            "(u32, Option<String>)"
        );
        assert_eq!(
            data_port
                .item_flows
                .get(3)
                .ok_or("missing flow 3")?
                .direction,
            "inout"
        );
        assert_eq!(
            data_port
                .item_flows
                .get(3)
                .ok_or("missing flow 3")?
                .item_type,
            "*mut [u8; 4]"
        );

        let act = node.actions.first().ok_or("missing ProcessBuffer")?;
        assert_eq!(act.in_params.len(), 2);
        assert_eq!(
            act.in_params.first().ok_or("missing in 0")?.type_name,
            "&[u8]"
        );
        assert_eq!(
            act.in_params.get(1).ok_or("missing in 1")?.type_name,
            "*const [T; 10]"
        );
        assert_eq!(act.out_params.len(), 4);
        assert_eq!(
            act.out_params.first().ok_or("missing out 0")?.type_name,
            "Result<Vec<u8>, E>"
        );
        assert_eq!(
            act.out_params.get(1).ok_or("missing out 1")?.type_name,
            "(u32, &'a mut [u8])"
        );
        assert_eq!(
            act.out_params.get(2).ok_or("missing out 2")?.type_name,
            "Option<String>"
        );
        assert_eq!(
            act.out_params.get(3).ok_or("missing out 3")?.type_name,
            "&mut Context<T>"
        );

        let con = node.constraints.first().ok_or("missing CheckEnvelope")?;
        assert_eq!(
            con.parameters,
            vec![
                "in samples: &[f64; 4]".to_string(),
                "out bounds: (f64, f64)".to_string(),
                "inout state_ref: &'a mut Result<T, E>".to_string(),
            ]
        );

        let calc = node.calcs.first().ok_or("missing ComputeTrajectory")?;
        assert_eq!(
            calc.return_type.as_deref(),
            Some("Result<(f64, [f64; 3]), ErrorCode>")
        );
        assert_eq!(calc.in_params.len(), 6);
        assert_eq!(
            calc.in_params
                .get(1)
                .ok_or("missing calc param 1")?
                .type_name,
            "[f64; 16]"
        );
        assert_eq!(
            calc.in_params
                .get(2)
                .ok_or("missing calc param 2")?
                .type_name,
            "(f64, f64)"
        );
        assert_eq!(
            calc.in_params
                .get(4)
                .ok_or("missing calc param 4")?
                .type_name,
            "Result<f64, E>"
        );
        assert_eq!(
            calc.in_params
                .get(5)
                .ok_or("missing calc param 5")?
                .type_name,
            "&[u8; 64]"
        );
        assert_eq!(
            calc.expression.as_deref(),
            Some("integrate ( vec_in , tol )")
        );

        let op = node.operations.first().ok_or("missing ExecuteOp")?;
        assert_eq!(op.return_type.as_deref(), Some("Option<String>"));
        assert_eq!(op.parameters.len(), 2);
        assert_eq!(
            op.parameters.first().ok_or("missing op param 0")?.type_name,
            "&[u8]"
        );
        assert_eq!(
            op.parameters.get(1).ok_or("missing op param 1")?.type_name,
            "Result<T, E>"
        );

        Ok(())
    }

    /// Verifies that `parse_container_item` fails closed with `Err(ParseError)` and valid source span coordinates on unrecognized container-scope keywords.
    #[test]
    fn test_defect_05_reject_unknown_container_keywords_fail_closed() -> Result<(), String> {
        let unknown_pkg_stmt = "package BadPkg {\n    atribute typo_attr : Integer;\n}\n";
        let err1 = SysmlParser::parse_source(unknown_pkg_stmt, "BadPkg")
            .err()
            .ok_or("unknown package-level keyword 'atribute' must return Err(ParseError)")?;
        assert_eq!(
            err1.span.line, 2,
            "expected error on line 2, got {:?}",
            err1
        );
        assert!(
            err1.span.col > 0,
            "expected positive column span, got {:?}",
            err1
        );

        let unknown_pkg_block = "package BadPkg {\n    enum def UnhandledEnum {\n        attribute a : Integer;\n    }\n}\n";
        let err2 = SysmlParser::parse_source(unknown_pkg_block, "BadPkg")
            .err()
            .ok_or("unknown braced block in package scope must return Err(ParseError)")?;
        assert_eq!(
            err2.span.line, 2,
            "expected error on line 2, got {:?}",
            err2
        );

        let unknown_part_stmt =
            "package BadPartPkg {\n    part def Node {\n        requirment typo_req;\n    }\n}\n";
        let err3 = SysmlParser::parse_source(unknown_part_stmt, "BadPartPkg")
            .err()
            .ok_or("unknown part-level keyword 'requirment' must return Err(ParseError)")?;
        assert_eq!(
            err3.span.line, 3,
            "expected error on line 3, got {:?}",
            err3
        );

        let unknown_inner_stmt =
            "package BadInnerPkg {\n    port def P {\n        atribute bad : Integer;\n    }\n}\n";
        let err4 = SysmlParser::parse_source(unknown_inner_stmt, "BadInnerPkg")
            .err()
            .ok_or("unknown inner statement in port def must return Err(ParseError)")?;
        assert_eq!(
            err4.span.line, 3,
            "expected error on line 3, got {:?}",
            err4
        );
        Ok(())
    }
}
