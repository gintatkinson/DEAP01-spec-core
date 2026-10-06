"""
Regression test suite for Tooling Defect: Lossy AST Serialization in sysmlv2_ast.py and compile_sysml.py.

Target Repository: gintatkinson/DEAP01-spec-core
Issue: #422 (https://github.com/gintatkinson/DEAP01-spec-core/issues/422)

Verifies:
1. Lossless round-trip parsing and serialization of typed part usages (is_def=False, type_name="AutopilotController"),
   asserting both in-memory AST attributes and re-serialized SysML text 'part fcc_board : AutopilotController;'.
2. Bidirectional port direction ('inout') preservation in to_sysml() round-trip serialization.
3. Clean 'action def' serialization without recursive self-invocation ('perform <ParentPart>;').
4. Guarded YAML imports across all parity auditor validator modules (uml.py, spec_validator.py,
   cardinality_validator.py, logical_ui_validator.py, reconcile_backlog.py).
5. Compiler pipeline compatibility (scripts/compile_sysml.py and sysmlv2_ingest.py).
6. Multi-generation round-trip idempotency (P -> S -> P -> S -> P -> S).
"""

import ast
import json
import os
import subprocess
import sys
import tempfile
import unittest

WORKSPACE_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SPEC_SCRIPTS_DIR = os.path.join(WORKSPACE_ROOT, "skills", "spec-orchestrator", "scripts")
if SPEC_SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SPEC_SCRIPTS_DIR)

