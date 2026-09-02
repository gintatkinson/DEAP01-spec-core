"""
Gate 27: Cited Research Inventory & Declared-Total Population Register Validator (CORE #97).
(`parity_auditor/validators/research_inventory_validator.py`)

Enforces:
1. Research Inventory Artifact Presence & Structure (docs/research/RESEARCH_INVENTORY.md)
2. Schema matching canonical template table headers:
   - Normative Standards & Baseline Documents Inventory:
     `Standard / Baseline ID`, `Issuing Body`, `Title`, `Applicable Clauses`, `Obligation Category`, `Declared Total`, `Clause Citation`
   - Declared-Total Population Register:
     `Category`, `Standard ID`, `Target Metric / Obligation Count`, `Verification Mechanism`, `Public Clause Citation`
   - External Additions & Domain Extensions Registry:
     All external additions and domain extensions must be clause-cited.
3. Clause Citation Traceability Mandate:
   - Every declared standard, population entry, and external addition must carry a valid public clause citation.
   - Un-cited or speculative additions (e.g. empty, TBD, N/A, unknown, missing clause locators) are strictly prohibited.
4. Declared-Total Population Register Arithmetic:
   - Declared totals per standard/category must be strictly positive and consistent.
"""

import os
import re
from typing import List, Optional, Set

from .base import IValidator
from ..core.findings import Finding
from ..core.models import (
    NormativeStandard,
    PopulationRegisterEntry,
    ExternalAdditionEntry,
    ResearchInventoryDocument,
)
from ..core.workspace import WorkspaceRepository
from ..parsers.research_inventory import (
    ResearchInventoryParser,
    is_valid_public_clause_citation,
    parse_markdown_table_rows,
    normalize_header_key,
)


