"""
Check 17 STPA UCA Table Contract & Parser Suite.
/// Realises: [Feat-070/Check17TableAwareASTValidator]

Regression suite verifying flexible UCA table column header matching,
robust 4-guide-word classification (including synonyms, classes, and compound
class attributions), and Cartesian completeness verification for 60-row
expanded UCA matrices (15 control actions x 4 guide words = 60 permutations).

All fixtures use domain-neutral identifiers (ControllerA, Action01..Action15).
"""
import os
import sys
import unittest
import pytest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

from scripts.verify_downstream_baseline import (
    check_safety_integrity_and_sora_completeness,
    MarkdownTableASTParser,
    CartesianProductValidator,
    classify_uca_guide_word,
    classify_uca_guide_words,
    validate_safety_matrix_ast,
)


class TestSTPAGuideWordClassification(unittest.TestCase):
    """Unit tests for classify_uca_guide_word and classify_uca_guide_words."""

    def test_canonical_guide_words(self):
        """Verify canonical STPA guide word phrasings map to GW-1..GW-4."""
        cases = [
            ("Not providing causes hazard", "GW-1"),
            ("Providing causes hazard", "GW-2"),
            ("Providing too early, too late, or out of order", "GW-3"),
            ("Stopped too soon or applied too long", "GW-4"),
            ("Not providing", "GW-1"),
            ("Providing", "GW-2"),
            ("Too early / Too late / Out of order", "GW-3"),
            ("Stopped too soon / Applied too long", "GW-4"),
        ]
        for text, expected_gw in cases:
            res = classify_uca_guide_word(text)
            self.assertIsNotNone(res, f"Failed to classify: {text}")
            self.assertEqual(res[0], expected_gw, f"Mismatch for '{text}': got {res[0]}, expected {expected_gw}")

    def test_synonyms_and_class_attributions(self):
        """Verify synonyms and class attribution codes map to the correct guide words."""
        cases = [
            # GW-1: Not providing / Omission
            ("Class a", "GW-1"),
            ("Class A", "GW-1"),
            ("Class a — not providing", "GW-1"),
            ("Omission", "GW-1"),
            ("Withheld", "GW-1"),
            ("GW-1", "GW-1"),
            # GW-2: Providing / Commission
            ("Class b", "GW-2"),
            ("Class B", "GW-2"),
            ("Class b — providing causes hazard", "GW-2"),
            ("Commission", "GW-2"),
            ("Incorrectly provided", "GW-2"),
            ("Unintended provision", "GW-2"),
            ("GW-2", "GW-2"),
            # GW-3: Too early, too late, out of order
            ("Class c", "GW-3"),
            ("Class C", "GW-3"),
            ("Class c — too late", "GW-3"),
            ("Too early", "GW-3"),
            ("Too late", "GW-3"),
            ("Out of order", "GW-3"),
            ("Early/late", "GW-3"),
            ("Timing", "GW-3"),
            ("GW-3", "GW-3"),
            # GW-4: Stopped too soon, applied too long
            ("Class d", "GW-4"),
            ("Class D", "GW-4"),
            ("Class d — stopped too soon", "GW-4"),
            ("Stopped too soon", "GW-4"),
            ("Applied too long", "GW-4"),
            ("Stopped early", "GW-4"),
            ("Duration", "GW-4"),
            ("Too soon", "GW-4"),
            ("GW-4", "GW-4"),
        ]
        for text, expected_gw in cases:
            res = classify_uca_guide_word(text)
            self.assertIsNotNone(res, f"Failed to classify: {text}")
            self.assertEqual(res[0], expected_gw, f"Mismatch for '{text}': got {res[0]}, expected {expected_gw}")

    def test_compound_class_attribution_expansion(self):
        """Verify compound class attribution cells expand into multiple guide words."""
        compound = "Class a — not providing; Class c — too late"
        matches = classify_uca_guide_words(compound)
        self.assertEqual(len(matches), 2)
        gw_ids = [m[0] for m in matches]
        self.assertIn("GW-1", gw_ids)
        self.assertIn("GW-3", gw_ids)

        all_four = "Class a: not providing, Class b: commission, Class c: timing, Class d: duration"
        matches_all = classify_uca_guide_words(all_four)
        self.assertEqual(len(matches_all), 4)
        gw_all_ids = [m[0] for m in matches_all]
        self.assertEqual(gw_all_ids, ["GW-1", "GW-3", "GW-4", "GW-2"])


