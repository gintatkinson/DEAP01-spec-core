#!/usr/bin/env python3
"""
Pre-Ingestion Digest Generator (Mechanism 1 for Input Validation)

Pre-computes .pipeline/input-digest.json containing SHA-256 digests, total line counts,
line-range bounds, and required structural section markers for input files.

Usage:
    python3 generate_input_digest.py [--input <file_or_dir>] [--output <path>]
"""

import argparse
import hashlib
import json
import os
import re
import sys


def parse_input_file(file_path):
    """
    Parses a single input file and extracts SHA-256, total lines, line range bounds,
    and structural section markers.
    """
    with open(file_path, 'rb') as f:
        content_bytes = f.read()

    sha256_hash = hashlib.sha256(content_bytes).hexdigest()
    text = content_bytes.decode('utf-8', errors='replace')
    lines = text.splitlines()
    total_lines = len(lines)

    structural_section_markers = []

    # Regex patterns for structural markers in Markdown, YANG, SysML, etc.
    md_header_pattern = re.compile(r'^\s*(#{1,6}\s+.*)', re.MULTILINE)
    yang_node_pattern = re.compile(r'^\s*(container|list|typedef|identity|grouping|module)\s+([a-zA-Z0-9_\-]+)', re.MULTILINE)

    for match in md_header_pattern.finditer(text):
        marker = match.group(1).strip()
        if marker not in structural_section_markers:
            structural_section_markers.append(marker)

    for match in yang_node_pattern.finditer(text):
        marker = f"{match.group(1)} {match.group(2)}"
        if marker not in structural_section_markers:
            structural_section_markers.append(marker)

    return {
        'sha256': sha256_hash,
        'total_lines': total_lines,
        'line_range': [1, total_lines] if total_lines > 0 else [0, 0],
        'line_range_bounds': [1, total_lines] if total_lines > 0 else [0, 0],
        'structural_section_markers': structural_section_markers,
        'content_bytes': content_bytes,
    }


def generate_digest(input_path, output_path):
    input_files = []
    if os.path.isfile(input_path):
        input_files.append(input_path)
    elif os.path.isdir(input_path):
        for root, _, files in os.walk(input_path):
            for file in sorted(files):
                if not file.startswith('.') and file.endswith(('.md', '.yang', '.sysml', '.proto', '.json', '.yaml', '.yml', '.txt')):
                    input_files.append(os.path.join(root, file))

    if not input_files:
        digest_data = {
            'sha256': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
            'total_lines': 0,
            'files': {},
            'structural_section_markers': []
        }
    else:
        combined_bytes = bytearray()
        total_lines = 0
        file_map = {}
        all_markers = []

        for fpath in input_files:
            parsed = parse_input_file(fpath)
            combined_bytes.extend(parsed['content_bytes'])
            total_lines += parsed['total_lines']
            for m in parsed['structural_section_markers']:
                if m not in all_markers:
                    all_markers.append(m)

            file_map[fpath] = {
                'sha256': parsed['sha256'],
                'total_lines': parsed['total_lines'],
                'line_range': parsed['line_range'],
                'line_range_bounds': parsed['line_range_bounds'],
                'structural_section_markers': parsed['structural_section_markers']
            }

        overall_sha256 = hashlib.sha256(combined_bytes).hexdigest()
        digest_data = {
            'sha256': overall_sha256,
            'total_lines': total_lines,
            'files': file_map,
            'structural_section_markers': all_markers
        }

    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(digest_data, f, indent=2)

    print(f"Successfully generated input digest at {output_path}")
    return digest_data


def main():
    parser = argparse.ArgumentParser(description="Generate input-digest.json for Input Validation Architecture")
    parser.add_argument("--input", default="schema", help="Path to input file or directory")
    parser.add_argument("--output", default=".pipeline/input-digest.json", help="Path to output JSON")
    args = parser.parse_args()

    generate_digest(args.input, args.output)


if __name__ == "__main__":
    main()
