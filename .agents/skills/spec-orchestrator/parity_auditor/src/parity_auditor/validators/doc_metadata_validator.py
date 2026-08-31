"""
Document Metadata & Frontmatter Integrity Validator.

Validates that specification, safety, architecture, and operational markdown
documents in docs/ contain standard frontmatter metadata tables with required fields:
1. Title (non-empty string, clean canonical format)
2. Version (semantic versioning format: v?X.Y[.Z])
3. Date or Release Date (ISO 8601 format: YYYY-MM-DD)

Emits structured Finding objects:
- doc-metadata-missing-field
- doc-metadata-invalid-date-format
- doc-metadata-invalid-version-format
- doc-metadata-concatenated-title
- doc-metadata-title-heading-mismatch

Gracefully skips READMEs, empty stubs, and documents marked as optional.
"""

import os
import re
from datetime import datetime
from typing import Dict, List, Optional, Set, Tuple, Any

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

EXCLUDED_DIRS = {".git", "node_modules", ".dart_tool", "build", "__pycache__"}


def _clean_str(val: str) -> str:
    """Strip markdown formatting, quotes, and whitespace from string/table cell value."""
    if not val:
        return ""
    cleaned = val.strip()
    cleaned = re.sub(r'^[*`_"\']+|[*`_"\']+$', '', cleaned).strip()
    return cleaned


def _normalize_key(key_raw: str) -> str:
    """Normalize metadata field name by lowercasing and stripping formatting."""
    if not key_raw:
        return ""
    clean = re.sub(r'[*`_:#]', '', key_raw).strip().lower()
    clean = re.sub(r'[\s\-/]+', '_', clean)

    if clean in ("title", "doc_title", "document_title", "name", "document_name", "system_title", "spec_title"):
        return "title"
    if clean in ("version", "doc_version", "document_version", "revision", "ver", "spec_version"):
        return "version"
    if clean in ("date", "doc_date", "document_date", "created", "created_date", "creation_date", "effective_date"):
        return "date"
    if clean in ("release_date", "released_date", "release", "published_date", "publication_date"):
        return "release_date"
    if clean in ("status", "doc_status", "document_status", "state"):
        return "status"
    if clean in ("target_baseline", "target_base_line", "baseline", "target_version"):
        return "target_baseline"
    if clean in ("document_id", "doc_id", "id", "document_identifier", "doc_identifier", "spec_id"):
        return "document_id"
    if clean in ("optional", "is_optional"):
        return "optional"
    return clean


def _is_iso_date(val: str) -> bool:
    """Check if string is a valid ISO 8601 date (YYYY-MM-DD)."""
    if not val or not re.match(r'^\d{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12]\d|3[01])$', val):
        return False
    try:
        datetime.strptime(val, "%Y-%m-%d")
        return True
    except ValueError:
        return False


def _is_semver(val: str) -> bool:
    """Check if string adheres to semantic versioning (v?X.Y[.Z])."""
    if not val:
        return False
    return bool(re.match(r'^v?\d+\.\d+(?:\.\d+)?(?:[-+][0-9A-Za-z.-]+)?$', val))


def _has_concatenated_title_metadata(val: str) -> bool:
    """Check if title contains embedded version, date, or document ID metadata."""
    if not val:
        return False
    # a) Embedded version tokens (e.g. v\d+\.\d+, \b\d+\.\d+\.\d+\b, version\s*:\s*\d+)
    if re.search(r'(?i)\bv\d+\.\d+(?:\.\d+)?\b', val):
        return True
    if re.search(r'\b\d+\.\d+\.\d+\b', val):
        return True
    if re.search(r'(?i)\bversion\s*:\s*\d+', val):
        return True
    # b) Embedded ISO date patterns (e.g. \b\d{4}-\d{2}-\d{2}\b)
    if re.search(r'\b\d{4}-(?:0[1-9]|1[0-2])-(?:0[1-9]|[12]\d|3[01])\b', val) or re.search(r'\b\d{4}-\d{2}-\d{2}\b', val):
        return True
    # c) Embedded document ID tokens in parentheses (e.g. \(DOC-.*?\), \(DOC-[A-Z]+-[A-Z0-9-]+\) or \(.*?DOC-[A-Za-z0-9_-]+.*?\))
    if re.search(r'\(.*?\bDOC-[A-Za-z0-9_-]+.*?\)', val, re.IGNORECASE):
        return True
    if re.search(r'\(DOC-[A-Z]+-[A-Z0-9-]+\)', val, re.IGNORECASE):
        return True
    if re.search(r'\(DOC-.*?\)', val, re.IGNORECASE):
        return True
    return False


