#!/usr/bin/env python3
"""
Unit Tests for SysML v2 Markdown / BOM Ingestion & Pipeline 0 Gate Fallback.

Tests:
1. MarkdownTranslator (skills/spec-orchestrator/scripts/translators/markdown_translator.py):
   - BOM / component table ingestion into PartDefs with typed AttributeDefs.
   - Interface / signal table ingestion into PortDefs with ItemFlowDefs.
   - Parametric limit table ingestion into SysMLConstraintDef assertion blocks.
   - Multi-section Markdown document parsing with package and component hierarchy.
   - Round-trip SysML v2 parsing: pkg.to_sysml() parsed by SysMLParser.parse_text().
   - Multi-file markdown translation and AST merging.
2. sysmlv2_ingest.py:
   - Format auto-detection (.md extension and table headers).
   - CLI format parameter handling (--format markdown).
   - Schema directory auto-discovery (schema/ and schema/extracted/).
   - End-to-end CLI execution producing .sysml model and digest compileable by compile_sysml.py.
3. scripts/compile_sysml.py:
   - enforce_pipeline0_compilation_gate() failure when no .sysml exists in schema/,
     verifying helpful remediation message to stderr and exit code 1.
   - enforce_pipeline0_compilation_gate() success with valid schema.

Pure Schema-Driven Invariant:
Zero hardcoded domain concepts. All test fixtures use generic abstract system architecture models.
"""

import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from typing import Dict, Any

# Ensure project root and spec-orchestrator scripts are in sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")

if SPEC_SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SPEC_SCRIPTS_DIR)
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from translators.markdown_translator import MarkdownTranslator, sanitize_identifier, parse_numeric_with_unit
from sysmlv2_ast import SysMLParser, SysMLPackage, PartDef, PortDef, AttributeDef, SysMLConstraintDef
from sysmlv2_ingest import detect_format, discover_schema_targets, ingest_schema
import scripts.compile_sysml as compile_sysml


