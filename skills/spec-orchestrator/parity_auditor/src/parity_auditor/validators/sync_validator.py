"""Compares the registered issue backlog against the local specification files.

Titles normalise through ``reconcile_backlog.py``'s own function, bound by reference in
``utils/spec_titles.py``. This module used to carry a private copy that had drifted from
it in two ways — it lacked the guard that keeps the original title when prefix-stripping
would empty it (so every prefix-only title, "Epic 2" and "Epic 3" alike, collapsed to one
key), and it folded ``_`` to a space (so two titles the reconciler keys apart looked
identical here). A gate that collides in a different space from the consumer it protects
reports collisions that do not exist and misses the ones that do; see
``utils/spec_titles.py`` for why the reconciler is the definition site.
"""

import os
import re
import subprocess
import json
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository
from ..utils.spec_titles import normalize_spec_title

class SyncValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
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
        tracker_rules = rules.tracker_rules
        backlog_dirs = rules.backlog_directories
        
        epics_dir = os.path.join(repo.workspace_dir, backlog_dirs.epics)
        features_dir = os.path.join(repo.workspace_dir, backlog_dirs.features)
        
        errors = []
        
        # 1. Fetch registered issues from GitHub
        cmd = tracker_rules.get("commands", {}).get("list_issues") if isinstance(tracker_rules, dict) else None
        if not cmd:
            print("Warning: Missing commands.list_issues in tracker_rules.")
            return []
            
        try:
            res = subprocess.run(cmd, capture_output=True, text=True, cwd=repo.workspace_dir, timeout=30)
            if res.returncode != 0:
                print(f"Warning: Failed to fetch issue backlog from remote: {res.stderr.strip()}")
                return []
            issues = json.loads(res.stdout)
        except Exception as e:
            print(f"Warning: Issue backlog offline or unavailable: {e}")
            return []
            
        # Parse tracker issues
        labels_config = tracker_rules.get("labels", {}) if isinstance(tracker_rules, dict) else {}
        epic_label = labels_config.get("epic", "epic").lower()
        feature_label = labels_config.get("feature", "feature").lower()
        
        # Keyed by (spec_type, normalized_title). The spec type comes from the issue
        # label and is part of the identity of a spec: an epic issue is not satisfied
        # by a same-titled feature file, and two issues that normalize to the same
        # title must not overwrite one another. Issue #303.
        tracker_specs = {}
        tracker_indices = {}

        for issue in issues:
            labels = []
            for l in issue.get("labels", []):
                if isinstance(l, dict):
                    labels.append(l.get("name", "").lower())
                elif isinstance(l, str):
                    labels.append(l.lower())
                    
            is_spec = False
            spec_type = None
            if epic_label in labels:
                is_spec = True
                spec_type = "epic"
            elif feature_label in labels:
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
                errors.append(Finding(
                    "tracker-issue-without-local-specification",
                    f"Missing specification file for registered Issue #{issue['number']} - '{issue['title']}'. Please check your branch baseline.",
                    location=spec_key[0],
                ))

        # 4. Check for index collisions
        for (std_type, idx), norm_title in local_indices.items():
            tracker_key = tracker_indices.get((std_type, idx))
            if tracker_key and tracker_key[1] != norm_title:
                issue_num = tracker_specs.get(tracker_key, {}).get("number", "unknown")
                errors.append(Finding(
                    "spec-index-collides-with-tracker-issue",
                    f"Index collision detected. Local specification with index {idx} ('{norm_title}') overlaps with registered Issue #{issue_num} ('{tracker_key[1]}').",
                    location=std_type,
                ))
                
        return errors
