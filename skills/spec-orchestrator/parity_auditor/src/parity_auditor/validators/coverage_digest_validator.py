r"""
Coverage-Digest Population Gate (Gate 28, closing #92).
(`parity_auditor/validators/coverage_digest_validator.py`)

Enforces:
1. Declared Total Obligations vs. Realized Specifications:
   Validates that all obligations declared in the Cited Research Inventory
   and Declared-Total Population Register (docs/research/RESEARCH_INVENTORY.md)
   are realized across specification artifacts (docs/features/, docs/epics/,
   docs/user-stories/, docs/use-cases/, docs/icd/, docs/conops/, docs/safety/, schema/).
2. Zero Phantom Realizations:
   Asserts that every realized obligation tag in specifications maps to an
   authoritative declared obligation in the population register:
       P_{phantom} = T_{realized} \setminus \Omega_{declared} = \emptyset
3. Population Metric & Coverage Digest Synthesis:
   Calculates and serializes the Coverage Digest tracking population metrics:
   - Total declared obligations vs realized count
   - Realization percentage by standard and category
   - Generates docs/research/COVERAGE_DIGEST.md
"""

import os
import re
import sys
from typing import Dict, List, Optional, Set, Tuple, Any

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.models import (
        CoverageDigest,
        ResearchInventoryDocument,
        PopulationRegisterEntry,
        ExternalAdditionEntry,
        NormativeStandard,
    )
    from ..core.workspace import WorkspaceRepository, extract_metadata_from_content
    from ..parsers.research_inventory import ResearchInventoryParser, parse_research_inventory
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.models import (
        CoverageDigest,
        ResearchInventoryDocument,
        PopulationRegisterEntry,
        ExternalAdditionEntry,
        NormativeStandard,
    )
    from parity_auditor.core.workspace import WorkspaceRepository, extract_metadata_from_content
    from parity_auditor.parsers.research_inventory import ResearchInventoryParser, parse_research_inventory


def _normalize_obligation_id(raw_id: str) -> str:
    """
    Normalize obligation identifier into canonical uppercase form.

    Examples:
        'OBL-01' -> 'OBL-01'
        'obl_1'  -> 'OBL-01'
        'SAF-1'  -> 'SAF-01'
        'EXT-02' -> 'EXT-02'
        'INT_01' -> 'INT-01'
    """
    raw = raw_id.strip()
    raw = re.sub(r'[*`_\[\]\'"]', '', raw).strip()

    # Pattern for prefix-number (e.g. OBL-01, SAF-1, EXT-2, INT-01, HAZ-01, REQ-01)
    m = re.match(r'^([A-Za-z]+)[_-]?0*(\d+[a-zA-Z0-9_-]*)$', raw)
    if m:
        prefix = m.group(1).upper()
        suffix = m.group(2)
        if suffix.isdigit():
            return f"{prefix}-{int(suffix):02d}"
        return f"{prefix}-{suffix.upper()}"

    # General alphanumeric normalization
    cleaned = re.sub(r'[^a-zA-Z0-9_-]', '', raw).upper()
    return cleaned if cleaned else raw.upper()


def _is_probable_obligation_token(token: str) -> bool:
    cleaned = re.sub(r'[*`_\[\]\'"]', '', token).strip()
    if "/" in cleaned or not cleaned:
        return False
    if re.match(r'^(?:OBL|SAF|INT|EXT|MET|CTL|HAZ|REQ|STD|NORM|DO|ARP|MIL)[-_]\w+', cleaned, re.IGNORECASE):
        return True
    if re.match(r'^(?:OBL|SAF|INT|EXT|MET|CTL|HAZ|REQ|STD|NORM)\d+', cleaned, re.IGNORECASE):
        return True
    return False


