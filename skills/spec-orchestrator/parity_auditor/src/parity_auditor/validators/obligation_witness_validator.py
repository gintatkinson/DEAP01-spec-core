r"""
Obligation-Witness Registry & Multi-Dimensional Verification Validator (Gate 29, closing #93).
(`parity_auditor/validators/obligation_witness_validator.py`)

Enforces:
1. Multi-Dimensional Obligation Witness Registry:
   Maintains and validates an authoritative registry of obligation witnesses across:
   - Specification Witnesses (W_{spec}): Features, Epics, User Stories, Use Cases, ICD, ConOps, SysML
   - Test Witnesses (W_{test}): Automated unit, integration, simulation, and hardware-in-the-loop tests
   - Code & Model Witnesses (W_{code} / W_{model}): Target source implementation files and executable SysML / discrete engines
2. Zero Phantom Witnesses:
   Asserts that all witness tags across specifications, test suites, and source codebases
   map to declared obligations in the Population Register:
       W_{phantom} = T_{witness} \setminus \Omega_{declared} = \emptyset
3. Stage-Aware Obligation Witness Completeness:
   Verifies that every declared obligation has valid witnesses across the active lifecycle stage:
   - Pre-feature / upstream compiler mode: passes gracefully without false alarms
   - Specification mode: enforces 100% Spec Witness coverage
   - Implementation / Verification mode: enforces full 3-way/4-way witness binding (Spec + Test + Code/Model)
4. Synthesis of Authoritative Registry Document:
   Generates docs/research/OBLIGATION_WITNESS_REGISTRY.md
"""

import os
import re
import sys
from typing import Dict, List, Optional, Set, Tuple, Any

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.models import (
        ObligationWitnessRecord,
        ObligationWitnessRegistry,
        ResearchInventoryDocument,
        PopulationRegisterEntry,
        ExternalAdditionEntry,
        NormativeStandard,
    )
    from ..core.workspace import WorkspaceRepository, extract_metadata_from_content
    from ..parsers.research_inventory import ResearchInventoryParser, parse_research_inventory
    from .coverage_digest_validator import _normalize_obligation_id, _parse_obligation_tags
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.models import (
        ObligationWitnessRecord,
        ObligationWitnessRegistry,
        ResearchInventoryDocument,
        PopulationRegisterEntry,
        ExternalAdditionEntry,
        NormativeStandard,
    )
    from parity_auditor.core.workspace import WorkspaceRepository, extract_metadata_from_content
    from parity_auditor.parsers.research_inventory import ResearchInventoryParser, parse_research_inventory
    from parity_auditor.validators.coverage_digest_validator import _normalize_obligation_id, _parse_obligation_tags


def _is_probable_obligation_token(token: str) -> bool:
    cleaned = re.sub(r'[*`_\[\]\'"]', '', token).strip()
    if "/" in cleaned or not cleaned:
        return False
    if re.match(r'^(?:OBL|SAF|INT|EXT|MET|CTL|HAZ|REQ|STD|NORM|DO|ARP|MIL)[-_]\w+', cleaned, re.IGNORECASE):
        return True
    if re.match(r'^(?:OBL|SAF|INT|EXT|MET|CTL|HAZ|REQ|STD|NORM)\d+', cleaned, re.IGNORECASE):
        return True
    return False


