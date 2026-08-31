#!/usr/bin/env python3
"""
Prune stale project entries and clean dead temporary directories.

Automated maintenance utility that:
1. Reads ~/.gemini/projects.json and prunes non-existent path entries.
2. Cleans ~/.gemini/tmp/ and ~/.gemini/history/ of dead temporary clone directories.
3. Cleans .DS_Store files from workspace directories.
"""

import argparse
import json
import os
import shutil
import sys

def prune_projects_json(projects_file=None, dry_run=False):
    """Prune non-existent path entries from projects.json."""
    if projects_file is None:
        projects_file = os.path.expanduser("~/.gemini/projects.json")

    if not os.path.exists(projects_file):
        print(f"Projects file '{projects_file}' does not exist — skipping pruning.")
        return [], []

    try:
        with open(projects_file, "r", encoding="utf-8") as f:
            data = json.load(f)
    except Exception as e:
        print(f"WARNING: Could not parse projects file '{projects_file}': {e}", file=sys.stderr)
        return [], []

    pruned = []
    kept = []

    if isinstance(data, list):
        new_list = []
        for item in data:
            path = None
            if isinstance(item, str):
                path = item
            elif isinstance(item, dict):
                path = item.get("path") or item.get("directory") or item.get("uri")

            if path and not os.path.exists(path):
                pruned.append(item)
            else:
                new_list.append(item)
                if path:
                    kept.append(path)
        data_to_write = new_list
    elif isinstance(data, dict):
        new_dict = {}
        for key, val in data.items():
            path = key if os.path.isabs(key) else None
            if isinstance(val, str) and os.path.isabs(val):
                path = val
            elif isinstance(val, dict):
                path = val.get("path") or val.get("directory") or val.get("uri") or path

            if path and not os.path.exists(path):
                pruned.append({key: val})
            else:
                new_dict[key] = val
                if path:
                    kept.append(path)
        data_to_write = new_dict
    else:
        print(f"WARNING: Unexpected structure in '{projects_file}' — skipping pruning.", file=sys.stderr)
        return [], []

    if pruned and not dry_run:
        try:
            with open(projects_file, "w", encoding="utf-8") as f:
                json.dump(data_to_write, f, indent=2)
            print(f"Pruned {len(pruned)} stale entry/entries from '{projects_file}'.")
        except Exception as e:
            print(f"ERROR: Failed to write updated '{projects_file}': {e}", file=sys.stderr)
    elif pruned and dry_run:
        print(f"[Dry Run] Would prune {len(pruned)} stale entry/entries from '{projects_file}'.")
    else:
        print(f"No stale entries found in '{projects_file}'.")

    return pruned, kept

def clean_dead_temp_dirs(tmp_dir=None, history_dir=None, active_paths=None, dry_run=False):
    """Clean dead temporary clone directories in tmp and history directories."""
    if tmp_dir is None:
        tmp_dir = os.path.expanduser("~/.gemini/tmp")
    if history_dir is None:
        history_dir = os.path.expanduser("~/.gemini/history")

    active_paths_set = set(os.path.abspath(p) for p in (active_paths or []))
    cleaned = []

    target_dirs = [tmp_dir, history_dir]
    for d in target_dirs:
        if not os.path.exists(d) or not os.path.isdir(d):
            continue

        for entry in os.listdir(d):
            full_path = os.path.join(d, entry)
            # If entry is a broken symlink or directory
            if os.path.islink(full_path) and not os.path.exists(full_path):
                cleaned.append(full_path)
                if not dry_run:
                    try:
                        os.unlink(full_path)
                    except OSError as e:
                        print(f"WARNING: Could not remove dead symlink '{full_path}': {e}", file=sys.stderr)
                else:
                    print(f"[Dry Run] Would remove dead symlink '{full_path}'")
            elif os.path.isdir(full_path):
                abs_entry = os.path.abspath(full_path)
                # Check if it's a dead temp clone dir or not in active paths
                if abs_entry not in active_paths_set:
                    cleaned.append(full_path)
                    if not dry_run:
                        try:
                            shutil.rmtree(full_path, ignore_errors=True)
                        except OSError as e:
                            print(f"WARNING: Could not remove dead directory '{full_path}': {e}", file=sys.stderr)
                    else:
                        print(f"[Dry Run] Would remove dead directory '{full_path}'")

    print(f"Cleaned {len(cleaned)} dead temporary directory/directories.")
    return cleaned

def clean_ds_store_files(workspace_dir=None, dry_run=False):
    """Clean .DS_Store files from workspace directory."""
    if workspace_dir is None:
        workspace_dir = os.getcwd()

    workspace_dir = os.path.abspath(workspace_dir)
    if not os.path.exists(workspace_dir):
        print(f"Workspace directory '{workspace_dir}' does not exist — skipping .DS_Store cleanup.")
        return []

    removed = []
    for root, _, files in os.walk(workspace_dir):
        for f in files:
            if f == ".DS_Store":
                ds_path = os.path.join(root, f)
                removed.append(ds_path)
                if not dry_run:
                    try:
                        os.remove(ds_path)
                    except OSError as e:
                        print(f"WARNING: Could not remove '{ds_path}': {e}", file=sys.stderr)
                else:
                    print(f"[Dry Run] Would remove '{ds_path}'")

    print(f"Cleaned {len(removed)} .DS_Store file(s) from workspace '{workspace_dir}'.")
    return removed

def main():
    parser = argparse.ArgumentParser(description="Prune stale projects and clean dead temporary clone directories.")
    parser.add_argument("--projects-file", help="Path to projects.json file", default=None)
    parser.add_argument("--tmp-dir", help="Path to tmp directory", default=None)
    parser.add_argument("--history-dir", help="Path to history directory", default=None)
    parser.add_argument("--workspace", help="Path to workspace directory for .DS_Store cleanup", default=None)
    parser.add_argument("--dry-run", action="store_true", help="Perform a dry run without modifying disk")
    args = parser.parse_args()

    pruned, kept = prune_projects_json(projects_file=args.projects_file, dry_run=args.dry_run)
    clean_dead_temp_dirs(tmp_dir=args.tmp_dir, history_dir=args.history_dir, active_paths=kept, dry_run=args.dry_run)
    clean_ds_store_files(workspace_dir=args.workspace, dry_run=args.dry_run)

    sys.exit(0)

if __name__ == "__main__":
    main()
