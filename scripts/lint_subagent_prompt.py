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


def check_step1_skill_directive(prompt_text: str) -> bool:
    """
    Semantically verifies that the prompt instructs executing `view_file` on `SKILL.md`
    as step 1 / first action / prerequisite before performing actions or tools.
    Supports flexible phrasing, ordering, markdown formatting, and whitespace.
    """
    if not prompt_text or not isinstance(prompt_text, str):
        return False
    has_view_file = bool(re.search(r'\bview_file\b', prompt_text))
    has_skill_md = bool(re.search(r'\bSKILL\.md\b', prompt_text, re.IGNORECASE))
    if not (has_view_file and has_skill_md):
        return False

    step_indicator_patterns = [
        r'\b(?:step\s*1|very\s*first\s*step|first\s*step|first\s*action)\b',
        r'\bas\s+(?:its\s+|your\s+)?(?:very\s+)?first\s+(?:step|action)\b',
        r'\bas\s+step\s*1\b',
        r'\b(?:prerequisite|must\s*read)\b',
        r'\b(?:before|prior\s+to)\s+(?:taking|running|executing|performing|proceeding|all|any)\b',
    ]

    for pattern in step_indicator_patterns:
        if re.search(f"{pattern}.*?view_file.*?SKILL\\.md", prompt_text, re.IGNORECASE | re.DOTALL):
            return True
        if re.search(f"{pattern}.*?SKILL\\.md.*?view_file", prompt_text, re.IGNORECASE | re.DOTALL):
            return True
        if re.search(f"view_file.*?{pattern}.*?SKILL\\.md", prompt_text, re.IGNORECASE | re.DOTALL):
            return True
        if re.search(f"view_file.*?SKILL\\.md.*?{pattern}", prompt_text, re.IGNORECASE | re.DOTALL):
            return True
        if re.search(f"SKILL\\.md.*?view_file.*?{pattern}", prompt_text, re.IGNORECASE | re.DOTALL):
            return True
        if re.search(f"SKILL\\.md.*?{pattern}.*?view_file", prompt_text, re.IGNORECASE | re.DOTALL):
            return True

    return False


def check_repository_classification(prompt_text: str) -> bool:
    """
    Semantically verifies that the prompt contains a valid repository classification
    indicator matching workspace lineage / configuration or standard role classifications.
    Supports markdown bolding, backticks, quotes, and hyphens.
    """
    if not prompt_text or not isinstance(prompt_text, str):
        return False

    # Check known classification keywords anywhere in prompt text
    known_classifications = [
        "UPSTREAM_SPEC_CORE_COMPILER",
        "PARENT_DOMAIN_DISTRIBUTION_TEMPLATE",
        "CHILD_DOMAIN_DISTRIBUTION_TEMPLATE",
        "DOWNSTREAM_CUSTOMER_PROJECT",
        "DOWNSTREAM_APPLICATION_WORKSPACE",
        "DOWNSTREAM_CUSTOMER_WORKSPACE",
        "DOWNSTREAM_PROJECT",
        "DOWNSTREAM_APPLICATION",
        "DOWNSTREAM_WORKSPACE",
        "LEAF_WORKSPACE",
        "LEAF_CUSTOMER_WORKSPACE",
        "CUSTOMER_APPLICATION_WORKSPACE",
        "CUSTOMER_PROJECT",
        "DOMAIN_PARENT",
        "DOMAIN_CHILD",
        "REPO_CLASSIFICATION",
        "REPOSITORY_CLASSIFICATION",
    ]
    for kc in known_classifications:
        if kc in prompt_text:
            return True

    # Header / explicit field declaration patterns (evaluated line-by-line / same-line)
    field_patterns = [
        r'^[ \t]*(?:\*{1,2}|_{1,2}|`|#+\s*)?(?:Repository\s*Classification|Repo\s*Classification|Classification)(?:\*{1,2}|_{1,2}|`|:)*[ \t]*[:=\-]?[ \t]*(?:\*{1,2}|_{1,2}|`)*[ \t]*[`"\']?([A-Za-z0-9_\-\.]+)[`"\']?',
        r'\boperating\s+within\s+classification[ \t]*[:=\-]?[ \t]*[`"\']?([A-Za-z0-9_\-\.]+)[`"\']?',
        r'\bwithin\s+classification[ \t]*[:=\-]?[ \t]*[`"\']?([A-Za-z0-9_\-\.]+)[`"\']?',
        r'\bclassification[ \t]*[:=\-][ \t]*[`"\']?([A-Za-z0-9_\-\.]+)[`"\']?',
    ]
    for pattern in field_patterns:
        match = re.search(pattern, prompt_text, re.IGNORECASE | re.MULTILINE)
        if match:
            val = match.group(1)
            if val and len(val.strip()) > 0:
                return True

    return False


