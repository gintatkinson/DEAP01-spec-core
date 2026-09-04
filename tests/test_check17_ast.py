"""
Check 17 Structural Table-Aware AST Validator Suite.
/// Realises: [Feat-070/Check17TableAwareASTValidator]

Drives the hardening of Check 17 (scripts/verify_downstream_baseline.py) from
shallow global regex keyword matching to a structural, table-aware AST validator
with dynamic Cartesian product set equality against the authoritative SysML v2
model (schema/*.sysml or .pipeline/schema.sysml).

All fixtures use domain-neutral identifiers only (ControllerA, ControllerB,
Action01..Action21); no domain-specific concepts are hardcoded anywhere.
"""
import os
import sys
import unittest

import pytest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

from scripts.verify_downstream_baseline import (  # noqa: E402  (sys.path setup above)
    check_safety_integrity_and_sora_completeness,
    MarkdownTableASTParser,
    CartesianProductValidator,
)

# Canonical STPA guide words (methodology constants, domain-independent).
GUIDE_WORD_CELL_TEXT = {
    "GW-1": "Not providing causes hazard",
    "GW-2": "Providing causes hazard",
    "GW-3": "Providing too early, too late, or out of order",
    "GW-4": "Stopped too soon or applied too long",
}


def build_sysml_model(controller_action_counts):
    """Build a neutral SysML v2 model text declaring controllers with action defs.

    Actions are numbered globally Action01..ActionNN across the given controllers.
    """
    lines = ["package NeutralSystem {"]
    idx = 1
    for controller, count in controller_action_counts.items():
        lines.append(f"    part def {controller} {{")
        for _ in range(count):
            lines.append(f"        action def Action{idx:02d};")
            idx += 1
        lines.append("    }")
    lines.append("}")
    return "\n".join(lines) + "\n"


def build_uca_table(combos):
    """Build a structural UCA markdown table for the given (action, guide_word_id) combos."""
    header = (
        "| UCA ID | Controller | Control Action | Guide Word | Hazard Reference | Loss Reference | Safety Constraint |\n"
        "| :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
    )
    rows = []
    for number, (action, gw_id) in enumerate(combos, start=1):
        rows.append(
            f"| UCA-{number:02d} | ControllerA | {action} | {GUIDE_WORD_CELL_TEXT[gw_id]} "
            f"| H-1 | L-1 | SC-{number:02d}: System shall prevent hazardous command for {action}. |"
        )
    return header + "\n".join(rows) + "\n"


def build_oso_table(oso_ids):
    """Build a structural SORA OSO traceability table for the given OSO ids."""
    header = (
        "| OSO ID | Robustness Level | Compliance Justification | Mitigation Reference |\n"
        "| :--- | :--- | :--- | :--- |\n"
    )
    rows = [
        f"| OSO-{oso_id:02d} | Robust | Justification via containment architecture for objective {oso_id:02d}. | M-{oso_id:02d}. |"
        for oso_id in oso_ids
    ]
    return header + "\n".join(rows) + "\n"


_PROOF_PARTS = {
    1: "**Part 1 — Proposition / Theorem Statement**: Formal bound on the operating envelope invariant.",
    2: "**Part 2 — Operational Assumptions & Domain Bounds**: Bounds on sensor noise and actuation latency.",
    3: "**Part 3 — Invariant / Barrier Function Definition**: Barrier certificate must remain non-positive.",
    4: "**Part 4 — Analytical / Inductive Derivation**: Stepwise derivation demonstrating invariant preservation.",
    5: "**Part 5 — Formal Conclusion & Q.E.D.**: Stability conclusion under stated conservatism.",
}


