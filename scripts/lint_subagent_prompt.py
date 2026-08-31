#!/usr/bin/env python3
"""
Subagent Prompt Payload Linter

Validates agent prompt text against canonical DEAP-spec-core invariants:
1. Mandatory view_file directive on SKILL.md before running actions (as step 1 / prerequisite).
2. Mandatory single-item micro-task scope (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case).
3. Mandatory defect filing directive supporting both 'gh issue create' and 'glab issue create'.
4. Mandatory 'PROCEED' authorization token.
5. Zero truncation/summarization markers and zero forbidden issue close commands.

Usage:
    python3 scripts/lint_subagent_prompt.py <file_or_string>
    python3 scripts/lint_subagent_prompt.py --file prompt.txt
"""

import sys
import os
import re
import argparse
from typing import List


def lint_subagent_prompt(prompt_text: str) -> List[str]:
    """
    Validates a subagent prompt payload and returns a list of violation error messages.
    Returns an empty list if the prompt meets all requirements.

    Requirements:
    a) Mandatory `view_file` directive on `SKILL.md` before running actions.
    b) Mandatory single-item micro-task scope (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case).
    c) Mandatory defect filing directive supporting both `gh issue create` and `glab issue create`.
    d) Mandatory `PROCEED` authorization token.
    e) Untruncated payload (no `[...]`, `[summarized]`, etc.).
    f) Zero forbidden issue closures (`gh issue close`, `glab issue close`).
    """
    errors: List[str] = []

    if not prompt_text or not isinstance(prompt_text, str) or not prompt_text.strip():
        errors.append("Prompt payload is empty or whitespace-only.")
        return errors

    # Check (a): Mandatory view_file directive on SKILL.md before running actions
    has_view_file = "view_file" in prompt_text
    has_skill_md = bool(re.search(r'\bSKILL\.md\b', prompt_text, re.IGNORECASE))
    step1_pattern = re.search(
        r'(?:step\s*1|very\s*first\s*step|first\s*step|as\s*its\s*very\s*first\s*step|before\s*(?:running|executing|any|proceeding|all)\s*(?:actions|tools|commands|steps|work)?|prerequisite|must\s*read).*view_file.*SKILL\.md|view_file.*SKILL\.md.*(?:step\s*1|very\s*first\s*step|first\s*step|as\s*its\s*very\s*first\s*step|before\s*(?:running|executing|any|proceeding|all)\s*(?:actions|tools|commands|steps|work)?|prerequisite|first)',
        prompt_text,
        re.IGNORECASE | re.DOTALL
    )

    if not (has_view_file and has_skill_md and step1_pattern):
        errors.append(
            "Prompt missing mandatory directive to execute 'view_file' on 'SKILL.md' before running actions or tools."
        )

    # Check (b): Mandatory single-item micro-task scope (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case)
    # Extract distinct identifiers
    raw_epics = re.findall(r'\b(?:EPIC|Epic)-[A-Za-z0-9_]+\b', prompt_text)
    raw_features = re.findall(r'\b(?:FEAT|FEATURE|Feat|Feature)-[A-Za-z0-9_]+\b', prompt_text)
    raw_stories = re.findall(r'\b(?:US|UserStory|Story)-[A-Za-z0-9_]+\b', prompt_text)
    raw_use_cases = re.findall(r'\b(?:UC|UseCase)-[A-Za-z0-9_]+\b', prompt_text)

    unique_epics = sorted(list({e.upper() for e in raw_epics}))
    unique_features = sorted(list({f.upper() for f in raw_features}))
    unique_stories = sorted(list({s.upper() for s in raw_stories}))
    unique_use_cases = sorted(list({u.upper() for u in raw_use_cases}))

    if len(unique_epics) > 1:
        errors.append(
            f"Prompt exceeds micro-task scope: multiple Epics detected ({unique_epics}). "
            f"Single-item micro-task scope is required (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case)."
        )
    if len(unique_features) > 1:
        errors.append(
            f"Prompt exceeds micro-task scope: multiple Features detected ({unique_features}). "
            f"Single-item micro-task scope is required (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case)."
        )
    if len(unique_stories) > 1:
        errors.append(
            f"Prompt exceeds micro-task scope: multiple User Stories detected ({unique_stories}). "
            f"Single-item micro-task scope is required (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case)."
        )
    if len(unique_use_cases) > 1:
        errors.append(
            f"Prompt exceeds micro-task scope: multiple Use Cases detected ({unique_use_cases}). "
            f"Single-item micro-task scope is required (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case)."
        )

    # Check for multi-item / batch scope phrasing
    batch_patterns = [
        (r'\ball\s+(?:epics|features|user\s*stories|use\s*cases)\b', "all epics/features/user stories/use cases"),
        (r'\bmultiple\s+(?:epics|features|user\s*stories|use\s*cases)\b', "multiple epics/features/user stories/use cases"),
        (r'\bbatch\s+(?:process|processing|execution|generation|mode)\b', "batch execution mode")
    ]
    for pat, desc in batch_patterns:
        if re.search(pat, prompt_text, re.IGNORECASE):
            errors.append(
                f"Prompt violates micro-task scope by specifying {desc}. Single-item micro-task scope is required."
            )

    # Check (c): Mandatory defect filing directive supporting both `gh issue create` and `glab issue create`
    has_gh_create = bool(re.search(r'\bgh\s+issue\s+create\b', prompt_text))
    has_glab_create = bool(re.search(r'\bglab\s+issue\s+create\b', prompt_text))

    if not (has_gh_create and has_glab_create):
        missing_tools = []
        if not has_gh_create:
            missing_tools.append("'gh issue create'")
        if not has_glab_create:
            missing_tools.append("'glab issue create'")
        errors.append(
            f"Prompt missing mandatory defect filing directive supporting both 'gh issue create' and 'glab issue create' (missing: {', '.join(missing_tools)})."
        )

    # Check (d): Mandatory `PROCEED` authorization token
    has_proceed = bool(re.search(r'\bPROCEED\b', prompt_text))
    if not has_proceed:
        errors.append("Prompt missing mandatory 'PROCEED' authorization token.")

    # Check (e): Truncation / summarization indicators
    truncation_patterns = [
        (r'\[\s*\.\.\.\s*\]', "elided ellipsis '[...]'"),
        (r'\[\s*summarized?\s*\]', "'[summarized]'"),
        (r'\[\s*truncated?\s*\]', "'[truncated]'"),
        (r'\bsummarized\s+(?:prompt|instructions?|directives?|payload)\b', "summarized prompt marker"),
        (r'\btruncated\s+(?:prompt|instructions?|directives?|payload)\b', "truncated prompt marker"),
    ]
    for pat, desc in truncation_patterns:
        if re.search(pat, prompt_text, re.IGNORECASE):
            errors.append(f"Prompt payload contains forbidden truncation/summarization indicator: {desc}.")

    # Check (f): Forbidden issue closure commands
    if re.search(r'\b(?:gh|glab)\s+issue\s+close\b', prompt_text, re.IGNORECASE):
        errors.append(
            "Prompt payload contains forbidden 'gh/glab issue close' directive (issue closure is reserved for Product Owner review)."
        )

    return errors


