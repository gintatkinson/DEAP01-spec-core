#!/usr/bin/env python3
"""
Unit tests for Check 20 (User Story Interactions), Check 21 (Safety Invariant & RTA Constraints),
and Check 22 (Acceptance Criteria & Test Case Bindings) Parity Matrix validation.
"""

import os
import sys
import tempfile
import json
import pytest

# Setup path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PARITY_AUDITOR_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
SPEC_ORCH_DIR = os.path.abspath(os.path.join(PARITY_AUDITOR_DIR, ".."))
PROJECT_ROOT = os.path.abspath(os.path.join(SPEC_ORCH_DIR, "..", ".."))
SPEC_SCRIPTS_DIR = os.path.join(SPEC_ORCH_DIR, "scripts")
PARITY_AUDITOR_SRC = os.path.join(PARITY_AUDITOR_DIR, "src")

for p in (SPEC_SCRIPTS_DIR, PARITY_AUDITOR_SRC, PROJECT_ROOT):
    if p not in sys.path:
        sys.path.insert(0, p)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.uml import UmlValidator
from scripts.compile_sysml import (
    parse_stpa_ucas,
    parse_fmeca_modes,
    compile_stpa_to_sysml,
    compile_stpa_to_ast
)


def create_mock_workspace(tmpdir, sysml_content, user_story_content=None, safety_doc_content=None):
    """Helper to populate a temporary workspace repository with SysML models, User Stories, and Safety docs."""
    pipeline_dir = os.path.join(tmpdir, ".pipeline")
    os.makedirs(pipeline_dir, exist_ok=True)
    schema_sysml = os.path.join(pipeline_dir, "schema.sysml")
    with open(schema_sysml, "w", encoding="utf-8") as f:
        f.write(sysml_content)

    rules_json = os.path.join(pipeline_dir, "codebase_rules.json")
    with open(rules_json, "w", encoding="utf-8") as f:
        json.dump({
            "meta": {"workspace": "test_workspace"},
            "backlog_directories": {
                "features": "docs/features",
                "epics": "docs/epics",
                "user_stories": "docs/user-stories",
                "use_cases": "docs/use-cases",
                "schemas": ".pipeline"
            },
            "validation_rules": {
                "mermaid_dotted_link_regex": r"-\.\s*[^\s\.]+\s*\.->",
                "forbidden_diagram_types": ["erDiagram"],
                "required_sections": {},
                "required_diagrams": {},
                "uml_primitives": ["String", "Integer", "Float", "Boolean"],
                "visibility_prefixes": ["+", "-", "#", "~"],
                "relationship_connectors": ["-->", "*--", "o--", "<|--", "--"],
                "choice_stereotypes": ["choice"],
                "multiplicity_regex": r"\[(?:\*|\d+(?:\.\.(?:\*|\d+))?)\]",
                "essential_feature_sections": [],
                "test_data_shape_regex": r"Test Data Shape",
                "test_data_block_regex": r"```json",
                "bdd_scenario_regexes": [r"\bGiven\b.*\bWhen\b.*\bThen\b"],
                "required_features_matrix_regex": r"## Required Features Matrix(.*?)(?=##|\Z)",
                "checkbox_syntax_regex": r"-\s*\[[ xX]\]\s*(.+)",
                "use_case_alternate_flows_header": "### Alternate Flows",
                "use_case_numbered_step_regex": r"^\s*\d+\.\s+",
                "use_case_flow_list_regex": r"(?:^|\n)####\s+[^\n]+",
                "realization_matrix_header": "## Realization Matrix",
                "realization_stories_header": "### Required User Stories",
                "realization_features_header": "### Required Features",
                "naming_conventions": {}
            }
        }, f)

    if user_story_content:
        us_dir = os.path.join(tmpdir, "docs", "user-stories")
        os.makedirs(us_dir, exist_ok=True)
        us_path = os.path.join(us_dir, "us-01-trajectory-deconfliction.md")
        with open(us_path, "w", encoding="utf-8") as f:
            f.write(user_story_content)

    if safety_doc_content:
        safety_dir = os.path.join(tmpdir, "docs", "safety")
        os.makedirs(safety_dir, exist_ok=True)
        safety_path = os.path.join(safety_dir, "STPA_MATRIX.md")
        with open(safety_path, "w", encoding="utf-8") as f:
            f.write(safety_doc_content)

    return WorkspaceRepository(tmpdir)