def build_stpa_document(uca_combos=None, oso_ids=None, proof_part_numbers=(1, 2, 3, 4, 5), fmeca_row_count=16):
    """Build a full 8-pillar safety matrix document (neutral identifiers)."""
    if uca_combos is None:
        uca_combos = []
    if oso_ids is None:
        oso_ids = list(range(1, 25))
    fmeca_rows = "\n".join(
        f"| FM-{i:02d} | Subsystem-{i:02d} | Failure Mode {i:02d} | Local Effect {i:02d} "
        f"| System Effect {i:02d} | 4 | 2 | 2 | 16 | Redundant Channel {i:02d} |"
        for i in range(1, fmeca_row_count + 1)
    )
    proof_lines = []
    if proof_part_numbers:
        proof_lines = ["### Theorem THM-01: Safe Operating Envelope Invariance"]
        proof_lines.extend(_PROOF_PARTS[n] for n in sorted(proof_part_numbers))

    return rf"""# STPA Safety Analysis, FMECA Matrix & SORA SAIL Assessment

> **Primary Commercial Toolchain Integration Context:** MATLAB / Simulink / Stateflow / Embedded Coder
> **Safety Standards:** JARUS SORA v2.5 | ASTM F3269-17 RTA | RTCA DO-365B

---

## 1. System Losses (**L-1..N**)

- **L-1**: Loss of primary function during operation.
- **L-2**: Loss of containment of the operating envelope.

---

## 2. System Hazards (**H-1..N**)

- **H-1**: System enters hazardous state outside the defined envelope.
- **H-2**: Untimely actuation of a control channel.

---

## 3. Hierarchical Control Structure Topology

The control structure consists of ControllerA, ControllerB, the Actuator Channel, and the Sensor Channel.

---

## 4. Unsafe Control Actions (**UCA-1..N**)

Systematic identification across 4 STPA guide words / failure mode categories.

{build_uca_table(uca_combos)}

---

## 5. Loss Scenarios (**LS-1..N**) & Causal Factors

- **LS-1**: Sensor channel degradation leads to stale state estimation (**H-1**, **L-1**).
- **LS-2**: Actuator channel packet loss delays control transitions.

---

## 6. Formal Safety Constraints (**SC-1..N**)

- **SC-1**: The system shall remain within the defined operating envelope under all conditions.
- **SC-2**: The assurance monitor shall switch to a certified safe recovery state promptly upon barrier violation.

---

## 7. FMECA Criticality Matrix

| Failure ID | Component / Subsystem | Failure Mode | Local Effect | System Effect | S | O | D | RPN | Mitigating Design Control |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
{fmeca_rows}

---

## 8. SORA SAIL Risk Mitigations & OSO Traceability Table

- **Ground Risk Class (GRC):** Final GRC = 4 (Initial GRC = 5, mitigations applied).
- **Air Risk Class (ARC):** Final ARC-c.
- **Specific Assurance and Integrity Level (SAIL):** SAIL III.

### Operational Safety Objectives

{build_oso_table(oso_ids)}

---

## 9. ASTM F3269-17 Run-Time Assurance (RTA) & Commercial Toolchain Architecture

The safety net monitor architecture complies with **ASTM F3269-17** Run-Time Assurance (RTA).
Formal invariant proofs and recovery supervisors are synthesized directly into
**MATLAB / Simulink / Stateflow / Embedded Coder** and verified with Simulink Design Verifier (SLDV).

---

## 10. Formal Safety Theorems

{chr(10).join(proof_lines)}
"""


def write_downstream_project(tmp_path, model_text, document_text):
    """Write a downstream project tree: schema/*.sysml model + docs/safety/STPA_MATRIX.md."""
    schema_dir = tmp_path / "schema"
    schema_dir.mkdir(parents=True, exist_ok=True)
    (schema_dir / "model.sysml").write_text(model_text, encoding="utf-8")

    safety_dir = tmp_path / "docs" / "safety"
    safety_dir.mkdir(parents=True, exist_ok=True)
    (safety_dir / "STPA_MATRIX.md").write_text(document_text, encoding="utf-8")
    return tmp_path


def _run_check17(tmp_path, model_text, document_text):
    write_downstream_project(tmp_path, model_text, document_text)
    check_safety_integrity_and_sora_completeness(str(tmp_path))


