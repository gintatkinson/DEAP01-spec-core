import os
import sys
import tempfile
import unittest
from unittest.mock import patch, MagicMock

# Add scripts directory to sys.path
scripts_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "scripts"))
if scripts_dir not in sys.path:
    sys.path.insert(0, scripts_dir)

import copy
from reconcile_backlog import (
    expand_relative_links_for_tracker,
    sync_issue_body_to_tracker,
    get_blob_url_base,
    update_checklist_in_file,
    resolve_issue_on_tracker,
    is_already_resolved,
    get_resolved_label,
    sanitize_latex_delimiters_for_tracker,
    get_structural_label,
    apply_structural_label,
    create_tracker_provider,
    GitLabV4Provider,
    DEFAULT_GITLAB_TRACKER_RULES,
)

class TestExpandRelativeLinksForTracker(unittest.TestCase):
    def setUp(self):
        self.workspace_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
        self.github_rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {"provider": "github"}
        }
        self.gitlab_rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": {"provider": "gitlab"}
        }

    def test_expand_relative_links_github(self):
        content = (
            "# Feature Spec\n\n"
            "See [Rule](rules/sysml-ssot-completeness.md) and "
            "[Doc](docs/architecture/blueprints/DEAP_MODEL.sysml).\n"
            "Parent: [Epic 1](../epics/epic-01.md)\n"
            "Anchor: [Section](../features/feat-02.md#acceptance-criteria)\n"
        )
        spec_path = os.path.join(self.workspace_dir, "docs", "features", "feat-01.md")
        mock_remote_info = {
            "raw": "https://github.com/gintatkinson/DEAP01-spec-core.git",
            "is_gitlab": False,
            "project_path": "gintatkinson/DEAP01-spec-core",
            "server_url": "https://github.com",
            "host": "github.com"
        }
        with patch("reconcile_backlog.get_current_branch", return_value="main"), \
             patch("reconcile_backlog.get_git_remote_info", return_value=mock_remote_info):
            expanded = expand_relative_links_for_tracker(
                content,
                filepath=spec_path,
                rules=self.github_rules,
                workspace_dir=self.workspace_dir
            )

        self.assertIn(
            "[Rule](https://github.com/gintatkinson/DEAP01-spec-core/blob/main/rules/sysml-ssot-completeness.md)",
            expanded
        )
        self.assertIn(
            "[Doc](https://github.com/gintatkinson/DEAP01-spec-core/blob/main/docs/architecture/blueprints/DEAP_MODEL.sysml)",
            expanded
        )
        self.assertIn(
            "[Epic 1](https://github.com/gintatkinson/DEAP01-spec-core/blob/main/docs/epics/epic-01.md)",
            expanded
        )
        self.assertIn(
            "[Section](https://github.com/gintatkinson/DEAP01-spec-core/blob/main/docs/features/feat-02.md#acceptance-criteria)",
            expanded
        )

    def test_expand_relative_links_gitlab(self):
        content = (
            "See [Rule](rules/sysml-ssot-completeness.md) and "
            "[Parent Epic](../epics/epic-01.md)."
        )
        spec_path = os.path.join(self.workspace_dir, "docs", "features", "feat-01.md")
        mock_remote_info = {
            "raw": "https://gitlab.com/gintatkinson/DEAP01-spec-core.git",
            "is_gitlab": True,
            "project_path": "gintatkinson/DEAP01-spec-core",
            "server_url": "https://gitlab.com",
            "host": "gitlab.com"
        }
        with patch("reconcile_backlog.get_current_branch", return_value="main"), \
             patch("reconcile_backlog.get_git_remote_info", return_value=mock_remote_info):
            expanded = expand_relative_links_for_tracker(
                content,
                filepath=spec_path,
                rules=self.gitlab_rules,
                workspace_dir=self.workspace_dir
            )

        self.assertIn(
            "[Rule](https://gitlab.com/gintatkinson/DEAP01-spec-core/-/blob/main/rules/sysml-ssot-completeness.md)",
            expanded
        )
        self.assertIn(
            "[Parent Epic](https://gitlab.com/gintatkinson/DEAP01-spec-core/-/blob/main/docs/epics/epic-01.md)",
            expanded
        )

    def test_expand_relative_links_preserves_absolute_and_special_links(self):
        content = (
            "External: [GitHub](https://github.com/org/repo)\n"
            "Insecure: [HTTP](http://example.com/spec)\n"
            "Mail: [Contact](mailto:dev@example.com)\n"
            "Anchor only: [Internal Link](#section-1)\n"
        )
        spec_path = os.path.join(self.workspace_dir, "docs", "features", "feat-01.md")
        with patch("reconcile_backlog.get_current_branch", return_value="main"):
            expanded = expand_relative_links_for_tracker(
                content,
                filepath=spec_path,
                rules=self.github_rules,
                workspace_dir=self.workspace_dir
            )

        self.assertIn("[GitHub](https://github.com/org/repo)", expanded)
        self.assertIn("[HTTP](http://example.com/spec)", expanded)
        self.assertIn("[Contact](mailto:dev@example.com)", expanded)
        self.assertIn("[Internal Link](#section-1)", expanded)

    def test_sync_issue_body_to_tracker_expands_relative_links(self):
        spec_content = (
            "---\ntitle: Feature One\ntype: feature\n---\n\n"
            "# Feature: Feature One\n\n"
            "## Requirements\n"
            "See [Rule](rules/sysml-ssot-completeness.md) and [Parent](../epics/epic-01.md).\n"
        )
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", delete=False) as tf:
            tf.write(spec_content)
            temp_path = tf.name

        try:
            mock_provider = MagicMock()
            mock_provider.edit_issue.return_value = True
            mock_provider.edit_issue_title.return_value = True
            mock_provider.add_label.return_value = True

            with patch("reconcile_backlog.get_current_branch", return_value="main"):
                sync_issue_body_to_tracker(
                    issue_num=101,
                    filepath=temp_path,
                    issue_type="Feature",
                    rules=self.github_rules,
                    provider_adapter=mock_provider
                )

            mock_provider.edit_issue.assert_called_once()
            called_content = mock_provider.edit_issue.call_args[0][1]
            self.assertIn(
                "https://github.com/gintatkinson/DEAP01-spec-core/blob/main/rules/sysml-ssot-completeness.md",
                called_content
            )
            self.assertIn(
                "https://github.com/gintatkinson/DEAP01-spec-core/blob/main/docs/epics/epic-01.md",
                called_content
            )
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)