# ==============================================================================
# TARGET 1 (Issue #40): Check 20 - User Story Interaction & Sequence Lifelines
# ==============================================================================

VALID_SYSML_INTERACTION = """
package AutonomousUAS_SSOT {
    part def FlightGuidanceComputer {
        action ExecuteManeuver(in targetHeading : Float);
        operation def ComputeVector(in cur : Float) : Float;
    }

    interaction TrajectoryDeconfliction {
        doc /* Collision avoidance deconfliction sequence */
        lifeline FlightGuidanceComputer;
        message ExecuteManeuver;
        trigger proximityAlert;
    }
}
"""

VALID_USER_STORY_INTERACTION = """---
title: "Autonomous Trajectory Deconfliction"
type: "user-story"
generation_mode: "subagent"
interaction: "TrajectoryDeconfliction"
test_case: "TC_CollisionAvoidance_001"
---

# User Story: Autonomous Trajectory Deconfliction

## BDD Scenario (OOA/OOD Realization)
**Given** UAS is maintaining active cruise trajectory
**When** Proximity alert is received from radar
**Then** Flight guidance executes emergency deconfliction maneuver

## UML Sequence Diagram
```mermaid
sequenceDiagram
    autonumber
    actor radarOperator as "radarOperator : RadarOperator"
    participant flightGuidance as "flightGuidance : FlightGuidanceComputer"

    radarOperator->>flightGuidance: ExecuteManeuver(targetHeading: Float)
    flightGuidance-->radarOperator: status : Status
```

## Formal SysML Test Case & Verification Binding
- **SysML Test Case Def:** `TC_CollisionAvoidance_001`
- **Subject Part:** `FlightGuidanceComputer`
- **Verified Safety Requirement:** `REQ_SAF_001`
- **Verification Objective:** "Verify emergency vector computed within 150ms"
- **Test Steps:**
  - `step inject_synthetic_threat`
  - `step assert_evasive_maneuver_commanded`
"""


def test_check20_valid_interaction_and_lifelines():
    """Verify Check 20 passes when user story sequence lifelines and messages match SysML model."""
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, VALID_SYSML_INTERACTION, VALID_USER_STORY_INTERACTION)
        validator = UmlValidator()
        errors = validator.validate_user_story_interactions_and_lifelines(repo)
        assert len(errors) == 0, f"Expected 0 errors, got: {[str(e) for e in errors]}"


def test_check20_invalid_lifeline_classifier():
    """Verify Check 20 flags error when sequence lifeline participant does not match any SysML part."""
    invalid_story = VALID_USER_STORY_INTERACTION.replace(
        'participant flightGuidance as "flightGuidance : FlightGuidanceComputer"',
        'participant invalidSubsystem as "invalidSubsystem : UnknownHardwareComponent"'
    )
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, VALID_SYSML_INTERACTION, invalid_story)
        validator = UmlValidator()
        errors = validator.validate_user_story_interactions_and_lifelines(repo)
        assert any(e.rule_id == "user-story-lifeline-part-invalid" for e in errors)
        assert any("UnknownHardwareComponent" in str(e) for e in errors)


