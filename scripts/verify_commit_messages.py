#!/usr/bin/env python3
"""
Mechanical Commit Message Non-Closure Linter & Gate

Enforces the Commit Message Non-Closure Invariant (.pipeline/constitution.md:266):
"Agents and automated scripts are strictly prohibited from using issue auto-closing keywords
(fix, fixes, fixed, close, closes, closed, resolve, resolves, resolved preceding #<id>) in git
commit messages. All commit messages referencing issues MUST use neutral citations: (#<id>)
or (refs #<id>) to prevent server-side premature auto-closure."
"""

import argparse
import os
import re
import subprocess
import sys

# Regex pattern matching GitHub/GitLab trigger keywords preceding an issue citation.
# Covers keywords like fix, fixes, fixed, close, closes, closed, resolve, resolves, resolved
# followed by optional colons/whitespace and #<id>.
TRIGGER_PATTERN = re.compile(
    r'\b(fix|fixes|fixed|close|closes|closed|resolve|resolves|resolved)(?::\s*|\s+)#\d+',
    re.IGNORECASE,
)


def verify_text(text: str) -> list[str]:
    """
    Check text for forbidden auto-closing issue patterns.
    Returns list of matched trigger strings found.
    """
    matches = []
    for match in TRIGGER_PATTERN.finditer(text):
        matches.append(match.group(0))
    return matches


def format_violation_error(matches: list[str], context: str = "", commit_msg: str = "") -> str:
    """
    Format a compliant failure diagnostic citing .pipeline/constitution.md:266.
    """
    lines = [
        "ERROR: Commit message violates the Commit Message Non-Closure Invariant (.pipeline/constitution.md:266).",
    ]
    if context:
        lines.append(f"Context: {context}")
    lines.append("Forbidden auto-closing trigger(s) detected:")
    for m in matches:
        lines.append(f"  - '{m}'")
    lines.append("")
    lines.append("Agents and automated scripts are strictly prohibited from using issue auto-closing keywords")
    lines.append("(fix, fixes, fixed, close, closes, closed, resolve, resolves, resolved preceding #<id>).")
    lines.append("All commit messages referencing issues MUST use neutral citations:")
    lines.append("  - '(refs #<id>)'")
    lines.append("  - '(#<id>)'")
    if commit_msg:
        lines.append("")
        lines.append("Offending message:")
        for line in commit_msg.strip().splitlines():
            lines.append(f"  > {line}")
    return "\n".join(lines)


def check_msg_file(file_path: str) -> int:
    """
    Verify commit message file (invoked by git commit-msg hook).
    """
    if not os.path.isfile(file_path):
        print(f"Error: Commit message file does not exist: {file_path}", file=sys.stderr)
        return 1
    try:
        with open(file_path, "r", encoding="utf-8", errors="replace") as f:
            content = f.read()
    except Exception as e:
        print(f"Error: Failed to read commit message file {file_path}: {e}", file=sys.stderr)
        return 1

    matches = verify_text(content)
    if matches:
        print(format_violation_error(matches, context=f"file {file_path}", commit_msg=content), file=sys.stderr)
        return 1
    return 0


def check_head() -> int:
    """
    Verify HEAD commit message using git log -1 --pretty=%B.
    """
    try:
        res = subprocess.run(
            ["git", "log", "-1", "--pretty=%B"],
            capture_output=True,
            text=True,
            check=False,
        )
    except Exception as e:
        print(f"Error: Failed to execute git log: {e}", file=sys.stderr)
        return 1

    if res.returncode != 0:
        print(f"Error: git log -1 failed: {res.stderr.strip()}", file=sys.stderr)
        return 1

    msg = res.stdout
    matches = verify_text(msg)
    if matches:
        print(format_violation_error(matches, context="HEAD commit", commit_msg=msg), file=sys.stderr)
        return 1
    return 0


def check_range(rev_range: str) -> int:
    """
    Verify commit messages across a git revision range (e.g. origin/main..HEAD).
    """
    try:
        res = subprocess.run(
            ["git", "log", "--format=%H%x00%B%x00", rev_range],
            capture_output=True,
            text=True,
            check=False,
        )
    except Exception as e:
        print(f"Error: Failed to execute git log for range '{rev_range}': {e}", file=sys.stderr)
        return 1

    if res.returncode != 0:
        print(f"Error: git log failed for range '{rev_range}': {res.stderr.strip()}", file=sys.stderr)
        return 1

    raw = res.stdout
    if not raw.strip():
        return 0

    parts = raw.split("\0")
    violations = 0
    idx = 0
    while idx + 1 < len(parts):
        commit_hash = parts[idx].strip()
        commit_msg = parts[idx + 1]
        idx += 2
        if not commit_hash:
            continue
        matches = verify_text(commit_msg)
        if matches:
            violations += 1
            print(
                format_violation_error(
                    matches,
                    context=f"commit {commit_hash[:10]} in range {rev_range}",
                    commit_msg=commit_msg,
                ),
                file=sys.stderr,
            )
            print("-" * 60, file=sys.stderr)

    return 1 if violations > 0 else 0


def check_text_arg(text: str) -> int:
    """
    Verify raw text argument directly.
    """
    matches = verify_text(text)
    if matches:
        print(format_violation_error(matches, context="raw text", commit_msg=text), file=sys.stderr)
        return 1
    return 0


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description="Verify commit messages adhere to the Commit Message Non-Closure Invariant (.pipeline/constitution.md:266)."
    )
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        "--msg-file",
        help="Path to commit message file to verify (for git commit-msg hook).",
    )
    group.add_argument(
        "--range",
        dest="rev_range",
        help="Git revision range to verify (e.g., origin/main..HEAD).",
    )
    group.add_argument(
        "--head",
        action="store_true",
        help="Verify the HEAD commit message.",
    )
    group.add_argument(
        "--check-text",
        help="Verify a raw commit message string.",
    )

    args = parser.parse_args(argv)

    if args.msg_file:
        return check_msg_file(args.msg_file)
    elif args.rev_range:
        return check_range(args.rev_range)
    elif args.head:
        return check_head()
    elif args.check_text is not None:
        return check_text_arg(args.check_text)

    return 0


if __name__ == "__main__":
    sys.exit(main())
