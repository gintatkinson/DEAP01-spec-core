#!/usr/bin/env python3
"""
Subagent Prompt Dispatcher and Generator for DEAP01-spec-core

Automates canonical subagent prompt generation, enforces repository standards,
and eliminates manual prompt summarization.

Usage:
    python3 scripts/dispatch_subagent.py --skill skills/feature-driven-implementation/SKILL.md --target scripts/dispatch_subagent.py
    python3 scripts/dispatch_subagent.py --skill skills/adversarial-code-auditor/SKILL.md --target src/module.py --role "Adversarial Code Auditor" --type adversarial_auditor
"""

import argparse
import json
import os
import re
import sys
import uuid
from typing import Optional, List

# Ensure repository root and scripts directory are in sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

try:
    from scripts.lint_subagent_prompt import (
        lint_prompt_text,
        lint_subagent_prompt,
        validate_subagent_preflight,
        check_mandate_fidelity,
        load_mandate_requirements,
    )
except ImportError:
    try:
        from lint_subagent_prompt import (
            lint_prompt_text,
            lint_subagent_prompt,
            validate_subagent_preflight,
            check_mandate_fidelity,
            load_mandate_requirements,
        )
    except ImportError:
        # Fallback if imported from another context
        lint_prompt_text = None
        lint_subagent_prompt = None
        validate_subagent_preflight = None
        check_mandate_fidelity = None
        load_mandate_requirements = None


DEFAULT_CLASSIFICATION = "DOWNSTREAM_APPLICATION_WORKSPACE"
DEFAULT_ROLE = "Context-Isolated Worker"
DEFAULT_TYPE = "code_modifier_worker"


def validate_skill_path(skill_path: str, base_dir: Optional[str] = None) -> str:
    """
    Validates that the provided skill path exists and references a canonical SKILL.md file.

    Args:
        skill_path: Relative or absolute path to SKILL.md.
        base_dir: Optional base directory to resolve relative paths against.

    Returns:
        The validated path string.

    Raises:
        ValueError: If the skill path is empty or does not target a SKILL.md file.
        FileNotFoundError: If the skill file does not exist on disk.
    """
    if not skill_path or not isinstance(skill_path, str) or not skill_path.strip():
        raise ValueError("Skill path must be a non-empty string.")

    cleaned_path = skill_path.strip()
    norm_path = os.path.normpath(cleaned_path)
    base_name = os.path.basename(norm_path)

    if base_name != "SKILL.md":
        raise ValueError(
            f"Invalid skill path '{cleaned_path}'. Skill file must be named 'SKILL.md'."
        )

    # Check existence directly or relative to base_dir / PROJECT_ROOT
    candidates = [cleaned_path]
    if base_dir:
        candidates.append(os.path.join(base_dir, cleaned_path))
    candidates.append(os.path.join(PROJECT_ROOT, cleaned_path))

    resolved_path = None
    for candidate in candidates:
        if os.path.isfile(candidate):
            resolved_path = candidate
            break

    if resolved_path is None:
        raise FileNotFoundError(
            f"Target skill file does not exist: '{cleaned_path}'"
        )

    return cleaned_path