class TestMarkdownTranslator(unittest.TestCase):
    """Unit tests for MarkdownTranslator AST generation and round-trip serialization."""

    def setUp(self):
        self.translator = MarkdownTranslator()

    def test_bom_component_table_ingestion(self):
        """Test BOM / component table ingestion into PartDefs with typed AttributeDefs."""
        md_content = """# CoreSystemModel

## Bill of Materials
| Component | Part Number | Mass (kg) | Power (W) | Redundant | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| ComputeEngine | CE-100-A | 2.5 | 45.0 | true | Primary compute controller unit |
| PowerModule | PM-200-B | 1.8 | 120.5 | false | Main power regulation module |
| SensorHub | SH-300-C | 0.75 | 12 | true | Multi-channel sensor interface hub |
"""
        pkg = self.translator.translate(md_content)
        self.assertEqual(pkg.name, "CoreSystemModel")
        self.assertEqual(len(pkg.part_defs), 3)

        part_map = {p.name: p for p in pkg.part_defs}
        self.assertIn("ComputeEngine", part_map)
        self.assertIn("PowerModule", part_map)
        self.assertIn("SensorHub", part_map)

        # Verify ComputeEngine attributes
        ce = part_map["ComputeEngine"]
        self.assertIn("Primary compute controller unit", ce.doc)
        ce_attrs = {a.name: a for a in ce.attributes}
        self.assertIn("part_number", ce_attrs)
        self.assertEqual(ce_attrs["part_number"].type_name, "String")
        self.assertEqual(ce_attrs["part_number"].default_value, '"CE-100-A"')

        self.assertIn("mass", ce_attrs)
        self.assertEqual(ce_attrs["mass"].type_name, "Real")
        self.assertEqual(ce_attrs["mass"].default_value, "2.5")
        self.assertEqual(ce_attrs["mass"].doc, "unit: kg")

        self.assertIn("power", ce_attrs)
        self.assertEqual(ce_attrs["power"].type_name, "Real")
        self.assertEqual(ce_attrs["power"].default_value, "45.0")
        self.assertEqual(ce_attrs["power"].doc, "unit: W")

        self.assertIn("redundant", ce_attrs)
        self.assertEqual(ce_attrs["redundant"].type_name, "Boolean")
        self.assertEqual(ce_attrs["redundant"].default_value, "true")

        # Verify SensorHub integer power parsing
        sh = part_map["SensorHub"]
        sh_attrs = {a.name: a for a in sh.attributes}
        self.assertIn("power", sh_attrs)
        self.assertEqual(sh_attrs["power"].type_name, "Integer")
        self.assertEqual(sh_attrs["power"].default_value, "12")

    def test_interface_signal_table_ingestion(self):
        """Test interface / signal table ingestion into PortDefs with ItemFlowDefs."""
        md_content = """# NetworkInterfaceModel

## System Interfaces
| Port | Direction | Type | Rate (Hz) | Unit | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| TelemetryOut | out | TelemetryPacket | 50.0 | Hz | Ethernet | Downlink telemetry stream |
| CommandIn | in | CommandPacket | 20.0 | Hz | CAN | Uplink command payload |
| ServiceBus | inout | BusMessage | 100.0 | msg/s | MilStd1553 | Bidirectional subsystem bus |
"""
        pkg = self.translator.translate(md_content)
        self.assertEqual(pkg.name, "NetworkInterfaceModel")

        # At package level or synthesized system component
        ports = pkg.port_defs if pkg.port_defs else pkg.part_defs[0].ports
        self.assertEqual(len(ports), 3)

        port_map = {p.name: p for p in ports}
        self.assertIn("TelemetryOut", port_map)
        self.assertIn("CommandIn", port_map)
        self.assertIn("ServiceBus", port_map)

        # Check TelemetryOut details
        p_out = port_map["TelemetryOut"]
        self.assertEqual(p_out.direction, "out")
        self.assertEqual(p_out.type_name, "TelemetryPacket")
        self.assertEqual(p_out.protocol_family, "Ethernet")
        self.assertEqual(p_out.doc, "Downlink telemetry stream")
        self.assertTrue(len(p_out.item_flows) > 0)
        flow_out = p_out.item_flows[0]
        self.assertEqual(flow_out.name, "TelemetryOut_flow")
        self.assertEqual(flow_out.direction, "out")
        self.assertEqual(flow_out.item_type, "TelemetryPacket")
        self.assertEqual(flow_out.rate_hz, 50.0)
        self.assertEqual(flow_out.unit, "Hz")

        # Check CommandIn details
        p_in = port_map["CommandIn"]
        self.assertEqual(p_in.direction, "in")
        self.assertEqual(p_in.type_name, "CommandPacket")
        self.assertEqual(p_in.protocol_family, "CAN")
        self.assertTrue(len(p_in.item_flows) > 0)
        flow_in = p_in.item_flows[0]
        self.assertEqual(flow_in.direction, "in")
        self.assertEqual(flow_in.rate_hz, 20.0)

    def test_parametric_limit_table_ingestion(self):
        """Test parametric limit table ingestion into SysMLConstraintDef assertion blocks."""
        md_content = """# PowerSubsystemModel

## Parametric Limits
| Parameter | Min | Max | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| BusVoltage | 18.0 | 36.0 | V | DC bus nominal voltage bounds |
| OperatingTemp | -40.0 | 85.0 | degC | Qualified thermal environmental range |
| MaxCurrent | 0.0 | 25.0 | A | Current consumption limit |
"""
        pkg = self.translator.translate(md_content)
        self.assertEqual(pkg.name, "PowerSubsystemModel")

        constraints = pkg.constraint_defs if pkg.constraint_defs else pkg.part_defs[0].constraints
        self.assertEqual(len(constraints), 3)

        con_map = {c.name: c for c in constraints}
        self.assertIn("assert_BusVoltage_range", con_map)
        self.assertIn("assert_OperatingTemp_range", con_map)
        self.assertIn("assert_MaxCurrent_range", con_map)

        con_v = con_map["assert_BusVoltage_range"]
        self.assertTrue(con_v.is_assertion)
        self.assertEqual(con_v.expression, "BusVoltage >= 18.0 and BusVoltage <= 36.0")
        self.assertIn("DC bus nominal voltage bounds", con_v.doc)
        self.assertIn("unit: V", con_v.doc)

        con_t = con_map["assert_OperatingTemp_range"]
        self.assertTrue(con_t.is_assertion)
        self.assertEqual(con_t.expression, "OperatingTemp >= -40.0 and OperatingTemp <= 85.0")

        # Verify corresponding attributes were created
        attrs = pkg.attribute_defs if pkg.attribute_defs else pkg.part_defs[0].attributes
        attr_map = {a.name: a for a in attrs}
        self.assertIn("BusVoltage", attr_map)
        self.assertEqual(attr_map["BusVoltage"].type_name, "Real")

    def test_multisection_markdown_hierarchy(self):
        """Test multi-section Markdown document parsing with package and component hierarchy."""
        md_content = """# ModularAvionicsSystem

General overview prose describing the architecture of the integrated avionics system.

## FlightController
Primary guidance, navigation, and control compute hardware.

### Interfaces
| Port | Direction | Type | Rate (Hz) | Unit |
| :--- | :--- | :--- | :--- | :--- |
| ControlActuation | out | ActuatorPacket | 100.0 | rad/s |

### Constraints
| Parameter | Min | Max | Unit |
| :--- | :--- | :--- | :--- |
| LoopFrequency | 95.0 | 105.0 | Hz |

## PowerDistribution
Auxiliary power distribution and fault isolation unit.

### Interfaces
| Port | Direction | Type | Rate (Hz) | Unit |
| :--- | :--- | :--- | :--- | :--- |
| PowerBus | inout | PowerState | 10.0 | V |
"""
        pkg = self.translator.translate(md_content)
        self.assertEqual(pkg.name, "ModularAvionicsSystem")
        self.assertEqual(len(pkg.part_defs), 2)

        part_map = {p.name: p for p in pkg.part_defs}
        self.assertIn("FlightController", part_map)
        self.assertIn("PowerDistribution", part_map)

        fc = part_map["FlightController"]
        self.assertIn("Primary guidance, navigation, and control", fc.doc)
        self.assertEqual(len(fc.ports), 1)
        self.assertEqual(fc.ports[0].name, "ControlActuation")
        self.assertEqual(fc.ports[0].direction, "out")
        self.assertEqual(len(fc.constraints), 1)
        self.assertEqual(fc.constraints[0].name, "assert_LoopFrequency_range")

        pd = part_map["PowerDistribution"]
        self.assertIn("Auxiliary power distribution", pd.doc)
        self.assertEqual(len(pd.ports), 1)
        self.assertEqual(pd.ports[0].name, "PowerBus")

    def test_roundtrip_sysmlv2_parsing(self):
        """Test round-trip SysML v2 parsing: pkg.to_sysml() parsed by SysMLParser.parse_text()."""
        md_content = """# IntegratedSystemSpec

## Bill of Materials
| Component | Part Number | Mass (kg) |
| :--- | :--- | :--- |
| CoreProcessor | CP-100 | 1.1 |
| RadioTransceiver | RT-200 | 0.8 |

## System Interfaces
| Port | Direction | Type |
| :--- | :--- | :--- |
| TelemetryOut | out | TelemetryData |

## Operational Limits
| Parameter | Min | Max | Unit |
| :--- | :--- | :--- | :--- |
| SupplyVoltage | 12.0 | 28.0 | V |
"""
        pkg = self.translator.translate(md_content)
        sysml_text = pkg.to_sysml()

        # Parse generated textual SysML model back into AST
        parsed_pkg = SysMLParser.parse_text(sysml_text)
        self.assertIsNotNone(parsed_pkg)
        self.assertEqual(parsed_pkg.name, "IntegratedSystemSpec")

        parsed_part_names = {p.name for p in parsed_pkg.part_defs}
        self.assertIn("CoreProcessor", parsed_part_names)
        self.assertIn("RadioTransceiver", parsed_part_names)

        # Check attribute preservation through round-trip
        core_part = next(p for p in parsed_pkg.part_defs if p.name == "CoreProcessor")
        attr_map = {a.name: a for a in core_part.attributes}
        self.assertIn("part_number", attr_map)
        self.assertIn("mass", attr_map)

    def test_multi_file_markdown_translation(self):
        """Test translation of multiple markdown files into a single merged SysMLPackage."""
        with tempfile.TemporaryDirectory() as tmpdir:
            file1 = os.path.join(tmpdir, "01_bom.md")
            with open(file1, "w", encoding="utf-8") as f:
                f.write("""# SubsystemAlpha
## Bill of Materials
| Component | Part Number | Mass (kg) |
| :--- | :--- | :--- |
| ModuleA | MA-1 | 0.5 |
""")

            file2 = os.path.join(tmpdir, "02_interfaces.md")
            with open(file2, "w", encoding="utf-8") as f:
                f.write("""# SubsystemAlpha
## System Interfaces
| Port | Direction | Type |
| :--- | :--- | :--- |
| PortAlpha | in | StreamA |
""")

            merged_pkg = self.translator.translate_files([file1, file2], default_name="MergedAlpha")
            self.assertEqual(merged_pkg.name, "MergedAlpha")

            # Check that ModuleA is present
            part_names = {p.name for p in merged_pkg.part_defs}
            self.assertIn("ModuleA", part_names)

            # Check that PortAlpha was merged
            port_names = {prt.name for prt in merged_pkg.port_defs}
            self.assertIn("PortAlpha", port_names)


