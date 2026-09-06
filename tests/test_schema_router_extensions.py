#!/usr/bin/env python3
"""
Unit test suite for SchemaRouter extensions (Objective 1 / Issue #226).
Tests constructor dependency injection, priority ordering with prepend registration,
clean handling of non-schema files (.gitkeep, .DS_Store, .gitignore, dotfiles),
logging of unhandled extensions, and regex parsing of .sysml, .kerml, .md schema files.
"""

import os
import sys
import tempfile
import unittest
from typing import Dict, Optional, Tuple, Any

TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
PARITY_AUDITOR_SRC = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")

for p in (PROJECT_ROOT, PARITY_AUDITOR_SRC):
    if p not in sys.path:
        sys.path.insert(0, p)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.parsers.base import IParser
from parity_auditor.parsers.schema_router import SchemaRouter, parse_schema_file


class DummyCustomParser(IParser):
    def __init__(self, target_ext: str, module_name: str, definitions: Dict[str, str]):
        self.target_ext = target_ext
        self.module_name = module_name
        self.definitions = definitions

    def can_parse(self, filepath: str) -> bool:
        return filepath.endswith(self.target_ext)

    def parse(self, filepath: str) -> Tuple[Optional[str], Dict[str, str]]:
        return self.module_name, self.definitions


