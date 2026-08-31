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
from parity_auditor.validators.concept_provenance_validator import (
    ConceptProvenanceValidator,
    _normalize_identifier,
)


class TestConceptProvenanceValidator(unittest.TestCase):
    def test_clean_upstream_landing_zone_passes(self):
        """Verify that empty schema/ landing zone passes gracefully with zero errors."""
        with tempfile.TemporaryDirectory() as tmpdir:
            os.makedirs(os.path.join(tmpdir, "schema"), exist_ok=True)
            os.makedirs(os.path.join(tmpdir, "docs", "conops"), exist_ok=True)

            with open(os.path.join(tmpdir, "docs", "conops", "conops.md"), "w", encoding="utf-8") as f:
                f.write("# Concept of Operations\n\nGeneric operational description.\n")

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_dynamic_extraction_from_sysml_and_markdown(self):
        """Verify dynamic ground truth extraction from abstract SysML model and extracted markdown."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            os.makedirs(extracted_dir, exist_ok=True)

            # Write abstract SysML schema
            sysml_content = """
            package GenericSystemSchema {
                attribute system_mass : Real = 1800.0 [kg];
                attribute nominal_power : Real = 75.5 [kW];
                attribute payload_volume : Real = 42.0 [m3];
            }
            """
            with open(os.path.join(schema_dir, "system.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            # Write abstract extracted markdown table
            md_content = """
            # Extracted Equipment Specification

            | Property | Value |
            | :--- | :--- |
            | battery_capacity | 100.0 kWh |
            | maximum_span | 12.0 m |
            """
            with open(os.path.join(extracted_dir, "specs.md"), "w", encoding="utf-8") as f:
                f.write(md_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            gt = validator.extract_ground_truth(repo)

            self.assertIn("system_mass", gt)
            self.assertEqual(gt["system_mass"].value, 1800.0)
            self.assertEqual(gt["nominal_power"].value, 75.5)
            self.assertEqual(gt["payload_volume"].value, 42.0)
            self.assertEqual(gt["battery_capacity"].value, 100.0)
            self.assertEqual(gt["maximum_span"].value, 12.0)

    def test_parametric_assertion_within_tolerance(self):
        """Verify that assertions within +/- 5% tolerance pass without error."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "specs.md"), "w", encoding="utf-8") as f:
                f.write("| Property | Value |\n|---|---|\n| system_mass | 1800.0 kg |\n")

            # 1820.0 is 1.11% deviation (< 5%)
            conops_content = """# ConOps
<!-- Source: schema/extracted/specs.md -->

The vehicle has a system_mass = 1820.0 kg for target mission operations.
"""
            with open(os.path.join(conops_dir, "conops.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_parametric_mismatch_detected(self):
        """Verify that assertions exceeding +/- 5% tolerance are flagged with error findings."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "specs.md"), "w", encoding="utf-8") as f:
                f.write("| Property | Value |\n|---|---|\n| system_mass | 1800.0 kg |\n")

            # 3000.0 is 66.7% deviation (> 5%)
            conops_content = """# ConOps
<!-- Source: schema/extracted/specs.md -->

The vehicle has a system_mass = 3000.0 kg for target mission operations.
"""
            with open(os.path.join(conops_dir, "conops.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "concept-provenance-parametric-mismatch")
            self.assertIn("system_mass", str(errors[0]))
            self.assertIn("3000.0", str(errors[0]))

    def test_missing_source_citation_detected(self):
        """Verify that specification claiming schema parameters without citation is flagged."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "specs.md"), "w", encoding="utf-8") as f:
                f.write("| Property | Value |\n|---|---|\n| system_mass | 1800.0 kg |\n")

            # Valid value but missing citation anchor
            conops_content = """# ConOps

The vehicle has a system_mass = 1800.0 kg for target mission operations.
"""
            with open(os.path.join(conops_dir, "conops.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)

            self.assertEqual(len(errors), 1)
            self.assertEqual(errors[0].rule_id, "concept-provenance-missing-source-citation")

    def test_circular_sysml_concept_dependency_fails_on_sysml_citation(self):
        """Verify that Level 1 concept documents citing .sysml emit circular-sysml-concept-dependency."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "Avenger5.sysml"), "w", encoding="utf-8") as f:
                f.write("package Avenger5 {\n    attribute system_mass = 1800.0 [kg];\n}\n")

            conops_content = """# Mission Intent
<!-- Source: schema/Avenger5.sysml -->

The vehicle has a system_mass = 1800.0 kg.
"""
            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)

            circular_errors = [e for e in errors if e.rule_id == "circular-sysml-concept-dependency"]
            self.assertEqual(len(circular_errors), 1)
            self.assertEqual(circular_errors[0].rule_id, "circular-sysml-concept-dependency")
            self.assertIn("Level 1 concept document cites mutable SysML model", str(circular_errors[0]))
            self.assertIn("schema/Avenger5.sysml", str(circular_errors[0]))

    def test_concept_document_citing_extracted_passes(self):
        """Verify that Level 1 concept documents citing Level 0 OEM ground truth in schema/extracted/ pass without circular dependency."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "flight_manual.md"), "w", encoding="utf-8") as f:
                f.write("# Flight Manual\n\n| Property | Value |\n|---|---|\n| system_mass | 1800.0 kg |\n")

            conops_content = """# Mission Intent
<!-- Source: schema/extracted/flight_manual.md -->

The vehicle has a system_mass = 1800.0 kg for primary operational profile.
"""
            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)

            circular_errors = [e for e in errors if e.rule_id == "circular-sysml-concept-dependency"]
            self.assertEqual(circular_errors, [])
            self.assertEqual(errors, [])

    def test_downstream_feature_spec_citing_sysml_passes(self):
        """Verify that downstream specification documents (e.g. features) citing SysML models pass without circular dependency error."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            feat_dir = os.path.join(tmpdir, "docs", "features")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(feat_dir, exist_ok=True)

            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write("package System {\n    attribute system_mass = 1800.0 [kg];\n}\n")

            feat_content = """# Feature: Power Distribution
<!-- Source: schema/model.sysml -->

The vehicle has a system_mass = 1800.0 kg for target mission operations.
"""
            with open(os.path.join(feat_dir, "feat-01.md"), "w", encoding="utf-8") as f:
                f.write(feat_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_semantic_oem_provenance_fails_on_parachute_assertion(self):
        """Verify that asserting recovery parachute when OEM declares Recovery system: No fails."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "oem_specs.md"), "w", encoding="utf-8") as f:
                f.write("# OEM Specifications\n\n| Property | Value |\n|---|---|\n| system_mass | 1800.0 kg |\n| Recovery system | No |\n")

            conops_content = """# Mission Intent
<!-- Source: schema/extracted/oem_specs.md -->

The vehicle incorporates a ballistic parachute for emergency recovery.
"""
            with open(os.path.join(conops_dir, "MISSION_INTENT.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)

            contra_errors = [e for e in errors if e.rule_id == "semantic-oem-provenance-contradiction"]
            self.assertEqual(len(contra_errors), 1)
            self.assertIn("Physical assertion ('ballistic parachute') contradicts Level 0 OEM Ground-Truth extraction baseline in schema/extracted/.", str(contra_errors[0]))
            self.assertEqual(contra_errors[0].location, "docs/conops/MISSION_INTENT.md:4")

    def test_semantic_oem_provenance_fails_on_elevon_assertion(self):
        """Verify that asserting elevon taxonomy when OEM declares ruddervator tail surfaces fails."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "flight_manual.md"), "w", encoding="utf-8") as f:
                f.write("# Flight Manual\n\nThe airframe utilizes V-tail ruddervator control surfaces.\n")

            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/flight_manual.md -->

Pitch and yaw control are governed by symmetrical elevon actuators.
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)

            contra_errors = [e for e in errors if e.rule_id == "semantic-oem-provenance-contradiction"]
            self.assertEqual(len(contra_errors), 1)
            self.assertIn("Physical assertion ('symmetrical elevon", str(contra_errors[0]))
            self.assertIn("contradicts Level 0 OEM Ground-Truth extraction baseline in schema/extracted/.", str(contra_errors[0]))

    def test_semantic_oem_provenance_fails_on_inverted_rs485_opcodes(self):
        """Verify that asserting inverted RS-485 opcodes (0x10 PBIT or 0x11 Exchange) when OEM defines 0x11 PBIT and 0x10 Exchange fails."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "esad_icd.md"), "w", encoding="utf-8") as f:
                f.write("# ESAD ICD\n\n- Opcode 0x11: PBIT\n- Opcode 0x10: Exchange\n")

            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/esad_icd.md -->

The ESAD interface executes Opcode 0x10 PBIT on startup.
Next, it performs Opcode 0x11 Exchange for session key agreement.
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)

            contra_errors = [e for e in errors if e.rule_id == "semantic-oem-provenance-contradiction"]
            self.assertEqual(len(contra_errors), 2)
            self.assertIn("Opcode 0x10 PBIT", str(contra_errors[0]))
            self.assertIn("Opcode 0x11 Exchange", str(contra_errors[1]))

    def test_semantic_oem_provenance_passes_when_adhering_to_oem(self):
        """Verify that concept document adhering 100% to OEM extraction baseline passes with zero errors."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "oem_baseline.md"), "w", encoding="utf-8") as f:
                f.write("""# OEM Baseline
| Property | Value |
|---|---|
| system_mass | 1800.0 kg |
| Recovery system | None |

- Control surfaces: Ruddervator configuration
- Opcode 0x11: PBIT
- Opcode 0x10: Exchange
""")

            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/oem_baseline.md -->

The vehicle has a system_mass = 1800.0 kg for target mission operations.
Flight control is maintained via ruddervator surfaces on the empennage.
No parachute is installed; safe recovery relies on conventional landing.
The ESAD bus executes Opcode 0x11 for PBIT and Opcode 0x10 for Exchange.
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)

            contra_errors = [e for e in errors if e.rule_id == "semantic-oem-provenance-contradiction"]
            self.assertEqual(contra_errors, [])
            self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()