class TestSTPATableParserColumnVariations(unittest.TestCase):
    """Unit tests for MarkdownTableASTParser column resolution on diverse header variations."""

    def test_generator_ssot_header(self):
        """Parse table with '| UCA ID | Control action (SSOT) | Class attribution | Linked hazards |'."""
        table = (
            "| UCA ID | Controller | Control action (SSOT) | Class attribution | Linked hazards | Loss Reference | Safety Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            "| UCA-001 | ControllerA | Action01 | Class a — not providing causes hazard | H-1 | L-1 | SC-001 |\n"
            "| UCA-002 | ControllerA | Action01 | Class b — providing causes hazard | H-1 | L-1 | SC-002 |\n"
        )
        rows = MarkdownTableASTParser.parse_stpa_table(table)
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0].uca_id, "UCA-001")
        self.assertEqual(rows[0].controller, "ControllerA")
        self.assertEqual(rows[0].control_action, "Action01")
        self.assertEqual(rows[0].guide_word, "Class a — not providing causes hazard")
        self.assertEqual(rows[0].hazard_ref, "H-1")

    def test_short_headers_action_failure_mode(self):
        """Parse table with '| ID | Controller | Action | Failure Mode | Hazard | Loss | Constraint |'."""
        table = (
            "| ID | Controller | Action | Failure Mode | Hazard | Loss | Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            "| UCA-01 | ControllerA | Action01 | Omission | H-1 | L-1 | SC-01 |\n"
            "| UCA-02 | ControllerA | Action01 | Commission | H-1 | L-1 | SC-02 |\n"
        )
        rows = MarkdownTableASTParser.parse_stpa_table(table)
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0].uca_id, "UCA-01")
        self.assertEqual(rows[0].control_action, "Action01")
        self.assertEqual(rows[0].guide_word, "Omission")
        self.assertEqual(rows[0].hazard_ref, "H-1")

    def test_command_and_type_headers(self):
        """Parse table with '| Identifier | Controller | Command | Type | Hazards | Loss Ref | Safety Constraint |'."""
        table = (
            "| Identifier | Controller | Command | Type | Hazards | Loss Ref | Safety Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- |\n"
            "| UCA-01 | ControllerA | Action01 | Not providing | H-1 | L-1 | SC-01 |\n"
        )
        rows = MarkdownTableASTParser.parse_stpa_table(table)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0].uca_id, "UCA-01")
        self.assertEqual(rows[0].control_action, "Action01")
        self.assertEqual(rows[0].guide_word, "Not providing")
        self.assertEqual(rows[0].hazard_ref, "H-1")

    def test_guide_word_hyphen_and_mode_headers(self):
        """Parse table with '| UCA | Subsystem | Action | Guide-Word | Linked Hazards | Loss | Constraint |'."""
        table = (
            "| UCA | Subsystem | Action | Guide-Word | Linked Hazards | Loss | Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            "| UCA-01 | SubsystemA | Action01 | Timing | H-1 | L-1 | SC-01 |\n"
        )
        rows = MarkdownTableASTParser.parse_stpa_table(table)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0].uca_id, "UCA-01")
        self.assertEqual(rows[0].controller, "SubsystemA")
        self.assertEqual(rows[0].control_action, "Action01")
        self.assertEqual(rows[0].guide_word, "Timing")