class TestSchemaRouterExtensions(unittest.TestCase):
    def setUp(self):
        self.repo = WorkspaceRepository(PROJECT_ROOT)

    def test_constructor_dependency_injection(self):
        custom_parser = DummyCustomParser(".custom", "CustomModule", {"custom:elem": "custom"})
        router = SchemaRouter(self.repo, parsers=[custom_parser])
        self.assertEqual(router._parsers, [custom_parser])
        self.assertTrue(router.can_parse("test_file.custom"))
        self.assertFalse(router.can_parse("test_file.yang"))

        mod_name, defs = router.parse("test_file.custom")
        self.assertEqual(mod_name, "CustomModule")
        self.assertEqual(defs, {"custom:elem": "custom"})

    def test_default_constructor_uses_regex_schema_parser(self):
        router = SchemaRouter(self.repo)
        self.assertEqual(len(router._parsers), 1)
        self.assertTrue(router.can_parse("model.sysml"))
        self.assertTrue(router.can_parse("model.yang"))

    def test_priority_order_with_register_prepend(self):
        router = SchemaRouter(self.repo, parsers=[])
        parser_low = DummyCustomParser(".dup", "LowPriorityModule", {"low:node": "low"})
        parser_high = DummyCustomParser(".dup", "HighPriorityModule", {"high:node": "high"})

        router.register(parser_low)
        router.register(parser_high, prepend=True)

        self.assertEqual(router._parsers[0], parser_high)
        self.assertEqual(router._parsers[1], parser_low)

        mod_name, defs = router.parse("sample.dup")
        self.assertEqual(mod_name, "HighPriorityModule")
        self.assertEqual(defs, {"high:node": "high"})

    def test_clean_handling_of_ignored_non_schema_files(self):
        router = SchemaRouter(self.repo)
        ignored_paths = [
            ".gitkeep",
            ".DS_Store",
            ".gitignore",
            ".env",
            ".hidden_config",
            "schema/.gitkeep",
            "schema/.DS_Store",
            "docs/schema/.gitignore",
        ]

        with self.assertLogs("parity_auditor.parsers.schema_router", level="WARNING") as cm:
            # Trigger a warning with an unhandled file to ensure assertLogs works
            router.parse("unhandled.dummy")
            # All ignored files should return (None, {}) without logging warnings
            for path in ignored_paths:
                mod_name, defs = router.parse(path)
                self.assertIsNone(mod_name)
                self.assertEqual(defs, {})

        # Ensure warning was logged ONLY for unhandled.dummy and not for any ignored files
        for record in cm.records:
            for ignored in ignored_paths:
                base = os.path.basename(ignored)
                self.assertNotIn(f"in {base}", record.getMessage())
        self.assertEqual(len(cm.records), 1)
        self.assertIn("unhandled.dummy", cm.records[0].getMessage())

    def test_warning_logging_for_unhandled_extensions(self):
        router = SchemaRouter(self.repo)
        with self.assertLogs("parity_auditor.parsers.schema_router", level="WARNING") as cm:
            mod_name, defs = router.parse("/workspace/schema/unsupported_format.xyz")
            self.assertEqual(mod_name, "unsupported_format.xyz")
            self.assertEqual(defs, {})

        self.assertEqual(len(cm.records), 1)
        self.assertIn(".xyz", cm.records[0].getMessage())
        self.assertIn("unsupported_format.xyz", cm.records[0].getMessage())

    def test_parse_schema_file_with_router_dependency_injection(self):
        custom_parser = DummyCustomParser(".ext", "InjectedModule", {"injected:key": "val"})
        custom_router = SchemaRouter(self.repo, parsers=[custom_parser])

        mod_name, defs = parse_schema_file("data.ext", repo=self.repo, router=custom_router)
        self.assertEqual(mod_name, "InjectedModule")
        self.assertEqual(defs, {"injected:key": "val"})

    def test_parsing_sysml_schema_file(self):
        content = """// SysML v2 Test Schema
package FlightControlSystem {
    part def FlightComputer;
    port def NavigationPort;
    action def CalculateTrajectory;
    state def ArmedState;
    constraint def MaxVelocityConstraint;
    requirement def SafetyRequirement;
    interface def BusInterface;
    item def TelemetryPacket;
    attribute def Altitude;
}
"""
        with tempfile.NamedTemporaryFile(suffix=".sysml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(content)
            f.flush()
            temp_path = f.name

        try:
            mod_name, defs = parse_schema_file(temp_path, repo=self.repo)
            self.assertEqual(mod_name, "FlightControlSystem")
            self.assertIn("part:FlightComputer", defs)
            self.assertIn("port:NavigationPort", defs)
            self.assertIn("action:CalculateTrajectory", defs)
            self.assertIn("state:ArmedState", defs)
            self.assertIn("constraint:MaxVelocityConstraint", defs)
            self.assertIn("requirement:SafetyRequirement", defs)
            self.assertIn("interface:BusInterface", defs)
            self.assertIn("item:TelemetryPacket", defs)
            self.assertIn("attribute:Altitude", defs)
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_parsing_kerml_schema_file(self):
        content = """/* KerML Test Schema */
package CoreDimensions {
    feature def MathFeature;
    function def CalculateThrust;
    type def Vector3;
    classifier def SpatialEntity;
    datatype def Timestamp;
    struct def ConfigStruct;
    behavior def FlightBehavior;
    step def InitializationStep;
    dimension LengthDimension;
    unit Meter;
}
"""
        with tempfile.NamedTemporaryFile(suffix=".kerml", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(content)
            f.flush()
            temp_path = f.name

        try:
            mod_name, defs = parse_schema_file(temp_path, repo=self.repo)
            self.assertEqual(mod_name, "CoreDimensions")
            self.assertIn("feature:MathFeature", defs)
            self.assertIn("function:CalculateThrust", defs)
            self.assertIn("type:Vector3", defs)
            self.assertIn("classifier:SpatialEntity", defs)
            self.assertIn("datatype:Timestamp", defs)
            self.assertIn("struct:ConfigStruct", defs)
            self.assertIn("behavior:FlightBehavior", defs)
            self.assertIn("step:InitializationStep", defs)
            self.assertIn("dimension:LengthDimension", defs)
            self.assertIn("unit:Meter", defs)
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)

    def test_parsing_md_schema_file(self):
        content = """# FlightDataDictionary

Specification of data types and entities.

concept FlightPlan
entity DroneTelemetry
type Coordinate3D
field TargetAltitude
property BatteryLevel
section DiagnosticsSection
structure WaypointStruct
model GuidanceModel
"""
        with tempfile.NamedTemporaryFile(suffix=".md", mode="w+", encoding="utf-8", delete=False) as f:
            f.write(content)
            f.flush()
            temp_path = f.name

        try:
            mod_name, defs = parse_schema_file(temp_path, repo=self.repo)
            self.assertEqual(mod_name, "FlightDataDictionary")
            self.assertIn("concept:FlightPlan", defs)
            self.assertIn("entity:DroneTelemetry", defs)
            self.assertIn("type:Coordinate3D", defs)
            self.assertIn("field:TargetAltitude", defs)
            self.assertIn("property:BatteryLevel", defs)
            self.assertIn("section:DiagnosticsSection", defs)
            self.assertIn("structure:WaypointStruct", defs)
            self.assertIn("model:GuidanceModel", defs)
        finally:
            if os.path.exists(temp_path):
                os.unlink(temp_path)


if __name__ == "__main__":
    unittest.main()
