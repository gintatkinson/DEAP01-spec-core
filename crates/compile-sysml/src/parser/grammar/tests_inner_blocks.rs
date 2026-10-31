//! Use-case doc comment, fail-closed inner block, `calc` return classification, and bijective round-trip tests.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-TESTS]
//! Contract: Verifies inner/outer `use case def` documentation concatenation, fail-closed `Err(ParseError)` rejection across all 12 inner declaration bodies, consecutive inner doc comment attachment, `calc def` composite return variants, and bijective `ConnectionDef` / `RequirementDef` / `CalcDef` round-trips.
//! Bounds & Faults: Executes deterministic in-memory parsing and serialization round-trip assertions with zero panics.

#[cfg(test)]
mod inner_blocks_and_roundtrip_tests {
    use super::super::*;

    #[test]
    fn test_parse_use_case_def_preserves_inner_doc_comments(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package TestPackage {
    use case def UC_01_Vision {
        doc /*
        Actors: HumanEngineer, CIContinuousIntegrationRunner
        Subject: SystemVisionEngine
        Preconditions: Compiler runtime environment is initialized with valid configuration flags.
        Trigger: Compilation pipeline startup command dispatched by operator or CI runner.
        Main Success Scenario:
        1. Operator dispatches compilation command with workspace schema root.
        2. SystemVisionEngine initializes compiler namespace UUID constant NS_DEAP.
        3. InvariantBootstrapEngine asserts zero hardcoded domain concept rules across runtime context.
        4. NamespaceUuidGenerator pre-computes topological UUIDv5 namespace roots.
        5. DomainVocabularyValidator inspects grammar vocabulary tokens against prohibited domain ontologies.
        6. SystemVisionEngine emits immutable CompilerRulePort descriptor to downstream ingestion engines.
        Alternate and Exception Flows:
        5a. Prohibited domain term detected: SystemVisionEngine halts pipeline and emits diagnostic E0100.
        5b. Namespace UUID collision: SystemVisionEngine regenerates topological anchor with depth offset and emits E0101.
        5c. Malformed configuration flags: Compiler logs invalid argument diagnostics and falls back to safe defaults.
        5d. File permission error on workspace: Execution halts safely emitting file access diagnostic.
        5e. Invariant verification timeout: Watchdog timer triggers and logs diagnostic E0199 before graceful termination.
        5f. Non-deterministic execution detected: Checksum validation fails and emits reproducibility diagnostic E0102.
        5g. Memory allocation failure during bootstrap: Engine triggers emergency page compaction.
        5h. Missing root namespace configuration: Fallback to fixed RFC 4122 compiler constant.
        5i. Incompatible schema revision requested: Compiler warns and falls back to backwards-compatible parser mode.
        5j. Circular dependency detected in rule definitions: Rule topological sorter breaks cycle and reports E0103.
        Postconditions: Foundational compiler invariants and UUIDv5 namespaces are established deterministically.
        */
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "TestModel")?;
        assert_eq!(pkg.use_case_defs.len(), 1);
        let uc = pkg
            .use_case_defs
            .first()
            .ok_or("missing first element of pkg.use_case_defs")?;
        assert_eq!(uc.name, "UC_01_Vision");
        assert!(uc.doc.is_some());
        let doc = uc.doc.as_ref().ok_or("use case doc missing")?;
        assert!(doc.contains("Actors: HumanEngineer, CIContinuousIntegrationRunner"));
        assert!(doc.contains("Subject: SystemVisionEngine"));
        assert!(doc.contains("Preconditions: Compiler runtime environment is initialized"));
        assert!(doc.contains("Trigger: Compilation pipeline startup command"));
        assert!(doc.contains("Main Success Scenario:"));
        assert!(doc.contains("1. Operator dispatches compilation command"));
        assert!(doc.contains("Alternate and Exception Flows:"));
        assert!(doc.contains("5a. Prohibited domain term detected"));
        assert!(doc.contains("5j. Circular dependency detected"));
        assert!(doc.contains("Postconditions: Foundational compiler invariants"));
        Ok(())
    }

    #[test]
    fn test_parse_use_case_def_outer_and_inner_doc_concatenation(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
doc /* Package doc */
package TestPackage {
    doc /* Outer summary for UC_05 */
    use case def UC_05_Physical_Metrology {
        doc /*
        Actors: CIContinuousIntegrationRunner, HumanEngineer
        Subject: MetrologyFlowEngine
        Preconditions: AST stream is available.
        Trigger: Metrology flow verification command.
        Main Success Scenario:
        1. GrammarLoweringEngine passes AST stream.
        Alternate and Exception Flows:
        5a. SI unit dimension mismatch.
        Postconditions: Units verified.
        */
        subject MetrologyFlowEngine;
        actor CIContinuousIntegrationRunner;
        step GrammarLoweringEngine;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "TestModel")?;
        assert_eq!(pkg.use_case_defs.len(), 1);
        let uc = pkg
            .use_case_defs
            .first()
            .ok_or("missing first element of pkg.use_case_defs")?;
        assert_eq!(uc.name, "UC_05_Physical_Metrology");
        assert_eq!(uc.subject.as_deref(), Some("MetrologyFlowEngine"));
        assert_eq!(uc.actors.len(), 1);
        assert_eq!(
            uc.actors
                .first()
                .ok_or("missing first element of uc.actors")?,
            "CIContinuousIntegrationRunner"
        );
        assert_eq!(uc.steps.len(), 1);
        assert_eq!(
            uc.steps
                .first()
                .ok_or("missing first element of uc.steps")?,
            "GrammarLoweringEngine"
        );
        assert!(uc.doc.is_some());
        let doc = uc.doc.as_ref().ok_or("use case doc missing")?;
        assert!(doc.contains("Outer summary for UC_05"));
        assert!(doc.contains("Actors: CIContinuousIntegrationRunner, HumanEngineer"));
        assert!(doc.contains("Main Success Scenario:"));
        assert!(doc.contains("5a. SI unit dimension mismatch."));
        Ok(())
    }

    /// Verifies that all 12 inner braced-body declaration parsers fail closed with `Err(ParseError)` on unrecognized statements and preserve consecutive standalone inner `doc` comments.
    #[test]
    fn test_issue_504_inner_blocks_fail_closed_and_preserve_consecutive_docs(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bad_cases = [
            (
                "port",
                "package P {\n    port def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "action",
                "package P {\n    action def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "requirement",
                "package P {\n    requirement def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "state",
                "package P {\n    state def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "connection",
                "package P {\n    connection def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "use case",
                "package P {\n    use case def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "item",
                "package P {\n    item def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "hazard",
                "package P {\n    hazard def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "risk",
                "package P {\n    risk def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "test case",
                "package P {\n    test case def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "capability",
                "package P {\n    capability def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
            (
                "interaction",
                "package P {\n    interaction def X {\n        bogus_stmt x;\n    }\n}\n",
            ),
        ];

        for (label, src) in bad_cases {
            let err = SysmlParser::parse_source(src, "P").err().ok_or_else(|| {
                format!(
                    "expected Err(ParseError) for unknown inner statement in {}",
                    label
                )
            })?;
            assert_eq!(
                err.span.line, 3,
                "expected error on line 3 for {}, got {:?}",
                label, err
            );
            assert!(
                err.span.col > 0,
                "expected positive column for {}, got {:?}",
                label,
                err
            );
        }

        let valid_docs_src = r#"
package ConsecutiveInnerDocsPkg {
    port def P1 {
        /* port header doc 1 */
        doc /* port header doc 2 */
        /* flow doc */
        inout flow sig : Vec<u8>;
        /* trailing port comment */
    }
    action def A1 {
        /* action inner doc 1 */
        doc /* action inner doc 2 */
        perform Controller;
        step execute_step;
        /* trailing action comment */
    }
    requirement def R1 {
        /* req doc 1 */
        /* req doc 2 */
        id = "REQ-1";
        /* attr doc */
        attribute priority : Integer = 1;
        /* trailing req comment */
    }
    state def S1 {
        /* state doc 1 */
        doc /* state doc 2 */
        entry init;
        /* trailing state comment */
    }
    connection def C1 {
        /* conn doc 1 */
        doc /* conn doc 2 */
        connect a.p to b.q;
        /* trailing conn comment */
    }
    use case def UC1 {
        /* uc doc 1 */
        doc /* uc doc 2 */
        subject Sys;
        /* trailing uc comment */
    }
    item def I1 {
        /* item doc 1 */
        doc /* item doc 2 */
        /* item attr doc */
        attribute payload : &[u8];
        /* trailing item comment */
    }
    hazard def H1 {
        /* hazard doc 1 */
        doc /* hazard doc 2 */
        attribute severity : Integer = 3;
        /* trailing hazard comment */
    }
    risk def K1 {
        /* risk doc 1 */
        doc /* risk doc 2 */
        attribute severity : Integer = 2;
        /* trailing risk comment */
    }
    test case def T1 {
        /* test doc 1 */
        doc /* test doc 2 */
        subject Sys;
        step check_out;
        /* trailing test comment */
    }
    capability def Cap1 {
        /* cap doc 1 */
        doc /* cap doc 2 */
        subsystem Nav;
        /* trailing cap comment */
    }
    interaction def Int1 {
        /* inter doc 1 */
        doc /* inter doc 2 */
        lifeline A;
        message m1;
        trigger t1;
        /* trailing inter comment */
    }
}
"#;
        let pkg = SysmlParser::parse_source(valid_docs_src, "ConsecutiveInnerDocsPkg")?;
        let p1 = pkg.port_defs.first().ok_or("missing P1")?;
        assert_eq!(p1.doc.as_deref(), Some("port header doc 1"));
        assert_eq!(
            p1.item_flows.first().and_then(|f| f.doc.as_deref()),
            Some("flow doc")
        );
        let a1 = pkg.action_defs.first().ok_or("missing A1")?;
        assert_eq!(a1.doc.as_deref(), Some("action inner doc 1"));
        let r1 = pkg.requirement_defs.first().ok_or("missing R1")?;
        assert_eq!(r1.text, "req doc 1");
        assert_eq!(
            r1.attributes.first().and_then(|a| a.doc.as_deref()),
            Some("attr doc")
        );
        let i1 = pkg.item_defs.first().ok_or("missing I1")?;
        assert_eq!(i1.doc.as_deref(), Some("item doc 1"));
        assert_eq!(
            i1.attributes.first().and_then(|a| a.doc.as_deref()),
            Some("item attr doc")
        );
        let h1 = pkg.hazard_defs.first().ok_or("missing H1")?;
        assert_eq!(h1.doc.as_deref(), Some("hazard doc 1"));
        let k1 = pkg.risk_defs.first().ok_or("missing K1")?;
        assert_eq!(k1.doc.as_deref(), Some("risk doc 1"));
        let t1 = pkg.test_case_defs.first().ok_or("missing T1")?;
        assert_eq!(t1.doc.as_deref(), Some("test doc 1"));
        let cap1 = pkg.capability_defs.first().ok_or("missing Cap1")?;
        assert_eq!(cap1.doc.as_deref(), Some("cap doc 1"));
        assert_eq!(cap1.description.as_deref(), Some("cap doc 1"));
        let int1 = pkg.interaction_defs.first().ok_or("missing Int1")?;
        assert_eq!(int1.doc.as_deref(), Some("inter doc 1"));

        Ok(())
    }

    /// Verifies composite parameter and return type handling across `parse_constraint_decl` and `parse_calc_decl`.
    #[test]
    fn test_issue_504_composite_types_in_constraint_calc_action_and_port(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let src = r#"
package Issue504CompositePkg {
    constraint def MultiParamConstraint(
        in pair : (u32, Option<String>),
        out outcome : Result<Vec<u8>, ErrorCode>,
        inout window : &[f64; 4],
        bare_param
    ) {
        pair != outcome;
    }

    calc def CalcColonReturn {
        return : Result<(u32, u64), E> = compute(x, y);
    }

    calc def CalcTypeReturn {
        return type *const [u8; 16];
    }

    calc def CalcBareTypeReturn {
        return &'a mut [u8; 32];
    }

    calc def CalcCallExprReturn {
        return compute_norm(a, b);
    }

    calc def CalcAssignNoTypeReturn {
        return out_val = x + y;
    }
}
"#;
        let pkg = SysmlParser::parse_source(src, "Issue504CompositePkg")?;
        let con = pkg
            .constraint_defs
            .first()
            .ok_or("missing MultiParamConstraint")?;
        assert_eq!(
            con.parameters,
            vec![
                "in pair: (u32, Option<String>)".to_string(),
                "out outcome: Result<Vec<u8>, ErrorCode>".to_string(),
                "inout window: &[f64; 4]".to_string(),
                "bare_param".to_string(),
            ]
        );
        assert_eq!(con.expression, "pair != outcome");

        let c0 = pkg.calc_defs.first().ok_or("missing CalcColonReturn")?;
        assert_eq!(c0.return_type.as_deref(), Some("Result<(u32, u64), E>"));
        assert_eq!(c0.expression.as_deref(), Some("compute ( x , y )"));

        let c1 = pkg.calc_defs.get(1).ok_or("missing CalcTypeReturn")?;
        assert_eq!(c1.return_type.as_deref(), Some("*const [u8; 16]"));
        assert_eq!(c1.expression, None);

        let c2 = pkg.calc_defs.get(2).ok_or("missing CalcBareTypeReturn")?;
        assert_eq!(c2.return_type.as_deref(), Some("&'a mut [u8; 32]"));
        assert_eq!(c2.expression, None);

        let c3 = pkg.calc_defs.get(3).ok_or("missing CalcCallExprReturn")?;
        assert_eq!(c3.return_type, None);
        assert_eq!(c3.expression.as_deref(), Some("compute_norm ( a , b )"));

        let c4 = pkg
            .calc_defs
            .get(4)
            .ok_or("missing CalcAssignNoTypeReturn")?;
        assert_eq!(c4.return_type, None);
        assert_eq!(c4.expression.as_deref(), Some("x + y"));

        Ok(())
    }

    /// Verifies bijective AST round-trip parity (`parse_source(pkg.to_sysml(0)) == pkg`) across `ConnectionDef`, `RequirementDef::derives`, and `CalcDef` bare return expressions.
    #[test]
    fn test_defect_iter3_bijective_roundtrip_connection_requirement_calc_asymmetries(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::semantic::serializer::SysmlSerializable;
        let src = r#"
package Issue527BijectivePkg {
    connection def BusLink {
        connect nav_out to cmd_in;
        attribute source_part : String = "FlightComputer";
        attribute target_part : String = "ActuatorUnit";
        attribute item_flow_ref : String = "NavigationTelemetry";
        attribute severity : Integer = 4;
        attribute protocol : String = "CAN_FD";
        attribute latency_ms : Real = 2.5;
    }
    requirement def REQ_Parent {
        id = "REQ-527";
        text = "Parent requirement";
        derived from REQ_Root;
        derives REQ_ChildA;
        derives SubPkg::REQ_ChildB;
    }
    calc def PassThrough {
        in input_val : Real;
        return input_val;
    }
    calc def BareVarAndIndex {
        in samples : [Real; 4];
        return x;
        return samples[0];
    }
}
"#;
        let pkg = SysmlParser::parse_source(src, "Issue527BijectivePkg")?;
        let c0 = pkg.connection_defs.first().ok_or("missing c0")?;
        assert_eq!(c0.source_part.as_deref(), Some("FlightComputer"));
        assert_eq!(c0.target_part.as_deref(), Some("ActuatorUnit"));
        assert_eq!(c0.item_flow_ref.as_deref(), Some("NavigationTelemetry"));
        assert_eq!(c0.severity, 4);
        assert_eq!(
            pkg.requirement_defs.first().ok_or("missing r0")?.derives,
            vec!["REQ_ChildA", "SubPkg::REQ_ChildB"]
        );
        let calc0 = pkg.calc_defs.first().ok_or("missing calc0")?;
        assert_eq!(calc0.return_type, None);
        assert_eq!(calc0.expression.as_deref(), Some("input_val"));
        let calc1 = pkg.calc_defs.get(1).ok_or("missing calc1")?;
        assert_eq!(calc1.return_type, None);
        assert_eq!(calc1.expression.as_deref(), Some("x\nsamples [ 0 ]"));
        let serialized = pkg.to_sysml(0);
        let reparsed = SysmlParser::parse_source(&serialized, "Issue527BijectivePkg")?;
        assert_eq!(reparsed, pkg);
        Ok(())
    }
}
