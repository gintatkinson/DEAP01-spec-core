# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Executive Deliverable Traceability & Completeness Validator (Check 27).
Resolves GitHub Issues #283 & #284.

Validates executive engineering deliverables in `docs/reports/` and `docs/management/`:
1. Rule `executive-table-unanchored-provenance`:
   - Scans markdown tables mentioning "Subsystem", "Part", or "System Control Action" in headers.
   - Asserts tables possess provenance columns or row-level citations to `schema/DEAP_MODEL.sysml`
     (or AST nodes / PartDefs), schema source documents (.md in schema/), or regulatory standards
     (STANAG, MIL-STD, DO-178, DO-254, ARP4754, ARP4761, ISO, IEEE, ASTM, FAA, EASA, SORA, JARUS).
   - Emits fatal error finding if rows lack SSOT/OEM citations.
2. Rule `executive-diagram-subsystem-incomplete`:
   - Scans Mermaid diagrams under headings matching "subsystem architecture" or "system architecture".
   - Passes when diagram contains `%% Realizes:` and `%% Coverage:` comments.
   - Passes when diagram encompasses all declared AST subsystems extracted from `schema/DEAP_MODEL.sysml`.
   - Passes when truncated diagram contains an explicit scoping rationale.
   - Emits fatal error finding if declared AST subsystems are omitted without metadata headers
     or explicit scoping rationale.
