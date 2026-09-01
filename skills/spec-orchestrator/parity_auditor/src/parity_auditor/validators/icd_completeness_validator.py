"""
ICD Completeness & Signal Flow Parity Validator (Gate 23).

Enforces:
1. SysML v2 AST ports, connections, and item flow extraction from schema/ or .pipeline/schema.sysml.
2. Presence of Level 1C ICD artifacts in docs/interfaces/ (ICD_01_SYSTEM_INTERFACE_MATRIX.md, ICD_02_MASTER_SIGNAL_DICTIONARY.md).
3. Zero dangling ports (100% port connection parity for declared output ports).
4. Complete signal dictionary coverage of SysML item flows in ICD_02_MASTER_SIGNAL_DICTIONARY.md.
5. Signal dictionary contract integrity (valid port references, SI units, safe defaults).
"""

import os
import re
import sys
from dataclasses import dataclass, field
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

# Import SysML v2 AST classes via the fail-closed loader (refs #76): resolve
# the real scripts dir or raise ImportError — never bind None silently.
from ..utils.sysml_loader import load_sysml_ast_members

_sysml_ast = load_sysml_ast_members([
    "SysMLPackage", "SysMLParser", "PartDef", "PortDef", "ItemDef", "AttributeDef",
])
SysMLPackage = _sysml_ast.SysMLPackage
SysMLParser = _sysml_ast.SysMLParser
PartDef = _sysml_ast.PartDef
PortDef = _sysml_ast.PortDef
ItemDef = _sysml_ast.ItemDef
AttributeDef = _sysml_ast.AttributeDef


@dataclass
class SysMLPort:
    subsystem: str
    name: str
    direction: str  # "in", "out", "inout"
    type_name: str
    full_name: str = ""

    def __post_init__(self):
        if not self.full_name:
            if self.subsystem:
                self.full_name = f"{self.subsystem}.{self.name}"
            else:
                self.full_name = self.name


@dataclass
class SysMLConnection:
    name: str
    source_subsystem: str
    source_port: str
    dest_subsystem: str
    dest_port: str
    source_full: str
    dest_full: str


@dataclass
class SysMLItemFlow:
    name: str
    source_full: str
    dest_full: str


@dataclass
class SysMLModelElements:
    ports: List[SysMLPort] = field(default_factory=list)
    connections: List[SysMLConnection] = field(default_factory=list)
    item_flows: List[SysMLItemFlow] = field(default_factory=list)
    item_defs: List[str] = field(default_factory=list)
    port_defs: Dict[str, List[Tuple[str, str, str]]] = field(default_factory=dict)


@dataclass
class ICDPortRow:
    port_id: str
    subsystem: str
    port_name: str
    direction: str
    port_type: str
    multiplicity: str = "1"
    protocol_profile: str = ""


@dataclass
class ICDConnectionRow:
    connection_id: str
    source_port: str
    dest_port: str
    flow_behavior: str = ""
    latency_max_ms: str = ""
    reliability_req: str = ""
    item_flows_conveyed: List[str] = field(default_factory=list)


@dataclass
class ICDSignalRow:
    signal_id: str
    signal_name: str
    source_port: str
    dest_port: str
    data_type: str = ""
    si_units: str = ""
    valid_range: str = ""
    update_rate: str = ""
    safe_default_value: str = ""
    schema_citation: str = ""


def _strip_sysml_comments(text: str) -> str:
    """Strip block and line comments from SysML content."""
    text = re.sub(r'/\*.*?\*/', '', text, flags=re.DOTALL)
    text = re.sub(r'//.*$', '', text, flags=re.MULTILINE)
    return text