def _strip_agile_prefix(heading: str) -> str:
    """Strip agile spec prefixes ('Epic:', 'Feature:', 'Use Case:', 'User Story:') from heading text."""
    if not heading:
        return ""
    stripped = re.sub(
        r'^\s*[*_`"\']*(?:Epic|Feature|Use\s+Case|User\s+Story)[*_`"\']*\s*:\s*',
        '',
        heading,
        flags=re.IGNORECASE,
    ).strip()
    return _clean_str(stripped)


def _extract_h1_heading(content: str) -> Optional[Tuple[str, int]]:
    """
    Extract the first top-level # H1 heading in the markdown document (outside frontmatter and codeblocks).
    Returns (cleaned_and_prefix_stripped_h1_text, 1_indexed_line_number) or None.
    """
    if not content:
        return None

    lines = content.splitlines()
    in_frontmatter = False
    in_codeblock = False

    if lines and lines[0].strip() == "---":
        in_frontmatter = True

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()

        # Handle YAML frontmatter boundary
        if in_frontmatter:
            if idx > 1 and stripped in ("---", "..."):
                in_frontmatter = False
            continue

        # Handle fenced code blocks
        if stripped.startswith("```") or stripped.startswith("~~~"):
            in_codeblock = not in_codeblock
            continue

        if in_codeblock:
            continue

        # Match top-level # H1 heading (single # followed by whitespace)
        match = re.match(r'^#\s+(.+)$', stripped)
        if match:
            raw_h1 = match.group(1).strip()
            # Strip any trailing markdown header hashes (e.g. # Heading #)
            raw_h1 = re.sub(r'\s+#+\s*$', '', raw_h1)
            h1_title = _strip_agile_prefix(raw_h1)
            return h1_title, idx

    return None


def _extract_yaml_frontmatter(content: str) -> Dict[str, Tuple[str, int]]:
    """Extract metadata dictionary from YAML frontmatter at line 1 (--- ... ---)."""
    yaml_metadata: Dict[str, Tuple[str, int]] = {}
    if not content:
        return yaml_metadata

    lines = content.splitlines()
    if not lines or lines[0].strip() != "---":
        return yaml_metadata

    end_idx = None
    for idx, line in enumerate(lines[1:], start=2):
        if line.strip() in ("---", "..."):
            end_idx = idx
            break

    if end_idx is None:
        return yaml_metadata

    for idx in range(2, end_idx):
        yline = lines[idx - 1]
        if ":" in yline:
            k_raw, v_raw = yline.split(":", 1)
            k_norm = _normalize_key(k_raw)
            v_clean = _clean_str(v_raw)
            if k_norm and v_clean and k_norm not in yaml_metadata:
                yaml_metadata[k_norm] = (v_clean, idx)

    return yaml_metadata


def _parse_table_block(table_lines: List[Tuple[str, int]], extracted: Dict[str, Tuple[str, int]]) -> None:
    """Parse a table block into metadata dictionary."""
    if len(table_lines) < 2:
        return

    header_cells = [c.strip() for c in table_lines[0][0].split("|")[1:-1]]
    sep_cells = [c.strip() for c in table_lines[1][0].split("|")[1:-1]]

    is_sep = all(bool(re.match(r'^:?-+:?$', c)) for c in sep_cells) if sep_cells else False
    if not is_sep:
        return

    header_norms = [_normalize_key(h) for h in header_cells]

    # Case A: Horizontal / Columnar table (e.g. | Title | Version | Date |)
    has_horizontal_keys = any(k in ("title", "version", "date", "release_date", "document_id", "target_baseline") for k in header_norms)
    if has_horizontal_keys and len(table_lines) >= 3:
        for row_text, lineno in table_lines[2:]:
            data_cells = [c.strip() for c in row_text.split("|")[1:-1]]
            for idx, k_norm in enumerate(header_norms):
                if idx < len(data_cells):
                    v_clean = _clean_str(data_cells[idx])
                    if k_norm and v_clean and k_norm not in extracted:
                        extracted[k_norm] = (v_clean, lineno)
    else:
        # Case B: Vertical key-value table (e.g. | Attribute | Specification Detail | or | Key | Value |)
        for row_text, lineno in table_lines[2:]:
            cells = [c.strip() for c in row_text.split("|")[1:-1]]
            if len(cells) >= 2:
                k_norm = _normalize_key(cells[0])
                v_clean = _clean_str(cells[1])
                if k_norm and v_clean and k_norm not in extracted:
                    extracted[k_norm] = (v_clean, lineno)