def test_check17_rejects_truncated_uca_cartesian_with_model(tmp_path, capsys):
    """A SysML model declaring 21 control actions plus a 16-row UCA table must be rejected.

    The mandatory Cartesian space is 21 actions x 4 guide words = 84 UCAs; a 16-row
    table is missing 68 permutations and Check 17 must name them.
    """
    model = build_sysml_model({"ControllerA": 10, "ControllerB": 11})
    combos = [(f"Action{number:02d}", gw) for number in range(1, 5) for gw in GUIDE_WORD_CELL_TEXT]
    doc = build_stpa_document(uca_combos=combos)

    with pytest.raises(SystemExit) as exc_info:
        _run_check17(tmp_path, model, doc)
    assert exc_info.value.code == 1

    captured = capsys.readouterr().err
    assert "expected 84" in captured, f"Cardinality error naming 84 expected permutations missing: {captured}"
    assert "Action05" in captured, f"Error did not name missing permutations: {captured}"


def test_check17_accepts_complete_cartesian_matrix(tmp_path):
    """A complete 4-action x 4-guide-word = 16-row Cartesian matrix must be accepted."""
    model = build_sysml_model({"ControllerA": 4})
    combos = [(f"Action{number:02d}", gw) for number in range(1, 5) for gw in GUIDE_WORD_CELL_TEXT]
    doc = build_stpa_document(uca_combos=combos)

    _run_check17(tmp_path, model, doc)


def test_check17_rejects_missing_guideword(tmp_path, capsys):
    """A matrix covering only 3 of 4 guide words for every action must be rejected."""
    model = build_sysml_model({"ControllerA": 4})
    combos = [
        (f"Action{number:02d}", gw)
        for number in range(1, 5)
        for gw in ("GW-1", "GW-2", "GW-3")
    ]
    doc = build_stpa_document(uca_combos=combos)

    with pytest.raises(SystemExit) as exc_info:
        _run_check17(tmp_path, model, doc)
    assert exc_info.value.code == 1

    captured = capsys.readouterr().err
    assert "GW-4" in captured, f"Error did not name the missing guide word permutations: {captured}"
    assert "Action01" in captured, f"Error did not name the affected action: {captured}"


def test_check17_rejects_oso_table_missing_oso_23_24(tmp_path, capsys):
    """A structural OSO table missing OSO-23/OSO-24 must be rejected even when prose mentions them.

    The prose line is present so shuffle-free regex scanning alone (legacy behavior)
    would accept the document; the structural table AST must still reject it.
    """
    model = build_sysml_model({"ControllerA": 4})
    combos = [(f"Action{number:02d}", gw) for number in range(1, 5) for gw in GUIDE_WORD_CELL_TEXT]
    doc = build_stpa_document(uca_combos=combos, oso_ids=list(range(1, 23)))
    doc += "\nTraceability references span the OSO-23 and OSO-24 objective categories.\n"

    with pytest.raises(SystemExit) as exc_info:
        _run_check17(tmp_path, model, doc)
    assert exc_info.value.code == 1

    captured = capsys.readouterr().err
    assert "OSO-23" in captured and "OSO-24" in captured, (
        f"Error did not name missing SORA Operational Safety Objectives: {captured}"
    )


def test_check17_rejects_proof_block_missing_derivation(tmp_path, capsys):
    """A proof block missing Part 4 (Analytical / Inductive Derivation) must be rejected."""
    model = build_sysml_model({"ControllerA": 4})
    combos = [(f"Action{number:02d}", gw) for number in range(1, 5) for gw in GUIDE_WORD_CELL_TEXT]
    doc = build_stpa_document(uca_combos=combos, proof_part_numbers=(1, 2, 3, 5))

    with pytest.raises(SystemExit) as exc_info:
        _run_check17(tmp_path, model, doc)
    assert exc_info.value.code == 1

    captured = capsys.readouterr().err
    assert "THM-01" in captured, f"Error did not name the malformed theorem: {captured}"
    assert "Missing Part 4 Derivation" in captured, f"Error did not name the missing derivation: {captured}"


