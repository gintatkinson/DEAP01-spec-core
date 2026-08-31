r"""
Safety Traceability & Bidirectional Set-Equality Validator.

Enforces bi-directional mathematical set-equality between STPA Unsafe Control Actions (UCAs)
documented in docs/safety/ and formal SysML constraint assertions in SysML AST / .pipeline/schema.sysml:
    UCAs_markdown \ UCAs_SysML = ∅
    UCAs_SysML \ UCAs_markdown = ∅
"""

import os
import re
from typing import Dict, List, Set, Optional, Tuple

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository


def _normalize_uca_id(raw_id: str) -> str:
    """Normalize UCA ID into canonical form, e.g. 'Assert_UCA_01', 'UCA-1', 'UCA_01' -> 'UCA-01'."""
    raw = raw_id.strip()
    m = re.search(r'UCA[_-]?0*(\d+[a-zA-Z0-9_-]*)', raw, re.IGNORECASE)
    if m:
        suffix = m.group(1)
        if suffix.isdigit():
            return f"UCA-{int(suffix):02d}"
        return f"UCA-{suffix.upper()}"
    return raw.upper()


def _normalize_fm_id(raw_id: str) -> str:
    """Normalize FMECA ID into canonical form, e.g. 'FM-1', 'FM_01' -> 'FM-01'."""
    raw = raw_id.strip()
    m = re.search(r'FM[_-]?0*(\d+[a-zA-Z0-9_-]*)', raw, re.IGNORECASE)
    if m:
        suffix = m.group(1)
        if suffix.isdigit():
            return f"FM-{int(suffix):02d}"
        return f"FM-{suffix.upper()}"
    return raw.upper()


class SafetyTraceValidator(IValidator):
    """Bidirectional Safety Traceability Validator."""

    def extract_markdown_ucas(self, repo: WorkspaceRepository) -> Tuple[Set[str], Dict[str, str]]:
        """Extracts all STPA UCA IDs defined in docs/safety/ markdown files."""
        workspace_dir = repo.workspace_dir
        safety_dir = os.path.join(workspace_dir, "docs", "safety")
        if not os.path.isdir(safety_dir):
            return set(), {}

        ucas: Set[str] = set()
        locations: Dict[str, str] = {}

        for root, _, files in os.walk(safety_dir):
            for f in sorted(files):
                if f.endswith(".md") and f != "README.md":
                    filepath = os.path.join(root, f)
                    rel_path = os.path.relpath(filepath, workspace_dir)
                    try:
                        with open(filepath, "r", encoding="utf-8", errors="ignore") as md_file:
                            content = md_file.read()
                    except Exception:
                        continue

                    # Search for UCA patterns
                    pattern = re.compile(r'\b(UCA(?:-|_|\s*)\d+[a-zA-Z0-9_-]*)\b', re.IGNORECASE)
                    for lineno_1idx, line in enumerate(content.splitlines(), start=1):
                        for match in pattern.finditer(line):
                            raw = match.group(1)
                            canon = _normalize_uca_id(raw)
                            ucas.add(canon)
                            if canon not in locations:
                                locations[canon] = f"{rel_path}:{lineno_1idx}"

        return ucas, locations

    def extract_sysml_ucas(self, repo: WorkspaceRepository) -> Tuple[Set[str], Dict[str, str]]:
        """Extracts all assert constraint Assert_UCA_* and constraint defs from SysML AST."""
        workspace_dir = repo.workspace_dir
        sysml_files = []

        pipeline_schema = os.path.join(workspace_dir, ".pipeline", "schema.sysml")
        if os.path.isfile(pipeline_schema):
            sysml_files.append(pipeline_schema)

        schema_dir = os.path.join(workspace_dir, "schema")
        if os.path.isdir(schema_dir):
            for root, _, files in os.walk(schema_dir):
                for f in sorted(files):
                    if f.endswith(".sysml"):
                        sysml_files.append(os.path.join(root, f))

        ucas: Set[str] = set()
        locations: Dict[str, str] = {}

        constraint_pattern = re.compile(
            r'\b(?:assert\s+constraint|constraint\s+def|constraint|requirement\s+def|requirement)\s+([a-zA-Z0-9_]+)',
            re.IGNORECASE
        )

        for sfile in sysml_files:
            rel_path = os.path.relpath(sfile, workspace_dir)
            try:
                with open(sfile, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            for lineno_1idx, line in enumerate(content.splitlines(), start=1):
                for match in constraint_pattern.finditer(line):
                    decl_name = match.group(1)
                    if "uca" in decl_name.lower():
                        canon = _normalize_uca_id(decl_name)
                        ucas.add(canon)
                        if canon not in locations:
                            locations[canon] = f"{rel_path}:{lineno_1idx}"

        return ucas, locations

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        r"""
        Verifies bidirectional mathematical set-equality:
            markdown_ucas \ sysml_ucas == ∅
            sysml_ucas \ markdown_ucas == ∅
        """
        md_ucas, md_locs = self.extract_markdown_ucas(repo)
        sysml_ucas, sysml_locs = self.extract_sysml_ucas(repo)

        # Upstream template clean landing zones: if both are empty, pass gracefully
        if not md_ucas and not sysml_ucas:
            return []

        errors: List[Finding] = []

        missing_in_sysml = sorted(md_ucas - sysml_ucas)
        missing_in_markdown = sorted(sysml_ucas - md_ucas)

        for uca in missing_in_sysml:
            loc = md_locs.get(uca, "docs/safety")
            errors.append(Finding(
                "safety-trace-uca-missing-in-sysml",
                f"STPA Unsafe Control Action '{uca}' defined in {loc} has no corresponding constraint assertion in SysML AST.",
                location=loc,
                detail={"uca_id": uca, "missing_in": "sysml"}
            ))

        for uca in missing_in_markdown:
            loc = sysml_locs.get(uca, ".pipeline/schema.sysml")
            errors.append(Finding(
                "safety-trace-uca-missing-in-markdown",
                f"SysML AST contains constraint assertion for '{uca}' at {loc}, but no corresponding UCA definition exists in docs/safety/.",
                location="docs/safety",
                detail={"uca_id": uca, "missing_in": "markdown"}
            ))

        return errors