3. Upstream clean landing zone passes cleanly (zero findings on empty schema/ or .gitkeep).
"""

import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.workspace import WorkspaceRepository
    from ..utils.sysml_loader import load_sysml_ast_members
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.workspace import WorkspaceRepository
    from parity_auditor.utils.sysml_loader import load_sysml_ast_members

# Fail-closed SysML loader
try:
    _sysml_ast = load_sysml_ast_members(["SysMLPackage", "SysMLParser"])
    SysMLPackage = _sysml_ast.SysMLPackage
    SysMLParser = _sysml_ast.SysMLParser
except Exception:
    SysMLPackage = None
    SysMLParser = None

RULE_TABLE_UNANCHORED = "executive-table-unanchored-provenance"
RULE_DIAGRAM_INCOMPLETE = "executive-diagram-subsystem-incomplete"

RE_SUBSYSTEM_TABLE_HEADER = re.compile(
    r'\b(subsystems?|parts?|partdefs?|system\s+control\s+actions?)\b',
    re.IGNORECASE,
)

RE_PROVENANCE_HEADER = re.compile(
    r'\b(ssot|sysml|binding|metamodel|partdef|ast|oem|source|reference|citation|standard|document|authority|provenance)\b',
    re.IGNORECASE,
)

RE_ROW_CITATION = re.compile(
    r'(?:'
    r'(?:schema/)?DEAP_MODEL\.sysml'
    r'|\.sysml\b'
    r'|\bpart\s*def\b'
    r'|\bpartdef\b'
    r'|schema/[A-Za-z0-9_.-]+\.(?:md|sysml)'
    r'|\b[A-Za-z0-9_.-]+\.md\b'
    r'|\b(STANAG|MIL[- ]?STD|DO[- ]?178[A-Z]?|DO[- ]?254[A-Z]?|ARP[- ]?4754[A-Z]?|ARP[- ]?4761[A-Z]?|ISO|IEEE|ASTM|FAA|EASA|SORA|JARUS)\b'
    r')',
    re.IGNORECASE,
)

RE_ARCH_HEADING = re.compile(
    r'\b(subsystem|system)\s+architecture\b',
    re.IGNORECASE,
)

RE_REALIZES = re.compile(r'%%\s*Realizes:', re.IGNORECASE)
RE_COVERAGE = re.compile(r'%%\s*Coverage:', re.IGNORECASE)

RE_SCOPING_RATIONALE = re.compile(
    r'\b(scoping\s+rationale|scope|scoped|rationale|subset|focused\s+view|simplified\s+view|isolated\s+view|omitted)\b',
    re.IGNORECASE,
)


def _collect_parts_from_package(pkg: Any, parts_set: Set[str]) -> None:
    """Recursively collect declared part def names from SysML package AST."""
    if pkg is None:
        return
    for part in getattr(pkg, "part_defs", []) or []:
        name = getattr(part, "name", None)
        if name:
            parts_set.add(name)
        for sub_part in getattr(part, "parts", []) or []:
            sub_name = getattr(sub_part, "name", None)
            if sub_name:
                parts_set.add(sub_name)
    for part in getattr(pkg, "parts", []) or []:
        name = getattr(part, "name", None)
        if name:
            parts_set.add(name)
    for sub_pkg in getattr(pkg, "sub_packages", []) or []:
        _collect_parts_from_package(sub_pkg, parts_set)


def _extract_ast_subsystems(workspace_dir: str) -> Set[str]:
    """Extract declared AST subsystem names from schema/*.sysml models."""
    subsystems: Set[str] = set()
    schema_dir = os.path.join(workspace_dir, "schema")
    if not os.path.isdir(schema_dir):
        return subsystems

    sysml_files = [
        os.path.join(schema_dir, f)
        for f in os.listdir(schema_dir)
        if f.endswith(".sysml") and not f.startswith(".")
    ]

    for sf in sysml_files:
        try:
            with open(sf, "r", encoding="utf-8") as f:
                content = f.read()
        except OSError:
            continue

        if SysMLParser is not None and content.strip():
            try:
                pkg = SysMLParser.parse_text(content)
                _collect_parts_from_package(pkg, subsystems)
            except Exception:
                pass

        # Robust regex fallback to discover declared part defs and part instances
        matches = re.findall(r'\bpart\s+def\s+([A-Za-z0-9_]+)', content)
        for m in matches:
            subsystems.add(m)
        part_inst_matches = re.findall(r'\bpart\s+([A-Za-z0-9_]+)\s*:\s*([A-Za-z0-9_]+)', content)
        for p_name, p_type in part_inst_matches:
            subsystems.add(p_name)
            subsystems.add(p_type)

    return subsystems


def _extract_schema_sources(workspace_dir: str) -> Set[str]:
    """Extract schema source document filenames (.md) present in schema/."""
    schema_dir = os.path.join(workspace_dir, "schema")
    if not os.path.isdir(schema_dir):
        return set()
    return {
        f.lower()
        for f in os.listdir(schema_dir)
        if f.endswith(".md") and not f.startswith(".")
    }


def _extract_tables(content: str, rel_path: str) -> List[Dict[str, Any]]:
    """Extract markdown tables with header contexts and line numbers."""
    tables: List[Dict[str, Any]] = []
    lines = content.splitlines()
    i = 0
    current_heading = ""
    in_code_block = False

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if stripped.startswith("```"):
            in_code_block = not in_code_block
            i += 1
            continue

        if in_code_block:
            i += 1
            continue

        if stripped.startswith("#"):
            current_heading = stripped.lstrip("#").strip()
            i += 1
            continue

        if stripped.startswith("|") and stripped.endswith("|") and "|" in stripped[1:-1]:
            if i + 1 < len(lines):
                next_stripped = lines[i + 1].strip()
                if next_stripped.startswith("|") and re.match(r"^\|(\s*:?-+:?\s*\|)+$", next_stripped):
                    header_line = stripped
                    start_line = i + 1
                    headers = [c.strip() for c in header_line[1:-1].split("|")]

                    rows: List[Tuple[int, str, List[str]]] = []
                    i += 2
                    while i < len(lines):
                        row_line = lines[i]
                        row_stripped = row_line.strip()
                        if not (row_stripped.startswith("|") and row_stripped.endswith("|")):
                            break
                        cells = [c.strip() for c in row_stripped[1:-1].split("|")]
                        rows.append((i + 1, row_line, cells))
                        i += 1

                    tables.append({
                        "start_line": start_line,
                        "heading": current_heading,
                        "headers": headers,
                        "rows": rows,
                        "rel_path": rel_path,
                    })
                    continue
        i += 1

    return tables


def _row_has_citation(
    row_str: str,
    cells: List[str],
    prov_col_indices: List[int],
    ast_subsystems: Set[str],
    schema_docs: Optional[Set[str]] = None,
) -> bool:
    """Check if a table row contains an SSOT, AST, schema doc, or standard citation."""
    if RE_ROW_CITATION.search(row_str):
        return True

    if schema_docs:
        row_str_lower = row_str.lower()
        for doc in schema_docs:
            if doc in row_str_lower:
                return True

    for idx in prov_col_indices:
        if idx < len(cells):
            cell_val = cells[idx]
            if RE_ROW_CITATION.search(cell_val):
                return True
            for ast_name in ast_subsystems:
                if re.search(r'\b' + re.escape(ast_name) + r'\b', cell_val, re.IGNORECASE):
                    return True
            if schema_docs:
                cell_val_lower = cell_val.lower()
                for doc in schema_docs:
                    if doc in cell_val_lower:
                        return True

    return False


def _extract_mermaid_blocks(content: str, rel_path: str) -> List[Dict[str, Any]]:
    """Extract Mermaid fenced blocks with surrounding section contexts and line numbers."""
    blocks: List[Dict[str, Any]] = []
    lines = content.splitlines()
    i = 0
    fence_pattern = re.compile(r"^\s*```+\s*mermaid\s*$", re.I)
    end_fence_pattern = re.compile(r"^\s*```+\s*$")

    current_heading = ""
    section_lines: List[str] = []

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if stripped.startswith("#"):
            current_heading = stripped.lstrip("#").strip()
            section_lines = []
            i += 1
            continue

        if fence_pattern.match(line):
            start_line = i + 1
            body: List[str] = []
            i += 1
            while i < len(lines):
                if end_fence_pattern.match(lines[i]):
                    break
                body.append(lines[i])
                i += 1

            raw_block = "\n".join(body)
            preceding_section_text = "\n".join(section_lines)

            following_lines: List[str] = []
            j = i + 1
            while j < len(lines):
                if lines[j].strip().startswith("#"):
                    break
                following_lines.append(lines[j])
                j += 1
            full_section_text = preceding_section_text + "\n" + "\n".join(following_lines)

            blocks.append({
                "start_line": start_line,
                "heading": current_heading,
                "raw_block": raw_block,
                "section_text": full_section_text,
                "rel_path": rel_path,
            })
            i += 1
            continue

        section_lines.append(line)
        i += 1

    return blocks


def _is_subsystem_in_diagram(sub: str, raw_block: str) -> bool:
    """Check if declared AST subsystem is present in diagram node IDs, node labels, or subgraphs."""
    lines = [l for l in raw_block.splitlines() if not l.strip().startswith("%%")]
    body = "\n".join(lines)
    if re.search(r'\b' + re.escape(sub) + r'\b', body, re.IGNORECASE):
        return True

    norm_sub = re.sub(r'[^a-zA-Z0-9]', '', sub).lower()
    norm_body = re.sub(r'[^a-zA-Z0-9]', '', body).lower()
    if norm_sub and norm_sub in norm_body:
        return True
    return False


class ExecutiveDeliverableTraceabilityValidator(IValidator):
    """
    Check 27: Executive Deliverable Traceability & Completeness Gate.
    Resolves GitHub Issues #283 & #284.
    """

    RULE_TABLE_UNANCHORED = RULE_TABLE_UNANCHORED
    RULE_DIAGRAM_INCOMPLETE = RULE_DIAGRAM_INCOMPLETE

    def __init__(self, workspace_repo: Optional[WorkspaceRepository] = None):
        self.workspace_repo = workspace_repo

    def validate(self, repo: WorkspaceRepository, **kwargs: Any) -> List[Finding]:
        """Validate executive deliverable documents under docs/reports/ and docs/management/."""
        target_repo = repo or self.workspace_repo
        if target_repo is None:
            return []

        root_dir = Path(target_repo.workspace_dir).resolve()
        findings: List[Finding] = []

        report_candidates: List[Path] = []
        for sdir in ("reports", "management"):
            target_dir = root_dir / "docs" / sdir
            if target_dir.is_dir():
                for p in target_dir.rglob("*.md"):
                    if p.name.startswith("."):
                        continue
                    rel_parts = p.relative_to(target_dir).parts
                    if any(part.startswith(".") for part in rel_parts[:-1]):
                        continue
                    if any(part == "defects" for part in rel_parts[:-1]):
                        continue
                    report_candidates.append(p)

        if not report_candidates:
            return []

        ast_subsystems = _extract_ast_subsystems(str(root_dir))
        schema_docs = _extract_schema_sources(str(root_dir))

        for doc_path in report_candidates:
            try:
                rel_path = str(doc_path.relative_to(root_dir))
                content = doc_path.read_text(encoding="utf-8")
            except Exception:
                continue

            # 1. Rule executive-table-unanchored-provenance
            tables = _extract_tables(content, rel_path)
            for table in tables:
                headers = table["headers"]
                matched_headers = [h for h in headers if RE_SUBSYSTEM_TABLE_HEADER.search(h)]
                if not matched_headers:
                    continue

                prov_col_indices = [
                    idx for idx, h in enumerate(headers)
                    if RE_PROVENANCE_HEADER.search(h)
                ]
                has_prov_cols = len(prov_col_indices) > 0

                unanchored_rows: List[Tuple[int, str, List[str]]] = []
                for row_line, row_str, cells in table["rows"]:
                    if not _row_has_citation(row_str, cells, prov_col_indices, ast_subsystems, schema_docs):
                        unanchored_rows.append((row_line, row_str, cells))

                if not has_prov_cols and unanchored_rows:
                    findings.append(Finding(
                        rule_id=RULE_TABLE_UNANCHORED,
                        message=(
                            f"{rel_path}:{table['start_line']}: [{RULE_TABLE_UNANCHORED}] "
                            f"Executive table under '{table['heading'] or 'Subsystem Allocation'}' has unanchored provenance: "
                            f"lacks required SSOT/OEM provenance columns and {len(unanchored_rows)} row(s) lack citations."
                        ),
                        location=f"{rel_path}:{table['start_line']}",
                    ))
                elif unanchored_rows:
                    for row_line, row_str, _ in unanchored_rows:
                        findings.append(Finding(
                            rule_id=RULE_TABLE_UNANCHORED,
                            message=(
                                f"{rel_path}:{row_line}: [{RULE_TABLE_UNANCHORED}] "
                                f"Executive table under '{table['heading'] or 'Subsystem Allocation'}' row lacks required SSOT/AST or regulatory citation: {row_str.strip()}"
                            ),
                            location=f"{rel_path}:{row_line}",
                        ))

            # 2. Rule executive-diagram-subsystem-incomplete
            mermaid_blocks = _extract_mermaid_blocks(content, rel_path)
            for diag in mermaid_blocks:
                heading = diag["heading"]
                if not RE_ARCH_HEADING.search(heading):
                    continue

                raw_block = diag["raw_block"]
                if RE_REALIZES.search(raw_block) and RE_COVERAGE.search(raw_block):
                    continue

                if not ast_subsystems:
                    continue

                missing_subsystems = {
                    sub for sub in ast_subsystems
                    if not _is_subsystem_in_diagram(sub, raw_block)
                }

                if not missing_subsystems:
                    continue

                section_text = diag["section_text"]
                if RE_SCOPING_RATIONALE.search(raw_block) or RE_SCOPING_RATIONALE.search(section_text):
                    continue

                findings.append(Finding(
                    rule_id=RULE_DIAGRAM_INCOMPLETE,
                    message=(
                        f"{rel_path}:{diag['start_line']}: [{RULE_DIAGRAM_INCOMPLETE}] "
                        f"Architecture diagram under '{heading}' is incomplete: "
                        f"missing declared AST subsystems ({', '.join(sorted(missing_subsystems))}) "
                        f"without %% Realizes:/%% Coverage: metadata headers or explicit scoping rationale."
                    ),
                    location=f"{rel_path}:{diag['start_line']}",
                ))

        return findings


def validate_executive_deliverable_traceability(repo_root: Optional[str] = None) -> List[Finding]:
    """Standalone validation entrypoint for Executive Deliverable Traceability & Completeness Gate."""
    if repo_root is None:
        repo_root = os.getcwd()
    repo = WorkspaceRepository(workspace_dir=str(repo_root))
    validator = ExecutiveDeliverableTraceabilityValidator(workspace_repo=repo)
    return validator.validate(repo)