def _extract_sysml_elements(content: str) -> SysMLModelElements:
    """Extract ports, connections, item flows, and port/item definitions from SysML content."""
    clean_text = _strip_sysml_comments(content)
    elements = SysMLModelElements()

    # 1. Extract port defs: port def NavTelemetryPort { out item PrimaryVelocity : Float32; ... }
    port_def_blocks = re.finditer(r'\bport\s+def\s+([A-Za-z0-9_]+)\s*\{([^}]*)\}', clean_text)
    for pdb in port_def_blocks:
        p_def_name = pdb.group(1)
        p_def_body = pdb.group(2)
        items = []
        item_matches = re.finditer(
            r'\b(in|out|inout)?\s*(?:item|attribute)?\s*([A-Za-z0-9_]+)\s*:\s*([A-Za-z0-9_<>:]+)',
            p_def_body
        )
        for im in item_matches:
            i_dir = im.group(1) or "inout"
            i_name = im.group(2)
            i_type = im.group(3)
            items.append((i_name, i_dir.lower(), i_type))
            if i_name not in elements.item_defs:
                elements.item_defs.append(i_name)
        elements.port_defs[p_def_name] = items

    # 2. Extract item defs: item def Payload { ... }
    item_def_matches = re.finditer(r'\bitem\s+def\s+([A-Za-z0-9_]+)', clean_text)
    for idm in item_def_matches:
        iname = idm.group(1)
        if iname not in elements.item_defs:
            elements.item_defs.append(iname)

    # 3. Extract part defs and their ports:
    part_blocks = re.finditer(r'\bpart\s+(?:def\s+)?([A-Za-z0-9_]+)\s*\{([^}]*)\}', clean_text)
    for pb in part_blocks:
        part_name = pb.group(1)
        part_body = pb.group(2)
        port_matches = re.finditer(
            r'\b(in|out|inout)?\s*port\s+(?:def\s+)?([A-Za-z0-9_]+)\s*:\s*([A-Za-z0-9_<>:]+)',
            part_body
        )
        for pm in port_matches:
            direction = (pm.group(1) or "inout").lower()
            port_name = pm.group(2)
            port_type = pm.group(3)
            elements.ports.append(SysMLPort(
                subsystem=part_name,
                name=port_name,
                direction=direction,
                type_name=port_type
            ))

    # If no part ports were found, check for top-level port statements
    if not elements.ports:
        port_matches = re.finditer(
            r'\b(in|out|inout)?\s*port\s+([A-Za-z0-9_]+)\s*:\s*([A-Za-z0-9_<>:]+)',
            clean_text
        )
        for pm in port_matches:
            direction = (pm.group(1) or "inout").lower()
            port_name = pm.group(2)
            port_type = pm.group(3)
            elements.ports.append(SysMLPort(
                subsystem="",
                name=port_name,
                direction=direction,
                type_name=port_type
            ))

    # 4. Extract connections:
    conn_matches = re.finditer(
        r'(?:connection\s+([A-Za-z0-9_]+)\s+)?connect\s+([A-Za-z0-9_.]+)\s+to\s+([A-Za-z0-9_.]+)',
        clean_text
    )
    for cm in conn_matches:
        conn_name = cm.group(1) or ""
        src_full = cm.group(2).strip()
        dst_full = cm.group(3).strip()

        src_subsys, src_port = src_full.split(".", 1) if "." in src_full else ("", src_full)
        dst_subsys, dst_port = dst_full.split(".", 1) if "." in dst_full else ("", dst_full)

        elements.connections.append(SysMLConnection(
            name=conn_name,
            source_subsystem=src_subsys,
            source_port=src_port,
            dest_subsystem=dst_subsys,
            dest_port=dst_port,
            source_full=src_full,
            dest_full=dst_full
        ))

    # 5. Extract flows / item flows:
    flow_matches = re.finditer(
        r'\b(?:item\s+)?flow\b(?:\s+[A-Za-z0-9_]+)?\s+from\s+([A-Za-z0-9_.]+)\s+to\s+([A-Za-z0-9_.]+)\s+(?:item|of)\s+([A-Za-z0-9_]+)',
        clean_text
    )
    for fm in flow_matches:
        src_full = fm.group(1).strip()
        dst_full = fm.group(2).strip()
        item_name = fm.group(3).strip()
        elements.item_flows.append(SysMLItemFlow(
            name=item_name,
            source_full=src_full,
            dest_full=dst_full
        ))

    flow_of_matches = re.finditer(
        r'\b(?:item\s+)?flow\b(?:\s+[A-Za-z0-9_]+)?\s+of\s+([A-Za-z0-9_]+)\s+from\s+([A-Za-z0-9_.]+)\s+to\s+([A-Za-z0-9_.]+)',
        clean_text
    )
    for fm in flow_of_matches:
        item_name = fm.group(1).strip()
        src_full = fm.group(2).strip()
        dst_full = fm.group(3).strip()
        elements.item_flows.append(SysMLItemFlow(
            name=item_name,
            source_full=src_full,
            dest_full=dst_full
        ))

    return elements


@dataclass
class ParsedTable:
    raw_headers: List[str]
    clean_headers: List[str]
    norm_headers: List[str]
    rows: List[Dict[str, str]]
    raw_rows: List[List[str]]


def _parse_detailed_markdown_tables(content: str) -> List[ParsedTable]:
    """Parses markdown tables into a list of ParsedTable objects with header and row structure."""
    tables: List[ParsedTable] = []
    lines = content.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i].strip()
        if line.startswith("|") and line.endswith("|"):
            header_line = line
            if i + 1 < len(lines):
                sep_line = lines[i + 1].strip()
                if re.match(r"^\|(\s*:?-+:?\s*\|)+$", sep_line):
                    raw_headers = [col.strip() for col in header_line.split("|")[1:-1]]
                    clean_headers = [re.sub(r'[*`_]', '', col).strip() for col in raw_headers]
                    norm_headers = [re.sub(r'[\s\-/]+', '_', h.lower()).strip('_') for h in clean_headers]
                    rows: List[Dict[str, str]] = []
                    raw_rows: List[List[str]] = []
                    j = i + 2
                    while j < len(lines):
                        row_line = lines[j].strip()
                        if not (row_line.startswith("|") and row_line.endswith("|")):
                            break
                        cols = [col.strip() for col in row_line.split("|")[1:-1]]
                        raw_rows.append(cols)
                        row_dict: Dict[str, str] = {}
                        for idx, h in enumerate(norm_headers):
                            val = cols[idx] if idx < len(cols) else ""
                            clean_val = val.strip("`").strip()
                            row_dict[h] = clean_val
                        rows.append(row_dict)
                        j += 1
                    tables.append(ParsedTable(
                        raw_headers=raw_headers,
                        clean_headers=clean_headers,
                        norm_headers=norm_headers,
                        rows=rows,
                        raw_rows=raw_rows
                    ))
                    i = j
                    continue
        i += 1
    return tables


