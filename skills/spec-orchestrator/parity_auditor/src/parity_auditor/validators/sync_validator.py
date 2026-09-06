"""Compares the registered issue backlog against the local specification files.

Titles normalise through ``reconcile_backlog.py``'s own function, bound by reference in
``utils/spec_titles.py``. This module used to carry a private copy that had drifted from
it in two ways -- it lacked the guard that keeps the original title when prefix-stripping
(so every prefix-only title, "Epic 2" and "Epic 3" alike, collapsed to one
key), and it folded ``_`` to a space (so two titles the reconciler keys apart looked
identical here). A gate that collides in a different space from the consumer it protects
reports collisions that do not exist and misses the ones that do; see
``utils/spec_titles.py`` for why the reconciler is the definition site.
"""

import os
import sys
import re
import json
import shutil
import signal
import subprocess
import ssl
import urllib.request
import urllib.parse
import urllib.error
import netrc
from typing import List, Dict, Optional, Any, Tuple
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository
from ..utils.spec_titles import normalize_spec_title


def _terminate_process_group(proc: subprocess.Popen) -> None:
    """Terminate the process group cleanly with SIGKILL."""
    try:
        if proc.poll() is None:
            try:
                pgid = os.getpgid(proc.pid)
                os.killpg(pgid, signal.SIGKILL)
            except (ProcessLookupError, PermissionError):
                try:
                    proc.kill()
                except Exception:
                    pass
    except (ProcessLookupError, PermissionError):
        pass


def run_bounded_process(
    cmd: List[str],
    cwd: Optional[str] = None,
    timeout: float = 30.0,
) -> Tuple[int, str, str]:
    """
    Run command in a dedicated process group session (start_new_session=True).
    Upon subprocess.TimeoutExpired or failure/exception, kill the entire process
    group with os.killpg(os.getpgid(proc.pid), signal.SIGKILL) to deterministically
    reclaim child processes and network sockets.
    """
    proc = None
    try:
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            cwd=cwd,
            start_new_session=True,
        )
        stdout, stderr = proc.communicate(timeout=timeout)
        return proc.returncode, stdout, stderr
    except subprocess.TimeoutExpired:
        if proc is not None:
            _terminate_process_group(proc)
            try:
                proc.communicate(timeout=2)
            except Exception:
                pass
        raise
    except Exception:
        if proc is not None:
            _terminate_process_group(proc)
            try:
                proc.communicate(timeout=2)
            except Exception:
                pass
        raise


def parse_git_remote_url(remote_url: str) -> Dict[str, Any]:
    """
    Parse a git remote origin URL into its components:
    - raw: raw URL string
    - is_gitlab: True if domain contains 'gitlab'
    - project_path: repository path (e.g. 'gintatkinson/DEAP01-spec-core')
    - server_url: base server URL (e.g. 'https://gitlab.com')
    - host: domain host name (e.g. 'gitlab.com' or 'github.com')
    """
    if not remote_url:
        return {"raw": "", "is_gitlab": False, "project_path": None, "server_url": None, "host": None}
    
    clean_url = remote_url.strip()
    if clean_url.endswith(".git"):
        clean_url = clean_url[:-4]
        
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
            "host": host,
        }
        
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


def get_git_remote_info(workspace_dir: Optional[str] = None) -> Optional[Dict[str, Any]]:
    try:
        rc, stdout, _ = run_bounded_process(["git", "remote", "get-url", "origin"], cwd=workspace_dir, timeout=10.0)
        if rc == 0 and stdout.strip():
            return parse_git_remote_url(stdout.strip())
    except Exception:
        pass
    return None


