"""
Unit tests for Check 22 (Physical Invariant Semantic Prose Gate) and SemanticProseInvariantValidator.

Verifies:
1. test_check22_rejects_ungrounded_landing_prose:
   Rejects positive operational landing/touchdown/flare/gear prose when landingGear == "No" or LifecycleType == "Expendable".
2. test_check22_rejects_ungrounded_recovery_prose:
   Rejects positive recovery/parachute prose when recoverySystem == "None" or chuteEnabled == false.
3. test_check22_accepts_valid_negative_assertions:
   Accepts valid negative assertions ("Recovery system: No", "does not deploy", "no parachute recovery",
   "zero recovery landing", "landing gear is not installed", "without vehicle recovery", MCDA trade studies, code blocks, comments).
4. test_check22_dynamic_domain_adaptation:
   Adapts dynamically to custom negative attributes without hardcoding and permits enabled features.
5. test_check22_upstream_clean_landing_zone:
   Gracefully passes with 0 findings when schema is empty or has only .gitkeep.
6. Registration in AGGREGATING_VALIDATORS and scripts/verify_downstream_baseline.py Check 22 integration.
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
from parity_auditor.validators.semantic_prose_invariant_validator import SemanticProseInvariantValidator
from parity_auditor.aggregator import AGGREGATING_VALIDATORS
from scripts.verify_downstream_baseline import check_semantic_prose_invariants, run_all_checks

# Load SysML AST classes
from parity_auditor.utils.sysml_loader import load_sysml_ast_members

_sysml_ast = load_sysml_ast_members([
    "SysMLPackage", "SysMLParser", "PartDef", "AttributeDef"
])
SysMLPackage = _sysml_ast.SysMLPackage
SysMLParser = _sysml_ast.SysMLParser


SAMPLE_EXPENDABLE_SYSML = """package AutonomousSystem_SSOT {
    doc /* SSOT for Expendable System Architecture */

    attribute lifecycleType : String = "Expendable";
    attribute recoverySystem : String = "None";
    attribute chuteEnabled : Boolean = false;
    attribute landingGear : String = "No";

    part def Airframe {
        attribute massKg : Float64 = 15.0;
    }
}
"""


class TestCheck22SemanticProseGate(unittest.TestCase):
    def setUp(self):
        self.validator = SemanticProseInvariantValidator()

    def test_registered_in_aggregating_validators(self):
        """Verify SemanticProseInvariantValidator is registered in AGGREGATING_VALIDATORS."""
        self.assertIn(SemanticProseInvariantValidator, AGGREGATING_VALIDATORS)

    def test_check22_rejects_ungrounded_landing_prose(self):
        """Verify positive operational landing/gear claims are rejected for expendable system."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_EXPENDABLE_SYSML)

            conops_md = """# Concept of Operations

## Mission Execution
During the final phase of flight, the UAV has landed safely at the primary runway.
The flight controller lowers landing gear and executes flare maneuver and touches down on runway.
"""
            with open(os.path.join(docs_dir, "CONOPS_01.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 2)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("semantic-prose-physical-invariant-violation", rule_ids)
            findings_text = " ".join([str(f) for f in findings])
            self.assertTrue(
                "has landed" in findings_text
                or "lowers landing gear" in findings_text
                or "flare maneuver" in findings_text
                or "touches down" in findings_text
            )

    def test_check22_rejects_ungrounded_recovery_prose(self):
        """Verify positive recovery and parachute claims are rejected when recovery is disabled."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "system.sysml"), "w", encoding="utf-8") as f:
                f.write("""package TestModel {
    attribute recoverySystem : String = "No";
    attribute chuteEnabled : Int = 0;
}
""")

            feat_md = """# Feature: Terminal Flight Sequence

## Operational Flow
1. Following vehicle recovery, flight data loggers are inspected.
2. The system deploys parachute upon receiving terminal abort signal.
3. Operator issues commanded recovery landing to return the asset.
"""
            with open(os.path.join(docs_dir, "Feat-01.md"), "w", encoding="utf-8") as f:
                f.write(feat_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 2)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("semantic-prose-physical-invariant-violation", rule_ids)
            findings_text = " ".join([str(f) for f in findings])
            self.assertTrue(
                "vehicle recovery" in findings_text
                or "deploys parachute" in findings_text
                or "recovery landing" in findings_text
            )

    def test_check22_accepts_valid_negative_assertions(self):
        """Verify valid negative assertions, MCDA trade studies, code blocks, and comments pass with 0 errors."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_EXPENDABLE_SYSML)

            conops_md = """# Concept of Operations

## System Architecture Summary
- Recovery System: None
- Landing Gear: No
- Chute: Disabled

## Physical Invariants & Operational Constraints
The vehicle is an expendable platform.
The flight controller does not deploy a parachute under any condition.
No parachute recovery is performed during terminal operations.
Zero recovery landing operations are conducted.
Landing gear is not installed on this airframe.
The mission terminates without vehicle recovery.

## Trade Study (MCDA Alternatives Analysis)
In trade space exploration, Alternative B included parachute recovery and wheeled landing gear,
where the system deploys parachute and lowers landing gear for runway landing, but was rejected.

## Glossary
- Vehicle Recovery: The physical retrieval of a reusable asset.
- Autoland: Autonomous landing sequence on designated runway.

## Architecture Diagram
```mermaid
flowchart TD
    StateTerminal --> StateImpact: Terminate Flight
```

<!-- Note: Following vehicle recovery discussion from legacy draft -->
"""
            with open(os.path.join(docs_dir, "CONOPS_01.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertEqual(findings, [])

    def test_check22_dynamic_domain_adaptation(self):
        """Verify dynamic domain adaptation for custom attributes and enabled features."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            # 1. Model with custom disabled capability (satellite uplink disabled, water proofing no)
            # but recovery system enabled as Parachute!
            with open(os.path.join(schema_dir, "custom.sysml"), "w", encoding="utf-8") as f:
                f.write("""package CustomDomain {
    attribute recoverySystem : String = "Parachute";
    attribute satelliteUplink : String = "Disabled";
    attribute waterProofing : String = "No";
}
""")

            doc_content = """# Feature Specification

## Telemetry Flow
The system transmits telemetry via satellite uplink during high altitude cruise.
The system deploys parachute for recovery safely at waypoint 4.
"""
            with open(os.path.join(docs_dir, "Feat-02.md"), "w", encoding="utf-8") as f:
                f.write(doc_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            # Satellite uplink should be flagged because satelliteUplink == Disabled
            # Parachute deployment should NOT be flagged because recoverySystem == "Parachute" (enabled)
            findings_text = " ".join([str(f) for f in findings])
            self.assertTrue("satellite uplink" in findings_text or "satellite" in findings_text)
            self.assertFalse("deploys parachute" in findings_text)

    def test_check22_upstream_clean_landing_zone(self):
        """Verify empty schema directory (clean landing zone) yields 0 findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            # Only .gitkeep
            with open(os.path.join(schema_dir, ".gitkeep"), "w", encoding="utf-8") as f:
                pass

            with open(os.path.join(docs_dir, "README.md"), "w", encoding="utf-8") as f:
                f.write("# Specification Template Landing Zone\n")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])
            self.assertEqual(findings, [])

    def test_check22_ast_attribute_def_grounding_nested_part(self):
        """Verify that AttributeDef declarations inside PartDef AST blocks ground physical invariants."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            sysml_content = """package SubsystemModel {
    part def UndercarriageAssembly {
        attribute landingGear : String = "None";
        attribute chuteEnabled : Boolean = false;
    }
}
"""
            with open(os.path.join(schema_dir, "parts.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            conops_md = """# Subsystem ConOps

## Landing & Recovery
The flight controller lowers landing gear and deploys parachute upon approach.
"""
            with open(os.path.join(docs_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_md)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            self.assertTrue(len(findings) >= 2)
            rule_ids = {f.rule_id for f in findings}
            self.assertIn("semantic-prose-physical-invariant-violation", rule_ids)
            findings_text = " ".join([str(f) for f in findings])
            self.assertTrue("landing gear" in findings_text)
            self.assertTrue("parachute" in findings_text)

    def test_check22_cross_reference_configuration_and_numeric_claims(self):
        """Verify cross-referencing configuration and numeric AST attribute declarations against narrative prose."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            sysml_content = """package PlatformSSOT {
    attribute lifecycleType : String = "Expendable";
    attribute waterProofing : String = "Disabled";
    attribute recoverySystem : String = "None";

    part def AvionicsCore {
        attribute satelliteUplink : String = "No";
    }
}
"""
            with open(os.path.join(schema_dir, "platform.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            # Document 1: Positive claims -> must fail
            with open(os.path.join(docs_dir, "Feat-Positive.md"), "w", encoding="utf-8") as f:
                f.write("""# Feature: Cruise and Return
The vehicle transmits high-bandwidth telemetry over satellite uplink and executes recovery landing.
""")

            # Document 2: Valid negative assertions -> must pass
            with open(os.path.join(docs_dir, "Feat-Negative.md"), "w", encoding="utf-8") as f:
                f.write("""# Feature: Expendable Terminal Phase
The platform operates without vehicle recovery.
Satellite uplink is not installed on this airframe.
Water proofing is disabled for expendable operations.
""")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            findings = self.validator.validate(repo, scan_dirs=["docs"])

            # Only Feat-Positive.md should have findings
            violating_files = {f.detail.get("file") for f in findings if f.detail}
            self.assertIn(os.path.join("docs", "features", "Feat-Positive.md"), violating_files)
            self.assertNotIn(os.path.join("docs", "features", "Feat-Negative.md"), violating_files)

    def test_check22_baseline_verification_integration(self):
        """Verify check_semantic_prose_invariants function behaves properly in clean and failing workspaces."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            with open(os.path.join(schema_dir, ".gitkeep"), "w", encoding="utf-8") as f:
                pass

            # Clean landing zone should return cleanly without exit
            check_semantic_prose_invariants(tmpdir)

        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            docs_dir = os.path.join(tmpdir, "docs")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(docs_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(SAMPLE_EXPENDABLE_SYSML)

            with open(os.path.join(docs_dir, "spec.md"), "w", encoding="utf-8") as f:
                f.write("# Spec\nFollowing vehicle recovery, the UAV has landed.\n")

            with self.assertRaises(SystemExit) as cm:
                check_semantic_prose_invariants(tmpdir)
            self.assertEqual(cm.exception.code, 1)


if __name__ == "__main__":
    unittest.main()
