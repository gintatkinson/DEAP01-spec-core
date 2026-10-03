#!/usr/bin/env python3
"""
Git remote URL parsing, provider detection, repository role resolution, and metadata preservation.
/// Realises: [InstallerMetadataManager]
"""
import json
import os
import re
import subprocess
import urllib.parse
from dataclasses import dataclass, field
from enum import Enum
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

from scripts.installer.rollback import (
    CLIValidationError,
    InstallerError,
    MetadataResolutionError,
)


class RepositoryRole(str, Enum):
    """Repository classification role according to architecture tiers."""
    UPSTREAM_SPEC_CORE_COMPILER = "UPSTREAM_SPEC_CORE_COMPILER"
    DOMAIN_DISTRIBUTION_TEMPLATE = "DOMAIN_DISTRIBUTION_TEMPLATE"
    DOWNSTREAM_CUSTOMER_PROJECT = "DOWNSTREAM_CUSTOMER_PROJECT"


class TrackerProvider(str, Enum):
    """Supported issue tracker and CI/CD platforms."""
    GITHUB = "github"
    GITLAB = "gitlab"
    JIRA = "jira"
    AUTO = "auto"


@dataclass(frozen=True)
class GitRemoteInfo:
    """Dissected Git remote origin URL parameters."""
    raw_url: str
    platform: str
    server_url: str
    namespace: str
    project: str


@dataclass
class InstallOptions:
    """Raw and normalized CLI configuration options."""
    installer_root: Path
    target_dir: Path
    cli_role: Optional[str] = None
    target_role: RepositoryRole = RepositoryRole.DOWNSTREAM_CUSTOMER_PROJECT
    provider: str = "auto"
    gitlab_url: str = "https://gitlab.com"
    gitlab_group: str = ""
    github_org: str = ""
    jira_url: str = "https://your-domain.atlassian.net"
    jira_project: str = ""
    jira_email: str = ""
    domain_url: str = ""
    domain_name: str = ""


@dataclass
class PreservedMetadata:
    """Downstream repository files captured before staging for non-destructive restore."""
    project_metadata_json: Optional[str] = None
    profile_config_json: Optional[str] = None
    schema_sysml: Optional[str] = None
    schema_digest_json: Optional[str] = None
    lineage_json: Optional[str] = None
    gitignore_lines: List[str] = field(default_factory=list)
    existing_schema_files: List[Tuple[str, bytes]] = field(default_factory=list)


@dataclass
class ProjectMetadataInfo:
    """Extracted project attributes used for dynamic README generation."""
    project_name: str = "Downstream Systems Engineering Project"
    project_desc: str = (
        "This repository is an installed downstream implementation workspace governed "
        "by the **Digital Engineering Agent Platform (DEAP)** for cyber-physical "
        "infrastructure safety, real-time control, run-time assurance (RTA), and autonomous operations."
    )
    tech_profile: str = "`Target Platform Execution Profile`"
    regulatory_frameworks: str = "`Schema-Derived Regulatory Standards`"
    domain_remote_url: str = ""


def parse_git_remote_url(remote_url: str) -> GitRemoteInfo:
    """
    Safely dissects Git remote URLs (HTTPS, HTTP, SSH, SCP-style) into components
    without invoking shell subprocesses.
    """
    url = remote_url.strip()
    if url.endswith(".git"):
        url = url[:-4]

    host = ""
    server_url = ""
    path = ""

    if "://" in url:
        parsed = urllib.parse.urlsplit(url)
        netloc = parsed.netloc
        host = netloc.split("@")[-1].split(":")[0]
        scheme = parsed.scheme if parsed.scheme in ("http", "https") else "https"
        server_url = f"{scheme}://{host}"
        path = parsed.path.strip("/")
    else:
        match = re.match(r"^(?:[^@]+@)?([^:/]+):?(?:\d+)?(?:/|:)?(.*)$", url)
        if match:
            host = match.group(1)
            path = match.group(2).strip("/")
            server_url = f"https://{host}"
        else:
            host = ""
            path = url.strip("/")
            server_url = ""

    parts = [p for p in path.split("/") if p]
    project = parts[-1] if parts else ""
    namespace = "/".join(parts[:-1]) if len(parts) > 1 else ""

    platform = "unknown"
    if "gitlab" in host.lower() or "gitlab" in url.lower():
        platform = "gitlab"
    elif "github" in host.lower() or "github" in url.lower():
        platform = "github"

    return GitRemoteInfo(
        raw_url=remote_url,
        platform=platform,
        server_url=server_url,
        namespace=namespace,
        project=project,
    )


