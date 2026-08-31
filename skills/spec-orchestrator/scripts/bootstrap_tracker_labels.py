#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""Provision the full tracker label taxonomy at install time.

Supports both GitHub (via GitHub CLI) and GitLab (via GitLab REST API v4).
Supports standard labels and GitLab scoped labels (type::*, status::*).

Usage:
    python3 skills/spec-orchestrator/scripts/bootstrap_tracker_labels.py [--provider github|gitlab] [--repo OWNER/NAME] [--project GROUP/PROJECT] [--gitlab-url URL] [--token TOKEN] [--dry-run] [--offline]
"""

import argparse
import json
import netrc
import os
import re
import shutil
import ssl
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from typing import Any, Dict, List, Optional, Tuple

# Presentation mapping (Color, Description).
# Supports both unscoped keys/labels and GitLab scoped labels (key::value).
LABEL_PRESENTATION: Dict[str, Tuple[str, str]] = {
    # Unscoped canonical keys
    "epic": ("800080", "Epic: a major functional domain or protocol module"),
    "feature": ("0366d6", "Feature: a single independently testable capability"),
    "user_story": ("0e8a16", "User Story: a BDD behavioural scenario"),
    "user-story": ("0e8a16", "User Story: a BDD behavioural scenario"),
    "use_case": ("fbca04", "Use Case: a formal UML system interaction"),
    "use-case": ("fbca04", "Use Case: a formal UML system interaction"),
    "ready_for_review": ("6f42c1", "Ready for review and validation"),
    "ready-for-review": ("6f42c1", "Ready for review and validation"),
    "resolved": (
        "0e8a16",
        "Dev complete, tests pass, merged to main. Awaiting Product Owner validation.",
    ),
    # GitHub label names
    "status:ready-for-review": ("6f42c1", "Ready for review and validation"),
    "status:fixed-resolved": (
        "0e8a16",
        "Dev complete, tests pass, merged to main. Awaiting Product Owner validation.",
    ),
    # GitLab scoped label names (type::* and status::*)
    "type::epic": ("800080", "Epic: a major functional domain or protocol module"),
    "type::feature": ("0366d6", "Feature: a single independently testable capability"),
    "type::user-story": ("0e8a16", "User Story: a BDD behavioural scenario"),
    "type::use-case": ("fbca04", "Use Case: a formal UML system interaction"),
    "status::ready-for-review": ("6f42c1", "Ready for review and validation"),
    "status::fixed-resolved": (
        "0e8a16",
        "Dev complete, tests pass, merged to main. Awaiting Product Owner validation.",
    ),
}

DEFAULT_GITHUB_LABELS: Dict[str, str] = {
    "epic": "epic",
    "feature": "feature",
    "user_story": "user-story",
    "use_case": "use-case",
    "resolved": "status:fixed-resolved",
}

DEFAULT_GITLAB_LABELS: Dict[str, str] = {
    "epic": "type::epic",
    "feature": "type::feature",
    "user_story": "type::user-story",
    "use_case": "type::use-case",
    "ready_for_review": "status::ready-for-review",
    "resolved": "status::fixed-resolved",
}


def normalize_color(color: Optional[str], with_hash: bool = False) -> str:
    """Normalize a color string to #RRGGBB (with_hash=True) or RRGGBB (with_hash=False)."""
    if not color:
        color = "0e8a16"
    clean = color.strip()
    if clean.startswith("#"):
        clean = clean[1:]
    if len(clean) not in (3, 6, 8):
        clean = "0e8a16"
    return f"#{clean.upper()}" if with_hash else clean.lower()


def get_label_presentation(key: str, name: str) -> Tuple[str, str]:
    """Retrieve color and description for a given label key or name."""
    if name in LABEL_PRESENTATION:
        return LABEL_PRESENTATION[name]
    if key in LABEL_PRESENTATION:
        return LABEL_PRESENTATION[key]
    if name.lower() in LABEL_PRESENTATION:
        return LABEL_PRESENTATION[name.lower()]
    if key.lower() in LABEL_PRESENTATION:
        return LABEL_PRESENTATION[key.lower()]
    
    # Normalized key lookup (e.g. user_story -> user-story)
    k_norm = key.replace("_", "-").lower()
    if k_norm in LABEL_PRESENTATION:
        return LABEL_PRESENTATION[k_norm]
    
    return ("ededed", str(name or key))


