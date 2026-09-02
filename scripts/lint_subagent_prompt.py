#!/usr/bin/env python3
"""
Subagent Prompt Payload Linter

Validates agent prompt text against canonical DEAP01-spec-core invariants:
1. Mandatory view_file directive on SKILL.md before running actions (as step 1 / prerequisite).
2. Mandatory single-item micro-task scope (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case).
3. Mandatory defect filing directive supporting both 'gh issue create' and 'glab issue create'.
4. Mandatory 'PROCEED' authorization token.
5. Zero truncation/summarization markers and zero forbidden issue close commands.
6. Corpus-sourced mandate fidelity (six pre-flight Requirements verbatim from
   rules/subagent-dispatch-standards.md, coverage 100 percent).

Usage:
    python3 scripts/lint_subagent_prompt.py <file_or_string>
    python3 scripts/lint_subagent_prompt.py --file prompt.txt
"""

import sys
import os
import re
import argparse
from typing import List


SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
SUBDAGENT_DISPATCH_STANDARDS_PATH = os.path.join(
    PROJECT_ROOT, "rules", "subagent-dispatch-standards.md"
)


def load_mandate_requirements(rules_path: str = SUBDAGENT_DISPATCH_STANDARDS_PATH):
    """
    Parse the six pre-flight checklist Requirement cells verbatim from the rules corpus.

    Returns:
        (requirements, error) where requirements is the list of verbatim Requirement
        strings, or None with a fail-closed error message when the corpus file is
        unreadable or contains no parseable Requirements at gate time.
    """
    try:
        with open(rules_path, "r", encoding="utf-8") as f:
            rules_text = f.read()
    except OSError as exc:
        return None, f"mandatory rules corpus unreadable at gate time: {rules_path} ({exc})"
    requirements = []
    for line in rules_text.splitlines():
        if re.match(r"^\|\s*\d+\.\s", line):
            cells = [c.strip() for c in line.split("|")]
            if len(cells) > 3 and cells[2]:
                requirements.append(cells[2])
    if not requirements:
        return None, f"mandatory rules corpus contains no parseable pre-flight Requirements: {rules_path}"
    return requirements, None


def check_mandate_fidelity(prompt_text: str, rules_path: str = SUBDAGENT_DISPATCH_STANDARDS_PATH):
    """
    Corpus-sourced mandate fidelity gate.

    Runtime-parses the six pre-flight Requirements verbatim from
    rules/subagent-dispatch-standards.md and computes coverage = present/total
    over the whitespace-normalized payload. Fails closed when the corpus file
    cannot be read at gate time.

    Returns:
        (ok, coverage, missing, error): ok is False when any Requirement is
        absent from the payload or when the corpus is unreadable.
    """
    if not prompt_text or not isinstance(prompt_text, str) or not prompt_text.strip():
        return False, 0.0, [], "prompt payload is empty or whitespace-only"
    requirements, gate_error = load_mandate_requirements(rules_path)
    if gate_error:
        return False, 0.0, [], gate_error
    normalized_prompt = re.sub(r"\s+", " ", prompt_text).strip()
    missing = [
        req for req in requirements
        if re.sub(r"\s+", " ", req).strip() not in normalized_prompt
    ]
    coverage = (len(requirements) - len(missing)) / len(requirements)
    if missing:
        return False, coverage, missing, None
    return True, coverage, [], None


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
    g) Corpus-sourced mandate fidelity (all six pre-flight Requirements verbatim
       from rules/subagent-dispatch-standards.md, coverage 100 percent).
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

    # Check (e): Truncation / summarization indicators.
    # The corpus-quoted pre-flight Requirement rows are masked out of the scan so
    # that verbatim mandate text (e.g. "Zero `[...]`, `[summarized]`, or
    # `[truncated]` markers") is never itself read as a truncation indicator.
    truncation_scan_text = re.sub(r"\s+", " ", prompt_text).strip()
    requirements, _mandate_corpus_error = load_mandate_requirements()
    if requirements:
        for req_text in requirements:
            truncation_scan_text = truncation_scan_text.replace(
                re.sub(r"\s+", " ", req_text).strip(), " "
            )
    truncation_patterns = [
        (r'\[\s*\.\.\.\s*\]', "elided ellipsis '[...]'"),
        (r'\[\s*summarized?\s*\]', "'[summarized]'"),
        (r'\[\s*truncated?\s*\]', "'[truncated]'"),
        (r'\bsummarized\s+(?:prompt|instructions?|directives?|payload)\b', "summarized prompt marker"),
        (r'\btruncated\s+(?:prompt|instructions?|directives?|payload)\b', "truncated prompt marker"),
    ]
    for pat, desc in truncation_patterns:
        if re.search(pat, truncation_scan_text, re.IGNORECASE):
            errors.append(f"Prompt payload contains forbidden truncation/summarization indicator: {desc}.")

    # Check (f): Forbidden issue closure commands
    if re.search(r'\b(?:gh|glab)\s+issue\s+close\b', prompt_text, re.IGNORECASE):
        errors.append(
            "Prompt payload contains forbidden 'gh/glab issue close' directive (issue closure is reserved for Product Owner review)."
        )

    # Check (g): Corpus-sourced mandate fidelity (rules/subagent-dispatch-standards.md).
    # Every one of the six pre-flight Requirements must be present verbatim
    # (whitespace-normalized). Coverage below 100 percent fails, naming the missing
    # Requirements. An unreadable corpus at gate time fails closed.
    fidelity_ok, fidelity_coverage, fidelity_missing, fidelity_error = check_mandate_fidelity(prompt_text)
    if fidelity_error:
        errors.append(f"Mandate fidelity gate failed closed: {fidelity_error}")
    elif not fidelity_ok:
        for req in fidelity_missing:
            errors.append(
                f"Mandate fidelity violation: missing mandatory Requirement verbatim from "
                f"rules/subagent-dispatch-standards.md: '{req}' (coverage {fidelity_coverage:.0%})"
            )

    return errors


