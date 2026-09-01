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
    TypedASTNode,
    ASTMetamodelGraphComparator,
    _normalize_identifier,
)


class TestConceptProvenanceASTValidator(unittest.TestCase):
    """Test suite for Issue #65: Typed AST Metamodel Graph Comparison and Section Isolation."""

    def test_schema_ast_extraction_sysml_and_markdown(self):
        """Verify dynamic Typed AST extraction from SysML models and Level 0 OEM schema markdown."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            os.makedirs(extracted_dir, exist_ok=True)

            # Abstract SysML v2 schema with PartDef, attributes, and ports
            sysml_content = """
            package PropulsionSystemSchema {
                part def EngineAssembly {
                    attribute dry_mass : Real = 450.0 [kg];
                    attribute max_thrust : Real = 12000.0 [N];
                    attribute bypass_ratio : Real = 5.2;
                }
                enum def FuelType { Kerosene, Hydrogen, Methane }
                port def FuelInletPort;
            }
            """
            with open(os.path.join(schema_dir, "engine.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            # Abstract Level 0 OEM extraction markdown with properties and opcodes
            md_content = """# OEM Interface Control Document

| Property | Value |
| :--- | :--- |
| dry_mass | 450.0 kg |
| Auxiliary Thruster | None |
| Guidance Mode | Inertial |

## Protocol Opcodes
- Opcode 0x21: TELEMETRY_SYNC
- Opcode 0x22: COMMAND_ACK
"""
            with open(os.path.join(extracted_dir, "icd.md"), "w", encoding="utf-8") as f:
                f.write(md_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            gt_graph = validator.extract_ground_truth_graph(repo)

            self.assertIsInstance(gt_graph, TypedASTNode)
            self.assertEqual(gt_graph.node_type, "Root")

            # Verify extracted attributes
            attr_nodes = [c for c in gt_graph.children if c.node_type == "Attribute"]
            attr_map = {c.name: c.value for c in attr_nodes}
            self.assertIn("dry_mass", attr_map)
            self.assertEqual(attr_map["dry_mass"], 450.0)
            self.assertIn("max_thrust", attr_map)
            self.assertEqual(attr_map["max_thrust"], 12000.0)

            # Verify extracted negative / prohibition property
            prop_nodes = [c for c in gt_graph.children if c.node_type == "Property"]
            neg_props = [p for p in prop_nodes if p.properties.get("enabled") is False or p.value in ("none", "no", "false", "n/a")]
            neg_names = [p.properties.get("normalized_name", _normalize_identifier(p.name)) for p in neg_props]
            self.assertIn("auxiliarythruster", neg_names)

            # Verify extracted mappings
            mapping_nodes = [c for c in gt_graph.children if c.node_type == "Mapping"]
            opcode_map = {m.properties.get("key"): m.properties.get("target") for m in mapping_nodes}
            self.assertEqual(opcode_map.get("0x21"), "TELEMETRY_SYNC")
            self.assertEqual(opcode_map.get("0x22"), "COMMAND_ACK")

    def test_mcda_trade_off_tables_analyzing_rejected_options_not_flagged(self):
        """
        Verify AST section isolation: MCDA trade-off tables and trade study sections
        analyzing rejected candidate options are NOT flagged as provenance contradictions.
        """
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            # Level 0 OEM Ground Truth: Auxiliary Thruster is None, Guidance Mode is Inertial
            with open(os.path.join(extracted_dir, "baseline_oem.md"), "w", encoding="utf-8") as f:
                f.write("""# OEM Baseline
