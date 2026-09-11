"""
Unit tests for SemanticDiagramASTValidator and Check 21 (Issue #252).

Verifies:
1. Detection of undeclared phantom nodes ('semantic-diagram-undeclared-node').
2. Detection of inverted telemetry / signal flows violating SysML connection topology ('semantic-diagram-inverted-flow').
3. Detection of ungrounded actuators with zero incoming command/power connections ('semantic-diagram-ungrounded-component').
4. Detection of invalid physical load/command paths ('semantic-diagram-invalid-load-path').
5. Conforming diagrams matching SysML AST ground truth pass with 0 errors.
6. Integration with verify_downstream_baseline.py Check 21.
7. Registration in AGGREGATING_VALIDATORS and parity_auditor CLI pipeline.
"""

import os
import sys
import tempfile
import unittest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

parity_src = os.path.join(repo_root, "skills", "spec-orchestrator", "parity_auditor", "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.semantic_diagram_ast_validator import SemanticDiagramASTValidator
from parity_auditor.aggregator import AGGREGATING_VALIDATORS
from scripts.verify_downstream_baseline import check_semantic_diagram_ast_parity, run_all_checks

# Load SysML AST classes
from parity_auditor.utils.sysml_loader import load_sysml_ast_members

_sysml_ast = load_sysml_ast_members([
    "SysMLPackage", "SysMLParser", "PartDef", "PortDef", "ItemDef", "ConnectionDef"
])
SysMLPackage = _sysml_ast.SysMLPackage
SysMLParser = _sysml_ast.SysMLParser


SAMPLE_SYSML_MODEL = """package UAS_Avionics_SSOT {
    doc /* Primary SysML Model for Test Suite */

    port def TelemetryPort {
        out item Velocity : Float32;
        out item Altitude : Float32;
    }

    port def NavTelemetryPort {
        out item PositionLat : Float64;
        out item PositionLon : Float64;
    }

    port def FlightControlInPort {
        in item PositionLat : Float64;
        in item PositionLon : Float64;
        in item Velocity : Float32;
    }

    port def ActuatorCommandPort {
        out item MotorRPMCmd : Float32;
        out item RudderAngleCmd : Float32;
    }

    port def ActuatorInputPort {
        in item MotorRPMCmd : Float32;
        in item RudderAngleCmd : Float32;
    }

    part def NavigationSubsystem {
        out port nav_out : NavTelemetryPort;
    }

    part def FlightControlSubsystem {
        in port nav_in : FlightControlInPort;
        out port act_cmd_out : ActuatorCommandPort;
    }

    part def MotorActuator {
        in port act_in : ActuatorInputPort;
    }

    part def RudderServoActuator {
        in port act_in : ActuatorInputPort;
    }

    connection Conn_Nav_To_FCC {
        connect NavigationSubsystem.nav_out to FlightControlSubsystem.nav_in;
    }

    connection Conn_FCC_To_Motor {
        connect FlightControlSubsystem.act_cmd_out to MotorActuator.act_in;
    }

    connection Conn_FCC_To_Rudder {
        connect FlightControlSubsystem.act_cmd_out to RudderServoActuator.act_in;
    }
}
"""


class TestSemanticDiagramASTValidator(unittest.TestCase):
    def setUp(self):
        self.pkg = SysMLParser.parse_text(SAMPLE_SYSML_MODEL)
        self.validator = SemanticDiagramASTValidator()

    def test_registered_in_aggregating_validators(self):
        """Verify SemanticDiagramASTValidator is registered in AGGREGATING_VALIDATORS."""
        self.assertIn(SemanticDiagramASTValidator, AGGREGATING_VALIDATORS)

    def test_conforming_diagram_passes_zero_errors(self):
        """Verify a conforming Mermaid flowchart passes with 0 findings."""
        diagram = """
flowchart TD
    subgraph NavigationSubsystem ["Navigation Subsystem"]
        NavNode["nav_out"]
    end
    subgraph FlightControlSubsystem ["Flight Control Subsystem"]
        FCCNode["nav_in"]
        CmdNode["act_cmd_out"]
    end
    subgraph MotorActuator ["Motor Actuator"]
        MotorNode["act_in"]
    end

    NavNode --> FCCNode
    CmdNode --> MotorNode
"""
        findings = self.validator.validate_diagram_ast(diagram, "docs/architecture/flow.md", self.pkg)
        self.assertEqual(len(findings), 0, f"Expected 0 findings, got: {findings}")

    def test_detection_of_undeclared_phantom_node(self):
        """Verify undeclared phantom nodes are flagged with semantic-diagram-undeclared-node."""
        diagram = """
flowchart TD
    NavigationSubsystem --> FlightControlSubsystem
    FlightControlSubsystem --> MotorActuator
    PhantomQuantumRadar["Phantom Quantum Radar Device"] --> FlightControlSubsystem
"""
        findings = self.validator.validate_diagram_ast(diagram, "docs/architecture/flow.md", self.pkg)
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("semantic-diagram-undeclared-node", rule_ids)
        self.assertTrue(any("PhantomQuantumRadar" in str(f) or "Phantom Quantum Radar" in str(f) for f in findings))

    def test_detection_of_inverted_telemetry_flow(self):
        """Verify inverted telemetry/signal flow is flagged with semantic-diagram-inverted-flow."""
        # SysML defines NavigationSubsystem -> FlightControlSubsystem.
        # Flowchart below reverses this direction: FlightControlSubsystem --> NavigationSubsystem.
        diagram = """
flowchart TD
    subgraph NavigationSubsystem ["Navigation Subsystem"]
        NavNode["nav_out"]
    end
    subgraph FlightControlSubsystem ["Flight Control Subsystem"]
        FCCNode["nav_in"]
    end

    FlightControlSubsystem -->|"telemetry_data"| NavigationSubsystem
"""
        findings = self.validator.validate_diagram_ast(diagram, "docs/architecture/flow.md", self.pkg)
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("semantic-diagram-inverted-flow", rule_ids)

    def test_detection_of_ungrounded_actuator(self):
        """Verify ungrounded actuator (0 incoming command/power connections) is flagged."""
        diagram = """
flowchart TD
    subgraph FlightControlSubsystem ["Flight Control Subsystem"]
        FCCNode["Flight Controller"]
    end
    subgraph MotorActuator ["Motor Actuator"]
        MotorNode["Motor ESC Unit"]
    end

    MotorNode -->|"status"| FCCNode
"""
        findings = self.validator.validate_diagram_ast(diagram, "docs/architecture/flow.md", self.pkg)
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("semantic-diagram-ungrounded-component", rule_ids)

    def test_detection_of_invalid_load_path(self):
        """Verify actuator directing driving commands upstream is flagged with semantic-diagram-invalid-load-path."""
        diagram = """
flowchart TD
    subgraph FlightControlSubsystem ["Flight Control Subsystem"]
        FCCNode["nav_in"]
    end
    subgraph MotorActuator ["Motor Actuator"]
        MotorNode["act_in"]
    end

    FCCNode --> MotorNode
    MotorNode -->|"command"| FCCNode
"""
        findings = self.validator.validate_diagram_ast(diagram, "docs/architecture/flow.md", self.pkg)
        rule_ids = [f.rule_id for f in findings]
        self.assertIn("semantic-diagram-invalid-load-path", rule_ids)

    def test_class_diagram_validation(self):
        """Verify classDiagram elements are checked against SysML AST."""
        conforming_cd = """
classDiagram
    class NavigationSubsystem {
        +NavTelemetryPort nav_out
    }
    class FlightControlSubsystem {
        +FlightControlInPort nav_in
    }
    NavigationSubsystem --> FlightControlSubsystem : Telemetry
"""
        findings = self.validator.validate_diagram_ast(conforming_cd, "docs/features/feat-01.md", self.pkg)
        self.assertEqual(len(findings), 0, f"Expected 0 findings for conforming classDiagram, got: {findings}")

        invalid_cd = """
classDiagram
    class GhostSubsystemComponent {
        +String ghostAttr
    }
"""
        invalid_findings = self.validator.validate_diagram_ast(invalid_cd, "docs/features/feat-01.md", self.pkg)
        rule_ids = [f.rule_id for f in invalid_findings]
        self.assertIn("semantic-diagram-undeclared-node", rule_ids)

    def test_workspace_repository_validation_clean_pass(self):
        """Verify validate(repo) runs cleanly over a mock workspace."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_MODEL)

            with open(os.path.join(docs_dir, "feat-01.md"), "w", encoding="utf-8") as f:
                f.write("""# Feature 01
```mermaid
flowchart TD
    NavigationSubsystem --> FlightControlSubsystem
    FlightControlSubsystem --> MotorActuator
```
""")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(len(findings), 0, f"Expected 0 findings, got: {findings}")

    def test_check21_baseline_gate(self):
        """Verify check_semantic_diagram_ast_parity function behaves properly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # Landing zone clean -> passes
            check_semantic_diagram_ast_parity(tmpdir)

            # With valid model and conforming diagram -> passes
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "architecture")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_MODEL)

            with open(os.path.join(docs_dir, "ARCH.md"), "w", encoding="utf-8") as f:
                f.write("""# Architecture
```mermaid
flowchart TD
    NavigationSubsystem --> FlightControlSubsystem
    FlightControlSubsystem --> MotorActuator
```
""")

            check_semantic_diagram_ast_parity(tmpdir)


if __name__ == "__main__":
    unittest.main()
