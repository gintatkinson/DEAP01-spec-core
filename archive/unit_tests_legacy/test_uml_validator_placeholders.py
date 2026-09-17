"""
Unit tests for UML validator draft metadata placeholders and semantic traceability (Issues #228, #229).

Tests:
1. Metadata table rows with draft placeholders (#[IssueID], #[EpicID], #[FeatureID], #[EpicIssueID], #TBD) pass validation without errors.
2. Placeholder tokens in prose or non-metadata contexts are rejected.
3. Issue #228: test cases referencing undefined requirements are flagged with test-case-verify-requirement-invalid even when AST has 0 requirement defs.
4. Issue #229: bareword prose mentions of an interaction name do not mark the interaction realized.
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
from parity_auditor.validators.uml import (
    UmlValidator,
    find_unresolved_placeholders,
    _is_metadata_header_table_row,
    ALWAYS_INVALID_PLACEHOLDER_PATTERNS,
)


class TestUmlValidatorPlaceholders(unittest.TestCase):
    def setUp(self):
        self.validator = UmlValidator()

    def test_metadata_table_placeholders_pass(self):
        """Verify that metadata table rows with allowed draft reference placeholders pass validation."""
        valid_metadata_table = """# Feature: Flight Dynamics
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #[IssueID] |
| **Title** | Flight Dynamics Control |
| **Parent Epic** | #[EpicID] |
| **Feature ID** | #[FeatureID] |
| **Status** | #TBD |
| **Parent Epic** | #[EpicIssueID] |
| Issue ID | #[IssueID] |
| Parent Epic | #[EpicID] |
| Feature ID | #[FeatureID] |
| Status | #TBD |
| **Epic ID** | #[EpicID] |
| **Epic Issue ID** | #[EpicIssueID] |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/Model.sysml](../../schema/Model.sysml) |

## Description
A fully populated description without placeholder tokens.
"""
        placeholders = list(find_unresolved_placeholders(valid_metadata_table))
        self.assertEqual(placeholders, [], f"Expected no unresolved placeholders in valid metadata table, got: {placeholders}")

        errors = []
        self.validator._validate_placeholders_and_links(
            valid_metadata_table, "Feature", "feat-01.md", errors, r"^\s*-\s*\[[ xX]\]"
        )
        self.assertEqual(errors, [], f"Expected 0 errors from _validate_placeholders_and_links, got: {errors}")

    def test_prose_and_non_metadata_placeholders_rejected(self):
        """Verify that draft reference placeholders in prose or invalid templates are rejected."""
        prose_cases = [
            ("This feature is tracked by #[IssueID].", "unresolved issue reference token"),
            ("The parent epic is #[EpicID].", "unresolved issue reference token"),
            ("Related feature is #[FeatureID].", "unresolved issue reference token"),
            ("The implementation status is #TBD currently.", "unresolved reference token"),
            ("- [ ] #[IssueID] - [Child Feature](../features/feat-01.md)", "unresolved issue reference token"),
            ("| **Title** | [Feature Title] |", "unpopulated template title"),
            ("| **Issue ID** | [POPULATE: Issue ID] |", "unreplaced [POPULATE:] placeholder token"),
            ("| **Issue ID** | #[UnknownCustomToken] |", "unresolved issue reference token"),
        ]

        for text, expected_label in prose_cases:
            with self.subTest(text=text):
                placeholders = list(find_unresolved_placeholders(text))
                self.assertTrue(
                    len(placeholders) > 0,
                    f"Expected placeholder to be flagged in '{text}', but none was found."
                )
                self.assertEqual(placeholders[0][1], expected_label)

    def test_issue_228_test_case_verifies_undefined_requirement_when_zero_ast_requirements(self):
        """Issue #228: test case referencing undefined requirement is flagged when AST has 0 requirement defs."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            user_stories_dir = os.path.join(tmpdir, "docs", "user-stories")
            os.makedirs(user_stories_dir, exist_ok=True)

            sysml_content = """package Model {
    part def FlightController;
    test case def TC_ArmingVerification {
        verify requirement REQ_Safety_01;
    }
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            story_content = """# User Story: Arming Procedure

| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 101 |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

## Acceptance Criteria
SysML Test Case Def: `TC_ArmingVerification`
"""
            with open(os.path.join(user_stories_dir, "us-01-arming.md"), "w", encoding="utf-8") as f:
                f.write(story_content)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate_acceptance_criteria_and_test_cases(repo)

            invalid_req_findings = [e for e in errors if e.rule_id == "test-case-verify-requirement-invalid"]
            self.assertTrue(
                len(invalid_req_findings) > 0,
                f"Expected 'test-case-verify-requirement-invalid' when AST has 0 requirements but test case verifies REQ_Safety_01, got: {errors}"
            )
            self.assertIn("REQ_Safety_01", str(invalid_req_findings[0]))

    def test_issue_229_bareword_prose_does_not_realize_sysml_interaction(self):
        """Issue #229: bareword prose mention of an interaction does not mark it as realized."""
        with tempfile.TemporaryDirectory() as tmpdir:
            schema_dir = os.path.join(tmpdir, "schema")
            os.makedirs(schema_dir, exist_ok=True)
            user_stories_dir = os.path.join(tmpdir, "docs", "user-stories")
            os.makedirs(user_stories_dir, exist_ok=True)

            sysml_content = """package Model {
    part def FlightController {
        action def armMotors;
    }
    interaction def ArmingInteraction {
        message armMotors;
    }
}
"""
            with open(os.path.join(schema_dir, "model.sysml"), "w", encoding="utf-8") as f:
                f.write(sysml_content)

            # Story only contains ArmingInteraction in bareword prose, NOT in frontmatter or formal header
            story_content_bareword = """# User Story: Arming

| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 101 |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

## Description
This specification discusses ArmingInteraction in passing prose.

```mermaid
sequenceDiagram
    autonumber
    actor Pilot
    participant C as FlightController
    Pilot->>C: armMotors()
```
"""
            story_path = os.path.join(user_stories_dir, "us-01-arming.md")
            with open(story_path, "w", encoding="utf-8") as f:
                f.write(story_content_bareword)

            repo = WorkspaceRepository(workspace_dir=tmpdir)
            errors = self.validator.validate_user_story_interactions_and_lifelines(repo)

            uncovered_findings = [e for e in errors if e.rule_id == "sysml-interaction-uncovered"]
            self.assertTrue(
                len(uncovered_findings) > 0,
                f"Expected 'sysml-interaction-uncovered' when interaction is only mentioned in bareword prose, got: {errors}"
            )
            self.assertIn("ArmingInteraction", str(uncovered_findings[0]))

            story_content_formal = """---
interaction: ArmingInteraction
---

# User Story: Arming

| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | 101 |
| **Generation Mode** | subagent |
| **Specification Source** | [schema/model.sysml](../../schema/model.sysml) |

## Description
Formally bound user story.

```mermaid
sequenceDiagram
    autonumber
    actor Pilot
    participant C as FlightController
    Pilot->>C: armMotors()
```
"""
            with open(story_path, "w", encoding="utf-8") as f:
                f.write(story_content_formal)

            errors_formal = self.validator.validate_user_story_interactions_and_lifelines(repo)
            uncovered_findings_formal = [e for e in errors_formal if e.rule_id == "sysml-interaction-uncovered"]
            self.assertEqual(
                uncovered_findings_formal,
                [],
                f"Expected 0 'sysml-interaction-uncovered' findings when formally bound, got: {errors_formal}"
            )


if __name__ == "__main__":
    unittest.main()
