#!/usr/bin/env python3
"""
Regression guard for issue #77: skill markdown files must never carry
workspace-relative links into ``docs/architecture/blueprints/``.
"""

import re
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
SKILLS_DIR = PROJECT_ROOT / "skills"
BLUEPRINT_NAMES = (
    "DEAP_LOGICAL_INTERFACE_SPECIFICATION_BLUEPRINT",
    "SYSML_SSOT_BIDIRECTIONAL_SYNCHRONIZATION_ARCHITECTURE",
)
MARKDOWN_LINK_RE = re.compile(r"\[[^\]]*\]\(([^()\s]+)\)")


def collect_relative_blueprint_links():
    """Yield ``(file, target)`` for every markdown link in skills/ that points
    into the blueprint directory via a workspace-relative target.

    Downstream workspaces installed via ``scripts/install_pipeline.sh`` ship an
    empty ``docs/architecture/blueprints/`` directory and the constitution's
    Downstream SSOT rule forbids duplicating master core blueprints, so such
    relative links can never resolve there (GitHub issue #77). References to
    master blueprints must therefore be absolute upstream URLs so they resolve
    in every checkout.
    """
    violations = []
    for markdown_file in sorted(SKILLS_DIR.rglob("*.md")):
        text = markdown_file.read_text(encoding="utf-8")
        for target in MARKDOWN_LINK_RE.findall(text):
            if "docs/architecture/blueprints" not in target:
                continue
            if target.startswith("http://") or target.startswith("https://"):
                continue
            violations.append(
                (markdown_file.relative_to(PROJECT_ROOT), target)
            )
    return violations


def test_no_relative_blueprint_links_in_skill_markdown_files():
    violations = collect_relative_blueprint_links()
    if violations:
        rendered = "\n".join(
            f"- {file}: {target}" for file, target in violations
        )
        raise AssertionError(
            "skill markdown files contain workspace-relative links into "
            "docs/architecture/blueprints/ that cannot resolve in downstream "
            "workspaces (issue #77); reference the canonical upstream URL "
            "instead:\n" + rendered
        )