def get_git_remote_url(repo_dir: Path) -> Optional[str]:
    """
    Queries git remote origin URL using list-style subprocess call:
    ['git', '-C', str(repo_dir), 'remote', 'get-url', 'origin']
    with fallback to:
    ['git', '-C', str(repo_dir), 'config', '--get', 'remote.origin.url'].
    """
    if not repo_dir.exists():
        return None
    try:
        res = subprocess.run(
            ["git", "-C", str(repo_dir), "remote", "get-url", "origin"],
            capture_output=True,
            text=True,
            check=False,
            shell=False,
        )
        if res.returncode == 0 and res.stdout.strip():
            return res.stdout.strip()

        res_fallback = subprocess.run(
            ["git", "-C", str(repo_dir), "config", "--get", "remote.origin.url"],
            capture_output=True,
            text=True,
            check=False,
            shell=False,
        )
        if res_fallback.returncode == 0 and res_fallback.stdout.strip():
            return res_fallback.stdout.strip()
    except Exception:
        pass
    return None


def detect_repository_role(
    target_dir: Path,
    cli_role: Optional[str],
    detected_project: str,
    remote_url: str,
    domain_url: str,
    domain_name: str,
) -> RepositoryRole:
    """
    Determines repository role following the canonical precedence rules:
    1. Explicit CLI --role parameter (domain-template vs customer-project).
    2. Directory basename or detected project matching 'uav-*' -> DOWNSTREAM_CUSTOMER_PROJECT.
    3. Explicit domain parameters (--domain-url, --domain-name), directory basename 'DEAP-*',
       or remote URL containing 'DEAP-*' -> DOMAIN_DISTRIBUTION_TEMPLATE.
    4. Existing '.pipeline/lineage.json' classification if present.
    5. Fallback -> DOWNSTREAM_CUSTOMER_PROJECT.
    """
    if cli_role:
        normalized = cli_role.strip().lower().replace("-", "_")
        if normalized in ("domain_template", "domain", "domain_distribution_template"):
            return RepositoryRole.DOMAIN_DISTRIBUTION_TEMPLATE
        elif normalized in (
            "customer_project",
            "customer",
            "downstream_customer_project",
            "downstream_application_workspace",
            "workspace",
        ):
            return RepositoryRole.DOWNSTREAM_CUSTOMER_PROJECT
        else:
            raise CLIValidationError(
                f"Error: Invalid --role '{cli_role}'. Valid values: "
                f"'domain-template', 'customer-project', 'DOMAIN_DISTRIBUTION_TEMPLATE', 'DOWNSTREAM_CUSTOMER_PROJECT'."
            )

    dir_base = target_dir.name
    if dir_base.startswith("uav-") or detected_project.startswith("uav-"):
        return RepositoryRole.DOWNSTREAM_CUSTOMER_PROJECT

    if (
        domain_url
        or domain_name
        or dir_base.startswith("DEAP-")
        or ("DEAP-" in remote_url)
        or detected_project.startswith("DEAP-")
    ):
        return RepositoryRole.DOMAIN_DISTRIBUTION_TEMPLATE

    lineage_path = target_dir / ".pipeline" / "lineage.json"
    if lineage_path.is_file():
        try:
            with open(lineage_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                meta_role = data.get("role") or data.get("classification") or ""
                if meta_role in ("DOMAIN_DISTRIBUTION_TEMPLATE", "DOWNSTREAM_CUSTOMER_PROJECT"):
                    return RepositoryRole(meta_role)
        except Exception:
            pass

    return RepositoryRole.DOWNSTREAM_CUSTOMER_PROJECT


def resolve_provider_and_metadata(
    options: InstallOptions,
    remote_info: Optional[GitRemoteInfo],
) -> InstallOptions:
    """
    Applies auto-detection heuristics:
    - If provider is 'auto' and remote platform is recognized, sets provider.
    - If provider is 'gitlab', auto-populates gitlab_group and gitlab_url if unset.
    - If provider is 'github', auto-populates github_org if unset.
    - If provider remains 'auto', defaults to 'github'.
    """
    if remote_info:
        if options.provider == "auto" and remote_info.platform != "unknown":
            options.provider = remote_info.platform
            print(f"Auto-detected platform '{options.provider}' from git remote: {remote_info.raw_url}")

        if options.provider == "gitlab":
            if not options.gitlab_group and remote_info.namespace:
                options.gitlab_group = remote_info.namespace
                print(f"Auto-detected GitLab group '{options.gitlab_group}' from git remote")
            if (
                options.gitlab_url == "https://gitlab.com"
                and remote_info.server_url
                and remote_info.server_url != "https://gitlab.com"
            ):
                options.gitlab_url = remote_info.server_url
                print(f"Auto-detected GitLab server URL '{options.gitlab_url}' from git remote")

        if options.provider == "github":
            if not options.github_org and remote_info.namespace:
                options.github_org = remote_info.namespace
                print(f"Auto-detected GitHub organization '{options.github_org}' from git remote")

    if options.provider == "auto":
        options.provider = "github"

    if options.provider not in ("github", "gitlab", "jira"):
        raise CLIValidationError(
            f"Error: Invalid provider '{options.provider}'. Must be one of 'github', 'gitlab', 'jira', or 'auto'."
        )

    return options


def capture_preserved_metadata(target_dir: Path) -> PreservedMetadata:
    """
    Scans target_dir before staging to capture existing downstream assets:
    - .pipeline/project_metadata.json
    - .pipeline/profile_config.json
    - .pipeline/schema.sysml
    - .pipeline/schema-digest.json
    - .pipeline/lineage.json
    - .gitignore (parsed into line entries)
    - schema/* files (customer models)
    """
    preserved = PreservedMetadata()
    if not target_dir.exists():
        return preserved

    pipeline_dir = target_dir / ".pipeline"
    if (pipeline_dir / "project_metadata.json").is_file():
        try:
            preserved.project_metadata_json = (pipeline_dir / "project_metadata.json").read_text(encoding="utf-8")
        except Exception:
            pass

    if (pipeline_dir / "profile_config.json").is_file():
        try:
            preserved.profile_config_json = (pipeline_dir / "profile_config.json").read_text(encoding="utf-8")
        except Exception:
            pass

    if (pipeline_dir / "schema.sysml").is_file():
        try:
            preserved.schema_sysml = (pipeline_dir / "schema.sysml").read_text(encoding="utf-8")
        except Exception:
            pass

    if (pipeline_dir / "schema-digest.json").is_file():
        try:
            preserved.schema_digest_json = (pipeline_dir / "schema-digest.json").read_text(encoding="utf-8")
        except Exception:
            pass

    if (pipeline_dir / "lineage.json").is_file():
        try:
            preserved.lineage_json = (pipeline_dir / "lineage.json").read_text(encoding="utf-8")
        except Exception:
            pass

    gitignore_path = target_dir / ".gitignore"
    if gitignore_path.is_file():
        try:
            with open(gitignore_path, "r", encoding="utf-8") as f:
                preserved.gitignore_lines = [line.strip() for line in f.readlines() if line.strip()]
        except Exception:
            pass

    schema_dir = target_dir / "schema"
    if schema_dir.is_dir():
        for p in schema_dir.rglob("*"):
            if p.is_file():
                try:
                    rel_p = str(p.relative_to(schema_dir))
                    preserved.existing_schema_files.append((rel_p, p.read_bytes()))
                except Exception:
                    pass

    return preserved


def restore_preserved_metadata(
    staging_dir: Path,
    preserved: PreservedMetadata,
) -> None:
    """
    Restores captured metadata directly into the staging directory prior to promotion.
    Guarantees 100% data fidelity without shell string truncation.
    """
    pipeline_dir = staging_dir / ".pipeline"
    pipeline_dir.mkdir(parents=True, exist_ok=True)

    if preserved.project_metadata_json is not None:
        (pipeline_dir / "project_metadata.json").write_text(preserved.project_metadata_json, encoding="utf-8")

    if preserved.profile_config_json is not None:
        (pipeline_dir / "profile_config.json").write_text(preserved.profile_config_json, encoding="utf-8")

    if preserved.schema_sysml is not None:
        (pipeline_dir / "schema.sysml").write_text(preserved.schema_sysml, encoding="utf-8")

    if preserved.schema_digest_json is not None:
        (pipeline_dir / "schema-digest.json").write_text(preserved.schema_digest_json, encoding="utf-8")

    if preserved.lineage_json is not None:
        (pipeline_dir / "lineage.json").write_text(preserved.lineage_json, encoding="utf-8")

    if preserved.existing_schema_files:
        schema_dir = staging_dir / "schema"
        schema_dir.mkdir(parents=True, exist_ok=True)
        for rel_path, data in preserved.existing_schema_files:
            dest = schema_dir / rel_path
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(data)


def discover_project_metadata(
    target_dir: Path,
    options: InstallOptions,
    remote_info: Optional[GitRemoteInfo],
) -> ProjectMetadataInfo:
    """
    Executes the 8-stage metadata discovery cascade for README generation:
    0. CLI --domain-name parameter
    1. Existing README.md heading title (cleaned of duplicate suffixes)
    2. .pipeline/project_metadata.json (project_name, description, technology_profile, regulatory_frameworks)
    3. codebase_rules.json (project_name)
    4. pubspec.yaml (Flutter / Dart profile)
    5. package.json (React / TypeScript profile)
    6. pyproject.toml (Python profile)
    7. .pipeline/profile_config.json, CMakeLists.txt, or package.xml (ROS2 C++ profile)
    8. Fallbacks to directory base name and generic descriptions.
    """
    project_name = ""
    project_desc = ""
    tech_profile = ""
    regulatory = ""

    # 0. Check explicit CLI --domain-name parameter
    if options.domain_name:
        project_name = options.domain_name

    # 1. Check existing README.md before overwriting
    readme_path = target_dir / "README.md"
    if not project_name and readme_path.is_file():
        try:
            with open(readme_path, "r", encoding="utf-8") as f:
                for line in f:
                    line = line.strip()
                    if line.startswith("# "):
                        title = line[2:].strip()
                        clean_cand = re.sub(
                            r"( -- Downstream (Systems Engineering|Cyber-Physical Infrastructure Safety|Safety-Critical Engineering) Project)+$",
                            "",
                            title,
                        ).strip()
                        if clean_cand and not re.search(
                            r"Getting started with GitLab|Downstream Systems Engineering Project|Downstream Cyber-Physical Infrastructure Safety Project|Downstream Safety-Critical Engineering Project",
                            clean_cand,
                        ):
                            project_name = clean_cand
                        break
        except Exception:
            pass

    # 2. Check .pipeline/project_metadata.json if present
    proj_meta_path = target_dir / ".pipeline" / "project_metadata.json"
    if proj_meta_path.is_file():
        try:
            with open(proj_meta_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                if not project_name:
                    project_name = data.get("project_name") or data.get("name") or ""
                if not project_desc and data.get("description"):
                    project_desc = data.get("description")
                if not tech_profile and (data.get("technology_profile") or data.get("profile")):
                    tech_profile = data.get("technology_profile") or data.get("profile")
                if not regulatory and data.get("regulatory_frameworks"):
                    regulatory = data.get("regulatory_frameworks")
        except Exception:
            pass

    # 3. Check codebase_rules.json if present
    cb_rules_path = target_dir / "codebase_rules.json"
    if not project_name and cb_rules_path.is_file():
        try:
            with open(cb_rules_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                project_name = data.get("project_name") or data.get("name") or ""
        except Exception:
            pass

    # 4. Check pubspec.yaml (Flutter / Dart)
    pubspec_path = target_dir / "pubspec.yaml"
    if pubspec_path.is_file():
        try:
            with open(pubspec_path, "r", encoding="utf-8") as f:
                for line in f:
                    if not project_name and re.match(r"^name:\s*", line):
                        project_name = re.sub(r"^name:\s*", "", line).strip()
                    if not project_desc and re.match(r"^description:\s*", line):
                        project_desc = re.sub(r"^description:\s*", "", line).strip()
            if not tech_profile:
                tech_profile = "`Flutter / Dart Embedded & Operator Console Profile`"
        except Exception:
            pass

    # 5. Check package.json (React / TypeScript Web)
    pkg_json_path = target_dir / "package.json"
    if pkg_json_path.is_file():
        try:
            with open(pkg_json_path, "r", encoding="utf-8") as f:
                data = json.load(f)
                if not project_name and data.get("name"):
                    project_name = data.get("name")
                if not project_desc and data.get("description"):
                    project_desc = data.get("description")
            if not tech_profile:
                tech_profile = "`React / TypeScript Web Operator Profile`"
        except Exception:
            pass

    # 6. Check pyproject.toml
    pyproject_path = target_dir / "pyproject.toml"
    if pyproject_path.is_file():
        try:
            with open(pyproject_path, "r", encoding="utf-8") as f:
                for line in f:
                    m = re.match(r'^name\s*=\s*["\']([^"\']+)["\']', line.strip())
                    if m:
                        pname = m.group(1).strip()
                        if pname != "deap01-spec-core" and not project_name:
                            project_name = pname
                        break
        except Exception:
            pass

    # 7. Check active platform profile configuration
    if not tech_profile:
        profile_cfg = target_dir / ".pipeline" / "profile_config.json"
        if profile_cfg.is_file():
            try:
                with open(profile_cfg, "r", encoding="utf-8") as f:
                    active_prof = json.load(f).get("active_profile", "")
                    if active_prof:
                        tech_profile = f"`{active_prof}`"
            except Exception:
                pass
        elif (target_dir / "CMakeLists.txt").is_file() or (target_dir / "package.xml").is_file():
            tech_profile = "`ROS2 C++ Real-Time` | `Target Embedded Platform Execution Profile`"

    # Fallbacks if still empty
    if not project_name:
        try:
            dir_base = target_dir.resolve().name
        except Exception:
            dir_base = target_dir.name
        if dir_base and dir_base not in (".", "/"):
            project_name = dir_base
        else:
            project_name = "Downstream Systems Engineering Project"

    if not project_desc:
        project_desc = (
            "This repository is an installed downstream implementation workspace governed by the "
            "**Digital Engineering Agent Platform (DEAP)** for cyber-physical infrastructure safety, "
            "real-time control, run-time assurance (RTA), and autonomous operations."
        )

    if not tech_profile:
        tech_profile = "`Target Platform Execution Profile`"

    if not regulatory:
        regulatory = "`Schema-Derived Regulatory Standards`"

    # Strip redundant trailing suffixes
    project_name = re.sub(
        r"( -- Downstream (Systems Engineering|Cyber-Physical Infrastructure Safety|Safety-Critical Engineering) Project)+$",
        "",
        project_name,
    ).strip()

    return ProjectMetadataInfo(
        project_name=project_name,
        project_desc=project_desc,
        tech_profile=tech_profile,
        regulatory_frameworks=regulatory,
    )


def resolve_domain_remote_url(
    installer_root: Path,
    target_dir: Path,
    options: InstallOptions,
    remote_info: Optional[GitRemoteInfo],
    meta: ProjectMetadataInfo,
) -> str:
    """
    Resolves authoritative upstream domain remote URL for README onboarding commands:
    1. Explicit --domain-url CLI argument.
    2. Target repository git remote URL (if not DEAP01-spec-core).
    3. Detected server + namespace + project from remote.
    4. Detected server + namespace + project_name slug.
    5. Installer root git remote URL (if not upstream spec core compiler).
    6. Fallback: https://github.com/{github_org}/{clean_name}.git
    """
    domain_remote_url = ""

    if options.domain_url:
        domain_remote_url = options.domain_url
    elif remote_info and "DEAP01-spec-core" not in remote_info.raw_url:
        domain_remote_url = remote_info.raw_url
    elif (
        remote_info
        and remote_info.server_url
        and remote_info.namespace
        and remote_info.project
        and remote_info.project != "DEAP01-spec-core"
    ):
        domain_remote_url = f"{remote_info.server_url}/{remote_info.namespace}/{remote_info.project}.git"
    elif (
        remote_info
        and remote_info.server_url
        and remote_info.namespace
        and meta.project_name
        and meta.project_name
        not in (
            "Downstream Systems Engineering Project",
            "Downstream Cyber-Physical Infrastructure Safety Project",
        )
    ):
        clean_name = meta.project_name.replace(" ", "-")
        domain_remote_url = f"{remote_info.server_url}/{remote_info.namespace}/{clean_name}.git"
    elif not (installer_root / ".pipeline" / "upstream").exists():
        installer_remote = get_git_remote_url(installer_root)
        if installer_remote and "DEAP01-spec-core" not in installer_remote:
            domain_remote_url = installer_remote

    if not domain_remote_url:
        clean_name = (meta.project_name or "downstream-project").replace(" ", "-")
        org = options.github_org or "gintatkinson"
        domain_remote_url = f"https://github.com/{org}/{clean_name}.git"

    return domain_remote_url
