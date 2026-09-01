#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com

"""
Backlog reconciliation script that synchronises local markdown spec files
with an external issue tracker (e.g. GitHub Issues).

Scans epics/, features/, user-stories/ and use-cases/ directories,
resolves issue-ID placeholders, updates dependency checklists, syncs
issue bodies, and marks completed items Fixed / Resolved.
A spec file is matched to its issue by the canonical `issue_id` in its YAML
frontmatter; normalized-title matching is a warning-only fallback for a spec
that has none yet.  See resolve_spec_issue_number and
constitution.md:57-59 § Unique Backlog Identifiers (#314, #316).
Never closes an issue: constitution.md:161 reserves Closed for Product Owner
validation (#309).  Hard-exits on any
referenced issue that does not exist in the tracker (hallucination gate).
"""

#!/usr/bin/env python3
import os
import re
import subprocess
import json
import netrc
import sys
import yaml
import traceback
import shutil
import tempfile
import copy
import ssl
import time
import base64
import argparse
import urllib.request
import urllib.parse
import urllib.error
from typing import Dict, List, Optional, Tuple, Any, Set, Union

def sanitize_github_token_env():
    """
    Sanitize environment by removing dummy or placeholder tokens
    that interfere with git/gh/glab terminal operations.
    """
    dummy_keywords = ("antigravity", "dummy", "placeholder", "invalid", "mock")
    for var in ("GITHUB_TOKEN", "GH_TOKEN", "GITLAB_TOKEN", "GL_TOKEN", "CI_JOB_TOKEN"):
        val = os.environ.get(var)
        if val and any(kw in val.lower() for kw in dummy_keywords):
            os.environ.pop(var, None)

sanitize_github_token_env()

DEFAULT_CODEBASE_RULES = {
    "meta": {
        "version": "1.0.0",
        "description": "Default pipeline and codebase compliance rules",
        "upstream_repository": "gintatkinson/DEAP01-spec-core",
    },
    "tracker_rules": {
        "provider": "github",
        "issue_id_placeholder": "#[IssueID]",
        "prefix_normalization_regex": r"^(epic|feature|feat|user[- ]story|use[- ]case|us|uc)[s]?(?:[- ]*\d+\s*[:\-]?|:)\s*",
        "title_extraction_prefixes_regex": r"(?:Feature\s+\d+\s*:\s*|Use\s+Case\s+\d+\s*:\s*|User\s+Story\s+\d+\s*:\s*)?",
        "truncation_headers": [
            "## Acceptance Criteria",
            "## User Stories"
        ],
        "truncation_message_template": "\n\n---\n*Warning: This issue body has been truncated because it exceeds the tracker size limit of {max_body_chars} characters.*\n*Please refer to the full specification file in the repository at `{rel_path}` for the complete details.*\n",
        "numeric_prefix": "#",
        "alphanumeric_prefix": "",
        "keys": {
            "issue_id": "number",
            "title": "title",
            "labels": "labels",
            "state": "state",
            "closed_state_value": "CLOSED",
            "open_state_value": "OPEN"
        },
        "labels": {
            "epic": "epic",
            "feature": "feature",
            "user_story": "user-story",
            "use_case": "use-case",
            "resolved": "status:fixed-resolved"
        },
        "close_comments": {
            "epic": "Epic completed. All constituent features successfully delivered and verified.",
            "feature": "Resolved. All acceptance criteria and verification tasks for feature '{title}' have been completed and verified.",
            "user_story": "Resolved. All dependent features/tasks for BDD scenario '{title}' have been completed and verified.",
            "use_case": "Resolved. All dependent user stories and features for use case '{title}' are completed.",
            "compiler": "Resolved. Compiler backlog test target(s) '{test_targets}' completed and verified."
        },
        "commands": {
            "list_issues": [
                "gh",
                "issue",
                "list",
                "--limit",
                "1000",
                "--state",
                "all",
                "--json",
                "number,title,state,labels"
            ],
            "edit_issue": [
                "gh",
                "issue",
                "edit",
                "{number}",
                "--body-file",
                "{temp_path}"
            ],
            "edit_issue_title": [
                "gh",
                "issue",
                "edit",
                "{number}",
                "--title",
                "{title}"
            ],
            "add_label": [
                "gh",
                "issue",
                "edit",
                "{number}",
                "--add-label",
                "{label}"
            ],
            "resolve_issue": [
                "gh",
                "issue",
                "edit",
                "{number}",
                "--add-label",
                "{label}"
            ],
            "comment_issue": [
                "gh",
                "issue",
                "comment",
                "{number}",
                "--body",
                "{comment}"
            ],
            "create_label": [
                "gh",
                "label",
                "create",
                "{label}",
                "--description",
                "{description}",
                "--color",
                "0E8A16",
                "--force"
            ]
        }
    },
    "backlog_directories": {
        "epics": "docs/epics",
        "features": "docs/features",
        "user_stories": "docs/user-stories",
        "use_cases": "docs/use-cases",
        "schemas": "schema"
    }
}

DEFAULT_GITLAB_TRACKER_RULES = {
    "provider": "gitlab",
    "issue_id_placeholder": "#[IssueID]",
    "prefix_normalization_regex": r"^(epic|feature|feat|user[- ]story|use[- ]case|us|uc)[s]?(?:[- ]*\d+\s*[:\-]?|:)\s*",
    "title_extraction_prefixes_regex": r"(?:Feature\s+\d+\s*:\s*|Use\s+Case\s+\d+\s*:\s*|User\s+Story\s+\d+\s*:\s*)?",
    "truncation_headers": [
        "## Acceptance Criteria",
        "## User Stories"
    ],
    "truncation_message_template": "\n\n---\n*Warning: This issue body has been truncated because it exceeds the tracker size limit of {max_body_chars} characters.*\n*Please refer to the full specification file in the repository at `{rel_path}` for the complete details.*\n",
    "numeric_prefix": "#",
    "alphanumeric_prefix": "",
    "keys": {
        "issue_id": "iid",
        "title": "title",
        "labels": "labels",
        "state": "state",
        "closed_state_value": "CLOSED",
        "open_state_value": "OPENED"
    },
    "labels": {
        "epic": "type::epic",
        "feature": "type::feature",
        "user_story": "type::user-story",
        "use_case": "type::use-case",
        "ready_for_review": "status::ready-for-review",
        "resolved": "status::fixed-resolved"
    },
    "close_comments": {
        "epic": "Epic completed. All constituent features successfully delivered and verified.",
        "feature": "Resolved. All acceptance criteria and verification tasks for feature '{title}' have been completed and verified.",
        "user_story": "Resolved. All dependent features/tasks for BDD scenario '{title}' have been completed and verified.",
        "use_case": "Resolved. All dependent user stories and features for use case '{title}' are completed.",
        "compiler": "Resolved. Compiler backlog test target(s) '{test_targets}' completed and verified."
    },
    "commands": {
        "list_issues": [
            "glab",
            "issue",
            "list",
            "--all",
            "--per-page",
            "1000",
            "--output",
            "json"
        ],
        "edit_issue": [
            "glab",
            "issue",
            "update",
            "{number}",
            "--description",
            "{temp_path}"
        ],
        "edit_issue_title": [
            "glab",
            "issue",
            "update",
            "{number}",
            "--title",
            "{title}"
        ],
        "add_label": [
            "glab",
            "issue",
            "update",
            "{number}",
            "--label",
            "{label}"
        ],
        "resolve_issue": [
            "glab",
            "issue",
            "update",
            "{number}",
            "--label",
            "{label}"
        ],
        "comment_issue": [
            "glab",
            "issue",
            "note",
            "{number}",
            "--message",
            "{comment}"
        ],
        "create_label": [
            "glab",
            "label",
            "create",
            "{label}",
            "--description",
            "{description}",
            "--color",
            "#0E8A16"
        ]
    }
}

DEFAULT_GITLAB_STRUCTURAL_LABELS = {
    "epic": "type::epic",
    "feature": "type::feature",
    "user_story": "type::user-story",
    "use_case": "type::use-case",
}

DEFAULT_JIRA_TRACKER_RULES = {
    "provider": "jira",
    "issue_id_placeholder": "#[IssueID]",
    "prefix_normalization_regex": r"^(epic|feature|feat|user[- ]story|use[- ]case|us|uc)[s]?(?:[- ]*\d+\s*[:\-]?|:)\s*",
    "title_extraction_prefixes_regex": r"(?:Feature\s+\d+\s*:\s*|Use\s+Case\s+\d+\s*:\s*|User\s+Story\s+\d+\s*:\s*)?",
    "truncation_headers": [
        "## Acceptance Criteria",
        "## User Stories"
    ],
    "truncation_message_template": "\n\n---\n*Warning: This issue body has been truncated because it exceeds the tracker size limit of {max_body_chars} characters.*\n*Please refer to the full specification file in the repository at `{rel_path}` for the complete details.*\n",
    "numeric_prefix": "",
    "alphanumeric_prefix": "",
    "keys": {
        "issue_id": "key",
        "title": "title",
        "labels": "labels",
        "state": "state",
        "closed_state_value": "CLOSED",
        "open_state_value": "OPEN"
    },
    "labels": {
        "epic": "type::epic",
        "feature": "type::feature",
        "user_story": "type::user-story",
        "use_case": "type::use-case",
        "ready_for_review": "status::ready-for-review",
        "resolved": "status::fixed-resolved"
    },
    "close_comments": {
        "epic": "Epic completed. All constituent features successfully delivered and verified.",
        "feature": "Resolved. All acceptance criteria and verification tasks for feature '{title}' have been completed and verified.",
        "user_story": "Resolved. All dependent features/tasks for BDD scenario '{title}' have been completed and verified.",
        "use_case": "Resolved. All dependent user stories and features for use case '{title}' are completed.",
        "compiler": "Resolved. Compiler backlog test target(s) '{test_targets}' completed and verified."
    }
}

DEFAULT_JIRA_STRUCTURAL_LABELS = {
    "epic": "type::epic",
    "feature": "type::feature",
    "user_story": "type::user-story",
    "use_case": "type::use-case",
}