class TestSysMLv2IngestMarkdown(unittest.TestCase):
    """Unit tests for sysmlv2_ingest.py format auto-detection, discovery, and CLI execution."""

    def test_format_detection_md_extension(self):
        """Test detect_format correctly identifies markdown files by extension."""
        fmt = detect_format("specs/hardware_bom.md", "# Hardware Specification\nSome prose...")
        self.assertEqual(fmt, "markdown")

    def test_format_detection_table_headers(self):
        """Test detect_format detects markdown tables even without .md extension."""
        bom_table = "| Component | Part Number | Mass (kg) |\n| :--- | :--- | :--- |\n| Motor | M-1 | 0.5 |"
        self.assertEqual(detect_format("specs/bom.txt", bom_table), "markdown")

        port_table = "| Port | Direction | Type |\n| :--- | :--- | :--- |\n| InPort | in | Real |"
        self.assertEqual(detect_format("specs/ports.txt", port_table), "markdown")

        param_table = "| Parameter | Min | Max |\n| :--- | :--- | :--- |\n| Voltage | 10.0 | 20.0 |"
        self.assertEqual(detect_format("specs/limits.txt", param_table), "markdown")

    def test_schema_directory_auto_discovery_extracted(self):
        """Test discover_schema_targets auto-discovers markdown files in schema/extracted/."""
        with tempfile.TemporaryDirectory() as tmpdir:
            extracted_dir = os.path.join(tmpdir, "schema", "extracted")
            os.makedirs(extracted_dir, exist_ok=True)
            bom_path = os.path.join(extracted_dir, "01_system_bom.md")
            with open(bom_path, "w", encoding="utf-8") as f:
                f.write("# OEM BOM\n| Component | Part Number |\n| :--- | :--- |\n| Unit | U-1 |")

            fmt, discovered = discover_schema_targets(os.path.join(tmpdir, "schema"))
            self.assertEqual(fmt, "markdown")
            self.assertEqual(len(discovered), 1)
            self.assertEqual(discovered[0], bom_path)

    def test_schema_directory_auto_discovery_schema_root_ignoring_readme(self):
        """Test discover_schema_targets auto-discovers markdown in schema/ while ignoring README.md."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            readme_path = os.path.join(schema_dir, "README.md")
            spec_path = os.path.join(schema_dir, "spec_table.md")

            with open(readme_path, "w", encoding="utf-8") as f:
                f.write("# Schema Landing Zone\nPlace schemas here.")
            with open(spec_path, "w", encoding="utf-8") as f:
                f.write("# Spec Table\n| Component | Part Number |\n| :--- | :--- |\n| Item | I-1 |")

            fmt, discovered = discover_schema_targets(schema_dir)
            self.assertEqual(fmt, "markdown")
            self.assertEqual(discovered, [spec_path])

    def test_end_to_end_ingestion_and_compilation(self):
        """Test end-to-end markdown ingestion produces valid SysML v2 and digest compileable by compile_sysml."""
        with tempfile.TemporaryDirectory() as tmpdir:
            md_path = os.path.join(tmpdir, "system_specification.md")
            with open(md_path, "w", encoding="utf-8") as f:
                f.write("""# AutonomousPlatform

