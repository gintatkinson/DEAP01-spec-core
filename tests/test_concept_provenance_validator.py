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

    def test_markdown_table_delimiters_ignored_without_phantom_contradictions(self):
        """Verify that markdown table delimiters (e.g. :---, ---:, :---:, |---|---|) do not trigger phantom contradictions."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "oem_specs.md"), "w", encoding="utf-8") as f:
                f.write("""# OEM Specs
| Property | Value | Notes |
| :--- | :---: | ---: |
| system_mass | 1800.0 kg | Nominal gross mass |
| Recovery system | None | Direct landing only |
""")

            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/oem_specs.md -->

| Subsystem | Metric | Target |
| :--- | :---: | ---: |
| Structure | system_mass | 1800.0 |
| Recovery | Method | Conventional Landing |
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_integer_pin_numbers_filtered_out_from_contradiction_extraction(self):
        """Verify that integer pin numbers and pinout tables are filtered out from value contradiction extraction."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "connector_icd.md"), "w", encoding="utf-8") as f:
                f.write("""# Connector ICD
| Pin | Signal | Function |
| :--- | :--- | :--- |
| 1 | +28VDC | Main power input |
| 2 | GND | Power return |
| 3 | RS485_TX | Differential transmit |
| 4 | RS485_RX | Differential receive |

- Pin 1: Power supply 28V
- Pin 2: Ground
""")

            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/connector_icd.md -->

## Sequence of Operations
| Step | Action |
| :--- | :--- |
| 1 | Apply bus power |
| 2 | Run initial health check |
| 3 | Synchronize telemetry |
| 4 | Ready state |

Interface wiring follows:
- Pin 1: Bus power connection
- Pin 2: Ground return
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_glossary_and_definitions_section_isolated_without_contradictions(self):
        """Verify that glossary, acronyms, and terminology definitions are isolated from value contradiction extraction."""
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

            # ConOps has a Glossary explaining what a ballistic parachute and elevons are,
            # but the normative architecture adheres strictly to OEM baseline.
            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/oem_baseline.md -->

## 1. Glossary & Terminology
| Term | Definition |
| :--- | :--- |
| Ballistic Parachute | Pyrotechnic emergency recovery system deployed in contingency |
| Elevon | Aerodynamic flight surface combining pitch and roll controls |
| PBIT | Periodic Built-In Test routine |

- **Ballistic Parachute**: Emergency deceleration mechanism.
- **Elevon**: Movable wing control surface.