class TestCheck17CartesianCompleteness60Rows(unittest.TestCase):
    """Cartesian completeness tests for 15 actions x 4 guide words = 60 UCA rows."""

    def _build_60_row_table(self, header_style="standard"):
        actions = [f"Action{i:02d}" for i in range(1, 16)]
        gw_phrasings = {
            "GW-1": "Not providing causes hazard",
            "GW-2": "Providing causes hazard",
            "GW-3": "Providing too early, too late, or out of order",
            "GW-4": "Stopped too soon or applied too long",
        }
        if header_style == "ssot":
            header = (
                "| UCA ID | Controller | Control action (SSOT) | Class attribution | Linked hazards | Loss Reference | Safety Constraint |\n"
                "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            )
            gw_phrasings = {
                "GW-1": "Class a — not providing causes hazard",
                "GW-2": "Class b — providing causes hazard",
                "GW-3": "Class c — too late",
                "GW-4": "Class d — stopped too soon",
            }
        elif header_style == "short":
            header = (
                "| ID | Controller | Action | Failure Mode | Hazard | Loss | Constraint |\n"
                "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            )
            gw_phrasings = {
                "GW-1": "Omission",
                "GW-2": "Commission",
                "GW-3": "Timing",
                "GW-4": "Duration",
            }
        else:
            header = (
                "| UCA ID | Controller | Control Action | Guide Word | Hazard Reference | Loss Reference | Safety Constraint |\n"
                "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            )

        rows = []
        counter = 1
        for action in actions:
            for gw_id in ("GW-1", "GW-2", "GW-3", "GW-4"):
                rows.append(
                    f"| UCA-{counter:03d} | ControllerA | {action} | {gw_phrasings[gw_id]} | H-1 | L-1 | SC-{counter:03d} |"
                )
                counter += 1

        return header + "\n".join(rows) + "\n"

    def _build_sysml_15_actions(self):
        lines = [
            "package NeutralSystem {",
            "    part def ControllerA {",
        ]
        for i in range(1, 16):
            lines.append(f"        action def Action{i:02d};")
        lines.append("    }")
        lines.append("}")
        return "\n".join(lines) + "\n"

    def _build_full_doc(self, uca_table):
        fmeca_rows = "\n".join(
            f"| FM-{i:02d} | Subsystem-{i:02d} | Failure Mode {i:02d} | Local Effect {i:02d} "
            f"| System Effect {i:02d} | 4 | 2 | 2 | 16 | Redundant Channel {i:02d} |"
            for i in range(1, 17)
        )
        oso_rows = "\n".join(
            f"| OSO-{i:02d} | Robust | Justification for OSO-{i:02d} | M-{i:02d} |"
            for i in range(1, 25)
        )
        return rf"""# STPA Safety Analysis, FMECA Matrix & SORA SAIL Assessment

> **Primary Commercial Toolchain Integration Context:** MATLAB / Simulink / Stateflow / Embedded Coder
> **Safety Standards:** JARUS SORA v2.5 | ASTM F3269-17 RTA | RTCA DO-365B

---

## 1. System Losses (**L-1..N**)
- **L-1**: Loss of primary function.

## 2. System Hazards (**H-1..N**)
- **H-1**: Hazard state.

## 3. Hierarchical Control Structure Topology
The control structure consists of ControllerA.

## 4. Unsafe Control Actions (**UCA-1..N**)

{uca_table}

---

## 5. Loss Scenarios (**LS-1..N**) & Causal Factors
- **LS-1**: Causal factors.

## 6. Formal Safety Constraints (**SC-1..N**)
- **SC-1**: Safety constraints.

## 7. FMECA Criticality Matrix

| Failure ID | Component / Subsystem | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
{fmeca_rows}

---

## 8. SORA SAIL Risk Mitigations & OSO Traceability Table

- **Ground Risk Class (GRC):** Final GRC = 4.
- **Air Risk Class (ARC):** Final ARC-c.
- **Specific Assurance and Integrity Level (SAIL):** SAIL III.

### Operational Safety Objectives

| OSO ID | Robustness Level | Compliance Justification | Mitigation Reference |
| :--- | :--- | :--- | :--- |
{oso_rows}

---

## 9. ASTM F3269-17 Run-Time Assurance (RTA) & Commercial Toolchain Architecture

The safety net monitor architecture complies with **ASTM F3269-17** Run-Time Assurance (RTA).
Formal invariant proofs are synthesized into **MATLAB / Simulink / Stateflow / Embedded Coder**.

---

## 10. Formal Safety Theorems

### Theorem THM-01: Safe Operating Envelope Invariance
**Part 1 — Proposition / Theorem Statement**: Safe envelope invariant.
**Part 2 — Operational Assumptions & Domain Bounds**: Bounded inputs.
**Part 3 — Invariant / Barrier Function Definition**: B(x) <= 0.
**Part 4 — Analytical / Inductive Derivation**: Preserved across all transitions.
**Part 5 — Formal Conclusion & Q.E.D.**: Envelope invariance holds.
"""

    def test_60_row_standard_table_cartesian_completeness_with_model(self):
        """Verify 60-row standard UCA table passes Cartesian completeness with 15-action SysML model."""
        uca_table = self._build_60_row_table("standard")
        doc = self._build_full_doc(uca_table)
        model = self._build_sysml_15_actions()

        stpa_rows = MarkdownTableASTParser.parse_stpa_table(doc)
        self.assertEqual(len(stpa_rows), 60)

        expected_actions = [f"Action{i:02d}" for i in range(1, 16)]
        report = CartesianProductValidator.verify_cartesian_completeness(stpa_rows, expected_actions)
        self.assertTrue(report.is_conforming)
        self.assertEqual(len(report.missing_permutations), 0)
        self.assertEqual(report.total_uca_rows, 60)
        self.assertEqual(report.expected_uca_rows, 60)

        errors, ast_report, _ = validate_safety_matrix_ast(doc, model_text=model)
        self.assertEqual(errors, [])
        self.assertTrue(ast_report.is_conforming)

    def test_60_row_ssot_header_cartesian_completeness_with_model(self):
        """Verify 60-row SSOT-header table passes Cartesian completeness with 15-action SysML model (#201)."""
        uca_table = self._build_60_row_table("ssot")
        doc = self._build_full_doc(uca_table)
        model = self._build_sysml_15_actions()

        stpa_rows = MarkdownTableASTParser.parse_stpa_table(doc)
        self.assertEqual(len(stpa_rows), 60)

        expected_actions = [f"Action{i:02d}" for i in range(1, 16)]
        report = CartesianProductValidator.verify_cartesian_completeness(stpa_rows, expected_actions)
        self.assertTrue(report.is_conforming)
        self.assertEqual(len(report.missing_permutations), 0)
        self.assertEqual(report.total_uca_rows, 60)
        self.assertEqual(report.expected_uca_rows, 60)

        errors, ast_report, _ = validate_safety_matrix_ast(doc, model_text=model)
        self.assertEqual(errors, [])
        self.assertTrue(ast_report.is_conforming)

    def test_60_row_short_header_cartesian_completeness_schemaless(self):
        """Verify 60-row short-header table passes Cartesian completeness in schema-less mode."""
        uca_table = self._build_60_row_table("short")
        doc = self._build_full_doc(uca_table)

        stpa_rows = MarkdownTableASTParser.parse_stpa_table(doc)
        self.assertEqual(len(stpa_rows), 60)

        errors, ast_report, _ = validate_safety_matrix_ast(doc, model_text=None)
        self.assertEqual(errors, [])
        self.assertTrue(ast_report.is_conforming)
        self.assertEqual(ast_report.total_uca_rows, 60)
        self.assertEqual(ast_report.expected_uca_rows, 60)

    def test_missing_guide_word_column_diagnostic(self):
        """Verify missing guide word column triggers clear diagnostic message."""
        table = (
            "| UCA ID | Controller | Control Action | Hazard Reference | Loss Reference | Safety Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- |\n"
            "| UCA-01 | ControllerA | Action01 | H-1 | L-1 | SC-01 |\n"
        )
        doc = self._build_full_doc(table)
        errors, ast_report, _ = validate_safety_matrix_ast(doc, model_text=None)
        self.assertTrue(any("guide word / failure mode column could not be resolved" in err for err in errors))

    def test_unclassifiable_guide_word_diagnostic(self):
        """Verify unclassifiable guide word triggers clear diagnostic message."""
        table = (
            "| UCA ID | Controller | Control Action | Guide Word | Hazard Reference | Loss Reference | Safety Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            "| UCA-01 | ControllerA | Action01 | UnknownRandomCategory | H-1 | L-1 | SC-01 |\n"
        )
        doc = self._build_full_doc(table)
        errors, ast_report, _ = validate_safety_matrix_ast(doc, model_text=None)
        self.assertTrue(any("unclassifiable guide word text" in err for err in errors))


if __name__ == "__main__":
    unittest.main()