## Bill of Materials
| Component | Part Number | Mass (kg) | Power (W) | Description |
| :--- | :--- | :--- | :--- | :--- |
| ComputeBoard | CB-500 | 1.4 | 35.0 | Primary onboard compute platform |
| SensorArray | SA-200 | 0.9 | 15.0 | Integrated sensor acquisition module |

## System Interfaces
| Port | Direction | Type | Rate (Hz) |
| :--- | :--- | :--- | :--- |
| StateVector | out | NavigationState | 100.0 |

## Operational Limits
| Parameter | Min | Max | Unit |
| :--- | :--- | :--- | :--- |
| InputVoltage | 18.0 | 32.0 | V |
""")
            out_sysml = os.path.join(tmpdir, "schema.sysml")
            out_digest = os.path.join(tmpdir, "schema-digest.json")

            pkg, digest = ingest_schema(
                schema_path=md_path,
                format_type="markdown",
                output_path=out_sysml,
                digest_path=out_digest
            )

            # Check output files exist
            self.assertTrue(os.path.exists(out_sysml))
            self.assertTrue(os.path.exists(out_digest))

            # Validate digest contents
            with open(out_digest, "r", encoding="utf-8") as f:
                digest_json = json.load(f)
            self.assertIn("sha256", digest_json)
            self.assertIn("node_counts", digest_json)
            self.assertIn("schema_nodes", digest_json)
            self.assertGreater(digest_json["node_counts"]["part_defs"], 0)

            # Compile with compile_sysml.parse_sysml
            with open(out_sysml, "r", encoding="utf-8") as f:
                compiled_ast = compile_sysml.parse_sysml(f.read())

            self.assertIn("ComputeBoard", compiled_ast["part_defs"])
            self.assertIn("SensorArray", compiled_ast["part_defs"])
            self.assertIn("StateVector", compiled_ast["port_defs"])
            self.assertIn("assert_InputVoltage_range", compiled_ast["constraint_defs"])

    def test_cli_execution_subprocess(self):
        """Test executing sysmlv2_ingest.py CLI with --format markdown via subprocess."""
        with tempfile.TemporaryDirectory() as tmpdir:
            input_md = os.path.join(tmpdir, "oem_manual.md")
            with open(input_md, "w", encoding="utf-8") as f:
                f.write("""# SubsystemModel