def detect_tracker_provider(
    cli_provider: Optional[str] = None,
    rules: Optional[Any] = None,
    workspace_dir: Optional[str] = None,
) -> str:
    """
    Detect the active issue tracker provider (gitlab vs github vs jira).
    Resolution precedence:
    1. CLI parameter override (if provided and not 'auto')
    2. Environment variables (TRACKER_PROVIDER / PROVIDER)
    3. codebase_rules.json tracker_rules.provider (if set and not 'auto' / 'github')
    4. CI environment variables (GITLAB_CI -> gitlab, GITHUB_ACTIONS -> github)
    5. Git remote origin URL (domain contains gitlab -> gitlab)
    6. codebase_rules.json configured provider fallback
    7. Default fallback: github
    """
    if cli_provider and str(cli_provider).lower() != "auto":
        return str(cli_provider).lower()
        
    env_provider = os.environ.get("TRACKER_PROVIDER") or os.environ.get("PROVIDER")
    if env_provider and env_provider.lower() != "auto":
        return env_provider.lower()

    if rules is not None:
        configured = None
        if hasattr(rules, "tracker_rules") and isinstance(rules.tracker_rules, dict):
            configured = rules.tracker_rules.get("provider")
        elif isinstance(rules, dict):
            configured = rules.get("tracker_rules", {}).get("provider") if "tracker_rules" in rules else rules.get("provider")
        if configured and str(configured).lower() not in ("auto", "github"):
            return str(configured).lower()

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

    if os.environ.get("GITLAB_CI") or os.environ.get("CI_SERVER_URL") or os.environ.get("CI_PROJECT_PATH"):
        return "gitlab"
    if os.environ.get("GITHUB_ACTIONS") or os.environ.get("GITHUB_REPOSITORY"):
        return "github"

    remote_info = get_git_remote_info(workspace_dir)
    if remote_info and remote_info.get("is_gitlab"):
        return "gitlab"

    if rules is not None:
        configured = None
        if hasattr(rules, "tracker_rules") and isinstance(rules.tracker_rules, dict):
            configured = rules.tracker_rules.get("provider")
        elif isinstance(rules, dict):
            configured = rules.get("tracker_rules", {}).get("provider") if "tracker_rules" in rules else rules.get("provider")
        if configured:
            return str(configured).lower()

    return "github"