def test_check20_invalid_message_operation():
    """Verify Check 20 flags error when sequence diagram message is not declared in SysML interaction or part."""
    invalid_story = VALID_USER_STORY_INTERACTION.replace(
        "radarOperator->>flightGuidance: ExecuteManeuver(targetHeading: Float)",
        "radarOperator->>flightGuidance: UnregisteredGhostOperation(targetHeading: Float)"
    )
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, VALID_SYSML_INTERACTION, invalid_story)
        validator = UmlValidator()
        errors = validator.validate_user_story_interactions_and_lifelines(repo)
        assert any(e.rule_id == "user-story-interaction-step-invalid" for e in errors)
        assert any("UnregisteredGhostOperation" in str(e) for e in errors)


def test_check20_uncovered_sysml_interaction():
    """Verify Check 20 flags error when a SysML interaction is not realized by any User Story."""
    sysml_with_extra_interaction = VALID_SYSML_INTERACTION + """
    interaction UncoveredEmergencyLanding {
        lifeline FlightGuidanceComputer;
        message ExecuteManeuver;
    }
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, sysml_with_extra_interaction, VALID_USER_STORY_INTERACTION)
        validator = UmlValidator()
        errors = validator.validate_user_story_interactions_and_lifelines(repo)
        assert any(e.rule_id == "sysml-interaction-uncovered" for e in errors)
        assert any("UncoveredEmergencyLanding" in str(e) for e in errors)


def test_check20_external_actor_exempt():
    """Verify external actors (lifeline role 'actor') are exempt from SysML part definition check."""
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, VALID_SYSML_INTERACTION, VALID_USER_STORY_INTERACTION)
        validator = UmlValidator()
        errors = validator.validate_user_story_interactions_and_lifelines(repo)
        # RadarOperator is external actor and not in SysML, must not be flagged
        assert not any("RadarOperator" in str(e) for e in errors)


# ==============================================================================
# TARGET 2 (Issue #41): STPA Compilation & Check 21 - Safety Invariants & RTA
# ==============================================================================

SAMPLE_STPA_MATRIX = """# STPA Unsafe Control Action (UCA) Matrix

| UCA ID | Controller | Control Action | STPA UCA Category | Environmental Context Vector | Triggered System Hazard | Severity Classification | SORA SAIL Target |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **UCA-UAS-01** | Flight Guidance Computer | Fail-Safe Return-to-Launch (RTL) | **1. Not Provided** | $t_{\\text{loss}} > 2.0\\text{ s}$, $d_{\\text{geo}} < 15\\text{ m}$, C2 Link Down | **H_UAS_1:** Lost-Link Flyaway / Airspace Infringement | Catastrophic | **SAIL IV–VI** |
| **UCA-UAS-02** | Motor ESC | Emergency Rotor Braking | **2. Provided Unsafely** | $h_{\\text{AGL}} > 50\\text{ m}$, high speed flight | **H_UAS_2:** Loss of Altitude Control | Critical | **SAIL IV** |

