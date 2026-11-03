//! Unit tests for `spec_audit::diagram_ast`.
//! Realises: [REQ-SPEC-AUDIT-DIAGRAM-AST]

use crate::spec_audit::diagram_ast::{
    parse_mermaid_diagram, ClassRelationType, DiagramAst, Visibility,
};
use crate::spec_audit::markdown_ast::MermaidBlock;

#[test]
fn test_class_diagram_parsing_with_visibility_and_types() -> Result<(), Box<dyn std::error::Error>>
{
    let raw = r#"classDiagram
    class FlightController {
        +float altitude
        -bool armed
        +execute_loop(int rate_hz) void
        #check_battery() bool
    }
    class SensorHub {
        +int sensor_count
    }
    FlightController *-- SensorHub : owns
"#;
    let block = MermaidBlock {
        raw: raw.to_string(),
        header: "classDiagram".to_string(),
        start_line: 10,
        end_line: 21,
    };

    let ast = parse_mermaid_diagram(&block)?;
    assert!(matches!(ast, DiagramAst::Class(_)), "Expected Class diagram AST");
    if let DiagramAst::Class(c_ast) = ast {
        assert_eq!(c_ast.classes.len(), 2);
        let fc = c_ast
            .classes
            .iter()
            .find(|c| c.name == "FlightController")
            .ok_or("missing FlightController")?;
        assert_eq!(fc.attributes.len(), 2);
        assert_eq!(fc.attributes.first().map(|a| a.name.as_str()), Some("altitude"));
        assert_eq!(fc.attributes.first().and_then(|a| a.visibility), Some(Visibility::Public));
        assert_eq!(fc.attributes.first().and_then(|a| a.type_name.as_deref()), Some("float"));

        assert_eq!(fc.operations.len(), 2);
        assert_eq!(fc.operations.first().map(|o| o.name.as_str()), Some("execute_loop"));
        assert_eq!(fc.operations.first().and_then(|o| o.visibility), Some(Visibility::Public));
        assert_eq!(fc.operations.first().and_then(|o| o.return_type.as_deref()), Some("void"));
        assert_eq!(
            fc.operations.first().map(|o| o.parameters.clone()).unwrap_or_default(),
            vec!["int rate_hz"]
        );

        assert_eq!(c_ast.relations.len(), 1);
        assert_eq!(c_ast.relations.first().map(|r| r.from_class.as_str()), Some("FlightController"));
        assert_eq!(c_ast.relations.first().map(|r| r.to_class.as_str()), Some("SensorHub"));
        assert_eq!(
            c_ast.relations.first().map(|r| r.relation_type.clone()),
            Some(ClassRelationType::Composition)
        );
        assert_eq!(c_ast.relations.first().and_then(|r| r.label.as_deref()), Some("owns"));
    }
    Ok(())
}

#[test]
fn test_class_diagram_curly_braces_in_member_fails() {
    let raw = r#"classDiagram
    class FlightController {
        +{abstract} void execute()
    }
"#;
    let block = MermaidBlock {
        raw: raw.to_string(),
        header: "classDiagram".to_string(),
        start_line: 1,
        end_line: 5,
    };

    let res = parse_mermaid_diagram(&block);
    assert!(res.is_err());
    assert!(res.err().is_some_and(|err| err.to_string().contains("prohibited curly braces")));
}

#[test]
fn test_class_diagram_secondary_colon_in_member_fails() {
    let raw = r#"classDiagram
    class FlightController {
        +altitude: float
    }
"#;
    let block = MermaidBlock {
        raw: raw.to_string(),
        header: "classDiagram".to_string(),
        start_line: 1,
        end_line: 5,
    };

    let res = parse_mermaid_diagram(&block);
    assert!(res.is_err());
    assert!(res.err().is_some_and(|err| err.to_string().contains("prohibited colon")));
}

#[test]
fn test_unquoted_angle_bracket_fails() {
    let raw = r#"stateDiagram-v2
    [*] --> Idle
    Idle --> Active : count > 0 / trigger
"#;
    let block = MermaidBlock {
        raw: raw.to_string(),
        header: "stateDiagram-v2".to_string(),
        start_line: 1,
        end_line: 4,
    };

    let res = parse_mermaid_diagram(&block);
    assert!(res.is_err());
    assert!(res.err().is_some_and(|err| err.to_string().contains("Unquoted '>' character")));
}

#[test]
fn test_quoted_angle_bracket_passes() {
    let raw = r#"stateDiagram-v2
    [*] --> Idle
    Idle --> Active : "count > 0 / trigger"
"#;
    let block = MermaidBlock {
        raw: raw.to_string(),
        header: "stateDiagram-v2".to_string(),
        start_line: 1,
        end_line: 4,
    };

    let res = parse_mermaid_diagram(&block);
    assert!(res.is_ok(), "Expected valid quoted comparison operator to pass: {:?}", res);
}

#[test]
fn test_node_label_unquoted_special_char_fails() {
    let raw = r#"flowchart TD
    A[Processing / Action] --> B
"#;
    let block = MermaidBlock {
        raw: raw.to_string(),
        header: "flowchart".to_string(),
        start_line: 1,
        end_line: 3,
    };

    let res = parse_mermaid_diagram(&block);
    assert!(res.is_err());
    assert!(res.err().is_some_and(|err| err.to_string().contains("special characters")));
}