def write_multifile_downstream_project(tmp_path, schema_files: dict, document_text: str):
    """Write a downstream project tree with multiple schema/*.sysml files + docs/safety/STPA_MATRIX.md."""
    schema_dir = tmp_path / "schema"
    schema_dir.mkdir(parents=True, exist_ok=True)
    for filename, content in schema_files.items():
        (schema_dir / filename).write_text(content, encoding="utf-8")

    safety_dir = tmp_path / "docs" / "safety"
    safety_dir.mkdir(parents=True, exist_ok=True)
    (safety_dir / "STPA_MATRIX.md").write_text(document_text, encoding="utf-8")
    return tmp_path


def _run_check17_multifile(tmp_path, schema_files: dict, document_text: str):
    write_multifile_downstream_project(tmp_path, schema_files, document_text)
    check_safety_integrity_and_sora_completeness(str(tmp_path))


def test_check17_rejects_truncated_uca_cartesian_with_multifile_schema(tmp_path, capsys):
    """A modular SysML schema split across 01_types.sysml (no actions) and 02_actions.sysml (21 actions)
    plus a 16-row UCA table must be rejected.

    The mandatory Cartesian space is 21 actions x 4 guide words = 84 UCAs; a 16-row
    table is missing 68 permutations and Check 17 must aggregate all schema files to detect and name them.
    """
    types_sysml = "package NeutralTypes {\n    part def SignalType;\n}\n"
    actions_sysml = build_sysml_model({"ControllerA": 10, "ControllerB": 11})
    schema_files = {
        "01_types.sysml": types_sysml,
        "02_actions.sysml": actions_sysml,
    }
    combos = [(f"Action{number:02d}", gw) for number in range(1, 5) for gw in GUIDE_WORD_CELL_TEXT]
    doc = build_stpa_document(uca_combos=combos)

    with pytest.raises(SystemExit) as exc_info:
        _run_check17_multifile(tmp_path, schema_files, doc)
    assert exc_info.value.code == 1

    captured = capsys.readouterr().err
    assert "expected 84" in captured, f"Cardinality error naming 84 expected permutations missing: {captured}"
    assert "Action05" in captured, f"Error did not name missing permutations: {captured}"


def test_check17_accepts_complete_cartesian_matrix_with_multifile_schema(tmp_path):
    """A complete 4-action x 4-guide-word = 16-row Cartesian matrix with modular schema must be accepted."""
    types_sysml = "package NeutralTypes {\n    part def SignalType;\n}\n"
    actions_sysml = build_sysml_model({"ControllerA": 4})
    schema_files = {
        "01_types.sysml": types_sysml,
        "02_actions.sysml": actions_sysml,
    }
    combos = [(f"Action{number:02d}", gw) for number in range(1, 5) for gw in GUIDE_WORD_CELL_TEXT]
    doc = build_stpa_document(uca_combos=combos)

    _run_check17_multifile(tmp_path, schema_files, doc)


def test_check17_accepts_uca_table_with_description_column(tmp_path):
    """Check 17 must accept UCA tables having description columns alongside control action columns (#211)."""
    model = build_sysml_model({"ControllerA": 4})
    combos = [(f"Action{number:02d}", gw) for number in range(1, 5) for gw in GUIDE_WORD_CELL_TEXT]
    header = (
        "| UCA ID | Controller | Unsafe Control Action Description | Control Action | Guide Word | Hazard Reference | Loss Reference | Safety Constraint |\n"
        "| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
    )
    rows = []
    for number, (action, gw_id) in enumerate(combos, start=1):
        rows.append(
            f"| UCA-{number:02d} | ControllerA | Unsafe command issued under abnormal conditions | {action} | {GUIDE_WORD_CELL_TEXT[gw_id]} "
            f"| H-1 | L-1 | SC-{number:02d}: Safe constraint for {action}. |"
        )
    uca_table = header + "\n".join(rows) + "\n"
    doc = build_stpa_document(uca_combos=[])
    doc = doc.replace(build_uca_table([]), uca_table)
    _run_check17(tmp_path, model, doc)


