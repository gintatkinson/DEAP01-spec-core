"""
Parser for Cited Research Inventory & Declared-Total Population Register (CORE #97).
(`parity_auditor/parsers/research_inventory.py`)

Parses:
1. Normative Standards & Baseline Documents Inventory table
   (Schema: Standard / Baseline ID, Issuing Body, Title, Applicable Clauses, Obligation Category, Declared Total, Clause Citation)
2. Declared-Total Population Register
   (Category, Standard ID, Target Metric / Obligation Count, Verification Mechanism, Public Clause Citation)
3. External Additions & Domain Extensions Registry (all clause-cited)
4. Clause-Level Allocation & Traceability Matrix
5. Normative Completeness & Gap Analysis Metrics
"""

import os
import re
from typing import Any, Dict, List, Optional, Sequence, Tuple, Union

from .base import IParser
from ..core.models import (
    NormativeStandard,
    PopulationRegisterEntry,
    ExternalAdditionEntry,
    ClauseAllocationEntry,
    ResearchInventoryDocument,
)


CLAUSE_LOCATOR_PATTERN = re.compile(
    r'(?:§+|(?:\b(?:section|clause|annex|task|app\.|appendix|table|article|chapter|part)\b))\s*[a-zA-Z0-9\.\-_]+',
    re.IGNORECASE,
)

INVALID_CITATION_KEYWORDS = {
    "tbd",
    "n/a",
    "none",
    "unknown",
    "internal",
    "custom",
    "uncited",
    "speculative",
    "un-cited",
    "placeholder",
}


def is_valid_public_clause_citation(citation: str, allow_template_tokens: bool = False) -> bool:
    """
    Asserts whether a given citation string represents a valid formal public clause citation.

    A valid clause citation:
    1. Must not be empty or whitespace.
    2. Must not be a placeholder token (e.g. {{...}}) unless allow_template_tokens is True.
    3. Must not be a forbidden vague placeholder (TBD, N/A, None, unknown, custom, etc.).
    4. Must contain an explicit clause/section/annex/task/appendix locator (e.g., '§6.4.2', 'Task 201', 'Annex B', 'Section 5.2').
    """
    if not citation or not citation.strip():
        return False

    cleaned = citation.strip()

    # Template token check
    if re.match(r'^\{\{[A-Za-z0-9_]+(?::[^\}]*)?\}\}$', cleaned):
        return allow_template_tokens

    # Normalize and check forbidden keywords
    norm = re.sub(r'[^a-zA-Z0-9\-/]', '', cleaned).lower()
    if norm in INVALID_CITATION_KEYWORDS:
        return False

    # Check for presence of section symbol '§' or keyword locators
    if "§" in cleaned:
        # Check that § is followed by some clause identifier
        match = re.search(r'§+\s*[a-zA-Z0-9\.\-_]+', cleaned)
        return bool(match)

    match = CLAUSE_LOCATOR_PATTERN.search(cleaned)
    return bool(match)


def parse_markdown_table_rows(table_text: str) -> Tuple[List[str], List[Dict[str, str]]]:
    """
    Parses a CommonMark markdown table string into header keys and a list of row dictionaries.
    """
    lines = [l.strip() for l in table_text.strip().splitlines() if l.strip()]
    if len(lines) < 2:
        return [], []

    # Filter out lines that don't look like table rows
    table_lines = [l for l in lines if l.startswith("|") and l.endswith("|")]
    if len(table_lines) < 2:
        return [], []

    def split_row(line: str) -> List[str]:
        inner = line[1:-1]
        raw_cells = inner.split("|")
        return [c.strip() for c in raw_cells]

    headers_raw = split_row(table_lines[0])
    headers = [re.sub(r'[*`_]', '', h).strip() for h in headers_raw]

    # Check if second line is separator
    separator_line = table_lines[1]
    if not re.match(r'^\s*\|(?:\s*:?-+:?\s*\|)+\s*$', separator_line):
        return headers, []

    data_rows = []
    for line in table_lines[2:]:
        cells = split_row(line)
        if len(cells) < len(headers):
            # Pad with empty strings
            cells.extend([""] * (len(headers) - len(cells)))
        row_dict = {}
        for h, c in zip(headers, cells):
            row_dict[h] = c
        data_rows.append(row_dict)

    return headers, data_rows


