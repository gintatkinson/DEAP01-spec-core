#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for upstream compiler repository mode exemptions in the parity auditor.

The upstream DEAP01-spec-core compiler repository is mis-scored by checks that
assume a downstream application workspace with populated landing zones and client
codebases:

- the docs/ landing zones (docs/features, docs/epics, ...) are clean BY DESIGN;
- there is no app_flutter/ or web_react/ client code to scan BY DESIGN;
- yet validators report 'Codebase is empty' / 'No test files found in the workspace'
  and missing-local-specification out-of-sync findings.

These tests drive the upstream-mode exemption.  Detection is the same sentinel used by
``reconcile_backlog.py`` (issue #68 mechanism) and documented in ``.pipeline/constitution.md``
and ``AGENTS.md``: the presence of ``.pipeline/upstream`` (or the
``DEAP_REPOSITORY_TYPE=UPSTREAM_SPEC_CORE_COMPILER`` environment variable).

Referenced upstream open-feature issues: #74, #73, #72, #70, #67, #64, #62, #61, #60, #59.
"""

import json
import os
import sys
import pytest

# Ensure parity_auditor is on sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PARITY_AUDITOR_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
PARITY_AUDITOR_SRC = os.path.join(PARITY_AUDITOR_DIR, "src")
for p in (PARITY_AUDITOR_SRC,):
    if p not in sys.path:
        sys.path.insert(0, p)

from parity_auditor.core.workspace import (
    WorkspaceRepository,
    is_upstream_compiler_repo,
    has_configured_target_code_directories,
)
from parity_auditor.core.findings import Finding
from parity_auditor.validators.sync_validator import SyncValidator
from parity_auditor.validators.schema_mapping_validator import SchemaMappingValidator
from parity_auditor.validators.profile_scoping_validator import ProfileScopingValidator
from parity_auditor.validators.test_completeness_validator import TestCompletenessValidator
from parity_auditor.validators.mermaid_syntax_validator import MermaidSyntaxValidator
from parity_auditor.validators.link_validator import LinkValidator
from parity_auditor.validators.conops_completeness_validator import (
    ConopsCompletenessValidator,
    MissionIntentCompletenessValidator,
)


def _write_rules(workspace_dir, **overrides):
    """Write a minimal valid codebase_rules.json into a temp workspace."""
    rules = {
        "meta": {"upstream_repository": "gintatkinson/DEAP01-spec-core"},
        "backlog_directories": {
            "epics": "docs/epics",
            "features": "docs/features",
            "user_stories": "docs/user-stories",
            "use_cases": "docs/use-cases",
            "schemas": "schema",
        },
        "tracker_rules": {
            "commands": {"list_issues": ["gh", "issue", "list", "--json", "number,title,state,labels"]},
            "labels": {"epic": "epic", "feature": "feature"},
        },
        "target_directories": {"react": "web_react", "flutter": "app_flutter"},
        "flutter_rules": {
            "file_extensions": [".dart"],
            "exclusions": ["build", ".dart_tool", ".git"],
            "ui_directories": ["widgets", "screens"],
        },
    }
    rules.update(overrides)
    os.makedirs(os.path.join(workspace_dir, ".pipeline", "logical-ui"), exist_ok=True)
    rules_path = os.path.join(workspace_dir, ".pipeline", "logical-ui", "codebase_rules.json")
    with open(rules_path, "w", encoding="utf-8") as f:
        json.dump(rules, f)
    return rules_path


class _FakeResult:
    def __init__(self, returncode, stdout="", stderr=""):
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr


def _issue(number, title, labels):
    return {"number": number, "title": title, "state": "OPEN",
            "labels": [{"name": l} for l in labels]}


# ---------------------------------------------------------------------------
# (a) Upstream-mode detection on a temp workspace with a .pipeline/upstream marker
# ---------------------------------------------------------------------------

def test_upstream_mode_detected_via_pipeline_upstream_marker(tmp_path, monkeypatch):
    monkeypatch.delenv("DEAP_REPOSITORY_TYPE", raising=False)
    os.makedirs(tmp_path / ".pipeline" / "upstream", exist_ok=True)
    assert is_upstream_compiler_repo(str(tmp_path)) is True


def test_upstream_mode_not_detected_without_marker(tmp_path, monkeypatch):
    monkeypatch.delenv("DEAP_REPOSITORY_TYPE", raising=False)
    os.makedirs(tmp_path / ".pipeline", exist_ok=True)
    assert is_upstream_compiler_repo(str(tmp_path)) is False


def test_upstream_mode_detected_via_environment_variable(tmp_path, monkeypatch):
    monkeypatch.setenv("DEAP_REPOSITORY_TYPE", "UPSTREAM_SPEC_CORE_COMPILER")
    assert is_upstream_compiler_repo(str(tmp_path)) is True


def test_workspace_repository_exposes_upstream_mode(tmp_path, monkeypatch):
    monkeypatch.delenv("DEAP_REPOSITORY_TYPE", raising=False)
    os.makedirs(tmp_path / ".pipeline" / "upstream", exist_ok=True)
    repo = WorkspaceRepository(str(tmp_path))
    assert repo.is_upstream_compiler_repo() is True


def test_has_configured_target_code_directories(tmp_path, monkeypatch):
    os.makedirs(tmp_path / "app_flutter", exist_ok=True)
    rules_path = _write_rules(str(tmp_path))
    monkeypatch.setenv("CODEBASE_RULES_PATH", rules_path)
    repo = WorkspaceRepository(str(tmp_path))
    assert has_configured_target_code_directories(repo) is True


def test_has_configured_target_code_directories_false_when_missing(tmp_path, monkeypatch):
    rules_path = _write_rules(str(tmp_path))
    monkeypatch.setenv("CODEBASE_RULES_PATH", rules_path)
    repo = WorkspaceRepository(str(tmp_path))
    assert has_configured_target_code_directories(repo) is False


# ---------------------------------------------------------------------------
# (b) Missing-spec out-of-sync findings: suppressed in upstream mode,
#     NOT suppressed without it
# ---------------------------------------------------------------------------

def _make_upstream_workspace(tmp_path, monkeypatch, marker=True):
    if marker:
        os.makedirs(tmp_path / ".pipeline" / "upstream", exist_ok=True)
    rules_path = _write_rules(str(tmp_path))
    monkeypatch.setenv("CODEBASE_RULES_PATH", rules_path)
    return WorkspaceRepository(str(tmp_path))


_FEATURE_ISSUE = _issue(74, "feat(auditor): implement Gate 26 ConOps and Mission Intent completeness validator engine", ["feature"])
_EPIC_ISSUE = _issue(12, "Epic 12: Architecture", ["epic"])


def test_sync_validator_suppresses_missing_spec_for_upstream_feature_issues(tmp_path, monkeypatch):
    repo = _make_upstream_workspace(tmp_path, monkeypatch, marker=True)

    issue_payload = json.dumps([_FEATURE_ISSUE, _EPIC_ISSUE])
    monkeypatch.setattr(
        "parity_auditor.validators.sync_validator.subprocess.run",
        lambda *a, **kw: _FakeResult(0, stdout=issue_payload),
    )

    errors = SyncValidator().validate(repo)
    messages = [str(e) for e in errors]
    # Feature-issue missing-spec findings are suppressed in upstream mode...
    assert not any("Issue #74" in m for m in messages), messages


def test_sync_validator_reports_missing_spec_without_upstream_marker(tmp_path, monkeypatch):
    repo = _make_upstream_workspace(tmp_path, monkeypatch, marker=False)

    issue_payload = json.dumps([_FEATURE_ISSUE])
    monkeypatch.setattr(
        "parity_auditor.validators.sync_validator.subprocess.run",
        lambda *a, **kw: _FakeResult(0, stdout=issue_payload),
    )

    errors = SyncValidator().validate(repo)
    messages = [str(e) for e in errors]
    # Without the upstream marker the finding MUST still be reported.
    assert any("Issue #74" in m for m in messages), messages


# ---------------------------------------------------------------------------
# (c) Empty-codebase configuration-state non-violations skipped in upstream mode
# ---------------------------------------------------------------------------

def test_empty_codebase_validators_skip_in_upstream_mode_without_target_dirs(tmp_path, monkeypatch):
    rules_path = _write_rules(str(tmp_path), target_directories={"react": None, "flutter": None})
    os.makedirs(tmp_path / ".pipeline" / "upstream", exist_ok=True)
    monkeypatch.setenv("CODEBASE_RULES_PATH", rules_path)
    repo = WorkspaceRepository(str(tmp_path))

    schema_errors = SchemaMappingValidator().validate(repo)
    profile_errors = ProfileScopingValidator().validate(repo)
    test_errors = TestCompletenessValidator().validate(repo)

    all_messages = [str(e) for e in schema_errors + profile_errors + test_errors]
    assert not any("Codebase is empty" in m for m in all_messages), all_messages
    assert not any("No test files found" in m for m in all_messages), all_messages


def test_empty_codebase_validators_still_report_without_upstream_marker(tmp_path, monkeypatch):
    rules_path = _write_rules(str(tmp_path), target_directories={"react": None, "flutter": None})
    os.makedirs(tmp_path / "schema", exist_ok=True)
    (tmp_path / "schema" / ".gitkeep").write_text("", encoding="utf-8")
    monkeypatch.setenv("CODEBASE_RULES_PATH", rules_path)
    repo = WorkspaceRepository(str(tmp_path))

    schema_errors = SchemaMappingValidator().validate(repo)
    all_messages = [str(e) for e in schema_errors]
    assert any("Codebase is empty" in m for m in all_messages), all_messages


def test_conops_and_mission_intent_skip_in_upstream_mode(tmp_path, monkeypatch):
    os.makedirs(tmp_path / ".pipeline" / "upstream", exist_ok=True)
    repo = WorkspaceRepository(str(tmp_path))

    conops_errors = ConopsCompletenessValidator().validate(repo)
    mission_errors = MissionIntentCompletenessValidator().validate(repo)

    assert conops_errors == []
    assert mission_errors == []


def test_conops_and_mission_intent_fail_closed_without_upstream_marker(tmp_path, monkeypatch):
    monkeypatch.delenv("DEAP_REPOSITORY_TYPE", raising=False)
    repo = WorkspaceRepository(str(tmp_path))

    conops_errors = ConopsCompletenessValidator().validate(repo)
    mission_errors = MissionIntentCompletenessValidator().validate(repo)

    assert len(conops_errors) == 1
    assert conops_errors[0].rule_id == "conops-corpus-missing"
    assert len(mission_errors) == 1
    assert mission_errors[0].rule_id == "mission-intent-corpus-missing"


# ---------------------------------------------------------------------------
# (d) Real content findings (Mermaid syntax, markdown links) still reported
#     in upstream mode
# ---------------------------------------------------------------------------

def test_mermaid_and_link_findings_still_reported_in_upstream_mode(tmp_path, monkeypatch):
    os.makedirs(tmp_path / ".pipeline" / "upstream", exist_ok=True)
    docs_dir = tmp_path / "docs"
    os.makedirs(docs_dir, exist_ok=True)
    broken_md = docs_dir / "spec.md"
    broken_md.write_text(
        "# Bad Spec\n\n"
        "```mermaid\ngraph TD\n"
        '    A["Source"] --> B["Target"]\n'
        "    Note over A,B: this note; has semicolons\n"
        "```\n\n"
        "See [broken link](../../missing/file.md).\n",
        encoding="utf-8",
    )
    rules_path = _write_rules(str(tmp_path))
    monkeypatch.setenv("CODEBASE_RULES_PATH", rules_path)
    repo = WorkspaceRepository(str(tmp_path))

    mermaid_errors = MermaidSyntaxValidator().validate(repo)
    assert mermaid_errors, "Mermaid syntax violations must still be reported in upstream mode"
    assert any("mermaid" in str(e).lower() or "semicolon" in str(e).lower() for e in mermaid_errors)

    link_errors = LinkValidator().validate(repo)
    assert link_errors, "Broken markdown links must still be reported in upstream mode"
    assert any("broken link" in str(e).lower() or "missing/file.md" in str(e) for e in link_errors)
