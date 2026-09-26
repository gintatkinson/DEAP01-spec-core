"""
Unit tests for AST Manifest Dispatch Contracts & Typed Parameter Dictionary Projection.

Covers remediation of:
- Issue #377: Spec generation lacks typed parameter dictionary AST projection
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
from parity_auditor.validators.factual_grounding_validator import (
    FactualGroundingValidator,
    SchemaGroundTruth,
)


SAMPLE_SYSML_AST = """package AutonomousVehicle_SSOT {
    doc /* SSOT for Autonomous Flight Vehicle */

    attribute ruddervatorCount : Integer = 4;
    attribute tailConfiguration : String = "X-tail";
    attribute catapultLaunchLimitG : Real = 12.0;
    attribute massKg : Real = 25.0;
    attribute nominalVoltageV : Real = 24.0;
    attribute telemetryRateHz : Real = 50.0;

    part def FlightControlComputer {
        port c2_bus : RS485;
        port telemetry : MAVLink;
        attribute clockFrequencyHz : Real = 400.0;
    }

    part def BatterySystem {
        attribute maxVoltageV : Real = 28.0;
    }
}
"""


class TestASTManifestDispatchContracts(unittest.TestCase):
    """Verifies that the specification engineering skill and validator enforce typed parameter dictionary AST projection."""

    def test_skill_mandates_typed_parameter_dictionary_projection(self):
        """Verify skills/schema-specification-engineering/SKILL.md mandates typed parameter dictionary AST projection."""
        skill_path = os.path.join(repo_root, "skills", "schema-specification-engineering", "SKILL.md")
        self.assertTrue(os.path.isfile(skill_path), f"SKILL.md not found at {skill_path}")

        with open(skill_path, "r", encoding="utf-8") as f:
            content = f.read()

        # Check for mandatory concepts required by Issue #377
        self.assertIn("Typed Parameter Dictionary", content)
        self.assertIn("closed-world", content.lower())
        self.assertIn("hallucinat", content.lower())
        self.assertIn("positive", content.lower())

    def test_schema_ground_truth_to_typed_parameter_dictionary(self):
        """Verify SchemaGroundTruth extracts and projects a typed parameter dictionary AST."""
        validator = FactualGroundingValidator()
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            dict_data = validator.extract_typed_parameter_dictionary(repo)

            self.assertIn("parameters", dict_data)
            params = dict_data["parameters"]

            # Verify key parameters are present with types and values
            self.assertIn("ruddervatorcount", params)
            self.assertEqual(params["ruddervatorcount"]["value"], 4)
            self.assertEqual(params["ruddervatorcount"]["type"], "Integer")

            self.assertIn("catapultlaunchlimitg", params)
            self.assertEqual(params["catapultlaunchlimitg"]["value"], 12.0)
            self.assertEqual(params["catapultlaunchlimitg"]["unit"], "g")

            self.assertIn("masskg", params)
            self.assertEqual(params["masskg"]["value"], 25.0)
            self.assertEqual(params["masskg"]["unit"], "kg")

            self.assertIn("nominalvoltagev", params)
            self.assertEqual(params["nominalvoltagev"]["value"], 24.0)

            # Verify declared protocols and parts are projected
            self.assertIn("rs485", dict_data.get("declared_protocols", []))
            self.assertIn("mavlink", dict_data.get("declared_protocols", []))

    def test_typed_parameter_dictionary_markdown_formatting(self):
        """Verify format_typed_parameter_dictionary_markdown produces a closed-world markdown reference."""
        validator = FactualGroundingValidator()
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_SYSML_AST)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            md_output = validator.format_typed_parameter_dictionary_markdown(repo)

            self.assertIn("Closed-World AST Typed Parameter Dictionary Reference", md_output)
            self.assertIn("| Parameter | Type | Value / Limit | Unit | Component Owner | Bound Type |", md_output)
            self.assertIn("catapultlaunchlimitg", md_output)
            self.assertIn("masskg", md_output)
            self.assertIn("ruddervatorcount", md_output)
            self.assertIn("Declared Protocols", md_output)


if __name__ == "__main__":
    unittest.main()
