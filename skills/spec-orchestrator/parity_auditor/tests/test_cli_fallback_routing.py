#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Regression tests for the upstream bug-report routing fallback in the parity
auditor CLI.

The abstract core tooling must never hardcode a customer repository name.  These
tests drive the exception-path reporter in ``parity_auditor.cli.main()`` and
assert that, when neither environment configuration nor a configurable meta
value is present, the suggested bug-report repository is the canonical core
tooling repository (``gintatkinson/DEAP01-spec-core``), and that the
``UPSTREAM_REPOSITORY`` environment variable takes precedence over the fallback.

Defect regression: a stale hardcoded customer repository literal in the
error-report suggestion routed downstream bug reports to a private customer
repository instead of the canonical tooling repository.
"""

import os
import sys

import pytest

# Ensure spec-orchestrator scripts and parity_auditor are on sys.path BEFORE the
# first parity_auditor import.  If cli.py is imported first with only parity
# src on sys.path, cardinality_validator cannot resolve ``sysmlv2_ast`` (its
# third import fallback points at a nonexistent parity_auditor/scripts dir) and
# silently binds SysMLParser = None, degrading every SysML AST check in the
# session.  This mirrors the bootstrap of the sibling parity test modules.
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PARITY_AUDITOR_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
SPEC_ORCH_DIR = os.path.abspath(os.path.join(PARITY_AUDITOR_DIR, ".."))
PROJECT_ROOT = os.path.abspath(os.path.join(SPEC_ORCH_DIR, "..", ".."))
SPEC_SCRIPTS_DIR = os.path.join(SPEC_ORCH_DIR, "scripts")
PARITY_AUDITOR_SRC = os.path.join(PARITY_AUDITOR_DIR, "src")

for p in (SPEC_SCRIPTS_DIR, PARITY_AUDITOR_SRC, PROJECT_ROOT):
    if p not in sys.path:
        sys.path.insert(0, p)

from parity_auditor import cli  # noqa: E402


def _force_internal_failure():
    """Stand-in for ``_main_impl`` that immediately raises."""
    raise RuntimeError("forced parity auditor internal failure")


def _neutralize_workspace_config(monkeypatch):
    """Force every .pipeline workspace-config lookup to miss, so the reported
    repository resolves purely from environment and fallback logic."""
    real_exists = os.path.exists

    def fake_exists(path):
        if ".pipeline" in str(path):
            return False
        return real_exists(path)

    monkeypatch.setattr(cli.os.path, "exists", fake_exists)


def _run_exception_reporter(monkeypatch, capsys):
    """Run ``cli.main()`` with a forced internal failure and return its output."""
    monkeypatch.setattr(cli, "_main_impl", _force_internal_failure)
    with pytest.raises(SystemExit) as excinfo:
        cli.main()
    assert excinfo.value.code == 1
    captured = capsys.readouterr()
    return captured.out + captured.err


def test_fallback_routes_to_canonical_core_tooling_repo(monkeypatch, capsys):
    """With no env vars and no configurable meta value, the suggested
    bug-report repo MUST be the canonical core compiler repository."""
    monkeypatch.delenv("UPSTREAM_REPOSITORY", raising=False)
    monkeypatch.delenv("GIT_REMOTE_ORIGIN", raising=False)
    _neutralize_workspace_config(monkeypatch)

    output = _run_exception_reporter(monkeypatch, capsys)

    assert "gintatkinson/DEAP01-spec-core" in output
    assert "--repo gintatkinson/DEAP01-spec-core" in output


def test_upstream_repository_env_var_takes_precedence(monkeypatch, capsys):
    """The UPSTREAM_REPOSITORY env var MUST take precedence over the fallback."""
    monkeypatch.setenv("UPSTREAM_REPOSITORY", "acme/parity-auditor")
    monkeypatch.delenv("GIT_REMOTE_ORIGIN", raising=False)
    _neutralize_workspace_config(monkeypatch)

    output = _run_exception_reporter(monkeypatch, capsys)

    assert "acme/parity-auditor" in output
    assert "--repo acme/parity-auditor" in output