def parse_git_remote_url(remote_url: str) -> Dict[str, Any]:
    """
    Parse a git remote origin URL into its components:
    - raw: raw URL string
    - is_gitlab: True if domain contains 'gitlab'
    - project_path: repository path (e.g. 'gintatkinson/DEAP-spec-core' or 'group/subgroup/project')
    - server_url: base server URL (e.g. 'https://gitlab.com' or 'https://gitlab.internal.corp')
    - host: domain host name (e.g. 'gitlab.com' or 'github.com')
    """
    if not remote_url:
        return {"raw": "", "is_gitlab": False, "project_path": None, "server_url": None, "host": None}

    clean_url = remote_url.strip()
    if clean_url.endswith(".git"):
        clean_url = clean_url[:-4]

    # Check if HTTP(S) / SSH URL with scheme (e.g. https://gitlab.com/owner/repo or ssh://git@gitlab.com/owner/repo)
    if "://" in clean_url:
        parsed = urllib.parse.urlparse(clean_url)
        path = parsed.path.lstrip("/")
        netloc = parsed.netloc
        host = netloc.split("@")[-1].split(":")[0]
        scheme = parsed.scheme if parsed.scheme in ("http", "https") else "https"
        server_url = f"{scheme}://{netloc.split('@')[-1]}"
        is_gitlab = "gitlab" in host.lower()
        return {
            "raw": remote_url,
            "is_gitlab": is_gitlab,
            "project_path": path,
            "server_url": server_url,
            "host": host,
        }

    # Check if SCP-style SSH URL (e.g. git@gitlab.com:owner/repo or git@gitlab.internal.corp:group/sub/repo)
    scp_match = re.match(r"^(?:[^@]+@)?([^:]+):(.+)$", clean_url)
    if scp_match:
        host = scp_match.group(1)
        path = scp_match.group(2).lstrip("/")
        is_gitlab = "gitlab" in host.lower()
        server_url = f"https://{host}"
        return {
            "raw": remote_url,
            "is_gitlab": is_gitlab,
            "project_path": path,
            "server_url": server_url,
            "host": host,
        }

    # Fallback parsing
    parts = clean_url.split("/")
    project_path = f"{parts[-2]}/{parts[-1]}" if len(parts) >= 2 else clean_url
    is_gitlab = "gitlab" in clean_url.lower()
    return {
        "raw": remote_url,
        "is_gitlab": is_gitlab,
        "project_path": project_path,
        "server_url": "https://gitlab.com" if is_gitlab else "https://github.com",
        "host": "gitlab.com" if is_gitlab else "github.com",
    }


def get_git_remote_info(workspace_dir: str) -> Optional[Dict[str, Any]]:
    """Retrieve and parse git remote URL for origin."""
    try:
        res = subprocess.run(
            ["git", "remote", "get-url", "origin"],
            cwd=workspace_dir,
            capture_output=True,
            text=True,
            check=True,
            timeout=10,
        )
        url = res.stdout.strip()
        return parse_git_remote_url(url)
    except Exception:
        return None


def find_workspace_dir(start: Optional[str] = None) -> str:
    """Walk up until a workspace configuration or root is found."""
    if not start:
        start = os.path.dirname(os.path.abspath(__file__))
    current = os.path.abspath(start)
    while True:
        if (
            os.path.exists(os.path.join(current, ".pipeline", "logical-ui", "codebase_rules.json"))
            or os.path.exists(os.path.join(current, ".pipeline", "codebase_rules.json"))
            or os.path.exists(os.path.join(current, "codebase_rules.json"))
            or os.path.exists(os.path.join(current, ".git"))
        ):
            return current
        parent = os.path.dirname(current)
        if parent == current:
            return os.path.abspath(start)
        current = parent


