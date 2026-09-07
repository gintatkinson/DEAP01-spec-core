#!/usr/bin/env python3
"""
Unit test suite for SysML v2 AST capability def support and parser mechanics (DEAP01-spec-core).

Verifies that:
1. Formal `capability def` nodes (and `perform Capability;` usages) parse cleanly into
   canonical AST dataclasses and AST dictionaries with name, doc comments, subsystem,
   and parent package attribution.
2. `SysMLParser.parse_text`, `SysMLParser.parse_to_dict`, and `SysMLPackage.to_dict()`
   faithfully preserve capability definitions across package and part levels.
3. `compile_sysml.py` and `sysmlv2_ingest.py` extract and digest capability definitions.
4. Round-trip textual serialization `to_sysml()` preserves capability definitions and docstrings.
"""

import os
import sys
import tempfile
import unittest
from typing import Dict, Any, List

PROJECT_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")
PARITY_AUDITOR_SRC = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")

for p in (SPEC_SCRIPTS_DIR, PARITY_AUDITOR_SRC, PROJECT_ROOT):
    if p not in sys.path:
        sys.path.insert(0, p)

from sysmlv2_ast import (
    SysMLPackage,
    PartDef,
    PortDef,
    ActionDef,
    AttributeDef,
    SysMLOperationDef,
    SysMLCapabilityDef,
    SysMLInteractionDef,
    SysMLConstraintDef,
    SysMLTestCaseDef,
    RequirementDef,
    StateDef,
    UseCaseDef,
    ItemDef,
    SysMLParser,
)
from scripts.compile_sysml import parse_sysml
from sysmlv2_ingest import ingest_schema, detect_format
from parity_auditor.parsers.schema_router import parse_schema_file
from parity_auditor.core.workspace import WorkspaceRepository


SAMPLE_SYSML_CAPABILITIES = """
package UAS_Surveillance_Mission {
    doc /* UAS wide-area surveillance and reconnaissance package */

    attribute missionId : String = "MISSION-001";
    port inout telemetryBus : TelemetryPort;

    doc /* Real-time detect and avoid autonomous capability */
    capability def DetectAndAvoid {
        subsystem FlightGuidance;
        description: "Executes emergency trajectory deconfliction and collision avoidance";
    }

    capability def PrecisionLanding {
        doc /* Autonomous beacon-guided precision approach and landing */
        subsystem RecoverySubsystem;
    }

    doc /* Long-range sensor surveillance capability */
    capability def OpticalSurveillance;

    perform AutonomousNavigation;

    part def MissionComputer {
        doc /* Primary mission processor and payload supervisor */
        attribute cpuLoad : Real = 0.25;

        capability def SensorFusion {
            doc /* Multi-sensor telemetry and LiDAR fusion capability */
            subsystem MissionComputer;
        }

        perform TargetTracking;
        perform capability ThermalImaging;

        action ExecuteScan;
    }

    part def FlightController {
        doc /* Real-time flight stabilization controller */
        perform FlightStabilization;
    }
}
"""