def check_leading_code_steering(prompt_text: str) -> bool:
    """
    Returns True if leading line-level code steering is detected before the Step 1 skill instruction.
    Returns False if clean (no leading steering).
    """
    if not prompt_text or not isinstance(prompt_text, str):
        return False

    view_file_pos = prompt_text.find("view_file")
    if view_file_pos == -1:
        return False

    preceding_text = prompt_text[:view_file_pos]
    leading_steering_patterns = [
        r'\b(?:replace|edit|modify|delete|change|patch|update)\s+lines?\s+\d+',
        r'\b(?:replace|edit|modify|patch|update)\s+line\s+\d+\s+with\b',
        r'\bapply\s+the\s+following\s+(?:diff|patch|code|changes?)\b',
        r'```diff\b',
        r'\bhere\s+is\s+the\s+(?:code|fix|patch|diff)\s+to\s+apply\b',
        r'\bjust\s+(?:change|replace|edit|modify|delete)\b',
        r'\bpaste\s+this\s+(?:code|diff|patch)\b',
    ]
    for pat in leading_steering_patterns:
        if re.search(pat, preceding_text, re.IGNORECASE):
            return True
    return False


def mask_mandate_text(prompt_text: str) -> str:
    """
    Masks out pre-flight checklist text and rule quotations so that verbatim
    or slightly formatted mandate text is never flagged as a truncation indicator.
    """
    if not prompt_text:
        return ""
    scan_text = re.sub(r"\s+", " ", prompt_text).strip()
    requirements, _err = load_mandate_requirements()
    if requirements:
        for req_text in requirements:
            norm_req = re.sub(r"\s+", " ", req_text).strip()
            scan_text = scan_text.replace(norm_req, " ")
            unquoted_req = re.sub(r"[`'\"]", "", norm_req)
            scan_text = scan_text.replace(unquoted_req, " ")

    scan_text = re.sub(
        r'Zero\s+`?\[\.\.\.\]`?,\s*`?\[summarized\]`?,\s*or\s*`?\[truncated\]`?\s*markers',
        " ",
        scan_text,
        flags=re.IGNORECASE,
    )
    scan_text = re.sub(
        r'Zero\s+\[\.\.\.\],\s*\[summarized\],\s*or\s*\[truncated\]\s*markers',
        " ",
        scan_text,
        flags=re.IGNORECASE,
    )
    return scan_text


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
    if not check_step1_skill_directive(prompt_text):
        errors.append(
            "Prompt missing mandatory directive to execute 'view_file' on 'SKILL.md' before running actions or tools."
        )

    # Check (b): Mandatory single-item micro-task scope (max 1 Epic, 1 Feature, 1 User Story, or 1 Use Case)
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
    truncation_scan_text = mask_mandate_text(prompt_text)
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
    truncation_scan_text = mask_mandate_text(prompt_text)
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
    if not check_repository_classification(prompt_text):
        return False, "ERROR: Prompt rejected: missing repository classification"

    # Check for view_file on SKILL.md
    has_view_file = bool(re.search(r'\bview_file\b', prompt_text))
    has_skill_md = bool(re.search(r'\bSKILL\.md\b', prompt_text, re.IGNORECASE))
    if not (has_view_file and has_skill_md):
        return False, "ERROR: Prompt rejected: missing view_file directive on SKILL.md"

    # Check that view_file on SKILL.md is instructed as step 1 / first action
    if not check_step1_skill_directive(prompt_text):
        return False, "ERROR: Prompt rejected: missing step 1 directive for view_file on SKILL.md"

    # Check for leading line-level steering before the view_file on SKILL.md directive
    if check_leading_code_steering(prompt_text):
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