def resolve_repository_classification(
    base_dir: Optional[str] = None,
    explicit_classification: Optional[str] = None,
) -> str:
    """
    Dynamically determines the repository classification for subagent dispatch.

    Resolution Precedence:
    1. Explicit classification argument (if provided and non-empty).
    2. Environment variable overrides:
       - `DEAP_REPOSITORY_TYPE`
       - `REPO_CLASSIFICATION`
       - `REPOSITORY_CLASSIFICATION`
    3. Workspace lineage metadata: `.pipeline/lineage.json` (or `lineage.json`).
       Inspects `classification`, `role`, or `tier` fields.
    4. Project constitution / README metadata: `.pipeline/constitution.md`, `constitution.md`, or `README.md`.
       Inspects for explicit repository classification/role declarations.
    5. Upstream sentinel: `.pipeline/upstream` -> `UPSTREAM_SPEC_CORE_COMPILER`.
    6. Default fallback:
       - `DOWNSTREAM_APPLICATION_WORKSPACE` if no upstream sentinel in workspace.
       - `UPSTREAM_SPEC_CORE_COMPILER` if `.pipeline/upstream` sentinel is present in workspace.

    Args:
        base_dir: Optional base directory to search for metadata files.
        explicit_classification: Optional explicitly provided classification.

    Returns:
        The resolved repository classification string.
    """
    # 1. Explicit classification argument
    if explicit_classification is not None and isinstance(explicit_classification, str):
        cleaned = explicit_classification.strip()
        if cleaned:
            return cleaned

    # 2. Environment variable overrides (Issue #90)
    for env_var in ("DEAP_REPOSITORY_TYPE", "REPO_CLASSIFICATION", "REPOSITORY_CLASSIFICATION"):
        env_val = os.environ.get(env_var)
        if env_val and env_val.strip():
            return env_val.strip()

    # Determine candidate search directories
    if base_dir:
        search_dirs = [os.path.abspath(base_dir)]
    else:
        cwd = os.path.abspath(os.getcwd())
        if cwd == PROJECT_ROOT or cwd.startswith(PROJECT_ROOT + os.sep):
            search_dirs = [cwd, PROJECT_ROOT]
        else:
            search_dirs = [cwd]

    for ws in search_dirs:
        # 3. Check lineage.json (Issue #87)
        lineage_candidates = [
            os.path.join(ws, ".pipeline", "lineage.json"),
            os.path.join(ws, "lineage.json"),
        ]
        for lineage_path in lineage_candidates:
            if os.path.isfile(lineage_path):
                try:
                    with open(lineage_path, "r", encoding="utf-8") as f:
                        data = json.load(f)
                    if isinstance(data, dict):
                        # Check explicit classification field
                        if data.get("classification") and isinstance(data["classification"], str):
                            cls_str = data["classification"].strip()
                            if cls_str:
                                return cls_str
                        # Check role field
                        if data.get("role") and isinstance(data["role"], str):
                            role_str = data["role"].strip()
                            role_norm = role_str.lower().replace("-", "_")
                            if role_norm in ("parent_domain_distribution_template", "domain_parent", "parent_domain_template"):
                                return "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE"
                            elif role_norm in ("child_domain_distribution_template", "domain_child", "child_domain_template"):
                                return "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE"
                            elif role_norm in ("downstream_application_workspace", "application_workspace", "downstream_workspace", "workspace"):
                                return "DOWNSTREAM_APPLICATION_WORKSPACE"
                            elif role_norm in ("upstream_spec_core_compiler", "upstream_core", "upstream_compiler", "spec_core_compiler"):
                                return "UPSTREAM_SPEC_CORE_COMPILER"
                            else:
                                return role_str
                        # Check tier field
                        if "tier" in data:
                            tier = data["tier"]
                            if tier == 0 or tier == "0":
                                return "UPSTREAM_SPEC_CORE_COMPILER"
                            elif tier == 1 or tier == "1":
                                return "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE"
                            elif tier == 2 or tier == "2":
                                return "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE"
                            elif isinstance(tier, int) and tier >= 3:
                                return "DOWNSTREAM_APPLICATION_WORKSPACE"
                except Exception:
                    pass

        # 4. Check .pipeline/constitution.md, constitution.md, or README.md (Issue #87)
        meta_candidates = [
            os.path.join(ws, ".pipeline", "constitution.md"),
            os.path.join(ws, "constitution.md"),
            os.path.join(ws, "README.md"),
        ]
        for meta_path in meta_candidates:
            if os.path.isfile(meta_path):
                try:
                    with open(meta_path, "r", encoding="utf-8") as f:
                        meta_content = f.read()
                    role_match = re.search(
                        r'(?:Repository\s*(?:Role|Classification)|Classification)[*`\s]*[:=][*`\s]*([A-Za-z0-9_]+)',
                        meta_content,
                        re.IGNORECASE,
                    )
                    if role_match:
                        matched_role = role_match.group(1).strip()
                        matched_norm = matched_role.lower().replace("-", "_")
                        if matched_norm in ("parent_domain_distribution_template", "domain_parent"):
                            return "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE"
                        elif matched_norm in ("child_domain_distribution_template", "domain_child"):
                            return "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE"
                        elif matched_norm in ("downstream_application_workspace", "application_workspace"):
                            return "DOWNSTREAM_APPLICATION_WORKSPACE"
                        elif matched_norm in ("upstream_spec_core_compiler", "upstream_core"):
                            return "UPSTREAM_SPEC_CORE_COMPILER"
                        elif matched_role:
                            return matched_role
                except Exception:
                    pass

        # 5. Check .pipeline/upstream sentinel marker
        upstream_marker = os.path.join(ws, ".pipeline", "upstream")
        if os.path.exists(upstream_marker):
            return "UPSTREAM_SPEC_CORE_COMPILER"

    return DEFAULT_CLASSIFICATION