def _fetch_gitlab_issues(
    workspace_dir: Optional[str] = None,
    tracker_rules: Optional[Dict[str, Any]] = None,
) -> List[Dict[str, Any]]:
    """
    Fetch registered issue backlog from GitLab using glab CLI or REST API v4.
    Maps GitLab iid to number and handles offline / error conditions cleanly.
    """
    if os.environ.get("OFFLINE"):
        return []

    if tracker_rules is None:
        tracker_rules = {}

    try:
        timeout = float(os.environ.get("PARITY_AUDITOR_GL_TIMEOUT", os.environ.get("PARITY_AUDITOR_GH_TIMEOUT", "30.0")))
    except (ValueError, TypeError):
        timeout = 30.0

    # 1. Attempt glab CLI if available
    if shutil.which("glab"):
        try:
            cmd = ["glab", "issue", "list", "--all", "--per-page", "1000", "--output", "json"]
            rc, stdout, stderr = run_bounded_process(cmd, cwd=workspace_dir, timeout=timeout)
            if rc == 0 and stdout.strip():
                issues = json.loads(stdout)
                if isinstance(issues, list):
                    for issue in issues:
                        if "iid" in issue and "number" not in issue:
                            issue["number"] = issue["iid"]
                    return issues
            else:
                if rc != 0:
                    print(f"Warning: glab CLI exited with code {rc}: {stderr.strip()}", file=sys.stderr)
        except subprocess.TimeoutExpired:
            print(f"Warning: glab CLI timed out after {timeout} seconds.", file=sys.stderr)
        except Exception as e:
            print(f"Warning: Failed to run glab CLI: {e}", file=sys.stderr)

    # 2. GitLab REST API v4 fallback
    server_url = None
    for env_var in ("GITLAB_URL", "CI_SERVER_URL", "GL_SERVER_URL"):
        val = os.environ.get(env_var)
        if val and val.strip():
            server_url = val.strip().rstrip("/")
            break
    if not server_url:
        server_url = tracker_rules.get("server_url")
    if not server_url:
        remote_info = get_git_remote_info(workspace_dir)
        if remote_info and remote_info.get("server_url") and remote_info.get("is_gitlab"):
            server_url = remote_info["server_url"]
    if not server_url:
        server_url = "https://gitlab.com"
    server_url = server_url.rstrip("/")

    raw_project_id = None
    for env_var in ("CI_PROJECT_PATH", "CI_PROJECT_ID", "GITLAB_PROJECT", "GL_PROJECT"):
        val = os.environ.get(env_var)
        if val and val.strip():
            raw_project_id = val.strip()
            break
    if not raw_project_id:
        raw_project_id = tracker_rules.get("project_id")
    if not raw_project_id:
        remote_info = get_git_remote_info(workspace_dir)
        if remote_info and remote_info.get("project_path"):
            raw_project_id = remote_info["project_path"]
    if not raw_project_id:
        env_repo = os.environ.get("UPSTREAM_REPOSITORY") or os.environ.get("GIT_REMOTE_ORIGIN")
        if env_repo:
            raw_project_id = env_repo.strip()

    if not raw_project_id:
        return []

    raw_str = str(raw_project_id).strip()
    if raw_str.isdigit():
        project_id_encoded = raw_str
    else:
        project_id_encoded = urllib.parse.quote(raw_str, safe="")

    # Resolve token
    token = None
    token_type = "PRIVATE-TOKEN"
    for var in ("GITLAB_TOKEN", "GL_TOKEN"):
        val = os.environ.get(var)
        if val and val.strip():
            token = val.strip()
            token_type = "PRIVATE-TOKEN"
            break
    if not token:
        job_token = os.environ.get("CI_JOB_TOKEN")
        if job_token and job_token.strip():
            token = job_token.strip()
            token_type = "JOB-TOKEN"
    if not token and shutil.which("glab"):
        try:
            rc, stdout, _ = run_bounded_process(["glab", "auth", "token"], cwd=workspace_dir, timeout=5.0)
            if rc == 0 and stdout.strip():
                token = stdout.strip()
                token_type = "PRIVATE-TOKEN"
        except Exception:
            pass
    if not token:
        try:
            hostname = urllib.parse.urlparse(server_url).hostname or "gitlab.com"
            auth = netrc.netrc().authenticators(hostname)
            if auth and auth[2] and auth[2].strip():
                token = auth[2].strip()
                token_type = "PRIVATE-TOKEN"
        except Exception:
            pass

    if not token:
        return []

    ca_cert_path = os.environ.get("GITLAB_CA_CERT_PATH") or os.environ.get("SSL_CERT_FILE")
    ctx = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
    if ca_cert_path and os.path.isfile(ca_cert_path):
        try:
            ctx.load_verify_locations(cafile=ca_cert_path)
        except Exception as e:
            print(f"Warning: Failed to load CA certificate from {ca_cert_path}: {e}", file=sys.stderr)

    all_issues = []
    page = 1

    try:
        while True:
            params = {
                "scope": "all",
                "per_page": 100,
                "page": page,
            }
            url = f"{server_url}/api/v4/projects/{project_id_encoded}/issues?{urllib.parse.urlencode(params)}"
            headers = {
                "Accept": "application/json",
                "User-Agent": "DEAP-Parity-Auditor/1.0",
                token_type: token,
            }
            req = urllib.request.Request(url=url, headers=headers, method="GET")
            with urllib.request.urlopen(req, context=ctx, timeout=timeout) as resp:
                raw_body = resp.read().decode("utf-8")
                issues = json.loads(raw_body) if raw_body.strip() else []
                resp_headers = {k: v for k, v in resp.headers.items()}

            if not isinstance(issues, list):
                break

            for issue in issues:
                if "iid" in issue and "number" not in issue:
                    issue["number"] = issue["iid"]
                all_issues.append(issue)

            next_page_hdr = resp_headers.get("X-Next-Page") or resp_headers.get("x-next-page")
            if next_page_hdr and str(next_page_hdr).strip() and str(next_page_hdr).strip() != "0":
                page = int(next_page_hdr)
            elif len(issues) == 100:
                page += 1
            else:
                break

        return all_issues
    except Exception as e:
        print(f"Warning: Failed to fetch GitLab issues via REST API v4: {e}", file=sys.stderr)
        return []