class ResearchInventoryValidator(IValidator):
    """Validator for Cited Research Inventory and Declared-Total Population Register."""

    def validate(self, repo: WorkspaceRepository) -> List[Finding]:
        findings: List[Finding] = []
        workspace_dir = repo.workspace_dir
        research_dir = os.path.join(workspace_dir, "docs", "research")
        inventory_file = os.path.join(research_dir, "RESEARCH_INVENTORY.md")

        # If in upstream compiler repo and file does not exist, pass cleanly
        if repo.is_upstream_compiler_repo() and not os.path.exists(inventory_file):
            return findings

        if not os.path.exists(inventory_file):
            if os.path.isdir(research_dir):
                # If research directory exists but inventory file is missing
                findings.append(
                    Finding(
                        rule_id="spec.research_inventory.missing_file",
                        message="Missing mandatory Cited Research Inventory at docs/research/RESEARCH_INVENTORY.md",
                        location="docs/research",
                        detail={"expected": "docs/research/RESEARCH_INVENTORY.md"},
                    )
                )
            return findings

        try:
            with open(inventory_file, "r", encoding="utf-8") as f:
                content = f.read()
        except Exception as e:
            findings.append(
                Finding(
                    rule_id="spec.research_inventory.read_error",
                    message=f"Failed to read docs/research/RESEARCH_INVENTORY.md: {e}",
                    location="docs/research/RESEARCH_INVENTORY.md",
                    detail={"error": str(e)},
                )
            )
            return findings

        parser = ResearchInventoryParser()
        doc = parser.parse(content)
        doc.filepath = "docs/research/RESEARCH_INVENTORY.md"
        doc_findings = self.validate_document(doc, "docs/research/RESEARCH_INVENTORY.md", content=content)
        findings.extend(doc_findings)

        return findings

    def validate_document(
        self,
        doc: ResearchInventoryDocument,
        rel_path: str = "docs/research/RESEARCH_INVENTORY.md",
        content: Optional[str] = None,
        is_template: bool = False,
    ) -> List[Finding]:
        """Validates a parsed ResearchInventoryDocument against schema and citation rules."""
        findings: List[Finding] = []

        # 1. Check Section 2: Normative Standards & Baseline Documents Inventory table
        if not doc.standards:
            findings.append(
                Finding(
                    rule_id="spec.research_inventory.missing_table",
                    message=f"{rel_path}: Missing or empty 'Normative Standards & Baseline Documents Inventory' table in Section 2.",
                    location=rel_path,
                    detail={"section": "2. Normative Standards & Baseline Documents Inventory"},
                )
            )

        # 2. Check Section 2 schema headers if raw content is provided
        if content:
            s2_match = re.search(
                r'##\s*2\.\s*(?:Normative\s+Standards(?:\s*&\s*Baseline\s+Documents\s+Inventory)?|Applicable\s+Regulatory\s*&\s*Domain\s+Standards\s+Baseline)(.*?)(?=\n##|\Z)',
                content,
                re.DOTALL | re.IGNORECASE,
            )
            if s2_match:
                headers, _ = parse_markdown_table_rows(s2_match.group(1))
                norm_headers = {normalize_header_key(h) for h in headers}
                required_std_headers = {
                    "standard_baseline_id": ["standard_baseline_id", "standard_id", "standard"],
                    "issuing_body": ["issuing_body", "issuing_body_sdo", "sdo", "organization"],
                    "title": ["title", "title_baseline", "document_title"],
                    "applicable_clauses": ["applicable_clauses", "applicable_clause", "clauses", "clause"],
                    "obligation_category": ["obligation_category", "category", "domain_category", "verification_scope"],
                    "declared_total": ["declared_total", "target_metric_obligation_count", "obligation_count", "total"],
                    "clause_citation": ["clause_citation", "public_clause_citation", "formal_clause_citation", "citation"],
                }
                for canon_name, aliases in required_std_headers.items():
                    if not any(alias in norm_headers for alias in aliases):
                        findings.append(
                            Finding(
                                rule_id="spec.research_inventory.schema_mismatch",
                                message=f"{rel_path}: Normative Standards table missing required column '{canon_name}'.",
                                location=rel_path,
                                detail={"missing_column": canon_name, "headers": headers},
                            )
                        )

        # 3. Check Section 3: Declared-Total Population Register
        if not doc.population_register and not is_template:
            findings.append(
                Finding(
                    rule_id="spec.research_inventory.missing_table",
                    message=f"{rel_path}: Missing or empty 'Declared-Total Population Register' table in Section 3.",
                    location=rel_path,
                    detail={"section": "3. Declared-Total Population Register"},
                )
            )

        # 4. Check Clause Citations across Normative Standards
        for i, std in enumerate(doc.standards):
            citation = std.clause_citation or std.applicable_clauses
            if not is_valid_public_clause_citation(citation, allow_template_tokens=is_template):
                findings.append(
                    Finding(
                        rule_id="spec.research_inventory.invalid_citation",
                        message=f"{rel_path}: Standard '{std.standard_id}' has invalid or missing public clause citation '{citation}'.",
                        location=rel_path,
                        detail={"standard_id": std.standard_id, "citation": citation, "row_index": i},
                    )
                )

        # 5. Check Clause Citations across Population Register Entries
        for i, entry in enumerate(doc.population_register):
            citation = entry.clause_citation
            if not is_valid_public_clause_citation(citation, allow_template_tokens=is_template):
                findings.append(
                    Finding(
                        rule_id="spec.research_inventory.invalid_citation",
                        message=f"{rel_path}: Population Register entry {entry.obligation_id or f'row {i+1}'} ({entry.category}) has invalid or missing public clause citation '{citation}'.",
                        location=rel_path,
                        detail={"obligation_id": entry.obligation_id, "category": entry.category, "citation": citation, "row_index": i},
                    )
                )

        # 6. Check Clause Citations across External Additions
        for i, ext in enumerate(doc.external_additions):
            citation = ext.clause_citation
            if not is_valid_public_clause_citation(citation, allow_template_tokens=is_template):
                findings.append(
                    Finding(
                        rule_id="spec.research_inventory.invalid_citation",
                        message=f"{rel_path}: External Addition {ext.extension_id or f'row {i+1}'} ('{ext.standard_id}') lacks authoritative public clause citation '{citation}'.",
                        location=rel_path,
                        detail={"extension_id": ext.extension_id, "standard_id": ext.standard_id, "citation": citation, "row_index": i},
                    )
                )

        # 7. Check Declared Totals Consistency
        totals_by_std = doc.get_totals_by_standard()
        for std_id, total in totals_by_std.items():
            if total <= 0:
                findings.append(
                    Finding(
                        rule_id="spec.research_inventory.total_mismatch",
                        message=f"{rel_path}: Standard '{std_id}' declares non-positive total obligations count {total}.",
                        location=rel_path,
                        detail={"standard_id": std_id, "declared_total": total},
                    )
                )

        # Cross-check Gap Analysis if present
        if doc.gap_analysis:
            declared_normative_raw = doc.gap_analysis.get("Declared Total Normative Obligations")
            if declared_normative_raw:
                m = re.search(r'\d+', str(declared_normative_raw))
                if m:
                    declared_total_val = int(m.group(0))
                    calculated_total = doc.get_total_declared_obligations()
                    if calculated_total > 0 and declared_total_val != calculated_total:
                        findings.append(
                            Finding(
                                rule_id="spec.research_inventory.total_mismatch",
                                message=f"{rel_path}: Normative Completeness table declares {declared_total_val} obligations, but standards table sums to {calculated_total}.",
                                location=rel_path,
                                detail={"declared_in_gap": declared_total_val, "sum_of_standards": calculated_total},
                            )
                        )

        return findings