## 2. Normative Architecture
The vehicle has a system_mass = 1800.0 kg.
Pitch and yaw control are governed by ruddervator surfaces on the V-tail empennage.
No parachute is installed; safe recovery relies on conventional runway landing.
The ESAD bus executes Opcode 0x11 for PBIT and Opcode 0x10 for Exchange.
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)
            self.assertEqual(errors, [])

    def test_negative_attribute_values_extracted_with_true_negative_sign(self):
        """
        Verify Issue #91: Negative attribute values with hyphens or separators are extracted
        with true negative float values and not flipped to positive values.
        """
        from parity_auditor.validators.concept_provenance_validator import TypedASTNode

        validator = ConceptProvenanceValidator()
        gt_root = TypedASTNode(node_id="root_gt", node_type="Root", name="GroundTruth")
        gt_root.children.append(TypedASTNode(
            node_id="attr_1", node_type="Attribute", name="pitch", value=-45.0, properties={"normalized_name": "pitch"}
        ))
        gt_root.children.append(TypedASTNode(
            node_id="attr_2", node_type="Attribute", name="yaw_roll", value=-165.0, properties={"normalized_name": "yawroll"}
        ))
        gt_root.children.append(TypedASTNode(
            node_id="attr_3", node_type="Attribute", name="operating_temperature_range", value=-20.0, properties={"normalized_name": "operatingtemperaturerange"}
        ))
        gt_root.children.append(TypedASTNode(
            node_id="attr_4", node_type="Attribute", name="minimum_altitude", value=-10.0, properties={"normalized_name": "minimumaltitude"}
        ))
        gt_root.children.append(TypedASTNode(
            node_id="attr_5", node_type="Attribute", name="cruise_speed", value=30.0, properties={"normalized_name": "cruisespeed"}
        ))

        test_content = """# ConOps Specification
<!-- Source: schema/extracted/specs.md -->

## Flight Characteristics
- The primary flight profile limits pitch -45 deg to +135 deg.
- Gimbal orientation constraints: yaw_roll -165 deg.
- Environmental operating temperature range -20 degC to +50 degC.
- Minimum altitude -10 m above sea level.
- Normal cruise speed - 30 m/s.
- Alternate colon format pitch: -45 deg.
- Alternate bold format **pitch**: -45 deg.
- Alternate equals format pitch = -45 deg.
- Alternate unicode minus format pitch \u221245 deg.
"""
        cand_graph = validator.extract_concept_graph(test_content, "docs/conops/conops.md", gt_root)
        attrs = [c for c in cand_graph.children if c.node_type == "Attribute"]
        attr_map = {}
        for a in attrs:
            attr_map.setdefault(a.properties.get("normalized_name", a.name.lower()), []).append(a.value)

        # Verify pitch extracted negative values
        self.assertIn("pitch", attr_map)
        for p_val in attr_map["pitch"]:
            self.assertEqual(p_val, -45.0, f"Expected -45.0 for pitch, got {p_val}")

        # Verify yaw_roll extracted -165.0
        self.assertIn("yawroll", attr_map)
        self.assertEqual(attr_map["yawroll"][0], -165.0)

        # Verify operating temperature range extracted -20.0
        self.assertIn("operatingtemperaturerange", attr_map)
        self.assertEqual(attr_map["operatingtemperaturerange"][0], -20.0)

        # Verify minimum altitude extracted -10.0
        self.assertIn("minimumaltitude", attr_map)
        self.assertEqual(attr_map["minimumaltitude"][0], -10.0)

        # Verify cruise speed prose with hyphen separator extracted +30.0
        self.assertIn("cruisespeed", attr_map)
        self.assertEqual(attr_map["cruisespeed"][0], 30.0)

    def test_end_to_end_negative_attribute_provenance_no_false_mismatch(self):
        """
        Verify that negative attribute assertions in Level 1 ConOps match Level 0 Ground Truth
        within tolerance and do not emit false parametric mismatch errors.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "oem_specs.md"), "w", encoding="utf-8") as f:
                f.write("""# OEM Specs
| Property | Value |
|---|---|
| pitch | -45.0 deg |
| yaw_roll | -165.0 deg |
| minimum_altitude | -10.0 m |
""")

            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/oem_specs.md -->

## Operational Limits
The platform envelope supports pitch -45 deg to +135 deg during maneuvers.
The sensor gimbal supports yaw_roll -165 deg maximum deflection.
The vehicle allows minimum_altitude -10 m during subterranean approach.
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            errors = validator.validate(repo)

            mismatch_errors = [e for e in errors if e.rule_id == "concept-provenance-parametric-mismatch"]
            self.assertEqual(mismatch_errors, [], f"Expected 0 parametric mismatch errors, got: {mismatch_errors}")
            self.assertEqual(errors, [])

    def test_metadata_tables_with_compound_titles_produce_zero_false_positives(self):
        """
        Verify Issue #216: Document metadata tables with compound titles (e.g. Format RS-485 Frame,
        Gate RBF3 Removal, PL40, Swarm C2 Ground Station) and metadata keys (Title, Issue ID, Type,
        Status, Parent Epic, Version) produce zero false-positive findings.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            feat_dir = os.path.join(tmpdir, "docs", "features")
            story_dir = os.path.join(tmpdir, "docs", "user-stories")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)
            os.makedirs(feat_dir, exist_ok=True)
            os.makedirs(story_dir, exist_ok=True)

            # Ground truth schema with legitimate physical parameters
            with open(os.path.join(extracted_dir, "system_specs.md"), "w", encoding="utf-8") as f:
                f.write("""# System Specifications
| Property | Value |
| :--- | :--- |
| Title | System Ground Truth Baseline Specification |
| Version | 2.0.0 |
| system_mass | 1800.0 kg |
| nominal_power | 75.5 kW |
""")

            # ConOps document with compound title in metadata table
            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/system_specs.md -->

| Property | Value |
| :--- | :--- |
| Title | Swarm C2 Ground Station Operational Architecture |
| Issue ID | #216 |
| Type | Concept Document |
| Status | Approved |
| Version | 1.0.0 |

