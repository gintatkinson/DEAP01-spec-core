r"""
Operational-to-Resource Allocation Quality Gate (Gate 24).

Enforces:
1. Dynamic Operational Activity Universe extraction:
   \Omega_{ops} = A_{ops} \cup \Phi_{lifecycle}
   from docs/conops/ (e.g. CONOPS.md, MISSION_INTENT.md).
2. Resource Implementation Universe extraction:
   R_{res} = F_{features} \cup D_{sysml\_actions}
   from docs/features/, docs/epics/, docs/user-stories/, docs/use-cases/, and SysML AST.
3. Theorem 1 (Zero Orphan Activities):
   O_{orphan} = { \omega \in \Omega_{ops} | \Pi_{alloc}(\omega) = \emptyset } = \emptyset
4. Theorem 2 (Zero Phantom Tags):
   P_{phantom} = { t \in T_{tags} | t \notin \Omega_{ops} } = \emptyset
5. Syntax parsing of '/// OperationalAllocation: [OA-XX, PhaseName, ...]' across Markdown and SysML.
6. Automated synthesis of OP_TO_RES_ALLOCATION_MATRIX.md.
"""

import os
import re
import sys
from typing import Dict, List, Optional, Set, Tuple, Any

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.workspace import WorkspaceRepository
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.workspace import WorkspaceRepository


def _normalize_oa_id(raw_id: str) -> str:
    """
    Normalize operational activity or phase identifier into canonical uppercase form.

    Examples:
        'OA-01' -> 'OA-01'
        'OA-1'  -> 'OA-01'
        'OA_01' -> 'OA-01'
        'OA_2'  -> 'OA-02'
        'Startup' -> 'STARTUP'
        'Phase 1: Startup' -> 'STARTUP'
    """
    raw = raw_id.strip()
    raw = re.sub(r'[*`_\[\]]', '', raw).strip()
    
    # Strip leading 'Phase N:' or 'Phase N -'
    m_phase_prefix = re.match(r'^Phase\s*\d*[:\-—\s]+\s*(.+)$', raw, re.IGNORECASE)
    if m_phase_prefix:
        raw = m_phase_prefix.group(1).strip()

    # Match OA or MET/METL identifier: OA-01, OA_1, OA-STARTUP, MET-01, METL-01
    m_oa = re.match(r'^(?:OA|MET|METL)[_-]?0*(\d+[a-zA-Z0-9_-]*)$', raw, re.IGNORECASE)
    if m_oa:
        suffix = m_oa.group(1)
        prefix = "MET" if raw.upper().startswith("MET") else "OA"
        if suffix.isdigit():
            return f"{prefix}-{int(suffix):02d}"
        return f"{prefix}-{suffix.upper()}"

    m_oa_named = re.match(r'^(?:OA|MET|METL)[_-]?([a-zA-Z0-9_-]+)$', raw, re.IGNORECASE)
    if m_oa_named:
        prefix = "MET" if raw.upper().startswith("MET") else "OA"
        return f"{prefix}-{m_oa_named.group(1).upper()}"

    # General phase/activity name normalization
    cleaned = re.sub(r'[^a-zA-Z0-9]', '', raw).upper()
    return cleaned if cleaned else raw.upper()