class TestMarkdownTableASTParserUCAColumns(unittest.TestCase):
    """Unit and regression tests for MarkdownTableASTParser column resolution (#211)."""

    def test_parse_stpa_table_description_before_control_action(self):
        """Verify control_action is extracted from 'Control Action' when 'Unsafe Control Action Description' appears first."""
        table = (
            "| UCA ID | Controller | Unsafe Control Action Description | Control Action | Guide Word | Hazard Reference | Loss Reference | Safety Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            "| UCA-01 | ControllerA | Commands thrust when obstacle present | Action01 | Not providing causes hazard | H-1 | L-1 | SC-01 |\n"
            "| UCA-02 | ControllerA | Inhibits braking on landing roll | Action02 | Providing causes hazard | H-1 | L-1 | SC-02 |\n"
        )
        rows = MarkdownTableASTParser.parse_stpa_table(table)
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[0].uca_id, "UCA-01")
        self.assertEqual(rows[0].controller, "ControllerA")
        self.assertEqual(rows[0].control_action, "Action01")
        self.assertEqual(rows[0].guide_word, "Not providing causes hazard")

        self.assertEqual(rows[1].uca_id, "UCA-02")
        self.assertEqual(rows[1].controller, "ControllerA")
        self.assertEqual(rows[1].control_action, "Action02")
        self.assertEqual(rows[1].guide_word, "Providing causes hazard")

    def test_parse_stpa_table_description_after_control_action(self):
        """Verify control_action is extracted when 'Control Action' appears before 'Unsafe Control Action Description'."""
        table = (
            "| UCA ID | Controller | Control Action | Unsafe Control Action Description | Guide Word | Hazard Reference | Loss Reference | Safety Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            "| UCA-01 | ControllerA | Action01 | Commands thrust when obstacle present | Not providing causes hazard | H-1 | L-1 | SC-01 |\n"
        )
        rows = MarkdownTableASTParser.parse_stpa_table(table)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0].uca_id, "UCA-01")
        self.assertEqual(rows[0].control_action, "Action01")

    def test_parse_stpa_table_unsafe_control_action_header(self):
        """Verify control_action is not confused by 'Unsafe Control Action' column header."""
        table = (
            "| UCA ID | Unsafe Control Action | Controller | Control Action | Guide Word | Hazard Reference | Loss Reference | Safety Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
            "| UCA-01 | High throttle command | ControllerA | Action01 | Not providing causes hazard | H-1 | L-1 | SC-01 |\n"
        )
        rows = MarkdownTableASTParser.parse_stpa_table(table)
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0].uca_id, "UCA-01")
        self.assertEqual(rows[0].control_action, "Action01")

    def test_cartesian_product_validation_with_description_column(self):
        """Verify CartesianProductValidator passes on table with description column."""
        combos = [(f"Action{number:02d}", gw) for number in range(1, 3) for gw in GUIDE_WORD_CELL_TEXT]
        header = (
            "| UCA ID | Controller | Unsafe Control Action Description | Control Action | Guide Word | Hazard Reference | Loss Reference | Safety Constraint |\n"
            "| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n"
        )
        rows = []
        for number, (action, gw_id) in enumerate(combos, start=1):
            rows.append(
                f"| UCA-{number:02d} | ControllerA | Descriptive context for {action} | {action} | {GUIDE_WORD_CELL_TEXT[gw_id]} "
                f"| H-1 | L-1 | SC-{number:02d} |"
            )
        table = header + "\n".join(rows) + "\n"
        stpa_rows = MarkdownTableASTParser.parse_stpa_table(table)
        expected_actions = ["Action01", "Action02"]
        report = CartesianProductValidator.verify_cartesian_completeness(stpa_rows, expected_actions)
        self.assertTrue(report.is_conforming)
        self.assertEqual(len(report.missing_permutations), 0)
        self.assertEqual(report.total_uca_rows, 8)
        self.assertEqual(report.expected_uca_rows, 8)


if __name__ == "__main__":
    unittest.main()


