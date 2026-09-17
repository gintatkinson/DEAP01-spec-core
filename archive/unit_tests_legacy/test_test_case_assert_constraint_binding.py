#!/usr/bin/env python3
"""
Regression test suite for Check 22 Acceptance Criteria & Test Case Verification Bindings (Issue #231).
Verifies that verify requirement bindings targeting assert constraint IDs and requirement defs
pass validation and invalid targets are rejected.
"""

import json
import os
import sys
import tempfile
import unittest

# Setup path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
PARITY_AUDITOR_SRC = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "parity_auditor", "src")
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")

for p in (SPEC_SCRIPTS_DIR, PARITY_AUDITOR_SRC, PROJECT_ROOT):
    if p not in sys.path:
        sys.path.insert(0, p)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.uml import UmlValidator


SYSML_SCHEMA_ASSERT_CONSTRAINT = """
package AutonomousDefenseSystem {
    part def WeaponController {}

    assert constraint Assert_UCA_33 {
        armState == ArmStatus::ARMED;
    }

    assert constraint Assert_UCA_34 {
        lockConfidence >= 0.98;
    }

    requirement def REQ_SAF_001 {
        id = "REQ-SAF-001";
        doc /* Safety interlock requirement */
    }

    test case def TC_UCA_33_Verification {
        subject WeaponController;
        verify requirement Assert_UCA_33;
        objective "Verify armState interlock per UCA-33";
        step trigger_arming;
        step assert_arm_status;
    }

    test case def TC_Req_Saf_001_Verification {
        subject WeaponController;
        verify requirement REQ_SAF_001;
        objective "Verify safety interlock per REQ-SAF-001";
        step verify_interlock;
    }
}
"""

USER_STORY_ASSERT_CONSTRAINT_BOUND = """---
issue_id: 201
title: "Verify Weapon Arming Interlock"
test_case: "TC_UCA_33_Verification"
---

# US-201: Verify Weapon Arming Interlock

As a Safety Verification System, I want to verify weapon arming interlocks, so that unintended discharge is prevented.

## BDD Acceptance Criteria
- SysML Test Case Def: `TC_UCA_33_Verification`
- Given safe default state, when arming signal is received, then armState transitions to ARMED.
"""

USER_STORY_REQ_DEF_BOUND = """---
issue_id: 202
title: "Verify Safety Interlock Requirement"
test_case: "TC_Req_Saf_001_Verification"
---

# US-202: Verify Safety Interlock Requirement

As a Safety Officer, I want to verify safety interlocks, so that the platform meets airworthiness standards.

## BDD Acceptance Criteria
- SysML Test Case Def: `TC_Req_Saf_001_Verification`
- Given armed state, when interlock is disengaged, then weapon firing is prohibited.
"""


def _create_workspace(tmpdir, schema_content, story_files_dict):
    pipeline_dir = os.path.join(tmpdir, ".pipeline")
    os.makedirs(pipeline_dir, exist_ok=True)
    with open(os.path.join(pipeline_dir, "schema.sysml"), "w", encoding="utf-8") as f:
        f.write(schema_content)

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
            }
        }, f)

    stories_dir = os.path.join(tmpdir, "docs", "user-stories")
    os.makedirs(stories_dir, exist_ok=True)
    for fname, fcontent in story_files_dict.items():
        with open(os.path.join(stories_dir, fname), "w", encoding="utf-8") as f:
            f.write(fcontent)

    return WorkspaceRepository(tmpdir)


class TestTestCaseAssertConstraintBinding(unittest.TestCase):
    """Test suite verifying test case verify requirement bindings against assert constraints and requirements."""

    def test_verify_requirement_targeting_assert_constraint_id_passes(self):
        """Test case verifying an assert constraint ID (e.g. Assert_UCA_33) passes without error."""
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = _create_workspace(tmpdir, SYSML_SCHEMA_ASSERT_CONSTRAINT, {
                "us-201-arming.md": USER_STORY_ASSERT_CONSTRAINT_BOUND,
                "us-202-interlock.md": USER_STORY_REQ_DEF_BOUND
            })
            validator = UmlValidator()
            errors = validator.validate_acceptance_criteria_and_test_cases(repo)
            self.assertEqual(len(errors), 0, f"Expected 0 errors, got: {[str(e) for e in errors]}")

    def test_verify_requirement_targeting_unknown_target_is_rejected(self):
        """Test case verifying an unknown requirement/constraint ID is flagged as invalid."""
        sysml_with_invalid_target = """
package AutonomousDefenseSystem {
    part def WeaponController {}

    test case def TC_Invalid_Target {
        subject WeaponController;
        verify requirement NONEXISTENT_UCA_999;
        objective "Invalid target";
    }
}
"""
        story_with_invalid_target = """---
issue_id: 203
title: "Invalid Verification Binding"
test_case: "TC_Invalid_Target"
---

# US-203: Invalid Verification Binding

## BDD Acceptance Criteria
- SysML Test Case Def: `TC_Invalid_Target`
- Given state, when action, then outcome.
"""
        with tempfile.TemporaryDirectory() as tmpdir:
            repo = _create_workspace(tmpdir, sysml_with_invalid_target, {
                "us-203-invalid.md": story_with_invalid_target
            })
            validator = UmlValidator()
            errors = validator.validate_acceptance_criteria_and_test_cases(repo)
            invalid_errors = [e for e in errors if e.rule_id == "test-case-verify-requirement-invalid"]
            self.assertTrue(len(invalid_errors) > 0)
            self.assertTrue(any("NONEXISTENT_UCA_999" in str(e) for e in invalid_errors))


if __name__ == "__main__":
    unittest.main()