#[test]
fn test_node_label_quoted_special_char_passes() {
    let raw = r#"flowchart TD
    A["Processing / Action"] --> B["Status: Active"]
"#;
    let block = MermaidBlock {
        raw: raw.to_string(),
        header: "flowchart".to_string(),
        start_line: 1,
        end_line: 3,
    };

    let res = parse_mermaid_diagram(&block);
    assert!(res.is_ok(), "Expected quoted node label with special chars to pass: {:?}", res);
}

#[test]
fn test_sequence_diagram_extraction() -> Result<(), Box<dyn std::error::Error>> {
    let raw = r#"sequenceDiagram
    actor Pilot
    participant FlightController
    participant SensorHub
    Pilot ->> FlightController: arm()
    FlightController ->> SensorHub: readSensors()
    SensorHub -->> FlightController: sensorData
    FlightController -->> Pilot: armedAck
"#;
    let block = MermaidBlock {
        raw: raw.to_string(),
        header: "sequenceDiagram".to_string(),
        start_line: 1,
        end_line: 9,
    };

    let ast = parse_mermaid_diagram(&block)?;
    assert!(matches!(ast, DiagramAst::Sequence(_)), "Expected Sequence diagram AST");
    if let DiagramAst::Sequence(s_ast) = ast {
        assert_eq!(s_ast.participants.len(), 3);
        let pilot = s_ast.participants.iter().find(|p| p.name == "Pilot").ok_or("missing Pilot")?;
        assert!(pilot.is_actor);

        assert_eq!(s_ast.messages.len(), 4);
        assert_eq!(s_ast.messages.first().map(|m| m.from.as_str()), Some("Pilot"));
        assert_eq!(s_ast.messages.first().map(|m| m.to.as_str()), Some("FlightController"));
        assert_eq!(
            s_ast.messages.first().and_then(|m| m.operation_name.as_deref()),
            Some("arm")
        );
        assert!(s_ast.messages.first().is_some_and(|m| !m.is_reply));

        assert_eq!(s_ast.messages.get(2).map(|m| m.from.as_str()), Some("SensorHub"));
        assert_eq!(s_ast.messages.get(2).map(|m| m.to.as_str()), Some("FlightController"));
        assert!(s_ast.messages.get(2).is_some_and(|m| m.is_reply));
    }
    Ok(())
}

#[test]
fn test_state_diagram_extraction_with_guard_and_action() -> Result<(), Box<dyn std::error::Error>> {
    let raw = r#"stateDiagram-v2
    [*] --> Idle
    Idle --> Active : "start [armed] / initMotors()"
    Active --> [*]
"#;
    let block = MermaidBlock {
        raw: raw.to_string(),
        header: "stateDiagram-v2".to_string(),
        start_line: 1,
        end_line: 5,
    };

    let ast = parse_mermaid_diagram(&block)?;
    assert!(matches!(ast, DiagramAst::State(_)), "Expected State diagram AST");
    if let DiagramAst::State(s_ast) = ast {
        assert_eq!(s_ast.states.len(), 2);
        assert_eq!(s_ast.transitions.len(), 3);

        let trans = s_ast.transitions.get(1).ok_or("missing transition 1")?;
        assert_eq!(trans.source, "Idle");
        assert_eq!(trans.target, "Active");
        assert_eq!(trans.event, Some("start".to_string()));
        assert_eq!(trans.guard, Some("armed".to_string()));
        assert_eq!(trans.action, Some("initMotors()".to_string()));
    }
    Ok(())
}

#[test]
fn test_inline_backtick_in_mermaid_node_label_rejected() -> Result<(), String> {
    let bad_raw = "graph TD\n    subgraph \"CompilerLintEnforcementGate System Boundary\"\n        UC_Main([\"Rustc E0453 Hard Rejection\"])\n        UC_Step01([\"Actor `Operator` submits input payload\"])\n        UC_Step02([\"`CompilerLintEnforcementGate` validates input schema\"])\n    end\n";
    let bad_block = MermaidBlock {
        raw: bad_raw.to_string(),
        header: "graph TD".to_string(),
        start_line: 1,
        end_line: 6,
    };
    if parse_mermaid_diagram(&bad_block).is_ok() {
        return Err("Expected Mermaid diagram with inline backticks to be rejected".to_string());
    }

    let good_raw = "graph TD\n    subgraph \"CompilerLintEnforcementGate System Boundary\"\n        UC_Main([\"Rustc E0453 Hard Rejection\"])\n        UC_Step01([\"Actor Operator submits input payload\"])\n        UC_Step02([\"CompilerLintEnforcementGate validates input schema\"])\n    end\n";
    let good_block = MermaidBlock {
        raw: good_raw.to_string(),
        header: "graph TD".to_string(),
        start_line: 1,
        end_line: 6,
    };
    if let Err(e) = parse_mermaid_diagram(&good_block) {
        return Err(format!("Expected sanitized Mermaid diagram to pass, got {:?}", e));
    }
    Ok(())
}