class TestSysMLParserCapabilityDef(unittest.TestCase):
    """Test suite for formal capability def AST nodes, perform usages, and dictionary serialization."""

    def test_capability_def_dataclass_and_dictionary_serialization(self):
        """Verify SysMLCapabilityDef dataclass instantiation, parent attribution, and to_dict()."""
        cap = SysMLCapabilityDef(
            name="AutonomousCollisionAvoidance",
            description="Real-time evasive maneuvers",
            subsystem="FlightGuidance",
            package_ref="UAS_Package",
            parent_package="UAS_Package",
            doc="Collision avoidance docstring"
        )
        self.assertEqual(cap.name, "AutonomousCollisionAvoidance")
        self.assertEqual(cap.description, "Real-time evasive maneuvers")
        self.assertEqual(cap.doc, "Collision avoidance docstring")
        self.assertEqual(cap.subsystem, "FlightGuidance")
        self.assertEqual(cap.parent_package, "UAS_Package")

        cap_dict = cap.to_dict()
        self.assertIsInstance(cap_dict, dict)
        self.assertEqual(cap_dict["name"], "AutonomousCollisionAvoidance")
        self.assertEqual(cap_dict["doc"], "Collision avoidance docstring")
        self.assertEqual(cap_dict["parent_package"], "UAS_Package")
        self.assertEqual(cap_dict["subsystem"], "FlightGuidance")

        sysml_code = cap.to_sysml()
        self.assertIn("capability def AutonomousCollisionAvoidance", sysml_code)
        self.assertIn("doc /* Collision avoidance docstring */", sysml_code)
        self.assertIn("subsystem FlightGuidance;", sysml_code)

    def test_parse_capability_defs_in_package(self):
        """Verify parsing of block and statement capability defs at package level with parent attribution."""
        pkg = SysMLParser.parse_text(SAMPLE_SYSML_CAPABILITIES)
        self.assertEqual(pkg.name, "UAS_Surveillance_Mission")
        self.assertEqual(pkg.doc, "UAS wide-area surveillance and reconnaissance package")

        # Package-level capabilities: DetectAndAvoid, PrecisionLanding, OpticalSurveillance, AutonomousNavigation
        cap_names = [c.name for c in pkg.capability_defs]
        self.assertIn("DetectAndAvoid", cap_names)
        self.assertIn("PrecisionLanding", cap_names)
        self.assertIn("OpticalSurveillance", cap_names)
        self.assertIn("AutonomousNavigation", cap_names)

        cap_map = {c.name: c for c in pkg.capability_defs}

        # Check DetectAndAvoid
        daa = cap_map["DetectAndAvoid"]
        self.assertEqual(daa.name, "DetectAndAvoid")
        self.assertIn("deconfliction", daa.description.lower() + daa.doc.lower())
        self.assertEqual(daa.subsystem, "FlightGuidance")
        self.assertEqual(daa.parent_package, "UAS_Surveillance_Mission")

        # Check PrecisionLanding
        pl = cap_map["PrecisionLanding"]
        self.assertEqual(pl.name, "PrecisionLanding")
        self.assertIn("precision approach", pl.doc.lower() + pl.description.lower())
        self.assertEqual(pl.subsystem, "RecoverySubsystem")
        self.assertEqual(pl.parent_package, "UAS_Surveillance_Mission")

        # Check OpticalSurveillance (statement capability def with leading doc)
        opt = cap_map["OpticalSurveillance"]
        self.assertEqual(opt.name, "OpticalSurveillance")
        self.assertIn("surveillance", opt.doc.lower() + opt.description.lower())
        self.assertEqual(opt.parent_package, "UAS_Surveillance_Mission")

        # Check AutonomousNavigation (from perform statement)
        nav = cap_map["AutonomousNavigation"]
        self.assertEqual(nav.name, "AutonomousNavigation")
        self.assertEqual(nav.parent_package, "UAS_Surveillance_Mission")

    def test_parse_capability_defs_in_parts_and_perform_usages(self):
        """Verify parsing of capability defs and perform statements inside part definitions."""
        pkg = SysMLParser.parse_text(SAMPLE_SYSML_CAPABILITIES)
        part_map = {p.name: p for p in pkg.part_defs}

        # Check MissionComputer capabilities
        mc = part_map["MissionComputer"]
        mc_caps = {c.name: c for c in mc.capabilities}
        self.assertIn("SensorFusion", mc_caps)
        self.assertIn("TargetTracking", mc_caps)
        self.assertIn("ThermalImaging", mc_caps)

        sf = mc_caps["SensorFusion"]
        self.assertIn("fusion", sf.doc.lower() + sf.description.lower())
        self.assertEqual(sf.subsystem, "MissionComputer")

        tt = mc_caps["TargetTracking"]
        self.assertEqual(tt.subsystem, "MissionComputer")

        ti = mc_caps["ThermalImaging"]
        self.assertEqual(ti.subsystem, "MissionComputer")

        # Check FlightController capabilities
        fc = part_map["FlightController"]
        fc_caps = {c.name: c for c in fc.capabilities}
        self.assertIn("FlightStabilization", fc_caps)
        self.assertEqual(fc_caps["FlightStabilization"].subsystem, "FlightController")

    def test_parse_to_dict_ast_representation(self):
        """Verify SysMLParser.parse_to_dict and SysMLPackage.to_dict() produce complete AST dicts."""
        ast_dict = SysMLParser.parse_to_dict(SAMPLE_SYSML_CAPABILITIES)
        self.assertEqual(ast_dict["name"], "UAS_Surveillance_Mission")
        self.assertEqual(ast_dict["doc"], "UAS wide-area surveillance and reconnaissance package")

        self.assertIn("capability_defs", ast_dict)
        cap_names = [c["name"] for c in ast_dict["capability_defs"]]
        self.assertIn("DetectAndAvoid", cap_names)
        self.assertIn("PrecisionLanding", cap_names)
        self.assertIn("OpticalSurveillance", cap_names)
        self.assertIn("AutonomousNavigation", cap_names)

        for c in ast_dict["capability_defs"]:
            self.assertIn("name", c)
            self.assertIn("doc", c)
            self.assertIn("parent_package", c)
            self.assertEqual(c["parent_package"], "UAS_Surveillance_Mission")

        # Check part_defs dictionary representation
        self.assertIn("part_defs", ast_dict)
        mc_dict = next(p for p in ast_dict["part_defs"] if p["name"] == "MissionComputer")
        mc_cap_names = [c["name"] for c in mc_dict["capabilities"]]
        self.assertIn("SensorFusion", mc_cap_names)
        self.assertIn("TargetTracking", mc_cap_names)
        self.assertIn("ThermalImaging", mc_cap_names)

    def test_compile_sysml_extract_ast_capabilities(self):
        """Verify compile_sysml.py parse_sysml and extract_sysml_ast extract all capability definitions."""
        ast = parse_sysml(SAMPLE_SYSML_CAPABILITIES)
        self.assertIn("capability_defs", ast)
        caps = ast["capability_defs"]
        self.assertIn("DetectAndAvoid", caps)
        self.assertIn("PrecisionLanding", caps)
        self.assertIn("OpticalSurveillance", caps)
        self.assertIn("AutonomousNavigation", caps)
        self.assertIn("SensorFusion", caps)
        self.assertIn("TargetTracking", caps)
        self.assertIn("ThermalImaging", caps)
        self.assertIn("FlightStabilization", caps)

    def test_sysmlv2_ingest_detect_format_and_digest(self):
        """Verify sysmlv2_ingest recognizes capability def schemas and digests capability nodes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_path = os.path.join(tmpdir, "capabilities_model.sysml")
            out_sysml = os.path.join(tmpdir, "schema.sysml")
            digest_path = os.path.join(tmpdir, "schema-digest.json")

            with open(schema_path, "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_CAPABILITIES)

            self.assertEqual(detect_format(schema_path, SAMPLE_SYSML_CAPABILITIES), "sysml")

            pkg, digest = ingest_schema(
                schema_path=schema_path,
                format_type="sysml",
                output_path=out_sysml,
                digest_path=digest_path
            )

            self.assertGreaterEqual(digest["node_counts"]["capability_defs"], 4)
            self.assertIn("DetectAndAvoid", digest["schema_nodes"])
            self.assertIn("PrecisionLanding", digest["schema_nodes"])
            self.assertIn("SensorFusion", digest["schema_nodes"])

    def test_regex_schema_router_parses_capability_defs(self):
        """Verify parity_auditor SchemaRouter and RegexSchemaParser parse capability defs from .sysml."""
        repo = WorkspaceRepository(PROJECT_ROOT)
        with tempfile.NamedTemporaryFile(suffix=".sysml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(SAMPLE_SYSML_CAPABILITIES)
            f.flush()
            temp_path = f.name

        try:
            mod_name, defs = parse_schema_file(temp_path, repo=repo)
            self.assertEqual(mod_name, "UAS_Surveillance_Mission")
            self.assertIn("capability:DetectAndAvoid", defs)
            self.assertIn("capability:PrecisionLanding", defs)
            self.assertIn("capability:OpticalSurveillance", defs)
            self.assertIn("capability:SensorFusion", defs)
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_roundtrip_serialization_preserves_capabilities(self):
        """Verify model -> to_sysml() -> re-parse preserves capability defs and docstrings."""
        pkg1 = SysMLParser.parse_text(SAMPLE_SYSML_CAPABILITIES)
        emitted_text = pkg1.to_sysml()

        pkg2 = SysMLParser.parse_text(emitted_text)
        self.assertEqual(pkg2.name, pkg1.name)

        cap_names_1 = [c.name for c in pkg1.capability_defs]
        cap_names_2 = [c.name for c in pkg2.capability_defs]
        self.assertEqual(sorted(cap_names_1), sorted(cap_names_2))


if __name__ == "__main__":
    unittest.main()
