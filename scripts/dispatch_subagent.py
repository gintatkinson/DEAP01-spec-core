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
import os
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


DEFAULT_CLASSIFICATION = "UPSTREAM_SPEC_CORE_COMPILER"
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


def construct_prompt_template(
    skill_path: str,
    target: str,
    role: str = DEFAULT_ROLE,
    subagent_type: str = DEFAULT_TYPE,
    classification: str = DEFAULT_CLASSIFICATION,
    instructions: Optional[str] = None,
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
        classification: Repository classification.
        instructions: Optional additional instructions.

    Returns:
        The generated prompt text.
    """
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
Repository Classification: {classification}
Target: {target}

Mandatory Instructions:
1. Step 1: Execute `view_file` on `{skill_path}` as your very first step before executing any file edits, commands, or tools. Strictly follow its instruction guidelines and formatting templates.
2. Repository Scope: You are operating within classification `{classification}`. Maintain all repository invariants and domain boundaries.
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
    classification: str = DEFAULT_CLASSIFICATION,
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
        classification: Repository classification.
        instructions: Optional custom instructions.
        base_dir: Optional base directory for skill resolution.

    Returns:
        The validated prompt text.

    Raises:
        ValueError: If input validation or prompt linting fails.
        FileNotFoundError: If skill file does not exist.
    """
    if not target or not isinstance(target, str) or not target.strip():
        raise ValueError("Target path or specification item must be a non-empty string.")

    valid_skill_path = validate_skill_path(skill, base_dir=base_dir)

    prompt_text = construct_prompt_template(
        skill_path=valid_skill_path,
        target=target.strip(),
        role=role.strip() if role else DEFAULT_ROLE,
        subagent_type=subagent_type.strip() if subagent_type else DEFAULT_TYPE,
        classification=classification.strip() if classification else DEFAULT_CLASSIFICATION,
        instructions=instructions,
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
    classification: str = DEFAULT_CLASSIFICATION,
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
        classification: Repository classification.
        output: Optional path to write payload file.
        instructions: Optional custom instructions.
        base_dir: Optional base directory for skill resolution.

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
        default=DEFAULT_CLASSIFICATION,
        help=f"Repository classification (default: '{DEFAULT_CLASSIFICATION}')",
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