def resolve_codebase_rules_path(workspace_dir: str) -> Optional[str]:
    """
    Check candidate paths for codebase_rules.json in order of priority:
    1. CODEBASE_RULES_PATH env var
    2. .pipeline/logical-ui/codebase_rules.json
    3. .pipeline/codebase_rules.json
    4. codebase_rules.json
    """
    candidate_paths = [
        os.environ.get("CODEBASE_RULES_PATH"),
        os.path.join(workspace_dir, ".pipeline", "logical-ui", "codebase_rules.json"),
        os.path.join(workspace_dir, ".pipeline", "codebase_rules.json"),
        os.path.join(workspace_dir, "codebase_rules.json"),
    ]
    for path in candidate_paths:
        if path and os.path.exists(path):
            return os.path.abspath(path)
    return None


def detect_tracker_provider(
    cli_provider: Optional[str] = None,
    rules: Optional[Dict[str, Any]] = None,
    workspace_dir: Optional[str] = None,
) -> str:
    """
    Detect whether the issue tracker provider is 'github' or 'gitlab'.
    Precedence:
    1. CLI flag (--provider)
    2. TRACKER_PROVIDER / PROVIDER env vars
    3. CI environment variables (GITLAB_CI, CI_SERVER_URL, CI_PROJECT_PATH, GITLAB_TOKEN, GL_TOKEN, CI_JOB_TOKEN -> gitlab; GITHUB_ACTIONS, GITHUB_REPOSITORY -> github)
    4. Rules config (tracker_rules.provider if explicit and not 'auto' / 'github')
    5. Git remote URL
    6. Rules config fallback or default 'github'
    """
    if cli_provider and cli_provider.lower() != "auto":
        return cli_provider.lower()

    env_provider = os.environ.get("TRACKER_PROVIDER") or os.environ.get("PROVIDER")
    if env_provider and env_provider.lower() != "auto":
        return env_provider.lower()

    # Detect from GitLab CI/CD or GitLab auth environment
    if (
        os.environ.get("GITLAB_CI")
        or os.environ.get("CI_SERVER_URL")
        or os.environ.get("CI_PROJECT_PATH")
        or os.environ.get("GITLAB_TOKEN")
        or os.environ.get("GL_TOKEN")
        or os.environ.get("CI_JOB_TOKEN")
    ):
        return "gitlab"

    # Detect from GitHub Actions environment
    if os.environ.get("GITHUB_ACTIONS") or os.environ.get("GITHUB_REPOSITORY"):
        return "github"

    # Detect from rules configuration
    if rules and isinstance(rules, dict):
        configured = rules.get("tracker_rules", {}).get("provider")
        if configured and configured.lower() not in ("auto", "github"):
            return configured.lower()

    # Detect from git remote origin
    if workspace_dir:
        remote_info = get_git_remote_info(workspace_dir)
        if remote_info and remote_info.get("is_gitlab"):
            return "gitlab"

    if rules and isinstance(rules, dict):
        configured = rules.get("tracker_rules", {}).get("provider")
        if configured and configured.lower() != "auto":
            return configured.lower()

    return "github"


def load_labels(workspace_dir: str, provider: Optional[str] = None) -> Dict[str, str]:
    """The configured taxonomy, or the built-in default if none is declared."""
    rules_path = resolve_codebase_rules_path(workspace_dir)
    loaded_rules: Dict[str, Any] = {}
    if rules_path:
        try:
            with open(rules_path, "r", encoding="utf-8") as fh:
                content = json.load(fh)
                if isinstance(content, dict):
                    loaded_rules = content
        except Exception as exc:
            print(f"Warning: could not read {rules_path}: {exc}", file=sys.stderr)

    effective_provider = provider or loaded_rules.get("tracker_rules", {}).get("provider", "github")

    if effective_provider == "gitlab":
        labels = dict(DEFAULT_GITLAB_LABELS)
        if loaded_rules.get("tracker_rules", {}).get("provider") == "gitlab":
            configured_labels = loaded_rules.get("tracker_rules", {}).get("labels", {})
            if isinstance(configured_labels, dict) and configured_labels:
                labels.update(configured_labels)
        return labels

    # GitHub provider
    labels = dict(DEFAULT_GITHUB_LABELS)
    configured_labels = loaded_rules.get("tracker_rules", {}).get("labels", {})
    if isinstance(configured_labels, dict) and configured_labels:
        labels.update(configured_labels)
    return labels