## Operational Characteristics
The operational system operates with system_mass = 1800.0 kg and nominal_power = 75.5 kW.
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            # Feature document with RS-485 compound title in metadata table
            feat_content = """# Feature: Serial Communication
<!-- Source: schema/extracted/system_specs.md -->

| Property | Value |
| :--- | :--- |
| Title | Format RS-485 Frame Protocol |
| Issue ID | #217 |
| Type | Feature |
| Parent Epic | EPIC-01 |
| Status | Draft |
| Generation Mode | Autonomous |
| Realized AST Nodes | PortDef, PartDef |
| Source References | schema/extracted/system_specs.md |

The serial bus operates under system_mass = 1800.0 kg constraints.
"""
            with open(os.path.join(feat_dir, "feat-01.md"), "w", encoding="utf-8") as f:
                f.write(feat_content)

            # User story with RBF3 removal compound title in metadata table
            story1_content = """# User Story: Arming Gate
<!-- Source: schema/extracted/system_specs.md -->

| Property | Value |
| :--- | :--- |
| Title | Gate RBF3 Removal Interlock |
| Issue ID | #218 |
| Type | User Story |
| Parent Epic | EPIC-01 |
| Status | Ready |

The system asserts system_mass = 1800.0 kg.
"""
            with open(os.path.join(story_dir, "us-01.md"), "w", encoding="utf-8") as f:
                f.write(story1_content)

            # User story with PL40 compound title in metadata table
            story2_content = """# User Story: Payload Interface
<!-- Source: schema/extracted/system_specs.md -->

| Property | Value |
| :--- | :--- |
| Title | PL40 Payload Protocol Integration |
| Issue ID | #219 |
| Type | User Story |
| Parent Epic | EPIC-01 |
| Status | In Progress |

The payload interface consumes nominal_power = 75.5 kW.
"""
            with open(os.path.join(story_dir, "us-02.md"), "w", encoding="utf-8") as f:
                f.write(story2_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()

            # Verify ground truth does not contain Title, Version, etc. as physical parameters
            gt = validator.extract_ground_truth(repo)
            self.assertIn("system_mass", gt)
            self.assertIn("nominal_power", gt)
            self.assertNotIn("title", gt)
            self.assertNotIn("version", gt)

            # Validate the repository specifications
            errors = validator.validate(repo)
            self.assertEqual(errors, [], f"Expected 0 findings from metadata tables with compound titles, got: {errors}")

    def test_pending_arbitration_registers_isolated_without_false_contradictions(self):
        """
        Verify Issue #216: Disputed intra-SSOT entries marked with PENDING_ARBITRATION in
        schema/SSOT_INPUT_REGISTER.md are isolated and not treated as immutable ground truth.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(schema_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            # Write SSOT_INPUT_REGISTER.md with approved and pending arbitration sections
            with open(os.path.join(schema_dir, "SSOT_INPUT_REGISTER.md"), "w", encoding="utf-8") as f:
                f.write("""# Single Source of Truth Input Register

## 1. Approved Baseline Parameters
| Parameter | Value | Status |
| :--- | :--- | :--- |
| system_mass | 1800.0 kg | APPROVED |
| cruise_speed | 30.0 m/s | APPROVED |

## 2. Disputed Parameters (Pending Arbitration)
| Parameter | Candidate A | Candidate B | Arbitration Status | Notes |
| :--- | :--- | :--- | :--- | :--- |
| battery_capacity | 120.0 kWh | 90.0 kWh | PENDING_ARBITRATION | Thermal vs mass trade-off |
| payload_capacity | 50.0 kg | 75.0 kg | PENDING_ARBITRATION | Structural review required |

## 3. Disputed Recovery System Architecture
| Architecture Option | Decision |
| :--- | :--- |
| Ballistic Parachute | PENDING_ARBITRATION |

## 4. Single-Row Disputed Entry
| sensor_range | 150.0 km | PENDING_ARBITRATION |
""")

            # ConOps document citing the SSOT input register for approved parameters
            conops_content = """# Concept of Operations
<!-- Source: schema/SSOT_INPUT_REGISTER.md -->

## System Overview
The operational platform operates with system_mass = 1800.0 kg and normal cruise speed - 30 m/s.
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()

            # Verify ground truth extraction:
            # - Approved parameters are extracted
            # - Pending arbitration parameters are excluded
            gt = validator.extract_ground_truth(repo)
            self.assertIn("system_mass", gt)
            self.assertEqual(gt["system_mass"].value, 1800.0)
            self.assertIn("cruise_speed", gt)
            self.assertEqual(gt["cruise_speed"].value, 30.0)

            self.assertNotIn("battery_capacity", gt)
            self.assertNotIn("payload_capacity", gt)
            self.assertNotIn("sensor_range", gt)

            # Verify validation passes cleanly
            errors = validator.validate(repo)
            self.assertEqual(errors, [], f"Expected 0 errors for valid ConOps against SSOT input register, got: {errors}")

    def test_extract_numeric_value_robustness(self):
        """
        Verify Issue #216: _extract_numeric_value correctly parses scalar numbers with units
        and rejects compound alphanumeric identifiers, hex opcodes, and metadata.
        """
        from parity_auditor.validators.concept_provenance_validator import (
            _extract_numeric_value,
            _is_metadata_key,
        )

        # Valid scalar quantities
        self.assertEqual(_extract_numeric_value("1800.0 kg")[0], 1800.0)
        self.assertEqual(_extract_numeric_value("1800.0 kg")[1], "kg")
        self.assertEqual(_extract_numeric_value("45.0 deg")[0], 45.0)
        self.assertEqual(_extract_numeric_value("45.0 deg")[1], "deg")
        self.assertEqual(_extract_numeric_value("75.5 kW")[0], 75.5)
        self.assertEqual(_extract_numeric_value("75.5 kW")[1], "kW")
        self.assertEqual(_extract_numeric_value("-10.5 m/s")[0], -10.5)
        self.assertEqual(_extract_numeric_value("-10.5 m/s")[1], "m/s")
        self.assertEqual(_extract_numeric_value("-20 degC to +50 degC")[0], -20.0)
        self.assertEqual(_extract_numeric_value("+12.5 %")[0], 12.5)
        self.assertEqual(_extract_numeric_value("100.0 kWh")[0], 100.0)
        self.assertEqual(_extract_numeric_value("42.0 [m3]")[0], 42.0)

        # Compound alphanumeric identifiers that MUST NOT be parsed as numbers
        self.assertIsNone(_extract_numeric_value("Format RS-485 Frame"))
        self.assertIsNone(_extract_numeric_value("RS-485"))
        self.assertIsNone(_extract_numeric_value("RS485"))
        self.assertIsNone(_extract_numeric_value("Gate RBF3 Removal"))
        self.assertIsNone(_extract_numeric_value("RBF3"))
        self.assertIsNone(_extract_numeric_value("PL40"))
        self.assertIsNone(_extract_numeric_value("PL-40"))
        self.assertIsNone(_extract_numeric_value("Swarm C2 Ground Station"))
        self.assertIsNone(_extract_numeric_value("AVENGER5"))
        self.assertIsNone(_extract_numeric_value("AVENGER-5"))
        self.assertIsNone(_extract_numeric_value("AVENGER_5"))
        self.assertIsNone(_extract_numeric_value("Subsystem AVENGER5 Package"))
        self.assertIsNone(_extract_numeric_value("Opcode 0x11"))
        self.assertIsNone(_extract_numeric_value("None"))
        self.assertIsNone(_extract_numeric_value("No"))
        self.assertIsNone(_extract_numeric_value("N/A"))

        # Metadata key recognition
        self.assertTrue(_is_metadata_key("Title", "title"))
        self.assertTrue(_is_metadata_key("Issue ID", "issueid"))
        self.assertTrue(_is_metadata_key("Type", "type"))
        self.assertTrue(_is_metadata_key("Parent Epic", "parentepic"))
        self.assertTrue(_is_metadata_key("Status", "status"))
        self.assertTrue(_is_metadata_key("Generation Mode", "generationmode"))
        self.assertTrue(_is_metadata_key("Realized AST Nodes", "realizedastnodes"))
        self.assertTrue(_is_metadata_key("Source References", "sourcereferences"))
        self.assertTrue(_is_metadata_key("Version", "version"))
        self.assertFalse(_is_metadata_key("system_mass", "systemmass"))
        self.assertFalse(_is_metadata_key("cruise_speed", "cruisespeed"))


if __name__ == "__main__":
    unittest.main()