def _parse_allocation_tags(text: str) -> List[str]:
    """
    Parses '/// OperationalAllocation: [OA-XX, ...]' tags from Markdown or SysML doc comments.

    Supports:
        /// OperationalAllocation: [OA-01, Startup]
        doc /* /// OperationalAllocation: [OA-02] */
        /* /// OperationalAllocation: [OA-03, ActiveExecution] */
    """
    tags: List[str] = []
    # Pattern matching '/// OperationalAllocation: [ ... ]' or '/// OperationalAllocation: ...'
    pattern = re.compile(
        r'///\s*OperationalAllocation\s*:\s*\[([^\]]+)\]',
        re.IGNORECASE
    )
    for match in pattern.finditer(text):
        raw_list = match.group(1)
        for part in raw_list.split(","):
            token = part.strip()
            if token:
                norm = _normalize_oa_id(token)
                if norm and norm not in tags:
                    tags.append(norm)

    # Fallback for unbracketed single tag: '/// OperationalAllocation: OA-01'
    if not tags:
        pattern_single = re.compile(
            r'///\s*OperationalAllocation\s*:\s*([a-zA-Z0-9_\-]+)',
            re.IGNORECASE
        )
        for match in pattern_single.finditer(text):
            token = match.group(1).strip()
            if token:
                norm = _normalize_oa_id(token)
                if norm and norm not in tags:
                    tags.append(norm)

    return tags