class GitLabV4LabelProvider:
    """
    GitLab REST API v4 Label Provider Adapter.
    Uses pure standard library urllib.request (zero external dependencies).
    Dispatches POST /api/v4/projects/:id/labels with PRIVATE-TOKEN or JOB-TOKEN.
    Normalizes color to #RRGGBB.
    Handles HTTP 409 Conflict idempotently as success (already exists).
    Supports --dry-run and --offline modes.
    """

    def __init__(
        self,
        server_url: Optional[str] = None,
        project_id: Optional[str] = None,
        token: Optional[str] = None,
        token_type: Optional[str] = None,
        ca_cert_path: Optional[str] = None,
        dry_run: bool = False,
        offline: bool = False,
        workspace_dir: Optional[str] = None,
        timeout_sec: int = 30,
        max_retries: int = 3,
        backoff_base_sec: float = 1.0,
    ):
        self.workspace_dir = workspace_dir or os.getcwd()
        self.dry_run = dry_run
        self.offline = offline
        self.timeout_sec = timeout_sec
        self.max_retries = max_retries
        self.backoff_base_sec = backoff_base_sec

        self.server_url = (server_url or self._resolve_server_url()).rstrip("/")
        self.raw_project_id = project_id or self._resolve_project_id()
        if self.raw_project_id:
            raw_str = str(self.raw_project_id).strip()
            if raw_str.isdigit():
                self.project_id_encoded = raw_str
            else:
                self.project_id_encoded = urllib.parse.quote(raw_str, safe="")
        else:
            self.project_id_encoded = ""

        resolved_token, resolved_token_type = self._resolve_token()
        self.token = token or resolved_token
        self.token_type = token_type or resolved_token_type or "PRIVATE-TOKEN"

        self.ca_cert_path = (
            ca_cert_path
            or os.environ.get("GITLAB_CA_CERT_PATH")
            or os.environ.get("SSL_CERT_FILE")
        )
        self.ssl_context = self._create_ssl_context(self.ca_cert_path)

    def _create_ssl_context(self, ca_cert_path: Optional[str]) -> ssl.SSLContext:
        ctx = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
        if ca_cert_path and os.path.isfile(ca_cert_path):
            try:
                ctx.load_verify_locations(cafile=ca_cert_path)
            except Exception as e:
                print(f"Warning: Failed to load CA certificate from {ca_cert_path}: {e}", file=sys.stderr)
        return ctx

    def _resolve_server_url(self) -> str:
        for env_var in ("GITLAB_URL", "CI_SERVER_URL", "GL_SERVER_URL"):
            val = os.environ.get(env_var)
            if val and val.strip():
                return val.strip().rstrip("/")
        remote_info = get_git_remote_info(self.workspace_dir)
        if remote_info and remote_info.get("server_url") and remote_info.get("is_gitlab"):
            return remote_info["server_url"]
        return "https://gitlab.com"

    def _resolve_project_id(self) -> Optional[str]:
        for env_var in ("CI_PROJECT_PATH", "CI_PROJECT_ID", "GITLAB_PROJECT", "GL_PROJECT"):
            val = os.environ.get(env_var)
            if val and val.strip():
                return val.strip()
        remote_info = get_git_remote_info(self.workspace_dir)
        if remote_info and remote_info.get("project_path"):
            return remote_info["project_path"]
        env_repo = os.environ.get("UPSTREAM_REPOSITORY") or os.environ.get("GIT_REMOTE_ORIGIN")
        if env_repo:
            return env_repo.strip()
        return None

    def _resolve_token(self) -> Tuple[Optional[str], str]:
        for var in ("GITLAB_TOKEN", "GL_TOKEN"):
            val = os.environ.get(var)
            if val and val.strip():
                return val.strip(), "PRIVATE-TOKEN"
        job_token = os.environ.get("CI_JOB_TOKEN")
        if job_token and job_token.strip():
            return job_token.strip(), "JOB-TOKEN"
        glab_path = shutil.which("glab")
        if glab_path:
            try:
                res = subprocess.run([glab_path, "auth", "token"], capture_output=True, text=True, timeout=5)
                if res.returncode == 0 and res.stdout.strip():
                    return res.stdout.strip(), "PRIVATE-TOKEN"
            except Exception:
                pass
        try:
            hostname = urllib.parse.urlparse(self.server_url).hostname or "gitlab.com"
            auth = netrc.netrc().authenticators(hostname)
            if auth and auth[2] and auth[2].strip():
                return auth[2].strip(), "PRIVATE-TOKEN"
        except Exception:
            pass
        return None, "PRIVATE-TOKEN"

    def create_label(self, name: str, description: str = "", color: str = "#0E8A16") -> bool:
        if not name:
            return False

        normalized_color = normalize_color(color, with_hash=True)

        if self.dry_run:
            url = f"{self.server_url}/api/v4/projects/{self.project_id_encoded or ':id'}/labels"
            payload_str = json.dumps({"name": name, "color": normalized_color, "description": description})
            print(f"  [dry-run] POST {url} {payload_str}")
            return True

        if self.offline:
            print(f"  [offline] create label '{name}' ({normalized_color})")
            return True

        if not self.project_id_encoded:
            print(f"  [FAILED] {name}: GitLab project ID/path could not be resolved.", file=sys.stderr)
            return False

        endpoint = f"projects/{self.project_id_encoded}/labels"
        url = f"{self.server_url}/api/v4/{endpoint}"
        payload = {
            "name": name,
            "color": normalized_color,
            "description": description or "",
        }
        body_bytes = json.dumps(payload).encode("utf-8")

        headers = {
            "Accept": "application/json",
            "Content-Type": "application/json; charset=utf-8",
            "User-Agent": "DEAP-Label-Bootstrapper/1.0",
        }
        if self.token:
            if self.token_type == "JOB-TOKEN":
                headers["JOB-TOKEN"] = self.token
            else:
                headers["PRIVATE-TOKEN"] = self.token

        req = urllib.request.Request(url=url, data=body_bytes, headers=headers, method="POST")

        attempt = 0
        while attempt < self.max_retries:
            attempt += 1
            try:
                with urllib.request.urlopen(req, context=self.ssl_context, timeout=self.timeout_sec) as resp:
                    if resp.status in (200, 201):
                        print(f"  [ok] {name}")
                        return True
                    print(f"  [ok] {name} (status {resp.status})")
                    return True
            except urllib.error.HTTPError as e:
                status_code = e.code
                raw_err = e.read().decode("utf-8", errors="ignore") if e.fp else ""
                # HTTP 409 Conflict: label already exists -> idempotent success
                if status_code == 409 or "already exists" in raw_err.lower():
                    print(f"  [ok] {name} (already exists)")
                    return True

                if status_code in (429, 502, 503, 504) and attempt < self.max_retries:
                    retry_after = e.headers.get("Retry-After") if e.headers else None
                    sleep_time = float(retry_after) if (retry_after and retry_after.isdigit()) else (self.backoff_base_sec * (2 ** (attempt - 1)))
                    time.sleep(sleep_time)
                    continue

                print(f"  [FAILED] {name}: HTTP {status_code} - {raw_err.strip()}", file=sys.stderr)
                return False
            except urllib.error.URLError as e:
                if attempt >= self.max_retries:
                    print(f"  [FAILED] {name}: Network transport failure: {e.reason}", file=sys.stderr)
                    return False
                time.sleep(self.backoff_base_sec * (2 ** (attempt - 1)))
            except Exception as e:
                print(f"  [FAILED] {name}: {e}", file=sys.stderr)
                return False

        return False