def construct_prompt_template(
    skill_path: str,
    target: str,
    role: str = DEFAULT_ROLE,
    subagent_type: str = DEFAULT_TYPE,
    classification: Optional[str] = None,
    instructions: Optional[str] = None,
    base_dir: Optional[str] = None,
) -> str:
    """
    Constructs the standard non-negotiable prompt template for subagent dispatch.

    Template components:
    (1) Step 1 `view_file` on `{skill_path}`.
    (2) Verbatim repository classification.
    (3) Target file / specification item.
    (4) Explicit instructions and standards.
    (5) Trailing authorization token `PROCEED`.

    Args:
        skill_path: Path to target SKILL.md file.
        target: Target file, directory, or specification item.
        role: Descriptive role of the subagent.
        subagent_type: Subagent type identifier.
        classification: Optional repository classification (dynamically resolved if None).
        instructions: Optional additional instructions.
        base_dir: Optional base directory to search for workspace metadata.

    Returns:
        The generated prompt text.
    """
    resolved_classification = resolve_repository_classification(
        base_dir=base_dir,
        explicit_classification=classification,
    )

    extra_block = ""
    if instructions and instructions.strip():
        extra_block = f"\nTask Details:\n{instructions.strip()}\n"

    checklist_block = ""
    if load_mandate_requirements is not None:
        requirements, requirement_error = load_mandate_requirements()
        if requirements:
            checklist_lines = [f"- {req}" for req in requirements]
            checklist_block = (
                "\nNormative Pre-Flight Checklist (verbatim from `rules/subagent-dispatch-standards.md`):\n"
                + "\n".join(checklist_lines)
                + "\n"
            )

    prompt = f"""You are a context-isolated subagent operating under the DEAP Engineering Framework.

Role: {role}
Subagent Type: {subagent_type}
Repository Classification: {resolved_classification}
Target: {target}

Mandatory Instructions:
1. Step 1: Execute `view_file` on `{skill_path}` as your very first step before executing any file edits, commands, or tools. Strictly follow its instruction guidelines and formatting templates.
2. Repository Scope: You are operating within classification `{resolved_classification}`. Maintain all repository invariants and domain boundaries.
3. Micro-Task Scope: Focus exclusively on target `{target}` within a single-item micro-task scope.
4. Engineering Standards: Follow test-driven development (RED-GREEN-REFACTOR) cycle discipline and strict verification before completion.
5. Defect Reporting: If any defects, anomalies, or bugs are detected, record them using `gh issue create` (GitHub) or `glab issue create` (GitLab). Issue closure is strictly reserved for Product Owner review.
{checklist_block}{extra_block}
PROCEED
"""
    return prompt


def generate_subagent_prompt(
    skill: str,
    target: str,
    role: str = DEFAULT_ROLE,
    subagent_type: str = DEFAULT_TYPE,
    classification: Optional[str] = None,
    instructions: Optional[str] = None,
    base_dir: Optional[str] = None,
) -> str:
    """
    Generates and validates a subagent dispatch prompt payload.

    Args:
        skill: Path to target SKILL.md.
        target: Target file or specification item.
        role: Descriptive role of the subagent.
        subagent_type: Subagent type identifier.
        classification: Optional repository classification (dynamically resolved if None).
        instructions: Optional custom instructions.
        base_dir: Optional base directory for skill resolution and workspace metadata.

    Returns:
        The validated prompt text.

    Raises:
        ValueError: If input validation or prompt linting fails.
        FileNotFoundError: If skill file does not exist.
    """
    if not target or not isinstance(target, str) or not target.strip():
        raise ValueError("Target path or specification item must be a non-empty string.")

    valid_skill_path = validate_skill_path(skill, base_dir=base_dir)
    resolved_classification = resolve_repository_classification(
        base_dir=base_dir,
        explicit_classification=classification,
    )

    prompt_text = construct_prompt_template(
        skill_path=valid_skill_path,
        target=target.strip(),
        role=role.strip() if role else DEFAULT_ROLE,
        subagent_type=subagent_type.strip() if subagent_type else DEFAULT_TYPE,
        classification=resolved_classification,
        instructions=instructions,
        base_dir=base_dir,
    )

    # Validate generated prompt using lint_prompt_text
    linter_fn = lint_prompt_text or lint_subagent_prompt
    if linter_fn is not None:
        errors = linter_fn(prompt_text)
        if errors:
            raise ValueError(
                f"Generated prompt failed lint validation with {len(errors)} violation(s):\n"
                + "\n".join(f"  - {err}" for err in errors)
            )

    return prompt_text


