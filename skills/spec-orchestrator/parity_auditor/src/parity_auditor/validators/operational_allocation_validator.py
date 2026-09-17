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
5. Theorem 3 (Zero Ungrounded Operational Activities):
   U_{ungrounded} = { \alpha \in A_{ops} | \sigma_{provenance}(\alpha) \notin \Sigma_{valid} } = \emptyset
6. Syntax parsing of '/// OperationalAllocation: [OA-XX, PhaseName, ...]' across Markdown and SysML.
7. Automated synthesis of OP_TO_RES_ALLOCATION_MATRIX.md.
"""

import json
import os
import re
import sys
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

try:
    _sysml_ast = load_sysml_ast_members(["SysMLPackage", "SysMLParser", "ActionDef"])
    SysMLPackage = _sysml_ast.SysMLPackage
    SysMLParser = _sysml_ast.SysMLParser
    ActionDef = _sysml_ast.ActionDef
except Exception:
    SysMLPackage = None
    SysMLParser = None
    ActionDef = None

CONOPS_SCHEMA_REL_PATH = os.path.join(".pipeline", "schemas", "conops_specification_schema.json")

DEFAULT_UAF_ACTIVITY_SCHEMA: Dict[str, Any] = {
    "type": "object",
    "required": ["id", "name", "description", "allocation_tag", "ssot_source"],
    "properties": {
        "id": {
            "type": "string",
            "description": "Unique activity identifier (e.g., OA-01).",
        },
        "name": {
            "type": "string",
            "description": "Formal activity name matching UAF operational activity definition.",
        },
        "description": {
            "type": "string",
            "description": "Technical description of the operational activity.",
        },
        "allocated_performer": {
            "type": "string",
            "description": "Operational performer or subsystem allocated to execute this activity.",
        },
        "ssot_source": {
            "type": "string",
            "minLength": 3,
            "description": "Authoritative Single Source of Truth citation (e.g., schema path, STANAG, MIL-STD, DO-178C, ISO standard).",
        },
        "allocation_tag": {
            "type": "string",
            "description": "Gate 24 allocation tag (e.g., /// OperationalAllocation: [OA-01]).",
        },
    },
    "additionalProperties": True,
}

GENERIC_PLACEHOLDERS: Set[str] = {
    "tbd", "none", "n/a", "na", "unknown", "unspecified",
    "placeholder", "-", "--", "---", "", "todo", "null", "undefined",
}


def load_uaf_activities_schema(workspace_dir: Optional[str] = None) -> Dict[str, Any]:
    """
    Ingests .pipeline/schemas/conops_specification_schema.json to load the schema
    contract for uaf_activities.
    """
    candidates = []
    if workspace_dir:
        candidates.append(os.path.join(workspace_dir, CONOPS_SCHEMA_REL_PATH))

    cur = os.path.dirname(os.path.abspath(__file__))
    while True:
        candidate = os.path.join(cur, CONOPS_SCHEMA_REL_PATH)
        candidates.append(candidate)
        parent = os.path.dirname(cur)
        if parent == cur:
            break
        cur = parent

    for cand in candidates:
        if os.path.isfile(cand):
            try:
                with open(cand, "r", encoding="utf-8") as f:
                    schema_data = json.load(f)
                items_schema = (
                    schema_data.get("properties", {})
                    .get("uaf_activities", {})
                    .get("items")
                )
                if isinstance(items_schema, dict):
                    return items_schema
            except Exception:
                pass

    return DEFAULT_UAF_ACTIVITY_SCHEMA


def _map_column_header_to_property(header: str) -> Optional[str]:
    """
    Maps markdown table column headers to schema property names:
    'id', 'name', 'allocated_performer', 'description', 'ssot_source', 'allocation_tag'.
    """
    clean = re.sub(r'[*`_#\[\]]', '', header).strip().lower()
    if not clean:
        return None

    # Check performer / allocated resource
    if any(k in clean for k in ("performer", "allocated node", "allocated resource", "subsystem")):
        return "allocated_performer"

    # Check SSOT / provenance / ground truth / source / citation
    if any(k in clean for k in ("ssot", "citation", "ground truth", "provenance", "source", "reference", "authority", "standard")):
        return "ssot_source"

    # Check allocation tag / traceability tag
    if any(k in clean for k in ("allocation tag", "gate 24", "allocation_tag", "tag")) or (
        "allocation" in clean and "performer" not in clean
    ):
        return "allocation_tag"

    # Check id (activity id, metl id, task id, phase id)
    if "id" in clean or clean in ("activity", "metl", "task", "phase"):
        return "id"

    # Check name / title
    if any(k in clean for k in ("name", "title")):
        return "name"

    # Check description / summary / purpose / scope
    if any(k in clean for k in ("desc", "summary", "purpose", "scope")):
        return "description"

    return None


def _clean_markdown(text: str) -> str:
    """
    Strips markdown formatting wrappers (backticks, bold/italics asterisks, brackets)
    while preserving meaningful internal characters like underscores in identifiers
    and paths (e.g. `schema/system_architecture.sysml` -> schema/system_architecture.sysml).
    """
    s = text.strip()
    # Strip enclosing backticks
    if s.startswith("`") and s.endswith("`") and len(s) >= 2:
        s = s[1:-1].strip()
    # Strip enclosing brackets
    if s.startswith("[") and s.endswith("]") and len(s) >= 2:
        s = s[1:-1].strip()
    # Strip enclosing bold / italics asterisks
    while (s.startswith("**") and s.endswith("**") and len(s) >= 4) or (
        s.startswith("*") and s.endswith("*") and len(s) >= 2
    ):
        if s.startswith("**") and s.endswith("**"):
            s = s[2:-2].strip()
        elif s.startswith("*") and s.endswith("*"):
            s = s[1:-1].strip()
    # Strip enclosing italics underscores if not part of identifier (e.g. _italic_)
    if s.startswith("_") and s.endswith("_") and len(s) >= 2 and s.count("_") == 2:
        s = s[1:-1].strip()
    return s.strip()


def _validate_activity_against_schema(
    activity: Dict[str, Any], schema: Dict[str, Any]
) -> List[str]:
    """
    Validates an extracted operational activity dictionary against the JSON schema contract.
    Checks:
    - Required fields are present and non-empty
    - Value types match property schema (e.g. string)
    - minLength constraints
    - Non-placeholder values (rejecting TBD, N/A, Unspecified, etc.)
    """
    errors: List[str] = []
    required_fields = schema.get("required", [])

    for req in required_fields:
        if req not in activity or activity[req] is None:
            errors.append(f"missing required property '{req}'")
        else:
            val = activity[req]
            if isinstance(val, str) and not val.strip():
                errors.append(f"required property '{req}' is empty")

    props = schema.get("properties", {})
    for key, val in activity.items():
        if key in ("location", "raw_row"):
            continue
        if key not in props:
            continue

        prop_schema = props[key]
        expected_type = prop_schema.get("type")

        if expected_type == "string":
            if not isinstance(val, str):
                errors.append(f"property '{key}' expected string, got {type(val).__name__}")
                continue

            clean_val = _clean_markdown(val)

            # Placeholder check for string fields
            if clean_val.lower() in GENERIC_PLACEHOLDERS:
                errors.append(f"property '{key}' contains placeholder value '{val}'")

            # minLength check
            min_len = prop_schema.get("minLength")
            if min_len is not None and len(clean_val) < min_len:
                errors.append(
                    f"property '{key}' length {len(clean_val)} is less than minLength {min_len}"
                )

    return errors


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
    raw = _clean_markdown(raw_id)
    
    # Strip leading 'Phase N:' or 'Phase N -' or 'Phase_Name'
    m_phase_prefix = re.match(r'^Phase\s*\d*[:\-\s_]+\s*(.+)$', raw, re.IGNORECASE)
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
    - Zero Ungrounded Operational Activities (Theorem 3: U_ungrounded = empty)
    """

    def __init__(self, workspace_dir: Optional[str] = None):
        self.workspace_dir = workspace_dir
        self.extracted_activities: Dict[str, Dict[str, Any]] = {}

    def extract_conops_universe(
        self, repo: WorkspaceRepository
    ) -> Tuple[Dict[str, str], Dict[str, str], Dict[str, Tuple[str, str]]]:
        """
        Dynamically extracts operational activities and lifecycle phases from docs/conops/.

        Returns:
            Tuple of:
            - ops_universe: Dict mapping canonical_id -> display_name
            - ops_locations: Dict mapping canonical_id -> 'rel_path:lineno'
            - ops_citations: Dict mapping canonical_id -> (citation_str, 'rel_path:lineno')
        """
        workspace_dir = repo.workspace_dir
        conops_dir = os.path.join(workspace_dir, "docs", "conops")
        if not os.path.isdir(conops_dir):
            return {}, {}, {}

        ops_universe: Dict[str, str] = {}
        ops_locations: Dict[str, str] = {}
        ops_citations: Dict[str, Tuple[str, str]] = {}
        self.extracted_activities.clear()

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
            in_sv5a_section = False
            in_table = False
            table_col_map: Dict[int, str] = {}

            for lineno_1idx, line in enumerate(lines, start=1):
                stripped = line.strip()
                if not stripped:
                    in_table = False
                    table_col_map = {}
                    continue

                # Section tracking
                if stripped.startswith("#"):
                    in_table = False
                    table_col_map = {}
                    header_lower = stripped.lower()
                    if re.search(r'(?:SV-5a|Op-to-Res|Operational-to-Resource(?:\s+Traceability)?\s+Allocation\s+Matrix)', stripped, re.IGNORECASE):
                        in_phases_section = False
                        in_activities_section = False
                        in_sv5a_section = True
                    elif "phase" in header_lower or "lifecycle" in header_lower or "mode" in header_lower:
                        in_phases_section = True
                        in_activities_section = False
                        in_sv5a_section = False
                    elif "activit" in header_lower or "conops" in header_lower or "operation" in header_lower:
                        in_activities_section = True
                        in_phases_section = False
                        in_sv5a_section = False
                    else:
                        in_phases_section = False
                        in_activities_section = False
                        in_sv5a_section = False

                if in_sv5a_section:
                    continue

                # 1. Parse markdown tables with structured column extraction
                if stripped.startswith("|") and stripped.endswith("|"):
                    parts = [p.strip() for p in stripped.split("|")[1:-1]]
                    # Check if separator row (| :--- | :--- |)
                    is_separator = all(re.match(r'^:?-+:?$', p) for p in parts if p)
                    if is_separator:
                        continue

                    if len(parts) >= 2:
                        first_col = parts[0]
                        # Check if first column is an OA or MET identifier: `OA-01`, OA_1, MET-01, etc.
                        m_oa = re.search(r'\b((?:OA|MET|METL)[_-]?\d+[a-zA-Z0-9_-]*)\b', first_col, re.IGNORECASE)

                        # Check if table header row defining columns
                        mapped_cols = {idx: _map_column_header_to_property(col) for idx, col in enumerate(parts)}
                        has_mapped_cols = any(mapped_cols.values())

                        is_header_row = (
                            not m_oa
                            and (
                                not in_table
                                or has_mapped_cols
                                or any(h in first_col.lower() for h in ("activity", "phase", "task", "metl", "lifecycle"))
                            )
                        )
                        if is_header_row:
                            in_table = True
                            table_col_map = mapped_cols
                            continue

                        in_table = True

                        # Build structured row dict mapping property names
                        row_dict: Dict[str, Any] = {}
                        for idx, prop_name in table_col_map.items():
                            if prop_name and idx < len(parts):
                                clean_val = _clean_markdown(parts[idx])
                                row_dict[prop_name] = clean_val

                        loc = f"{rel_path}:{lineno_1idx}"
                        row_dict["location"] = loc

                        if m_oa:
                            raw_id = m_oa.group(1)
                            canon = _normalize_oa_id(raw_id)
                            display_name = (
                                row_dict.get("name")
                                or (_clean_markdown(parts[1]) if len(parts) > 1 else raw_id)
                                or raw_id
                            )
                            row_dict["id"] = canon
                            row_dict.setdefault("name", display_name)
                            row_dict.setdefault("description", display_name)

                            # Default allocation_tag if not in table columns
                            if "allocation_tag" not in row_dict or not row_dict["allocation_tag"]:
                                row_dict["allocation_tag"] = f"/// OperationalAllocation: [{canon}]"

                            ops_universe[canon] = display_name
                            if canon not in ops_locations:
                                ops_locations[canon] = loc
                            if "ssot_source" in row_dict and row_dict["ssot_source"]:
                                ops_citations[canon] = (row_dict["ssot_source"], loc)

                            if canon in self.extracted_activities:
                                existing = self.extracted_activities[canon]
                                for k, v in existing.items():
                                    if (k not in row_dict or not row_dict[k]) and v:
                                        row_dict[k] = v
                            self.extracted_activities[canon] = row_dict
                            continue

                        # Check if table row defines a phase name
                        if in_phases_section or "phase" in first_col.lower():
                            clean_name = _clean_markdown(first_col)
                            if clean_name and not clean_name.lower().startswith("phase id") and not clean_name.lower().startswith("---"):
                                canon = _normalize_oa_id(clean_name)
                                if canon and canon not in ("PHASE", "NAME", "ID", "DESCRIPTION"):
                                    ops_universe[canon] = clean_name
                                    if canon not in ops_locations:
                                        ops_locations[canon] = f"{rel_path}:{lineno_1idx}"
                            continue
                else:
                    in_table = False
                    table_col_map = {}

                # 2. Parse bullet points / headings:
                oa_match = re.search(r'\b((?:OA|MET|METL)[_-]?\d+[a-zA-Z0-9_-]*)\b', stripped, re.IGNORECASE)
                if oa_match:
                    raw_id = oa_match.group(1)
                    canon = _normalize_oa_id(raw_id)
                    m_desc = re.search(r'(?:(?:OA|MET|METL)[_-]?\d+[a-zA-Z0-9_-]*)[*`_]*\s*[:\-]\s*(.+)$', stripped, re.IGNORECASE)
                    display_name = m_desc.group(1).strip() if m_desc else raw_id
                    display_name = _clean_markdown(display_name)
                    loc = f"{rel_path}:{lineno_1idx}"
                    ops_universe[canon] = display_name
                    if canon not in ops_locations:
                        ops_locations[canon] = loc
                    if canon not in self.extracted_activities:
                        self.extracted_activities[canon] = {
                            "id": canon,
                            "name": display_name,
                            "description": display_name,
                            "location": loc,
                            "allocation_tag": f"/// OperationalAllocation: [{canon}]",
                        }
                    continue

                # Parse Phase declarations: - **Phase 1: Startup** or ### Phase 1: Startup or Phase_Startup
                phase_match = re.search(r'\bPhase\s*\d*[:\-\s_]+\s*([a-zA-Z0-9_-]+)', stripped, re.IGNORECASE)
                if phase_match:
                    phase_name = phase_match.group(1).strip()
                    phase_name = _clean_markdown(phase_name)
                    if phase_name:
                        canon = _normalize_oa_id(phase_name)
                        ops_universe[canon] = phase_name
                        if canon not in ops_locations:
                            ops_locations[canon] = f"{rel_path}:{lineno_1idx}"
                    continue

                # If inside explicit phases section, check list items: - **Startup** or - Startup:
                if in_phases_section and (stripped.startswith("-") or stripped.startswith("*")):
                    item_text = re.sub(r'^[-*]\s*', '', stripped).strip()
                    m_token = re.match(r'^\*{0,2}([a-zA-Z0-9_-]+)\*{0,2}(?:\s*[:\-]|$)', item_text)
                    if m_token:
                        pname = m_token.group(1).strip()
                        if pname and len(pname) > 2 and pname.lower() not in ("phase", "phases", "lifecycle", "table", "the", "all"):
                            canon = _normalize_oa_id(pname)
                            ops_universe[canon] = pname
                            if canon not in ops_locations:
                                ops_locations[canon] = f"{rel_path}:{lineno_1idx}"

        return ops_universe, ops_locations, ops_citations

    def extract_sysml_actions(self, repo: WorkspaceRepository) -> List[Dict[str, Any]]:
        """
        Ingests SysML v2 AST action def and activity def nodes using load_sysml_ast_members
        when SysML models are present.
        """
        workspace_dir = repo.workspace_dir
        actions: List[Dict[str, Any]] = []

        scan_dirs = ["schema", ".pipeline"]
        sysml_files: List[Tuple[str, str]] = []
        for sdir in scan_dirs:
            full_dir = os.path.join(workspace_dir, sdir)
            if os.path.isdir(full_dir):
                for root, _, files in os.walk(full_dir):
                    for f in sorted(files):
                        if f.endswith(".sysml") and f != "README.md":
                            full_p = os.path.join(root, f)
                            rel_p = os.path.relpath(full_p, workspace_dir)
                            sysml_files.append((full_p, rel_p))

        for full_p, rel_p in sysml_files:
            try:
                with open(full_p, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            if not content.strip():
                continue

            # 1. AST-based extraction via SysMLParser
            if SysMLParser is not None:
                try:
                    pkg = SysMLParser.parse_text(
                        content,
                        default_name=os.path.splitext(os.path.basename(rel_p))[0],
                    )
                    self._collect_ast_actions(pkg, rel_p, actions)
                except Exception:
                    pass

            # 2. Direct extraction for activity def and action def declarations
            for m in re.finditer(
                r'(?:(?:doc\s*/\*|\s*/\*)(.*?)\*/\s*)?\b(action|activity)\s+(?:def\s+)?([a-zA-Z0-9_]+)',
                content,
                re.DOTALL | re.IGNORECASE,
            ):
                doc_comment = (m.group(1) or "").strip()
                kind = f"{m.group(2).lower()} def"
                act_name = m.group(3)
                if not any(a["name"] == act_name and a["file"] == rel_p for a in actions):
                    actions.append({
                        "name": act_name,
                        "syntax_form": kind,
                        "doc": doc_comment,
                        "location": f"{rel_p}:{act_name}",
                        "canonical_id": _normalize_oa_id(act_name),
                        "file": rel_p,
                    })

        return actions

    def _collect_ast_actions(
        self, pkg: Any, rel_path: str, actions: List[Dict[str, Any]]
    ):
        if not pkg:
            return

        def _add_action(act: Any):
            act_name = getattr(act, "name", "")
            if not act_name:
                return
            act_doc = getattr(act, "doc", "") or ""
            if not any(a["name"] == act_name and a["file"] == rel_path for a in actions):
                actions.append({
                    "name": act_name,
                    "syntax_form": "action def",
                    "doc": act_doc,
                    "location": f"{rel_path}:{act_name}",
                    "canonical_id": _normalize_oa_id(act_name),
                    "file": rel_path,
                })

        for act in (getattr(pkg, "action_defs", []) or []):
            _add_action(act)
        for act in (getattr(pkg, "actions", []) or []):
            _add_action(act)

        for part in (getattr(pkg, "part_defs", []) or []):
            for act in (getattr(part, "actions", []) or []):
                _add_action(act)

        for sub in (getattr(pkg, "sub_packages", []) or []):
            self._collect_ast_actions(sub, rel_path, actions)

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
                        if loc not in allocated_tags.get(t, []):
                            allocated_tags.setdefault(t, []).append(loc)
                        if loc not in raw_tag_map.get(t, []):
                            raw_tag_map.setdefault(t, []).append(loc)

        # Ingest SysML v2 AST action def and activity def nodes using load_sysml_ast_members
        sysml_actions = self.extract_sysml_actions(repo)
        for act in sysml_actions:
            loc = act["location"]
            canon_id = act["canonical_id"]
            if canon_id.startswith("OA-") or canon_id.startswith("MET-"):
                if loc not in allocated_tags.get(canon_id, []):
                    allocated_tags.setdefault(canon_id, []).append(loc)
                if loc not in raw_tag_map.get(canon_id, []):
                    raw_tag_map.setdefault(canon_id, []).append(loc)
            if act.get("doc"):
                for t in _parse_allocation_tags(act["doc"]):
                    existing = allocated_tags.get(t, [])
                    if not any(e.startswith(f"{act['file']}:") for e in existing):
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
        - Theorem 3 (Zero Ungrounded Operational Activities): U_ungrounded = \emptyset
        """
        ops_universe, ops_locations, _ = self.extract_conops_universe(repo)
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

        # Theorem 3: Zero Ungrounded Operational Activities (schema-driven validation)
        uaf_schema = load_uaf_activities_schema(repo.workspace_dir)
        for canon_id in sorted(ops_universe.keys()):
            if canon_id.startswith("OA-") or canon_id.startswith("MET-") or canon_id.startswith("METL-"):
                disp_name = ops_universe.get(canon_id, canon_id)
                loc = ops_locations.get(canon_id, "docs/conops/CONOPS.md")
                act_data = self.extracted_activities.get(canon_id)

                is_unanchored = False
                if not act_data:
                    is_unanchored = True
                else:
                    if "location" in act_data:
                        loc = act_data["location"]
                    schema_errors = _validate_activity_against_schema(act_data, uaf_schema)
                    if any("ssot_source" in err for err in schema_errors) or "ssot_source" not in act_data:
                        is_unanchored = True

                if is_unanchored:
                    findings.append(Finding(
                        "operational-activity-unanchored-provenance",
                        f"Operational activity '{disp_name}' ({canon_id}) at {loc} lacks valid SSOT ground truth citation (Theorem 3 violation: unanchored operational activity).",
                        location=loc,
                        detail={"activity_id": canon_id, "display_name": disp_name, "location": loc}
                    ))

        # DoDAF SV-5a / OMG UAF Op-to-Res Allocation Matrix Validation (Issue #347)
        findings.extend(self._validate_conops_sv5a_matrix(repo, ops_universe))

        return findings

    def _validate_conops_sv5a_matrix(
        self, repo: WorkspaceRepository, ops_universe: Dict[str, str]
    ) -> List[Finding]:
        r"""
        Validates DoDAF SV-5a / OMG UAF Op-to-Res Operational-to-Resource Allocation Matrix.

        Checks:
        1. Graceful pass when ops_universe is empty or lacks OA-* activities.
        2. Inspects markdown documentation under docs/conops/ (checking CONOPS.md,
           units/conops/06_UAF_OPERATIONAL_ACTIVITIES.md, 06_UAF_OPERATIONAL_ACTIVITIES.md,
           or any .md file in docs/conops/).
        3. If header matching r'(?:SV-5a|Op-to-Res|Operational-to-Resource(?:\s+Traceability)?\s+Allocation\s+Matrix)'
           is absent, emits 'operational-allocation-missing-sv5a-matrix'.
        4. If header is present, parses markdown table rows under that section and verifies
           that 100% of declared OA-* activities in ops_universe are mapped in the SV-5a table.
           If any declared OA-* activity is missing, emits 'operational-allocation-sv5a-incomplete'.
        """
        oa_activities = sorted([k for k in ops_universe.keys() if k.startswith("OA-")])
        if not oa_activities:
            return []

        workspace_dir = repo.workspace_dir
        conops_dir = os.path.join(workspace_dir, "docs", "conops")
        if not os.path.isdir(conops_dir):
            return []

        candidate_rel_paths = [
            os.path.join("docs", "conops", "CONOPS.md"),
            os.path.join("docs", "conops", "units", "conops", "06_UAF_OPERATIONAL_ACTIVITIES.md"),
            os.path.join("docs", "conops", "06_UAF_OPERATIONAL_ACTIVITIES.md"),
        ]

        target_full_path: Optional[str] = None
        target_rel_loc: Optional[str] = None

        for c_rel in candidate_rel_paths:
            full_p = os.path.join(workspace_dir, c_rel)
            if os.path.isfile(full_p):
                target_full_path = full_p
                target_rel_loc = c_rel.replace("\\", "/")
                break

        if target_full_path is None:
            for root, _, files in os.walk(conops_dir):
                for f in sorted(files):
                    if f.endswith(".md") and f != "README.md":
                        full_p = os.path.join(root, f)
                        target_full_path = full_p
                        target_rel_loc = os.path.relpath(full_p, workspace_dir).replace("\\", "/")
                        break
                if target_full_path:
                    break

        if target_full_path is None or target_rel_loc is None:
            target_rel_loc = "docs/conops/CONOPS.md"
            return [Finding(
                rule_id="operational-allocation-missing-sv5a-matrix",
                message=f"ConOps baseline at {target_rel_loc} lacks mandatory DoDAF SV-5a / OMG UAF Op-to-Res Operational-to-Resource Allocation Matrix.",
                location=target_rel_loc,
                detail={"rel_loc": target_rel_loc}
            )]

        try:
            with open(target_full_path, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
        except Exception:
            return [Finding(
                rule_id="operational-allocation-missing-sv5a-matrix",
                message=f"ConOps baseline at {target_rel_loc} lacks mandatory DoDAF SV-5a / OMG UAF Op-to-Res Operational-to-Resource Allocation Matrix.",
                location=target_rel_loc,
                detail={"rel_loc": target_rel_loc}
            )]

        header_pattern = re.compile(
            r'(?:SV-5a|Op-to-Res|Operational-to-Resource(?:\s+Traceability)?\s+Allocation\s+Matrix)',
            re.IGNORECASE
        )

        lines = content.splitlines()
        header_idx = -1
        header_line = ""
        for idx, line in enumerate(lines):
            stripped = line.strip()
            if stripped.startswith("#") and header_pattern.search(stripped):
                header_idx = idx
                header_line = stripped
                break

        if header_idx == -1:
            return [Finding(
                rule_id="operational-allocation-missing-sv5a-matrix",
                message=f"ConOps baseline at {target_rel_loc} lacks mandatory DoDAF SV-5a / OMG UAF Op-to-Res Operational-to-Resource Allocation Matrix.",
                location=target_rel_loc,
                detail={"rel_loc": target_rel_loc}
            )]

        m_head = re.match(r'^(#{1,6})\s+', header_line)
        header_level = len(m_head.group(1)) if m_head else 0

        section_lines = []
        for line in lines[header_idx + 1:]:
            stripped = line.strip()
            if stripped.startswith('#'):
                m_next = re.match(r'^(#{1,6})\s+(.*)$', stripped)
                if m_next:
                    next_level = len(m_next.group(1))
                    next_title = m_next.group(2).strip()
                    if header_level > 0:
                        if next_level < header_level:
                            break
                        if next_level == header_level:
                            m_sec = re.match(r'^(\d+(\.\d+)*)', next_title)
                            m_orig = re.match(r'^(?:#+\s*)?(\d+(\.\d+)*)', header_line.lstrip('#').strip())
                            if m_sec and m_orig and m_sec.group(1).startswith(m_orig.group(1) + "."):
                                pass
                            else:
                                break
                    elif next_level <= 2:
                        break
            section_lines.append(line)

        table_mapped_activities: Set[str] = set()
        for line in section_lines:
            stripped = line.strip()
            if stripped.startswith('|') and stripped.endswith('|'):
                parts = [p.strip() for p in stripped.split('|')[1:-1]]
                if all(re.match(r'^:?-+:?$', p) for p in parts if p):
                    continue
                if any(h in parts[0].lower() for h in ("activity id", "operational activity id", "activity", "oa id", "id")):
                    continue
                row_oas = re.findall(r'\b((?:OA)[_-]?\d+[a-zA-Z0-9_-]*)\b', stripped, re.IGNORECASE)
                for raw_oa in row_oas:
                    norm_oa = _normalize_oa_id(raw_oa)
                    if norm_oa:
                        table_mapped_activities.add(norm_oa)

        missing = sorted([oa for oa in oa_activities if oa not in table_mapped_activities])
        if missing:
            missing_list = ", ".join(missing)
            return [Finding(
                rule_id="operational-allocation-sv5a-incomplete",
                message=f"DoDAF SV-5a / UAF Op-to-Res Matrix at {target_rel_loc} is incomplete: missing operational activities [{missing_list}].",
                location=target_rel_loc,
                detail={
                    "rel_loc": target_rel_loc,
                    "missing_activities": missing,
                    "declared_activities": oa_activities,
                    "mapped_activities": sorted(table_mapped_activities),
                }
            )]

        return []

    def synthesize_allocation_matrix(self, repo: WorkspaceRepository) -> str:
        """
        Generates markdown content for docs/conops/OP_TO_RES_ALLOCATION_MATRIX.md.
        """
        ops_universe, ops_locations, _ = self.extract_conops_universe(repo)
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
            "- **Theorem 3 (Zero Ungrounded Operational Activities):**",
            "  $$U_{\\text{ungrounded}} = \\{ \\alpha \\in A_{\\text{ops}} \\mid \\sigma_{\\text{provenance}}(\\alpha) \\notin \\Sigma_{\\text{valid}} \\} = \\emptyset$$",
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