class GitHubCLILabelProvider:
    """
    GitHub CLI Label Provider Adapter.
    Uses 'gh label create --force' with optional --repo support.
    Supports --dry-run and --offline modes.
    """

    def __init__(
        self,
        repo: Optional[str] = None,
        dry_run: bool = False,
        offline: bool = False,
        workspace_dir: Optional[str] = None,
    ):
        self.repo = repo
        self.dry_run = dry_run
        self.offline = offline
        self.workspace_dir = workspace_dir or os.getcwd()

    def create_label(self, name: str, description: str = "", color: str = "0e8a16") -> bool:
        if not name:
            return False

        normalized_color = normalize_color(color, with_hash=False)
        cmd = [
            "gh", "label", "create", name,
            "--color", normalized_color,
            "--description", description,
            "--force",
        ]
        if self.repo:
            cmd += ["--repo", self.repo]

        if self.dry_run:
            print("  [dry-run] " + " ".join(cmd))
            return True

        if self.offline:
            print("  [offline] " + " ".join(cmd))
            return True

        try:
            result = subprocess.run(
                cmd,
                cwd=self.workspace_dir,
                capture_output=True,
                text=True,
                timeout=30,
            )
            if result.returncode == 0:
                print(f"  [ok] {name}")
                return True
            else:
                err = (result.stderr or "").strip()
                print(f"  [FAILED] {name}: {err}", file=sys.stderr)
                return False
        except Exception as e:
            print(f"  [FAILED] {name}: {e}", file=sys.stderr)
            return False