def _parse_obligation_tags(text: str) -> List[str]:
    """
    Parses obligation realization tags from Markdown specifications or doc comments.

    Supported syntax:
        /// ObligationAllocation: [OBL-01, SAF-01]
        /// ObligationWitness: [OBL-01]
        /// RealisesObligation: [OBL-01]
        /// Obligation: [OBL-01]
        /// Realises: [OBL-01]
        doc /* /// ObligationAllocation: [OBL-02] */
    """
    tags: List[str] = []

    # Tag patterns with brackets: (pattern, is_generic_realises)
    patterns = [
        (re.compile(r'///\s*ObligationAllocation\s*:\s*\[([^\]]+)\]', re.IGNORECASE), False),
        (re.compile(r'///\s*ObligationWitness\s*:\s*\[([^\]]+)\]', re.IGNORECASE), False),
        (re.compile(r'///\s*RealisesObligation\s*:\s*\[([^\]]+)\]', re.IGNORECASE), False),
        (re.compile(r'///\s*Obligation\s*:\s*\[([^\]]+)\]', re.IGNORECASE), False),
        (re.compile(r'///\s*Realises\s*:\s*\[([^\]]+)\]', re.IGNORECASE), True),
    ]

    for pat, is_generic_realises in patterns:
        for match in pat.finditer(text):
            raw_list = match.group(1)
            for part in raw_list.split(","):
                token = part.strip()
                if token:
                    if is_generic_realises and not _is_probable_obligation_token(token):
                        continue
                    norm = _normalize_obligation_id(token)
                    if norm and norm not in tags:
                        tags.append(norm)

    # Fallback for unbracketed single tags: /// ObligationAllocation: OBL-01
    single_patterns = [
        re.compile(r'///\s*ObligationAllocation\s*:\s*([A-Za-z0-9_-]+)', re.IGNORECASE),
        re.compile(r'///\s*Obligation\s*:\s*([A-Za-z0-9_-]+)', re.IGNORECASE),
        re.compile(r'///\s*RealisesObligation\s*:\s*([A-Za-z0-9_-]+)', re.IGNORECASE),
    ]
    for pat in single_patterns:
        for match in pat.finditer(text):
            token = match.group(1).strip()
            if token and not token.startswith("["):
                norm = _normalize_obligation_id(token)
                if norm and norm not in tags:
                    tags.append(norm)

    return tags


