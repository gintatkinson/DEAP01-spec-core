"""Enforces markdown link integrity across all specifications and repository documentation."""
import os
import re
from typing import List

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

# Match markdown links: [text](link) ending in .md or with an anchor .md#anchor
_LINK_RE = re.compile(r'\[[^\]]+\]\(([^)]+)\)')
_GITHUB_BLOB_RE = re.compile(r'https://github\.com/[^\s/]+/[^\s/]+/blob/[^\s/]+/[^\s\)\]\'">]+')

class LinkValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        workspace_dir = repo.workspace_dir
        errors: List[Finding] = []

        repo_name = os.path.basename(os.path.abspath(workspace_dir))

        search_dirs = [
            os.path.join(workspace_dir, "docs"),
            os.path.join(workspace_dir, "rules"),
            os.path.join(workspace_dir, "skills"),
            os.path.join(workspace_dir, ".pipeline"),
        ]

        markdown_files = []
        # Add root markdown files
        for item in os.listdir(workspace_dir):
            if item.endswith(".md") and not item.startswith("."):
                markdown_files.append(os.path.join(workspace_dir, item))

        # Walk all documentation directories
        for root_dir in search_dirs:
            if not os.path.isdir(root_dir):
                continue
            for dirpath, _, filenames in os.walk(root_dir):
                # Skip historical point-in-time audit snapshots
                if "docs/audits" in dirpath or "docs/decisions" in dirpath or "docs/designs" in dirpath:
                    continue
                for filename in filenames:
                    if filename.endswith(".md") and not filename.startswith("."):
                        markdown_files.append(os.path.join(dirpath, filename))

        # Also add configured backlog dirs if outside docs/
        rules = repo.get_codebase_rules()
        backlog_dirs = getattr(rules, "backlog_directories", None)
        if backlog_dirs:
            for dir_key in ["features", "epics", "user_stories", "use_cases"]:
                rel = getattr(backlog_dirs, dir_key, None)
                if rel:
                    target = os.path.join(workspace_dir, rel)
                    if os.path.isdir(target):
                        for f in os.listdir(target):
                            if f.endswith(".md") and not f.startswith("."):
                                full_p = os.path.join(target, f)
                                if full_p not in markdown_files:
                                    markdown_files.append(full_p)

        for filepath in markdown_files:
            rel_path = os.path.relpath(filepath, workspace_dir)
            source_dir = os.path.dirname(filepath)

            try:
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception:
                continue

            links_to_check = []
            for match in _LINK_RE.finditer(content):
                links_to_check.append(match.group(1).strip())

            for match in _GITHUB_BLOB_RE.finditer(content):
                m_url = match.group(0).strip()
                if m_url not in links_to_check:
                    links_to_check.append(m_url)

            for link_raw in links_to_check:
                # Prohibit non-portable file:/// URI scheme
                if link_raw.startswith("file://") or link_raw.startswith("file:/"):
                    errors.append(Finding(
                        "markdown-forbidden-file-protocol-link",
                        f"{rel_path}: Absolute file URI scheme is forbidden: '{link_raw}'. Links must use repository-relative paths.",
                        location=rel_path
                    ))
                    continue

                # Skip template placeholders / examples
                is_placeholder = any(placeholder in link_raw for placeholder in [
                    "-XX-", "XX-name", "link-to-", "URL", "target", "example.com", "file.sysml",
                    "docs/features/feat-", "docs/epics/epic-", "docs/user-stories/us-", "docs/use-cases/uc-",
                    "EPIC-001.md", "system.sysml", "schema/..."
                ]) or bool(re.search(r'(?:^|[/\\])(?:SystemModel|[A-Za-z0-9_]*[Ee]xample|[A-Za-z0-9_]*[Tt]emplate|[A-Za-z0-9_]*[Pp]laceholder)\.sysml', link_raw))

                if is_placeholder:
                    if not os.path.exists(os.path.join(workspace_dir, link_raw)):
                        continue

                link_target = link_raw.split("#")[0].strip()

                if not link_target:
                    continue  # In-page anchor like `#section`

                # Check if it's an external GitHub/GitLab blob URL for a different repository
                if ("github.com/" in link_raw or "gitlab.com/" in link_raw) and repo_name not in link_raw:
                    continue

                is_blob = False
                if "blob/" in link_target and repo_name in link_target:
                    parts = link_target.split("blob/")
                    if len(parts) > 1:
                        branch_and_path = parts[1]
                        path_parts = branch_and_path.split("/", 1)
                        if len(path_parts) > 1:
                            link_target = path_parts[1]
                            is_blob = True
                elif "tree/" in link_target and repo_name in link_target:
                    parts = link_target.split("tree/")
                    if len(parts) > 1:
                        branch_and_path = parts[1]
                        path_parts = branch_and_path.split("/", 1)
                        if len(path_parts) > 1:
                            link_target = path_parts[1]
                            is_blob = True

                if link_target.startswith("http://") or link_target.startswith("https://") or link_target.startswith("mailto:"):
                    continue

                if link_target.startswith("/"):
                    resolved_path = os.path.join(workspace_dir, link_target.lstrip("/"))
                elif is_blob:
                    resolved_path = os.path.join(workspace_dir, link_target)
                else:
                    # Check relative to source_dir first, then relative to workspace_dir,
                    # and scan schema / pipeline directories for matching schema files
                    rel_to_source = os.path.normpath(os.path.join(source_dir, link_target))
                    rel_to_root = os.path.normpath(os.path.join(workspace_dir, link_target))
                    schema_path = os.path.normpath(os.path.join(workspace_dir, "schema", os.path.basename(link_target)))
                    pipeline_path = os.path.normpath(os.path.join(workspace_dir, ".pipeline", os.path.basename(link_target)))

                    if os.path.exists(rel_to_source):
                        resolved_path = rel_to_source
                    elif os.path.exists(rel_to_root):
                        resolved_path = rel_to_root
                    elif os.path.exists(schema_path):
                        resolved_path = schema_path
                    elif os.path.exists(pipeline_path):
                        resolved_path = pipeline_path
                    else:
                        resolved_path = rel_to_source

                if not os.path.exists(resolved_path):
                    errors.append(Finding(
                        "markdown-broken-link-reference",
                        f"{rel_path}: Broken markdown link points to non-existent target '{link_raw}'.",
                        location=rel_path
                    ))

        return errors
