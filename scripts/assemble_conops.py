#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Deterministic ConOps & Mission Intent Assembly Engine (ISO 29148 / NATO STANAG 4586 / OMG UAF).
Addresses Issues #113 and #114.

Compiles modular unit files from docs/conops/units/conops/ and docs/conops/units/mission_intent/
into canonical CONOPS.md and MISSION_INTENT.md specification documents with automated
TOC generation, cross-reference anchor validation, and zero-placeholder integrity gating.
"""

import argparse
import datetime
import os
import re
import sys
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple, Union


# Match template placeholders like {{SYSTEM_IDENTIFIER}}, {{OA_01_DESCRIPTION}}, etc.
PLACEHOLDER_PATTERN = re.compile(r"\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}")


def slugify(text: str) -> str:
    """
    Converts a heading string into a standard GitHub Markdown anchor slug.
    Example: "1. Scope & System Identification" -> "1-scope--system-identification"
    """
    # Remove math markers or backticks
    clean = re.sub(r"[`$*]", "", text).strip()
    # Replace symbols and punctuation except hyphens/underscores/spaces
    # GitHub markdown slug logic: lowercase, remove non-alphanumerics except hyphen and space, replace space with hyphen
    slug = ""
    for ch in clean.lower():
        if ch.isalnum() or ch in ("-", "_", " "):
            slug += ch
    slug = slug.replace(" ", "-")
    # Collapse multiple consecutive hyphens? GitHub leaves multiple hyphens for '&' -> '--'
    return slug


def extract_headings(content: str) -> List[Tuple[int, str, str]]:
    """
    Extracts markdown headings from content.
    Returns list of tuples: (level, title, slug).
    Ignores code blocks and math blocks.
    """
    headings: List[Tuple[int, str, str]] = []
    in_code_block = False
    in_math_block = False

    for line in content.splitlines():
        stripped = line.strip()
        if stripped.startswith("```"):
            in_code_block = not in_code_block
            continue
        if stripped.startswith("$$"):
            in_math_block = not in_math_block
            continue
        if in_code_block or in_math_block:
            continue

        m = re.match(r"^(#{1,6})\s+(.+)$", stripped)
        if m:
            level = len(m.group(1))
            title = m.group(2).strip()
            slug = slugify(title)
            headings.append((level, title, slug))

    return headings


def generate_table_of_contents(headings: List[Tuple[int, str, str]], max_depth: int = 3) -> str:
    """
    Generates a Markdown Table of Contents from a list of headings.
    Skips the top-level H1 title.
    """
    toc_lines: List[str] = ["## Table of Contents", ""]
    for level, title, slug in headings:
        if level == 1:
            continue
        if level > max_depth:
            continue
        indent = "  " * (level - 2)
        clean_title = re.sub(r"[`*]", "", title)
        toc_lines.append(f"{indent}- [{clean_title}](#{slug})")

    toc_lines.append("")
    return "\n".join(toc_lines)


def verify_markdown_links(content: str) -> List[str]:
    """
    Verifies internal anchor links (#slug) in the document against defined headings.
    Returns list of error messages for any broken anchor links.
    """
    headings = extract_headings(content)
    valid_slugs: Set[str] = {slug for _, _, slug in headings}
    errors: List[str] = []

    # Find markdown links: [text](#anchor)
    anchor_links = re.findall(r"\[([^\]]+)\]\(#([^\)]+)\)", content)
    for text, anchor in anchor_links:
        if anchor not in valid_slugs:
            errors.append(f"Broken anchor link: [{text}](#{anchor}) does not match any heading in document.")

    return errors


def validate_unit_integrity(unit_paths: List[str]) -> Tuple[bool, List[str]]:
    """
    Validates unit integrity:
    1. File is non-empty (stripped length > 0).
    2. Zero unresolved placeholder tokens matching {{...}}.
    Returns (is_valid, list_of_errors).
    """
    errors: List[str] = []

    for path in unit_paths:
        if not os.path.isfile(path):
            errors.append(f"Unit file not found: {path}")
            continue

        try:
            with open(path, "r", encoding="utf-8") as f:
                text = f.read()
        except Exception as e:
            errors.append(f"Failed to read unit file '{path}': {e}")
            continue

        if not text.strip():
            errors.append(f"Unit file '{path}' is empty.")
            continue

        # Check for placeholder tokens
        placeholders = PLACEHOLDER_PATTERN.findall(text)
        if placeholders:
            unique_tokens = sorted(list(set(placeholders)))
            errors.append(
                f"Unit file '{path}' contains {len(placeholders)} unresolved placeholder token(s): {', '.join(unique_tokens)}"
            )

    return (len(errors) == 0, errors)


def _extract_doc_metadata(units: List[Tuple[str, str]]) -> Dict[str, str]:
    """
    Extracts or infers document title, system name, version, and date from the unit contents.
    """
    metadata: Dict[str, str] = {
        "title": "Concept of Operations",
        "version": "1.0.0",
        "date": datetime.date.today().isoformat(),
        "system": "AutonomousCyberPhysicalSystem",
    }

    for _, content in units:
        # Check for metadata attribute table: | **Title** | ... |
        for line in content.splitlines():
            m_title = re.search(r"\|\s*\*\*Title\*\*\s*\|\s*([^|]+)\|", line, re.IGNORECASE)
            if m_title:
                metadata["title"] = m_title.group(1).strip()
            m_ver = re.search(r"\|\s*\*\*Version\*\*\s*\|\s*([^|]+)\|", line, re.IGNORECASE)
            if m_ver:
                metadata["version"] = m_ver.group(1).strip()
            m_date = re.search(r"\|\s*\*\*Date\*\*\s*\|\s*([^|]+)\|", line, re.IGNORECASE)
            if m_date:
                metadata["date"] = m_date.group(1).strip()

            # Check for H1 header if title not set
            if line.startswith("# ") and metadata["title"] == "Concept of Operations":
                metadata["title"] = line[2:].strip()

    return metadata


def assemble_document(
    units_dir: str,
    doc_title: Optional[str] = None,
    doc_version: Optional[str] = None,
    doc_date: Optional[str] = None,
) -> Tuple[str, List[str]]:
    """
    Compiles unit files located in units_dir into a single verified Markdown document.
    Returns (compiled_document_text, error_list).
    """
    errors: List[str] = []
    if not os.path.isdir(units_dir):
        return "", [f"Units directory '{units_dir}' does not exist."]

    # Find all .md files in units_dir sorted alphabetically/numerically
    filenames = sorted([f for f in os.listdir(units_dir) if f.endswith(".md")])
    if not filenames:
        return "", [f"No markdown unit files (*.md) found in '{units_dir}'."]

    unit_paths = [os.path.join(units_dir, f) for f in filenames]

    # Validate unit integrity
    is_valid, integrity_errors = validate_unit_integrity(unit_paths)
    if not is_valid:
        errors.extend(integrity_errors)
        return "", errors

    # Read units
    units: List[Tuple[str, str]] = []
    for path in unit_paths:
        with open(path, "r", encoding="utf-8") as f:
            units.append((os.path.basename(path), f.read()))

    meta = _extract_doc_metadata(units)
    if doc_title:
        meta["title"] = doc_title
    if doc_version:
        meta["version"] = doc_version
    if doc_date:
        meta["date"] = doc_date

    # Build document body
    body_sections: List[str] = []
    h1_found = False

    for name, content in units:
        lines = content.splitlines()
        filtered_lines: List[str] = []
        i = 0
        while i < len(lines):
            line = lines[i]
            # Strip redundant metadata table if present in unit files
            if line.strip().startswith("| Attribute | Value |") or line.strip().startswith("| **Title**"):
                # Skip table
                while i < len(lines) and lines[i].strip().startswith("|"):
                    i += 1
                continue

            # If H1 is already declared, avoid duplicating H1
            if line.strip().startswith("# "):
                if h1_found:
                    # Downgrade to H2 or skip
                    line = "#" + line
                else:
                    h1_found = True
                    meta["title"] = line.strip()[2:].strip()

            filtered_lines.append(line)
            i += 1

        cleaned_unit = "\n".join(filtered_lines).strip()
        if cleaned_unit:
            body_sections.append(cleaned_unit)

    full_body = "\n\n".join(body_sections)

    # Document Header Table
    header_table = f"""| Attribute | Value |
| :--- | :--- |
| **Title** | {meta['title']} |
| **Version** | {meta['version']} |
| **Date** | {meta['date']} |
"""

    # Extract headings for TOC
    headings = extract_headings(full_body)
    toc = generate_table_of_contents(headings, max_depth=2)

    # If first section starts with H1, place TOC after H1
    if full_body.startswith("# "):
        parts = full_body.split("\n", 1)
        h1_line = parts[0]
        remainder = parts[1] if len(parts) > 1 else ""
        assembled = f"{header_table}\n{h1_line}\n\n{toc}\n{remainder.strip()}\n"
    else:
        assembled = f"{header_table}\n# {meta['title']}\n\n{toc}\n{full_body.strip()}\n"

    # Verify link consistency
    link_errors = verify_markdown_links(assembled)
    if link_errors:
        errors.extend(link_errors)

    return assembled, errors


def assemble_conops(
    input_dir: str,
    output_dir: str,
    verify_only: bool = False,
) -> bool:
    """
    Orchestrates the assembly and validation of both CONOPS.md and MISSION_INTENT.md.
    """
    print(f"[*] ConOps Assembly Engine starting: input='{input_dir}', output='{output_dir}', verify_only={verify_only}")

    # Determine paths
    conops_units_dir = os.path.join(input_dir, "conops")
    if not os.path.isdir(conops_units_dir) and os.path.isdir(os.path.join(input_dir, "units", "conops")):
        conops_units_dir = os.path.join(input_dir, "units", "conops")

    mission_units_dir = os.path.join(input_dir, "mission_intent")
    if not os.path.isdir(mission_units_dir) and os.path.isdir(os.path.join(input_dir, "units", "mission_intent")):
        mission_units_dir = os.path.join(input_dir, "units", "mission_intent")

    all_errors: List[str] = []

    # 1. Assemble CONOPS.md
    if os.path.isdir(conops_units_dir):
        print(f"[*] Assembling Concept of Operations from '{conops_units_dir}'...")
        conops_doc, conops_errs = assemble_document(
            units_dir=conops_units_dir,
            doc_title="Concept of Operations (ConOps)",
        )
        if conops_errs:
            all_errors.extend([f"[CONOPS] {err}" for err in conops_errs])
        elif not verify_only:
            os.makedirs(output_dir, exist_ok=True)
            out_file = os.path.join(output_dir, "CONOPS.md")
            with open(out_file, "w", encoding="utf-8") as f:
                f.write(conops_doc)
            print(f"[+] Successfully wrote compiled ConOps to '{out_file}'.")
    else:
        print(f"[-] ConOps units directory not found at '{conops_units_dir}'. Skipping CONOPS.md assembly.")

    # 2. Assemble MISSION_INTENT.md
    if os.path.isdir(mission_units_dir):
        print(f"[*] Assembling Tactical Mission Intent from '{mission_units_dir}'...")
        mission_doc, mission_errs = assemble_document(
            units_dir=mission_units_dir,
            doc_title="Tactical Mission Intent & Execution Plan",
        )
        if mission_errs:
            all_errors.extend([f"[MISSION_INTENT] {err}" for err in mission_errs])
        elif not verify_only:
            os.makedirs(output_dir, exist_ok=True)
            out_file = os.path.join(output_dir, "MISSION_INTENT.md")
            with open(out_file, "w", encoding="utf-8") as f:
                f.write(mission_doc)
            print(f"[+] Successfully wrote compiled Mission Intent to '{out_file}'.")
    else:
        print(f"[-] Mission Intent units directory not found at '{mission_units_dir}'. Skipping MISSION_INTENT.md assembly.")

    if all_errors:
        print("\n[!] ConOps Assembly Errors encountered:")
        for err in all_errors:
            print(f"    - {err}")
        return False

    print("[+] All ConOps assembly and verification checks passed cleanly.")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Deterministic ConOps & Mission Intent Assembly Engine (ISO 29148 / NATO STANAG 4586 / OMG UAF)."
    )
    parser.add_argument(
        "--input-dir",
        default="docs/conops/units",
        help="Input directory containing 'conops/' and 'mission_intent/' unit markdown directories (default: docs/conops/units).",
    )
    parser.add_argument(
        "--output-dir",
        default="docs/conops",
        help="Output directory where assembled CONOPS.md and MISSION_INTENT.md are written (default: docs/conops).",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Verify unit integrity and link resolution without writing output files.",
    )

    args = parser.parse_args()

    success = assemble_conops(
        input_dir=args.input_dir,
        output_dir=args.output_dir,
        verify_only=args.verify,
    )
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