class OperationalAllocationValidator(IValidator):
    """
    Gate 24: Enforces complete Operational-to-Resource Allocation parity.

    Asserts:
    - Zero Orphan Activities (Theorem 1: O_orphan = empty)
    - Zero Phantom Tags (Theorem 2: P_phantom = empty)
    """

    def extract_conops_universe(self, repo: WorkspaceRepository) -> Tuple[Dict[str, str], Dict[str, str]]:
        """
        Dynamically extracts operational activities and lifecycle phases from docs/conops/.

        Returns:
            Tuple of:
            - ops_universe: Dict mapping canonical_id -> display_name
            - ops_locations: Dict mapping canonical_id -> 'rel_path:lineno'
        """
        workspace_dir = repo.workspace_dir
        conops_dir = os.path.join(workspace_dir, "docs", "conops")
        if not os.path.isdir(conops_dir):
            return {}, {}

        ops_universe: Dict[str, str] = {}
        ops_locations: Dict[str, str] = {}

        conops_files: List[Tuple[str, str]] = []
        for root, _, files in os.walk(conops_dir):
            for f in sorted(files):
                if f.endswith(".md") and f != "README.md":
                    full_p = os.path.join(root, f)
                    rel_p = os.path.relpath(full_p, workspace_dir)
                    conops_files.append((full_p, rel_p))

        for full_path, rel_path in conops_files:
            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            lines = content.splitlines()
            in_phases_section = False
            in_activities_section = False

            for lineno_1idx, line in enumerate(lines, start=1):
                stripped = line.strip()
                if not stripped:
                    continue

                # Section tracking
                if stripped.startswith("#"):
                    header_lower = stripped.lower()
                    if "phase" in header_lower or "lifecycle" in header_lower or "mode" in header_lower:
                        in_phases_section = True
                        in_activities_section = False
                    elif "activit" in header_lower or "conops" in header_lower or "operation" in header_lower:
                        in_activities_section = True
                        in_phases_section = False
                    else:
                        in_phases_section = False
                        in_activities_section = False

                # 1. Parse markdown tables with Activity ID or Phase columns
                if stripped.startswith("|") and stripped.endswith("|"):
                    parts = [p.strip() for p in stripped.split("|")[1:-1]]
                    if len(parts) >= 2:
                        first_col = parts[0]
                        second_col = parts[1]
                        # Check if first column is an OA or MET identifier: `OA-01`, OA_1, MET-01, etc.
                        m_oa = re.search(r'\b((?:OA|MET|METL)[_-]?\d+[a-zA-Z0-9_-]*)\b', first_col, re.IGNORECASE)
                        if m_oa:
                            raw_id = m_oa.group(1)
                            canon = _normalize_oa_id(raw_id)
                            display_name = re.sub(r'[*`_]', '', second_col).strip() or raw_id
                            ops_universe[canon] = display_name
                            if canon not in ops_locations:
                                ops_locations[canon] = f"{rel_path}:{lineno_1idx}"
                            continue

                        # Check if table row defines a phase name
                        if in_phases_section or "phase" in first_col.lower():
                            clean_name = re.sub(r'[*`_]', '', first_col).strip()
                            if clean_name and not clean_name.lower().startswith("phase id") and not clean_name.lower().startswith("---"):
                                canon = _normalize_oa_id(clean_name)
                                if canon and canon not in ("PHASE", "NAME", "ID", "DESCRIPTION"):
                                    ops_universe[canon] = clean_name
                                    if canon not in ops_locations:
                                        ops_locations[canon] = f"{rel_path}:{lineno_1idx}"
                            continue

                # 2. Parse bullet points / headings:
                # - **OA-01**: Sensor calibration ...
                # - **Phase 1: Startup**
                # ### OA-01: Initialization
                # ### Phase 1: Startup
                oa_match = re.search(r'\b((?:OA|MET|METL)[_-]?\d+[a-zA-Z0-9_-]*)\b', stripped, re.IGNORECASE)
                if oa_match:
                    raw_id = oa_match.group(1)
                    canon = _normalize_oa_id(raw_id)
                    # Extract name after colon or dash if present
                    m_desc = re.search(r'(?:(?:OA|MET|METL)[_-]?\d+[a-zA-Z0-9_-]*)[*`_]*\s*[:\-—]\s*(.+)$', stripped, re.IGNORECASE)
                    display_name = m_desc.group(1).strip() if m_desc else raw_id
                    display_name = re.sub(r'[*`_]', '', display_name).strip()
                    ops_universe[canon] = display_name
                    if canon not in ops_locations:
                        ops_locations[canon] = f"{rel_path}:{lineno_1idx}"
                    continue

                # Parse Phase declarations: - **Phase 1: Startup** or ### Phase 1: Startup
                phase_match = re.search(r'\bPhase\s*\d*[:\-—\s]+\s*([a-zA-Z0-9_-]+)', stripped, re.IGNORECASE)
                if phase_match:
                    phase_name = phase_match.group(1).strip()
                    phase_name = re.sub(r'[*`_]', '', phase_name).strip()
                    if phase_name:
                        canon = _normalize_oa_id(phase_name)
                        ops_universe[canon] = phase_name
                        if canon not in ops_locations:
                            ops_locations[canon] = f"{rel_path}:{lineno_1idx}"
                    continue

                # If inside explicit phases section, check list items: - **Startup** or - Startup:
                if in_phases_section and (stripped.startswith("-") or stripped.startswith("*")):
                    item_text = re.sub(r'^[-*]\s*', '', stripped).strip()
                    m_token = re.match(r'^\*{0,2}([a-zA-Z0-9_-]+)\*{0,2}(?:\s*[:\-—]|$)', item_text)
                    if m_token:
                        pname = m_token.group(1).strip()
                        if pname and len(pname) > 2 and pname.lower() not in ("phase", "phases", "lifecycle", "table", "the", "all"):
                            canon = _normalize_oa_id(pname)
                            ops_universe[canon] = pname
                            if canon not in ops_locations:
                                ops_locations[canon] = f"{rel_path}:{lineno_1idx}"

        return ops_universe, ops_locations

    def extract_allocation_tags(self, repo: WorkspaceRepository) -> Tuple[Dict[str, List[str]], Dict[str, List[str]]]:
        """
        Extracts all '/// OperationalAllocation: [...]' tags across specifications and SysML AST.

        Returns:
            Tuple of:
            - allocated_tags: Dict mapping canonical_id -> list of location strings ('rel_path:lineno')
            - raw_tag_map: Dict mapping raw tag token -> list of location strings
        """
        workspace_dir = repo.workspace_dir
        allocated_tags: Dict[str, List[str]] = {}
        raw_tag_map: Dict[str, List[str]] = {}

        rules = repo.get_codebase_rules()
        backlog = rules.backlog_directories if rules else None

        scan_dirs = ["docs/features", "docs/epics", "docs/user-stories", "docs/use-cases", "docs/designs", "schema"]
        if backlog:
            for attr in ("epics", "features", "user_stories", "use_cases", "schemas"):
                rel = getattr(backlog, attr, None)
                if rel and rel not in scan_dirs:
                    scan_dirs.append(rel)

        target_files: List[Tuple[str, str]] = []
        for sdir in scan_dirs:
            full_dir = os.path.join(workspace_dir, sdir)
            if not os.path.isdir(full_dir):
                continue
            for root, _, files in os.walk(full_dir):
                for f in sorted(files):
                    if (f.endswith(".md") or f.endswith(".sysml")) and f != "README.md":
                        full_p = os.path.join(root, f)
                        rel_p = os.path.relpath(full_p, workspace_dir)
                        target_files.append((full_p, rel_p))

        # Check .pipeline/schema.sysml
        pipeline_sysml = os.path.join(workspace_dir, ".pipeline", "schema.sysml")
        if os.path.isfile(pipeline_sysml):
            target_files.append((pipeline_sysml, ".pipeline/schema.sysml"))

        for full_path, rel_path in target_files:
            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            lines = content.splitlines()
            for lineno_1idx, line in enumerate(lines, start=1):
                if "OperationalAllocation" in line:
                    tags = _parse_allocation_tags(line)
                    loc = f"{rel_path}:{lineno_1idx}"
                    for t in tags:
                        allocated_tags.setdefault(t, []).append(loc)
                        raw_tag_map.setdefault(t, []).append(loc)

        return allocated_tags, raw_tag_map

    def _has_feature_specifications(self, repo: WorkspaceRepository) -> bool:
        """
        Determines whether the workspace has authored feature specifications.
        """
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
            try:
                feat_files = repo.get_feature_files(features_dir)
                if feat_files:
                    return True
            except Exception:
                pass

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

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        r"""
        Verifies Operational-to-Resource Allocation mathematical theorems:
        - Theorem 1 (Zero Orphan Activities): O_orphan = \Omega_{ops} \setminus T_{tags} = \emptyset
        - Theorem 2 (Zero Phantom Tags): P_phantom = T_{tags} \setminus \Omega_{ops} = \emptyset
        """
        ops_universe, ops_locations = self.extract_conops_universe(repo)
        allocated_tags, _ = self.extract_allocation_tags(repo)

        # Upstream clean landing zones: if both are empty, pass gracefully
        if not ops_universe and not allocated_tags:
            return []

        allow_missing_specs: bool = kwargs.get("allow_missing_specs", False)
        has_features: bool = self._has_feature_specifications(repo)

        # Stage-awareness: if allow_missing_specs is True or no feature specifications
        # have been authored yet in the workspace (Phase 1 pre-feature / ConOps lifecycle stage),
        # suppress orphan-allocation findings for downstream features/tasks (Theorem 1).
        skip_orphan_check = allow_missing_specs or not has_features

        findings: List[Finding] = []

        # Theorem 1: Zero Orphan Activities (enforced when not in pre-feature stage)
        if not skip_orphan_check:
            orphan_activities = sorted(set(ops_universe.keys()) - set(allocated_tags.keys()))
            for orphan in orphan_activities:
                loc = ops_locations.get(orphan, "docs/conops/CONOPS.md")
                disp_name = ops_universe.get(orphan, orphan)
                findings.append(Finding(
                    "operational-allocation-orphan-activity",
                    f"Operational activity or lifecycle phase '{disp_name}' ({orphan}) defined at {loc} has zero allocated resources (Theorem 1 violation: O_orphan != ∅). Missing '/// OperationalAllocation: [{orphan}]' tag in specifications or SysML model.",
                    location=loc,
                    detail={"orphan_id": orphan, "display_name": disp_name, "location": loc}
                ))

        # Theorem 2: Zero Phantom Tags (strictly enforced)
        phantom_tags = sorted(set(allocated_tags.keys()) - set(ops_universe.keys()))
        for phantom in phantom_tags:
            locs = allocated_tags.get(phantom, ["unknown"])
            for loc in locs:
                findings.append(Finding(
                    "operational-allocation-phantom-tag",
                    f"Allocation tag '{phantom}' at {loc} references undeclared operational activity or lifecycle phase not found in CONOPS baseline (Theorem 2 violation: P_phantom != ∅).",
                    location=loc,
                    detail={"phantom_tag": phantom, "location": loc}
                ))

        return findings

    def synthesize_allocation_matrix(self, repo: WorkspaceRepository) -> str:
        """
        Generates markdown content for docs/conops/OP_TO_RES_ALLOCATION_MATRIX.md.
        """
        ops_universe, ops_locations = self.extract_conops_universe(repo)
        allocated_tags, _ = self.extract_allocation_tags(repo)

        total_ops = len(ops_universe)
        allocated_count = sum(1 for op_id in ops_universe if op_id in allocated_tags)
        coverage_pct = (allocated_count / total_ops * 100.0) if total_ops > 0 else 100.0

        lines: List[str] = [
            "| **Attribute** | **Value** |",
            "| :--- | :--- |",
            "| **Document Title** | OMG UAF Operational-to-Resource Allocation Matrix (Op-to-Res) |",
            "| **Document ID** | UAF-OP-RES-MATRIX-001 |",
            "| **Standard Alignment** | OMG UAF v1.2 / v2.0 & ISO/IEC/IEEE 15288:2023 §6.4.2–§6.4.9 |",
            "| **Quality Gate** | Gate 24 (OperationalAllocationValidator) |",
            f"| **Allocation Coverage** | {coverage_pct:.1f}% ({allocated_count}/{total_ops}) |",
            "",
            "# Operational-to-Resource Allocation Matrix (Op-to-Res)",
            "",
            "## 1. Executive Summary & Mathematical Formalism",
            "",
            "This document establishes the authoritative bidirectional allocation between the Operational Activity Universe ($\\Omega_{\\text{ops}}$) and Resource Implementation Universe ($R_{\\text{res}}$).",
            "",
            "- **Theorem 1 (Zero Orphan Activities):**",
            "  $$O_{\\text{orphan}} = \\{ \\omega \\in \\Omega_{\\text{ops}} \\mid \\Pi_{\\text{alloc}}(\\omega) = \\emptyset \\} = \\emptyset$$",
            "- **Theorem 2 (Zero Phantom Tags):**",
            "  $$P_{\\text{phantom}} = \\{ t \\in T_{\\text{tags}} \\mid t \\notin \\Omega_{\\text{ops}} \\} = \\emptyset$$",
            "",
            "## 2. Operational-to-Resource Traceability Matrix",
            "",
            "| Operational Activity / Phase ID | Name / Description | Definition Location | Allocated Resources / Specifications | Status |",
            "| :--- | :--- | :--- | :--- | :--- |",
        ]

        for op_id in sorted(ops_universe.keys()):
            disp_name = ops_universe[op_id]
            def_loc = ops_locations.get(op_id, "docs/conops/CONOPS.md")
            alloc_locs = allocated_tags.get(op_id, [])
            if alloc_locs:
                alloc_str = "<br>".join(f"`{loc}`" for loc in alloc_locs)
                status_str = "✅ ALLOCATED"
            else:
                alloc_str = "*None (Orphan)*"
                status_str = "❌ UNALLOCATED"

            lines.append(f"| `{op_id}` | {disp_name} | `{def_loc}` | {alloc_str} | {status_str} |")

        lines.append("")
        return "\n".join(lines)


if __name__ == "__main__":
    repo = WorkspaceRepository()
    validator = OperationalAllocationValidator()
    errors = validator.validate(repo)
    if errors:
        for err in errors:
            print(f"[{getattr(err, 'rule_id', 'ERROR')}] {err}")
        sys.exit(1)
    else:
        print("[OK] Gate 24 (OperationalAllocationValidator): All operational-to-resource allocation checks passed.")
        sys.exit(0)
