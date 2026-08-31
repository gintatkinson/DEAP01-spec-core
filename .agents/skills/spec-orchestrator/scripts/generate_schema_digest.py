#!/usr/bin/env python3
"""
Pre-Dispatch Schema Ingestion Gate (Mechanism 1)

Parses input specification schemas (YANG, etc.), computes SHA-256 digests,
line counts, and node counts across containers, lists, leaves, typedefs,
identities, and groupings. Writes output to .pipeline/schema-digest.json.

Usage:
    python3 generate_schema_digest.py [--input <file_or_dir>] [--output <path>]
"""

import argparse
import hashlib
import json
import os
import re
import sys


def parse_schema_file(file_path):
    """
    Parses a single schema file and extracts SHA-256, total lines, node counts, and schema nodes.
    """
    with open(file_path, 'rb') as f:
        content_bytes = f.read()

    sha256_hash = hashlib.sha256(content_bytes).hexdigest()
    text = content_bytes.decode('utf-8', errors='replace')
    lines = text.splitlines()
    total_lines = len(lines)

    node_counts = {
        'containers': 0,
        'lists': 0,
        'leaves': 0,
        'typedefs': 0,
        'identities': 0,
        'groupings': 0
    }
    schema_nodes = []

    # Regex patterns for YANG schema node declarations
    container_pattern = re.compile(r'^\s*container\s+([a-zA-Z0-9_\-]+)\s*\{', re.MULTILINE)
    list_pattern = re.compile(r'^\s*list\s+([a-zA-Z0-9_\-]+)\s*\{', re.MULTILINE)
    leaf_pattern = re.compile(r'^\s*(?:leaf|leaf-list|anyxml)\s+([a-zA-Z0-9_\-]+)\s*[\{;]', re.MULTILINE)
    typedef_pattern = re.compile(r'^\s*typedef\s+([a-zA-Z0-9_\-]+)\s*[\{;]', re.MULTILINE)
    identity_pattern = re.compile(r'^\s*identity\s+([a-zA-Z0-9_\-]+)\s*[\{;]', re.MULTILINE)
    grouping_pattern = re.compile(r'^\s*grouping\s+([a-zA-Z0-9_\-]+)\s*[\{;]', re.MULTILINE)

    for match in container_pattern.finditer(text):
        node_counts['containers'] += 1
        schema_nodes.append(match.group(1))

    for match in list_pattern.finditer(text):
        node_counts['lists'] += 1
        schema_nodes.append(match.group(1))

    for match in leaf_pattern.finditer(text):
        node_counts['leaves'] += 1
        schema_nodes.append(match.group(1))

    for match in typedef_pattern.finditer(text):
        node_counts['typedefs'] += 1
        schema_nodes.append(match.group(1))

    for match in identity_pattern.finditer(text):
        node_counts['identities'] += 1
        schema_nodes.append(match.group(1))

    for match in grouping_pattern.finditer(text):
        node_counts['groupings'] += 1
        schema_nodes.append(match.group(1))

    return {
        'sha256': sha256_hash,
        'total_lines': total_lines,
        'node_counts': node_counts,
        'schema_nodes': sorted(list(set(schema_nodes))),
        'content_bytes': content_bytes,
    }


def generate_digest(input_path, output_path):
    schema_files = []
    if os.path.isfile(input_path):
        schema_files.append(input_path)
    elif os.path.isdir(input_path):
        for root, _, files in os.walk(input_path):
            for file in sorted(files):
                if file.endswith(('.yang', '.json', '.proto', '.sysml')):
                    schema_files.append(os.path.join(root, file))

    if not schema_files:
        # Fallback empty structure
        digest_data = {
            'sha256': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
            'total_lines': 0,
            'node_counts': {
                'containers': 0,
                'lists': 0,
                'leaves': 0,
                'typedefs': 0,
                'identities': 0,
                'groupings': 0
            },
            'schema_nodes': []
        }
    else:
        combined_bytes = bytearray()
        total_lines = 0
        total_counts = {
            'containers': 0,
            'lists': 0,
            'leaves': 0,
            'typedefs': 0,
            'identities': 0,
            'groupings': 0
        }
        all_nodes = set()

        for sf in schema_files:
            parsed = parse_schema_file(sf)
            combined_bytes.extend(parsed['content_bytes'])
            total_lines += parsed['total_lines']
            for key in total_counts:
                total_counts[key] += parsed['node_counts'][key]
            all_nodes.update(parsed['schema_nodes'])

        overall_sha256 = hashlib.sha256(combined_bytes).hexdigest()
        digest_data = {
            'sha256': overall_sha256,
            'total_lines': total_lines,
            'node_counts': total_counts,
            'schema_nodes': sorted(list(all_nodes))
        }

    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(digest_data, f, indent=2)

    print(f"Successfully generated schema digest at {output_path}")
    return digest_data


def main():
    parser = argparse.ArgumentParser(description="Generate schema-digest.json for Mechanism 1")
    parser.add_argument("--input", default="schema", help="Path to schema file or directory")
    parser.add_argument("--output", default=".pipeline/schema-digest.json", help="Path to output JSON")
    args = parser.parse_args()

    generate_digest(args.input, args.output)


if __name__ == "__main__":
    main()