def normalize_header_key(key: str) -> str:
    """Normalizes header string for robust matching."""
    clean = re.sub(r'[*`_]', '', key).strip().lower()
    clean = re.sub(r'[\s\-/\\]+', '_', clean)
    return clean.strip('_')


class ResearchInventoryParser(IParser):
    """Parser implementation for Cited Research Inventory documents."""

    def can_parse(self, filepath_or_content: str) -> bool:
        norm_input = filepath_or_content.lower()
        if "research_inventory" in norm_input or "research-inventory" in norm_input:
            return True
        if os.path.exists(filepath_or_content):
            try:
                with open(filepath_or_content, "r", encoding="utf-8") as f:
                    content = f.read(2048)
            except Exception:
                return False
        else:
            content = filepath_or_content[:2048]

        return (
            "Cited Research Inventory" in content
            or "Normative Standards & Baseline Documents Inventory" in content
            or "Declared-Total Population Register" in content
            or "Normative Standards" in content
        )

    def parse(self, filepath_or_content: str) -> ResearchInventoryDocument:
        filepath = ""
        if os.path.exists(filepath_or_content):
            filepath = filepath_or_content
            with open(filepath_or_content, "r", encoding="utf-8") as f:
                content = f.read()
        else:
            content = filepath_or_content

        doc = ResearchInventoryDocument(filepath=filepath)
        doc.metadata = self.extract_metadata(content)
        doc.standards = self.extract_standards_table(content)
        doc.population_register = self.extract_population_register(content)
        doc.external_additions = self.extract_external_additions(content)
        doc.clause_allocations = self.extract_clause_allocations(content)
        doc.gap_analysis = self.extract_gap_analysis(content)

        return doc

    def extract_metadata(self, content: str) -> Dict[str, Any]:
        """Extracts key-value metadata from leading table or frontmatter."""
        meta: Dict[str, Any] = {}
        # 1. Table metadata
        table_match = re.search(r'\|\s*Attribute\s*\|\s*Value\s*\|(.*?)(?=\n#|\n##|\Z)', content, re.DOTALL | re.IGNORECASE)
        if table_match:
            _, rows = parse_markdown_table_rows(table_match.group(0))
            for r in rows:
                k = r.get("Attribute", "").strip()
                v = r.get("Value", "").strip()
                if k and v:
                    clean_k = re.sub(r'[*`_]', '', k).strip().lower()
                    meta[clean_k] = v

        # 2. Scope attributes
        scope_match = re.search(r'##\s*1\.\s*Scope\s*&\s*System\s*Identification(.*?)(?=\n##|\Z)', content, re.DOTALL | re.IGNORECASE)
        if scope_match:
            scope_text = scope_match.group(1)
            for line in scope_text.splitlines():
                m = re.match(r'-\s*\*\*([^*]+)\*\*:\s*(.*)', line.strip())
                if m:
                    k = m.group(1).strip().lower()
                    v = m.group(2).strip()
                    meta[k] = v

        return meta

    def extract_standards_table(self, content: str) -> List[NormativeStandard]:
        """Extracts rows from Normative Standards & Baseline Documents Inventory table."""
        standards: List[NormativeStandard] = []
        section_match = re.search(
            r'##\s*2\.\s*(?:Normative\s+Standards(?:\s*&\s*Baseline\s+Documents\s+Inventory)?|Applicable\s+Regulatory\s*&\s*Domain\s+Standards\s+Baseline)(.*?)(?=\n##|\Z)',
            content,
            re.DOTALL | re.IGNORECASE,
        )
        if not section_match:
            return standards

        section_text = section_match.group(1)
        _, rows = parse_markdown_table_rows(section_text)

        for row in rows:
            norm_row = {normalize_header_key(k): v for k, v in row.items()}

            std_id = (
                norm_row.get("standard_baseline_id")
                or norm_row.get("standard_id")
                or norm_row.get("standard")
                or norm_row.get("baseline_id")
                or ""
            )
            issuing_body = (
                norm_row.get("issuing_body")
                or norm_row.get("issuing_body_sdo")
                or norm_row.get("sdo")
                or norm_row.get("organization")
                or ""
            )
            title = (
                norm_row.get("title")
                or norm_row.get("title_baseline")
                or norm_row.get("document_title")
                or ""
            )
            applicable_clauses = (
                norm_row.get("applicable_clauses")
                or norm_row.get("applicable_clause")
                or norm_row.get("clauses")
                or norm_row.get("clause")
                or ""
            )
            obligation_cat = (
                norm_row.get("obligation_category")
                or norm_row.get("category")
                or norm_row.get("domain_category")
                or norm_row.get("verification_scope")
                or ""
            )
            declared_total_raw = (
                norm_row.get("declared_total")
                or norm_row.get("target_metric_obligation_count")
                or norm_row.get("obligation_count")
                or norm_row.get("total")
                or "1"
            )
            declared_total = 1
            num_match = re.search(r'\d+', str(declared_total_raw))
            if num_match:
                try:
                    declared_total = int(num_match.group(0))
                except ValueError:
                    declared_total = 1

            clause_citation = (
                norm_row.get("clause_citation")
                or norm_row.get("public_clause_citation")
                or norm_row.get("formal_clause_citation")
                or norm_row.get("citation")
                or applicable_clauses
            )

            if std_id:
                standards.append(
                    NormativeStandard(
                        standard_id=std_id,
                        issuing_body=issuing_body,
                        title=title,
                        applicable_clauses=applicable_clauses,
                        obligation_category=obligation_cat,
                        declared_total=declared_total,
                        clause_citation=clause_citation,
                        raw=row,
                    )
                )

        return standards

    def extract_population_register(self, content: str) -> List[PopulationRegisterEntry]:
        """Extracts rows from Section 3 Declared-Total Population Register table."""
        entries: List[PopulationRegisterEntry] = []
        section_match = re.search(
            r'##\s*3\.\s*Declared[- ]Total\s+Population\s+Register(.*?)(?=\n##|\Z)',
            content,
            re.DOTALL | re.IGNORECASE,
        )
        if not section_match:
            return entries

        section_text = section_match.group(1)
        _, rows = parse_markdown_table_rows(section_text)

        for row in rows:
            norm_row = {normalize_header_key(k): v for k, v in row.items()}

            ob_id = (
                norm_row.get("obligation_id")
                or norm_row.get("population_id")
                or norm_row.get("id")
            )
            if ob_id:
                ob_id = re.sub(r'[`*_\'"]', '', ob_id).strip()

            category = (
                norm_row.get("category")
                or norm_row.get("obligation_category")
                or ""
            )
            standard_id = (
                norm_row.get("standard_id")
                or norm_row.get("standard_baseline_id")
                or norm_row.get("standard")
                or ""
            )
            target_metric = (
                norm_row.get("target_metric_obligation_count")
                or norm_row.get("target_metric")
                or norm_row.get("target_subsystem_port")
                or ""
            )
            ob_count = 1
            num_match = re.search(r'\d+', str(target_metric))
            if num_match:
                try:
                    ob_count = int(num_match.group(0))
                except ValueError:
                    ob_count = 1

            verification = (
                norm_row.get("verification_mechanism")
                or norm_row.get("verification_method")
                or norm_row.get("verification_scope")
                or ""
            )
            clause_citation = (
                norm_row.get("public_clause_citation")
                or norm_row.get("formal_clause_citation")
                or norm_row.get("clause_citation")
                or norm_row.get("citation")
                or ""
            )

            if category or standard_id or clause_citation:
                entries.append(
                    PopulationRegisterEntry(
                        category=category,
                        standard_id=standard_id,
                        obligation_count=ob_count,
                        verification_mechanism=verification,
                        clause_citation=clause_citation,
                        obligation_id=ob_id,
                        target_metric=str(target_metric),
                        raw=row,
                    )
                )

        return entries

    def extract_external_additions(self, content: str) -> List[ExternalAdditionEntry]:
        """Extracts rows from Section 4 External Additions & Domain Extensions Registry table."""
        entries: List[ExternalAdditionEntry] = []
        section_match = re.search(
            r'##\s*4\.\s*External\s+Additions\s*&\s*Domain\s+Extensions\s+Registry(.*?)(?=\n##|\Z)',
            content,
            re.DOTALL | re.IGNORECASE,
        )
        if not section_match:
            return entries

        section_text = section_match.group(1)
        _, rows = parse_markdown_table_rows(section_text)

        for row in rows:
            norm_row = {normalize_header_key(k): v for k, v in row.items()}

            ext_id = (
                norm_row.get("extension_id")
                or norm_row.get("addition_id")
                or norm_row.get("id")
            )
            if ext_id:
                ext_id = re.sub(r'[`*_\'"]', '', ext_id).strip()

            category = (
                norm_row.get("category")
                or norm_row.get("domain_category")
                or "Domain Extension"
            )
            std_id = (
                norm_row.get("standard_baseline_id")
                or norm_row.get("standard_id")
                or norm_row.get("domain_extension_baseline")
                or norm_row.get("standard")
                or ""
            )
            declared_total_raw = (
                norm_row.get("declared_total")
                or norm_row.get("target_metric_obligation_count")
                or "1"
            )
            declared_total = 1
            num_match = re.search(r'\d+', str(declared_total_raw))
            if num_match:
                try:
                    declared_total = int(num_match.group(0))
                except ValueError:
                    declared_total = 1

            target_metric = norm_row.get("target_metric_obligation_count") or str(declared_total)
            verification = (
                norm_row.get("verification_mechanism")
                or norm_row.get("verification_method")
                or ""
            )
            citation = (
                norm_row.get("public_clause_citation")
                or norm_row.get("clause_citation")
                or norm_row.get("citation")
                or ""
            )
            justification = (
                norm_row.get("justification_domain_scope")
                or norm_row.get("justification")
                or norm_row.get("domain_scope")
            )

            if std_id or citation:
                entries.append(
                    ExternalAdditionEntry(
                        category=category,
                        standard_id=std_id,
                        declared_total=declared_total,
                        verification_mechanism=verification,
                        clause_citation=citation,
                        extension_id=ext_id,
                        target_metric=target_metric,
                        justification=justification,
                        raw=row,
                    )
                )

        return entries

    def extract_clause_allocations(self, content: str) -> List[ClauseAllocationEntry]:
        """Extracts rows from Clause-Level Allocation & Traceability Matrix table."""
        allocations: List[ClauseAllocationEntry] = []
        section_match = re.search(
            r'##\s*5\.\s*Clause[- ]Level\s+Allocation\s*&\s*Traceability\s+Matrix(.*?)(?=\n##|\Z)',
            content,
            re.DOTALL | re.IGNORECASE,
        )
        if not section_match:
            return allocations

        section_text = section_match.group(1)
        _, rows = parse_markdown_table_rows(section_text)

        for row in rows:
            norm_row = {normalize_header_key(k): v for k, v in row.items()}
            pop_id = norm_row.get("population_id") or norm_row.get("obligation_id") or ""
            if pop_id:
                pop_id = re.sub(r'[`*_\'"]', '', pop_id).strip()
            std_id = norm_row.get("standard_id") or ""
            citation = norm_row.get("clause_citation") or ""
            title = norm_row.get("clause_title_requirement_excerpt") or norm_row.get("clause_title") or ""
            phase = norm_row.get("specification_phase") or ""
            downstream = norm_row.get("downstream_spec_file_tag") or norm_row.get("downstream_spec_file") or ""

            if pop_id or std_id or citation:
                allocations.append(
                    ClauseAllocationEntry(
                        population_id=pop_id,
                        standard_id=std_id,
                        clause_citation=citation,
                        clause_title=title,
                        specification_phase=phase,
                        downstream_spec_file=downstream,
                        raw=row,
                    )
                )

        return allocations


    def extract_gap_analysis(self, content: str) -> Dict[str, Any]:
        """Extracts rows from Normative Completeness & Gap Analysis table."""
        metrics: Dict[str, Any] = {}
        section_match = re.search(
            r'##\s*6\.\s*Normative\s+Completeness\s*&\s*Gap\s+Analysis(.*?)(?=\n##|\Z)',
            content,
            re.DOTALL | re.IGNORECASE,
        )
        if not section_match:
            # Check for Section 5 heading fallback
            section_match = re.search(
                r'##\s*5\.\s*Normative\s+Completeness\s*&\s*Gap\s+Analysis(.*?)(?=\n##|\Z)',
                content,
                re.DOTALL | re.IGNORECASE,
            )
        if not section_match:
            return metrics

        section_text = section_match.group(1)
        _, rows = parse_markdown_table_rows(section_text)

        for row in rows:
            param = row.get("Metric Parameter", "")
            val = row.get("Value", "")
            if param:
                metrics[param.strip()] = val.strip()

        return metrics


def parse_research_inventory(content_or_path: str) -> ResearchInventoryDocument:
    """Convenience helper to parse a research inventory markdown content or file path."""
    parser = ResearchInventoryParser()
    return parser.parse(content_or_path)