PARITY_AUDITOR_SRC = os.path.join(WORKSPACE_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
if PARITY_AUDITOR_SRC not in sys.path:
    sys.path.insert(0, PARITY_AUDITOR_SRC)

from sysmlv2_ast import SysMLParser, PortDef, PartDef, ActionDef, SysMLPartUsage


class TestTypedPartUsageParity(unittest.TestCase):
    """Verifies lossless round-trip parsing and serialization of typed part usages."""

    def test_typed_part_usage_in_memory_ast(self):
        """Verify in-memory AST attributes of typed part usage (is_def=False, type_name set)."""
        sysml_snippet = """
        package TestPkg {
            part def AvionicsBay {
                part fcc_board : AutopilotController;
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_snippet)
        self.assertIsNotNone(pkg, "Failed to parse SysML package")
        self.assertEqual(len(pkg.part_defs), 1, "Expected 1 part def in package")

        parent_part = pkg.part_defs[0]
        self.assertEqual(parent_part.name, "AvionicsBay")
        self.assertTrue(parent_part.is_def)
        self.assertEqual(len(parent_part.parts), 1, "Expected 1 subpart inside AvionicsBay")

        subpart = parent_part.parts[0]
        self.assertEqual(subpart.name, "fcc_board")
        self.assertFalse(subpart.is_def, "Typed part usage must have is_def=False")
        self.assertEqual(
            subpart.type_name,
            "AutopilotController",
            "Typed part usage type_name must match declared type",
        )

    def test_typed_part_usage_serialization(self):
        """Verify that typed part usage serializes to 'part name : Type;' and not 'part def name'."""
        sysml_snippet = """
        package TestPkg {
            part def AvionicsBay {
                part fcc_board : AutopilotController;
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_snippet)
        self.assertIsNotNone(pkg)
        serialized = pkg.to_sysml()

        self.assertNotIn(
            "part def fcc_board",
            serialized,
            "Typed part usage was erroneously serialized as 'part def'",
        )
        self.assertIn(
            "part fcc_board : AutopilotController",
            serialized,
            "Serialized output missing typed part usage declaration",
        )

    def test_typed_part_usage_with_body_block(self):
        """Verify typed part usage with nested body block preserves attributes and ports."""
        sysml_snippet = """
        package TestPkg {
            part def AvionicsBay {
                part fcc_board : AutopilotController {
                    inout port rs485_esad : RS485Port;
                    attribute board_rev : Integer = 2;
                }
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_snippet)
        self.assertIsNotNone(pkg)
        parent_part = pkg.part_defs[0]
        subpart = parent_part.parts[0]

        self.assertEqual(subpart.name, "fcc_board")
        self.assertFalse(subpart.is_def)
        self.assertEqual(subpart.type_name, "AutopilotController")
        self.assertEqual(len(subpart.ports), 1)
        self.assertEqual(subpart.ports[0].name, "rs485_esad")
        self.assertEqual(subpart.ports[0].direction, "inout")
        self.assertEqual(len(subpart.attributes), 1)
        self.assertEqual(subpart.attributes[0].name, "board_rev")

        serialized = pkg.to_sysml()
        self.assertNotIn("part def fcc_board", serialized)
        self.assertIn("part fcc_board : AutopilotController {", serialized)
        self.assertIn("inout port rs485_esad : RS485Port;", serialized)
        self.assertIn("attribute board_rev : Integer = 2;", serialized)

        # Round-trip re-parse
        reparsed = SysMLParser.parse_text(serialized)
        reparsed_subpart = reparsed.part_defs[0].parts[0]
        self.assertEqual(reparsed_subpart.name, "fcc_board")
        self.assertFalse(reparsed_subpart.is_def)
        self.assertEqual(reparsed_subpart.type_name, "AutopilotController")
        self.assertEqual(reparsed_subpart.ports[0].direction, "inout")


class TestPortDirectionParity(unittest.TestCase):
    """Verifies bidirectional port direction ('inout') preservation in AST and serialization."""

    def test_inout_port_modifier_preserved(self):
        """Verify that PortDefinition.to_sysml() preserves 'inout', 'in', and 'out' modifiers."""
        sysml_snippet = """
        package TestPkg {
            part def Controller {
                inout port rs485_esad : RS485Port;
                in port gps_rx : GPSPort;
                out port telem_tx : TelemetryPort;
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_snippet)
        self.assertIsNotNone(pkg)
        ports = pkg.part_defs[0].ports
        self.assertEqual(len(ports), 3)

        port_map = {p.name: p for p in ports}
        self.assertEqual(port_map["rs485_esad"].direction, "inout")
        self.assertEqual(port_map["gps_rx"].direction, "in")
        self.assertEqual(port_map["telem_tx"].direction, "out")

        serialized = pkg.to_sysml()
        self.assertIn(
            "inout port rs485_esad : RS485Port;",
            serialized,
            "PortDefinition.to_sysml() stripped 'inout' modifier from port definition",
        )
        self.assertIn(
            "in port gps_rx : GPSPort;",
            serialized,
            "PortDefinition.to_sysml() stripped 'in' modifier from port definition",
        )
        self.assertIn(
            "out port telem_tx : TelemetryPort;",
            serialized,
            "PortDefinition.to_sysml() stripped 'out' modifier from port definition",
        )

        # Re-parse serialized output and confirm port directions are maintained
        reparsed = SysMLParser.parse_text(serialized)
        reparsed_map = {p.name: p for p in reparsed.part_defs[0].ports}
        self.assertEqual(reparsed_map["rs485_esad"].direction, "inout")
        self.assertEqual(reparsed_map["gps_rx"].direction, "in")
        self.assertEqual(reparsed_map["telem_tx"].direction, "out")


class TestActionDefParity(unittest.TestCase):
    """Verifies clean action def serialization without recursive self-invocation."""

    def test_action_def_no_circular_perform(self):
        """Verify action def inside part def does not inject circular parent performer calls."""
        sysml_snippet = """
        package TestPkg {
            part def ASTK_A5_V2_5_IAU {
                action def ExecuteFlightGuidance;
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_snippet)
        self.assertIsNotNone(pkg)
        action = pkg.part_defs[0].actions[0]
        self.assertTrue(action.is_def, "ActionDef must have is_def=True")
        self.assertEqual(action.performer, "", "action def must not assign parent as performer")

        serialized = pkg.to_sysml()
        self.assertIn(
            "action def ExecuteFlightGuidance;",
            serialized,
            "Serialized output missing clean 'action def ExecuteFlightGuidance;'",
        )
        self.assertNotIn(
            "perform ASTK_A5_V2_5_IAU",
            serialized,
            "Action definition injected circular self-performance of parent part def",
        )

    def test_action_def_with_body_no_circular_perform(self):
        """Verify action def with block body does not inject parent perform calls."""
        sysml_snippet = """
        package TestPkg {
            part def GuidanceComputer {
                action def NavigateWaypoints {
                    attribute max_waypoints : Integer = 50;
                }
            }
        }
        """
        pkg = SysMLParser.parse_text(sysml_snippet)
        self.assertIsNotNone(pkg)
        serialized = pkg.to_sysml()

        self.assertIn("action def NavigateWaypoints {", serialized)
        self.assertNotIn(
            "perform GuidanceComputer",
            serialized,
            "Action definition block injected circular parent perform call",
        )


class TestGuardedYamlImports(unittest.TestCase):
    """Verifies guarded YAML imports across all parity auditor validator modules."""

    VALIDATOR_MODULES = [
        "skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/uml.py",
        "skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/spec_validator.py",
        "skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/cardinality_validator.py",
        "skills/spec-orchestrator/parity_auditor/src/parity_auditor/validators/logical_ui_validator.py",
        "skills/spec-orchestrator/scripts/reconcile_backlog.py",
    ]

    def test_guarded_yaml_imports_across_all_validators(self):
        """Verify that every 'import yaml' statement is enclosed in a try-except ImportError block."""
        for rel_path in self.VALIDATOR_MODULES:
            abs_path = os.path.join(WORKSPACE_ROOT, rel_path)
            self.assertTrue(os.path.isfile(abs_path), f"File not found: {rel_path}")

            with open(abs_path, "r", encoding="utf-8") as f:
                tree = ast.parse(f.read(), filename=rel_path)

            yaml_imports = []
            for node in ast.walk(tree):
                if isinstance(node, ast.Import):
                    for alias in node.names:
                        if alias.name == "yaml":
                            yaml_imports.append(node)
                elif isinstance(node, ast.ImportFrom):
                    if node.module == "yaml":
                        yaml_imports.append(node)

            self.assertGreater(
                len(yaml_imports),
                0,
                f"Expected at least one yaml import in {rel_path}",
            )

            for imp_node in yaml_imports:
                enclosed_in_try = False
                catches_importerror = False

                for candidate in ast.walk(tree):
                    if isinstance(candidate, ast.Try):
                        for child in candidate.body:
                            if child == imp_node or any(imp_node == n for n in ast.walk(child)):
                                enclosed_in_try = True
                                for handler in candidate.handlers:
                                    if handler.type is None:
                                        catches_importerror = True
                                    elif isinstance(handler.type, ast.Name):
                                        if handler.type.id in ("ImportError", "Exception", "BaseException"):
                                            catches_importerror = True
                                break
                        if enclosed_in_try:
                            break

                self.assertTrue(
                    enclosed_in_try,
                    f"{rel_path}: 'import yaml' on line {imp_node.lineno} is not enclosed in a try block",
                )
                self.assertTrue(
                    catches_importerror,
                    f"{rel_path}: 'import yaml' try block does not catch ImportError",
                )

    def test_validator_frontmatter_fallback_without_pyyaml(self):
        """Verify frontmatter parsing helper works reliably even when yaml is None."""
        from parity_auditor.validators import uml, spec_validator, cardinality_validator

        sample_frontmatter = (
            "---\n"
            "id: 204\n"
            "title: Avionics Controller Subsystem\n"
            "features: [FEAT-01, FEAT-02]\n"
            "---\n"
            "# Spec Body Content\n"
        )

        orig_uml_yaml = uml.yaml
        orig_spec_yaml = spec_validator.yaml
        orig_card_yaml = cardinality_validator.yaml

        try:
            uml.yaml = None
            spec_validator.yaml = None
            cardinality_validator.yaml = None

            uml_data = uml._safe_load_yaml_frontmatter("id: 204\ntitle: Avionics Controller Subsystem\nfeatures: [FEAT-01, FEAT-02]\n")
            self.assertEqual(uml_data.get("id"), "204")
            self.assertEqual(uml_data.get("title"), "Avionics Controller Subsystem")
            self.assertEqual(uml_data.get("features"), ["FEAT-01", "FEAT-02"])

            spec_data = spec_validator._extract_frontmatter(sample_frontmatter)
            self.assertEqual(spec_data.get("id"), "204")
            self.assertEqual(spec_data.get("title"), "Avionics Controller Subsystem")
            self.assertEqual(spec_data.get("features"), ["FEAT-01", "FEAT-02"])

            card_data = cardinality_validator._extract_frontmatter(sample_frontmatter)
            self.assertEqual(card_data.get("id"), "204")
            self.assertEqual(card_data.get("title"), "Avionics Controller Subsystem")
        finally:
            uml.yaml = orig_uml_yaml
            spec_validator.yaml = orig_spec_yaml
            cardinality_validator.yaml = orig_card_yaml


class TestCompilerPipelineCompatibility(unittest.TestCase):
    """Verifies compiler pipeline compatibility across scripts/compile_sysml.py and sysmlv2_ingest.py."""

    def setUp(self):
        self.sample_sysml = """
        package TestPkg {
            part def AutopilotController {
                inout port rs485_esad : RS485Port;
                action def ExecuteFlightGuidance;
            }

            part def AvionicsBay {
                part fcc_board : AutopilotController;
            }
        }
        """

    def test_compile_sysml_script_compatibility(self):
        """Verify scripts/compile_sysml.py compiles model with typed parts, inout ports, and action defs."""
        compile_script = os.path.join(WORKSPACE_ROOT, "scripts", "compile_sysml.py")
        self.assertTrue(os.path.isfile(compile_script), "compile_sysml.py not found")

        with tempfile.NamedTemporaryFile("w", suffix=".sysml", delete=False) as f:
            f.write(self.sample_sysml)
            tmp_path = f.name

        try:
            result = subprocess.run(
                [sys.executable, compile_script, tmp_path],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(
                result.returncode,
                0,
                f"compile_sysml.py failed with stderr: {result.stderr}",
            )
            parsed_ast = json.loads(result.stdout)

            # Assert extracted constructs
            part_defs = parsed_ast.get("part_defs", [])
            self.assertIn("AutopilotController", part_defs)
            self.assertIn("AvionicsBay", part_defs)
            self.assertIn("fcc_board", part_defs)

            port_defs = parsed_ast.get("port_defs", [])
            self.assertIn("rs485_esad", port_defs)

            action_defs = parsed_ast.get("action_defs", [])
            self.assertIn("ExecuteFlightGuidance", action_defs)
        finally:
            if os.path.exists(tmp_path):
                os.remove(tmp_path)

    def test_sysmlv2_ingest_compatibility(self):
        """Verify sysmlv2_ingest.py parses and serializes without semantic loss or syntax degradation."""
        ingest_script = os.path.join(
            WORKSPACE_ROOT, "skills", "spec-orchestrator", "scripts", "sysmlv2_ingest.py"
        )
        self.assertTrue(os.path.isfile(ingest_script), "sysmlv2_ingest.py not found")

        with tempfile.NamedTemporaryFile("w", suffix=".sysml", delete=False) as f_in:
            f_in.write(self.sample_sysml)
            tmp_in = f_in.name

        with tempfile.NamedTemporaryFile("w", suffix=".sysml", delete=False) as f_out:
            tmp_out = f_out.name

        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f_digest:
            tmp_digest = f_digest.name

        try:
            result = subprocess.run(
                [
                    sys.executable,
                    ingest_script,
                    "--schema",
                    tmp_in,
                    "--format",
                    "sysml",
                    "--out",
                    tmp_out,
                    "--digest",
                    tmp_digest,
                ],
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(
                result.returncode,
                0,
                f"sysmlv2_ingest.py failed with stderr: {result.stderr}",
            )

            with open(tmp_out, "r", encoding="utf-8") as f:
                output_sysml = f.read()

            self.assertIn("part fcc_board : AutopilotController;", output_sysml)
            self.assertIn("inout port rs485_esad : RS485Port;", output_sysml)
            self.assertIn("action def ExecuteFlightGuidance;", output_sysml)
            self.assertNotIn("perform AutopilotController", output_sysml)
            self.assertNotIn("part def fcc_board", output_sysml)

            with open(tmp_digest, "r", encoding="utf-8") as f:
                digest_json = json.load(f)

            self.assertIn("sha256", digest_json)
            self.assertIn("node_counts", digest_json)
            self.assertGreater(digest_json["node_counts"].get("part_defs", 0), 0)
        finally:
            for p in (tmp_in, tmp_out, tmp_digest):
                if os.path.exists(p):
                    os.remove(p)


class TestMultiGenerationRoundTripIdempotency(unittest.TestCase):
    """Verifies multi-generation round-trip idempotency (P -> S -> P -> S -> P -> S)."""

    def test_multi_generation_round_trip_idempotency(self):
        """Verify text and AST stability across multiple serialize-parse generations."""
        source_snippet = """
        package TestPkg {
            part def AutopilotController {
                attribute cycle_time_ms : Real = 4.0;
                inout port rs485_esad : RS485Port;
                in port gps_rx : GPSPort;
                out port telem_tx : TelemetryPort;
                action def ExecuteFlightGuidance;
            }

            part def AvionicsBay {
                part fcc_board : AutopilotController {
                    inout port bus_port : RS485Port;
                }
                part payload_computer : MissionComputer;
            }
        }
        """

        # Generation 1: Parse T0 -> P1 -> Serialize S1
        p1 = SysMLParser.parse_text(source_snippet)
        self.assertIsNotNone(p1)
        s1 = p1.to_sysml()

        # Generation 2: Parse S1 -> P2 -> Serialize S2
        p2 = SysMLParser.parse_text(s1)
        self.assertIsNotNone(p2)
        s2 = p2.to_sysml()

        # Generation 3: Parse S2 -> P3 -> Serialize S3
        p3 = SysMLParser.parse_text(s2)
        self.assertIsNotNone(p3)
        s3 = p3.to_sysml()

        # Textual idempotency: S1 == S2 == S3
        self.assertEqual(
            s1,
            s2,
            "Serialized SysML drifted between Generation 1 and Generation 2",
        )
        self.assertEqual(
            s2,
            s3,
            "Serialized SysML drifted between Generation 2 and Generation 3",
        )

        # AST node counts idempotency
        self.assertEqual(
            p1.node_counts(),
            p2.node_counts(),
            "AST node counts drifted between Generation 1 and Generation 2",
        )
        self.assertEqual(
            p2.node_counts(),
            p3.node_counts(),
            "AST node counts drifted between Generation 2 and Generation 3",
        )

        # Detailed structural attribute assertions
        p1_parts = [(p.name, p.is_def, p.type_name) for p in p1.get_all_parts()]
        p2_parts = [(p.name, p.is_def, p.type_name) for p in p2.get_all_parts()]
        p3_parts = [(p.name, p.is_def, p.type_name) for p in p3.get_all_parts()]
        self.assertEqual(p1_parts, p2_parts)
        self.assertEqual(p2_parts, p3_parts)

        p1_ports = [(p.name, p.direction, p.type_name) for p in p1.get_all_ports()]
        p2_ports = [(p.name, p.direction, p.type_name) for p in p2.get_all_ports()]
        p3_ports = [(p.name, p.direction, p.type_name) for p in p3.get_all_ports()]
        self.assertEqual(p1_ports, p2_ports)
        self.assertEqual(p2_ports, p3_ports)

        p1_actions = [(a.name, a.is_def, a.performer) for a in p1.get_all_actions()]
        p2_actions = [(a.name, a.is_def, a.performer) for a in p2.get_all_actions()]
        p3_actions = [(a.name, a.is_def, a.performer) for a in p3.get_all_actions()]
        self.assertEqual(p1_actions, p2_actions)
        self.assertEqual(p2_actions, p3_actions)


if __name__ == "__main__":
    unittest.main()