def _parse_witness_tags(text: str) -> List[str]:
    """
    Parses obligation witness tags from comments, docstrings, or code lines.

    Supported patterns:
        /// ObligationWitness: [OBL-01, SAF-01]
        /// TestWitness: [OBL-01]
        /// CodeWitness: [OBL-01]
        /// SpecWitness: [OBL-01]
        /// Realises: [OBL-01]
        /// Obligation: [OBL-01]
        @witness(OBL-01)
        # /// ObligationWitness: [OBL-01]
        /* /// ObligationWitness: [OBL-01] */
    """
    tags: List[str] = []

    # Bracketed patterns with (pattern, is_generic_realises)
    patterns = [
        (re.compile(r'///\s*(?:ObligationWitness|TestWitness|CodeWitness|SpecWitness|ModelWitness|ObligationAllocation|RealisesObligation|Obligation)\s*:\s*\[([^\]]+)\]', re.IGNORECASE), False),
        (re.compile(r'///\s*Realises\s*:\s*\[([^\]]+)\]', re.IGNORECASE), True),
        (re.compile(r'@witness\s*\(\s*([^\)]+)\s*\)', re.IGNORECASE), False),
        (re.compile(r'@obligation\s*\(\s*([^\)]+)\s*\)', re.IGNORECASE), False),
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

    # Unbracketed single tags: /// ObligationWitness: OBL-01
    single_patterns = [
        re.compile(r'///\s*(?:ObligationWitness|TestWitness|CodeWitness|SpecWitness|ModelWitness)\s*:\s*([A-Za-z0-9_-]+)', re.IGNORECASE),
        re.compile(r'///\s*Realises\s*:\s*([A-Za-z0-9_-]+)', re.IGNORECASE),
    ]
    for pat in single_patterns:
        for match in pat.finditer(text):
            token = match.group(1).strip()
            if token and not token.startswith("["):
                if not _is_probable_obligation_token(token):
                    continue
                norm = _normalize_obligation_id(token)
                if norm and norm not in tags:
                    tags.append(norm)

    return tags


class ObligationWitnessValidator(IValidator):
    """
    Validator for Obligation-Witness Registry (Gate 29 / #93).
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

    def build_witness_registry(
        self, repo: WorkspaceRepository, doc: Optional[ResearchInventoryDocument] = None
    ) -> ObligationWitnessRegistry:
        """
        Constructs the authoritative ObligationWitnessRegistry by scanning:
        1. Declared obligations in RESEARCH_INVENTORY.md
        2. Specification witnesses across docs/ and schema/
        3. Test witnesses across tests/ and test suites
        4. Code / Model witnesses across target codebase directories
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

        registry = ObligationWitnessRegistry()

        # 1. Initialize records from declared obligations
        # From Population Register
        for i, entry in enumerate(doc.population_register):
            raw_id = entry.obligation_id or f"OBL-{i+1:02d}"
            canon_id = _normalize_obligation_id(raw_id)
            registry.records[canon_id] = ObligationWitnessRecord(
                obligation_id=canon_id,
                standard_id=entry.standard_id,
                category=entry.category or "Normative Obligation",
                clause_citation=entry.clause_citation,
                verification_mechanism=entry.verification_mechanism,
            )

        # From External Additions
        for i, ext in enumerate(doc.external_additions):
            raw_id = ext.extension_id or f"EXT-{i+1:02d}"
            canon_id = _normalize_obligation_id(raw_id)
            if canon_id not in registry.records:
                registry.records[canon_id] = ObligationWitnessRecord(
                    obligation_id=canon_id,
                    standard_id=ext.standard_id,
                    category=ext.category or "Domain Extension",
                    clause_citation=ext.clause_citation,
                    verification_mechanism=ext.verification_mechanism,
                )

        # From Section 5 allocations
        for alloc in doc.clause_allocations:
            if alloc.population_id:
                canon_id = _normalize_obligation_id(alloc.population_id)
                if canon_id not in registry.records:
                    registry.records[canon_id] = ObligationWitnessRecord(
                        obligation_id=canon_id,
                        standard_id=alloc.standard_id,
                        category="Allocated Obligation",
                        clause_citation=alloc.clause_citation,
                    )

        # Fallback from Section 2 Standards if empty
        if not registry.records and doc.standards:
            for i, std in enumerate(doc.standards):
                canon_id = _normalize_obligation_id(f"OBL-{i+1:02d}")
                registry.records[canon_id] = ObligationWitnessRecord(
                    obligation_id=canon_id,
                    standard_id=std.standard_id,
                    category=std.obligation_category or "Normative Standard",
                    clause_citation=std.clause_citation or std.applicable_clauses,
                )

        declared_ids = set(registry.records.keys())

        # 2. Scan Specification Witnesses (W_spec)
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

                    # Metadata / frontmatter
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
                                            if norm in registry.records:
                                                registry.records[norm].spec_witnesses.append(loc)
                                            else:
                                                registry.phantom_witnesses.setdefault(norm, []).append(loc)
                                elif isinstance(val, str):
                                    norm = _normalize_obligation_id(val)
                                    if norm:
                                        loc = f"{rel_path}:1"
                                        if norm in registry.records:
                                            registry.records[norm].spec_witnesses.append(loc)
                                        else:
                                            registry.phantom_witnesses.setdefault(norm, []).append(loc)

                    # Content lines
                    for lineno_1idx, line in enumerate(content.splitlines(), start=1):
                        tags = _parse_witness_tags(line)
                        if not tags:
                            tags = _parse_obligation_tags(line)
                        loc = f"{rel_path}:{lineno_1idx}"
                        for t in tags:
                            if t in registry.records:
                                registry.records[t].spec_witnesses.append(loc)
                            else:
                                registry.phantom_witnesses.setdefault(t, []).append(loc)

        # 3. Scan Test Witnesses (W_test)
        test_dirs = [
            os.path.join(workspace_dir, "tests"),
            os.path.join(workspace_dir, "test"),
            os.path.join(workspace_dir, "app_flutter", "test"),
        ]

        test_exts = (".py", ".dart", ".ts", ".tsx", ".js")
        for tdir in test_dirs:
            if not os.path.isdir(tdir):
                continue
            for root, _, files in os.walk(tdir):
                for f in sorted(files):
                    if not any(f.endswith(ext) for ext in test_exts):
                        continue
                    filepath = os.path.join(root, f)
                    rel_path = os.path.relpath(filepath, workspace_dir)
                    try:
                        with open(filepath, "r", encoding="utf-8", errors="ignore") as file_obj:
                            content = file_obj.read()
                    except Exception:
                        continue

                    for lineno_1idx, line in enumerate(content.splitlines(), start=1):
                        tags = _parse_witness_tags(line)
                        loc = f"{rel_path}:{lineno_1idx}"
                        for t in tags:
                            if t in registry.records:
                                registry.records[t].test_witnesses.append(loc)
                            else:
                                registry.phantom_witnesses.setdefault(t, []).append(loc)

        # 4. Scan Code & Model Witnesses (W_code / W_model)
        code_dirs = [
            os.path.join(workspace_dir, "app_flutter", "lib"),
            os.path.join(workspace_dir, "src"),
            os.path.join(workspace_dir, "lib"),
            os.path.join(workspace_dir, "models"),
        ]

        code_exts = (".dart", ".py", ".ts", ".tsx", ".js", ".sysml", ".m")
        exclusions = {"node_modules", ".git", "build", ".dart_tool", "dist", ".pytest_cache"}

        for cdir in code_dirs:
            if not os.path.isdir(cdir):
                continue
            for root, dirs, files in os.walk(cdir):
                dirs[:] = [d for d in dirs if d not in exclusions]
                for f in sorted(files):
                    if not any(f.endswith(ext) for ext in code_exts):
                        continue
                    filepath = os.path.join(root, f)
                    rel_path = os.path.relpath(filepath, workspace_dir)
                    try:
                        with open(filepath, "r", encoding="utf-8", errors="ignore") as file_obj:
                            content = file_obj.read()
                    except Exception:
                        continue

                    is_model = "model" in rel_path.lower() or f.endswith(".sysml") or f.endswith(".m")

                    for lineno_1idx, line in enumerate(content.splitlines(), start=1):
                        tags = _parse_witness_tags(line)
                        loc = f"{rel_path}:{lineno_1idx}"
                        for t in tags:
                            if t in registry.records:
                                if is_model:
                                    registry.records[t].model_witnesses.append(loc)
                                else:
                                    registry.records[t].code_witnesses.append(loc)
                            else:
                                registry.phantom_witnesses.setdefault(t, []).append(loc)

        # Deduplicate witnesses per record
        for rec in registry.records.values():
            rec.spec_witnesses = sorted(list(dict.fromkeys(rec.spec_witnesses)))
            rec.test_witnesses = sorted(list(dict.fromkeys(rec.test_witnesses)))
            rec.code_witnesses = sorted(list(dict.fromkeys(rec.code_witnesses)))
            rec.model_witnesses = sorted(list(dict.fromkeys(rec.model_witnesses)))

        return registry

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """
        Validates obligation witness registry invariants across the workspace.
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
                        rule_id="spec.obligation_witness.missing_inventory",
                        message="Missing mandatory Cited Research Inventory at docs/research/RESEARCH_INVENTORY.md for Obligation-Witness validation.",
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
                    rule_id="spec.obligation_witness.read_error",
                    message=f"Failed to read docs/research/RESEARCH_INVENTORY.md: {e}",
                    location="docs/research/RESEARCH_INVENTORY.md",
                    detail={"error": str(e)},
                )
            )
            return findings

        parser = ResearchInventoryParser()
        doc = parser.parse(content)
        registry = self.build_witness_registry(repo, doc=doc)

        allow_missing_specs: bool = kwargs.get("allow_missing_specs", False)
        spec_only: bool = kwargs.get("spec_only", False)
        has_features: bool = self._has_feature_specifications(repo)
        has_codebase: bool = repo.has_configured_target_code_directories()

        # Build Section 5 allocation map
        alloc_target_map: Dict[str, str] = {}
        alloc_phase_map: Dict[str, str] = {}
        for alloc in doc.clause_allocations:
            if alloc.population_id:
                norm_p = _normalize_obligation_id(alloc.population_id)
                if alloc.downstream_spec_file:
                    alloc_target_map[norm_p] = alloc.downstream_spec_file.strip("`* ")
                if alloc.specification_phase:
                    alloc_phase_map[norm_p] = alloc.specification_phase.strip("`* ")

        # 1. Theorem 2: Zero Phantom Witnesses (strictly enforced across all files)
        for phantom, locs in registry.phantom_witnesses.items():
            for loc in locs:
                findings.append(
                    Finding(
                        rule_id="obligation-witness-phantom-obligation",
                        message=f"Witness tag at {loc} references undeclared obligation '{phantom}' not found in Declared-Total Population Register (Theorem 2 violation: W_phantom != ∅).",
                        location=loc,
                        detail={"phantom_id": phantom, "location": loc},
                    )
                )

        # 2. Obligation Witness Completeness (Gate 29, Issue #111):
        for ob_id, rec in sorted(registry.records.items()):
            target_spec = alloc_target_map.get(ob_id, "")
            phase_spec = alloc_phase_map.get(ob_id, "")
            target_abs = os.path.join(workspace_dir, target_spec) if target_spec else ""
            target_exists = bool(target_abs and os.path.exists(target_abs))
            is_conops_target = bool(
                (target_spec and ("CONOPS" in target_spec.upper() or "MISSION_INTENT" in target_spec.upper()))
                or ("PHASE 1" in phase_spec.upper() or "CONOPS" in phase_spec.upper() or "MISSION_INTENT" in phase_spec.upper())
            )
            is_feature_target = bool(
                (target_spec and target_spec.startswith("docs/features"))
                or ("PHASE 2" in phase_spec.upper() or "FEATURE" in phase_spec.upper() or "LOGICAL" in phase_spec.upper())
            )

            if target_spec:
                is_witnessed_in_assigned = any(
                    target_spec in w_loc or os.path.normpath(w_loc.split(":")[0]) == os.path.normpath(target_spec)
                    for w_loc in rec.spec_witnesses
                )
            else:
                is_witnessed_in_assigned = len(rec.spec_witnesses) > 0

            # Determine whether to enforce spec witness
            should_enforce_spec = (
                not allow_missing_specs
                or target_exists
                or is_conops_target
                or (has_features and is_feature_target)
                or len(rec.spec_witnesses) > 0
                or is_witnessed_in_assigned
            )

            if should_enforce_spec and not is_witnessed_in_assigned:
                loc = target_spec if target_exists else "docs/research/RESEARCH_INVENTORY.md"
                if len(rec.spec_witnesses) == 0:
                    msg = f"Declared obligation '{ob_id}' ({rec.category}, Standard: {rec.standard_id}) has zero specification witnesses in workspace."
                    if target_spec:
                        msg += f" (expected in '{target_spec}')"
                else:
                    msg = f"Declared obligation '{ob_id}' ({rec.category}, Standard: {rec.standard_id}) is assigned to '{target_spec}' in Section 5 but is not witnessed in that specification document."
                findings.append(
                    Finding(
                        rule_id="obligation-unwitnessed",
                        message=msg,
                        location=loc,
                        detail={"obligation_id": ob_id, "standard_id": rec.standard_id, "assigned_spec": target_spec},
                    )
                )

            # If in implementation / codebase mode and not spec-only: check test and code witnesses
            # Do NOT allow allow_missing_specs to bypass test/code witness checks on specifications
            # that exist in workspace, are ConOps/active targets, or have spec witnesses.
            if has_codebase and not spec_only and not repo.is_upstream_compiler_repo():
                should_enforce_test_code = (
                    not allow_missing_specs
                    or target_exists
                    or is_conops_target
                    or (has_features and is_feature_target)
                    or len(rec.spec_witnesses) > 0
                )
                if should_enforce_test_code:
                    if len(rec.test_witnesses) == 0:
                        findings.append(
                            Finding(
                                rule_id="obligation-witness-missing-test-witness",
                                message=f"Declared obligation '{ob_id}' ({rec.category}, Standard: {rec.standard_id}) has zero automated test witnesses in workspace test suite.",
                                location="docs/research/RESEARCH_INVENTORY.md",
                                detail={"obligation_id": ob_id, "standard_id": rec.standard_id},
                            )
                        )
                    if (len(rec.code_witnesses) + len(rec.model_witnesses)) == 0:
                        findings.append(
                            Finding(
                                rule_id="obligation-witness-missing-code-witness",
                                message=f"Declared obligation '{ob_id}' ({rec.category}, Standard: {rec.standard_id}) has zero implementation or discrete model witnesses in codebase.",
                                location="docs/research/RESEARCH_INVENTORY.md",
                                detail={"obligation_id": ob_id, "standard_id": rec.standard_id},
                            )
                        )

        return findings

    def synthesize_witness_registry(self, repo: WorkspaceRepository) -> str:
        """
        Generates markdown content for docs/research/OBLIGATION_WITNESS_REGISTRY.md.
        """
        registry = self.build_witness_registry(repo)
        coverage_pct = registry.witness_coverage_percentage()

        lines: List[str] = [
            "| **Attribute** | **Value** |",
            "| :--- | :--- |",
            "| **Document Title** | Multi-Dimensional Obligation-Witness Registry |",
            "| **Document ID** | REGISTRY-OBL-WITNESS-001 |",
            "| **Quality Gate** | Gate 29 (ObligationWitnessValidator / CORE #93) |",
            f"| **Witness Coverage** | {coverage_pct:.1f}% ({registry.total_witnessed()}/{registry.total_declared()}) |",
            f"| **Fully Verified** | {registry.total_fully_witnessed()}/{registry.total_declared()} Obligations |",
            "",
            "# Multi-Dimensional Obligation-Witness Registry",
            "",
            "## 1. Executive Summary & Verification Completeness",
            "",
            "| Metric Parameter | Value | Status |",
            "| :--- | :--- | :--- |",
            f"| Total Declared Obligations | {registry.total_declared()} | Baseline |",
            f"| Witnessed Obligations | {registry.total_witnessed()} | {'✅ 100% Complete' if registry.total_witnessed() == registry.total_declared() else '⚠️ Incomplete'} |",
            f"| Fully Witnessed (Spec + Test + Code) | {registry.total_fully_witnessed()} | {'✅ Complete' if registry.total_fully_witnessed() == registry.total_declared() else '⚠️ Partial'} |",
            f"| Witness Coverage Percentage | {coverage_pct:.1f}% | {'✅ Conforming' if coverage_pct >= 100.0 else '⚠️ Incomplete'} |",
            f"| Phantom Witness Tags | {len(registry.phantom_witnesses)} | {'✅ Zero (Conforming)' if len(registry.phantom_witnesses) == 0 else '❌ Non-Conforming'} |",
            "",
            "## 2. Multi-Dimensional Obligation Witness Traceability Matrix",
            "",
            "| Obligation ID | Category / Standard | Spec Witnesses ($W_{\\text{spec}}$) | Test Witnesses ($W_{\\text{test}}$) | Code/Model Witnesses ($W_{\\text{code}}$) | Status |",
            "| :--- | :--- | :--- | :--- | :--- | :--- |",
        ]

        for ob_id in sorted(registry.records.keys()):
            rec = registry.records[ob_id]
            spec_str = "<br>".join(f"`{w}`" for w in rec.spec_witnesses) if rec.spec_witnesses else "*None*"
            test_str = "<br>".join(f"`{w}`" for w in rec.test_witnesses) if rec.test_witnesses else "*None*"
            code_all = rec.code_witnesses + rec.model_witnesses
            code_str = "<br>".join(f"`{w}`" for w in code_all) if code_all else "*None*"

            if rec.is_fully_witnessed:
                status_str = "✅ FULLY WITNESSED"
            elif rec.is_witnessed:
                status_str = "⚠️ PARTIAL WITNESS"
            else:
                status_str = "❌ UNWITNESSED"

            lines.append(
                f"| `{ob_id}` | **{rec.category}**<br>{rec.standard_id} | {spec_str} | {test_str} | {code_str} | {status_str} |"
            )

        lines.append("")
        return "\n".join(lines)


if __name__ == "__main__":
    repo = WorkspaceRepository()
    validator = ObligationWitnessValidator()
    findings = validator.validate(repo)
    if findings:
        print(f"Obligation Witness Violations ({len(findings)}):")
        for f in findings:
            print(f"  - {f}")
        sys.exit(1)
    else:
        print("Success: Obligation Witness registry gate passed.")
        sys.exit(0)