| Property | Value |
|---|---|
| system_mass | 1800.0 kg |
| Auxiliary Thruster | None |
| Guidance Mode | Inertial |
""")

            # ConOps contains an MCDA Trade Study evaluating Alternative A (Auxiliary Thruster) vs Baseline,
            # where Alternative A is explicitly marked Rejected / Discarded.
            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/baseline_oem.md -->

## 1. System Overview
The vehicle operates with system_mass = 1800.0 kg and utilizes standard Inertial guidance.

## 2. Multi-Criteria Decision Analysis (MCDA) Trade Study: Secondary Propulsion
The engineering team conducted a trade study regarding secondary thruster options:

| Option / Candidate | Configuration | Mass Penalty | Power Draw | Decision | Rationale |
| :--- | :--- | :--- | :--- | :--- | :--- |
| Option A | Auxiliary Thruster System | +45 kg | 2.4 kW | REJECTED | Exceeds mass and thermal budget limits |
| Option B (Selected) | Cold Gas Attitude Control | +8 kg | 0.2 kW | ACCEPTED | Complies with baseline OEM envelope |

### Alternative Analysis Notes
- Candidate Option A (Auxiliary Thruster) was thoroughly investigated during Phase 1 conceptual architecture.
- Although Auxiliary Thrusters provide higher angular acceleration, the option was discarded due to weight constraints.

## 3. Normative Baseline Architecture
The baseline architecture strictly omits auxiliary thrusters and relies exclusively on cold gas attitude control.
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            findings = validator.validate(repo)

            # Because Option A is in an MCDA Trade Study section and marked REJECTED, zero contradiction findings should be emitted!
            contra_findings = [f for f in findings if f.rule_id == "semantic-oem-provenance-contradiction"]
            self.assertEqual(contra_findings, [], f"Expected zero contradiction findings, got: {contra_findings}")
            self.assertEqual(findings, [])

    def test_normative_contradiction_is_flagged_when_asserted_as_baseline(self):
        """Verify that when a prohibited or contradicting subsystem is asserted in normative sections, it IS flagged."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            extracted_dir = os.path.join(schema_dir, "extracted")
            conops_dir = os.path.join(tmpdir, "docs", "conops")
            os.makedirs(extracted_dir, exist_ok=True)
            os.makedirs(conops_dir, exist_ok=True)

            with open(os.path.join(extracted_dir, "baseline_oem.md"), "w", encoding="utf-8") as f:
                f.write("""# OEM Baseline
| Property | Value |
|---|---|
| system_mass | 1800.0 kg |
| Auxiliary Thruster | None |
| Guidance Mode | Inertial |
""")

            # Normative section asserts active auxiliary thruster
            conops_content = """# Concept of Operations
<!-- Source: schema/extracted/baseline_oem.md -->

## 1. System Baseline Configuration
The active flight configuration incorporates an auxiliary thruster for precision maneuvering.
Guidance is performed via Optical mode.
"""
            with open(os.path.join(conops_dir, "CONOPS.md"), "w", encoding="utf-8") as f:
                f.write(conops_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            validator = ConceptProvenanceValidator()
            findings = validator.validate(repo)

            contra_findings = [f for f in findings if f.rule_id == "semantic-oem-provenance-contradiction"]
            self.assertTrue(len(contra_findings) >= 1)

    def test_pure_schema_driven_graph_comparison_zero_hardcoded_domain_strings(self):
        """
        Verify pure schema-driven graph comparison with arbitrary domain concepts:
        Validates that comparison operates purely on AST Metamodel graphs without domain string hardcoding.
        """
        comparator = ASTMetamodelGraphComparator()

        # Ground truth graph with dynamic properties and mappings
        gt_root = TypedASTNode(node_id="root_gt", node_type="Root", name="GroundTruth")
        gt_root.children.append(TypedASTNode(
            node_id="attr_1",
            node_type="Attribute",
            name="coolant_flow_rate",
            value=250.0,
            unit="L/min",
            source_file="schema/extracted/cryo.md",
            line_number=5
        ))
        gt_root.children.append(TypedASTNode(
            node_id="prop_1",
            node_type="Property",
            name="Quantum Flux Damper",
            properties={"normalized_name": "quantumfluxdamper", "enabled": False},
            value="None",
            source_file="schema/extracted/cryo.md",
            line_number=8
        ))
        gt_root.children.append(TypedASTNode(
            node_id="map_1",
            node_type="Mapping",
            name="OpcodeMapping",
            properties={"domain": "opcode", "key": "0x55", "target": "INITIALIZE_CRYO"},
            source_file="schema/extracted/cryo.md",
            line_number=12
        ))

        # Candidate normative graph with contradictory mapping and prohibited property
        cand_root = TypedASTNode(node_id="root_cand", node_type="Root", name="CandidateConOps")
        cand_root.children.append(TypedASTNode(
            node_id="cand_prop_1",
            node_type="Property",
            name="Quantum Flux Damper",
            properties={"normalized_name": "quantumfluxdamper", "enabled": True},
            value="Active",
            source_file="docs/conops/cryo_ops.md",
            line_number=15,
            is_normative=True
        ))
        cand_root.children.append(TypedASTNode(
            node_id="cand_map_1",
            node_type="Mapping",
            name="OpcodeMapping",
            properties={"domain": "opcode", "key": "0x55", "target": "PURGE_CHAMBER"},
            source_file="docs/conops/cryo_ops.md",
            line_number=22,
            is_normative=True
        ))

        mismatches = comparator.compare_graphs(gt_root, cand_root)
        self.assertEqual(len(mismatches), 2)
        self.assertTrue(any("quantumfluxdamper" in m.lower() or "quantum flux damper" in m.lower() for m in mismatches))
        self.assertTrue(any("0x55" in m and "INITIALIZE_CRYO" in m for m in mismatches))


if __name__ == "__main__":
    unittest.main()