class TestReconcileBacklogDependencyGating(unittest.TestCase):
    """
    Unit and integration tests ensuring reconcile_backlog.py strictly blocks automated issue
    resolution when dependencies or blocker checklist items are unmapped, missing, or open.
    (Fixes Issue #31)
    """

    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.rules = {
            "tracker_rules": {
                "provider": "github",
                "dependency_regex": r"(-\s*\[\s*([ xX])\s*\]\s*(#|#\[|\#\s*)?([A-Za-z0-9\-]+))",
                "keys": {
                    "issue_id": "number",
                    "title": "title",
                    "labels": "labels",
                    "state": "state",
                    "closed_state_value": "CLOSED",
                    "open_state_value": "OPEN",
                },
                "labels": {
                    "resolved": "status:fixed-resolved",
                },
                "commands": {
                    "resolve_issue": ["gh", "issue", "edit", "{number}", "--add-label", "{label}"],
                    "comment_issue": ["gh", "issue", "comment", "{number}", "--body", "{comment}"],
                },
            }
        }

    def tearDown(self):
        self.temp_dir.cleanup()

    def _create_spec_file(self, filename: str, content: str) -> str:
        filepath = os.path.join(self.temp_dir.name, filename)
        with open(filepath, "w", encoding="utf-8") as f:
            f.write(content)
        return filepath

    def test_unmapped_missing_dependency_prevents_resolution(self):
        """
        If a dependency issue referenced in the checklist is not found in the tracker (dep_issue is None),
        update_checklist_in_file MUST return completed=False to prevent premature resolution (#31).
        """
        content = (
            "# Epic: Autonomous Guidance\n\n"
            "## Dependencies\n"
            "- [x] #101 - Core Engine (Closed)\n"
            "- [ ] #999 - Missing / Unmapped Dependency\n"
        )
        spec_file = self._create_spec_file("epic-01.md", content)
        issue_dict = {
            101: {"number": 101, "title": "Core Engine", "state": "CLOSED", "labels": []},
            # Note: 999 is intentionally missing from issue_dict
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertFalse(completed, "Specification with missing/unmapped dependency must NOT be completed")

    def test_unmapped_missing_dependency_already_checked_prevents_resolution(self):
        """
        If a dependency issue referenced in the checklist is missing from tracker but was pre-checked [x] in markdown,
        update_checklist_in_file MUST return completed=False.
        """
        content = (
            "# Epic: Autonomous Guidance\n\n"
            "## Dependencies\n"
            "- [x] #101 - Core Engine (Closed)\n"
            "- [x] #999 - Unmapped Hallucinated Dep\n"
        )
        spec_file = self._create_spec_file("epic-01b.md", content)
        issue_dict = {
            101: {"number": 101, "title": "Core Engine", "state": "CLOSED", "labels": []},
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertFalse(completed, "Specification with unmapped dependency marked [x] must NOT be completed")

    def test_unresolved_placeholder_prevents_resolution(self):
        """
        If a dependency checklist item contains an unresolved placeholder like #[IssueID],
        update_checklist_in_file MUST return completed=False.
        """
        content = (
            "# User Story: Obstacle Avoidance\n\n"
            "## Dependencies\n"
            "- [x] #101 - Sensor Driver (Closed)\n"
            "- [ ] #[StoryIssueID] - Unassigned Story\n"
        )
        spec_file = self._create_spec_file("us-01.md", content)
        issue_dict = {
            101: {"number": 101, "title": "Sensor Driver", "state": "CLOSED", "labels": []},
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertFalse(completed, "Specification with unresolved placeholder token must NOT be completed")

    def test_open_dependency_prevents_resolution(self):
        """
        If any referenced dependency is still in OPEN state on the tracker,
        update_checklist_in_file MUST return completed=False.
        """
        content = (
            "# Use Case: Waypoint Navigation\n\n"
            "## Dependencies\n"
            "- [x] #101 - Path Planning (Closed)\n"
            "- [ ] #102 - Actuator Control (Open)\n"
        )
        spec_file = self._create_spec_file("uc-01.md", content)
        issue_dict = {
            101: {"number": 101, "title": "Path Planning", "state": "CLOSED", "labels": []},
            102: {"number": 102, "title": "Actuator Control", "state": "OPEN", "labels": []},
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertFalse(completed, "Specification with open dependencies must NOT be completed")

    def test_unchecked_plain_checkbox_prevents_resolution(self):
        """
        If a specification checklist contains unchecked manual blocker items (e.g. - [ ] Manual Task),
        it MUST NOT be marked completed even if tracker issues are closed.
        """
        content = (
            "# Epic: Safety Architecture\n\n"
            "## Dependencies\n"
            "- [x] #101 - Safety Boundary Validator (Closed)\n"
            "- [ ] Manual flight clearance sign-off\n"
        )
        spec_file = self._create_spec_file("epic-02.md", content)
        issue_dict = {
            101: {"number": 101, "title": "Safety Boundary Validator", "state": "CLOSED", "labels": []},
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertFalse(completed, "Specification with unchecked manual blocker checkbox must NOT be completed")

    def test_all_closed_dependencies_allow_resolution(self):
        """
        When all declared dependencies are CLOSED on the tracker,
        update_checklist_in_file returns completed=True and updates checkbox marks to [x].
        """
        content = (
            "# Epic: Flight Control\n\n"
            "## Dependencies\n"
            "- [ ] #101 - Yaw Control\n"
            "- [ ] #102 - Pitch Control\n"
        )
        spec_file = self._create_spec_file("epic-03.md", content)
        issue_dict = {
            101: {"number": 101, "title": "Yaw Control", "state": "CLOSED", "labels": []},
            102: {"number": 102, "title": "Pitch Control", "state": "CLOSED", "labels": []},
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertTrue(completed, "Specification with all closed dependencies must be completed")
        self.assertIn("- [x] #101", updated_content)
        self.assertIn("- [x] #102", updated_content)

    def test_empty_checklist_prevents_resolution(self):
        """
        A specification without any tracked dependencies / checklist items must NOT be marked completed.
        """
        content = (
            "# Epic: Empty Spec\n\n"
            "This specification has no deliverables or dependency checklist items.\n"
        )
        spec_file = self._create_spec_file("epic-04.md", content)
        issue_dict = {101: {"number": 101, "state": "CLOSED"}}

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertFalse(completed, "Specification with no dependency checklist items must NOT be marked completed")

    def test_broken_reference_link_prevents_resolution(self):
        """
        If a checklist item has a broken reference or link format (e.g. - [ ] [Broken](../missing.md)),
        it must remain uncompleted.
        """
        content = (
            "# Epic: Broken Links\n\n"
            "## Dependencies\n"
            "- [x] #101 - Valid Link (Closed)\n"
            "- [ ] [Broken Spec Reference](../missing/spec.md)\n"
        )
        spec_file = self._create_spec_file("epic-05.md", content)
        issue_dict = {
            101: {"number": 101, "title": "Valid Link", "state": "CLOSED", "labels": []},
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)
        self.assertFalse(completed, "Specification with broken spec links must NOT be completed")

    def test_mock_tracker_resolution_gated_by_dependencies(self):
        """
        Verify resolve_issue_on_tracker is never called when unmapped dependencies are present.
        """
        content = (
            "# Epic: Tracker Integration Guard\n\n"
            "## Dependencies\n"
            "- [x] #101 - Subsystem A (Closed)\n"
            "- [ ] #999 - Missing Subsystem\n"
        )
        spec_file = self._create_spec_file("epic-06.md", content)
        issue_dict = {
            101: {"number": 101, "title": "Subsystem A", "state": "CLOSED", "labels": []},
            200: {"number": 200, "title": "Tracker Integration Guard", "state": "OPEN", "labels": []},
        }

        mock_adapter = MagicMock()
        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.rules)

        if completed and not is_already_resolved(issue_dict[200], self.rules):
            resolve_issue_on_tracker(200, "Epic completed.", rules=self.rules, provider_adapter=mock_adapter)

        # resolve_issue_on_tracker must not have been invoked
        mock_adapter.add_label.assert_not_called()
        mock_adapter.comment_issue.assert_not_called()

class TestSanitizeLatexDelimitersForTracker(unittest.TestCase):
    """
    Unit tests ensuring sanitize_latex_delimiters_for_tracker converts non-mathematical
    alphanumeric identifiers enclosed in LaTeX math delimiters ($...$) to bold text (**...**)
    before tracker API upload, while preserving genuine mathematical formulas intact.
    (Fixes Issue #46)
    """

    def test_sanitize_latex_delimiters_for_tracker(self):
        # 1. Non-mathematical alphanumeric identifiers converted to bold text
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$SC-01$"), "**SC-01**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$H-1$"), "**H-1**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$OSO-11$"), "**OSO-11**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$L-1$"), "**L-1**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$UCA-1$"), "**UCA-1**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$REQ-SYS-001$"), "**REQ-SYS-001**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$LS-1$"), "**LS-1**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("**$SC-01$**"), "**SC-01**")

        # Range identifiers
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$SC-1..N$"), "**SC-1..N**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$L-1..N$"), "**L-1..N**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$H-1..N$"), "**H-1..N**")
        self.assertEqual(sanitize_latex_delimiters_for_tracker("$UCA-1..N$"), "**UCA-1..N**")

        # Prose context with mixed tokens
        prose_input = (
            "Geofence boundary breach ($H-1$, $L-1$). "
            r"Enforces pitch limits between $-15^\circ$ and $+25^\circ$. "
            "Safety constraints ($SC-1..N$) mitigate Unsafe Control Actions ($UCA-1..N$)."
        )
        expected_prose = (
            "Geofence boundary breach (**H-1**, **L-1**). "
            r"Enforces pitch limits between $-15^\circ$ and $+25^\circ$. "
            "Safety constraints (**SC-1..N**) mitigate Unsafe Control Actions (**UCA-1..N**)."
        )
        self.assertEqual(sanitize_latex_delimiters_for_tracker(prose_input), expected_prose)

        # 2. Genuine mathematical formulas preserved intact
        self.assertEqual(
            sanitize_latex_delimiters_for_tracker(r"$\sum_{i=1}^n x_i$"),
            r"$\sum_{i=1}^n x_i$"
        )
        self.assertEqual(
            sanitize_latex_delimiters_for_tracker(r"$E = mc^2$"),
            r"$E = mc^2$"
        )
        self.assertEqual(
            sanitize_latex_delimiters_for_tracker(r"$\text{RPN} = S \times O \times D$"),
            r"$\text{RPN} = S \times O \times D$"
        )
        self.assertEqual(
            sanitize_latex_delimiters_for_tracker(r"$$SW(N) = L_{immediate}(N) + \sum_{C \in Containers(N)} L(C)$$"),
            r"$$SW(N) = L_{immediate}(N) + \sum_{C \in Containers(N)} L(C)$$"
        )

    def test_sync_issue_body_to_tracker_sanitizes_latex_delimiters(self):
        spec_content = (
            "---\ntitle: Test Safety Spec\ntype: feature\n---\n\n"
            "# Feature: Test Safety Spec\n\n"
            "- **$SC-01$**: Flight controller limit.\n"
            "- Heading error $H-1$ leading to $L-1$.\n"
            r"- Equation: $\text{RPN} = S \times O \times D$." "\n"
        )
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", delete=False) as tf:
            tf.write(spec_content)
            temp_path = tf.name

        try:
            mock_provider = MagicMock()
            mock_provider.edit_issue.return_value = True
            mock_provider.edit_issue_title.return_value = True
            mock_provider.add_label.return_value = True

            rules = {
                "tracker_rules": {"provider": "github"},
                "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"}
            }

            with patch("reconcile_backlog.get_current_branch", return_value="main"):
                sync_issue_body_to_tracker(
                    issue_num=102,
                    filepath=temp_path,
                    issue_type="Feature",
                    rules=rules,
                    provider_adapter=mock_provider
                )

            mock_provider.edit_issue.assert_called_once()
            called_content = mock_provider.edit_issue.call_args[0][1]
            self.assertIn("**SC-01**", called_content)
            self.assertIn("**H-1**", called_content)
            self.assertIn("**L-1**", called_content)
            self.assertNotIn("$SC-01$", called_content)
            self.assertNotIn("$H-1$", called_content)
            self.assertNotIn("$L-1$", called_content)
            self.assertIn(r"$\text{RPN} = S \times O \times D$", called_content)
        finally:
            if os.path.exists(temp_path):
                os.remove(temp_path)


class TestCommitMessageNonClosureInvariant(unittest.TestCase):
    """Verifies that commit message guidelines and validators reject auto-closing trigger keywords."""

    def test_auto_closing_keywords_detected(self):
        trigger_patterns = [
            r'\bfix\s+#\d+',
            r'\bfixes\s+#\d+',
            r'\bfixed\s+#\d+',
            r'\bclose\s+#\d+',
            r'\bcloses\s+#\d+',
            r'\bclosed\s+#\d+',
            r'\bresolve\s+#\d+',
            r'\bresolves\s+#\d+',
            r'\bresolved\s+#\d+',
        ]
        import re

        # Bad commit messages that would trigger server-side auto-closure
        bad_messages = [
            "feat(compiler): multi-mode FMECA extraction (fix #50, fix #51)",
            "fix(rules): update validator (closes #47)",
            "docs: update handoff (resolved #12)",
            "feat: add feature closes #99",
        ]

        # Good commit messages using neutral citation syntax
        good_messages = [
            "feat(compiler): multi-mode FMECA extraction (#50, #51)",
            "fix(rules): update validator (refs #47)",
            "docs: update handoff (#12)",
            "feat: add feature (#99)",
        ]

        combined_regex = re.compile("|".join(trigger_patterns), re.IGNORECASE)

        for msg in bad_messages:
            self.assertTrue(bool(combined_regex.search(msg)), f"Failed to flag trigger in: {msg}")

        for msg in good_messages:
            self.assertFalse(bool(combined_regex.search(msg)), f"False positive flag in: {msg}")


class TestGitLabProviderParity(unittest.TestCase):
    """
    Unit tests verifying 100% feature parity for the GitLab provider adapter in reconcile_backlog.py.
    """

    def setUp(self):
        self.workspace_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
        self.gitlab_rules = {
            "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
            "tracker_rules": copy.deepcopy(DEFAULT_GITLAB_TRACKER_RULES)
        }
        self.temp_dir = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_gitlab_issue_creation_with_full_markdown_body(self):
        """
        Verifies issue creation payload retains Mermaid diagrams, KaTeX formulas,
        and metadata tables without truncation.
        """
        spec_content = (
            "---\n"
            "title: Tactical UAS DAA Collision Avoidance\n"
            "type: feature\n"
            "dal: DAL A\n"
            "---\n\n"
            "# Feature: Tactical UAS DAA Collision Avoidance\n\n"
            "## Architecture Statechart\n\n"
            "```mermaid\n"
            "stateDiagram-v2\n"
            "    [*] --> ClearAirspace\n"
            "    ClearAirspace --> TrafficAlert: DAA_Traffic_Intruder_Detected\n"
            "    TrafficAlert --> ResolutionAdvisory: Tau_Closure <= 25s\n"
            "    ResolutionAdvisory --> ClearAirspace: Intruder_Clear\n"
            "```\n\n"
            "## Mathematical Invariants\n\n"
            "Risk Priority Number formulation:\n"
            "$$\n"
            r"\text{RPN} = S \times O \times D" "\n"
            "$$\n\n"
            r"Enforces lateral separation $d_{\mathrm{sep}} \ge 500\,\mathrm{m}$." "\n\n"
            "## Requirements\n"
            "See [Architecture Blueprint](docs/architecture/blueprints/DEAP_MODEL.sysml) "
            "and [Parent Epic](../epics/epic-01.md).\n"
        )
        spec_path = os.path.join(self.temp_dir.name, "feat-daa.md")
        with open(spec_path, "w", encoding="utf-8") as f:
            f.write(spec_content)

        # 1. Test GitLabV4Provider.create_issue with full payload
        provider = GitLabV4Provider(
            server_url="https://gitlab.example.com",
            project_id="uas/infrastructure-safety",
            token="glpat-mock-token",
            workspace_dir=self.workspace_dir
        )

        mock_response = {
            "iid": 42,
            "title": "Tactical UAS DAA Collision Avoidance",
            "description": spec_content,
            "state": "opened",
            "labels": ["type::feature"]
        }
        with patch.object(provider, "_api_request", return_value=(201, mock_response, {})) as mock_api:
            res = provider.create_issue(
                title="Tactical UAS DAA Collision Avoidance",
                description=spec_content,
                labels=["type::feature"]
            )
            self.assertIsNotNone(res)
            self.assertEqual(res["number"], 42)
            mock_api.assert_called_once()
            called_endpoint, called_kwargs = mock_api.call_args[0][0], mock_api.call_args[1]
            self.assertEqual(called_endpoint, "projects/uas%2Finfrastructure-safety/issues")
            self.assertEqual(called_kwargs.get("method"), "POST")
            self.assertEqual(called_kwargs["data"]["title"], "Tactical UAS DAA Collision Avoidance")
            self.assertEqual(called_kwargs["data"]["description"], spec_content)
            self.assertEqual(called_kwargs["data"]["labels"], "type::feature")

        # 2. Test sync_issue_body_to_tracker preserves Mermaid, KaTeX, and frontmatter table
        mock_adapter = MagicMock()
        mock_adapter.edit_issue.return_value = True
        mock_adapter.edit_issue_title.return_value = True
        mock_adapter.add_label.return_value = True

        mock_remote_info = {
            "raw": "https://gitlab.example.com/uas/infrastructure-safety.git",
            "is_gitlab": True,
            "project_path": "uas/infrastructure-safety",
            "server_url": "https://gitlab.example.com",
            "host": "gitlab.example.com"
        }

        with patch("reconcile_backlog.get_current_branch", return_value="main"), \
             patch("reconcile_backlog.get_git_remote_info", return_value=mock_remote_info):
            sync_issue_body_to_tracker(
                issue_num=42,
                filepath=spec_path,
                issue_type="Feature",
                rules=self.gitlab_rules,
                provider_adapter=mock_adapter
            )

        mock_adapter.edit_issue.assert_called_once()
        synced_body = mock_adapter.edit_issue.call_args[0][1]

        # Verify frontmatter converted to table
        self.assertIn("| Attribute | Specification Detail |", synced_body)
        self.assertIn("| **Title** | Tactical UAS DAA Collision Avoidance |", synced_body)
        self.assertIn("| **Dal** | DAL A |", synced_body)

        # Verify Mermaid diagrams preserved
        self.assertIn("```mermaid", synced_body)
        self.assertIn("stateDiagram-v2", synced_body)
        self.assertIn("ClearAirspace --> TrafficAlert", synced_body)

        # Verify KaTeX formulas preserved without truncation
        self.assertIn(r"\text{RPN} = S \times O \times D", synced_body)
        self.assertIn(r"$d_{\mathrm{sep}} \ge 500\,\mathrm{m}$", synced_body)

        # Verify relative links expanded to GitLab blob URLs
        self.assertIn("https://gitlab.example.com/uas/infrastructure-safety/-/blob/main/docs/architecture/blueprints/DEAP_MODEL.sysml", synced_body)
        self.assertIn("https://gitlab.example.com/uas/infrastructure-safety/-/blob/main/docs/epics/epic-01.md", synced_body)

    def test_gitlab_scoped_label_mapping(self):
        """
        Verifies type::epic, type::feature, type::user-story, type::use-case, status::fixed-resolved mapping.
        """
        self.assertEqual(get_structural_label("epic", self.gitlab_rules), "type::epic")
        self.assertEqual(get_structural_label("feature", self.gitlab_rules), "type::feature")
        self.assertEqual(get_structural_label("user_story", self.gitlab_rules), "type::user-story")
        self.assertEqual(get_structural_label("user-story", self.gitlab_rules), "type::user-story")
        self.assertEqual(get_structural_label("User Story", self.gitlab_rules), "type::user-story")
        self.assertEqual(get_structural_label("use_case", self.gitlab_rules), "type::use-case")
        self.assertEqual(get_structural_label("use-case", self.gitlab_rules), "type::use-case")
        self.assertEqual(get_structural_label("Use Case", self.gitlab_rules), "type::use-case")
        self.assertEqual(get_resolved_label(self.gitlab_rules), "status::fixed-resolved")

        # Test apply_structural_label on GitLab provider adapter
        mock_adapter = MagicMock()
        mock_adapter.create_label.return_value = True
        mock_adapter.add_label.return_value = True

        res = apply_structural_label(101, "User Story", rules=self.gitlab_rules, provider_adapter=mock_adapter)
        self.assertTrue(res)
        mock_adapter.create_label.assert_called_with(
            "type::user-story",
            description="User Story specification item, applied by the backlog reconciler.",
            color="#0E8A16"
        )
        mock_adapter.add_label.assert_called_with(101, "type::user-story")

    def test_gitlab_dependency_checklist_generation(self):
        """
        Verifies cross-reference resolution and checkbox generation (- [ ] #101) for GitLab issues.
        """
        spec_content = (
            "# Epic: Tactical UAS Safe Landing\n\n"
            "## Dependencies\n"
            "- [ ] #101 - Terrain Elevation Mapping\n"
            "- [ ] #102 - Optical Flow Landing Sensor\n"
        )
        spec_file = os.path.join(self.temp_dir.name, "epic-landing.md")
        with open(spec_file, "w", encoding="utf-8") as f:
            f.write(spec_content)

        issue_dict = {
            101: {"iid": 101, "number": 101, "title": "Terrain Elevation Mapping", "state": "CLOSED", "labels": [{"name": "type::feature"}]},
            102: {"iid": 102, "number": 102, "title": "Optical Flow Landing Sensor", "state": "OPENED", "labels": [{"name": "type::feature"}]},
        }

        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.gitlab_rules)
        self.assertFalse(completed, "Checklist with open dependency #102 must not be completed")
        self.assertIn("- [x] #101", updated_content)
        self.assertIn("- [ ] #102", updated_content)

        # Now close issue 102 and re-run
        issue_dict[102]["state"] = "CLOSED"
        updated_content2, completed2 = update_checklist_in_file(spec_file, issue_dict, self.gitlab_rules)
        self.assertTrue(completed2, "Checklist with all closed dependencies must be completed")
        self.assertIn("- [x] #101", updated_content2)
        self.assertIn("- [x] #102", updated_content2)

    def test_gitlab_unmapped_dependency_gating(self):
        """
        Verifies that open or unmapped dependencies prevent premature status::fixed-resolved labeling.
        """
        spec_content = (
            "# Feature: ASTM F3269-17 RTA Switching Logic\n\n"
            "## Dependencies\n"
            "- [x] #101 - Primary Autopilot Channel (Closed)\n"
            "- [ ] #999 - Unmapped Backup Channel\n"
        )
        spec_file = os.path.join(self.temp_dir.name, "feat-rta.md")
        with open(spec_file, "w", encoding="utf-8") as f:
            f.write(spec_content)

        issue_dict = {
            101: {"iid": 101, "number": 101, "title": "Primary Autopilot Channel", "state": "CLOSED", "labels": [{"name": "type::feature"}]},
            201: {"iid": 201, "number": 201, "title": "ASTM F3269-17 RTA Switching Logic", "state": "OPENED", "labels": [{"name": "type::feature"}]},
        }

        mock_adapter = MagicMock()
        updated_content, completed = update_checklist_in_file(spec_file, issue_dict, self.gitlab_rules)
        self.assertFalse(completed, "Unmapped dependency #999 must block completion")

        if completed and not is_already_resolved(issue_dict[201], self.gitlab_rules):
            resolve_issue_on_tracker(201, "Resolved.", rules=self.gitlab_rules, provider_adapter=mock_adapter)

        # Must not have resolved issue on tracker
        mock_adapter.create_label.assert_not_called()
        mock_adapter.add_label.assert_not_called()
        mock_adapter.comment_issue.assert_not_called()

    def test_gitlab_offline_mode(self):
        """
        Verifies offline reconciliation execution.
        """
        provider = GitLabV4Provider(
            server_url="https://gitlab.example.com",
            project_id="uas/infrastructure-safety",
            token="glpat-mock-token",
            offline=True,
            workspace_dir=self.workspace_dir
        )
        self.assertTrue(provider.offline)
        self.assertEqual(provider.list_issues(), [])
        self.assertIsNone(provider.create_issue("Title", "Description"))
        self.assertFalse(provider.edit_issue(42, "Description"))
        self.assertFalse(provider.edit_issue_title(42, "Title"))
        self.assertFalse(provider.add_label(42, "type::feature"))
        self.assertFalse(provider.comment_issue(42, "Comment"))
        self.assertFalse(provider.create_label("type::feature"))

        # Test factory instantiation in offline mode
        factory_provider = create_tracker_provider("gitlab", rules=self.gitlab_rules, offline=True, workspace_dir=self.workspace_dir)
        self.assertIsInstance(factory_provider, GitLabV4Provider)
        self.assertTrue(factory_provider.offline)
        self.assertEqual(factory_provider.list_issues(), [])


if __name__ == "__main__":
    unittest.main()