def _parse_markdown_tables(content: str) -> List[List[Dict[str, str]]]:
    """Parses markdown tables into a list of tables, where each table is a list of row dicts."""
    return [t.rows for t in _parse_detailed_markdown_tables(content)]



def _find_sysml_files(repo: WorkspaceRepository, schemas_dir: Optional[str] = None) -> List[str]:
    """Locates all SysML files in workspace."""
    sysml_files: List[str] = []

    # 1. Custom schemas_dir argument if provided
    if schemas_dir and os.path.exists(schemas_dir):
        if os.path.isfile(schemas_dir) and schemas_dir.endswith(".sysml"):
            sysml_files.append(schemas_dir)
        elif os.path.isdir(schemas_dir):
            for root, _, files in os.walk(schemas_dir):
                for f in sorted(files):
                    if f.endswith(".sysml") and not f.startswith("."):
                        sysml_files.append(os.path.join(root, f))

    # 2. Check standard schema directories in workspace
    for s_name in ("schema", "schemas"):
        cand = os.path.join(repo.workspace_dir, s_name)
        if os.path.isdir(cand):
            for root, _, files in os.walk(cand):
                for f in sorted(files):
                    if f.endswith(".sysml") and not f.startswith("."):
                        p = os.path.join(root, f)
                        if p not in sysml_files:
                            sysml_files.append(p)

    # 3. Check codebase rules configured directory
    try:
        rules = repo.get_codebase_rules()
        if rules and rules.backlog_directories and rules.backlog_directories.schemas:
            cand = os.path.join(repo.workspace_dir, rules.backlog_directories.schemas)
            if os.path.isdir(cand):
                for root, _, files in os.walk(cand):
                    for f in sorted(files):
                        if f.endswith(".sysml") and not f.startswith("."):
                            p = os.path.join(root, f)
                            if p not in sysml_files:
                                sysml_files.append(p)
    except Exception:
        pass

    # 4. Check .pipeline/schema.sysml
    pipeline_sysml = os.path.join(repo.workspace_dir, ".pipeline", "schema.sysml")
    if os.path.exists(pipeline_sysml) and pipeline_sysml not in sysml_files:
        sysml_files.append(pipeline_sysml)

    return sysml_files


def _normalize_subsys_token(token: str) -> str:
    """Strip markdown formatting, digits, brackets from subsystem token."""
    t = re.sub(r'[*`_\[\]]', '', token).strip()
    t = re.sub(r'^\d+\.\s*', '', t).strip()
    return t


def _normalize_type(t: str) -> str:
    """Normalize data type name to canonical lowercase string."""
    t_clean = t.strip().lower()
    if t_clean in ("bool", "boolean"):
        return "boolean"
    if t_clean in ("float32", "float", "single"):
        return "float32"
    if t_clean in ("float64", "double", "real"):
        return "float64"
    if t_clean in ("int32", "int", "integer"):
        return "int32"
    if t_clean in ("uint32", "uint", "unsignedint"):
        return "uint32"
    if t_clean in ("string", "str", "text"):
        return "string"
    return t_clean


PRIMITIVE_TYPE_GROUPS = {
    "boolean": {"bool", "boolean"},
    "float": {"float", "float32", "float64", "double", "single", "real"},
    "integer": {"int", "int8", "int16", "int32", "int64", "uint", "uint8", "uint16", "uint32", "uint64", "integer", "unsignedint", "natural", "positive"},
    "string": {"string", "str", "char", "text"},
}


def _are_types_incompatible(t1: str, t2: str) -> bool:
    """Check if two data types are incompatible primitives."""
    t1_norm = _normalize_type(t1)
    t2_norm = _normalize_type(t2)
    if not t1_norm or not t2_norm:
        return False
    if t1_norm == t2_norm:
        return False

    generic_types = {"dataport", "commandport", "telemetryport", "eventport", "port", "inout", "in", "out", "periodicstream", "realtimesync", "asyncevent"}
    if t1_norm in generic_types or t2_norm in generic_types:
        return False

    g1 = None
    g2 = None
    for group_name, members in PRIMITIVE_TYPE_GROUPS.items():
        if t1_norm in members:
            g1 = group_name
        if t2_norm in members:
            g2 = group_name

    if g1 is not None and g2 is not None:
        return g1 != g2

    if t1_norm in ("bool", "boolean") and t2_norm not in ("bool", "boolean"):
        return True
    if t2_norm in ("bool", "boolean") and t1_norm not in ("bool", "boolean"):
        return True

    return False


