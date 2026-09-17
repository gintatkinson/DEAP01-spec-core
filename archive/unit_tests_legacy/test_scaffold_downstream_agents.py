"""
Unit tests for downstream agent scaffolding utility.
/// Realises: [ScaffoldDownstreamAgents]
"""
import os
import sys
import tempfile
import shutil
import subprocess
import pytest

repo_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if repo_root not in sys.path:
    sys.path.insert(0, repo_root)

from scripts.scaffold_downstream_agents import (
    scaffold_downstream_agents,
    UPSTREAM_HEADER,
    DOWNSTREAM_HEADER,
)


@pytest.fixture
def temp_installer_root():
    """Create a temporary installer root directory with an upstream AGENTS.md."""
    temp_dir = tempfile.mkdtemp(prefix="test_installer_root_")
    agents_path = os.path.join(temp_dir, "AGENTS.md")
    sample_content = (
        "# Project-Scoped Rules\n\n"
        + UPSTREAM_HEADER
        + "\n\n## Governance Rules\n- Rule 1: Always verify before acting.\n"
    )
    with open(agents_path, "w", encoding="utf-8") as f:
        f.write(sample_content)
    yield temp_dir
    shutil.rmtree(temp_dir, ignore_errors=True)


@pytest.fixture
def temp_target_dir():
    """Create a clean temporary target directory."""
    temp_dir = tempfile.mkdtemp(prefix="test_target_dir_")
    yield temp_dir
    shutil.rmtree(temp_dir, ignore_errors=True)


def test_scaffold_downstream_agents_transformation(temp_installer_root, temp_target_dir):
    """Verify that scaffold_downstream_agents transforms UPSTREAM_SPEC_CORE_COMPILER

    to DOWNSTREAM_CUSTOMER_PROJECT / DOMAIN_TEMPLATE_CHILD and writes both
    .agents/AGENTS.md and root AGENTS.md.
    """
    scaffold_downstream_agents(temp_installer_root, temp_target_dir)

    dot_agents = os.path.join(temp_target_dir, ".agents", "AGENTS.md")
    root_agents = os.path.join(temp_target_dir, "AGENTS.md")

    assert os.path.isfile(dot_agents), "Target .agents/AGENTS.md was not created"
    assert os.path.isfile(root_agents), "Target root AGENTS.md was not created"

    with open(dot_agents, "r", encoding="utf-8") as f:
        dot_content = f.read()
    with open(root_agents, "r", encoding="utf-8") as f:
        root_content = f.read()

    assert dot_content == root_content, ".agents/AGENTS.md and root AGENTS.md must match"

    # Verify upstream header was replaced
    assert "UPSTREAM_SPEC_CORE_COMPILER" not in dot_content
    # Verify downstream header items
    assert "DOWNSTREAM_CUSTOMER_PROJECT" in dot_content
    assert "DOMAIN_TEMPLATE_CHILD" in dot_content
    assert "gintatkinson/DEAP-avionic-flight-safety" in dot_content
    assert "Tactical UAS" in dot_content

    # Verify preserved downstream rules
    assert "Rule 1: Always verify before acting." in dot_content


def test_scaffold_downstream_agents_scaffolds_claude_md(temp_installer_root, temp_target_dir):
    """Verify that CLAUDE.md is scaffolded if missing, containing MATLAB/Simulink context."""
    claude_path = os.path.join(temp_target_dir, "CLAUDE.md")
    assert not os.path.exists(claude_path)

    scaffold_downstream_agents(temp_installer_root, temp_target_dir)

    assert os.path.isfile(claude_path), "Target CLAUDE.md was not scaffolded"
    with open(claude_path, "r", encoding="utf-8") as f:
        claude_content = f.read()

    assert "Claude Code Project Guidelines" in claude_content
    assert "Primary Commercial Toolchain Integration Context" in claude_content
    assert "MATLAB / Simulink / Stateflow / Embedded Coder" in claude_content


def test_scaffold_downstream_agents_preserves_existing_claude_md(temp_installer_root, temp_target_dir):
    """Verify that an existing CLAUDE.md is not overwritten."""
    claude_path = os.path.join(temp_target_dir, "CLAUDE.md")
    custom_content = "# Pre-existing Custom CLAUDE.md"
    with open(claude_path, "w", encoding="utf-8") as f:
        f.write(custom_content)

    scaffold_downstream_agents(temp_installer_root, temp_target_dir)

    with open(claude_path, "r", encoding="utf-8") as f:
        assert f.read() == custom_content


def test_scaffold_downstream_agents_scaffolds_readme_md(temp_installer_root, temp_target_dir):
    """Verify that README.md is scaffolded if missing."""
    readme_path = os.path.join(temp_target_dir, "README.md")
    assert not os.path.exists(readme_path)

    scaffold_downstream_agents(temp_installer_root, temp_target_dir)

    assert os.path.isfile(readme_path), "Target README.md was not scaffolded"
    with open(readme_path, "r", encoding="utf-8") as f:
        readme_content = f.read()

    assert len(readme_content.strip()) > 0
    assert "DOWNSTREAM_APPLICATION_WORKSPACE" in readme_content or "DOWNSTREAM_CUSTOMER_PROJECT" in readme_content


def test_scaffold_downstream_agents_preserves_existing_readme_md(temp_installer_root, temp_target_dir):
    """Verify that an existing README.md is not overwritten."""
    readme_path = os.path.join(temp_target_dir, "README.md")
    custom_content = "# Pre-existing Custom README.md"
    with open(readme_path, "w", encoding="utf-8") as f:
        f.write(custom_content)

    scaffold_downstream_agents(temp_installer_root, temp_target_dir)

    with open(readme_path, "r", encoding="utf-8") as f:
        assert f.read() == custom_content


def test_scaffold_downstream_agents_missing_source_raises(temp_target_dir):
    """Verify FileNotFoundError is raised if installer_root lacks AGENTS.md."""
    empty_installer = tempfile.mkdtemp(prefix="test_empty_installer_")
    try:
        with pytest.raises(FileNotFoundError):
            scaffold_downstream_agents(empty_installer, temp_target_dir)
    finally:
        shutil.rmtree(empty_installer, ignore_errors=True)


def test_scaffold_downstream_agents_cli(temp_installer_root, temp_target_dir):
    """Verify CLI entrypoint execution."""
    script_path = os.path.join(repo_root, "scripts", "scaffold_downstream_agents.py")
    res = subprocess.run(
        [sys.executable, script_path, temp_installer_root, temp_target_dir],
        capture_output=True,
        text=True,
    )
    assert res.returncode == 0, f"CLI execution failed: {res.stderr}"

    dot_agents = os.path.join(temp_target_dir, ".agents", "AGENTS.md")
    root_agents = os.path.join(temp_target_dir, "AGENTS.md")
    claude_path = os.path.join(temp_target_dir, "CLAUDE.md")
    readme_path = os.path.join(temp_target_dir, "README.md")

    assert os.path.isfile(dot_agents)
    assert os.path.isfile(root_agents)
    assert os.path.isfile(claude_path)
    assert os.path.isfile(readme_path)