def validate_subagent_preflight(prompt_text: str) -> tuple[bool, str]:
    """
    Executes the Subagent Self-Rejection Pre-Flight Gate check.

    Verifies that:
    1. Prompt payload is non-empty.
    2. Prompt contains zero truncation / summarization markers ('[...]', '[summarized]', '[truncated]').
    3. Prompt contains zero forbidden issue closure commands ('gh issue close', 'glab issue close').
    4. Prompt declares the repository classification (e.g. 'UPSTREAM_SPEC_CORE_COMPILER' or 'Repository Classification:').
    5. Prompt begins with the instruction to execute `view_file` on `SKILL.md` by exact path as step 1 / first action, before any line-level code steering.
    6. Prompt contains zero leading line-level steering or premature edits ahead of the step 1 skill read directive.
    7. Prompt respects single-item micro-task scope.

    Returns:
        (True, "") if prompt passes pre-flight gate.
        (False, "ERROR: Prompt rejected: <reason>") if prompt is rejected.
    """
    if not prompt_text or not isinstance(prompt_text, str) or not prompt_text.strip():
        return False, "ERROR: Prompt rejected: prompt payload is empty or whitespace-only"

    # Check for truncation/summarization markers.
    # The corpus-quoted pre-flight Requirement rows are masked out of the scan so
    # that verbatim mandate text (e.g. "Zero `[...]`, `[summarized]`, or
    # `[truncated]` markers") is never itself read as a truncation indicator.
    truncation_scan_text = re.sub(r"\s+", " ", prompt_text).strip()
    requirements, _mandate_corpus_error = load_mandate_requirements()
    if requirements:
        for req_text in requirements:
            truncation_scan_text = truncation_scan_text.replace(
                re.sub(r"\s+", " ", req_text).strip(), " "
            )
    truncation_patterns = [
        (r'\[\s*\.\.\.\s*\]', "elided ellipsis '[...]'"),
        (r'\[\s*summarized?\s*\]', "'[summarized]'"),
        (r'\[\s*truncated?\s*\]', "'[truncated]'"),
        (r'\bsummarized\s+(?:prompt|instructions?|directives?|payload)\b', "summarized prompt marker"),
        (r'\btruncated\s+(?:prompt|instructions?|directives?|payload)\b', "truncated prompt marker"),
    ]
    for pat, desc in truncation_patterns:
        if re.search(pat, truncation_scan_text, re.IGNORECASE):
            return False, f"ERROR: Prompt rejected: prompt truncation/summarization detected ({desc})"

    # Check for forbidden issue closure commands
    if re.search(r'\b(?:gh|glab)\s+issue\s+close\b', prompt_text, re.IGNORECASE):
        return False, "ERROR: Prompt rejected: forbidden issue closure command"

    # Check for repository classification
    has_classification = bool(
        re.search(r'(?:Repository\s*Classification|Classification)\s*:\s*[A-Za-z0-9_]+', prompt_text, re.IGNORECASE)
        or "UPSTREAM_SPEC_CORE_COMPILER" in prompt_text
        or "REPO_CLASSIFICATION" in prompt_text
    )
    if not has_classification:
        return False, "ERROR: Prompt rejected: missing repository classification"

    # Check for view_file on SKILL.md
    has_view_file = "view_file" in prompt_text
    has_skill_md = bool(re.search(r'\bSKILL\.md\b', prompt_text, re.IGNORECASE))
    if not (has_view_file and has_skill_md):
        return False, "ERROR: Prompt rejected: missing view_file directive on SKILL.md"

    # Check that view_file on SKILL.md is instructed as step 1 / first action
    step1_pattern = re.search(
        r'(?:step\s*1|very\s*first\s*step|first\s*step|as\s*its\s*very\s*first\s*step|before\s*(?:running|executing|any|proceeding|all)\s*(?:actions|tools|commands|steps|work)?|prerequisite|must\s*read).*view_file.*SKILL\.md|view_file.*SKILL\.md.*(?:step\s*1|very\s*first\s*step|first\s*step|as\s*its\s*very\s*first\s*step|before\s*(?:running|executing|any|proceeding|all)\s*(?:actions|tools|commands|steps|work)?|prerequisite|first)',
        prompt_text,
        re.IGNORECASE | re.DOTALL
    )
    if not step1_pattern:
        return False, "ERROR: Prompt rejected: missing step 1 directive for view_file on SKILL.md"

    # Check for leading line-level steering before the view_file on SKILL.md directive
    view_file_pos = prompt_text.find("view_file")
    preceding_text = prompt_text[:view_file_pos]
    leading_steering_patterns = [
        r'\b(?:replace|edit|modify|delete|change)\s+lines?\s+\d+',
        r'\b(?:replace|edit|modify)\s+line\s+\d+\s+with\b',
        r'\bapply\s+the\s+following\s+diff\b',
        r'```diff\b',
        r'\bhere\s+is\s+the\s+(?:code|fix|patch)\s+to\s+apply\b',
        r'\bjust\s+(?:change|replace|edit)\b',
    ]
    for pat in leading_steering_patterns:
        if re.search(pat, preceding_text, re.IGNORECASE):
            return False, "ERROR: Prompt rejected: leading line-level steering detected"

    # Check single-item micro-task scope
    raw_epics = re.findall(r'\b(?:EPIC|Epic)-[A-Za-z0-9_]+\b', prompt_text)
    raw_features = re.findall(r'\b(?:FEAT|FEATURE|Feat|Feature)-[A-Za-z0-9_]+\b', prompt_text)
    raw_stories = re.findall(r'\b(?:US|UserStory|Story)-[A-Za-z0-9_]+\b', prompt_text)
    raw_use_cases = re.findall(r'\b(?:UC|UseCase)-[A-Za-z0-9_]+\b', prompt_text)

    unique_epics = set(e.upper() for e in raw_epics)
    unique_features = set(f.upper() for f in raw_features)
    unique_stories = set(s.upper() for s in raw_stories)
    unique_use_cases = set(u.upper() for u in raw_use_cases)

    if len(unique_epics) > 1 or len(unique_features) > 1 or len(unique_stories) > 1 or len(unique_use_cases) > 1:
        return False, "ERROR: Prompt rejected: micro-task scope violation"

    batch_patterns = [
        r'\ball\s+(?:epics|features|user\s*stories|use\s*cases)\b',
        r'\bmultiple\s+(?:epics|features|user\s*stories|use\s*cases)\b',
        r'\bbatch\s+(?:process|processing|execution|generation|mode)\b',
    ]
    for pat in batch_patterns:
        if re.search(pat, prompt_text, re.IGNORECASE):
            return False, "ERROR: Prompt rejected: micro-task scope violation"

    return True, ""


lint_prompt_text = lint_subagent_prompt



def main():
    parser = argparse.ArgumentParser(
        description="Subagent Prompt Payload Linter for DEAP01-spec-core",
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
