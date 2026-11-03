//! Requirement, import, inline connector, multi-root package, test case, and port round-trip grammar tests.
//!
//! Safety & Traceability: Realises: [REQ-PARSER-GRAMMAR-TESTS]
//! Contract: Verifies requirement member clauses (`attribute`, `assume`, `require`, `derived from`, encapsulated `constraint`), `import` wildcards, inline connectors, multi-root package forests, `test case def` assertion links, and `PortDef` protocol bijective round-trips.
//! Bounds & Faults: Executes deterministic in-memory parsing and serialization assertions with zero panics.

#[cfg(test)]
mod decl_and_port_tests {
    use super::super::*;

    #[test]
    fn test_parse_requirement_with_attributes() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package ReqPkg {
    requirement def REQ_0001_Demo {
        id = "REQ-0001";
        text = "The system shall process data.";
        attribute uuidv5 : String = "5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9";
        attribute complexity_class : String = "Class P";
        attribute ac_01_demo : String = "Given: input. When: run. Then: success.";
        verify by AC_01_Demo;
        satisfy by DemoEngine;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = pkg.requirement_defs.first().ok_or("missing req")?;
        assert_eq!(req.name, "REQ_0001_Demo");
        assert_eq!(req.req_id, "REQ-0001");
        assert_eq!(req.text, "The system shall process data.");
        assert_eq!(req.attributes.len(), 3);
        let a0 = req.attributes.first().ok_or("missing a0")?;
        assert_eq!(a0.name, "uuidv5");
        assert_eq!(a0.default_value.as_deref(), Some("\"5a4d7a99-0419-5c2b-ba0e-ccabdb05f1c9\""));
        let a1 = req.attributes.get(1).ok_or("missing a1")?;
        assert_eq!(a1.name, "complexity_class");
        assert_eq!(a1.default_value.as_deref(), Some("\"Class P\""));
        let a2 = req.attributes.get(2).ok_or("missing a2")?;
        assert_eq!(a2.name, "ac_01_demo");
        assert_eq!(a2.default_value.as_deref(), Some("\"Given: input. When: run. Then: success.\""));
        assert_eq!(req.verified_by, vec!["AC_01_Demo"]);
        assert_eq!(req.satisfied_by, vec!["DemoEngine"]);
        Ok(())
    }

    #[test]
    fn test_parse_requirement_with_assumptions_constraints_derivations(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package ReqPkg {
    requirement def REQ_0032_Test {
        id = "REQ-0032";
        text = "Req with constraints and derivations.";
        assume ValidSchema;
        require Invariant_Alpha;
        require Invariant_Beta;
        derived from REQ_0031_Parent;
        derived from REQ_0019_Sanitization;
        verify by AC_01;
        satisfy by Engine;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = pkg.requirement_defs.first().ok_or("missing req")?;
        assert_eq!(req.assumes, vec!["ValidSchema"]);
        assert_eq!(req.requires, vec!["Invariant_Alpha", "Invariant_Beta"]);
        assert_eq!(req.derived_from, vec!["REQ_0031_Parent", "REQ_0019_Sanitization"]);
        Ok(())
    }

    #[test]
    fn test_parse_requirement_with_encapsulated_constraints(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package ReqPkg {
    requirement def REQ_0001_Test {
        id = "REQ-0001";
        text = "Requirement with inner constraints.";
        doc /* Formula Alpha */
        constraint def Invariant_Alpha;
        assert constraint Invariant_Beta {
            x > 0;
        }
        require Invariant_Alpha;
        require Invariant_Beta;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.requirement_defs.len(), 1);
        let req = pkg.requirement_defs.first().ok_or("missing req")?;
        assert_eq!(req.constraints.len(), 2);
        let c0 = req.constraints.first().ok_or("missing c0")?;
        assert_eq!(c0.name, "Invariant_Alpha");
        assert_eq!(c0.doc.as_deref(), Some("Formula Alpha"));
        assert!(!c0.is_assertion);
        let c1 = req.constraints.get(1).ok_or("missing c1")?;
        assert_eq!(c1.name, "Invariant_Beta");
        assert!(c1.is_assertion);
        assert_eq!(c1.expression, "x > 0");
        Ok(())
    }

    #[test]
    fn test_parse_import_declarations() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package Subsystem_Imports {
    import Subsystem_1::*;
    import Subsystem_2::Engine;
    import Subsystem_3::Core::**;
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.imports.len(), 3);
        let i0 = pkg.imports.first().ok_or("missing i0")?;
        assert_eq!(i0.path, "Subsystem_1");
        assert!(i0.is_wildcard && !i0.is_recursive);

        let i1 = pkg.imports.get(1).ok_or("missing i1")?;
        assert_eq!(i1.path, "Subsystem_2::Engine");
        assert!(!i1.is_wildcard && !i1.is_recursive);

        let i2 = pkg.imports.get(2).ok_or("missing i2")?;
        assert_eq!(i2.path, "Subsystem_3::Core");
        assert!(i2.is_wildcard && i2.is_recursive);
        Ok(())
    }

    #[test]
    fn test_parse_inline_connector_in_part_def() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
part def DEAPCompilerSystem {
    connect a.out to b.in;
}
"#;
        let pkg = SysmlParser::parse_source(source, "Default")?;
        assert_eq!(pkg.part_defs.len(), 1);
        let part = pkg.part_defs.first().ok_or("missing part")?;
        assert_eq!(part.name, "DEAPCompilerSystem");
        assert_eq!(part.connections.len(), 1);
        let conn = part.connections.first().ok_or("missing conn")?;
        assert_eq!(conn.source_port, "a.out");
        assert_eq!(conn.target_port, "b.in");
        assert_eq!(conn.source_part.as_deref(), Some("a"));
        assert_eq!(conn.target_part.as_deref(), Some("b"));
        assert_eq!(conn.name, "c_a_out_to_b_in");
        Ok(())
    }

    #[test]
    fn test_parse_multiple_root_packages() -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package Subsystem_1 {
    part def Engine;
}
package Subsystem_2 {
    part def Controller;
}
"#;
        let pkg = SysmlParser::parse_source(source, "RootSystem")?;
        assert_eq!(pkg.name, "RootSystem");
        assert_eq!(pkg.packages.len(), 2);
        let sub1 = pkg.packages.first().ok_or("missing sub1")?;
        assert_eq!(sub1.name, "Subsystem_1");
        assert_eq!(sub1.part_defs.len(), 1);
        assert_eq!(sub1.part_defs.first().ok_or("missing Engine")?.name, "Engine");
        let sub2 = pkg.packages.get(1).ok_or("missing sub2")?;
        assert_eq!(sub2.name, "Subsystem_2");
        assert_eq!(sub2.part_defs.len(), 1);
        assert_eq!(sub2.part_defs.first().ok_or("missing Controller")?.name, "Controller");
        Ok(())
    }

    #[test]
    fn test_parse_test_case_def_with_assert_constraint_ids(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let source = r#"
package AutonomousSystem {
    assert constraint Assert_UCA_33 {
        interlockState == InterlockStatus::ENGAGED;
    }

    assert constraint Assert_UCA_34 {
        lockConfidence >= 0.98;
    }

    test case def TC_UCA_33_Verification {
        subject SafetyController;
        verify requirement Assert_UCA_33, Assert_UCA_34;
        objective "Verify safety interlock per UCA-33";
        step trigger_interlock;
    }
}
"#;
        let pkg = SysmlParser::parse_source(source, "TestModel")?;
        assert_eq!(pkg.test_case_defs.len(), 1);
        let tc = pkg.test_case_defs.first().ok_or("missing tc")?;
        assert_eq!(tc.name, "TC_UCA_33_Verification");
        assert_eq!(tc.verified_requirements, vec!["Assert_UCA_33", "Assert_UCA_34"]);
        Ok(())
    }

    /// Verifies that `PortDef` `protocol_family`, `port_category`, and `electrical_attributes` strip outer string quotes on parse and round-trip idempotently through `to_sysml`.
    #[test]
    fn test_port_def_protocol_family_bijective_roundtrip() -> Result<(), Box<dyn std::error::Error>>
    {
        use crate::semantic::serializer::SysmlSerializable;
        let src = r#"package PortRoundtripPkg {
    part def Controller {
        port inout BusPort : String {
            attribute protocol_family : String = "FilesystemUTF8";
            attribute port_category : String = "ControlPort";
        }
    }
}
"#;
        let pkg1 = SysmlParser::parse_source(src, "PortRoundtripPkg")?;
        let part1 = pkg1.part_defs.first().ok_or("missing Controller")?;
        let port1 = part1.ports.first().ok_or("missing BusPort")?;
        assert_eq!(port1.protocol_family, "FilesystemUTF8");
        assert_eq!(port1.port_category, "ControlPort");

        let serialized = pkg1.to_sysml(0);
        let pkg2 = SysmlParser::parse_source(&serialized, "PortRoundtripPkg")?;
        assert_eq!(pkg1, pkg2);
        Ok(())
    }
}