def _fetch_github_issues(
    workspace_dir: Optional[str] = None,
    tracker_rules: Optional[Dict[str, Any]] = None,
) -> List[Dict[str, Any]]:
    """
    Fetch registered issue backlog from GitHub via configured command or gh CLI.
    """
    if os.environ.get("OFFLINE"):
        return []

    if tracker_rules is None:
        tracker_rules = {}

    cmd = tracker_rules.get("commands", {}).get("list_issues") if isinstance(tracker_rules, dict) else None
    if not cmd:
        if not shutil.which("gh"):
            return []
        cmd = ["gh", "issue", "list", "--limit", "1000", "--state", "all", "--json", "number,title,state,labels"]

    try:
        timeout = float(os.environ.get("PARITY_AUDITOR_GH_TIMEOUT", "30.0"))
    except (ValueError, TypeError):
        timeout = 30.0

    try:
        rc, stdout, stderr = run_bounded_process(cmd, cwd=workspace_dir, timeout=timeout)
        if rc != 0:
            print(f"Warning: Failed to fetch issue backlog from remote: {stderr.strip()}", file=sys.stderr)
            return []
        issues = json.loads(stdout)
        if isinstance(issues, list):
            return issues
        return []
    except Exception as e:
        print(f"Warning: Issue backlog offline or unavailable: {e}", file=sys.stderr)
        return []


class SyncValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        # Upstream compiler repository exemption: docs/epics and docs/features are
        # clean landing zones BY DESIGN and feature issues without local
        # specification files are upstream tooling features tracked via git commits
        # rather than markdown specs. Mirrors the reconciler's upstream-mode
        # exemption of the same class of findings (issue #68 mechanism, issues
        # #74/#73/#72/#70/#67/#64/#62/#61/#60/#59).
        if repo.is_upstream_compiler_repo() and not repo.has_configured_target_code_directories():
            print("Note: Upstream compiler repository mode. Skipping out-of-sync "
                  "missing-local-specification findings for upstream tooling feature issues.")
            return []

        rules = repo.get_codebase_rules()
        tracker_rules = rules.tracker_rules if hasattr(rules, "tracker_rules") and isinstance(rules.tracker_rules, dict) else {}
        backlog_dirs = rules.backlog_directories
        
        epics_dir = os.path.join(repo.workspace_dir, backlog_dirs.epics)
        features_dir = os.path.join(repo.workspace_dir, backlog_dirs.features)
        
        errors = []
        
        # 1. Detect provider and fetch registered issues
        provider = detect_tracker_provider(
            cli_provider=kwargs.get("provider"),
            rules=tracker_rules,
            workspace_dir=repo.workspace_dir,
        )
        
        if provider == "gitlab":
            issues = _fetch_gitlab_issues(workspace_dir=repo.workspace_dir, tracker_rules=tracker_rules)
        else:
            issues = _fetch_github_issues(workspace_dir=repo.workspace_dir, tracker_rules=tracker_rules)

        if not issues:
            return []
            
        # Parse tracker issues and support label formats (GitHub & GitLab)
        labels_config = tracker_rules.get("labels", {}) if isinstance(tracker_rules, dict) else {}
        raw_epic_label = str(labels_config.get("epic", "epic")).lower()
        raw_feature_label = str(labels_config.get("feature", "feature")).lower()
        
        epic_labels = {
            raw_epic_label,
            f"type::{raw_epic_label}",
            f"type:{raw_epic_label}",
            "epic",
            "type::epic",
        }
        feature_labels = {
            raw_feature_label,
            f"type::{raw_feature_label}",
            f"type:{raw_feature_label}",
            "feature",
            "type::feature",
        }
        
        # Keyed by (spec_type, normalized_title). The spec type comes from the issue
        # label and is part of the identity of a spec: an epic issue is not satisfied
        # by a same-titled feature file, and two issues that normalize to the same
        # title must not overwrite one another. Issue #303.
        tracker_specs = {}
        tracker_indices = {}

        for issue in issues:
            # Map iid to number if missing
            if "iid" in issue and "number" not in issue:
                issue["number"] = issue["iid"]
            elif issue.get("number") is None and "iid" in issue:
                issue["number"] = issue["iid"]

            labels = []
            for l in issue.get("labels", []):
                if isinstance(l, dict):
                    labels.append(str(l.get("name", "")).lower())
                elif isinstance(l, str):
                    labels.append(l.lower())
                    
            is_spec = False
            spec_type = None
            if any(lbl in epic_labels for lbl in labels):
                is_spec = True
                spec_type = "epic"
            elif any(lbl in feature_labels for lbl in labels):
                is_spec = True
                spec_type = "feature"
                
            if not is_spec:
                continue
                
            title = issue.get("title", "")
            norm_title = normalize_spec_title(title)
            tracker_specs[(spec_type, norm_title)] = issue

            # Extract index, e.g. "Epic 2: Common Types" -> index 2
            match = re.search(r'\b(epic|feature|feat)[s]?[- ]*(\d+)', title, re.IGNORECASE)
            if match:
                idx = int(match.group(2))
                std_type = "epic" if match.group(1).lower().startswith("epic") else "feature"
                # Store the full tracker_specs key so the collision report can look the
                # issue back up even when the title-derived type differs from the label.
                tracker_indices[(std_type, idx)] = (spec_type, norm_title)
                
        # 2. Scan local files
        local_specs = set()
        local_indices = {}
        
        def scan_local_dir(directory, std_type):
            if not os.path.exists(directory):
                return
            for filename in os.listdir(directory):
                if not filename.endswith(".md"):
                    continue
                filepath = os.path.join(directory, filename)
                title = None
                try:
                    with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                        content = f.read(2048)
                    title_match = re.search(r'^title:\s*(["\']?)(.*?)\1\s*$', content, re.MULTILINE)
                    if title_match:
                        title = title_match.group(2).strip()
                    else:
                        h1_match = re.search(r'^#\s+(.*?)$', content, re.MULTILINE)
                        if h1_match:
                            title = h1_match.group(1).strip()
                except Exception:
                    pass
                    
                if title:
                    norm_title = normalize_spec_title(title)
                    local_specs.add((std_type, norm_title))
                    
                    match = re.search(r'\b(epic|feature|feat)[s]?[- ]*(\d+)', filename, re.IGNORECASE)
                    if not match:
                        match = re.search(r'\b(epic|feature|feat)[s]?[- ]*(\d+)', title, re.IGNORECASE)
                    if match:
                        idx = int(match.group(2))
                        local_indices[(std_type, idx)] = norm_title
                        
        scan_local_dir(epics_dir, "epic")
        scan_local_dir(features_dir, "feature")
        
        # 3. Check for missing local specs
        for spec_key, issue in tracker_specs.items():
            if spec_key not in local_specs:
                issue_num = issue.get("number", issue.get("iid", "unknown"))
                errors.append(Finding(
                    "tracker-issue-without-local-specification",
                    f"Missing specification file for registered Issue #{issue_num} - '{issue.get('title', '')}'. Please check your branch baseline.",
                    location=spec_key[0],
                ))

        # 4. Check for index collisions
        for (std_type, idx), norm_title in local_indices.items():
            tracker_key = tracker_indices.get((std_type, idx))
            if tracker_key and tracker_key[1] != norm_title:
                issue_num = tracker_specs.get(tracker_key, {}).get("number", tracker_specs.get(tracker_key, {}).get("iid", "unknown"))
                errors.append(Finding(
                    "spec-index-collides-with-tracker-issue",
                    f"Index collision detected. Local specification with index {idx} ('{norm_title}') overlaps with registered Issue #{issue_num} ('{tracker_key[1]}').",
                    location=std_type,
                ))
                
        return errors

