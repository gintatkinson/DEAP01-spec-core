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
- **Repository Classification:** `UPSTREAM_SPEC_CORE_COMPILER` (Digital Engineering Agent Platform Core Specification Compiler)
- **Sentinel Indicator:** The presence of `.pipeline/upstream/` and `skills/spec-orchestrator/` denotes that this repository is the **Upstream Specification Core Compiler**, NOT a downstream customer application workspace or domain template.
- **Domain Template & Customer Data Boundary:** Domain-specific platforms (e.g. UAS safety, automotive, medical) and customer applications belong in downstream distribution repositories, and must NOT be committed to this upstream specification core compiler repository."""

DOWNSTREAM_HEADER = """## Repository Role & Scope Classification
- **Repository Classification:** `DOWNSTREAM_CUSTOMER_PROJECT` / `DOMAIN_TEMPLATE_CHILD` (Domain-Specific Safety-Critical Engineering Project)
- **Parent Domain Template:** `gintatkinson/DEAP-avionic-flight-safety` (Common Aviation Safety Standards)
- **Sentinel Indicator:** The absence of `.pipeline/upstream/` denotes that this repository is an active **Downstream Customer Project Workspace**, authorized for concrete application code implementation and domain feature delivery.
- **Customer Application Scope:** Tactical UAS flight safety, contingency management, run-time assurance (RTA), detect-and-avoid (DAA), and autopilot integration under SORA / ASTM / DO-178C frameworks. Customer-specific application code, domain nodes/modules, domain tests, mission envelopes, and proprietary safety models are developed, tested, and maintained directly within this project workspace across any target domain (Aerospace, Medical, Space, Industrial AGV, Subsea, Rail)."""

DEFAULT_CLAUDE = """# Claude Code Project Guidelines

## Primary Commercial Toolchain Integration Context
This project explicitly declares MATLAB / Simulink / Stateflow / Embedded Coder as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

## Workflow & Quality Gates
- Follow all pipeline rules in `rules/` and skills in `skills/` and `.agents/skills/`.
- Strict Planning Gate: Do not execute unauthorized modifications without an approved implementation plan.
- Execute baseline verification: `pytest tests/test_baseline.py` and `python3 scripts/verify_downstream_baseline.py --no-domain`.
"""

DEFAULT_README = """# Downstream Safety-Critical Engineering Project

> **Repository Role:** `DOWNSTREAM_APPLICATION_WORKSPACE`
> **Parent Domain Template:** `gintatkinson/DEAP-avionic-flight-safety`

---

## 1. System Overview

This repository is an installed downstream implementation workspace governed by the **Digital Engineering Agent Platform (DEAP)** for tactical UAS and safety-critical autonomous systems.

### 1.1 Primary Commercial Toolchain Integration Context

This platform explicitly declares **MATLAB / Simulink / Stateflow / Embedded Coder** as the Primary Tier-1 Commercial Toolchain Integration Context (Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

---

## 2. Pipeline Structure & Governance

- `.agents/` & `AGENTS.md`: Agent behavior rules, role boundaries, and subagent dispatch protocols.
- `CLAUDE.md`: Claude Code guidelines and verification gates.
- `.pipeline/`: Constitution (`constitution.md`), domain specifications, and execution profiles.
- `rules/` & `skills/`: Platform engineering rules and agent workflow skills.
- `schema/`: Contract definitions and SysML v2 schemas.
- `tests/`: Automated baseline verification and safety compliance tests.
"""


def scaffold_downstream_agents(installer_root: str, target_dir: str) -> None:
    """
    Transforms AGENTS.md from installer_root to downstream classification,
    writing both target_dir/.agents/AGENTS.md and target_dir/AGENTS.md.
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

    target_dot_agents_dir = os.path.join(target_dir, ".agents")
    os.makedirs(target_dot_agents_dir, exist_ok=True)

    dot_agents_path = os.path.join(target_dot_agents_dir, "AGENTS.md")
    root_agents_path = os.path.join(target_dir, "AGENTS.md")

    with open(dot_agents_path, "w", encoding="utf-8") as f:
        f.write(transformed)

    with open(root_agents_path, "w", encoding="utf-8") as f:
        f.write(transformed)

    claude_path = os.path.join(target_dir, "CLAUDE.md")
    if not os.path.exists(claude_path):
        with open(claude_path, "w", encoding="utf-8") as f:
            f.write(DEFAULT_CLAUDE)

    readme_path = os.path.join(target_dir, "README.md")
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
