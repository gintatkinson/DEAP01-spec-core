#!/usr/bin/env python3
"""
Clean up Git hooks and whitelist pipeline infrastructure directories.
Removes pre-commit and pre-push hooks to prevent auto-triggered compiler runs.
Appends whitelist rules to .gitignore and stages pipeline directories.
"""

import argparse
import os
import subprocess
import sys

INFRA_DIRS = ["/skills", "/rules", "/.pipeline", "/.agents", "/scripts"]
WHITELIST_HEADER = "\n# Pipeline infrastructure (whitelisted by setup_git_hooks.py)\n"

STAGE_DIRS = [".pipeline/", "skills/", "rules/", "scripts/", ".agents/"]


def _purge_ds_store(repo_root):
    """Recursively locate and delete all .DS_Store files in the repository."""
    removed = 0
    for root, _, files in os.walk(repo_root):
        for f in files:
            if f == ".DS_Store":
                path = os.path.join(root, f)
                try:
                    os.remove(path)
                    removed += 1
                except OSError as e:
                    print(f"Warning: Failed to remove .DS_Store at {path}: {e}", file=sys.stderr)
    if removed:
        print(f"Purged {removed} .DS_Store file{'s' if removed != 1 else ''} from repository")


def _whitelist_infrastructure(repo_root):
    gitignore_path = os.path.join(repo_root, ".gitignore")
    if not os.path.isfile(gitignore_path):
        with open(gitignore_path, "w", encoding="utf-8") as f:
            pass
        print(f"Created missing .gitignore file at {gitignore_path}")

    git_dir = os.path.join(repo_root, ".git")
    if not os.path.isdir(git_dir) or not os.path.isfile(os.path.join(git_dir, "HEAD")):
        print("Warning: not a git repository — skipping whitelist modifications", file=sys.stderr)
        return

    with open(gitignore_path, "r", encoding="utf-8") as f:
        content = f.read()

    patterns = []
    for d in INFRA_DIRS:
        for suffix in ("/", "/**"):
            pattern = f"!{d}{suffix}"
            if pattern not in content:
                patterns.append(pattern)

    if patterns:
        with open(gitignore_path, "a", encoding="utf-8") as f:
            f.write(WHITELIST_HEADER)
            for p in patterns:
                f.write(f"{p}\n")
        print(f"Appended {len(patterns)} whitelist entr{'y' if len(patterns)==1 else 'ies'} to .gitignore")
    else:
        print("Infrastructure whitelist entries already present in .gitignore")

    result = subprocess.run(
        ["git", "add"] + STAGE_DIRS,
        capture_output=True, cwd=repo_root
    )
    if result.returncode != 0:
        print(f"Error: git add failed: {result.stderr.decode().strip()}", file=sys.stderr)
        sys.exit(1)
    print("Staged pipeline infrastructure directories")


def setup_git_hooks():
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.dirname(script_dir)
    git_dir = os.path.join(repo_root, ".git")
    
    if not os.path.isdir(git_dir):
        print(f"Error: .git directory not found at {git_dir}", file=sys.stderr)
        sys.exit(1)

    _purge_ds_store(repo_root)
        
    hooks_dir = os.path.join(git_dir, "hooks")
    errored = False

    pre_commit_path = os.path.join(hooks_dir, "pre-commit")
    pre_commit_script = (
        "#!/bin/sh\n"
        "# Pre-commit hook: Subagent Output Integrity & Escape Tokens Gate (Mechanism 3 & 4)\n"
        "python3 scripts/verify_subagent_output.py --dir docs\n"
    )
    try:
        with open(pre_commit_path, "w", encoding="utf-8") as f:
            f.write(pre_commit_script)
        os.chmod(pre_commit_path, 0o755)
        print(f"Successfully installed Git pre-commit hook: {pre_commit_path}")
    except Exception as e:
        print(f"Error setting pre-commit hook: {e}", file=sys.stderr)
        errored = True

    pre_push_path = os.path.join(hooks_dir, "pre-push")
    if os.path.exists(pre_push_path):
        try:
            os.remove(pre_push_path)
            print(f"Successfully removed pre-push hook: {pre_push_path}")
        except Exception as e:
            print(f"Error removing Git hook {pre_push_path}: {e}", file=sys.stderr)
            errored = True

    if errored:
        sys.exit(1)

    _whitelist_infrastructure(repo_root)


def main():
    parser = argparse.ArgumentParser(
        description="Clean up Git hooks and whitelist pipeline infrastructure directories."
    )
    parser.add_argument(
        "--install",
        action="store_true",
        help="Install git hooks and whitelist infrastructure (default action)",
    )
    parser.parse_args()

    setup_git_hooks()


if __name__ == "__main__":
    main()