def bootstrap_labels(
    provider_name: str,
    labels: Dict[str, str],
    repo: Optional[str] = None,
    project: Optional[str] = None,
    gitlab_url: Optional[str] = None,
    token: Optional[str] = None,
    dry_run: bool = False,
    offline: bool = False,
    workspace_dir: Optional[str] = None,
) -> bool:
    """Bootstrap all tracker labels using the selected provider."""
    print(f"Provisioning {len(labels)} tracker labels ({provider_name})...")

    if provider_name == "gitlab":
        provider = GitLabV4LabelProvider(
            server_url=gitlab_url,
            project_id=project,
            token=token,
            dry_run=dry_run,
            offline=offline,
            workspace_dir=workspace_dir,
        )
    else:
        provider = GitHubCLILabelProvider(
            repo=repo,
            dry_run=dry_run,
            offline=offline,
            workspace_dir=workspace_dir,
        )

    failures = []
    for key, name in sorted(labels.items()):
        color, description = get_label_presentation(key, name)
        success = provider.create_label(name=name, description=description, color=color)
        if not success:
            failures.append(name)

    if failures:
        print(
            f"\n{len(failures)} label(s) could not be provisioned. The just-in-time "
            "path in create_issue.sh remains as a fallback, so filing still works — "
            "but the tracker's label filter will stay incomplete until this succeeds.",
            file=sys.stderr,
        )
        return False

    if not dry_run and not offline:
        print("Tracker label taxonomy provisioned.")
    return True


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--provider",
        choices=["auto", "github", "gitlab"],
        help="Issue tracker provider ('github' or 'gitlab'). Defaults to auto-detection.",
    )
    parser.add_argument("--repo", help="Target GitHub repository as OWNER/NAME")
    parser.add_argument("--project", help="Target GitLab project path (group/project) or numeric ID")
    parser.add_argument("--gitlab-url", help="GitLab base URL (e.g. https://gitlab.internal.corp)")
    parser.add_argument("--token", help="Authentication token for GitLab REST API or GitHub")
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print the commands / API requests without executing them",
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        help="Run in offline mode without making network or CLI calls",
    )
    args = parser.parse_args(argv)

    workspace_dir = find_workspace_dir(os.path.dirname(os.path.abspath(__file__)))

    # Load rules if present
    rules_path = resolve_codebase_rules_path(workspace_dir)
    rules = None
    if rules_path:
        try:
            with open(rules_path, "r", encoding="utf-8") as fh:
                rules = json.load(fh)
        except Exception:
            rules = None

    provider = detect_tracker_provider(
        cli_provider=args.provider,
        rules=rules,
        workspace_dir=workspace_dir,
    )

    labels = load_labels(workspace_dir, provider=provider)

    success = bootstrap_labels(
        provider_name=provider,
        labels=labels,
        repo=args.repo,
        project=args.project,
        gitlab_url=args.gitlab_url,
        token=args.token,
        dry_run=args.dry_run,
        offline=args.offline,
        workspace_dir=workspace_dir,
    )
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