def parse_git_remote_url(remote_url: str) -> Dict[str, Any]:
    """
    Parse a git remote origin URL into its components:
    - raw: raw URL string
    - is_gitlab: True if domain contains 'gitlab'
    - project_path: repository path (e.g. 'gintatkinson/DEAP01-spec-core' or 'group/subgroup/project')
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
            "host": host
        }
    
    # Check if SCP-style SSH URL (e.g. git@gitlab.com:owner/repo or git@gitlab.internal.corp:group/sub/repo)
    scp_match = re.match(r'^(?:[^@]+@)?([^:]+):(.+)$', clean_url)
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
            "host": host
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
        "host": "gitlab.com" if is_gitlab else "github.com"
    }

def get_git_remote_info(workspace_dir: str) -> Optional[Dict[str, Any]]:
    try:
        res = subprocess.run(
            ["git", "remote", "get-url", "origin"],
            cwd=workspace_dir,
            capture_output=True,
            text=True,
            check=True,
            timeout=30
        )
        url = res.stdout.strip()
        return parse_git_remote_url(url)
    except Exception:
        return None

def detect_tracker_provider(cli_provider: Optional[str] = None, rules: Optional[Dict[str, Any]] = None, workspace_dir: Optional[str] = None) -> str:
    if cli_provider and cli_provider.lower() != "auto":
        return cli_provider.lower()
        
    env_provider = os.environ.get("TRACKER_PROVIDER") or os.environ.get("PROVIDER")
    if env_provider and env_provider.lower() != "auto":
        return env_provider.lower()

    if rules and isinstance(rules, dict):
        configured = rules.get("tracker_rules", {}).get("provider")
        if configured and configured.lower() not in ("auto", "github"):
            return configured.lower()

    # Detect from Jira environment variables
    if (
        os.environ.get("JIRA_SERVER_URL")
        or os.environ.get("JIRA_URL")
        or os.environ.get("JIRA_PROJECT_KEY")
        or os.environ.get("JIRA_PROJECT")
        or os.environ.get("JIRA_API_TOKEN")
        or os.environ.get("JIRA_PAT")
        or os.environ.get("JIRA_TOKEN")
    ):
        return "jira"

    # Detect from CI environment variables
    if os.environ.get("GITLAB_CI") or os.environ.get("CI_SERVER_URL") or os.environ.get("CI_PROJECT_PATH"):
        return "gitlab"
    if os.environ.get("GITHUB_ACTIONS") or os.environ.get("GITHUB_REPOSITORY"):
        return "github"

    # Detect from git remote
    if workspace_dir:
        remote_info = get_git_remote_info(workspace_dir)
        if remote_info and remote_info.get("is_gitlab"):
            return "gitlab"

    # Fallback to configured provider or "github"
    if rules and isinstance(rules, dict):
        configured = rules.get("tracker_rules", {}).get("provider")
        if configured:
            return configured.lower()
            
    return "github"

class GitLabV4Provider:
    """
    Native GitLab REST API v4 Provider Adapter.
    Uses standard library urllib.request (zero external dependencies).
    Supports personal/project access tokens (PRIVATE-TOKEN) and CI job tokens (JOB-TOKEN).
    Supports self-hosted instances, custom root CA certs, and glab CLI fallback.
    """

    def __init__(
        self,
        server_url: Optional[str] = None,
        project_id: Optional[str] = None,
        token: Optional[str] = None,
        token_type: Optional[str] = None,
        ca_cert_path: Optional[str] = None,
        timeout_sec: int = 30,
        max_retries: int = 3,
        backoff_base_sec: float = 1.0,
        offline: bool = False,
        workspace_dir: Optional[str] = None,
    ):
        self.workspace_dir = workspace_dir or os.getcwd()
        self.offline = offline
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

        self.timeout_sec = timeout_sec
        self.max_retries = max_retries
        self.backoff_base_sec = backoff_base_sec
        self.ca_cert_path = ca_cert_path or os.environ.get("GITLAB_CA_CERT_PATH") or os.environ.get("SSL_CERT_FILE")
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

    def _api_request(
        self,
        endpoint: str,
        method: str = "GET",
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
    ) -> Tuple[int, Any, Dict[str, str]]:
        if self.offline:
            return 200, [], {}

        if not self.project_id_encoded:
            raise ValueError("GitLab project ID / path could not be resolved.")

        clean_endpoint = endpoint.lstrip("/")
        url = f"{self.server_url}/api/v4/{clean_endpoint}"
        if params:
            query_str = urllib.parse.urlencode(params)
            url = f"{url}?{query_str}"

        headers = {
            "Accept": "application/json",
            "User-Agent": "DEAP-Backlog-Reconciler/1.0",
        }
        if self.token:
            if self.token_type == "JOB-TOKEN":
                headers["JOB-TOKEN"] = self.token
            else:
                headers["PRIVATE-TOKEN"] = self.token

        body_bytes = None
        if data is not None:
            body_bytes = json.dumps(data).encode("utf-8")
            headers["Content-Type"] = "application/json; charset=utf-8"

        req = urllib.request.Request(url=url, data=body_bytes, headers=headers, method=method)

        attempt = 0
        while attempt < self.max_retries:
            attempt += 1
            try:
                with urllib.request.urlopen(req, context=self.ssl_context, timeout=self.timeout_sec) as resp:
                    status_code = resp.status
                    resp_headers = {k: v for k, v in resp.headers.items()}
                    raw_body = resp.read().decode("utf-8")
                    parsed_body = json.loads(raw_body) if raw_body.strip() else {}
                    return status_code, parsed_body, resp_headers
            except urllib.error.HTTPError as e:
                status_code = e.code
                error_headers = {k: v for k, v in e.headers.items()} if e.headers else {}
                raw_err = e.read().decode("utf-8", errors="ignore") if e.fp else ""
                
                if status_code in (429, 502, 503, 504) and attempt < self.max_retries:
                    retry_after = error_headers.get("Retry-After")
                    if retry_after and retry_after.isdigit():
                        sleep_time = float(retry_after)
                    else:
                        sleep_time = self.backoff_base_sec * (2 ** (attempt - 1))
                    time.sleep(sleep_time)
                    continue
                
                raise RuntimeError(
                    f"GitLab API HTTP {status_code} Error on {method} {url}: {raw_err}"
                ) from e
            except urllib.error.URLError as e:
                if attempt >= self.max_retries:
                    raise RuntimeError(
                        f"Network transport failure connecting to {self.server_url}: {e.reason}"
                    ) from e
                time.sleep(self.backoff_base_sec * (2 ** (attempt - 1)))

        raise TimeoutError(f"Exceeded maximum retries ({self.max_retries}) for GitLab API request: {url}")

    def list_issues(self) -> List[Dict[str, Any]]:
        """
        Fetch all project issues using GitLab REST API v4 with pagination.
        Falls back to glab CLI if token is absent and glab is available,
        or returns empty list in offline/unauthenticated mode with notice.
        """
        if self.offline:
            print("[Notice] GitLab tracker in offline mode. Operating in offline/local specification mode.")
            return []

        if not self.token and shutil.which("glab"):
            return self._list_issues_via_glab()

        if not self.token:
            print("[Notice] No GitLab authentication token found (GITLAB_TOKEN, GL_TOKEN, CI_JOB_TOKEN). "
                  "Operating in offline/local specification mode.")
            return []

        if not self.project_id_encoded:
            print("[Notice] GitLab project path/ID not resolved. Operating in offline/local specification mode.")
            return []

        all_issues = []
        page = 1
        endpoint = f"projects/{self.project_id_encoded}/issues"

        try:
            print(f"Fetching active and closed issues from GitLab REST API v4 ({self.server_url})...")
            while True:
                params = {
                    "scope": "all",
                    "state": "all",
                    "per_page": 100,
                    "page": page,
                }
                status_code, issues, headers = self._api_request(endpoint, method="GET", params=params)
                if not isinstance(issues, list):
                    break

                for issue in issues:
                    if "iid" in issue:
                        issue["number"] = issue["iid"]
                    if "state" in issue:
                        issue["state"] = str(issue["state"]).upper()
                    all_issues.append(issue)

                next_page_hdr = headers.get("X-Next-Page") or headers.get("x-next-page")
                if next_page_hdr and str(next_page_hdr).strip() and str(next_page_hdr).strip() != "0":
                    page = int(next_page_hdr)
                elif len(issues) == 100:
                    page += 1
                else:
                    break

            return all_issues
        except Exception as e:
            err_msg = str(e)
            print(f"[Notice] GitLab issue tracker unavailable ({err_msg}). Operating in offline/local specification mode.")
            return []

    def _list_issues_via_glab(self) -> List[Dict[str, Any]]:
        try:
            cmd = ["glab", "issue", "list", "--all", "--per-page", "1000", "--output", "json"]
            res = subprocess.run(cmd, cwd=self.workspace_dir, capture_output=True, text=True, timeout=30)
            if res.returncode == 0 and res.stdout.strip():
                issues = json.loads(res.stdout)
                for issue in issues:
                    if "iid" in issue:
                        issue["number"] = issue["iid"]
                    if "state" in issue:
                        issue["state"] = str(issue["state"]).upper()
                return issues
        except Exception as e:
            print(f"[Notice] glab CLI fallback failed: {e}")
        return []

    def create_issue(self, title: str, description: str, labels: Optional[Any] = None) -> Optional[Dict[str, Any]]:
        if self.offline or not self.token or not self.project_id_encoded:
            return None
        endpoint = f"projects/{self.project_id_encoded}/issues"
        payload = {
            "title": title,
            "description": description or "",
        }
        if labels:
            if isinstance(labels, list):
                payload["labels"] = ",".join(str(l) for l in labels if l)
            else:
                payload["labels"] = str(labels)
        try:
            status_code, data, _ = self._api_request(endpoint, method="POST", data=payload)
            if isinstance(data, dict):
                if "iid" in data:
                    data["number"] = data["iid"]
                return data
        except Exception as e:
            print(f"  [Warning] Failed to create GitLab issue '{title}': {e}", file=sys.stderr)
        return None

    def edit_issue(self, iid: Any, description: str) -> bool:
        if self.offline or not self.token or not self.project_id_encoded:
            if shutil.which("glab"):
                return self._edit_issue_via_glab(iid, description)
            return False
        endpoint = f"projects/{self.project_id_encoded}/issues/{iid}"
        payload = {"description": description}
        try:
            status_code, data, _ = self._api_request(endpoint, method="PUT", data=payload)
            return status_code in (200, 201)
        except Exception as e:
            print(f"  [Warning] Failed to update GitLab issue #{iid} description: {e}", file=sys.stderr)
            if shutil.which("glab"):
                return self._edit_issue_via_glab(iid, description)
            return False

    def edit_issue_title(self, iid: Any, title: str) -> bool:
        if self.offline or not self.token or not self.project_id_encoded:
            if shutil.which("glab"):
                return self._edit_issue_title_via_glab(iid, title)
            return False
        endpoint = f"projects/{self.project_id_encoded}/issues/{iid}"
        payload = {"title": title}
        try:
            status_code, data, _ = self._api_request(endpoint, method="PUT", data=payload)
            return status_code in (200, 201)
        except Exception as e:
            print(f"  [Warning] Failed to update GitLab issue #{iid} title: {e}", file=sys.stderr)
            if shutil.which("glab"):
                return self._edit_issue_title_via_glab(iid, title)
            return False

    def add_label(self, iid: Any, label: str) -> bool:
        if not label:
            return False
        if self.offline or not self.token or not self.project_id_encoded:
            if shutil.which("glab"):
                return self._add_label_via_glab(iid, label)
            return False
        endpoint = f"projects/{self.project_id_encoded}/issues/{iid}"
        payload = {"add_labels": label}
        try:
            status_code, data, _ = self._api_request(endpoint, method="PUT", data=payload)
            return status_code in (200, 201)
        except Exception as e:
            print(f"  [Warning] Failed to add label '{label}' to GitLab issue #{iid}: {e}", file=sys.stderr)
            if shutil.which("glab"):
                return self._add_label_via_glab(iid, label)
            return False

    def comment_issue(self, iid: Any, comment: str) -> bool:
        if not comment:
            return False
        if self.offline or not self.token or not self.project_id_encoded:
            if shutil.which("glab"):
                return self._comment_issue_via_glab(iid, comment)
            return False
        endpoint = f"projects/{self.project_id_encoded}/issues/{iid}/notes"
        payload = {"body": comment}
        try:
            status_code, data, _ = self._api_request(endpoint, method="POST", data=payload)
            return status_code in (200, 201)
        except Exception as e:
            print(f"  [Warning] Failed to post comment on GitLab issue #{iid}: {e}", file=sys.stderr)
            if shutil.which("glab"):
                return self._comment_issue_via_glab(iid, comment)
            return False

    def create_label(self, label: str, description: str = "", color: str = "#0E8A16") -> bool:
        if not label:
            return False
        if self.offline or not self.token or not self.project_id_encoded:
            return False
        endpoint = f"projects/{self.project_id_encoded}/labels"
        payload = {
            "name": label,
            "description": description or "",
            "color": color or "#0E8A16",
        }
        try:
            status_code, data, _ = self._api_request(endpoint, method="POST", data=payload)
            return status_code in (200, 201)
        except Exception as e:
            err_str = str(e)
            if "409" in err_str or "already exists" in err_str.lower():
                return True
            return False

    def _edit_issue_via_glab(self, iid: Any, description: str) -> bool:
        try:
            cmd = ["glab", "issue", "update", str(iid), "--description", description]
            res = subprocess.run(cmd, cwd=self.workspace_dir, capture_output=True, text=True, timeout=30)
            return res.returncode == 0
        except Exception:
            return False

    def _edit_issue_title_via_glab(self, iid: Any, title: str) -> bool:
        try:
            cmd = ["glab", "issue", "update", str(iid), "--title", title]
            res = subprocess.run(cmd, cwd=self.workspace_dir, capture_output=True, text=True, timeout=30)
            return res.returncode == 0
        except Exception:
            return False

    def _add_label_via_glab(self, iid: Any, label: str) -> bool:
        try:
            cmd = ["glab", "issue", "update", str(iid), "--label", label]
            res = subprocess.run(cmd, cwd=self.workspace_dir, capture_output=True, text=True, timeout=30)
            return res.returncode == 0
        except Exception:
            return False

    def _comment_issue_via_glab(self, iid: Any, comment: str) -> bool:
        try:
            cmd = ["glab", "issue", "note", str(iid), "--message", comment]
            res = subprocess.run(cmd, cwd=self.workspace_dir, capture_output=True, text=True, timeout=30)
            return res.returncode == 0
        except Exception:
            return False

class JiraV2V3Provider:
    """
    Native Jira REST API v2/v3 Provider Adapter.
    Uses standard library urllib.request (zero external dependencies).
    Supports Jira Cloud Basic Auth (email + API token) and Jira Data Center / Server Bearer PAT (Personal Access Token).
    Supports .netrc credential resolution, self-hosted instances, custom root CA certs, and dynamic workflow transitions.
    """

    def __init__(
        self,
        server_url: Optional[str] = None,
        project_key: Optional[str] = None,
        email: Optional[str] = None,
        token: Optional[str] = None,
        token_type: Optional[str] = None,
        ca_cert_path: Optional[str] = None,
        timeout_sec: int = 30,
        max_retries: int = 3,
        backoff_base_sec: float = 1.0,
        offline: bool = False,
        workspace_dir: Optional[str] = None,
    ):
        self.workspace_dir = workspace_dir or os.getcwd()
        self.offline = offline
        resolved_server_url = self._resolve_server_url()
        self.server_url = (server_url or resolved_server_url or "").rstrip("/")
        self.project_key = (project_key or self._resolve_project_key() or "").strip()

        resolved_email, resolved_token, resolved_token_type = self._resolve_auth()
        self.email = email if email is not None else resolved_email
        self.token = token if token is not None else resolved_token
        self.token_type = token_type if token_type is not None else (resolved_token_type or ("Basic" if self.email else "Bearer"))
        self.auth_type = self.token_type.lower() if self.token_type else ("basic" if self.email else "bearer")

        self.timeout_sec = timeout_sec
        self.max_retries = max_retries
        self.backoff_base_sec = backoff_base_sec
        self.ca_cert_path = ca_cert_path or os.environ.get("JIRA_CA_CERT_PATH") or os.environ.get("SSL_CERT_FILE")
        self.ssl_context = self._create_ssl_context(self.ca_cert_path)

    def _create_ssl_context(self, ca_cert_path: Optional[str]) -> ssl.SSLContext:
        ctx = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
        if ca_cert_path and os.path.isfile(ca_cert_path):
            try:
                ctx.load_verify_locations(cafile=ca_cert_path)
            except Exception as e:
                print(f"Warning: Failed to load CA certificate from {ca_cert_path}: {e}", file=sys.stderr)
        return ctx

    def _resolve_server_url(self) -> Optional[str]:
        for env_var in ("JIRA_SERVER_URL", "JIRA_URL", "JIRA_HOST", "JIRA_BASE_URL"):
            val = os.environ.get(env_var)
            if val and val.strip():
                url = val.strip().rstrip("/")
                if not url.startswith("http://") and not url.startswith("https://"):
                    url = f"https://{url}"
                return url
        return None

    def _resolve_project_key(self) -> Optional[str]:
        for env_var in ("JIRA_PROJECT_KEY", "JIRA_PROJECT", "JIRA_KEY"):
            val = os.environ.get(env_var)
            if val and val.strip():
                return val.strip()
        return None

    def _resolve_auth(self) -> Tuple[Optional[str], Optional[str], Optional[str]]:
        email = None
        for env_var in ("JIRA_EMAIL", "JIRA_USER", "JIRA_USERNAME"):
            val = os.environ.get(env_var)
            if val and val.strip():
                email = val.strip()
                break

        # Check PAT first (Data Center / Server Bearer)
        pat = os.environ.get("JIRA_PAT")
        if pat and pat.strip():
            return email, pat.strip(), "Bearer"

        # Check API token (Cloud basic auth)
        api_token = os.environ.get("JIRA_API_TOKEN")
        if api_token and api_token.strip():
            return email, api_token.strip(), "Basic" if email else "Bearer"

        # Generic token
        token = os.environ.get("JIRA_TOKEN")
        if token and token.strip():
            token_type = "Basic" if email else "Bearer"
            return email, token.strip(), token_type

        # Netrc resolution fallback
        try:
            target_host = "jira.atlassian.net"
            if self.server_url:
                parsed = urllib.parse.urlparse(self.server_url)
                if parsed.hostname:
                    target_host = parsed.hostname
            auth = netrc.netrc().authenticators(target_host)
            if auth:
                netrc_user, _, netrc_pwd = auth
                res_email = email or (netrc_user.strip() if netrc_user else None)
                res_token = netrc_pwd.strip() if netrc_pwd else None
                res_type = "Basic" if res_email else "Bearer"
                return res_email, res_token, res_type
        except Exception:
            pass

        return email, None, "Basic" if email else "Bearer"

    def _build_headers(self, has_data: bool = False) -> Dict[str, str]:
        headers: Dict[str, str] = {
            "Accept": "application/json",
            "User-Agent": "DEAP-Backlog-Reconciler/1.0",
        }
        if has_data:
            headers["Content-Type"] = "application/json; charset=utf-8"

        if self.token:
            if self.token_type and self.token_type.lower() in ("bearer", "pat"):
                headers["Authorization"] = f"Bearer {self.token}"
            elif self.email:
                creds = f"{self.email}:{self.token}"
                b64_creds = base64.b64encode(creds.encode("utf-8")).decode("utf-8")
                headers["Authorization"] = f"Basic {b64_creds}"
            else:
                headers["Authorization"] = f"Bearer {self.token}"

        return headers

    def _api_request(
        self,
        endpoint: str,
        method: str = "GET",
        params: Optional[Dict[str, Any]] = None,
        data: Optional[Dict[str, Any]] = None,
    ) -> Tuple[int, Any, Dict[str, str]]:
        if self.offline:
            return 200, {}, {}

        if not self.server_url:
            raise ValueError("Jira server URL could not be resolved.")

        clean_endpoint = endpoint.lstrip("/")
        url = f"{self.server_url}/{clean_endpoint}"
        if params:
            query_str = urllib.parse.urlencode(params)
            url = f"{url}?{query_str}"

        headers = self._build_headers(has_data=(data is not None))

        body_bytes = None
        if data is not None:
            body_bytes = json.dumps(data).encode("utf-8")

        req = urllib.request.Request(url=url, data=body_bytes, headers=headers, method=method)

        attempt = 0
        while attempt < self.max_retries:
            attempt += 1
            try:
                with urllib.request.urlopen(req, context=self.ssl_context, timeout=self.timeout_sec) as resp:
                    status_code = resp.status
                    resp_headers = {k: v for k, v in resp.headers.items()}
                    raw_body = resp.read().decode("utf-8")
                    parsed_body = json.loads(raw_body) if raw_body.strip() else {}
                    return status_code, parsed_body, resp_headers
            except urllib.error.HTTPError as e:
                status_code = e.code
                error_headers = {k: v for k, v in e.headers.items()} if e.headers else {}
                raw_err = e.read().decode("utf-8", errors="ignore") if e.fp else ""

                if status_code in (429, 502, 503, 504) and attempt < self.max_retries:
                    retry_after = error_headers.get("Retry-After") or error_headers.get("retry-after")
                    if retry_after:
                        try:
                            sleep_time = float(retry_after)
                        except ValueError:
                            sleep_time = self.backoff_base_sec * (2 ** (attempt - 1))
                    else:
                        sleep_time = self.backoff_base_sec * (2 ** (attempt - 1))
                    time.sleep(sleep_time)
                    continue

                raise RuntimeError(
                    f"Jira API HTTP {status_code} Error on {method} {url}: {raw_err}"
                ) from e
            except urllib.error.URLError as e:
                if attempt >= self.max_retries:
                    raise RuntimeError(
                        f"Network transport failure connecting to {self.server_url}: {e.reason}"
                    ) from e
                time.sleep(self.backoff_base_sec * (2 ** (attempt - 1)))

        raise TimeoutError(f"Exceeded maximum retries ({self.max_retries}) for Jira API request: {url}")

    def list_issues(self) -> List[Dict[str, Any]]:
        """
        Fetch all project issues using Jira REST API search (JQL) with pagination.
        Returns list of issue dicts matching DEAP tracker schema:
        id, key, title, body, state, labels, issue_type.
        """
        if self.offline:
            print("[Notice] Jira tracker in offline mode. Operating in offline/local specification mode.")
            return []

        if not self.server_url:
            print("[Notice] Jira server URL not resolved (JIRA_SERVER_URL). Operating in offline/local specification mode.")
            return []

        if not self.token:
            print("[Notice] No Jira authentication token found (JIRA_PAT, JIRA_API_TOKEN, JIRA_TOKEN). "
                  "Operating in offline/local specification mode.")
            return []

        all_issues: List[Dict[str, Any]] = []
        start_at = 0
        max_results = 50
        jql = f"project = '{self.project_key}' ORDER BY key ASC" if self.project_key else "ORDER BY key ASC"
        endpoint = "rest/api/2/search"

        try:
            print(f"Fetching active and closed issues from Jira REST API ({self.server_url})...")
            while True:
                params = {
                    "jql": jql,
                    "startAt": start_at,
                    "maxResults": max_results,
                    "fields": "*all",
                }
                status_code, data, headers = self._api_request(endpoint, method="GET", params=params)
                if not isinstance(data, dict):
                    break

                issues_batch = data.get("issues", [])
                if not isinstance(issues_batch, list) or not issues_batch:
                    break

                for raw_issue in issues_batch:
                    fields = raw_issue.get("fields", {}) or {}
                    description = fields.get("description")
                    if isinstance(description, (dict, list)):
                        body = json.dumps(description)
                    elif description is None:
                        body = ""
                    else:
                        body = str(description)

                    status_obj = fields.get("status", {}) or {}
                    status_name = status_obj.get("name", "")
                    issuetype_obj = fields.get("issuetype", {}) or {}
                    issuetype_name = issuetype_obj.get("name", "")

                    state = status_name.upper() if status_name else "OPEN"

                    issue_dict = {
                        "id": str(raw_issue.get("id", "")),
                        "key": raw_issue.get("key", ""),
                        "number": raw_issue.get("key", ""),
                        "title": fields.get("summary", ""),
                        "summary": fields.get("summary", ""),
                        "body": body,
                        "state": state,
                        "status": status_name,
                        "labels": fields.get("labels", []) or [],
                        "issue_type": issuetype_name,
                        "fields": fields,
                    }
                    all_issues.append(issue_dict)

                total = data.get("total", len(all_issues))
                if (start_at + len(issues_batch)) >= total:
                    break
                start_at += len(issues_batch)

            return all_issues
        except Exception as e:
            err_msg = str(e)
            print(f"[Notice] Jira issue tracker unavailable ({err_msg}). Operating in offline/local specification mode.")
            return []

    def get_issue(self, issue_key: str) -> Optional[Dict[str, Any]]:
        """
        Fetch a single Jira issue by key (e.g. 'DEAP-123') and convert to DEAP tracker schema.
        """
        if self.offline or not self.server_url or not self.token:
            return None
        endpoint = f"rest/api/2/issue/{issue_key}"
        try:
            status_code, data, _ = self._api_request(endpoint, method="GET")
            if isinstance(data, dict) and ("key" in data or "id" in data):
                fields = data.get("fields", {}) or {}
                description = fields.get("description")
                if isinstance(description, (dict, list)):
                    body = json.dumps(description)
                elif description is None:
                    body = ""
                else:
                    body = str(description)

                status_obj = fields.get("status", {}) or {}
                status_name = status_obj.get("name", "")
                issuetype_obj = fields.get("issuetype", {}) or {}
                issuetype_name = issuetype_obj.get("name", "")

                return {
                    "id": str(data.get("id", "")),
                    "key": data.get("key", issue_key),
                    "number": data.get("key", issue_key),
                    "title": fields.get("summary", ""),
                    "body": body,
                    "state": status_name.upper() if status_name else "OPEN",
                    "labels": fields.get("labels", []) or [],
                    "issue_type": issuetype_name,
                    "fields": fields,
                }
        except Exception as e:
            print(f"  [Warning] Failed to fetch Jira issue '{issue_key}': {e}", file=sys.stderr)
        return None

    def create_issue(
        self,
        title: str,
        body: str = "",
        labels: Optional[List[str]] = None,
        issue_type: str = "Task",
        parent_key: Optional[str] = None,
        custom_fields: Optional[Dict[str, Any]] = None,
    ) -> Optional[Dict[str, Any]]:
        """
        Create a new Jira issue via POST /rest/api/2/issue.
        """
        if self.offline or not self.server_url or not self.token:
            return None
        endpoint = "rest/api/2/issue"
        fields: Dict[str, Any] = {
            "summary": title,
            "description": body or "",
            "issuetype": {"name": issue_type or "Task"},
        }
        if self.project_key:
            fields["project"] = {"key": self.project_key}
        if labels:
            fields["labels"] = labels if isinstance(labels, list) else [labels]
        if parent_key:
            fields["parent"] = {"key": parent_key}
        if custom_fields:
            fields.update(custom_fields)

        payload = {"fields": fields}
        try:
            status_code, data, _ = self._api_request(endpoint, method="POST", data=payload)
            if isinstance(data, dict):
                if "key" in data and "number" not in data:
                    data["number"] = data["key"]
                return data
        except Exception as e:
            print(f"  [Warning] Failed to create Jira issue '{title}': {e}", file=sys.stderr)
        return None

    def edit_issue(
        self,
        issue_key: str,
        title: Optional[str] = None,
        body: Optional[str] = None,
        labels: Optional[List[str]] = None,
        custom_fields: Optional[Dict[str, Any]] = None,
        **kwargs,
    ) -> bool:
        """
        Edit an existing Jira issue via PUT /rest/api/2/issue/{issue_key}.
        """
        if self.offline or not self.server_url or not self.token:
            return False

        if body is None and "description" in kwargs:
            body = kwargs["description"]
        if title is not None and body is None and ("\n" in title or len(title) > 255 or title.startswith("#")):
            body = title
            title = None

        endpoint = f"rest/api/2/issue/{issue_key}"
        fields: Dict[str, Any] = {}
        if title is not None:
            fields["summary"] = title
        if body is not None:
            fields["description"] = body
        if labels is not None:
            fields["labels"] = labels if isinstance(labels, list) else [labels]
        if custom_fields:
            fields.update(custom_fields)

        if not fields:
            return True

        payload = {"fields": fields}
        try:
            status_code, data, _ = self._api_request(endpoint, method="PUT", data=payload)
            return status_code in (200, 204)
        except Exception as e:
            print(f"  [Warning] Failed to edit Jira issue '{issue_key}': {e}", file=sys.stderr)
            return False

    def edit_issue_title(self, issue_key: Any, title: str) -> bool:
        return self.edit_issue(str(issue_key), title=title)

    def add_label(self, issue_key: str, label: str) -> bool:
        """
        Add a label to a Jira issue via PUT /rest/api/2/issue/{key} using Jira update operation.
        """
        if not label or self.offline or not self.server_url or not self.token:
            return False
        endpoint = f"rest/api/2/issue/{issue_key}"
        payload = {
            "update": {
                "labels": [{"add": label}]
            }
        }
        try:
            status_code, data, _ = self._api_request(endpoint, method="PUT", data=payload)
            return status_code in (200, 204)
        except Exception as e:
            print(f"  [Warning] Failed to add label '{label}' to Jira issue '{issue_key}': {e}", file=sys.stderr)
            return False

    def create_label(self, label: str, description: str = "", color: str = "#0E8A16") -> bool:
        """
        In Jira, labels are created dynamically upon assignment; return True.
        """
        return True

    def comment_issue(self, issue_key: str, comment_body: str) -> bool:
        """
        Add a comment to a Jira issue via POST /rest/api/2/issue/{key}/comment.
        """
        if not comment_body or self.offline or not self.server_url or not self.token:
            return False
        endpoint = f"rest/api/2/issue/{issue_key}/comment"
        payload = {"body": comment_body}
        try:
            status_code, data, _ = self._api_request(endpoint, method="POST", data=payload)
            return status_code in (200, 201)
        except Exception as e:
            print(f"  [Warning] Failed to comment on Jira issue '{issue_key}': {e}", file=sys.stderr)
            return False

    def transition_issue(
        self,
        issue_key: str,
        target_status: str,
        resolution_note: Optional[str] = None,
    ) -> bool:
        """
        Transition a Jira issue to a target workflow status via GET/POST /rest/api/2/issue/{key}/transitions.
        Matches transition by name (case-insensitive for 'Done', 'Resolved', 'Fixed', 'Completed', 'Ready for Review', etc.)
        and POSTs transition ID with optional comment.
        """
        if self.offline or not self.server_url or not self.token:
            return False
        endpoint = f"rest/api/2/issue/{issue_key}/transitions"
        try:
            status_code, data, _ = self._api_request(endpoint, method="GET")
            if not isinstance(data, dict):
                return False
            transitions = data.get("transitions", [])
            if not isinstance(transitions, list) or not transitions:
                print(f"  [Warning] No transitions available for Jira issue '{issue_key}'", file=sys.stderr)
                return False

            target_norm = target_status.strip().lower()
            if target_norm.startswith("status::") or target_norm.startswith("status:"):
                target_norm = target_norm.split(":", 1)[-1].lstrip(":")
            target_norm_clean = target_norm.replace("-", " ").replace("_", " ")

            matched_transition_id = None

            # 1. Exact or normalized match against transition name or target status name
            for t in transitions:
                t_name = str(t.get("name", "")).strip().lower()
                to_name = str(t.get("to", {}).get("name", "")).strip().lower()
                t_name_clean = t_name.replace("-", " ").replace("_", " ")
                to_name_clean = to_name.replace("-", " ").replace("_", " ")
                if (
                    target_norm in (t_name, to_name)
                    or target_norm_clean in (t_name_clean, to_name_clean)
                ):
                    matched_transition_id = t.get("id")
                    break

            # 2. Case-insensitive alias matching for common completion / review states
            if not matched_transition_id:
                completion_aliases = {
                    "done", "resolved", "fixed", "completed", "closed",
                    "resolve issue", "close issue", "finish"
                }
                review_aliases = {
                    "ready for review", "in review", "review", "ready-for-review", "under review"
                }

                is_completion_target = (
                    target_norm_clean in completion_aliases
                    or "fixed" in target_norm_clean
                    or "resolved" in target_norm_clean
                    or "done" in target_norm_clean
                    or "complete" in target_norm_clean
                    or "close" in target_norm_clean
                )
                is_review_target = (
                    target_norm_clean in review_aliases
                    or "review" in target_norm_clean
                )

                for t in transitions:
                    t_name = str(t.get("name", "")).strip().lower()
                    to_name = str(t.get("to", {}).get("name", "")).strip().lower()
                    t_name_clean = t_name.replace("-", " ").replace("_", " ")
                    to_name_clean = to_name.replace("-", " ").replace("_", " ")

                    if is_completion_target and (
                        t_name in completion_aliases
                        or to_name in completion_aliases
                        or t_name_clean in completion_aliases
                        or to_name_clean in completion_aliases
                    ):
                        matched_transition_id = t.get("id")
                        break
                    elif is_review_target and (
                        t_name in review_aliases
                        or to_name in review_aliases
                        or t_name_clean in review_aliases
                        or to_name_clean in review_aliases
                    ):
                        matched_transition_id = t.get("id")
                        break

            if not matched_transition_id:
                print(
                    f"  [Warning] Could not find transition matching '{target_status}' for Jira issue '{issue_key}'. "
                    f"Available: {[t.get('name') for t in transitions]}",
                    file=sys.stderr,
                )
                return False

            payload: Dict[str, Any] = {
                "transition": {"id": str(matched_transition_id)}
            }
            if resolution_note:
                payload["update"] = {
                    "comment": [
                        {"add": {"body": resolution_note}}
                    ]
                }

            post_status, _, _ = self._api_request(endpoint, method="POST", data=payload)
            return post_status in (200, 201, 204)
        except Exception as e:
            print(f"  [Warning] Failed to transition Jira issue '{issue_key}' to '{target_status}': {e}", file=sys.stderr)
            return False

    def create_issue_link(
        self,
        inward_key: str,
        outward_key: str,
        link_type: str = "Relates",
    ) -> bool:
        """
        Link two Jira issues via POST /rest/api/2/issueLink.
        """
        if self.offline or not self.server_url or not self.token:
            return False
        endpoint = "rest/api/2/issueLink"
        payload = {
            "type": {"name": link_type or "Relates"},
            "inwardIssue": {"key": inward_key},
            "outwardIssue": {"key": outward_key},
        }
        try:
            status_code, data, _ = self._api_request(endpoint, method="POST", data=payload)
            return status_code in (200, 201, 204)
        except Exception as e:
            print(f"  [Warning] Failed to create issue link between '{inward_key}' and '{outward_key}': {e}", file=sys.stderr)
            return False

JiraRESTProvider = JiraV2V3Provider

class GitHubCLIProvider:
    """
    GitHub CLI (gh) tracker provider adapter.
    Preserves full backwards compatibility with existing gh CLI workflows.
    """
    def __init__(self, workspace_dir: Optional[str] = None, offline: bool = False, tracker_rules: Optional[Dict[str, Any]] = None):
        self.workspace_dir = workspace_dir or os.getcwd()
        self.offline = offline
        self.tracker_rules = tracker_rules or DEFAULT_CODEBASE_RULES.get("tracker_rules", {})

    def list_issues(self) -> List[Dict[str, Any]]:
        if self.offline:
            print("[Notice] GitHub tracker in offline mode. Operating in offline/local specification mode.")
            return []
        commands = self.tracker_rules.get("commands", {})
        cmd = commands.get("list_issues")
        if not cmd:
            raise ValueError("Missing 'tracker_rules.commands.list_issues' in codebase_rules.json")
        print(f"Fetching active and closed issues from tracker provider 'github'...")
        res = subprocess.run(cmd, cwd=self.workspace_dir, capture_output=True, text=True, timeout=30)
        if res.returncode != 0:
            err_msg = res.stderr.strip() if res.stderr else ""
            err_lower = err_msg.lower()
            if "no git remotes found" in err_lower or "not a git repository" in err_lower or "could not read username" in err_lower:
                print(f"[Notice] Issue tracker unavailable ({err_msg}). Operating in offline/local specification mode.")
                return []
            raise Exception(f"Failed to fetch issues: {err_msg}")
        try:
            return json.loads(res.stdout)
        except json.JSONDecodeError:
            return []

    def create_issue(self, title: str, description: str, labels: Optional[Any] = None) -> Optional[Dict[str, Any]]:
        return None

    def edit_issue(self, issue_num: Any, description: str) -> bool:
        commands = self.tracker_rules.get("commands", {})
        edit_cmd_template = commands.get("edit_issue")
        if not edit_cmd_template:
            raise ValueError("Missing 'tracker_rules.commands.edit_issue' in codebase_rules.json")
        temp_path = None
        try:
            with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", delete=False) as tf:
                tf.write(description)
                temp_path = tf.name
            cmd = [str(issue_num) if c == "{number}" else (temp_path if c == "{temp_path}" else c) for c in edit_cmd_template]
            res = subprocess.run(cmd, cwd=self.workspace_dir, check=True, capture_output=True, timeout=30)
            return res.returncode == 0
        finally:
            if temp_path and os.path.exists(temp_path):
                os.remove(temp_path)

    def edit_issue_title(self, issue_num: Any, title: str) -> bool:
        commands = self.tracker_rules.get("commands", {})
        template = commands.get("edit_issue_title")
        if not template:
            return False
        cmd = [str(issue_num) if c == "{number}" else (title if c == "{title}" else c) for c in template]
        res = subprocess.run(cmd, cwd=self.workspace_dir, check=True, capture_output=True, timeout=30)
        return res.returncode == 0

    def add_label(self, issue_num: Any, label: str) -> bool:
        commands = self.tracker_rules.get("commands", {})
        add_template = commands.get("add_label") or commands.get("resolve_issue")
        if not add_template:
            return False
        cmd = [str(issue_num) if c == "{number}" else (label if c == "{label}" else c) for c in add_template]
        res = subprocess.run(cmd, cwd=self.workspace_dir, check=True, capture_output=True, timeout=30)
        return res.returncode == 0

    def comment_issue(self, issue_num: Any, comment: str) -> bool:
        commands = self.tracker_rules.get("commands", {})
        comment_template = commands.get("comment_issue")
        if not comment_template or not comment:
            return False
        cmd = [str(issue_num) if c == "{number}" else (comment if c == "{comment}" else c) for c in comment_template]
        res = subprocess.run(cmd, cwd=self.workspace_dir, check=True, capture_output=True, timeout=30)
        return res.returncode == 0

    def create_label(self, label: str, description: str = "", color: str = "0E8A16") -> bool:
        commands = self.tracker_rules.get("commands", {})
        create_template = commands.get("create_label")
        if not create_template:
            return False
        cmd = [
            label if c == "{label}" else (description if c == "{description}" else c)
            for c in create_template
        ]
        res = subprocess.run(cmd, cwd=self.workspace_dir, capture_output=True, timeout=30)
        return res.returncode == 0

def create_tracker_provider(
    provider_name: str,
    rules: Optional[Dict[str, Any]] = None,
    workspace_dir: Optional[str] = None,
    offline: bool = False,
    cli_gitlab_url: Optional[str] = None,
    cli_project: Optional[str] = None,
    cli_jira_url: Optional[str] = None,
    cli_jira_project: Optional[str] = None,
    cli_jira_email: Optional[str] = None,
):
    if provider_name == "gitlab":
        server_url = cli_gitlab_url or (rules.get("tracker_rules", {}).get("server_url") if rules else None)
        project_id = cli_project or (rules.get("tracker_rules", {}).get("project_id") if rules else None)
        return GitLabV4Provider(
            server_url=server_url,
            project_id=project_id,
            offline=offline,
            workspace_dir=workspace_dir,
        )
    elif provider_name == "jira":
        server_url = cli_jira_url or (rules.get("tracker_rules", {}).get("server_url") if rules else None)
        project_key = cli_jira_project or (rules.get("tracker_rules", {}).get("project_key") if rules else None) or (rules.get("tracker_rules", {}).get("project") if rules else None)
        email = cli_jira_email or (rules.get("tracker_rules", {}).get("email") if rules else None)
        return JiraV2V3Provider(
            server_url=server_url,
            project_key=project_key,
            email=email,
            offline=offline,
            workspace_dir=workspace_dir,
        )
    else:
        return GitHubCLIProvider(
            workspace_dir=workspace_dir,
            offline=offline,
            tracker_rules=(rules.get("tracker_rules", {}) if rules else None),
        )

def resolve_codebase_rules_path(workspace_dir: str):
    candidate_paths = [
        os.environ.get("CODEBASE_RULES_PATH"),
        os.path.join(workspace_dir, ".pipeline", "logical-ui", "codebase_rules.json"),
        os.path.join(workspace_dir, ".pipeline", "codebase_rules.json"),
        os.path.join(workspace_dir, "codebase_rules.json"),
    ]
    for path in candidate_paths:
        if path and os.path.exists(path):
            return path
    return None

def resolve_linter_script(workspace_dir: str):
    candidates = [
        os.path.join(workspace_dir, "skills", "spec-orchestrator", "scripts", "verify_model_coverage.py"),
        os.path.join(workspace_dir, "scripts", "verify_model_coverage.py"),
    ]
    for path in candidates:
        if os.path.exists(path):
            return path
    return None

def deep_merge(base, override):
    result = dict(base)
    for key, value in override.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = deep_merge(result[key], value)
        else:
            result[key] = value
    return result

def load_codebase_rules(workspace_dir, provider=None):
    rules = copy.deepcopy(DEFAULT_CODEBASE_RULES)
    rules_path = resolve_codebase_rules_path(workspace_dir)
    loaded = {}
    if rules_path:
        try:
            with open(rules_path, "r", encoding="utf-8") as f:
                content = json.load(f)
                if isinstance(content, dict):
                    loaded = content
                    rules = deep_merge(rules, loaded)
        except Exception as e:
            print(f"Warning: Failed to load codebase_rules.json from {rules_path}: {e}")
    
    # If provider is gitlab (either passed explicitly, in env, or configured in loaded rules)
    effective_provider = provider or rules.get("tracker_rules", {}).get("provider")
    if effective_provider == "gitlab":
        gitlab_defaults = {"tracker_rules": copy.deepcopy(DEFAULT_GITLAB_TRACKER_RULES)}
        rules = deep_merge(rules, gitlab_defaults)
        if loaded.get("tracker_rules", {}).get("provider") == "gitlab":
            rules["tracker_rules"] = deep_merge(rules["tracker_rules"], loaded["tracker_rules"])
        else:
            rules["tracker_rules"]["labels"] = copy.deepcopy(DEFAULT_GITLAB_TRACKER_RULES["labels"])
            rules["tracker_rules"]["keys"] = copy.deepcopy(DEFAULT_GITLAB_TRACKER_RULES["keys"])
        
        # Ensure DEFAULT_GITLAB_TRACKER_RULES["keys"] take precedence for GitLab
        gitlab_keys = copy.deepcopy(DEFAULT_GITLAB_TRACKER_RULES["keys"])
        if isinstance(rules.get("tracker_rules", {}).get("keys"), dict):
            for k, v in rules["tracker_rules"]["keys"].items():
                if k not in gitlab_keys:
                    gitlab_keys[k] = v
        rules["tracker_rules"]["keys"] = gitlab_keys

        # Ensure GitLab scoped labels take precedence whenever labels are absent or contain GitHub default unscoped label values
        if "labels" not in rules["tracker_rules"] or not isinstance(rules["tracker_rules"]["labels"], dict):
            rules["tracker_rules"]["labels"] = copy.deepcopy(DEFAULT_GITLAB_TRACKER_RULES["labels"])
        else:
            github_unscoped = {"epic", "feature", "user-story", "use-case", "user_story", "use_case", "status:fixed-resolved"}
            gitlab_default_labels = DEFAULT_GITLAB_TRACKER_RULES["labels"]
            for label_key, default_val in gitlab_default_labels.items():
                curr_val = rules["tracker_rules"]["labels"].get(label_key)
                if curr_val is None or curr_val in github_unscoped:
                    rules["tracker_rules"]["labels"][label_key] = default_val

        rules["tracker_rules"]["provider"] = "gitlab"

    elif effective_provider == "jira":
        jira_defaults = {"tracker_rules": copy.deepcopy(DEFAULT_JIRA_TRACKER_RULES)}
        rules = deep_merge(rules, jira_defaults)
        if loaded.get("tracker_rules", {}).get("provider") == "jira":
            rules["tracker_rules"] = deep_merge(rules["tracker_rules"], loaded["tracker_rules"])
        else:
            rules["tracker_rules"]["labels"] = copy.deepcopy(DEFAULT_JIRA_TRACKER_RULES["labels"])
            rules["tracker_rules"]["keys"] = copy.deepcopy(DEFAULT_JIRA_TRACKER_RULES["keys"])
        
        # Ensure DEFAULT_JIRA_TRACKER_RULES["keys"] take precedence for Jira
        jira_keys = copy.deepcopy(DEFAULT_JIRA_TRACKER_RULES["keys"])
        if isinstance(rules.get("tracker_rules", {}).get("keys"), dict):
            for k, v in rules["tracker_rules"]["keys"].items():
                if k not in jira_keys:
                    jira_keys[k] = v
        rules["tracker_rules"]["keys"] = jira_keys

        # Ensure Jira scoped labels take precedence whenever labels are absent or contain GitHub default unscoped label values
        if "labels" not in rules["tracker_rules"] or not isinstance(rules["tracker_rules"]["labels"], dict):
            rules["tracker_rules"]["labels"] = copy.deepcopy(DEFAULT_JIRA_TRACKER_RULES["labels"])
        else:
            github_unscoped = {"epic", "feature", "user-story", "use-case", "user_story", "use_case", "status:fixed-resolved"}
            jira_default_labels = DEFAULT_JIRA_TRACKER_RULES["labels"]
            for label_key, default_val in jira_default_labels.items():
                curr_val = rules["tracker_rules"]["labels"].get(label_key)
                if curr_val is None or curr_val in github_unscoped:
                    rules["tracker_rules"]["labels"][label_key] = default_val

        rules["tracker_rules"]["provider"] = "jira"

    return rules

def get_git_remote_repo(workspace_dir):
    try:
        remote_info = get_git_remote_info(workspace_dir)
        if remote_info and remote_info.get("project_path"):
            return remote_info["project_path"]
    except Exception as e:
        print(f"Warning: Failed to auto-detect git remote: {e}")
    return None

def get_upstream_repository(rules, workspace_dir):
    env_repo = os.environ.get("UPSTREAM_REPOSITORY") or os.environ.get("GIT_REMOTE_ORIGIN")
    if env_repo:
        return env_repo
    git_repo = get_git_remote_repo(workspace_dir)
    if git_repo:
        return git_repo
    if rules and isinstance(rules, dict):
        return rules.get("meta", {}).get("upstream_repository", "gintatkinson/DEAP01-spec-core")
    return "gintatkinson/DEAP01-spec-core"

def format_issue_reference(issue_id, tracker_rules):
    issue_id_str = str(issue_id)
    if issue_id_str.isdigit():
        prefix = tracker_rules.get("numeric_prefix", "#")
        return f"{prefix}{issue_id_str}"
    else:
        prefix = tracker_rules.get("alphanumeric_prefix", "")
        return f"{prefix}{issue_id_str}"

def normalize_title(title, rules=None):
    if not title:
        return ""
    # Strip quotes and leading/trailing whitespace
    title = title.strip().strip('"\'')
    # Strip common prefixes (e.g., epic-01:, feat-02:, us-03:, uc-04:, etc.)
    regex = r'^(epic|feature|feat|user[- ]story|use[- ]case|us|uc)[s]?(?:[- ]*\d+)?\s*[:\-]?\s*'
    stripped = re.sub(regex, '', title, flags=re.IGNORECASE)
    if stripped.strip():
        title = stripped
    # Normalize hyphens to spaces to handle typographic variations
    title = title.replace("-", " ")
    # Strip any remaining punctuation and normalize spacing
    title = re.sub(r'[^\w\s]', '', title)
    title = " ".join(title.split())
    return title.lower()


def normalize_spec_slug(title, rules=None):
    """
    Standardized slugification that preserves stop words.
    Converts 'Fiber Cable and Strand Inventory' to 'fiber-cable-and-strand-inventory'.
    """
    if not title:
        return ""
    # Strip quotes and leading/trailing whitespace
    title = title.strip().strip('"\'')
    # Strip common prefixes
    regex = r'^(epic|feature|feat|user[- ]story|use[- ]case|us|uc)[s]?(?:[- ]*\d+)?\s*[:\-]?\s*'
    stripped = re.sub(regex, '', title, flags=re.IGNORECASE)
    if stripped.strip():
        title = stripped
    # Normalize hyphens to spaces to handle typographic variations
    title = title.replace("-", " ")
    # Strip any remaining punctuation and normalize spacing
    title = re.sub(r'[^\w\s]', '', title)
    # Join with hyphens to form a slug, preserving all words
    title = "-".join(title.split())
    return title.lower()


def normalize_label(label):
    """Reduce a tracker label to the form every label comparison happens in (#329).

    The reconciler compared labels for exact equality, so an issue filed with
    `"User Story"` lowercased to `"user story"`, never matched the configured
    `"user-story"`, was bucketed nowhere, and stayed orphaned while its specification
    reported "no issue on the tracker". Case and word separators are presentation, not
    identity: `"User Story"`, `"user story"` and `"user_story"` all name one label.

    Only whitespace, underscores and hyphens fold. A namespaced label such as
    `status:fixed-resolved` keeps its colon, because that *is* part of the name.

    #313 (package N3) introduced `issue_has_label` with deliberately exact matching and
    recorded case-insensitivity as belonging to this issue. Every comparison site in the
    module now routes through here so there is one rule rather than two.
    """
    if not label:
        return ""
    text = str(label).strip().lower()
    if not text:
        return ""
    return re.sub(r"[\s_\-]+", "-", text).strip("-")


# The spec type a reference declares about itself, keyed by the spelling found. The
# values are the constitutional type names; `SPEC_TYPE_ALIASES` is consulted only with a
# separator-folded key, so one entry covers "User Story", "user-story" and "user_story".
SPEC_TYPE_ALIASES = {
    "epic": "epic",
    "epics": "epic",
    "feature": "feature",
    "features": "feature",
    "feat": "feature",
    "user story": "user-story",
    "user stories": "user-story",
    "us": "user-story",
    "use case": "use-case",
    "use cases": "use-case",
    "uc": "use-case",
}

# A type word only marks a type when a separator, a digit or the end of the reference
# follows it. Without that boundary `us` would claim "User Access Control" and `uc`
# would claim "UCS Migration" — the same over-eager prefix stripping that produced #319
# in the first place, reintroduced in the code meant to contain it.
_REFERENCE_TYPE_RE = re.compile(
    r'^\s*["\'#]*\s*'
    r'(?P<type>epics?|features?|feat|user[-_ ]?stor(?:y|ies)|use[-_ ]?cases?|us|uc)'
    r'(?=[\s\-_:.#]|\d|$)',
    re.IGNORECASE,
)


def spec_type_of_reference(reference):
    """The spec type a reference names explicitly, or None when it is type-neutral.

    `"feat-07-geo-location"` declares itself a Feature; `"Geo Location"` and `"#101"`
    declare nothing. This is the namespace discriminator #319 asks for: entity
    resolution must isolate namespaces by entity type, and a reference that names its
    own type is the only evidence available for doing so.
    """
    if reference is None:
        return None
    match = _REFERENCE_TYPE_RE.match(str(reference))
    if not match:
        return None
    key = re.sub(r"[\s\-_]+", " ", match.group("type").strip().lower())
    return SPEC_TYPE_ALIASES.get(key)


def extract_title(filepath):
    try:
        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read(2048)  # Read first 2KB
        
        # Try finding title in table or YAML frontmatter
        table_title_match = re.search(r'^\s*\|\s*\*\*Title\*\*\s*\|\s*(.*?)\s*\|\s*$', content, re.MULTILINE | re.IGNORECASE)
        if table_title_match:
            return table_title_match.group(1).strip()

        title_match = re.search(r'^title:\s*(["\']?)(.*?)\1\s*$', content, re.MULTILINE)
        title_match = re.search(r'^title:\s*(["\']?)(.*?)\1\s*$', content, re.MULTILINE)
        if title_match:
            return title_match.group(2).strip()
            
        # Fallback to # H1 title
        h1_match = re.search(r'^#\s+(.*?)$', content, re.MULTILINE)
        if h1_match:
            return h1_match.group(1).strip()
    except Exception as e:
        print(f"Error reading title from {filepath}: {e}")
    return None

def extract_epic_from_body(body_content):
    """
    Extracts the parent epic reference (filename, slug, or issue ID) from markdown body content.
    """
    if not body_content:
        return None
        
    # 1. Search for any markdown link pointing to an epic file under /epics/ or epics/ or starting with epic-
    links = re.findall(r'\[([^\]]+)\]\(([^)]+)\)', body_content)
    for link_text, link_url in links:
        url_lower = link_url.lower()
        if "epics/" in url_lower or "epic-" in url_lower:
            filename = os.path.basename(link_url)
            if filename.endswith(".md"):
                filename = filename[:-3]
            if "epic" in filename.lower():
                return filename
            if "epic" in link_text.lower():
                return link_text

    # 2. Line-by-line scanning for "Parent Epic" heading/section or inline references
    lines = body_content.splitlines()
    parent_epic_section = False
    for line in lines:
        line_stripped = line.strip()
        if re.search(r'^#+\s+Parent\s+Epic', line_stripped, re.IGNORECASE):
            parent_epic_section = True
            continue
        
        is_parent_epic_line = "parent epic" in line_stripped.lower()
        if parent_epic_section or is_parent_epic_line:
            # Check for issue ID first (e.g. #101)
            issue_match = re.search(r'#(\d+)\b', line_stripped)
            if issue_match:
                return issue_match.group(0)
                
            # Check for issue ID placeholder with potential link (e.g. - [ ] #[EpicID] - [Title](../epics/epic-01.md))
            placeholder_match = re.search(r'#\[EpicIssueID\]|#\[EpicID\]', line_stripped, re.IGNORECASE)
            if placeholder_match:
                link_match = re.search(r'\[([^\]]+)\]\(([^)]+)\)', line_stripped)
                if link_match:
                    link_text, link_url = link_match.groups()
                    filename = os.path.basename(link_url)
                    if filename.endswith(".md"):
                        filename = filename[:-3]
                    return filename
            
            # Check for generic markdown link in Parent Epic line/section
            link_match = re.search(r'\[([^\]]+)\]\(([^)]+)\)', line_stripped)
            if link_match:
                link_text, link_url = link_match.groups()
                filename = os.path.basename(link_url)
                if filename.endswith(".md"):
                    filename = filename[:-3]
                return filename
                
            # Check for explicit text pattern e.g. "Parent Epic: epic-01-geo-location"
            val_match = re.search(r'(?:parent\s+epic\s*[:\-]\s*|\*\*\s*parent\s+epic\s*\*\*\s*[:\-]\s*)([^\n]+)', line_stripped, re.IGNORECASE)
            if val_match:
                val = val_match.group(1).strip().strip('[]-* ')
                if val:
                    return val
            
            # If we hit another heading, stop section scan
            if parent_epic_section and line_stripped.startswith('#'):
                parent_epic_section = False
                
    # 3. Global fallback scan for "Parent Epic" inline references
    for line in lines:
        line_stripped = line.strip()
        if "parent epic" in line_stripped.lower():
            link_match = re.search(r'\[([^\]]+)\]\(([^)]+)\)', line_stripped)
            if link_match:
                link_text, link_url = link_match.groups()
                filename = os.path.basename(link_url)
                if filename.endswith(".md"):
                    filename = filename[:-3]
                return filename
            issue_match = re.search(r'#(\d+)\b', line_stripped)
            if issue_match:
                return issue_match.group(0)
                
    return None

def get_all_issues(rules=None, provider_adapter=None):
    if provider_adapter:
        return provider_adapter.list_issues()
    if not rules:
        raise ValueError("Configuration rules are missing.")
    tracker_rules = rules.get("tracker_rules")
    if not tracker_rules:
        raise ValueError("Missing 'tracker_rules' in codebase_rules.json")
    provider = tracker_rules.get("provider", "github")
    
    if provider == "gitlab":
        adapter = GitLabV4Provider(
            server_url=tracker_rules.get("server_url"),
            project_id=tracker_rules.get("project_id"),
        )
        return adapter.list_issues()
    elif provider == "jira":
        adapter = JiraV2V3Provider(
            server_url=tracker_rules.get("server_url"),
            project_key=tracker_rules.get("project_key") or tracker_rules.get("project"),
            email=tracker_rules.get("email"),
        )
        return adapter.list_issues()

    commands = tracker_rules.get("commands")
    if not commands or "list_issues" not in commands:
        raise ValueError("Missing 'tracker_rules.commands.list_issues' in codebase_rules.json")
    
    print(f"Fetching active and closed issues from tracker provider '{provider}'...")
    cmd = commands["list_issues"]
    res = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
    if res.returncode != 0:
        err_msg = res.stderr.strip() if res.stderr else ""
        err_lower = err_msg.lower()
        if "no git remotes found" in err_lower or "not a git repository" in err_lower or "could not read username" in err_lower:
            print(f"[Notice] Issue tracker unavailable ({err_msg}). Operating in offline/local specification mode.")
            return []
        raise Exception(f"Failed to fetch issues: {err_msg}")
    try:
        return json.loads(res.stdout)
    except json.JSONDecodeError:
        return []

def update_checklist_in_file(filepath, issue_dict, rules=None):
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    pattern = tracker_rules.get("dependency_regex", r"(-\s*\[\s*([ xX])\s*\]\s*(#|#\[|\#\s*)?([A-Za-z0-9\-]+))")
    PLACEHOLDER_PATTERN = re.compile(r'^(IssueID|EpicIssueID|StoryIssueID|FeatureIssueID|UseCaseIssueID|StoryID|N/A)\]?$')
    
    updated_content = content
    all_deps_closed = True
    has_deps = False
    
    keys = tracker_rules.get("keys", {})
    state_key = keys.get("state", "state")
    closed_state = keys.get("closed_state_value", "CLOSED").upper()
    
    matches = re.findall(pattern, content)
    for match_tuple in matches:
        # Support variable number of groups depending on user-configured regex
        if isinstance(match_tuple, str):
            full_match = match_tuple
            mark = ' '
            prefix = ''
            dep_num_str = match_tuple
        else:
            full_match = match_tuple[0]
            mark = match_tuple[1]
            prefix = match_tuple[2] if len(match_tuple) > 2 else ''
            dep_num_str = match_tuple[3] if len(match_tuple) > 3 else match_tuple[-1]

        # 1. Skip plain markdown checkboxes that have no issue reference prefix, but flag if unchecked
        if not prefix:
            if mark == ' ':
                has_deps = True
                all_deps_closed = False
            continue

        # 2. Skip unresolved template placeholders
        if isinstance(dep_num_str, str) and PLACEHOLDER_PATTERN.match(dep_num_str):
            ref_str = format_issue_reference(dep_num_str, tracker_rules)
            print(f"  [Deferred] Unresolved placeholder {ref_str} in {os.path.basename(filepath)} — skipping")
            has_deps = True
            all_deps_closed = False
            continue
        has_deps = True
        dep_num = int(dep_num_str) if dep_num_str.isdigit() else dep_num_str
        dep_issue = issue_dict.get(dep_num)
        
        if dep_issue is None:
            ref_str = format_issue_reference(dep_num, tracker_rules)
            print(f"  [Warning] Dependency {ref_str} not found in tracker for {os.path.basename(filepath)} — skipping item")
            all_deps_closed = False
            continue
            
        is_closed = (str(dep_issue[state_key]).upper() == closed_state) or is_already_resolved(dep_issue, rules)
        target_mark = 'x' if is_closed else ' '
        
        if mark != target_mark:
            # Replace the specific checkbox character
            old_box = f"[{mark}]"
            new_box = f"[{target_mark}]"
            updated_content = updated_content.replace(full_match, full_match.replace(old_box, new_box, 1), 1)
            ref_str = format_issue_reference(dep_num, tracker_rules)
            print(f"  [Checklist] Updated dependency {ref_str} to [{target_mark}] in {os.path.basename(filepath)}")
            
        if not is_closed:
            all_deps_closed = False

    # Check for any remaining unchecked checkboxes in updated_content
    # A specification item cannot be considered completed if any blocker checkbox remains unchecked (- [ ])
    if re.search(r'-\s*\[\s*\]', updated_content) or re.search(r'-\s*\[ \]', updated_content):
        has_deps = True
        all_deps_closed = False

    if updated_content != content:
        updated_content = write_markdown_file(filepath, updated_content)
            
    return updated_content, (has_deps and all_deps_closed)

def convert_frontmatter_to_table(content):
    if not content:
        return content
        
    stripped = content.lstrip()
    if (
        stripped.startswith("| Attribute | Specification Detail |")
        or stripped.startswith("| Metadata | Value |")
    ):
        return content

    cleaned_comments = re.sub(r'^<!--.*?-->\s*', '', content, flags=re.DOTALL).lstrip()
    if (
        cleaned_comments.startswith("| Attribute | Specification Detail |")
        or cleaned_comments.startswith("| Metadata | Value |")
    ):
        return content

    if not content.startswith("---"):
        return content
    
    parts = content.split("---", 2)
    if len(parts) < 3:
        return content
        
    frontmatter_text = parts[1]
    body_text = parts[2].lstrip()
    
    try:
        data = yaml.safe_load(frontmatter_text.replace('\x01', ''))
        if not isinstance(data, dict):
            return content
    except Exception as e:
        print(f"Error parsing frontmatter YAML: {e}")
        return content
    
    table_lines = [
        "| Attribute | Specification Detail |",
        "| :--- | :--- |"
    ]
    
    for key, value in data.items():
        if isinstance(value, list):
            val = ", ".join(str(item) for item in value)
        elif value is None:
            val = ""
        else:
            val = str(value)
        
        val = val.replace('\n', '<br>').replace('|', '\\|')
        label = str(key).replace('_', ' ').title()
        table_lines.append(f"| **{label}** | {val} |")
        
    table_text = "\n".join(table_lines) + "\n\n"
    return table_text + body_text

def deduplicate_markdown_sections(content):
    lines = content.splitlines()
    seen_headers = set()
    output_lines = []
    skip_section = False
    for line in lines:
        header_match = re.match(r'^(#+)\s+(.*)$', line)
        if header_match:
            header_level = header_match.group(1)
            header_title = header_match.group(2).strip().lower()
            norm_title = re.sub(r'^\d+\.\s*', '', header_title)
            section_key = f"{header_level} {norm_title}"
            if section_key in seen_headers:
                skip_section = True
            else:
                seen_headers.add(section_key)
                skip_section = False
        if not skip_section:
            output_lines.append(line)
    return "\n".join(output_lines) + "\n"

def get_blob_url_base(rules=None, workspace_dir=None, branch=None):
    if workspace_dir is None:
        workspace_dir = find_workspace_dir(os.getcwd())

    upstream_repo = get_upstream_repository(rules, workspace_dir) or "gintatkinson/DEAP01-spec-core"
    provider_name = detect_tracker_provider(rules=rules, workspace_dir=workspace_dir)

    if not branch or branch == "HEAD":
        branch = get_current_branch(workspace_dir) or "main"
        if branch == "HEAD":
            branch = "main"

    remote_info = get_git_remote_info(workspace_dir) if workspace_dir else None
    remote_info = remote_info or {}

    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    server_url_override = tracker_rules.get("server_url") or tracker_rules.get("url")

    if provider_name == "gitlab":
        is_gitlab_remote = remote_info.get("is_gitlab", False)
        server_url = (
            server_url_override
            or (remote_info.get("server_url") if is_gitlab_remote else None)
            or os.environ.get("GITLAB_URL")
            or os.environ.get("CI_SERVER_URL")
            or "https://gitlab.com"
        ).rstrip("/")

        proj_path = (
            tracker_rules.get("project_id")
            or remote_info.get("project_path")
            or os.environ.get("CI_PROJECT_PATH")
            or upstream_repo
            or ""
        ).strip("/")

        if proj_path.startswith("http://") or proj_path.startswith("https://"):
            parsed_proj = urllib.parse.urlparse(proj_path)
            server_url = f"{parsed_proj.scheme}://{parsed_proj.netloc}".rstrip("/")
            proj_path = parsed_proj.path.lstrip("/")

        if proj_path.endswith(".git"):
            proj_path = proj_path[:-4]

        if proj_path.startswith("github.com/"):
            proj_path = proj_path[len("github.com/"):]

        return f"{server_url}/{proj_path}/-/blob/{branch}"
    else:
        is_gitlab_remote = remote_info.get("is_gitlab", False)
        server_url = (
            server_url_override
            if (server_url_override and not is_gitlab_remote)
            else (remote_info.get("server_url") if (remote_info.get("server_url") and not is_gitlab_remote) else "https://github.com")
        ).rstrip("/")

        proj_path = (
            tracker_rules.get("project_id")
            or tracker_rules.get("project_key")
            or remote_info.get("project_path")
            or upstream_repo
            or ""
        ).strip("/")

        if proj_path.startswith("http://") or proj_path.startswith("https://"):
            parsed_proj = urllib.parse.urlparse(proj_path)
            server_url = f"{parsed_proj.scheme}://{parsed_proj.netloc}".rstrip("/")
            proj_path = parsed_proj.path.lstrip("/")

        if proj_path.endswith(".git"):
            proj_path = proj_path[:-4]

        if proj_path.startswith("github.com/"):
            proj_path = proj_path[len("github.com/"):]
        elif proj_path.startswith("gitlab.com/"):
            proj_path = proj_path[len("gitlab.com/"):]

        if is_gitlab_remote:
            gitlab_server = (remote_info.get("server_url") or "https://gitlab.com").rstrip("/")
            return f"{gitlab_server}/{proj_path}/-/blob/{branch}"

        return f"{server_url}/{proj_path}/blob/{branch}"

def rewrite_header_repository_urls(content, active_repo, rules=None, workspace_dir=None):
    if not content or not active_repo:
        return content
    parts = active_repo.split('/')
    active_name = parts[-1].lower()
    active_owner = parts[0].lower() if len(parts) > 1 else ""

    provider_name = detect_tracker_provider(rules=rules, workspace_dir=workspace_dir)

    def replacer(match):
        full_url = match.group(0)
        url_owner = match.group(1)
        url_repo = match.group(2)
        url_owner_lower = url_owner.lower()
        url_repo_lower = url_repo.lower()

        is_target_repo = (
            (url_owner_lower == active_owner and url_repo_lower == active_name) or
            (url_repo_lower == active_name) or
            (url_repo_lower == "deap-spec-core") or
            (url_repo_lower == "deap01-spec-core") or
            ("pipeline-repo" in url_repo_lower)
        )

        if is_target_repo:
            if provider_name == "gitlab":
                remote_info = get_git_remote_info(workspace_dir) if workspace_dir else None
                server_url = (
                    (remote_info.get("server_url") if remote_info else None)
                    or (rules.get("tracker_rules", {}).get("server_url") if rules else None)
                    or os.environ.get("GITLAB_URL")
                    or os.environ.get("CI_SERVER_URL")
                    or "https://gitlab.com"
                ).rstrip("/")
                return f"{server_url}/{active_repo}/-/blob/"
            return f"https://github.com/{active_repo}/blob/"
        return full_url

    pattern = r'https://(?:github\.com|gitlab\.com|[^/\s]+)/([^/]+)/([^/]+)/(?:-\\)?blob/'
    return re.sub(pattern, replacer, content)

def sanitize_source_references(content, workspace_dir=None, rules=None):
    if not content:
        return content

    if workspace_dir is None:
        workspace_dir = find_workspace_dir(os.getcwd())

    upstream_repo = get_upstream_repository(rules, workspace_dir) or "gintatkinson/DEAP01-spec-core"
    content = rewrite_header_repository_urls(content, upstream_repo, rules=rules, workspace_dir=workspace_dir)
    branch = get_current_branch(workspace_dir)
    if not branch or branch == "HEAD":
        branch = "main"

    blob_base = get_blob_url_base(rules=rules, workspace_dir=workspace_dir, branch=branch)

    abs_workspace = os.path.abspath(workspace_dir).rstrip("/\\")
    real_workspace = os.path.realpath(workspace_dir).rstrip("/\\")
    repo_name = upstream_repo.split("/")[-1] if "/" in upstream_repo else upstream_repo

    def replacer(match):
        full_uri = match.group(0)
        path_part = match.group(1)

        if abs_workspace != "/" and path_part.startswith(abs_workspace):
            rel = path_part[len(abs_workspace):].lstrip("/")
            return f"{blob_base}/{rel}"
        elif real_workspace != "/" and path_part.startswith(real_workspace):
            rel = path_part[len(real_workspace):].lstrip("/")
            return f"{blob_base}/{rel}"

        repo_substr = f"/{repo_name}/"
        if repo_substr in path_part:
            rel = path_part.split(repo_substr, 1)[1]
            return f"{blob_base}/{rel}"

        parts = path_part.split("/")
        if len(parts) > 3 and parts[1] in ("Users", "home"):
            rel = "/".join(parts[3:])
            return f"{blob_base}/{rel}"

        return full_uri

    pattern = r'file://(/[^\s\)\>"\']+)'
    return re.sub(pattern, replacer, content)

def expand_relative_links_for_tracker(content, filepath=None, rules=None, workspace_dir=None, branch=None):
    """Expand relative markdown links to full blob URLs for web issue tracker payloads.
    
    Local git files maintain clean, canonical relative links for branch isolation
    and offline navigation. During tracker dispatch, this transforms relative file links
    into provider-aware web blob URLs (e.g. GitHub /blob/<branch>/... or GitLab /-/blob/<branch>/...)
    so links resolve correctly when viewed in issue tracker web interfaces (#45).
    """
    if not content:
        return content

    if workspace_dir is None:
        if filepath:
            workspace_dir = find_workspace_dir(filepath)
        if not workspace_dir:
            workspace_dir = find_workspace_dir(os.getcwd())

    if not branch or branch == "HEAD":
        branch = get_current_branch(workspace_dir)
        if not branch or branch == "HEAD":
            branch = "main"

    blob_base = get_blob_url_base(rules=rules, workspace_dir=workspace_dir, branch=branch)
    if not blob_base:
        return content

    slash_chars = "/" + chr(92)
    abs_workspace = os.path.abspath(workspace_dir).rstrip(slash_chars) if workspace_dir else ""
    rel_file_dir = ""
    if filepath and abs_workspace:
        abs_filepath = os.path.abspath(filepath)
        abs_file_dir = os.path.dirname(abs_filepath)
        try:
            r = os.path.relpath(abs_file_dir, abs_workspace).replace(chr(92), "/")
            if not r.startswith("..") and r != ".":
                rel_file_dir = r
        except ValueError:
            rel_file_dir = ""

    def replace_link(match):
        label = match.group(1)
        target = match.group(2).strip()

        if not target or target.startswith(("http://", "https://", "mailto:", "#", "git@", "ftp://", "tel:")):
            return match.group(0)

        fragment = ""
        target_path = target
        if "#" in target:
            target_path, fragment = target.split("#", 1)
            fragment = "#" + fragment

        if not target_path:
            return match.group(0)

        if target_path.startswith("/"):
            norm_target = os.path.normpath(target_path.lstrip("/")).replace(chr(92), "/")
        elif target_path.startswith(("../", "./")):
            if rel_file_dir:
                norm_target = os.path.normpath(os.path.join(rel_file_dir, target_path)).replace(chr(92), "/")
            else:
                cleaned = target_path.lstrip("./").lstrip("../")
                if cleaned.startswith(("epics/", "features/", "use-cases/", "user-stories/")):
                    norm_target = f"docs/{cleaned}"
                else:
                    norm_target = cleaned
        else:
            if target_path.startswith(("docs/", "rules/", "skills/", "schema/", ".pipeline/", "scripts/", "tests/", "assets/", "lib/", "bin/")):
                norm_target = os.path.normpath(target_path).replace(chr(92), "/")
            elif abs_workspace and os.path.exists(os.path.join(abs_workspace, target_path)):
                norm_target = os.path.normpath(target_path).replace(chr(92), "/")
            elif rel_file_dir:
                norm_target = os.path.normpath(os.path.join(rel_file_dir, target_path)).replace(chr(92), "/")
            else:
                norm_target = os.path.normpath(target_path).replace(chr(92), "/")

        if norm_target.startswith("../") or norm_target == "..":
            return match.group(0)

        norm_target = norm_target.lstrip("./").lstrip("/")
        blob_url = f"{blob_base.rstrip('/')}/{norm_target}{fragment}"
        return f"[{label}]({blob_url})"

    pattern = r'\[([^\]]+)\]\(([^)]+)\)'
    return re.sub(r'\[([^\]]+)\]\(([^)]+)\)', replace_link, content)


def sanitize_mermaid_diagrams(content):
    if not content or "```mermaid" not in content:
        return content

    lines = content.splitlines()
    in_mermaid = False
    sanitized_lines = []

    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if stripped.startswith("```mermaid"):
            in_mermaid = True
            sanitized_lines.append(line)
            i += 1
            continue
        elif in_mermaid and stripped.startswith("```"):
            in_mermaid = False
            sanitized_lines.append(line)
            i += 1
            continue

        if in_mermaid and i + 1 < len(lines):
            next_line = lines[i + 1]
            next_stripped = next_line.strip()
            if not next_stripped.startswith("```"):
                arrow_match = re.search(r'(-+>+|=+>+|--|-\.->?|-\.)\s*$', stripped)
                starts_with_gt = next_stripped.startswith('>')

                if arrow_match or starts_with_gt:
                    arrow_op = arrow_match.group(1) if arrow_match else ""
                    rest_next = next_stripped

                    if starts_with_gt:
                        if arrow_op == "->":
                            line = re.sub(r'->\s*$', '->>', line)
                        elif arrow_op == "--":
                            line = re.sub(r'--\s*$', '-->', line)
                        rest_next = re.sub(r'^>\s*', '', next_stripped)

                    joined = f"{line.rstrip()} {rest_next}"
                    sanitized_lines.append(joined)
                    i += 2
                    continue

        sanitized_lines.append(line)
        i += 1

    return "\n".join(sanitized_lines) + ("\n" if content.endswith("\n") else "")

def sanitize_latex_delimiters_for_tracker(content: str) -> str:
    """Sanitize non-mathematical alphanumeric identifiers wrapped in $...$ to bold text.
    
    Transforms tokens like $SC-01$, $H-1$, $OSO-11$, $L-1$, $UCA-1$, $REQ-SYS-001$ into
    **SC-01**, **H-1**, etc., prior to tracker API upload, preventing KaTeX math rendering
    corruption (such as triple-text duplication) in tracker web UIs while preserving
    genuine mathematical formulas intact.
    """
    if not content:
        return content
    # First handle cases already enclosed in bold like **$SC-01$**
    content = re.sub(r"\*\*+\$([A-Za-z][A-Za-z0-9]*-[0-9A-Za-z_.-]+)\$\*\*+", r"**\1**", content)
    # Then handle standard inline $SC-01$
    content = re.sub(r"\$([A-Za-z][A-Za-z0-9]*-[0-9A-Za-z_.-]+)\$", r"**\1**", content)
    return content

def write_markdown_file(filepath, content, workspace_dir=None, rules=None):
    if workspace_dir is None:
        workspace_dir = find_workspace_dir(filepath)
    sanitized_content = sanitize_source_references(content, workspace_dir=workspace_dir, rules=rules)
    sanitized_content = sanitize_mermaid_diagrams(sanitized_content)
    deduped_content = deduplicate_markdown_sections(sanitized_content)
    with open(filepath, "w", encoding="utf-8") as f:
        f.write(deduped_content)
    return deduped_content

DEFAULT_STRUCTURAL_LABELS = {
    "epic": "epic",
    "feature": "feature",
    "user_story": "user-story",
    "use_case": "use-case",
}

STRUCTURAL_LABEL_DESCRIPTION_TEMPLATE = (
    "{item_type} specification item, applied by the backlog reconciler."
)


def structural_label_key(issue_type):
    """Reduce an item type ("User Story") to its `tracker_rules.labels` key.

    The four spec loops name their type in prose; the configuration keys it in snake
    case. Deriving one from the other keeps the taxonomy in a single place — the
    configuration — instead of restating it at four call sites.
    """
    return re.sub(r"[\s\-]+", "_", str(issue_type or "").strip().lower())


def get_structural_label(issue_type, rules=None):
    """The configured tracker label for an item type, or None if it has none.

    `.pipeline/constitution.md:93-95` § *Labeling Taxonomy* fixes exactly four label
    types "or as defined by the issue tracker configuration", so the names are read
    from `tracker_rules.labels` and never hardcoded here (#313). The module-level
    default exists only so a configuration predating this key still labels correctly,
    mirroring how `get_resolved_label` defaults.
    """
    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    key = structural_label_key(issue_type)
    labels = tracker_rules.get("labels", {})
    if key in labels and labels[key]:
        return labels[key]
    provider = tracker_rules.get("provider", "github")
    if provider == "gitlab":
        return DEFAULT_GITLAB_STRUCTURAL_LABELS.get(key)
    elif provider == "jira":
        return DEFAULT_JIRA_STRUCTURAL_LABELS.get(key)
    return DEFAULT_STRUCTURAL_LABELS.get(key)


def issue_has_label(issue_record, label):
    """Does this tracker record already carry `label`?

    Tracker payloads express labels either as objects with a "name" or as bare strings,
    so both are accepted — the same shapes `is_already_resolved` handles.

    Comparison folds case and word separators through `normalize_label` (#329). It was
    exact when #313 added this function, which meant an issue already carrying
    `"User Story"` was re-labelled on every run; the module now has one comparison rule.
    """
    target = normalize_label(label)
    if not target:
        return False
    for item in (issue_record or {}).get("labels") or []:
        name = item.get("name", "") if isinstance(item, dict) else str(item)
        if normalize_label(name) == target:
            return True
    return False


def sync_issue_title_to_tracker(issue_num, filepath, rules=None, issue_record=None, provider_adapter=None):
    """Push the frontmatter title to the tracker when the two have drifted (#315).

    Tracker issues are created with a generic title derived from the schema node, while
    the spec's YAML `title` is refined afterwards; nothing ever pushed the refined value
    back, so the two diverged permanently and normalized-title matching gained
    collisions it need not have had.

    The edit is issued **only when the titles actually differ**. `AGENTS.md` § *Backlog
    Reconciliation Mandate* runs this script before every merge, so an unconditional
    edit would rewrite an unchanged title on every run and bury real changes in tracker
    noise. When no tracker record is available the title is sent, because correctness of
    the sync outranks the noise it costs.

    Returns True when an edit was issued.
    """
    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    title = extract_title(filepath)
    if not title:
        return False

    keys = tracker_rules.get("keys", {})
    current_title = (issue_record or {}).get(keys.get("title", "title"))
    if current_title is not None and str(current_title) == title:
        return False

    if provider_adapter:
        return provider_adapter.edit_issue_title(issue_num, title)

    template = tracker_rules.get("commands", {}).get("edit_issue_title")
    if not template:
        print(
            "  [Warning] No 'tracker_rules.commands.edit_issue_title' configured; the "
            f"tracker title for {format_issue_reference(issue_num, tracker_rules)} will "
            "stay out of sync with the specification frontmatter (#315).",
            file=sys.stderr,
        )
        return False

    cmd = [
        str(issue_num) if c == "{number}" else (title if c == "{title}" else c)
        for c in template
    ]
    subprocess.run(cmd, check=True, capture_output=True, timeout=30)
    return True


def apply_structural_label(issue_num, issue_type, rules=None, issue_record=None, provider_adapter=None):
    """Apply the configured structural label for this item type (#313).

    Bootstrapping reuses the `create_label` command #309 added — `--force` makes it a
    no-op where the label already exists — because a fresh downstream repository has no
    such label and applying one that does not exist fails the sync. This is the same
    just-in-time provisioning #309 established; making it install-time is issue #323 and
    is not attempted here.

    Idempotent: an issue already carrying the label produces no tracker traffic at all,
    and neither bootstrap nor application changes anything when repeated.

    Returns True when the label was applied.
    """
    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    label = get_structural_label(issue_type, rules)
    if not label:
        print(
            f"  [Warning] No structural label configured for item type '{issue_type}' "
            "in tracker_rules.labels; skipping (#313).",
            file=sys.stderr,
        )
        return False

    if issue_has_label(issue_record, label):
        return False

    description = STRUCTURAL_LABEL_DESCRIPTION_TEMPLATE.format(item_type=issue_type)

    if provider_adapter:
        provider_adapter.create_label(label, description=description, color="#0E8A16")
        success = provider_adapter.add_label(issue_num, label)
        if success:
            print(f"  [Sync Issue Body] Applied structural label '{label}'.")
        return success

    commands = tracker_rules.get("commands", {})
    create_template = commands.get("create_label")
    if create_template:
        cmd = [
            label if c == "{label}" else (description if c == "{description}" else c)
            for c in create_template
        ]
        subprocess.run(cmd, capture_output=True, timeout=30)

    add_template = commands.get("add_label") or commands.get("resolve_issue")
    if not add_template:
        print(
            "  [Warning] No 'tracker_rules.commands.add_label' configured; structural "
            f"label '{label}' cannot be applied to "
            f"{format_issue_reference(issue_num, tracker_rules)} (#313).",
            file=sys.stderr,
        )
        return False

    cmd = [
        str(issue_num) if c == "{number}" else (label if c == "{label}" else c)
        for c in add_template
    ]
    subprocess.run(cmd, check=True, capture_output=True, timeout=30)
    print(f"  [Sync Issue Body] Applied structural label '{label}'.")
    return True


def sync_issue_body_to_tracker(issue_num, filepath, issue_type="Feature", rules=None,
                               issue_record=None, provider_adapter=None):
    """Push the specification to its tracker issue: body, title (#315) and label (#313).

    `issue_record` is the tracker's own payload for this issue, when the caller has it.
    It is what makes the two additions conditional rather than unconditional — the title
    is only re-sent when it differs, and the label only applied when it is absent.
    """
    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    ref_str = format_issue_reference(issue_num, tracker_rules)
    print(f"  [Sync Issue Body] Syncing {ref_str} ({issue_type}) to tracker...")
    
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()
        
    workspace_dir = find_workspace_dir(filepath)
    content = sanitize_source_references(content, workspace_dir=workspace_dir, rules=rules)
    content = expand_relative_links_for_tracker(content, filepath=filepath, rules=rules, workspace_dir=workspace_dir)
    content = sanitize_latex_delimiters_for_tracker(content)
    content = sanitize_mermaid_diagrams(content)
    content = convert_frontmatter_to_table(content)
    content = deduplicate_markdown_sections(content)
        
    val_rules = rules.get("validation_rules", {}) if rules else {}
    max_body_chars = val_rules.get("max_body_characters", 65536)
    trunc_limit = max_body_chars - 5536
    
    if len(content) > trunc_limit:
        truncation_headers = tracker_rules.get("truncation_headers", ["## Acceptance Criteria", "## User Stories"])
        header_index = -1
        for header in truncation_headers:
            header_index = content.find(header)
            if header_index != -1:
                break
        
        project_root = workspace_dir if workspace_dir else find_workspace_dir(filepath)
        rel_path = os.path.relpath(filepath, project_root)
        
        truncation_template = tracker_rules.get("truncation_message_template", (
            "\n\n---\n*Warning: This issue body has been truncated because it exceeds the tracker size limit of {max_body_chars} characters.*\n"
            "*Please refer to the full specification file in the repository at `{rel_path}` for the complete details.*\n\n"
        )).format(max_body_chars=max_body_chars, rel_path=rel_path)
        
        if header_index != -1:
            preserved_tail = content[header_index:]
            avail_head_len = trunc_limit - len(preserved_tail) - len(truncation_template)
            if avail_head_len > 0:
                content = content[:avail_head_len] + truncation_template + preserved_tail
            else:
                content = content[:trunc_limit] + truncation_template
        else:
            content = content[:trunc_limit] + truncation_template
        
    if provider_adapter:
        provider_adapter.edit_issue(issue_num, content)
    else:
        temp_path = None
        try:
            with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".md", delete=False) as tf:
                tf.write(content)
                temp_path = tf.name
            
            edit_cmd_template = tracker_rules.get("commands", {}).get("edit_issue")
            if not edit_cmd_template:
                raise ValueError("Missing 'tracker_rules.commands.edit_issue' in codebase_rules.json")
            cmd = [str(issue_num) if c == "{number}" else (temp_path if c == "{temp_path}" else c) for c in edit_cmd_template]
            subprocess.run(cmd, check=True, capture_output=True, timeout=30)
        finally:
            if temp_path and os.path.exists(temp_path):
                os.remove(temp_path)

    # The body is only part of the update. The tracker title drifts from the frontmatter
    # (#315) and generated issues carry no structural tier (#313); both are this call
    # site sending too little.
    sync_issue_title_to_tracker(issue_num, filepath, rules=rules, issue_record=issue_record, provider_adapter=provider_adapter)
    apply_structural_label(issue_num, issue_type, rules=rules, issue_record=issue_record, provider_adapter=provider_adapter)

RESOLVED_LABEL_DESCRIPTION = (
    "Dev complete, tests pass, merged to main. Awaiting Product Owner validation."
)


def get_resolved_label(rules=None):
    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    provider = tracker_rules.get("provider", "github")
    default_resolved = "status::fixed-resolved" if provider in ("gitlab", "jira") else "status:fixed-resolved"
    return tracker_rules.get("labels", {}).get("resolved", default_resolved)


def is_already_resolved(issue_record, rules=None):
    """Has this issue already been marked Fixed / Resolved?

    This guard replaces closing (#309). The call sites were gated on the issue being
    open, so closing it was what stopped the next run from acting again. Without a
    replacement guard the reconciler would re-post the completion comment on every run,
    and AGENTS.md requires a run before every merge.

    Tracker payloads express labels either as objects with a "name" or as bare strings,
    so both are accepted. Comparison folds through `normalize_label` for the same reason
    `issue_has_label` does (#329): a case variant read as "not resolved" would re-post
    the completion comment on the next run, which is exactly what this guard prevents.
    """
    label = normalize_label(get_resolved_label(rules))
    if not label:
        return False
    for item in (issue_record or {}).get("labels") or []:
        name = item.get("name", "") if isinstance(item, dict) else str(item)
        if normalize_label(name) == label:
            return True
    return False


def resolve_issue_on_tracker(issue_num, comment, rules=None, provider_adapter=None):
    """Mark an issue Fixed / Resolved. Never closes it.

    `.pipeline/constitution.md:161` makes `Closed` unreachable without Product Owner
    validation. This function applies the resolved label and posts the evidence comment,
    leaving the issue open for that decision.
    """
    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    label = get_resolved_label(rules)
    ref_str = format_issue_reference(issue_num, tracker_rules)
    print(f"  [Resolve Issue] Marking {ref_str} Fixed / Resolved via label '{label}'...")

    if provider_adapter:
        provider_adapter.create_label(label, description=RESOLVED_LABEL_DESCRIPTION, color="#0E8A16")
        provider_adapter.add_label(issue_num, label)
        if comment:
            provider_adapter.comment_issue(issue_num, comment)
        return

    commands = tracker_rules.get("commands", {})
    create_template = commands.get("create_label")
    if create_template:
        cmd = [
            label if c == "{label}" else (RESOLVED_LABEL_DESCRIPTION if c == "{description}" else c)
            for c in create_template
        ]
        subprocess.run(cmd, capture_output=True, timeout=30)

    resolve_template = commands.get("resolve_issue")
    if not resolve_template:
        raise ValueError("Missing 'tracker_rules.commands.resolve_issue' in codebase_rules.json")
    cmd = [
        str(issue_num) if c == "{number}" else (label if c == "{label}" else c)
        for c in resolve_template
    ]
    subprocess.run(cmd, check=True, capture_output=True, timeout=30)

    comment_template = commands.get("comment_issue")
    if comment_template and comment:
        cmd = [
            str(issue_num) if c == "{number}" else (comment if c == "{comment}" else c)
            for c in comment_template
        ]
        subprocess.run(cmd, check=True, capture_output=True, timeout=30)


def is_upstream_repository(workspace_dir: Optional[str] = None) -> bool:
    """
    Check if the workspace is an Upstream Specification Core Compiler repository
    characterized by abstract compiler tooling and clean landing zones.
    """
    ws = workspace_dir or os.getcwd()
    env_type = os.environ.get("DEAP_REPOSITORY_TYPE")
    if env_type and env_type.strip() == "UPSTREAM_SPEC_CORE_COMPILER":
        return True
    upstream_marker = os.path.join(ws, ".pipeline", "upstream")
    if os.path.exists(upstream_marker):
        return True
    return False


def extract_test_targets_from_text(text: str) -> List[str]:
    """
    Extract test target specifications from markdown issue text / description / annotations.
    Supports:
    - HTML comments: <!-- test-target: path --> or <!-- test-targets: path1, path2 -->
    - Key-value lines: Test-Target: path or Test-Targets: path1, path2 (case-insensitive)
    - Key-value lines: @test-target: path or @test-targets: path1, path2
    - Markdown section: ## Test Targets with bullet list
    """
    if not text:
        return []
    targets = []

    # 1. HTML comment annotations: <!-- test-target: ... --> or <!-- test-targets: ... -->
    html_matches = re.findall(r'<!--\s*test[-_]targets?:\s*([^\n>]+?)\s*-->', text, re.IGNORECASE)
    for m in html_matches:
        parts = [p.strip().strip('`"\'') for p in re.split(r'[,;\s]+', m.strip()) if p.strip()]
        targets.extend(parts)

    # 2. Key-value line annotations: (?:^|\n)\s*(?:@)?test[-_]targets?:\s*([^\n]+)
    kv_matches = re.findall(r'(?:^|\n)\s*(?:@)?test[-_]targets?:\s*([^\n]+)', text, re.IGNORECASE)
    for m in kv_matches:
        if "-->" in m:
            continue
        parts = [p.strip().strip('`"\'') for p in re.split(r'[,;\s]+', m.strip()) if p.strip()]
        targets.extend(parts)

    # 3. Markdown section with bullets: ## Test Targets\n- target1\n- target2
    section_match = re.search(r'#+\s+Test\s+Targets?\s*\n((?:\s*[-*]\s+[^\n]+\n?)+)', text, re.IGNORECASE)
    if section_match:
        bullet_lines = section_match.group(1).splitlines()
        for line in bullet_lines:
            line_match = re.match(r'\s*[-*]\s+(.+)', line)
            if line_match:
                item = line_match.group(1).strip().strip('`"\'')
                parts = [p.strip().strip('`"\'') for p in re.split(r'[,;]+', item) if p.strip()]
                targets.extend(parts)

    # Deduplicate while preserving insertion order
    seen = set()
    result = []
    for t in targets:
        clean_t = t.strip().strip('`"\'')
        if clean_t and clean_t not in seen:
            seen.add(clean_t)
            result.append(clean_t)
    return result


def load_compiler_backlog_manifest(
    workspace_dir: Optional[str] = None,
    manifest_path: Optional[str] = None,
    rules: Optional[Dict[str, Any]] = None,
) -> Dict[str, Dict[str, Any]]:
    """
    Load mapping between compiler backlog issue numbers and their test targets.
    Normalizes to:
    {
        "<issue_id>": {
            "test_targets": ["tests/test_foo.py", ...],
            "title": "...",
            "description": "..."
        }
    }
    """
    ws = workspace_dir or os.getcwd()
    manifest_data = None

    if manifest_path and os.path.exists(manifest_path):
        try:
            with open(manifest_path, "r", encoding="utf-8") as f:
                if manifest_path.endswith((".yaml", ".yml")):
                    manifest_data = yaml.safe_load(f)
                else:
                    manifest_data = json.load(f)
        except Exception as e:
            print(f"Warning: Failed to load manifest from {manifest_path}: {e}", file=sys.stderr)

    if manifest_data is None and rules and isinstance(rules, dict):
        manifest_data = (
            rules.get("compiler_backlog_manifest")
            or rules.get("upstream_backlog_manifest")
            or rules.get("backlog_manifest")
        )

    if manifest_data is None:
        candidate_paths = [
            os.path.join(ws, ".pipeline", "compiler_backlog_manifest.json"),
            os.path.join(ws, ".pipeline", "backlog_manifest.json"),
            os.path.join(ws, ".pipeline", "upstream_backlog_manifest.json"),
            os.path.join(ws, "compiler_backlog_manifest.json"),
            os.path.join(ws, "backlog_manifest.json"),
            os.path.join(ws, ".pipeline", "compiler_backlog_manifest.yaml"),
            os.path.join(ws, ".pipeline", "compiler_backlog_manifest.yml"),
        ]
        for cp in candidate_paths:
            if os.path.exists(cp):
                try:
                    with open(cp, "r", encoding="utf-8") as f:
                        if cp.endswith((".yaml", ".yml")):
                            manifest_data = yaml.safe_load(f)
                        else:
                            manifest_data = json.load(f)
                    if manifest_data:
                        break
                except Exception as e:
                    print(f"Warning: Failed to load manifest from {cp}: {e}", file=sys.stderr)

    if not manifest_data:
        return {}

    normalized = {}
    if isinstance(manifest_data, dict):
        for raw_id, val in manifest_data.items():
            issue_key = str(raw_id).strip().lstrip("#")
            if isinstance(val, list):
                normalized[issue_key] = {"test_targets": val}
            elif isinstance(val, str):
                normalized[issue_key] = {"test_targets": [val]}
            elif isinstance(val, dict):
                entry = dict(val)
                if "test_targets" not in entry:
                    if "test_target" in entry:
                        entry["test_targets"] = [entry["test_target"]]
                    elif "tests" in entry:
                        entry["test_targets"] = entry["tests"] if isinstance(entry["tests"], list) else [entry["tests"]]
                    else:
                        entry["test_targets"] = []
                normalized[issue_key] = entry
    elif isinstance(manifest_data, list):
        for item in manifest_data:
            if not isinstance(item, dict):
                continue
            raw_id = item.get("issue_id") or item.get("issue_number") or item.get("number") or item.get("id") or item.get("issue")
            if raw_id is None:
                continue
            issue_key = str(raw_id).strip().lstrip("#")
            entry = dict(item)
            if "test_targets" not in entry:
                if "test_target" in entry:
                    entry["test_targets"] = [entry["test_target"]]
                elif "tests" in entry:
                    entry["test_targets"] = entry["tests"] if isinstance(entry["tests"], list) else [entry["tests"]]
                else:
                    entry["test_targets"] = []
            normalized[issue_key] = entry

    return normalized


def execute_test_target(
    target: str,
    workspace_dir: Optional[str] = None,
    test_executor: Optional[Any] = None,
) -> Tuple[bool, str]:
    """
    Execute a test target.
    Returns (success_bool, output_str).
    """
    ws = workspace_dir or os.getcwd()
    if test_executor is not None:
        if callable(test_executor):
            try:
                res = test_executor(target, ws)
                if isinstance(res, tuple) and len(res) == 2:
                    return bool(res[0]), str(res[1])
                elif isinstance(res, bool):
                    return res, ("OK" if res else "FAILED")
                return bool(res), str(res)
            except Exception as e:
                return False, f"Test executor exception on '{target}': {e}"

    clean_target = target.strip()
    if clean_target.startswith(("python ", "python3 ", "pytest ")):
        cmd = clean_target.split()
    else:
        cmd = [sys.executable, "-m", "unittest", clean_target]

    try:
        res = subprocess.run(
            cmd,
            cwd=ws,
            capture_output=True,
            text=True,
            timeout=180,
        )
        combined_output = (res.stdout or "") + "\n" + (res.stderr or "")
        return (res.returncode == 0, combined_output)
    except subprocess.TimeoutExpired:
        return (False, f"Test execution timed out after 180s: {' '.join(cmd)}")
    except Exception as e:
        return (False, f"Test execution failed with error: {e}")


def reconcile_upstream_compiler_backlog(
    workspace_dir: Optional[str] = None,
    rules: Optional[Dict[str, Any]] = None,
    provider_adapter: Optional[Any] = None,
    manifest: Optional[Union[Dict[str, Any], List[Dict[str, Any]]]] = None,
    manifest_path: Optional[str] = None,
    test_executor: Optional[Any] = None,
    dry_run: bool = False,
) -> Dict[str, Any]:
    """
    Reconcile compiler backlog issues in upstream repositories with clean landing zones.
    Maps tracker issues to test targets via manifest or annotations, executes test targets,
    and transitions passing issues to status:fixed-resolved.
    """
    ws = workspace_dir or os.getcwd()
    if rules is None:
        rules = load_codebase_rules(ws)

    # 1. Load manifest mappings
    manifest_map = {}
    if manifest:
        if isinstance(manifest, dict):
            for k, v in manifest.items():
                issue_key = str(k).strip().lstrip("#")
                if isinstance(v, list):
                    manifest_map[issue_key] = {"test_targets": v}
                elif isinstance(v, str):
                    manifest_map[issue_key] = {"test_targets": [v]}
                elif isinstance(v, dict):
                    entry = dict(v)
                    if "test_targets" not in entry:
                        if "test_target" in entry:
                            entry["test_targets"] = [entry["test_target"]]
                        else:
                            entry["test_targets"] = []
                    manifest_map[issue_key] = entry
        elif isinstance(manifest, list):
            for item in manifest:
                if isinstance(item, dict):
                    raw_id = item.get("issue_id") or item.get("issue_number") or item.get("number") or item.get("id") or item.get("issue")
                    if raw_id is not None:
                        issue_key = str(raw_id).strip().lstrip("#")
                        entry = dict(item)
                        if "test_targets" not in entry:
                            if "test_target" in entry:
                                entry["test_targets"] = [entry["test_target"]]
                            else:
                                entry["test_targets"] = []
                        manifest_map[issue_key] = entry
    else:
        manifest_map = load_compiler_backlog_manifest(workspace_dir=ws, manifest_path=manifest_path, rules=rules)

    # 2. Retrieve issues from tracker
    issues = []
    if provider_adapter:
        try:
            issues = provider_adapter.list_issues()
        except Exception as e:
            print(f"Warning: Failed to fetch issues from provider adapter: {e}", file=sys.stderr)
    else:
        try:
            issues = get_all_issues(rules)
        except Exception as e:
            print(f"Warning: Failed to fetch issues: {e}", file=sys.stderr)

    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    keys = tracker_rules.get("keys", {})
    id_key = keys.get("issue_id", "number")
    title_key = keys.get("title", "title")
    labels_key = keys.get("labels", "labels")
    state_key = keys.get("state", "state")

    issue_dict = {}
    for iss in issues:
        raw_id = iss.get(id_key) or iss.get("number") or iss.get("iid") or iss.get("key")
        if raw_id is not None:
            issue_dict[str(raw_id).strip().lstrip("#")] = iss
            if isinstance(raw_id, int):
                issue_dict[raw_id] = iss
            elif isinstance(raw_id, str) and raw_id.isdigit():
                issue_dict[int(raw_id)] = iss

    all_issue_ids = set(manifest_map.keys())
    for k in issue_dict.keys():
        all_issue_ids.add(str(k).strip().lstrip("#"))

    close_comments = tracker_rules.get("close_comments", {})
    comment_template = (
        close_comments.get("compiler")
        or close_comments.get("upstream_compiler")
        or "Resolved. Compiler backlog test target(s) '{test_targets}' completed and verified."
    )

    processed_count = 0
    passed_count = 0
    failed_count = 0
    skipped_count = 0
    resolved_ids = []
    results_detail = {}

    for issue_id_str in sorted(all_issue_ids, key=lambda x: int(x) if x.isdigit() else x):
        issue_record = issue_dict.get(issue_id_str)
        if issue_record is None and issue_id_str.isdigit():
            issue_record = issue_dict.get(int(issue_id_str))

        if issue_record:
            state_val = str(issue_record.get(state_key, "")).upper()
            closed_val = keys.get("closed_state_value", "CLOSED").upper()
            if state_val == closed_val:
                continue

        test_targets = []
        if issue_id_str in manifest_map:
            manifest_targets = manifest_map[issue_id_str].get("test_targets", [])
            test_targets.extend(manifest_targets)

        if issue_record:
            body_text = issue_record.get("body") or issue_record.get("description") or ""
            annotation_targets = extract_test_targets_from_text(body_text)
            for at in annotation_targets:
                if at not in test_targets:
                    test_targets.append(at)

        if not test_targets:
            continue

        processed_count += 1
        issue_num: Any = int(issue_id_str) if issue_id_str.isdigit() else issue_id_str

        if issue_record and is_already_resolved(issue_record, rules):
            print(f"  [Skipped] Issue #{issue_num} already marked {get_resolved_label(rules)}.")
            skipped_count += 1
            results_detail[issue_num] = {
                "status": "already_resolved",
                "test_targets": test_targets,
                "passed": True,
            }
            continue

        all_passed = True
        test_outputs = []
        for target in test_targets:
            success, output = execute_test_target(target, workspace_dir=ws, test_executor=test_executor)
            test_outputs.append({"target": target, "success": success, "output": output})
            if not success:
                all_passed = False
                break

        if all_passed:
            passed_count += 1
            resolved_ids.append(issue_num)
            results_detail[issue_num] = {
                "status": "resolved",
                "test_targets": test_targets,
                "passed": True,
                "outputs": test_outputs,
            }
            formatted_targets = ", ".join(test_targets)
            comment_text = comment_template.format(
                test_targets=formatted_targets,
                title=issue_record.get(title_key, f"Issue #{issue_num}") if issue_record else f"Issue #{issue_num}",
            )
            print(f"  [Pass] Issue #{issue_num} verified passing test targets: {formatted_targets}")
            if not dry_run:
                resolve_issue_on_tracker(
                    issue_num,
                    comment_text,
                    rules=rules,
                    provider_adapter=provider_adapter,
                )
                if issue_record:
                    resolved_label = get_resolved_label(rules)
                    labels_list = issue_record.setdefault(labels_key, [])
                    if isinstance(labels_list, list):
                        labels_list.append({"name": resolved_label})
        else:
            failed_count += 1
            results_detail[issue_num] = {
                "status": "failed",
                "test_targets": test_targets,
                "passed": False,
                "outputs": test_outputs,
            }
            print(f"  [Fail] Issue #{issue_num} test target failed.")

    return {
        "processed": processed_count,
        "passed": passed_count,
        "failed": failed_count,
        "skipped": skipped_count,
        "resolved": resolved_ids,
        "results": results_detail,
    }


def blocked_specs_from_linter_output(output_text, workspace_dir, rules=None):
    """Specification files the linter rejected, from its output.

    Intersected with the files that actually exist in the backlog directories. A bare
    regex over the output also catches documents merely *cited* by a finding — a
    remediation note reading "see rules/document-references.md" made the reconciler
    skip the constitution, which it had never been asked to validate. Only items the
    linter genuinely rejected belong in the skip set (#321).
    """
    mentioned = set(re.findall(r"([\w.-]+\.md)", output_text or ""))
    if not mentioned:
        return set()

    backlog = (rules or {}).get("backlog_directories", {}) or {}
    spec_names = set()
    for key in ("epics", "features", "user_stories", "use_cases"):
        rel = backlog.get(key) if isinstance(backlog, dict) else getattr(backlog, key, None)
        if not rel:
            continue
        target = os.path.join(workspace_dir, rel)
        if os.path.isdir(target):
            spec_names.update(n for n in os.listdir(target) if n.endswith(".md"))
    return mentioned & spec_names


def get_current_branch(workspace_dir):
    res = subprocess.run(["git", "branch", "--show-current"], cwd=workspace_dir, capture_output=True, text=True, timeout=30)
    if res.returncode == 0 and res.stdout.strip():
        return res.stdout.strip()
    res = subprocess.run(["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=workspace_dir, capture_output=True, text=True, timeout=30)
    if res.returncode == 0 and res.stdout.strip():
        return res.stdout.strip()
    return "master"

def extract_metadata_from_content(content: str) -> Dict[str, Any]:
    if not content:
        return {}

    # 1. Parse native Markdown tables starting with '| Attribute | Specification Detail |' or '| Metadata | Value |'
    table_header_re = re.compile(
        r'^\s*\|\s*(?:Attribute|Metadata)\s*\|\s*(?:Specification Detail|Value)\s*\|\s*$',
        re.IGNORECASE | re.MULTILINE
    )
    table_header_match = table_header_re.search(content)
    if table_header_match:
        data: Dict[str, Any] = {}
        table_start_idx = table_header_match.start()
        lines = content[table_start_idx:].splitlines()
        
        in_table = False
        for i, line in enumerate(lines):
            line_str = line.strip()
            if not line_str.startswith("|") or not line_str.endswith("|"):
                if in_table:
                    break
                continue

            if i == 0 or table_header_re.match(line_str):
                in_table = True
                continue

            # Skip table separator row (| :--- | :--- |)
            if re.match(r'^\s*\|\s*:?-+:?\s*\|\s*:?-+:?\s*\|\s*$', line_str):
                continue

            parts = [p.strip() for p in line_str.split("|")]
            if len(parts) >= 4:
                raw_key = parts[1].strip()
                raw_val = parts[2].strip()

                # Clean markdown formatting from key
                clean_key = re.sub(r'[*`_]', '', raw_key).strip()
                # Normalize key to snake_case
                norm_key = re.sub(r'[\s\-]+', '_', clean_key.lower())
                norm_key = re.sub(r'[^a-z0-9_]', '', norm_key).strip('_')

                if not norm_key:
                    continue

                # Specific key normalizations
                if norm_key in ("issue_id", "issueid", "id"):
                    norm_key = "issue_id"
                elif norm_key in ("parent_epic", "parentepic", "parent_epic_id", "epic"):
                    norm_key = "epic"
                elif norm_key in ("specification_source", "spec_source", "specsource"):
                    norm_key = "spec_source"
                elif norm_key in ("sysml_test_case", "test_case", "testcase"):
                    norm_key = "sysml_test_case"
                elif norm_key in ("sysml_interaction", "interaction"):
                    norm_key = "sysml_interaction"
                elif norm_key in ("schema_containers", "schemacontainers", "schema_container"):
                    norm_key = "schema_containers"
                elif norm_key in ("interface_type", "interfacetype"):
                    norm_key = "interface_type"
                elif norm_key in ("generation_mode", "generationmode"):
                    norm_key = "generation_mode"

                val: Any = raw_val
                # Convert issue_id to int (stripping # or quotes)
                if norm_key == "issue_id":
                    clean_id = re.sub(r'["\'#]', '', str(val)).strip()
                    if clean_id.isdigit():
                        val = int(clean_id)
                    else:
                        val = clean_id
                else:
                    val = val.replace(r'\|', '|')
                    if norm_key in ("labels", "tags", "realizes"):
                        if val.startswith("[") and val.endswith("]"):
                            val = [item.strip().strip('"\'`') for item in val[1:-1].split(",") if item.strip()]
                        elif "," in val:
                            val = [item.strip().strip('"\'`') for item in val.split(",") if item.strip()]
                        elif val:
                            val = [val.strip().strip('"\'`')]
                        else:
                            val = []
                    elif norm_key == "schema_containers":
                        clean_str = val.strip().strip('"\'`')
                        if clean_str.startswith("[") and clean_str.endswith("]"):
                            val = [item.strip().strip('"\'`') for item in clean_str[1:-1].split(",") if item.strip()]
                        elif "," in clean_str:
                            val = [item.strip().strip('"\'`') for item in clean_str.split(",") if item.strip()]
                        elif clean_str:
                            val = [clean_str]
                        else:
                            val = []
                    elif norm_key == "interface_type":
                        clean_str = val.strip().strip('"\'`')
                        if clean_str.startswith("[") and clean_str.endswith("]"):
                            val = [item.strip().strip('"\'`') for item in clean_str[1:-1].split(",") if item.strip()]
                        elif "," in clean_str:
                            val = [item.strip().strip('"\'`') for item in clean_str.split(",") if item.strip()]

                data[norm_key] = val
                if norm_key == "epic":
                    data["parent_epic"] = val
                elif norm_key == "spec_source":
                    data["specification_source"] = val
                elif norm_key == "sysml_test_case":
                    data["test_case"] = val
                    data["test_cases"] = [val] if isinstance(val, str) else val

        if data:
            return data

    # 2. Retain legacy YAML frontmatter fallback
    if content.startswith("---"):
        parts = content.split("---", 2)
        if len(parts) >= 3:
            frontmatter_text = parts[1]
            try:
                data = yaml.safe_load(frontmatter_text.replace('\x01', ''))
                if isinstance(data, dict):
                    if "issue_id" in data:
                        raw_id = data["issue_id"]
                        clean_id = re.sub(r'["\'#]', '', str(raw_id)).strip()
                        if clean_id.isdigit():
                            data["issue_id"] = int(clean_id)
                    return data
            except Exception as e:
                print(f"Error parsing legacy YAML frontmatter: {e}")

    return {}


def extract_metadata(filepath):
    try:
        with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read()
        return extract_metadata_from_content(content)
    except Exception as e:
        print(f"Error parsing metadata from {filepath}: {e}")
    return {}

def lookup_canonical_issue_key(raw_id, issue_dict):
    """Return the key under which `raw_id` sits in `issue_dict`, or None if absent.

    `issue_dict` is keyed twice per issue — once int, once str — because tracker
    payloads and frontmatter disagree about the type. Frontmatter may also quote the
    value or write it as a reference (`"901"`, `#901`), so all three spellings are
    reduced to the one key the caller can index with.
    """
    if raw_id is None or isinstance(raw_id, bool):
        return None
    if isinstance(raw_id, int):
        candidates = [raw_id, str(raw_id)]
    else:
        text = str(raw_id).strip().strip('"\'').lstrip("#").strip()
        if not text:
            return None
        candidates = [text, int(text)] if text.isdigit() else [text]
    for candidate in candidates:
        if candidate in issue_dict:
            return candidate
    return None


def resolve_spec_issue_number(filepath, title, title_map, issue_dict, rules=None,
                              item_type="Feature", claimed=None):
    """Resolve a local spec file to its tracker issue. Canonical `issue_id` first.

    `.pipeline/constitution.md:57-59` § *Unique Backlog Identifiers* mandates an
    `issue_id: <int>` in every spec's frontmatter and states that "Matching by title
    normalization is prohibited as a primary selector." This function is where that
    precedence is enforced (#314), and it is the only resolution path the four spec
    loops in main() use (#316).

    Order:

    1. Frontmatter `issue_id` present and on the tracker — used, full stop.
    2. Frontmatter `issue_id` present but absent from the tracker — **hard error**. A
       fall-through to title matching here is exactly #316: the title can match some
       unrelated issue, and `sync_issue_body_to_tracker` would then overwrite that
       issue's body. It is also the same class of defect as the referenced-but-missing
       issue the module already refuses to invent.
    3. No `issue_id` yet (first registration) — title normalization, with a warning
       naming the file, because the constitution allows it only as a fallback.

    `claimed` is an optional dict shared across all four loops. Two spec files
    resolving to one issue number means one of them is about to be overwritten, so it
    fails loudly with both paths rather than syncing.
    """
    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    meta = extract_metadata(filepath)
    fm_id = meta.get("issue_id")
    basename = os.path.basename(filepath)

    declared = str(fm_id).strip().strip('"\'').lstrip("#").strip() if fm_id is not None else ""

    issue_num = None
    if declared:
        issue_num = lookup_canonical_issue_key(fm_id, issue_dict)
        if issue_num is None:
            declared_ref = format_issue_reference(declared, tracker_rules)
            print(
                f"[FATAL] {item_type} '{basename}' declares issue_id {declared_ref}, "
                "which does not exist on the tracker. Refusing to fall back to title "
                "matching: that is how an unrelated issue's body gets overwritten "
                f"(#316). Correct or remove the issue_id in {filepath}",
                file=sys.stderr,
            )
            sys.exit(1)
    else:
        issue_num = title_map.get(normalize_title(title, rules))
        if issue_num is not None:
            print(
                f"  [Warning] {item_type} '{basename}' has no issue_id in its "
                f"frontmatter; fell back to matching by normalized title and resolved "
                f"{format_issue_reference(issue_num, tracker_rules)}. "
                ".pipeline/constitution.md:58-59 prohibits title normalization as a "
                f"primary selector — add 'issue_id: {issue_num}' to {filepath}"
            )

    if issue_num is None:
        return None

    if claimed is not None:
        key = str(issue_num)
        previous = claimed.get(key)
        if previous is not None and os.path.abspath(previous) != os.path.abspath(filepath):
            print(
                "[FATAL] Two specification files resolve to the same issue "
                f"{format_issue_reference(issue_num, tracker_rules)}: "
                f"{previous} and {filepath}. Syncing both would overwrite one body with "
                "the other (#316). Give each file its own issue_id.",
                file=sys.stderr,
            )
            sys.exit(1)
        claimed[key] = filepath

    return issue_num


def build_epic_alias_map(epics_dir, rules=None):
    """Every spelling an Epic can be referenced by -> that Epic's canonical normalized title.

    The map resolves *cross-references between items* — the `epic:` frontmatter key and
    the parent-epic link in a body — so that a child ends up in the right Epic's
    checklist. It has never resolved a file's own identity, and after #314/#316 it must
    not: `resolve_spec_issue_number` is the sole authority there, and an alias that
    claimed a Feature's slug would assert an identity the resolver never granted.

    Aliases are deliberately generous, including type-erased ones: an Epic titled
    "Epic 07: Geo Location" is reachable as `geo location`, because children routinely
    name their parent by bare title. #319 is what that generosity cost — `feat-07-geo-
    location` normalizes to `geo location` too, so a Feature-typed reference resolved to
    the Epic. The gate for that is at *lookup* time in `resolve_epic_reference`, on the
    type the reference declares about itself, rather than by deleting the alias: deleting
    it would break every legitimate reference-by-title, which is the common case.

    What is enforced here is the other half of the collision. An alias claimed by two
    Epics with different canonical titles is dropped rather than kept, because keeping it
    resolves by `os.listdir` order — a filesystem accident, not a resolution rule.
    """
    alias_map = {}
    ambiguous = set()
    if not epics_dir or not os.path.exists(epics_dir):
        return alias_map

    for fn in sorted(os.listdir(epics_dir)):
        if not fn.endswith(".md"):
            continue
        fp = os.path.join(epics_dir, fn)
        title = extract_title(fp)
        meta = extract_metadata(fp)
        canonical_norm = normalize_title(title, rules) if title else ""
        if not canonical_norm:
            canonical_norm = fn[:-3].lower()

        aliases = set()
        if title:
            aliases.add(title.strip().strip('"\'').lower())
            aliases.add(normalize_title(title, rules))
            aliases.add(re.sub(r'^\w+[- ]*\d+\s*[:\-]?\s*', '', title, flags=re.IGNORECASE).strip().lower())

        fn_slug = fn[:-3]
        aliases.add(fn_slug.lower())
        aliases.add(fn_slug.lower().replace("-", " "))
        aliases.add(normalize_title(fn_slug, rules))

        fm_id = meta.get("id") or meta.get("epic")
        if fm_id:
            fm_id_str = str(fm_id).strip().strip('"\'')
            aliases.add(fm_id_str.lower())
            aliases.add(fm_id_str.lower().replace("-", " "))
            aliases.add(normalize_title(fm_id_str, rules))

        for sample in [title, fn_slug, str(fm_id) if fm_id else ""]:
            if sample:
                m = re.search(r'\b(epic[- ]*\d+)\b', sample, re.IGNORECASE)
                if m:
                    id_prefix = m.group(1).lower()
                    aliases.add(id_prefix)
                    aliases.add(id_prefix.replace("-", " "))
                    aliases.add(id_prefix.replace(" ", "-"))

        for alias in sorted(a for a in aliases if a):
            if alias in ambiguous:
                continue
            existing = alias_map.get(alias)
            if existing is not None and existing != canonical_norm:
                print(
                    f"  [Warning] Epic alias '{alias}' is claimed by both "
                    f"'{existing}' and '{canonical_norm}'; dropping it rather than "
                    "resolving by directory order (#319). Reference the intended Epic "
                    "by its filename slug or issue number."
                )
                del alias_map[alias]
                ambiguous.add(alias)
                continue
            alias_map[alias] = canonical_norm

    return alias_map


def resolve_epic_reference(epic_ref, epic_alias_map, epic_id_to_norm, rules=None):
    """Resolve a parent-epic reference to the referenced Epic's normalized title.

    Order: issue number first, then the alias map, then bare normalization.

    The namespace gate for #319 sits between those last two. A reference that names its
    own type — `feat-07-geo-location`, `us-03-operator`, `uc-04-device-state` — is not an
    Epic reference, so the alias map is not consulted for it and no epic is returned.
    Falling through to `normalize_title` instead would not be enough: the whole point of
    the collision is that a Feature and an Epic sharing a suffix normalize to the same
    string, so the bare normalization matches the Epic's canonical title just as the
    alias did.

    Returning None is reported rather than silent. A parent link that quietly fails to
    resolve is indistinguishable from a specification that declares no parent at all,
    and the reference itself is the thing that needs correcting.
    """
    if not epic_ref:
        return None

    if isinstance(epic_ref, int):
        if epic_ref in epic_id_to_norm:
            return epic_id_to_norm[epic_ref]
        ref_str = str(epic_ref)
    else:
        ref_str = str(epic_ref).strip().strip('"\'')

    clean_ref = ref_str
    if clean_ref.startswith('#'):
        clean_ref = clean_ref[1:].strip()

    if clean_ref in epic_id_to_norm:
        return epic_id_to_norm[clean_ref]
    if clean_ref.isdigit() and int(clean_ref) in epic_id_to_norm:
        return epic_id_to_norm[int(clean_ref)]

    declared_type = spec_type_of_reference(ref_str)
    if declared_type is not None and declared_type != "epic":
        print(
            f"  [Warning] Parent-epic reference '{ref_str}' names a {declared_type}, "
            "not an Epic; refusing to resolve it through the Epic alias map (#319). A "
            "type-erased alias would otherwise attach this item to an unrelated Epic "
            "sharing the same title suffix."
        )
        return None

    if ref_str.lower() in epic_alias_map:
        return epic_alias_map[ref_str.lower()]
    norm = normalize_title(ref_str, rules)
    if norm in epic_alias_map:
        return epic_alias_map[norm]
    ref_space = ref_str.lower().replace("-", " ")
    if ref_space in epic_alias_map:
        return epic_alias_map[ref_space]
    return norm


def resolve_type_context(line, filepath, section_context):
    # 1. URL path check
    link_match = re.search(r'\[([^\]]+)\]\(([^)]+)\)', line)
    if link_match:
        path = link_match.group(2)
        if "docs/features" in path or "/features/" in path:
            return "feature"
        elif "docs/user-stories" in path or "/user-stories/" in path:
            return "user-story"
        elif "docs/use-cases" in path or "/use-cases/" in path:
            return "use-case"
        elif "docs/epics" in path or "/epics/" in path:
            return "epic"
            
    # 2. Section context check
    if section_context:
        return section_context
        
    # 3. Line prefix/keywords check
    line_lower = line.lower()
    if "use case" in line_lower or "use-case" in line_lower or "uc-" in line_lower:
        return "use-case"
    if "user story" in line_lower or "user-story" in line_lower or "us-" in line_lower:
        return "user-story"
    if "feature" in line_lower or "feat-" in line_lower:
        return "feature"
    if "epic" in line_lower:
        return "epic"
        
    # 4. File folder context check (default fallback)
    parent_dir = os.path.basename(os.path.dirname(filepath))
    if "features" in parent_dir:
        return "feature"
    elif "user-stories" in parent_dir:
        return "user-story"
    elif "use-cases" in parent_dir:
        return "use-case"
    elif "epics" in parent_dir:
        return "epic"
        
    return None

def resolve_issue_ids_in_file(filepath, epic_titles, feature_titles, story_titles, usecase_titles, rules=None):
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()
        
    tracker_rules = rules.get("tracker_rules", {}) if rules else {}
    placeholder = tracker_rules.get("issue_id_placeholder", "#[IssueID]")
    title_extraction_prefixes_regex = tracker_rules.get("title_extraction_prefixes_regex", r"(?:Feature\s+\d+\s*:\s*|Use\s+Case\s+\d+\s*:\s*|User\s+Story\s+\d+\s*:\s*)?")
    
    if placeholder not in content and "#[EpicIssueID]" not in content:
        return content
        
    lines = content.splitlines()
    updated = False
    
    section_context = None
    for i, line in enumerate(lines):
        # Track section context based on headers
        header_match = re.match(r'^(#+)\s+(.*)$', line)
        if header_match:
            header_text = header_match.group(2).lower()
            if "use case" in header_text:
                section_context = "use-case"
            elif "user story" in header_text or "user-story" in header_text:
                section_context = "user-story"
            elif "feature" in header_text or "requirement" in header_text:
                section_context = "feature"
            elif "epic" in header_text:
                section_context = "epic"
                
        if placeholder not in line and "#[EpicIssueID]" not in line:
            continue
            
        active_placeholder = placeholder if placeholder in line else "#[EpicIssueID]"
        escaped_active = re.escape(active_placeholder)
        
        title = None
        link_label_match = re.search(r'\[([^\]]+)\]\(', line)
        if link_label_match:
            title = link_label_match.group(1).strip()
        else:
            pattern = escaped_active + r'(?:\s*[-:]\s*)?' + title_extraction_prefixes_regex + r'(.*)$'
            dash_match = re.search(pattern, line)
            if dash_match:
                title = dash_match.group(1).strip()
                title = re.sub(r'\(.*?\)', '', title).strip()
                title = title.strip('[]-* ')
                
        if (not title or not title.strip()) and re.search(r'issue[\s\-_]*id\s*:', line, re.IGNORECASE):
            title = extract_title(filepath)

        if title:
            norm = normalize_title(title, rules)
            type_context = resolve_type_context(line, filepath, section_context)
            if active_placeholder == "#[EpicIssueID]":
                type_context = "epic"
            issue_num = None
            if type_context == "epic":
                issue_num = epic_titles.get(norm)
            elif type_context == "feature":
                issue_num = feature_titles.get(norm)
            elif type_context == "user-story":
                issue_num = story_titles.get(norm)
            elif type_context == "use-case":
                issue_num = usecase_titles.get(norm)
                
            if not issue_num:
                issue_num = (feature_titles.get(norm) or 
                             story_titles.get(norm) or 
                             usecase_titles.get(norm) or 
                             epic_titles.get(norm))
                             
            if issue_num:
                ref_str = format_issue_reference(issue_num, tracker_rules)
                lines[i] = line.replace(active_placeholder, ref_str)
                updated = True
                print(f"  [Resolve ID] Resolved {active_placeholder} to {ref_str} for '{title}' (type: {type_context}) in {os.path.basename(filepath)}")
            else:
                print(f"  [Warning] Could not resolve {active_placeholder} for title '{title}' in {os.path.basename(filepath)}")
                
    if updated:
        new_content = "\n".join(lines) + "\n"
        return write_markdown_file(filepath, new_content)
        
    return content

def reconcile_epic_checklists(filepath, child_features, child_stories, child_usecases, epic_titles, feature_titles, story_titles, usecase_titles, rules):
    with open(filepath, "r", encoding="utf-8") as f:
        content = f.read()

    lines = content.splitlines()
    
    idx_req = -1
    idx_usecases = -1
    idx_stories = -1
    idx_next = -1
    
    for idx, line in enumerate(lines):
        line_clean = line.strip()
        if line_clean.startswith("## 2. Requirements & Checklist") or (line_clean.startswith("## 2.") and "Checklist" in line_clean):
            idx_req = idx
        elif re.match(r'^#{3,4}\s+Associated\s+Use\s+Cases(?!\s*(?:&|and)\s*User\s+Stories)', line_clean, re.IGNORECASE):
            idx_usecases = idx
        elif re.match(r'^#{3,4}\s+Associated\s+User\s+Stories', line_clean, re.IGNORECASE):
            idx_stories = idx
        elif idx_req != -1 and line_clean.startswith("## ") and idx > idx_req and not line_clean.startswith("## 2."):
            if idx_next == -1:
                idx_next = idx

    if idx_usecases == -1:
        for idx, line in enumerate(lines):
            if re.match(r'^#{3,4}\s+Associated\s+Use\s+Cases', line.strip(), re.IGNORECASE):
                idx_usecases = idx
                break

    def extract_items_from_range(start_idx, end_idx):
        items = []
        if start_idx == -1:
            return items
        limit = end_idx if end_idx != -1 else len(lines)
        for i in range(start_idx + 1, limit):
            l = lines[i].strip()
            if l.startswith("## "):
                break
            if l.startswith("### ") or l.startswith("#### "):
                continue
            if l.startswith("- [ ]") or l.startswith("- [x]") or l.startswith("- [X]"):
                l_lower = l.lower()
                ignore_exact = {
                    "feat-xx-name",
                    "uc-xx-name",
                    "us-xx-name",
                    "feature title",
                    "use case title",
                    "user story title",
                    "epic title",
                }
                prefix_patterns = {"feature 1:", "use case 1:", "user story 1:"}
                title_part = re.sub(r'^-\s*\[[ xX]\]\s*', '', l_lower)
                if any(p == title_part for p in ignore_exact):
                    continue
                if any(title_part.startswith(p) for p in prefix_patterns):
                    continue
                items.append(lines[i])
        return items

    PLACEHOLDER_PATTERNS = [
        re.compile(r'^\s*[-*]*\s*\(?\s*\*?To be populated.*?\*?\)?\s*$', re.IGNORECASE),
        re.compile(r'^\s*[-*]*\s*\*?TBD\*?\s*$', re.IGNORECASE),
        re.compile(r'^\s*[-*]*\s*\*?N/A\*?\s*$', re.IGNORECASE),
    ]

    def filter_content_lines(slice_lines):
        filtered = []
        for l in slice_lines:
            stripped = l.strip()
            if stripped and not any(p.match(stripped) for p in PLACEHOLDER_PATTERNS):
                filtered.append(l)
            elif not stripped:
                filtered.append(l)
        return filtered

    end_req = idx_usecases if idx_usecases != -1 else (idx_stories if idx_stories != -1 else idx_next)
    end_usecases = idx_stories if idx_stories != -1 else idx_next
    end_stories = idx_next
    
    existing_features = extract_items_from_range(idx_req, end_req)
    existing_usecases = extract_items_from_range(idx_usecases, end_usecases)
    existing_stories = extract_items_from_range(idx_stories, end_stories)
    
    indent = ""
    for item in existing_features + existing_usecases + existing_stories:
        m = re.match(r'^(\s*)', item)
        if m and m.group(1):
            indent = m.group(1)
            break

    workspace_root = find_workspace_dir(filepath)
    branch_name = get_current_branch(workspace_root)
    if not branch_name or branch_name == "HEAD":
        branch_name = "main"
    blob_base = get_blob_url_base(rules=rules, workspace_dir=workspace_root, branch=branch_name)
    
    def format_item(item_type, filename, title, issue_num):
        tracker_rules = rules.get("tracker_rules", {}) if rules else {}
        ref_str = format_issue_reference(issue_num, tracker_rules) if (issue_num and issue_num != 0) else tracker_rules.get("issue_id_placeholder", "#[IssueID]")
        
        if item_type == "feature":
            path_part = f"docs/features/{filename}.md"
        elif item_type == "use-case":
            path_part = f"docs/use-cases/{filename}.md"
        else:
            path_part = f"docs/user-stories/{filename}.md"
            
        return f"{indent}- [ ] {ref_str} - [{title}]({blob_base}/{path_part}) (semantic linkage justification)"

    def get_filename_key(item_str):
        m = re.search(r'(?:docs/)?(features|use-cases|user-stories)/([a-zA-Z0-9_\-]+)\.md', item_str)
        if m:
            return m.group(2)
        return None

    def sanitize_existing_item(item, title_map, child_list):
        tracker_rules = rules.get("tracker_rules", {}) if rules else {}
        placeholder = tracker_rules.get("issue_id_placeholder", "#[IssueID]")
        if "#0" in item or "#[" in item:
            title = None
            m_title = re.search(r'\[([^\]]+)\]\(', item)
            if m_title:
                title = m_title.group(1)
            else:
                key = get_filename_key(item)
                if key and child_list:
                    for fn, t in child_list:
                        if fn == key:
                            title = t
                            break
            issue_num = None
            if title:
                issue_num = title_map.get(normalize_title(title, rules))
            if issue_num and issue_num != 0:
                ref_str = format_issue_reference(issue_num, tracker_rules)
                item = re.sub(r'#0\b|#\[(?:IssueID|FeatureIssueID|UseCaseIssueID|StoryIssueID)\]', ref_str, item)
            else:
                item = re.sub(r'#0\b', placeholder, item)
        return item

    final_features = []
    seen_feats = set()
    for item in existing_features:
        key = get_filename_key(item)
        sanitized = sanitize_existing_item(item, feature_titles, child_features)
        if key:
            seen_feats.add(key)
        final_features.append(sanitized)
            
    for fn, title in child_features:
        if fn not in seen_feats:
            issue_num = feature_titles.get(normalize_title(title, rules))
            final_features.append(format_item("feature", fn, title, issue_num))
            seen_feats.add(fn)
            
    final_usecases = []
    seen_ucs = set()
    for item in existing_usecases:
        key = get_filename_key(item)
        sanitized = sanitize_existing_item(item, usecase_titles, child_usecases)
        if key:
            seen_ucs.add(key)
        final_usecases.append(sanitized)
            
    for fn, title in child_usecases:
        if fn not in seen_ucs:
            issue_num = usecase_titles.get(normalize_title(title, rules))
            final_usecases.append(format_item("use-case", fn, title, issue_num))
            seen_ucs.add(fn)
            
    final_stories = []
    seen_stories = set()
    for item in existing_stories:
        key = get_filename_key(item)
        sanitized = sanitize_existing_item(item, story_titles, child_stories)
        if key:
            seen_stories.add(key)
        final_stories.append(sanitized)
            
    for fn, title in child_stories:
        if fn not in seen_stories:
            issue_num = story_titles.get(normalize_title(title, rules))
            final_stories.append(format_item("user-story", fn, title, issue_num))
            seen_stories.add(fn)

    def is_item_or_placeholder(line):
        l = line.strip()
        if not l:
            return False
        if l.startswith("- [ ]") or l.startswith("- [x]") or l.startswith("- [X]"):
            return True
        if any(p.match(l) for p in PLACEHOLDER_PATTERNS):
            return True
        return False

    def filter_non_item_lines(slice_lines):
        filtered = []
        prev_blank = False
        for l in slice_lines:
            if is_item_or_placeholder(l):
                continue
            stripped = l.strip()
            if not stripped:
                if not prev_blank:
                    filtered.append(l)
                    prev_blank = True
            else:
                filtered.append(l)
                prev_blank = False
        return filtered

    new_lines = []
    if idx_req != -1:
        new_lines.extend(lines[:idx_req + 1])
        if not final_features:
            new_lines.append(f"{indent}*To be populated after Phase 3*")
        else:
            new_lines.extend(final_features)
        
        if idx_usecases != -1:
            end_req = idx_usecases
            new_lines.extend(filter_non_item_lines(lines[idx_req + 1 : end_req]))
            new_lines.append(lines[idx_usecases])
        else:
            end_req = idx_stories if idx_stories != -1 else (idx_next if idx_next != -1 else len(lines))
            new_lines.extend(filter_non_item_lines(lines[idx_req + 1 : end_req]))
            new_lines.append("")
            new_lines.append(f"{indent}### Associated Use Cases & User Stories")
            new_lines.append("")
            new_lines.append(f"{indent}#### Associated Use Cases")
            
        if not final_usecases:
            new_lines.append(f"{indent}*To be populated after Phase 3*")
        else:
            new_lines.extend(final_usecases)
        
        if idx_stories != -1:
            end_usecases = idx_stories
            start_usecases = idx_usecases if idx_usecases != -1 else idx_req
            new_lines.extend(filter_non_item_lines(lines[start_usecases + 1 : end_usecases]))
            new_lines.append(lines[idx_stories])
        else:
            end_usecases = idx_next if idx_next != -1 else len(lines)
            start_usecases = idx_usecases if idx_usecases != -1 else idx_req
            new_lines.extend(filter_non_item_lines(lines[start_usecases + 1 : end_usecases]))
            new_lines.append("")
            new_lines.append(f"{indent}#### Associated User Stories")
            
        if not final_stories:
            new_lines.append(f"{indent}*To be populated after Phase 3*")
        else:
            new_lines.extend(final_stories)
        
        start_after_stories = idx_stories if idx_stories != -1 else (idx_usecases if idx_usecases != -1 else idx_req)
        if idx_next != -1:
            new_lines.extend(filter_non_item_lines(lines[start_after_stories + 1 : idx_next]))
            new_lines.extend(lines[idx_next:])
        else:
            new_lines.extend(filter_non_item_lines(lines[start_after_stories + 1 :]))
    else:
        return

    new_content = "\n".join(new_lines) + "\n"
    if new_content != content:
        write_markdown_file(filepath, new_content)
        print(f"  [Reconcile Checklist] Updated checklists in {os.path.basename(filepath)}")

def find_workspace_dir(start_path):
    curr = os.path.abspath(start_path)
    if os.path.isfile(curr):
        curr = os.path.dirname(curr)
    while True:
        if os.path.exists(os.path.join(curr, ".pipeline", "logical-ui", "codebase_rules.json")):
            return curr
        if os.path.exists(os.path.join(curr, ".pipeline", "codebase_rules.json")):
            return curr
        if os.path.exists(os.path.join(curr, "codebase_rules.json")):
            return curr
        if os.path.exists(os.path.join(curr, ".git")):
            return curr
        parent = os.path.dirname(curr)
        if parent == curr:
            break
        curr = parent
    return os.path.dirname(os.path.abspath(start_path)) if os.path.isfile(start_path) else os.path.abspath(start_path)

def assert_no_mock_cli(workspace_dir=None):
    if not workspace_dir:
        workspace_dir = find_workspace_dir(os.getcwd())
    workspace_dir = os.path.abspath(workspace_dir)
    scratch_dir = os.path.abspath(os.path.join(workspace_dir, "scratch"))
    scratch_bin = os.path.join(scratch_dir, "bin")
    forbidden_cmds = ["gh", "glab", "git", "flutter"]

    for cmd in forbidden_cmds:
        binary_path = os.path.join(scratch_bin, cmd)
        if os.path.exists(binary_path):
            print(f"[FATAL] Zero-mocking policy violation: Forbidden mock CLI binary detected at {binary_path}", file=sys.stderr)
            sys.exit(1)

        resolved = shutil.which(cmd)
        if resolved:
            resolved_abs = os.path.abspath(resolved)
            if resolved_abs.startswith(scratch_dir + os.sep) or resolved_abs == scratch_dir:
                print(f"[FATAL] Zero-mocking policy violation: Forbidden mock CLI binary detected at {resolved_abs}", file=sys.stderr)
                sys.exit(1)

def main():
    parser = argparse.ArgumentParser(
        description="Backlog reconciliation script that synchronises local markdown spec files with an external issue tracker (e.g. GitHub Issues, GitLab Issues)."
    )
    parser.add_argument(
        "docs_dir",
        nargs="?",
        default=None,
        help="Optional path to the documentation directory containing epics, features, user_stories, and use_cases. Defaults to workspace root.",
    )
    parser.add_argument(
        "--provider",
        choices=["github", "gitlab", "jira", "auto"],
        default=None,
        help="Issue tracker provider ('github', 'gitlab', or 'jira'). Defaults to auto-detection or codebase rules configuration.",
    )
    parser.add_argument(
        "--gitlab-url",
        default=None,
        help="GitLab instance URL (default: https://gitlab.com or GITLAB_URL / CI_SERVER_URL environment variables).",
    )
    parser.add_argument(
        "--project",
        default=None,
        help="GitLab project path or numeric ID (e.g. 'gintatkinson/DEAP01-spec-core' or CI_PROJECT_PATH).",
    )
    parser.add_argument(
        "--jira-url",
        default=None,
        help="Jira instance base URL (e.g. 'https://your-domain.atlassian.net' or JIRA_SERVER_URL environment variable).",
    )
    parser.add_argument(
        "--jira-project",
        default=None,
        help="Jira project key code (e.g. 'UAS' or JIRA_PROJECT_KEY environment variable).",
    )
    parser.add_argument(
        "--jira-email",
        default=None,
        help="Jira account email address (for Jira Cloud Basic Auth or JIRA_EMAIL environment variable).",
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        help="Run in offline mode without contacting external tracker.",
    )
    parser.add_argument(
        "--manifest",
        "--compiler-manifest",
        dest="manifest",
        default=None,
        help="Path to compiler backlog manifest JSON/YAML mapping issues to test targets.",
    )
    parser.add_argument(
        "--upstream",
        action="store_true",
        help="Force upstream compiler backlog reconciliation mode.",
    )
    args = parser.parse_args()

    sanitize_github_token_env()
    script_dir = os.path.dirname(os.path.abspath(__file__))
    workspace_dir = find_workspace_dir(script_dir)
    assert_no_mock_cli(workspace_dir)

    # Programmatic gate: Run linter before proceeding with reconciliation
    blocked_specs = set()
    rules_preview = load_codebase_rules(workspace_dir)
    linter_script = resolve_linter_script(workspace_dir)
    if linter_script and os.path.exists(linter_script):
        print("Running pre-reconciliation linter validation...")
        cmd = [sys.executable, linter_script, "--spec-only", "--allow-missing-specs"]
        try:
            res = subprocess.run(cmd, cwd=workspace_dir, capture_output=True, text=True, timeout=30)
            if res.returncode != 0:
                output_text = (res.stdout or "") + "\n" + (res.stderr or "")
                lines = [line.strip() for line in output_text.splitlines()]
                error_lines = [line for line in lines if line.startswith("- ")]
                
                is_exclusive_checklist_placeholder = False
                if error_lines:
                    is_exclusive_checklist_placeholder = True
                    for err in error_lines:
                        err_lower = err.lower()
                        if "placeholder" not in err_lower and "checklist" not in err_lower and "required features matrix" not in err_lower:
                            is_exclusive_checklist_placeholder = False
                            break
                
                if is_exclusive_checklist_placeholder:
                    print("[Warning] Pre-reconciliation linter validation found only checklist warning issues/placeholders. Proceeding with warnings.", file=sys.stderr)
                    for err in error_lines:
                        print(f"  [Warning Detail] {err}", file=sys.stderr)
                else:
                    # Issue #321 - a failing linter used to abort the entire run, so one
                    # incomplete work-in-progress draft withheld synchronisation from every
                    # finished, unrelated specification. The gate is not weakened: the
                    # offending items are skipped and the run still exits non-zero at the
                    # end. What changes is that valid work is no longer held hostage.
                    blocked_specs = blocked_specs_from_linter_output(
                        output_text, workspace_dir, rules_preview
                    )
                    print("[BLOCKED] Pre-reconciliation linter validation failed for "
                          f"{len(blocked_specs)} specification(s). These will be SKIPPED; "
                          "everything else still synchronises, and this run will exit "
                          "non-zero.", file=sys.stderr)
                    for name in sorted(blocked_specs):
                        print(f"  [Blocked] {name}", file=sys.stderr)
                    print(res.stdout, file=sys.stderr)
            else:
                print("Pre-reconciliation linter validation passed successfully.")
        except subprocess.TimeoutExpired:
            print("[FATAL] Pre-reconciliation linter validation timed out after 30 seconds. Aborting.", file=sys.stderr)
            sys.exit(1)
    else:
        print("[INFO] Pre-reconciliation linter not found; skipping pre-validation.")

    try:
        provider_name = detect_tracker_provider(cli_provider=args.provider, rules=rules_preview, workspace_dir=workspace_dir)
        rules = load_codebase_rules(workspace_dir, provider=provider_name)

        provider_adapter = create_tracker_provider(
            provider_name=provider_name,
            rules=rules,
            workspace_dir=workspace_dir,
            offline=args.offline,
            cli_gitlab_url=args.gitlab_url,
            cli_project=args.project,
            cli_jira_url=args.jira_url,
            cli_jira_project=args.jira_project,
            cli_jira_email=args.jira_email,
        )

        try:
            issues = get_all_issues(rules, provider_adapter=provider_adapter)
        except Exception as e:
            print(f"Error fetching issues: {e}")
            print("Please ensure issue tracker CLI / API is authenticated and configured.")
            sys.exit(1)

        if is_upstream_repository(workspace_dir) or getattr(args, "upstream", False) or getattr(args, "manifest", None):
            print("Reconciling upstream compiler backlog issues against test targets...")
            upstream_res = reconcile_upstream_compiler_backlog(
                workspace_dir=workspace_dir,
                rules=rules,
                provider_adapter=provider_adapter,
                manifest_path=getattr(args, "manifest", None),
            )
            print(
                f"Upstream compiler backlog reconciliation: {upstream_res['passed']} passed, "
                f"{upstream_res['failed']} failed, {upstream_res['skipped']} skipped, "
                f"{len(upstream_res['resolved'])} resolved."
            )

        tracker_rules = rules.get("tracker_rules", {}) if rules else {}
        keys = tracker_rules.get("keys", {})
        id_key = keys.get("issue_id", "number")
        title_key = keys.get("title", "title")
        labels_key = keys.get("labels", "labels")
        state_key = keys.get("state", "state")
        
        close_comments = tracker_rules.get("close_comments", {})
        epic_comment = close_comments.get("epic", "Epic completed. All constituent features successfully delivered and verified.")
        feature_comment_template = close_comments.get("feature", "Resolved. All acceptance criteria and verification tasks for feature '{title}' have been completed and verified.")
        story_comment_template = close_comments.get("user_story", "Resolved. All dependent features/tasks for BDD scenario '{title}' have been completed and verified.")
        usecase_comment_template = close_comments.get("use_case", "Resolved. All dependent user stories and features for use case '{title}' are completed.")

        issue_dict = {}
        for issue in issues:
            raw_id = issue[id_key]
            issue_dict[raw_id] = issue
            if isinstance(raw_id, str) and raw_id.isdigit():
                issue_dict[int(raw_id)] = issue
            elif isinstance(raw_id, int):
                issue_dict[str(raw_id)] = issue

        epic_titles = {}
        story_titles = {}
        usecase_titles = {}
        feature_titles = {}

        # Both sides of every comparison fold through normalize_label (#329): an issue
        # filed as "User Story" lowercases to "user story", never matched "user-story",
        # and was bucketed nowhere — its specification then reported no issue on the
        # tracker and the duplicate stayed open and orphaned.
        labels_config = tracker_rules.get("labels", {})
        epic_label = normalize_label(labels_config.get("epic", "epic"))
        story_label = normalize_label(labels_config.get("user_story", "user-story"))
        usecase_label = normalize_label(labels_config.get("use_case", "use-case"))
        feature_label = normalize_label(labels_config.get("feature", "feature"))

        for num, issue in issue_dict.items():
            if isinstance(num, str) and num.isdigit() and int(num) in epic_titles:
                continue
            norm_title = normalize_title(issue[title_key], rules)
            labels = []
            for l in issue.get(labels_key, []):
                if isinstance(l, dict):
                    labels.append(normalize_label(l.get("name", "")))
                elif isinstance(l, str):
                    labels.append(normalize_label(l))

            if epic_label in labels:
                epic_titles[norm_title] = num
            elif story_label in labels:
                story_titles[norm_title] = num
            elif usecase_label in labels:
                usecase_titles[norm_title] = num
            elif feature_label in labels:
                feature_titles[norm_title] = num
            
        backlog_dirs = rules.get("backlog_directories")
        if not backlog_dirs:
            raise ValueError("Missing 'backlog_directories' in codebase_rules.json")
            
        epics_rel = backlog_dirs.get("epics")
        features_rel = backlog_dirs.get("features")
        stories_rel = backlog_dirs.get("user_stories")
        usecases_rel = backlog_dirs.get("use_cases")
        
        if not all([epics_rel, features_rel, stories_rel, usecases_rel]):
            raise ValueError("Missing epic, features, user_stories, or use_cases path in backlog_directories configuration")
            
        upstream_repo = get_upstream_repository(rules, workspace_dir)
        if not upstream_repo:
            raise ValueError("Missing 'meta.upstream_repository' in codebase_rules.json and remote origin is not configured")

        if args.docs_dir:
            docs_dir = os.path.abspath(args.docs_dir)
            epics_dir = os.path.join(docs_dir, os.path.basename(epics_rel))
            features_dir = os.path.join(docs_dir, os.path.basename(features_rel))
            stories_dir = os.path.join(docs_dir, os.path.basename(stories_rel))
            usecases_dir = os.path.join(docs_dir, os.path.basename(usecases_rel))
            print(f"Scanning backlog files in {docs_dir}...")
        else:
            epics_dir = os.path.join(workspace_dir, epics_rel)
            features_dir = os.path.join(workspace_dir, features_rel)
            stories_dir = os.path.join(workspace_dir, stories_rel)
            usecases_dir = os.path.join(workspace_dir, usecases_rel)
            print("Scanning backlog files...")

        # Build Epic Alias Map for robust child-to-epic title/ID resolution. Extracted to
        # module scope by #319 so the collision it used to contain is testable.
        epic_alias_map = build_epic_alias_map(epics_dir, rules)

        # Build reverse lookup map for Epic issue IDs to normalized titles
        epic_id_to_norm = {}
        for norm_title, issue_id in epic_titles.items():
            epic_id_to_norm[str(issue_id)] = norm_title
            if isinstance(issue_id, int):
                epic_id_to_norm[issue_id] = norm_title
            elif isinstance(issue_id, str) and issue_id.isdigit():
                epic_id_to_norm[int(issue_id)] = norm_title

        def resolve_epic_norm(epic_ref):
            return resolve_epic_reference(epic_ref, epic_alias_map, epic_id_to_norm, rules)

        # Dynamic relationship scanning
        feature_to_epic = {}
        if os.path.exists(features_dir):
            for fn in os.listdir(features_dir):
                if fn.endswith(".md"):
                    fp = os.path.join(features_dir, fn)
                    meta = extract_metadata(fp)
                    epic_name = meta.get("epic") or meta.get("parent_epic")
                    if not epic_name:
                        try:
                            with open(fp, "r", encoding="utf-8", errors="ignore") as f:
                                body_content = f.read()
                            epic_name = extract_epic_from_body(body_content)
                        except Exception as e:
                            print(f"Warning: Failed to extract epic from body of feature {fn}: {e}")
                    if epic_name:
                        resolved_epic = resolve_epic_norm(epic_name)
                        if resolved_epic:
                            feature_to_epic[fn[:-3]] = {resolved_epic}

        story_to_epic = {}
        if os.path.exists(stories_dir):
            for fn in os.listdir(stories_dir):
                if fn.endswith(".md"):
                    fp = os.path.join(stories_dir, fn)
                    meta = extract_metadata(fp)
                    epic_name = meta.get("epic") or meta.get("parent_epic")
                    if not epic_name:
                        try:
                            with open(fp, "r", encoding="utf-8", errors="ignore") as f:
                                body_content = f.read()
                            epic_name = extract_epic_from_body(body_content)
                        except Exception as e:
                            print(f"Warning: Failed to extract epic from body of story {fn}: {e}")
                    epics = set()
                    if epic_name:
                        resolved_epic = resolve_epic_norm(epic_name)
                        if resolved_epic:
                            epics.add(resolved_epic)
                    
                    with open(fp, "r", encoding="utf-8", errors="ignore") as f:
                        content = f.read()
                    feature_refs = re.findall(r'(?:docs/features/|/features/)([a-zA-Z0-9_\-]+)\.md', content)
                    realizes = meta.get("realizes", [])
                    if isinstance(realizes, list):
                        for r in realizes:
                            if isinstance(r, str):
                                # r might be a path like docs/features/foo.md or just foo
                                r_clean = os.path.basename(r)
                                if r_clean.endswith(".md"):
                                    r_clean = r_clean[:-3]
                                feature_refs.append(r_clean)
                                
                    for feat in feature_refs:
                        if feat in feature_to_epic:
                            epics.update(feature_to_epic[feat])
                    
                    story_to_epic[fn[:-3]] = epics

        usecase_to_epic = {}
        if os.path.exists(usecases_dir):
            for fn in os.listdir(usecases_dir):
                if fn.endswith(".md"):
                    fp = os.path.join(usecases_dir, fn)
                    meta = extract_metadata(fp)
                    epic_name = meta.get("epic") or meta.get("parent_epic")
                    if not epic_name:
                        try:
                            with open(fp, "r", encoding="utf-8", errors="ignore") as f:
                                body_content = f.read()
                            epic_name = extract_epic_from_body(body_content)
                        except Exception as e:
                            print(f"Warning: Failed to extract epic from body of use case {fn}: {e}")
                    epics = set()
                    if epic_name:
                        resolved_epic = resolve_epic_norm(epic_name)
                        if resolved_epic:
                            epics.add(resolved_epic)
                        
                    with open(fp, "r", encoding="utf-8", errors="ignore") as f:
                        content = f.read()
                    feature_refs = re.findall(r'(?:docs/features/|/features/)([a-zA-Z0-9_\-]+)\.md', content)
                    realizes = meta.get("realizes", [])
                    if isinstance(realizes, list):
                        for r in realizes:
                            if isinstance(r, str):
                                r_clean = os.path.basename(r)
                                if r_clean.endswith(".md"):
                                    r_clean = r_clean[:-3]
                                feature_refs.append(r_clean)
                                
                    for feat in feature_refs:
                        if feat in feature_to_epic:
                            epics.update(feature_to_epic[feat])
                    story_refs = re.findall(r'(?:docs/user-stories/|/user-stories/)([a-zA-Z0-9_\-]+)\.md', content)
                    for story in story_refs:
                        if story in story_to_epic:
                            epics.update(story_to_epic[story])
                            
                    usecase_to_epic[fn[:-3]] = epics

        # Reconcile Epic checklists
        if os.path.exists(epics_dir):
            for filename in sorted(os.listdir(epics_dir)):
                if not filename.endswith(".md"):
                    continue
                filepath = os.path.join(epics_dir, filename)
                if filename in blocked_specs:
                    print(f'  [Skipped] {filename} - blocked by linter findings (#321)')
                    continue
                title = extract_title(filepath)
                if not title:
                    continue
                epic_norm = normalize_title(title, rules)
                
                child_features = []
                if os.path.exists(features_dir):
                    for feat_fn in sorted(os.listdir(features_dir)):
                        if feat_fn.endswith(".md"):
                            feat_fp = os.path.join(features_dir, feat_fn)
                            if epic_norm in feature_to_epic.get(feat_fn[:-3], set()):
                                feat_title = extract_title(feat_fp)
                                if feat_title:
                                    child_features.append((feat_fn[:-3], feat_title))

                child_stories = []
                if os.path.exists(stories_dir):
                    for story_fn in sorted(os.listdir(stories_dir)):
                        if story_fn.endswith(".md"):
                            story_fp = os.path.join(stories_dir, story_fn)
                            if epic_norm in story_to_epic.get(story_fn[:-3], set()):
                                story_title = extract_title(story_fp)
                                if story_title:
                                    child_stories.append((story_fn[:-3], story_title))

                child_usecases = []
                if os.path.exists(usecases_dir):
                    for uc_fn in sorted(os.listdir(usecases_dir)):
                        if uc_fn.endswith(".md"):
                            uc_fp = os.path.join(usecases_dir, uc_fn)
                            if epic_norm in usecase_to_epic.get(uc_fn[:-3], set()):
                                uc_title = extract_title(uc_fp)
                                if uc_title:
                                    child_usecases.append((uc_fn[:-3], uc_title))

                reconcile_epic_checklists(
                    filepath, 
                    child_features, 
                    child_stories, 
                    child_usecases, 
                    epic_titles, 
                    feature_titles, 
                    story_titles, 
                    usecase_titles, 
                    rules
                )

        # One issue belongs to exactly one spec file. Shared across all four loops so a
        # collision is caught whatever type the colliding files are (#316).
        claimed_issues = {}

        # Process Epics
        if os.path.exists(epics_dir):
            for filename in sorted(os.listdir(epics_dir)):
                if not filename.endswith(".md"):
                    continue
                filepath = os.path.join(epics_dir, filename)
                if filename in blocked_specs:
                    print(f'  [Skipped] {filename} - blocked by linter findings (#321)')
                    continue
                resolve_issue_ids_in_file(filepath, epic_titles, feature_titles, story_titles, usecase_titles, rules=rules)
                title = extract_title(filepath)
                if not title:
                    continue
                
                issue_num = resolve_spec_issue_number(
                    filepath, title, epic_titles, issue_dict, rules=rules,
                    item_type="Epic", claimed=claimed_issues,
                )
                if issue_num is not None:
                    updated_content, completed = update_checklist_in_file(filepath, issue_dict, rules)
                    is_open = str(issue_dict[issue_num][state_key]).upper() == keys.get("open_state_value", "OPEN").upper()
                    if is_open:
                        sync_issue_body_to_tracker(
                            issue_num, filepath, issue_type="Epic", rules=rules,
                            issue_record=issue_dict[issue_num], provider_adapter=provider_adapter,
                        )
                        if completed and not is_already_resolved(issue_dict[issue_num], rules):
                            resolve_issue_on_tracker(
                                issue_num, 
                                epic_comment,
                                rules=rules,
                                provider_adapter=provider_adapter,
                            )
                            issue_dict[issue_num].setdefault("labels", []).append(
                                {"name": get_resolved_label(rules)}
                            )
                else:
                    print(
                        f"Warning: No Epic issue on the tracker for {filename} — "
                        f"no issue_id in its frontmatter and no title match for '{title}'"
                    )

        # Process Features
        if os.path.exists(features_dir):
            for filename in sorted(os.listdir(features_dir)):
                if not filename.endswith(".md"):
                    continue
                filepath = os.path.join(features_dir, filename)
                if filename in blocked_specs:
                    print(f'  [Skipped] {filename} - blocked by linter findings (#321)')
                    continue
                resolve_issue_ids_in_file(filepath, epic_titles, feature_titles, story_titles, usecase_titles, rules=rules)
                title = extract_title(filepath)
                if not title:
                    continue
                
                issue_num = resolve_spec_issue_number(
                    filepath, title, feature_titles, issue_dict, rules=rules,
                    item_type="Feature", claimed=claimed_issues,
                )
                if issue_num is not None:
                    _, completed = update_checklist_in_file(filepath, issue_dict, rules)
                    is_open = str(issue_dict[issue_num][state_key]).upper() == keys.get("open_state_value", "OPEN").upper()
                    if is_open:
                        sync_issue_body_to_tracker(
                            issue_num, filepath, issue_type="Feature", rules=rules,
                            issue_record=issue_dict[issue_num], provider_adapter=provider_adapter,
                        )
                        if completed and not is_already_resolved(issue_dict[issue_num], rules):
                            resolve_issue_on_tracker(
                                issue_num,
                                feature_comment_template.format(title=title),
                                rules=rules,
                                provider_adapter=provider_adapter,
                            )
                            issue_dict[issue_num].setdefault("labels", []).append(
                                {"name": get_resolved_label(rules)}
                            )
                else:
                    print(
                        f"Warning: No Feature issue on the tracker for {filename} — "
                        f"no issue_id in its frontmatter and no title match for '{title}'"
                    )

        # Process User Stories
        if os.path.exists(stories_dir):
            for filename in sorted(os.listdir(stories_dir)):
                if not filename.endswith(".md"):
                    continue
                filepath = os.path.join(stories_dir, filename)
                if filename in blocked_specs:
                    print(f'  [Skipped] {filename} - blocked by linter findings (#321)')
                    continue
                resolve_issue_ids_in_file(filepath, epic_titles, feature_titles, story_titles, usecase_titles, rules=rules)
                title = extract_title(filepath)
                if not title:
                    continue
                
                issue_num = resolve_spec_issue_number(
                    filepath, title, story_titles, issue_dict, rules=rules,
                    item_type="User Story", claimed=claimed_issues,
                )
                if issue_num is not None:
                    _, completed = update_checklist_in_file(filepath, issue_dict, rules)
                    is_open = str(issue_dict[issue_num][state_key]).upper() == keys.get("open_state_value", "OPEN").upper()
                    if is_open:
                        sync_issue_body_to_tracker(
                            issue_num, filepath, issue_type="User Story", rules=rules,
                            issue_record=issue_dict[issue_num], provider_adapter=provider_adapter,
                        )
                        if completed and not is_already_resolved(issue_dict[issue_num], rules):
                            resolve_issue_on_tracker(
                                issue_num,
                                story_comment_template.format(title=title),
                                rules=rules,
                                provider_adapter=provider_adapter,
                            )
                            issue_dict[issue_num].setdefault("labels", []).append(
                                {"name": get_resolved_label(rules)}
                            )
                else:
                    print(
                        f"Warning: No User Story issue on the tracker for {filename} — "
                        f"no issue_id in its frontmatter and no title match for '{title}'"
                    )

        # Process Use Cases
        if os.path.exists(usecases_dir):
            for filename in sorted(os.listdir(usecases_dir)):
                if not filename.endswith(".md"):
                    continue
                filepath = os.path.join(usecases_dir, filename)
                if filename in blocked_specs:
                    print(f'  [Skipped] {filename} - blocked by linter findings (#321)')
                    continue
                resolve_issue_ids_in_file(filepath, epic_titles, feature_titles, story_titles, usecase_titles, rules=rules)
                title = extract_title(filepath)
                if not title:
                    continue
                
                issue_num = resolve_spec_issue_number(
                    filepath, title, usecase_titles, issue_dict, rules=rules,
                    item_type="Use Case", claimed=claimed_issues,
                )
                if issue_num is not None:
                    _, completed = update_checklist_in_file(filepath, issue_dict, rules)
                    is_open = str(issue_dict[issue_num][state_key]).upper() == keys.get("open_state_value", "OPEN").upper()
                    if is_open:
                        sync_issue_body_to_tracker(
                            issue_num, filepath, issue_type="Use Case", rules=rules,
                            issue_record=issue_dict[issue_num], provider_adapter=provider_adapter,
                        )
                        if completed and not is_already_resolved(issue_dict[issue_num], rules):
                            resolve_issue_on_tracker(
                                issue_num,
                                usecase_comment_template.format(title=title),
                                rules=rules,
                                provider_adapter=provider_adapter,
                            )
                            issue_dict[issue_num].setdefault("labels", []).append(
                                {"name": get_resolved_label(rules)}
                            )
                else:
                    print(
                        f"Warning: No Use Case issue on the tracker for {filename} — "
                        f"no issue_id in its frontmatter and no title match for '{title}'"
                    )

        if blocked_specs:
            # Issue #321 - skipping is not tolerance. Everything valid has now been
            # synchronised, but the corpus still contains specifications the linter
            # rejected, so the run reports failure. A caller that treated a skipped
            # item as published would be the gate quietly disappearing.
            print(
                f"Backlog reconciliation complete for valid specifications. "
                f"{len(blocked_specs)} specification(s) were SKIPPED because the "
                f"linter rejected them: {', '.join(sorted(blocked_specs))}",
                file=sys.stderr,
            )
            sys.exit(1)

        print("Backlog reconciliation complete.")

    except BaseException as e:
        exit_code = 1
        if isinstance(e, SystemExit):
            if isinstance(e.code, int):
                exit_code = e.code
            elif e.code is None:
                exit_code = 0
        
        if exit_code != 0:
            tb_str = traceback.format_exc()
            print(tb_str, file=sys.stderr)
            try:
                # Insert the src directory of the parity_auditor package into sys.path
                src_dir = os.path.abspath(os.path.join(workspace_dir, "skills", "spec-orchestrator", "parity_auditor", "src"))
                if src_dir not in sys.path:
                    sys.path.insert(0, src_dir)
                from parity_auditor.utils.diagnostics import serialize_diagnostics
                serialize_diagnostics(
                    workspace_dir=workspace_dir,
                    tool_name="reconcile_backlog",
                    exit_code=exit_code,
                    errors=[str(e)],
                    traceback_str=tb_str
                )
            except Exception as diag_err:
                print(f"Warning: Failed to serialize diagnostics: {diag_err}", file=sys.stderr)
        raise

if __name__ == "__main__":
    main()
# Refresh commit timestamp
