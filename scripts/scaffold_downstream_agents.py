#!/usr/bin/env python3
"""
Scaffolding utility for downstream project agent configuration and governance files.
Transforms upstream compiler classification to downstream customer project / domain child.
/// Realises: [ScaffoldDownstreamAgents]
"""
import argparse
import os
import re
import sys

UPSTREAM_HEADER = """## Repository Role & Scope Classification
- **Repository Classification:** `UPSTREAM_SPEC_CORE_COMPILER` (Digital Engineering Agent Platform Core Specification Compiler).
- **Sentinel Indicator:** The presence of `.pipeline/upstream/` and `skills/spec-orchestrator/` denotes that this repository is the **Upstream Specification Core Compiler**, NOT a downstream customer application workspace or domain template.
- **Domain Template & Customer Data Boundary:** Domain-specific platforms and customer applications belong in downstream distribution repositories, and must NOT be committed to this upstream specification core compiler repository."""

DOWNSTREAM_HEADER = """## Repository Role & Scope Classification
- **Repository Classification:** `DOWNSTREAM_CUSTOMER_PROJECT` / `DOMAIN_TEMPLATE_CHILD` (Domain-Specific Safety-Critical Engineering Project)
- **Sentinel Indicator:** The absence of `.pipeline/upstream/` denotes that this repository is an active **Downstream Customer Project Workspace**, authorized for concrete application code implementation and domain feature delivery.
- **Customer Application Scope:** Downstream domain-specific application code, modules, tests, and models developed, tested, and maintained directly within this project workspace across any target domain."""

DEFAULT_CLAUDE = """# Claude Code Project Guidelines

## Workflow & Quality Gates
- Follow all pipeline rules by executing `view_file` on `.pipeline/ACTIVE_RULES_BUNDLE.md`, and skills in `skills/` and `.agents/skills/`.
- Strict Planning Gate: Do not execute unauthorized modifications without an approved implementation plan.
- Execute baseline verification: `python3 scripts/verify_downstream_baseline.py --no-domain`.
"""

DEFAULT_README = """# Downstream Safety-Critical Engineering Project

> **Repository Role:** `DOWNSTREAM_APPLICATION_WORKSPACE`

---

## 1. System Overview

This repository is an installed downstream implementation workspace governed by the **Digital Engineering Agent Platform (DEAP)** for safety-critical autonomous systems and model-based engineering.

---

## 2. Pipeline Structure & Governance

- `AGENTS.md`: Agent behavior rules, role boundaries, and subagent dispatch protocols.
- `CLAUDE.md`: Claude Code guidelines and verification gates.
- `.pipeline/`: Constitution (`constitution.md`), active governance rules bundle (`ACTIVE_RULES_BUNDLE.md`), domain specifications, and execution profiles.
- `rules/` & `skills/`: Platform engineering rules and agent workflow skills.
- `schema/`: Contract definitions and SysML v2 schemas.
- Semantic acceptance verification: Automated baseline verification and compliance checks via pipeline scripts.
"""


def scaffold_downstream_agents(installer_root: str, target_dir: str) -> None:
    """
    Transforms AGENTS.md from installer_root to downstream classification,
    writing target_dir/AGENTS.md.
    Scaffolds target_dir/CLAUDE.md and target_dir/README.md if missing.
    """
    src_candidates = [
        os.path.join(installer_root, "AGENTS.md"),
        os.path.join(installer_root, ".agents", "AGENTS.md"),
    ]
    src_agents_path = None
    for cand in src_candidates:
        if os.path.isfile(cand):
            src_agents_path = cand
            break

    if src_agents_path is None:
        raise FileNotFoundError(
            f"Source AGENTS.md not found in installer root candidates: {src_candidates}"
        )

    with open(src_agents_path, "r", encoding="utf-8") as f:
        content = f.read()

    if UPSTREAM_HEADER in content:
        transformed = content.replace(UPSTREAM_HEADER, DOWNSTREAM_HEADER)
    else:
        transformed = re.sub(
            r'## Repository Role & Scope Classification\n- \*\*Repository Classification:\*\* `UPSTREAM_SPEC_CORE_COMPILER`[^\n]*\n- \*\*Sentinel Indicator:\*\* [^\n]*\n- \*\*Domain Template & Customer Data Boundary:\*\* [^\n]*',
            DOWNSTREAM_HEADER,
            content,
        )

    os.makedirs(target_dir, exist_ok=True)

    root_agents_path = os.path.join(target_dir, "AGENTS.md")
    claude_path = os.path.join(target_dir, "CLAUDE.md")
    readme_path = os.path.join(target_dir, "README.md")

    for target_path in [root_agents_path, claude_path, readme_path]:
        if os.path.exists(target_path):
            try:
                os.chmod(target_path, 0o644)
            except OSError:
                pass

    with open(root_agents_path, "w", encoding="utf-8") as f:
        f.write(transformed)

    if not os.path.exists(claude_path):
        with open(claude_path, "w", encoding="utf-8") as f:
            f.write(DEFAULT_CLAUDE)

    if not os.path.exists(readme_path):
        with open(readme_path, "w", encoding="utf-8") as f:
            f.write(DEFAULT_README)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Scaffold downstream AGENTS.md, CLAUDE.md, and README.md governance armor."
    )
    parser.add_argument("installer_root", help="Path to installer root repository")
    parser.add_argument("target_dir", help="Path to target downstream workspace")
    args = parser.parse_args()

    scaffold_downstream_agents(args.installer_root, args.target_dir)


if __name__ == "__main__":
    main()