def _extract_table_metadata(content: str) -> Dict[str, Tuple[str, int]]:
    """Extract metadata dictionary from Markdown tables."""
    table_metadata: Dict[str, Tuple[str, int]] = {}
    if not content:
        return table_metadata

    lines = content.splitlines()
    i = 0
    while i < len(lines):
        line_raw = lines[i]
        line_clean = re.sub(r'^\s*>\s*', '', line_raw).strip()
        if line_clean.startswith("|") and line_clean.endswith("|"):
            table_lines: List[Tuple[str, int]] = []
            while i < len(lines):
                l_clean = re.sub(r'^\s*>\s*', '', lines[i]).strip()
                if l_clean.startswith("|") and l_clean.endswith("|"):
                    table_lines.append((l_clean, i + 1))
                    i += 1
                else:
                    break
            _parse_table_block(table_lines, table_metadata)
        else:
            i += 1

    return table_metadata


class DocMetadataValidator(IValidator):
    """Validator for markdown document frontmatter metadata tables and schemas."""

    def extract_metadata(self, content: str) -> Tuple[Dict[str, Tuple[str, int]], bool]:
        """
        Extract metadata key-value dictionary and optional flag from markdown document.

        Returns:
            Tuple of (extracted_dict, is_optional), where extracted_dict maps
            normalized key -> (cleaned_value, line_number_1indexed).
        """
        extracted: Dict[str, Tuple[str, int]] = {}
        is_optional = False

        if not content or not content.strip():
            return extracted, True

        # Check explicit optional/stub comments or markers
        if re.search(r'<!--\s*(?:doc[-_]metadata|metadata)?\s*:?\s*(?:optional|stub|draft)\s*-->', content, re.IGNORECASE):
            is_optional = True
        if re.search(r'<!--\s*optional\s*-->', content, re.IGNORECASE):
            is_optional = True

        # 1. Parse YAML frontmatter if present
        yaml_data = _extract_yaml_frontmatter(content)
        for k, v in yaml_data.items():
            if k not in extracted:
                extracted[k] = v

        # 2. Extract H1 heading
        h1_info = _extract_h1_heading(content)
        if h1_info is not None and h1_info[0]:
            # If YAML frontmatter is NOT present (or didn't provide a title), extract title from H1 heading
            if "title" not in yaml_data:
                extracted["title"] = h1_info

        # 3. Parse Markdown tables (vertical key-value or horizontal columnar)
        table_data = _extract_table_metadata(content)
        for k, v in table_data.items():
            if k not in extracted:
                extracted[k] = v

        # Check status or optional field in metadata
        opt_entry = extracted.get("optional")
        if opt_entry and opt_entry[0].lower() in ("true", "yes", "1"):
            is_optional = True

        status_entry = extracted.get("status")
        if status_entry and status_entry[0].lower() in ("optional", "stub"):
            is_optional = True

        return extracted, is_optional

    def _parse_table_block(self, table_lines: List[Tuple[str, int]], extracted: Dict[str, Tuple[str, int]]) -> None:
        """Parse a table block into metadata dictionary (delegates to module helper)."""
        _parse_table_block(table_lines, extracted)

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """
        Executes document metadata and frontmatter validation.

        Scans markdown documents under docs/ and asserts:
        1. Title presence in YAML frontmatter or Markdown table / H1 heading
        2. Exact match between YAML title and the # H1 heading text (if YAML frontmatter is present)
        3. Title format (clean canonical name without concatenated metadata)
        4. Version presence and semver conformance (v?X.Y[.Z])
        5. Date / Release Date presence and ISO 8601 conformance (YYYY-MM-DD)
        """
        workspace_dir = repo.workspace_dir
        docs_dir = os.path.join(workspace_dir, "docs")
        if not os.path.isdir(docs_dir):
            return []

        errors: List[Finding] = []
        doc_files: List[Tuple[str, str]] = []

        for root, dirs, files in os.walk(docs_dir):
            dirs[:] = [d for d in dirs if d not in EXCLUDED_DIRS]
            for f in sorted(files):
                if f.endswith(".md"):
                    if f.lower() == "readme.md":
                        continue
                    full_path = os.path.join(root, f)
                    rel_path = os.path.relpath(full_path, workspace_dir)
                    doc_files.append((full_path, rel_path))

        for full_path, rel_path in doc_files:
            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            if not content.strip():
                continue

            yaml_data = _extract_yaml_frontmatter(content)
            table_data = _extract_table_metadata(content)
            h1_info = _extract_h1_heading(content)
            extracted, is_optional = self.extract_metadata(content)

            # Skip stub/optional files marked as optional
            if is_optional:
                continue

            # 1. Validate presence of Title, Version, Date or Release Date
            has_title = bool(extracted.get("title", ("", 0))[0])
            has_version = bool(extracted.get("version", ("", 0))[0])
            has_date = bool(extracted.get("date", ("", 0))[0] or extracted.get("release_date", ("", 0))[0])

            if not (has_title and has_version and has_date):
                missing = []
                if not has_title:
                    missing.append("Title")
                if not has_version:
                    missing.append("Version")
                if not has_date:
                    missing.append("Date (or Release Date)")

                errors.append(Finding(
                    "doc-metadata-missing-field",
                    f"{rel_path}: Missing mandatory document metadata field(s): {', '.join(missing)}.",
                    location=rel_path,
                    detail={"missing_fields": missing}
                ))

            # 2. Validate Date format (ISO 8601 YYYY-MM-DD)
            date_entry = extracted.get("date") or extracted.get("release_date")
            if date_entry and date_entry[0]:
                raw_date, d_line = date_entry
                field_name = "Release Date" if "release_date" in extracted and "date" not in extracted else "Date"
                if not _is_iso_date(raw_date):
                    loc = f"{rel_path}:{d_line}" if d_line > 0 else rel_path
                    errors.append(Finding(
                        "doc-metadata-invalid-date-format",
                        f"{loc}: Invalid date format for '{field_name}': '{raw_date}'. Expected ISO 8601 format (YYYY-MM-DD).",
                        location=loc,
                        detail={"field": field_name, "value": raw_date, "expected_format": "YYYY-MM-DD"}
                    ))

            # 3. Validate Version format (semantic versioning v?X.Y[.Z])
            version_entry = extracted.get("version")
            if version_entry and version_entry[0]:
                raw_ver, v_line = version_entry
                if not _is_semver(raw_ver):
                    loc = f"{rel_path}:{v_line}" if v_line > 0 else rel_path
                    errors.append(Finding(
                        "doc-metadata-invalid-version-format",
                        f"{loc}: Invalid version format: '{raw_ver}'. Expected semantic versioning format (v?X.Y[.Z], e.g. '1.0.0' or 'v1.0').",
                        location=loc,
                        detail={"field": "Version", "value": raw_ver, "expected_format": "v?X.Y[.Z]"}
                    ))

            # 4. Validate Title format (clean canonical name without concatenated metadata)
            titles_to_check: List[Tuple[str, int]] = []
            if "title" in yaml_data:
                titles_to_check.append(yaml_data["title"])
            if h1_info and h1_info[0]:
                titles_to_check.append(h1_info)
            if "title" in table_data:
                titles_to_check.append(table_data["title"])
            if "title" in extracted:
                titles_to_check.append(extracted["title"])

            seen_titles: Set[str] = set()
            for raw_title, t_line in titles_to_check:
                if raw_title in seen_titles:
                    continue
                seen_titles.add(raw_title)
                if _has_concatenated_title_metadata(raw_title):
                    loc = f"{rel_path}:{t_line}" if t_line > 0 else rel_path
                    errors.append(Finding(
                        "doc-metadata-concatenated-title",
                        f"{rel_path}: Document title contains concatenated metadata attributes ('{raw_title}'). Titles must be clean canonical names without embedded versions, dates, or IDs.",
                        location=loc,
                        detail={"title": raw_title}
                    ))

            # 5. Exact match between YAML title and the # H1 heading text (if YAML frontmatter title is present)
            if "title" in yaml_data:
                yaml_title = yaml_data["title"][0]
                h1_title = h1_info[0] if h1_info is not None else ""
                if yaml_title != h1_title:
                    errors.append(Finding(
                        "doc-metadata-title-heading-mismatch",
                        f"{rel_path}: YAML title ('{yaml_title}') does not match H1 heading ('{h1_title}').",
                        location=rel_path,
                        detail={"yaml_title": yaml_title, "h1_title": h1_title}
                    ))

        return errors