## FMECA Failure Modes
| FMECA ID | Component | Failure Mode | Effect | Safety Invariant |
| :--- | :--- | :--- | :--- | :--- |
| **FMECA-UAS-01** | Magnetometer | Magnetic flux saturation | Compass heading divergence | flux <= 250.0 uT |
"""


def test_compile_stpa_to_sysml_and_ast():
    """Verify scripts/compile_sysml.py compiles STPA UCAs into formal SysML assert constraints."""
    ucas = parse_stpa_ucas(SAMPLE_STPA_MATRIX)
    assert len(ucas) == 2
    assert ucas[0]["id"] == "UCA-UAS-01"
    assert ucas[1]["id"] == "UCA-UAS-02"

    fmecas = parse_fmeca_modes(SAMPLE_STPA_MATRIX)
    assert len(fmecas) == 1
    assert fmecas[0]["id"] == "FMECA-UAS-01"

    sysml_code = compile_stpa_to_sysml(SAMPLE_STPA_MATRIX)
    assert "package AutonomousUAS_SafetyConstraints" in sysml_code
    assert "assert constraint Assert_UCA_UAS_01" in sysml_code
    assert "assert constraint Assert_UCA_UAS_02" in sysml_code
    assert "constraint def Constraint_FMECA_UAS_01" in sysml_code

    ast_pkg = compile_stpa_to_ast(SAMPLE_STPA_MATRIX)
    assert len(ast_pkg.constraint_defs) == 3
    assert ast_pkg.constraint_defs[0].is_assertion is True
    assert ast_pkg.constraint_defs[1].is_assertion is True
    assert ast_pkg.constraint_defs[2].is_assertion is False


def test_check21_valid_safety_invariants_and_rta_assertions():
    """Verify Check 21 passes when all STPA UCAs have assert constraint definitions in SysML AST."""
    sysml_with_assertions = """
    package AutonomousUAS_SafetyConstraints {
        assert constraint Assert_UCA_UAS_01 {
            doc /* STPA RTA Invariant for UCA-UAS-01 */
            not (lossDuration > 2.0 and c2LinkStatus == False) or (rtlActive == True);
        }

        assert constraint Assert_UCA_UAS_02 {
            doc /* STPA RTA Invariant for UCA-UAS-02 */
            altitudeAGL >= 50.0;
        }
    }
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, sysml_with_assertions, safety_doc_content=SAMPLE_STPA_MATRIX)
        validator = UmlValidator()
        errors = validator.validate_safety_invariants_and_rta_constraints(repo)
        assert len(errors) == 0, f"Expected 0 errors, got: {[str(e) for e in errors]}"


def test_check21_missing_assert_constraint_for_uca():
    """Verify Check 21 flags error when an STPA UCA lacks an assert constraint in SysML AST."""
    sysml_missing_uca02 = """
    package AutonomousUAS_SafetyConstraints {
        assert constraint Assert_UCA_UAS_01 {
            altitudeAGL >= 50.0;
        }
    }
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, sysml_missing_uca02, safety_doc_content=SAMPLE_STPA_MATRIX)
        validator = UmlValidator()
        errors = validator.validate_safety_invariants_and_rta_constraints(repo)
        assert any(e.rule_id == "stpa-uca-missing-assert-constraint" for e in errors)
        assert any("UCA-UAS-02" in str(e) for e in errors)


def test_check21_empty_rta_assertion_expression():
    """Verify Check 21 flags error when an assert constraint has an empty mathematical expression."""
    sysml_empty_expr = """
    package AutonomousUAS_SafetyConstraints {
        assert constraint Assert_UCA_UAS_01 {
            doc /* UCA-UAS-01 */
        }
        assert constraint Assert_UCA_UAS_02 {
            doc /* UCA-UAS-02 */
            altitudeAGL >= 50.0;
        }
    }
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, sysml_empty_expr, safety_doc_content=SAMPLE_STPA_MATRIX)
        validator = UmlValidator()
        errors = validator.validate_safety_invariants_and_rta_constraints(repo)
        assert any(e.rule_id == "rta-constraint-expression-empty" for e in errors)
        assert any("Assert_UCA_UAS_01" in str(e) for e in errors)


# ==============================================================================
# TARGET 3 (Issue #42): Check 22 - Acceptance Criteria & Test Case Bindings
# ==============================================================================

VALID_SYSML_TESTCASE = """
package AutonomousUAS_Verification {
    part def FlightGuidanceComputer {}

    requirement def REQ_SAF_001 {
        id = "REQ-SAF-001";
        doc /* Safe separation distance */
    }

    test case def TC_CollisionAvoidance_001 {
        subject FlightGuidanceComputer;
        verify requirement REQ_SAF_001;
        objective "Verify emergency collision avoidance within 150ms";
        step inject_threat;
        step assert_response;
    }
}
"""


