#!/usr/bin/env python3
"""
Tracker configuration updates in codebase_rules.json and .gitlab-ci.yml, and tracker label bootstrapping.
/// Realises: [InstallerTrackerManager]
"""
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Optional

from scripts.installer.metadata import InstallOptions


def configure_tracker_rules(
    staging_dir: Path,
    options: InstallOptions,
) -> None:
    """
    Updates 'tracker_rules' inside:
    - staging_dir/.pipeline/logical-ui/codebase_rules.json
    - staging_dir/codebase_rules.json (if present)
    """
    rule_files = [
        staging_dir / ".pipeline" / "logical-ui" / "codebase_rules.json",
        staging_dir / "codebase_rules.json",
    ]

    for rules_file in rule_files:
        if not rules_file.is_file():
            continue

        try:
            with open(rules_file, "r", encoding="utf-8") as f:
                data = json.load(f)
        except Exception:
            continue

        if "tracker_rules" not in data or not isinstance(data["tracker_rules"], dict):
            data["tracker_rules"] = {}

        tracker_rules = data["tracker_rules"]

        if options.provider == "gitlab" or options.gitlab_group or options.gitlab_url != "https://gitlab.com":
            if options.provider != "auto":
                tracker_rules["provider"] = options.provider
            if options.provider == "gitlab":
                tracker_rules["labels"] = {
                    "epic": "type::epic",
                    "feature": "type::feature",
                    "user_story": "type::user-story",
                    "use_case": "type::use-case",
                    "ready_for_review": "status::ready-for-review",
                    "resolved": "status::fixed-resolved",
                }
            if options.gitlab_url:
                tracker_rules["server_url"] = options.gitlab_url
            if options.gitlab_group:
                tracker_rules["group"] = options.gitlab_group

        elif (
            options.provider == "jira"
            or options.jira_project
            or options.jira_email
            or options.jira_url != "https://your-domain.atlassian.net"
        ):
            if options.provider != "auto":
                tracker_rules["provider"] = options.provider
            if options.provider == "jira":
                tracker_rules["numeric_prefix"] = ""
                tracker_rules["alphanumeric_prefix"] = ""
                tracker_rules["keys"] = {
                    "issue_id": "key",
                    "title": "title",
                    "labels": "labels",
                    "state": "state",
                    "closed_state_value": "CLOSED",
                    "open_state_value": "OPEN",
                }
                tracker_rules["labels"] = {
                    "epic": "type::epic",
                    "feature": "type::feature",
                    "user_story": "type::user-story",
                    "use_case": "type::use-case",
                    "ready_for_review": "status::ready-for-review",
                    "resolved": "status::fixed-resolved",
                }
            if options.jira_url:
                tracker_rules["server_url"] = options.jira_url
            if options.jira_project:
                tracker_rules["project_key"] = options.jira_project
            if options.jira_email:
                tracker_rules["email"] = options.jira_email

        elif options.provider == "github":
            tracker_rules["provider"] = "github"
            if options.github_org:
                tracker_rules["owner"] = options.github_org

        try:
            with open(rules_file, "w", encoding="utf-8") as f:
                json.dump(data, f, indent=2)
        except Exception as e:
            sys.stderr.write(f"Warning: Failed to write {rules_file}: {e}\n")


def deploy_gitlab_ci_template(
    installer_root: Path,
    staging_dir: Path,
    options: InstallOptions,
) -> None:
    """
    Deploys .gitlab-ci.yml from .pipeline/templates/.gitlab-ci.yml or .pipeline/.gitlab-ci.yml
    to staging_dir/.gitlab-ci.yml when provider is GitLab.
    """
    if options.provider == "gitlab" or options.gitlab_group or options.gitlab_url != "https://gitlab.com":
        candidates = [
            installer_root / ".pipeline" / "templates" / ".gitlab-ci.yml",
            installer_root / ".pipeline" / ".gitlab-ci.yml",
            staging_dir / ".pipeline" / "templates" / ".gitlab-ci.yml",
            staging_dir / ".pipeline" / ".gitlab-ci.yml",
        ]
        dest = staging_dir / ".gitlab-ci.yml"
        for cand in candidates:
            if cand.is_file():
                dest.parent.mkdir(parents=True, exist_ok=True)
                try:
                    shutil.copy2(cand, dest)
                except Exception:
                    pass
                break


def bootstrap_tracker_labels(target_dir: Path) -> None:
    """
    Executes skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py via list-style subprocess.
    """
    script_path = target_dir / "skills" / "spec-orchestrator" / "scripts" / "bootstrap_tracker_labels.py"
    if script_path.is_file():
        print("Bootstrapping issue tracker label taxonomy...")
        res = subprocess.run(
            [sys.executable, str(script_path)],
            cwd=str(target_dir),
            check=False,
            shell=False,
        )
        if res.returncode != 0:
            print("Note: Tracker labels could not be provisioned automatically (e.g. offline or unauthenticated).")
            print("You can re-run label provisioning anytime: python3 skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py")