## Bill of Materials
| Component | Part Number | Mass (kg) |
| :--- | :--- | :--- |
| SubsystemCore | SC-01 | 3.5 |
""")
            out_sysml = os.path.join(tmpdir, "output_model.sysml")
            out_digest = os.path.join(tmpdir, "output_digest.json")

            cli_script = os.path.join(SPEC_SCRIPTS_DIR, "sysmlv2_ingest.py")
            cmd = [
                sys.executable,
                cli_script,
                "--schema", input_md,
                "--format", "markdown",
                "--out", out_sysml,
                "--digest", out_digest
            ]
            result = subprocess.run(cmd, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, f"CLI execution failed:\n{result.stderr}")
            self.assertTrue(os.path.exists(out_sysml))
            self.assertTrue(os.path.exists(out_digest))


class TestCompileSysMLGateFallback(unittest.TestCase):
    """Unit tests for scripts/compile_sysml.py enforce_pipeline0_compilation_gate remediation."""

    def test_enforce_compilation_gate_missing_schema_outputs_remediation(self):
        """
        Test that running enforce_pipeline0_compilation_gate() when no .sysml
        exists in schema/ outputs the helpful remediation message and exits with code 1.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            # Switch working directory to empty tempdir with empty schema/ directory
            old_cwd = os.getcwd()
            os.chdir(tmpdir)
            os.makedirs("schema", exist_ok=True)
            try:
                stderr_stream = io.StringIO()
                old_stderr = sys.stderr
                sys.stderr = stderr_stream
                try:
                    exit_code = compile_sysml.enforce_pipeline0_compilation_gate()
                finally:
                    sys.stderr = old_stderr

                stderr_output = stderr_stream.getvalue()
                self.assertEqual(exit_code, 1)

                # Verify remediation message components
                self.assertIn("Error: No .sysml schema file found in schema/.", stderr_output)
                self.assertIn("If starting from unstructured OEM prose manuals, PDF documentation, or BOM markdown tables:", stderr_output)
                self.assertIn("1. Place your OEM documentation or extract tables into schema/ or schema/extracted/.", stderr_output)
                self.assertIn("2. Execute Step 0.0 Level 0 OEM Ground Truth Ingestion:", stderr_output)
                self.assertIn("sysmlv2_ingest.py --schema <path_to_markdown> --format markdown --out schema/model.sysml", stderr_output)
                self.assertIn("3. Re-run compile_sysml.py --compile to satisfy the compilation gate.", stderr_output)
            finally:
                os.chdir(old_cwd)

    def test_enforce_compilation_gate_success_with_valid_schema(self):
        """Test that enforce_pipeline0_compilation_gate() succeeds with exit code 0 when valid schema exists."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_file = os.path.join(tmpdir, "valid_model.sysml")
            with open(schema_file, "w", encoding="utf-8") as f:
                f.write("""package ValidSystem {
    part def ControllerUnit {
        attribute cycle_time : Real = 0.01;
    }
}
""")
            out_sysml = os.path.join(tmpdir, "schema.sysml")
            out_digest = os.path.join(tmpdir, "schema-digest.json")

            exit_code = compile_sysml.enforce_pipeline0_compilation_gate(
                schema_path=schema_file,
                output_path=out_sysml,
                digest_path=out_digest
            )
            self.assertEqual(exit_code, 0)
            self.assertTrue(os.path.exists(out_sysml))
            self.assertTrue(os.path.exists(out_digest))


if __name__ == "__main__":
    unittest.main()
