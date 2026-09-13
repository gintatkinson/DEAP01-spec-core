"""
Unit tests for FactualGroundingValidator within parity_auditor test package.
"""

import os
import sys
import tempfile
import unittest

parity_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
parity_src = os.path.join(parity_root, "src")
if parity_src not in sys.path:
    sys.path.insert(0, parity_src)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.factual_grounding_validator import FactualGroundingValidator
from parity_auditor.aggregator import AGGREGATING_VALIDATORS

SAMPLE_GROUND_TRUTH_SYSML = """package AutonomousVehicle_SSOT {
    doc /* SSOT for Autonomous Flight Vehicle */

    attribute ruddervatorCount : Integer = 4;
    attribute tailConfiguration : String = "X-tail";
    attribute catapultLaunchLimitG : Real = 12.0;

    part def FlightControlComputer {
        port c2_bus : RS485;
        port telemetry : MAVLink;
    }
}
"""


class TestFactualGroundingValidatorInternal(unittest.TestCase):
    def setUp(self):
        self.validator = FactualGroundingValidator()

    def test_registered_in_aggregating_validators(self):
        """Verify FactualGroundingValidator is in AGGREGATING_VALIDATORS."""
        self.assertIn(FactualGroundingValidator, AGGREGATING_VALIDATORS)

    def test_clean_landing_zone(self):
        """Verify clean landing zone passes with 0 findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            with open(os.path.join(schema_dir, ".gitkeep"), "w") as f:
                f.write("")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo)
            self.assertEqual(findings, [])

    def test_detects_structural_and_protocol_drift(self):
        """Verify structural count drift and unverified protocol detection."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w") as f:
                f.write(SAMPLE_GROUND_TRUTH_SYSML)

            with open(os.path.join(docs_dir, "FEAT_DRIFT.md"), "w") as f:
                f.write("""# Feature Drift
The vehicle uses 2 ruddervators and communicates over STANAG 4586 datalink.
""")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            rule_ids = {f.rule_id for f in findings}
            self.assertIn("factual-grounding-numeric-drift", rule_ids)
            self.assertIn("factual-grounding-unverified-protocol", rule_ids)


if __name__ == "__main__":
    unittest.main()
