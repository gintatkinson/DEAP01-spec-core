#!/usr/bin/env python3
"""
Transcript Ingestion Verification Gate (Mechanism 3 & 4)

Parses subagent transcript.jsonl log file to verify `view_file` was called on explicit target input file paths.
Asserts zero `...` truncation or summarization tokens were emitted during input ingestion.
Returns exit 1 if view_file ingestion proof is missing or if summarization tokens are detected.

Usage:
    python3 scripts/verify_input_ingestion.py [--transcript <log_path>] [--input-digest <digest_json>] [--target-files file1 file2 ...]
"""

import argparse
import json
import os
import sys


TRUNCATION_TOKENS = [
    "...",
    "<truncated",
    "[truncated]",
    "summarized",
    "truncated content",
]


def normalize_path(path_str):
    if not path_str:
        return ""
    return os.path.normpath(os.path.abspath(path_str))


def check_ingestion(transcript_path, digest_path=None, target_files=None):
    targets = set()

    if target_files:
        for t in target_files:
            targets.add(t)
            targets.add(normalize_path(t))
            targets.add(os.path.basename(t))

    if digest_path and os.path.exists(digest_path):
        try:
            with open(digest_path, 'r', encoding='utf-8') as df:
                digest_data = json.load(df)
                files_map = digest_data.get('files', {})
                for fpath in files_map.keys():
                    targets.add(fpath)
                    targets.add(normalize_path(fpath))
                    targets.add(os.path.basename(fpath))
        except Exception as e:
            print(f"Warning: Could not read digest file {digest_path}: {e}")

    if not targets:
        print("No target files specified or found in digest for verification.")
        return True, "No target files to verify."

    if not os.path.exists(transcript_path):
        print(f"ERROR: Transcript file not found at {transcript_path}")
        return False, f"Transcript file not found at {transcript_path}"

    viewed_paths = set()
    truncation_found = False
    truncation_details = []

    with open(transcript_path, 'r', encoding='utf-8', errors='replace') as tf:
        for line in tf:
            line_str = line.strip()
            if not line_str:
                continue
            try:
                record = json.loads(line_str)
            except json.JSONDecodeError:
                continue

            # Check tool_calls in PLANNER_RESPONSE or MODEL steps
            tool_calls = record.get('tool_calls', [])
            for call in tool_calls:
                name = call.get('name') or call.get('function', {}).get('name')
                if name == 'view_file':
                    args = call.get('args') or call.get('arguments') or {}
                    if isinstance(args, str):
                        try:
                            args = json.loads(args)
                        except Exception:
                            args = {}
                    path_val = args.get('AbsolutePath') or args.get('path') or args.get('file') or args.get('TargetFile')
                    if path_val:
                        viewed_paths.add(path_val)
                        viewed_paths.add(normalize_path(path_val))
                        viewed_paths.add(os.path.basename(path_val))

            # Check for truncation / summarization tokens in TOOL_RESPONSE or step content
            content = record.get('content', '')
            if isinstance(content, str) and content:
                # Check for explicit truncation tokens in tool response or output
                for token in TRUNCATION_TOKENS:
                    if token in content:
                        # Ensure token is an actual truncation marker rather than standard code doc
                        if token == "...":
                            # Check if ... occurs as standalone line or truncated indicator
                            lines = content.splitlines()
                            for l in lines:
                                if l.strip() == "..." or l.strip().startswith("... (truncated") or "... truncated" in l:
                                    truncation_found = True
                                    truncation_details.append(f"Detected standalone truncation token '{token}' in transcript step {record.get('step_index')}")
                        else:
                            truncation_found = True
                            truncation_details.append(f"Detected truncation token '{token}' in transcript step {record.get('step_index')}")

    # Check that all target files were called with view_file
    missing_targets = []
    if target_files:
        for t in target_files:
            norm_t = normalize_path(t)
            base_t = os.path.basename(t)
            if t not in viewed_paths and norm_t not in viewed_paths and base_t not in viewed_paths:
                missing_targets.append(t)
    elif digest_path and os.path.exists(digest_path):
        with open(digest_path, 'r', encoding='utf-8') as df:
            digest_data = json.load(df)
            for fpath in digest_data.get('files', {}).keys():
                norm_f = normalize_path(fpath)
                base_f = os.path.basename(fpath)
                if fpath not in viewed_paths and norm_f not in viewed_paths and base_f not in viewed_paths:
                    missing_targets.append(fpath)

    if missing_targets:
        err_msg = f"ERROR: Missing view_file ingestion proof for target files: {missing_targets}"
        print(err_msg)
        return False, err_msg

    if truncation_found:
        err_msg = f"ERROR: Truncation or summarization tokens detected during input ingestion: {truncation_details}"
        print(err_msg)
        return False, err_msg

    print("Verification PASSED: view_file ingestion proof confirmed for all targets with 0 truncation tokens.")
    return True, "PASSED"


def main():
    parser = argparse.ArgumentParser(description="Verify view_file input ingestion proof in subagent transcript.")
    parser.add_argument("--transcript", required=True, help="Path to transcript.jsonl file")
    parser.add_argument("--input-digest", help="Path to input-digest.json file")
    parser.add_argument("--target-files", nargs="*", help="List of target input file paths")
    args = parser.parse_args()

    success, msg = check_ingestion(args.transcript, args.input_digest, args.target_files)
    if not success:
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