def _parse_rate_hz(val: str) -> Optional[float]:
    """Parse update frequency in Hz from a string."""
    if not val:
        return None
    val_clean = val.strip()
    m_khz = re.search(r'([0-9]+(?:\.[0-9]+)?)\s*khz\b', val_clean, re.IGNORECASE)
    if m_khz:
        return float(m_khz.group(1)) * 1000.0
    m = re.search(r'([0-9]+(?:\.[0-9]+)?)\s*hz\b', val_clean, re.IGNORECASE)
    if m:
        return float(m.group(1))
    return None


MANDATORY_SIGNAL_COLUMNS = {
    "Signal ID": ["signal_id", "sig_id", "id"],
    "Signal Name": ["signal_name", "name"],
    "Source Port": ["source_port", "src_port", "source"],
    "Dest Port": ["dest_port", "dst_port", "dest", "destination_port"],
    "Data Type": ["data_type", "type"],
    "SI Units": ["si_units", "units", "unit"],
    "Valid Range": ["valid_range", "range"],
    "Update Rate": ["update_rate", "rate", "frequency"],
    "Safe Default Value": ["safe_default_value", "safe_default", "safe_value", "fault_safe_value", "fault_safe", "default_value", "failsafe_value", "failsafe_default"],
    "Schema Citation": ["schema_citation", "citation", "schema_reference", "schema_ref", "source_reference", "source_citation", "provenance"],
}