def test_check22_valid_acceptance_criteria_and_test_case_bindings():
    """Verify Check 22 passes when User Story BDD scenarios bind to valid SysML test case defs."""
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, VALID_SYSML_TESTCASE, VALID_USER_STORY_INTERACTION)
        validator = UmlValidator()
        errors = validator.validate_acceptance_criteria_and_test_cases(repo)
        assert len(errors) == 0, f"Expected 0 errors, got: {[str(e) for e in errors]}"


def test_check22_user_story_missing_test_case_binding():
    """Verify Check 22 flags error when User Story lacks a formal test case binding."""
    story_without_tc = VALID_USER_STORY_INTERACTION.replace('test_case: "TC_CollisionAvoidance_001"', '').replace('`TC_CollisionAvoidance_001`', '')
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, VALID_SYSML_TESTCASE, story_without_tc)
        validator = UmlValidator()
        errors = validator.validate_acceptance_criteria_and_test_cases(repo)
        assert any(e.rule_id == "user-story-missing-test-case-binding" for e in errors)


def test_check22_bound_test_case_not_in_ast():
    """Verify Check 22 flags error when User Story binds to a test case not defined in SysML AST."""
    story_with_fake_tc = VALID_USER_STORY_INTERACTION.replace('test_case: "TC_CollisionAvoidance_001"', 'test_case: "TC_NonExistent_999"').replace('`TC_CollisionAvoidance_001`', '`TC_NonExistent_999`')
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, VALID_SYSML_TESTCASE, story_with_fake_tc)
        validator = UmlValidator()
        errors = validator.validate_acceptance_criteria_and_test_cases(repo)
        assert any(e.rule_id == "user-story-test-case-not-in-ast" for e in errors)
        assert any("TC_NonExistent_999" in str(e) for e in errors)


def test_check22_test_case_missing_verify_requirement():
    """Verify Check 22 flags error when SysML test case def does not declare any verify requirement."""
    sysml_no_verify = """
    package AutonomousUAS_Verification {
        part def FlightGuidanceComputer {}
        requirement def REQ_SAF_001 {}

        test case def TC_CollisionAvoidance_001 {
            subject FlightGuidanceComputer;
            objective "Missing verify requirement statement";
        }
    }
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, sysml_no_verify, VALID_USER_STORY_INTERACTION)
        validator = UmlValidator()
        errors = validator.validate_acceptance_criteria_and_test_cases(repo)
        assert any(e.rule_id == "test-case-missing-verify-requirement" for e in errors)


def test_check22_test_case_verifies_unknown_requirement():
    """Verify Check 22 flags error when SysML test case verifies a requirement not in the AST."""
    sysml_unknown_req = """
    package AutonomousUAS_Verification {
        part def FlightGuidanceComputer {}
        requirement def REQ_SAF_001 {}

        test case def TC_CollisionAvoidance_001 {
            subject FlightGuidanceComputer;
            verify requirement REQ_UNKNOWN_999;
        }
    }
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, sysml_unknown_req, VALID_USER_STORY_INTERACTION)
        validator = UmlValidator()
        errors = validator.validate_acceptance_criteria_and_test_cases(repo)
        assert any(e.rule_id == "test-case-verify-requirement-invalid" for e in errors)
        assert any("REQ_UNKNOWN_999" in str(e) for e in errors)


def test_check22_sysml_test_case_unbound():
    """Verify Check 22 flags error when a SysML test case is not bound to any User Story."""
    sysml_with_orphan_tc = VALID_SYSML_TESTCASE + """
    test case def TC_OrphanedVerification_002 {
        subject FlightGuidanceComputer;
        verify requirement REQ_SAF_001;
        objective "Unbound verification test case";
    }
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        repo = create_mock_workspace(tmpdir, sysml_with_orphan_tc, VALID_USER_STORY_INTERACTION)
        validator = UmlValidator()
        errors = validator.validate_acceptance_criteria_and_test_cases(repo)
        assert any(e.rule_id == "sysml-test-case-unbound" for e in errors)
        assert any("TC_OrphanedVerification_002" in str(e) for e in errors)