class CoverageDigestValidator(IValidator):
    """
    Validator for Coverage-Digest Population Gate (Gate 28 / #92).
    """

    def _has_feature_specifications(self, repo: WorkspaceRepository) -> bool:
        """Determines whether the workspace has authored feature specifications."""
        if hasattr(repo, "get_features") and callable(getattr(repo, "get_features")):
            try:
                feats = repo.get_features()
                if feats:
                    return True
            except Exception:
                pass

        workspace_dir = repo.workspace_dir
        rules = repo.get_codebase_rules()
        backlog = rules.backlog_directories if rules else None
        features_rel = getattr(backlog, "features", "docs/features") if backlog else "docs/features"
        features_dir = os.path.join(workspace_dir, features_rel)

        if os.path.isdir(features_dir):
            for root, _, files in os.walk(features_dir):
                for f in files:
                    if f.endswith(".md") and f != "README.md":
                        full_p = os.path.join(root, f)
                        try:
                            if os.path.getsize(full_p) > 0:
                                return True
                        except OSError:
                            pass

        return False

    def extract_declared_obligations_from_doc(
        self, doc: ResearchInventoryDocument
    ) -> Dict[str, Dict[str, Any]]:
        """
        Extracts declared obligation universe from a parsed ResearchInventoryDocument.

        Returns:
            Dict[obligation_id, {
                'standard_id': str,
                'category': str,
                'obligation_count': int,
                'clause_citation': str,
                'verification_mechanism': str,
                'source': str
            }]
        """
        declared: Dict[str, Dict[str, Any]] = {}

        # 1. Section 3: Declared-Total Population Register
        for i, entry in enumerate(doc.population_register):
            raw_id = entry.obligation_id or f"OBL-{i+1:02d}"
            canon_id = _normalize_obligation_id(raw_id)
            declared[canon_id] = {
                "standard_id": entry.standard_id,
                "category": entry.category or "Normative Obligation",
                "obligation_count": entry.obligation_count,
                "clause_citation": entry.clause_citation,
                "verification_mechanism": entry.verification_mechanism,
                "source": "population_register",
            }

        # 2. Section 4: External Additions & Domain Extensions Registry
        for i, ext in enumerate(doc.external_additions):
            raw_id = ext.extension_id or f"EXT-{i+1:02d}"
            canon_id = _normalize_obligation_id(raw_id)
            declared[canon_id] = {
                "standard_id": ext.standard_id,
                "category": ext.category or "Domain Extension",
                "obligation_count": ext.declared_total,
                "clause_citation": ext.clause_citation,
                "verification_mechanism": ext.verification_mechanism,
                "source": "external_addition",
            }

        # 3. Section 5: Clause-Level Allocation Matrix (if entries have population_id not yet seen)
        for i, alloc in enumerate(doc.clause_allocations):
            if alloc.population_id:
                canon_id = _normalize_obligation_id(alloc.population_id)
                if canon_id not in declared:
                    declared[canon_id] = {
                        "standard_id": alloc.standard_id,
                        "category": "Allocated Obligation",
                        "obligation_count": 1,
                        "clause_citation": alloc.clause_citation,
                        "verification_mechanism": "",
                        "source": "clause_allocation",
                    }

        # 4. Fallback if Section 3 and 4 were empty: create entries from Section 2 Standards
        if not declared and doc.standards:
            for i, std in enumerate(doc.standards):
                canon_id = _normalize_obligation_id(f"OBL-{i+1:02d}")
                declared[canon_id] = {
                    "standard_id": std.standard_id,
                    "category": std.obligation_category or "Normative Standard",
                    "obligation_count": std.declared_total,
                    "clause_citation": std.clause_citation or std.applicable_clauses,
                    "verification_mechanism": "",
                    "source": "standards_table",
                }

        return declared

    def extract_realized_specifications(
        self, repo: WorkspaceRepository, doc: Optional[ResearchInventoryDocument] = None
    ) -> Tuple[Dict[str, List[str]], Dict[str, List[str]]]:
        """
        Scans all specification documents and models in the workspace to extract
        obligation realization allocations.

        Returns:
            Tuple[realized_tags_map, raw_locations_map]
            where realized_tags_map is Dict[obligation_id, List[location_str]]
        """
        workspace_dir = repo.workspace_dir
        realized_map: Dict[str, List[str]] = {}
        raw_map: Dict[str, List[str]] = {}

        spec_dirs = [
            os.path.join(workspace_dir, "docs", "features"),
            os.path.join(workspace_dir, "docs", "epics"),
            os.path.join(workspace_dir, "docs", "user-stories"),
            os.path.join(workspace_dir, "docs", "use-cases"),
            os.path.join(workspace_dir, "docs", "conops"),
            os.path.join(workspace_dir, "docs", "icd"),
            os.path.join(workspace_dir, "docs", "safety"),
            os.path.join(workspace_dir, "schema"),
        ]

        # Scan files in spec directories
        for sdir in spec_dirs:
            if not os.path.isdir(sdir):
                continue
            for root, _, files in os.walk(sdir):
                for f in sorted(files):
                    if not (f.endswith(".md") or f.endswith(".sysml") or f.endswith(".yang")):
                        continue
                    filepath = os.path.join(root, f)
                    rel_path = os.path.relpath(filepath, workspace_dir)
                    try:
                        with open(filepath, "r", encoding="utf-8", errors="ignore") as file_obj:
                            content = file_obj.read()
                    except Exception:
                        continue

                    # 1. Frontmatter metadata check
                    if f.endswith(".md"):
                        fm = extract_metadata_from_content(content)
                        if fm:
                            for key in ("obligations", "normative_obligations", "allocated_obligations", "obligation_id"):
                                val = fm.get(key)
                                if isinstance(val, list):
                                    for item in val:
                                        norm = _normalize_obligation_id(str(item))
                                        if norm:
                                            loc = f"{rel_path}:1"
                                            realized_map.setdefault(norm, []).append(loc)
                                elif isinstance(val, str):
                                    norm = _normalize_obligation_id(val)
                                    if norm:
                                        loc = f"{rel_path}:1"
                                        realized_map.setdefault(norm, []).append(loc)

                    # 2. Tag check in content lines
                    for lineno_1idx, line in enumerate(content.splitlines(), start=1):
                        tags = _parse_obligation_tags(line)
                        loc = f"{rel_path}:{lineno_1idx}"
                        for t in tags:
                            realized_map.setdefault(t, []).append(loc)
                            raw_map.setdefault(t, []).append(loc)

        return realized_map, raw_map

    def build_coverage_digest(
        self, repo: WorkspaceRepository, doc: Optional[ResearchInventoryDocument] = None
    ) -> CoverageDigest:
        """
        Constructs a CoverageDigest capturing the population metrics of declared vs realized obligations.
        """
        workspace_dir = repo.workspace_dir
        if doc is None:
            inventory_path = os.path.join(workspace_dir, "docs", "research", "RESEARCH_INVENTORY.md")
            if os.path.isfile(inventory_path):
                try:
                    with open(inventory_path, "r", encoding="utf-8") as f:
                        doc = parse_research_inventory(f.read())
                except Exception:
                    doc = ResearchInventoryDocument()
            else:
                doc = ResearchInventoryDocument()

        declared_map = self.extract_declared_obligations_from_doc(doc)
        realized_map, _ = self.extract_realized_specifications(repo, doc=doc)

        declared_ids = set(declared_map.keys())
        realized_ids = set(k for k in realized_map.keys() if k in declared_ids)
        phantom_ids = sorted(set(realized_map.keys()) - declared_ids)
        unrealized_ids = sorted(declared_ids - realized_ids)

        # Totals by standard
        declared_by_std: Dict[str, int] = {}
        realized_by_std: Dict[str, int] = {}
        declared_by_cat: Dict[str, int] = {}
        realized_by_cat: Dict[str, int] = {}

        for ob_id, info in declared_map.items():
            std = info.get("standard_id", "Unknown Standard")
            cat = info.get("category", "Uncategorized")
            cnt = info.get("obligation_count", 1)

            declared_by_std[std] = declared_by_std.get(std, 0) + cnt
            declared_by_cat[cat] = declared_by_cat.get(cat, 0) + cnt

            if ob_id in realized_ids:
                realized_by_std[std] = realized_by_std.get(std, 0) + cnt
                realized_by_cat[cat] = realized_by_cat.get(cat, 0) + cnt
            else:
                realized_by_std.setdefault(std, 0)
                realized_by_cat.setdefault(cat, 0)

        total_declared = sum(info.get("obligation_count", 1) for info in declared_map.values())
        total_realized = sum(
            info.get("obligation_count", 1) for ob_id, info in declared_map.items() if ob_id in realized_ids
        )

        realization_pct = (total_realized / total_declared * 100.0) if total_declared > 0 else 100.0

        return CoverageDigest(
            total_declared_obligations=total_declared,
            total_realized_obligations=total_realized,
            realization_percentage=realization_pct,
            declared_by_standard=declared_by_std,
            realized_by_standard=realized_by_std,
            declared_by_category=declared_by_cat,
            realized_by_category=realized_by_cat,
            obligation_realization_map={k: v for k, v in realized_map.items() if k in declared_ids},
            unrealized_obligations=unrealized_ids,
            phantom_realizations=phantom_ids,
        )

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """
        Validates the declared population against realized specifications.
        """
        findings: List[Finding] = []
        workspace_dir = repo.workspace_dir
        research_dir = os.path.join(workspace_dir, "docs", "research")
        inventory_file = os.path.join(research_dir, "RESEARCH_INVENTORY.md")

        # Upstream compiler repository mode: pass cleanly if inventory is absent
        if repo.is_upstream_compiler_repo() and not os.path.exists(inventory_file):
            return findings

        if not os.path.exists(inventory_file):
            if os.path.isdir(research_dir):
                findings.append(
                    Finding(
                        rule_id="spec.coverage_digest.missing_inventory",
                        message="Missing mandatory Cited Research Inventory at docs/research/RESEARCH_INVENTORY.md for Coverage-Digest validation.",
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
                    rule_id="spec.coverage_digest.read_error",
                    message=f"Failed to read docs/research/RESEARCH_INVENTORY.md: {e}",
                    location="docs/research/RESEARCH_INVENTORY.md",
                    detail={"error": str(e)},
                )
            )
            return findings

        parser = ResearchInventoryParser()
        doc = parser.parse(content)

        declared_map = self.extract_declared_obligations_from_doc(doc)
        realized_map, _ = self.extract_realized_specifications(repo, doc=doc)

        allow_missing_specs: bool = kwargs.get("allow_missing_specs", False)
        has_features: bool = self._has_feature_specifications(repo)

        # 1. Theorem 2: Zero Phantom Realizations (strictly enforced across all lifecycle stages)
        declared_ids = set(declared_map.keys())
        phantom_ids = sorted(set(realized_map.keys()) - declared_ids)
        for phantom in phantom_ids:
            locs = realized_map.get(phantom, ["unknown"])
            for loc in locs:
                findings.append(
                    Finding(
                        rule_id="coverage-digest-phantom-obligation",
                        message=f"Realized specification at {loc} references undeclared obligation '{phantom}' not found in Declared-Total Population Register (Theorem 2 violation: P_phantom != ∅).",
                        location=loc,
                        detail={"phantom_id": phantom, "location": loc},
                    )
                )

        # 2. Obligation Realization Enforcement (Gate 28, Issue #110):
        alloc_target_map: Dict[str, str] = {}
        alloc_phase_map: Dict[str, str] = {}
        for alloc in doc.clause_allocations:
            if alloc.population_id:
                norm_p = _normalize_obligation_id(alloc.population_id)
                if alloc.downstream_spec_file:
                    alloc_target_map[norm_p] = alloc.downstream_spec_file.strip("`* ")
                if alloc.specification_phase:
                    alloc_phase_map[norm_p] = alloc.specification_phase.strip("`* ")

        realized_ids = set(k for k in realized_map.keys() if k in declared_ids)
        unrealized_ids = sorted(declared_ids - realized_ids)

        for un_id in unrealized_ids:
            info = declared_map.get(un_id, {})
            std = info.get("standard_id", "Unknown Standard")
            cat = info.get("category", "Normative Obligation")
            cit = info.get("clause_citation", "")
            target_spec = alloc_target_map.get(un_id, "")
            phase_spec = alloc_phase_map.get(un_id, "")

            target_abs = os.path.join(workspace_dir, target_spec) if target_spec else ""
            target_exists = bool(target_abs and os.path.exists(target_abs))
            is_conops_target = bool(
                target_spec and (
                    "CONOPS" in target_spec.upper()
                    or "MISSION_INTENT" in target_spec.upper()
                    or "Phase 1" in phase_spec
                )
            )

            # Enforce realization:
            # - When allow_missing_specs is False (strict mode: all declared obligations must be realized)
            # - When target spec file exists in workspace (completed specification target)
            # - When target is ConOps / Mission Intent / Phase 1 (mandatory structural phase)
            # - When feature specs exist and target is a feature spec
            should_enforce = (
                not allow_missing_specs
                or target_exists
                or is_conops_target
                or (has_features and target_spec.startswith("docs/features"))
            )

            if should_enforce:
                target_desc = target_spec if target_spec else "workspace specifications"
                findings.append(
                    Finding(
                        rule_id="coverage-digest-obligation-unrealized",
                        message=f"Declared obligation '{un_id}' ({cat}, Standard: {std}, Citation: '{cit}') allocated to '{target_desc}' has zero realized specifications in workspace.",
                        location="docs/research/RESEARCH_INVENTORY.md",
                        detail={"obligation_id": un_id, "standard_id": std, "category": cat, "target": target_desc},
                    )
                )

        return findings

    def synthesize_coverage_digest(self, repo: WorkspaceRepository) -> str:
        """
        Generates markdown content for docs/research/COVERAGE_DIGEST.md.
        """
        digest = self.build_coverage_digest(repo)
        inventory_path = os.path.join(repo.workspace_dir, "docs", "research", "RESEARCH_INVENTORY.md")
        doc = None
        if os.path.isfile(inventory_path):
            try:
                with open(inventory_path, "r", encoding="utf-8") as f:
                    doc = parse_research_inventory(f.read())
            except Exception:
                pass

        declared_map = self.extract_declared_obligations_from_doc(doc) if doc else {}

        lines: List[str] = [
            "| **Attribute** | **Value** |",
            "| :--- | :--- |",
            "| **Document Title** | Population Coverage Digest & Obligation Realization Tracking |",
            "| **Document ID** | DIGEST-OBL-POP-001 |",
            "| **Quality Gate** | Gate 28 (CoverageDigestValidator / CORE #92) |",
            f"| **Population Coverage** | {digest.realization_percentage:.1f}% ({digest.total_realized_obligations}/{digest.total_declared_obligations}) |",
            f"| **Status** | {'✅ FULLY REALIZED' if digest.is_fully_realized() else '⚠️ PARTIAL REALIZATION'} |",
            "",
            "# Population Coverage Digest & Obligation Realization Tracking",
            "",
            "## 1. Executive Summary & Population Metrics",
            "",
            digest.generate_markdown_summary(),
            "",
            "## 2. Realization Metrics by Normative Standard",
            "",
            "| Standard / Baseline ID | Declared Obligations | Realized Obligations | Coverage % | Status |",
            "| :--- | :--- | :--- | :--- | :--- |",
        ]

        for std in sorted(digest.declared_by_standard.keys()):
            decl = digest.declared_by_standard[std]
            real = digest.realized_by_standard.get(std, 0)
            pct = (real / decl * 100.0) if decl > 0 else 100.0
            status = "✅ 100%" if pct >= 100.0 else f"⚠️ {pct:.0f}%"
            lines.append(f"| {std} | {decl} | {real} | {pct:.1f}% | {status} |")

        lines.extend([
            "",
            "## 3. Obligation-to-Specification Realization Traceability Matrix",
            "",
            "| Obligation ID | Category | Standard ID | Realized Specification Files / Tags | Status |",
            "| :--- | :--- | :--- | :--- | :--- |",
        ])

        for ob_id in sorted(declared_map.keys()):
            info = declared_map[ob_id]
            cat = info.get("category", "")
            std = info.get("standard_id", "")
            alloc_locs = digest.obligation_realization_map.get(ob_id, [])
            if alloc_locs:
                alloc_str = "<br>".join(f"`{loc}`" for loc in alloc_locs)
                status_str = "✅ REALIZED"
            else:
                alloc_str = "*None (Unrealized)*"
                status_str = "❌ UNREALIZED"
            lines.append(f"| `{ob_id}` | {cat} | {std} | {alloc_str} | {status_str} |")

        lines.append("")
        return "\n".join(lines)


if __name__ == "__main__":
    repo = WorkspaceRepository()
    validator = CoverageDigestValidator()
    findings = validator.validate(repo)
    if findings:
        print(f"Coverage Digest Violations ({len(findings)}):")
        for f in findings:
            print(f"  - {f}")
        sys.exit(1)
    else:
        print("Success: Coverage Digest population gate passed.")
        sys.exit(0)