class ICDCompletenessValidator(IValidator):
    """
    Gate 23: Enforces 100% topological port binding, zero dangling ports,
    and complete signal dictionary coverage for Level 1C ICD artifacts.
    """

    PORT_ROSTER_RE = re.compile(
        r"^\|\s*`?(PORT-[A-Za-z0-9_-]+)`?\s*\|\s*([A-Za-z0-9_-]+)\s*\|\s*([A-Za-z0-9_-]+)\s*\|\s*(IN|OUT|INOUT)\s*\|",
        re.MULTILINE | re.IGNORECASE
    )
    CONN_ROW_RE = re.compile(
        r"^\|\s*`?(CONN-[A-Za-z0-9_-]+)`?\s*\|\s*`?([A-Za-z0-9_.:-]+)`?\s*\|\s*`?([A-Za-z0-9_.:-]+)`?\s*\|",
        re.MULTILINE | re.IGNORECASE
    )
    SIGNAL_ROW_RE = re.compile(
        r"^\|\s*`?(SIG-[A-Za-z0-9_-]+)`?\s*\|\s*([A-Za-z0-9_]+)\s*\|\s*`?([A-Za-z0-9_.:-]+)`?\s*\|\s*`?([A-Za-z0-9_.:-]+)`?\s*\|\s*([A-Za-z0-9_]+)\s*\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*([^|]+)\|\s*`?([^|`]+)`?\s*\|",
        re.MULTILINE
    )

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        findings: List[Finding] = []
        schemas_dir = kwargs.get("schemas_dir")

        # 1. Discover and extract SysML elements
        sysml_files = _find_sysml_files(repo, schemas_dir)
        sysml_model = SysMLModelElements()

        for sf in sysml_files:
            try:
                with open(sf, "r", encoding="utf-8") as f:
                    content = f.read()
                elem = _extract_sysml_elements(content)
                sysml_model.ports.extend(elem.ports)
                sysml_model.connections.extend(elem.connections)
                sysml_model.item_flows.extend(elem.item_flows)
                for id_name in elem.item_defs:
                    if id_name not in sysml_model.item_defs:
                        sysml_model.item_defs.append(id_name)
                for p_def_name, items in elem.port_defs.items():
                    sysml_model.port_defs[p_def_name] = items
            except Exception:
                pass

        # 2. Check clean empty workspace / landing zone
        has_sysml_ports = len(sysml_model.ports) > 0 or len(sysml_model.port_defs) > 0
        has_sysml_connections = len(sysml_model.connections) > 0
        has_sysml_flows = len(sysml_model.item_flows) > 0

        icd01_path = os.path.join(repo.workspace_dir, "docs", "interfaces", "ICD_01_SYSTEM_INTERFACE_MATRIX.md")
        icd02_path = os.path.join(repo.workspace_dir, "docs", "interfaces", "ICD_02_MASTER_SIGNAL_DICTIONARY.md")

        if not has_sysml_ports and not has_sysml_connections and not has_sysml_flows:
            if not os.path.exists(icd01_path) and not os.path.exists(icd02_path):
                return []

        # 3. Check for presence of ICD suite artifacts
        missing_artifacts = []
        if not os.path.exists(icd01_path):
            missing_artifacts.append("docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md")
        if not os.path.exists(icd02_path):
            missing_artifacts.append("docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md")

        if missing_artifacts:
            if has_sysml_ports or has_sysml_connections or has_sysml_flows:
                for ma in missing_artifacts:
                    findings.append(Finding(
                        "icd-artifact-missing",
                        f"SysML model contains ports/connections, but required Level 1C ICD artifact '{ma}' is missing.",
                        location=ma,
                        detail={"missing_artifact": ma}
                    ))
            return findings

        # 4. Parse ICD_01 and ICD_02 Markdown contents
        with open(icd01_path, "r", encoding="utf-8") as f:
            icd01_text = f.read()
        with open(icd02_path, "r", encoding="utf-8") as f:
            icd02_text = f.read()

        icd01_tables_detailed = _parse_detailed_markdown_tables(icd01_text)
        icd02_tables_detailed = _parse_detailed_markdown_tables(icd02_text)

        icd01_ports: List[ICDPortRow] = []
        icd01_connections: List[ICDConnectionRow] = []
        icd02_signals: List[ICDSignalRow] = []

        # Extract from structured tables
        for tbl in icd01_tables_detailed:
            for row in tbl.rows:
                if "port_id" in row and "port_name" in row:
                    direction = row.get("direction", "INOUT").upper()
                    icd01_ports.append(ICDPortRow(
                        port_id=row["port_id"],
                        subsystem=row.get("subsystem", ""),
                        port_name=row["port_name"],
                        direction=direction,
                        port_type=row.get("port_type", ""),
                        multiplicity=row.get("multiplicity", "1"),
                        protocol_profile=row.get("protocol_profile", "")
                    ))
                elif "connection_id" in row and "source_port" in row and "dest_port" in row:
                    raw_flows = row.get("item_flows_conveyed", "") or row.get("flows_conveyed", "") or row.get("item_flows", "")
                    flows = [f.strip("`").strip() for f in raw_flows.split(",") if f.strip("`").strip()]
                    icd01_connections.append(ICDConnectionRow(
                        connection_id=row["connection_id"],
                        source_port=row["source_port"],
                        dest_port=row["dest_port"],
                        flow_behavior=row.get("flow_behavior", ""),
                        latency_max_ms=row.get("latency_max_ms", ""),
                        reliability_req=row.get("reliability_req", ""),
                        item_flows_conveyed=flows
                    ))

        for tbl in icd02_tables_detailed:
            for row in tbl.rows:
                if "signal_id" in row and "signal_name" in row:
                    safe_val = (
                        row.get("safe_default_value", "")
                        or row.get("safe_default", "")
                        or row.get("safe_value", "")
                        or row.get("fault_safe_value", "")
                        or row.get("default_value", "")
                        or row.get("failsafe_value", "")
                    )
                    schema_cite = (
                        row.get("schema_citation", "")
                        or row.get("citation", "")
                        or row.get("schema_reference", "")
                        or row.get("schema_ref", "")
                        or row.get("source_reference", "")
                        or row.get("source_citation", "")
                    )
                    icd02_signals.append(ICDSignalRow(
                        signal_id=row["signal_id"],
                        signal_name=row["signal_name"],
                        source_port=row.get("source_port", "") or row.get("src_port", "") or row.get("source", ""),
                        dest_port=row.get("dest_port", "") or row.get("dst_port", "") or row.get("dest", ""),
                        data_type=row.get("data_type", "") or row.get("type", ""),
                        si_units=row.get("si_units", "") or row.get("units", "") or row.get("unit", ""),
                        valid_range=row.get("valid_range", "") or row.get("range", ""),
                        update_rate=row.get("update_rate", "") or row.get("rate", "") or row.get("frequency", ""),
                        safe_default_value=safe_val,
                        schema_citation=schema_cite
                    ))

        # Fallback regex parsing if tables were empty
        if not icd01_ports:
            for m in self.PORT_ROSTER_RE.finditer(icd01_text):
                icd01_ports.append(ICDPortRow(
                    port_id=m.group(1),
                    subsystem=m.group(2),
                    port_name=m.group(3),
                    direction=m.group(4).upper(),
                    port_type=""
                ))

        if not icd01_connections:
            for m in self.CONN_ROW_RE.finditer(icd01_text):
                icd01_connections.append(ICDConnectionRow(
                    connection_id=m.group(1),
                    source_port=m.group(2),
                    dest_port=m.group(3)
                ))

        if not icd02_signals:
            for m in self.SIGNAL_ROW_RE.finditer(icd02_text):
                icd02_signals.append(ICDSignalRow(
                    signal_id=m.group(1),
                    signal_name=m.group(2),
                    source_port=m.group(3),
                    dest_port=m.group(4),
                    data_type=m.group(5),
                    si_units=m.group(6).strip(),
                    valid_range=m.group(7).strip(),
                    update_rate=m.group(8).strip(),
                    safe_default_value=m.group(9).strip(),
                    schema_citation=m.group(10).strip()
                ))

        # Build port lookup sets
        declared_port_ids = {p.port_id for p in icd01_ports}
        declared_port_names = {p.port_name for p in icd01_ports}
        declared_full_ports = {f"{p.subsystem}.{p.port_name}" for p in icd01_ports if p.subsystem}
        all_declared_tokens = declared_port_ids | declared_port_names | declared_full_ports

        bound_src_ports: Set[str] = set()
        bound_dst_ports: Set[str] = set()
        icd01_conveyed_flows: Set[str] = set()

        for c in icd01_connections:
            bound_src_ports.add(c.source_port)
            bound_dst_ports.add(c.dest_port)
            icd01_conveyed_flows.update(c.item_flows_conveyed)

        sysml_bound_src = {c.source_full for c in sysml_model.connections} | {c.source_port for c in sysml_model.connections}

        # 5. Validate Canonical N² Matrix in ICD_01
        declared_subsystems = sorted(list({
            p.subsystem for p in icd01_ports if p.subsystem
        } | {
            sp.subsystem for sp in sysml_model.ports if sp.subsystem
        } | {
            c.source_subsystem for c in sysml_model.connections if c.source_subsystem
        } | {
            c.dest_subsystem for c in sysml_model.connections if c.dest_subsystem
        }))

        if len(declared_subsystems) >= 2:
            n2_candidates = [
                t for t in icd01_tables_detailed
                if "port_id" not in t.norm_headers
                and "connection_id" not in t.norm_headers
                and not (len(t.norm_headers) <= 2 and "attribute" in t.norm_headers)
            ]
            if not n2_candidates:
                findings.append(Finding(
                    "icd-n2-matrix-malformed",
                    "Canonical N² Subsystem Interface Matrix table is missing in ICD_01_SYSTEM_INTERFACE_MATRIX.md.",
                    location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                    detail={"reason": "missing_n2_table", "subsystems": declared_subsystems}
                ))
            else:
                n2_tbl = n2_candidates[0]
                dest_headers = n2_tbl.clean_headers[1:]
                if any(not h.strip() for h in dest_headers):
                    findings.append(Finding(
                        "icd-n2-matrix-malformed",
                        "Canonical N² matrix contains an empty destination column header.",
                        location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                        detail={"reason": "empty_dest_header"}
                    ))
                norm_dest_headers = [_normalize_subsys_token(h) for h in dest_headers]
                for subsys in declared_subsystems:
                    if not any(subsys.lower() == dh.lower() or subsys.lower() in dh.lower() or dh.lower() in subsys.lower() for dh in norm_dest_headers if dh):
                        findings.append(Finding(
                            "icd-n2-matrix-malformed",
                            f"Canonical N² matrix is missing destination column header for subsystem '{subsys}'.",
                            location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                            detail={"missing_dest_header": subsys}
                        ))

                source_row_labels = [row[0] if len(row) > 0 else "" for row in n2_tbl.raw_rows]
                if any(not lbl.strip() for lbl in source_row_labels):
                    findings.append(Finding(
                        "icd-n2-matrix-malformed",
                        "Canonical N² matrix contains an empty source row header.",
                        location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                        detail={"reason": "empty_source_header"}
                    ))
                norm_source_labels = [_normalize_subsys_token(lbl) for lbl in source_row_labels]
                for subsys in declared_subsystems:
                    if not any(subsys.lower() == sl.lower() or subsys.lower() in sl.lower() or sl.lower() in subsys.lower() for sl in norm_source_labels if sl):
                        findings.append(Finding(
                            "icd-n2-matrix-malformed",
                            f"Canonical N² matrix is missing source row header for subsystem '{subsys}'.",
                            location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                            detail={"missing_source_header": subsys}
                        ))

        # 6. Validate Mandatory Columns in Signal Dictionary (ICD_02)
        sig_candidates = [
            t for t in icd02_tables_detailed
            if "signal_id" in t.norm_headers or "signal_name" in t.norm_headers or any("sig" in h for h in t.norm_headers)
        ]
        if sig_candidates:
            sig_tbl = sig_candidates[0]
            for col_name, tokens in MANDATORY_SIGNAL_COLUMNS.items():
                if not any(any(tok == h or tok in h for h in sig_tbl.norm_headers) for tok in tokens):
                    findings.append(Finding(
                        "icd-missing-mandatory-column",
                        f"Master Signal Flow Dictionary table in ICD_02 is missing mandatory column '{col_name}'.",
                        location="docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                        detail={"missing_column": col_name}
                    ))

        # 7. Check for dangling ports (Rule 1: icd-dangling-port-detected)
        # A. Check declared OUT / INOUT ports in ICD_01
        for p in icd01_ports:
            if p.direction in ("OUT", "INOUT"):
                is_bound = (
                    p.port_id in bound_src_ports
                    or p.port_name in bound_src_ports
                    or f"{p.subsystem}.{p.port_name}" in bound_src_ports
                )
                if not is_bound:
                    findings.append(Finding(
                        "icd-dangling-port-detected",
                        f"Declared output port '{p.port_id}' ({p.subsystem}.{p.port_name}) in ICD_01 is not bound to any active connection in the Connection Binding Roster Table.",
                        location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                        detail={"port_id": p.port_id, "subsystem": p.subsystem, "port_name": p.port_name}
                    ))

        # B. Check declared out ports in SysML AST
        for sp in sysml_model.ports:
            if sp.direction in ("out", "inout"):
                is_bound = (
                    sp.full_name in sysml_bound_src
                    or sp.name in sysml_bound_src
                    or sp.full_name in bound_src_ports
                    or sp.name in bound_src_ports
                )
                if not is_bound:
                    already_reported = any(
                        getattr(f, "detail", {}).get("port_name") == sp.name
                        for f in findings
                        if getattr(f, "rule_id", "") == "icd-dangling-port-detected"
                    )
                    if not already_reported:
                        findings.append(Finding(
                            "icd-dangling-port-detected",
                            f"SysML output port '{sp.full_name}' is declared in schema but not bound to any active connection.",
                            location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                            detail={"subsystem": sp.subsystem, "port_name": sp.name, "port_id": sp.full_name}
                        ))

        # 8. Check for unmapped signals (Rule 2: icd-unmapped-signal-detected)
        signal_names_in_icd02 = {s.signal_name for s in icd02_signals if s.signal_name}
        signal_ids_in_icd02 = {s.signal_id for s in icd02_signals if s.signal_id}

        sysml_flows = {f.name for f in sysml_model.item_flows if f.name}
        required_signals = sorted(sysml_flows | icd01_conveyed_flows)

        for sig in required_signals:
            if sig not in signal_names_in_icd02 and sig not in signal_ids_in_icd02:
                findings.append(Finding(
                    "icd-unmapped-signal-detected",
                    f"SysML item flow signal '{sig}' is conveyed in system architecture but not cataloged in ICD_02_MASTER_SIGNAL_DICTIONARY.md.",
                    location="docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                    detail={"signal_name": sig}
                ))

        # 9. Check signal contract integrity in ICD_02 (Rules 4 & 5)
        for s in icd02_signals:
            # Check port references
            if all_declared_tokens:
                if s.source_port and s.source_port not in all_declared_tokens:
                    findings.append(Finding(
                        "icd-invalid-port-ref",
                        f"Signal '{s.signal_id}' ({s.signal_name}) cites undeclared Source Port '{s.source_port}'.",
                        location="docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                        detail={"signal_id": s.signal_id, "source_port": s.source_port}
                    ))
                if s.dest_port and s.dest_port not in all_declared_tokens:
                    findings.append(Finding(
                        "icd-invalid-port-ref",
                        f"Signal '{s.signal_id}' ({s.signal_name}) cites undeclared Dest Port '{s.dest_port}'.",
                        location="docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                        detail={"signal_id": s.signal_id, "dest_port": s.dest_port}
                    ))

            # Check SI units
            if not s.si_units or s.si_units.strip().upper() in ("TBD", "N/A", "NONE"):
                findings.append(Finding(
                    "icd-missing-units",
                    f"Signal '{s.signal_id}' ({s.signal_name}) has missing or TBD SI units.",
                    location="docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                    detail={"signal_id": s.signal_id}
                ))

            # Check safe default value
            if not s.safe_default_value or s.safe_default_value.strip().upper() in ("TBD", "N/A", "NONE"):
                findings.append(Finding(
                    "icd-missing-safe-default",
                    f"Signal '{s.signal_id}' ({s.signal_name}) has missing or TBD safe default value.",
                    location="docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                    detail={"signal_id": s.signal_id}
                ))

            # Check schema citation
            if not s.schema_citation or s.schema_citation.strip().upper() in ("TBD", "N/A", "NONE"):
                findings.append(Finding(
                    "icd-missing-schema-citation",
                    f"Signal '{s.signal_id}' ({s.signal_name}) has missing or TBD Schema Citation.",
                    location="docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                    detail={"signal_id": s.signal_id}
                ))

        # 10. Check Port Data Type Compatibility
        def _resolve_port_types(port_token: str) -> Set[str]:
            resolved = set()
            for p in icd01_ports:
                if port_token in (p.port_id, p.port_name, f"{p.subsystem}.{p.port_name}"):
                    if p.port_type:
                        resolved.add(p.port_type)
            for sp in sysml_model.ports:
                if port_token in (sp.name, sp.full_name, f"{sp.subsystem}.{sp.name}"):
                    if sp.type_name:
                        resolved.add(sp.type_name)
            expanded = set()
            for t in resolved:
                if t in sysml_model.port_defs:
                    for item_name, item_dir, item_type in sysml_model.port_defs[t]:
                        expanded.add(item_type)
                else:
                    expanded.add(t)
            return expanded

        for c in icd01_connections:
            src_types = _resolve_port_types(c.source_port)
            dst_types = _resolve_port_types(c.dest_port)
            for st in src_types:
                for dt in dst_types:
                    if _are_types_incompatible(st, dt):
                        findings.append(Finding(
                            "icd-port-type-incompatibility",
                            f"Port data type incompatibility on connection '{c.connection_id}': source port '{c.source_port}' (type '{st}') is incompatible with destination port '{c.dest_port}' (type '{dt}').",
                            location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                            detail={"source_port": c.source_port, "dest_port": c.dest_port, "source_type": st, "dest_type": dt}
                        ))

        for sc in sysml_model.connections:
            src_types = _resolve_port_types(sc.source_full) | _resolve_port_types(sc.source_port)
            dst_types = _resolve_port_types(sc.dest_full) | _resolve_port_types(sc.dest_port)
            for st in src_types:
                for dt in dst_types:
                    if _are_types_incompatible(st, dt):
                        already_reported = any(
                            f.rule_id == "icd-port-type-incompatibility"
                            and getattr(f, "detail", {}).get("source_port") in (sc.source_full, sc.source_port)
                            for f in findings
                        )
                        if not already_reported:
                            findings.append(Finding(
                                "icd-port-type-incompatibility",
                                f"Port data type incompatibility on SysML connection '{sc.name}': source port '{sc.source_full}' (type '{st}') is incompatible with destination port '{sc.dest_full}' (type '{dt}').",
                                location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                                detail={"source_port": sc.source_full, "dest_port": sc.dest_full, "source_type": st, "dest_type": dt}
                            ))

        for s in icd02_signals:
            if s.data_type:
                dst_types = _resolve_port_types(s.dest_port)
                for dt in dst_types:
                    if _are_types_incompatible(s.data_type, dt):
                        findings.append(Finding(
                            "icd-port-type-incompatibility",
                            f"Signal '{s.signal_id}' data type '{s.data_type}' from '{s.source_port}' is incompatible with destination port '{s.dest_port}' (type '{dt}').",
                            location="docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                            detail={"signal_id": s.signal_id, "source_port": s.source_port, "dest_port": s.dest_port, "signal_type": s.data_type, "dest_type": dt}
                        ))

        # 11. Check Incompatible Update Rates (Fast Publisher vs Slow Subscriber)
        port_rates: Dict[str, float] = {}
        for p in icd01_ports:
            r = _parse_rate_hz(p.protocol_profile) or _parse_rate_hz(p.port_name)
            if r is not None:
                port_rates[p.port_id] = r
                port_rates[p.port_name] = r
                if p.subsystem:
                    port_rates[f"{p.subsystem}.{p.port_name}"] = r

        for sp in sysml_model.ports:
            r = _parse_rate_hz(sp.type_name)
            if r is not None:
                port_rates[sp.name] = r
                port_rates[sp.full_name] = r

        for c in icd01_connections:
            src_rate = port_rates.get(c.source_port)
            dst_rate = port_rates.get(c.dest_port)
            if src_rate is not None and dst_rate is not None and src_rate > dst_rate * 2.0:
                findings.append(Finding(
                    "icd-incompatible-update-rate",
                    f"Incompatible update rate on connection '{c.connection_id}': fast publisher '{c.source_port}' ({src_rate} Hz) is connected to slow subscriber '{c.dest_port}' ({dst_rate} Hz) without rate adapter or decimation.",
                    location="docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
                    detail={"source_port": c.source_port, "dest_port": c.dest_port, "publisher_rate": src_rate, "subscriber_rate": dst_rate}
                ))

        for s in icd02_signals:
            sig_rate = _parse_rate_hz(s.update_rate)
            dst_rate = port_rates.get(s.dest_port)
            if sig_rate is not None and dst_rate is not None and sig_rate > dst_rate * 2.0:
                findings.append(Finding(
                    "icd-incompatible-update-rate",
                    f"Signal '{s.signal_id}' ({s.signal_name}) update rate ({sig_rate} Hz) from publisher '{s.source_port}' exceeds subscriber '{s.dest_port}' capacity ({dst_rate} Hz) without rate adapter.",
                    location="docs/interfaces/ICD_02_MASTER_SIGNAL_DICTIONARY.md",
                    detail={"signal_id": s.signal_id, "source_port": s.source_port, "dest_port": s.dest_port, "signal_rate": sig_rate, "subscriber_rate": dst_rate}
                ))

        return findings


if __name__ == "__main__":
    repo = WorkspaceRepository()
    validator = ICDCompletenessValidator()
    errors = validator.validate(repo)
    if errors:
        for err in errors:
            print(f"[{getattr(err, 'rule_id', 'ERROR')}] {err}")
        sys.exit(1)
    else:
        print("[OK] Gate 23 (ICDCompletenessValidator): All interface and signal completeness checks passed.")
        sys.exit(0)