def dispatch_subagent(
    skill: str,
    target: str,
    role: str = DEFAULT_ROLE,
    subagent_type: str = DEFAULT_TYPE,
    classification: Optional[str] = None,
    output: Optional[str] = None,
    instructions: Optional[str] = None,
    base_dir: Optional[str] = None,
) -> str:
    """
    Constructs, validates, writes, and returns a canonical subagent prompt payload.

    Args:
        skill: Path to target SKILL.md.
        target: Target file or specification item.
        role: Descriptive role of the subagent.
        subagent_type: Subagent type identifier.
        classification: Optional repository classification (dynamically resolved if None).
        output: Optional path to write payload file.
        instructions: Optional custom instructions.
        base_dir: Optional base directory for skill resolution and workspace metadata.

    Returns:
        The path to the generated payload file.
    """
    prompt_text = generate_subagent_prompt(
        skill=skill,
        target=target,
        role=role,
        subagent_type=subagent_type,
        classification=classification,
        instructions=instructions,
        base_dir=base_dir,
    )

    if check_mandate_fidelity is not None:
        fidelity_ok, fidelity_coverage, fidelity_missing, fidelity_error = check_mandate_fidelity(prompt_text)
        if not fidelity_ok:
            if fidelity_error:
                fidelity_detail = fidelity_error
            else:
                fidelity_detail = "missing mandatory Requirements: " + "; ".join(
                    f"'{req}'" for req in fidelity_missing
                )
            print(
                f"HALT: mandate fidelity gate rejected outgoing payload "
                f"(coverage {fidelity_coverage:.0%}): {fidelity_detail}",
                file=sys.stderr,
            )
            sys.exit(2)

    output_path = output
    if not output_path:
        output_path = f"/tmp/subagent_prompt_{uuid.uuid4().hex}.md"

    output_dir = os.path.dirname(os.path.abspath(output_path))
    if output_dir:
        os.makedirs(output_dir, exist_ok=True)

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(prompt_text)

    return output_path


def main():
    parser = argparse.ArgumentParser(
        description="DEAP Subagent Prompt Generator and Dispatcher",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--skill",
        required=True,
        help="Path to target SKILL.md (e.g. skills/feature-driven-implementation/SKILL.md)",
    )
    parser.add_argument(
        "--target",
        required=True,
        help="Path to target file or specification item (e.g. scripts/dispatch_subagent.py)",
    )
    parser.add_argument(
        "--role",
        default=DEFAULT_ROLE,
        help=f"Descriptive role of the subagent (default: '{DEFAULT_ROLE}')",
    )
    parser.add_argument(
        "--type",
        dest="subagent_type",
        default=DEFAULT_TYPE,
        help=f"Subagent type identifier (default: '{DEFAULT_TYPE}')",
    )
    parser.add_argument(
        "--classification",
        default=None,
        help="Repository classification (default: dynamically resolved from workspace / environment)",
    )
    parser.add_argument(
        "--output",
        default=None,
        help="Optional path to write payload (defaults to /tmp/subagent_prompt_<uuid>.md)",
    )
    parser.add_argument(
        "--instructions",
        default=None,
        help="Optional additional task instructions",
    )

    args = parser.parse_args()

    try:
        payload_path = dispatch_subagent(
            skill=args.skill,
            target=args.target,
            role=args.role,
            subagent_type=args.subagent_type,
            classification=args.classification,
            output=args.output,
            instructions=args.instructions,
        )

        with open(payload_path, "r", encoding="utf-8") as f:
            content = f.read()

        print(f"Generated subagent prompt payload written to: {payload_path}\n")
        print(content)
        sys.exit(0)

    except Exception as exc:
        print(f"Error dispatching subagent: {exc}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