def main():
    parser = argparse.ArgumentParser(
        description="Subagent Prompt Payload Linter for DEAP-spec-core",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("input", nargs="?", default=None, help="Prompt text string or file path containing prompt payload")
    parser.add_argument("--file", "-f", dest="file_path", default=None, help="File path containing prompt payload")

    args = parser.parse_args()

    target_text = ""
    target_source = ""

    if args.file_path:
        target_source = args.file_path
        if not os.path.exists(args.file_path):
            print(f"Error: File not found: {args.file_path}", file=sys.stderr)
            sys.exit(1)
        with open(args.file_path, "r", encoding="utf-8") as f:
            target_text = f.read()
    elif args.input:
        if os.path.isfile(args.input):
            target_source = args.input
            with open(args.input, "r", encoding="utf-8") as f:
                target_text = f.read()
        else:
            target_source = "<inline_string>"
            target_text = args.input
    else:
        # Check standard input if piped
        if not sys.stdin.isatty():
            target_source = "<stdin>"
            target_text = sys.stdin.read()
        else:
            parser.print_help()
            sys.exit(1)

    errors = lint_subagent_prompt(target_text)

    if errors:
        print(f"Prompt linting FAILED for {target_source} with {len(errors)} violation(s):", file=sys.stderr)
        for idx, err in enumerate(errors, 1):
            print(f"  {idx}. {err}", file=sys.stderr)
        sys.exit(1)
    else:
        print(f"Prompt linting PASSED for {target_source}.")
        sys.exit(0)


if __name__ == "__main__":
    main()
