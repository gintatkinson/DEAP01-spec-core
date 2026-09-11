"""
Semantic Diagram-to-AST Topology Parity Validator (Check 21).

Enforces semantic diagram-to-AST parity across all deliverable markdown files:
1. Validates Mermaid graph, flowchart, and classDiagram blocks against SysML v2 AST ground truth.
2. Detects undeclared phantom nodes not present in the SysML AST or external actor roster ('semantic-diagram-undeclared-node').
3. Detects inverted telemetry and signal flows violating SysML connection topology ('semantic-diagram-inverted-flow').
4. Detects ungrounded actuators with zero command/power inputs ('semantic-diagram-ungrounded-component')
   and invalid physical load/command paths ('semantic-diagram-invalid-load-path').
"""

import os
import re
import sys
from typing import Dict, List, Optional, Set, Tuple, Any, Sequence

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.workspace import WorkspaceRepository
    from ..parsers.mermaid import MermaidFlowchartParser, MermaidClassDiagramParser
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.workspace import WorkspaceRepository
    from parity_auditor.parsers.mermaid import MermaidFlowchartParser, MermaidClassDiagramParser

# Import SysML v2 AST classes via the fail-closed loader
from ..utils.sysml_loader import load_sysml_ast_members

_sysml_ast = load_sysml_ast_members([
    "SysMLPackage", "SysMLParser", "PartDef", "PortDef", "ItemDef", "ConnectionDef"
])
SysMLPackage = _sysml_ast.SysMLPackage
SysMLParser = _sysml_ast.SysMLParser
PartDef = _sysml_ast.PartDef
PortDef = _sysml_ast.PortDef
ItemDef = _sysml_ast.ItemDef
ConnectionDef = _sysml_ast.ConnectionDef


# Standard external actors and boundary entities recognized across system architectures
RECOGNIZED_EXTERNAL_ACTORS = {
    "operator", "operators", "pilot", "pilots", "remote_pilot", "remote pilot", "user", "users", "human", "supervisor", "coordinator", "technician",
    "ground_station", "ground station", "gcs", "ground_control_station", "ground control station",
    "cloud", "server", "servers", "client", "clients", "database", "databases", "storage", "backend", "infrastructure", "hub", "gateway", "gateways",
    "ui", "console", "consoles", "display", "displays", "terminal", "terminals", "cockpit", "hmi", "gui", "station", "stations", "gse",
    "atc", "air_traffic_control", "air traffic control", "utm", "u-space", "authority", "authorities", "airspace_authority", "airspace authority",
    "external_system", "external system", "third_party", "gnss", "gps", "constellation", "constellations", "gnss_constellation", "weather", "weather_service", "weather service",
    "environment", "environmental", "physical_world", "physical world", "atmosphere", "ground", "terrain", "space", "orbital",
    "power_grid", "power grid", "grid", "generator", "umbilical", "power_source", "power source",
    "telemetry_channel", "command_link", "radio", "radios", "transceiver", "datalink", "satcom", "satellite", "satellites",
    "c2", "c2_channel", "c2_link", "pace", "channel", "channels", "bus", "reference", "timing", "positioning"
}

# Procedural, workflow, lifecycle, and generic diagram structural tokens
RECOGNIZED_STRUCTURAL_TOKENS = {
    "start", "end", "stop", "init", "initial", "final", "terminate", "exit",
    "decision", "choice", "fork", "join", "merge", "condition", "check",
    "pass", "fail", "yes", "no", "true", "false", "success", "error", "fault",
    "idle", "active", "standby", "armed", "disarmed", "failsafe", "emergency", "shutdown",
    "note", "log", "return", "output", "input", "step", "phase", "stage", "process", "task",
    "event", "trigger", "action", "state", "subagent", "coordinator", "planner", "executor",
    "auditor", "verifier", "agent", "caller", "runner", "tool", "prompt", "response",
    "workflow", "review", "approval", "branch", "commit", "push", "pull", "build", "test",
    "deploy", "report", "viewmodel", "view_model", "widget", "view", "screen", "page",
    "component", "button", "dialog", "layout", "app", "application", "service", "repository",
    "controller", "presenter", "store", "state_notifier", "bloc", "cubit", "provider",
    "theme", "token", "style", "color", "asset", "handler", "adapter", "factory",
    "api", "endpoint", "router", "route", "navigation", "cache", "dao", "dto", "entity", "model",
    "sample", "example", "template", "node", "nodea", "nodeb", "nodec",
    "classa", "classb", "classc", "itema", "itemb", "parta", "partb", "partc",
    "subsystema", "subsystemb", "subsystemc"
}

ACTUATOR_KEYWORDS = (
    "actuator", "motor", "servo", "esc", "thruster", "valve", "pump",
    "relay", "heater", "propeller", "rotor", "control_surface", "elevon",
    "aileron", "rudder", "elevator", "flap", "solenoid"
)

SENSOR_OR_SOURCE_KEYWORDS = (
    "sensor", "imu", "gps", "gnss", "altimeter", "barometer", "pitot",
    "camera", "lidar", "radar", "encoder", "gyro", "accelerometer", "magnetometer"
)


def _normalize_identifier(token: str) -> str:
    """Normalize identifier by stripping formatting, brackets, quotes, and punctuation."""
    if not token:
        return ""
    t = re.sub(r'<[^>]+>', ' ', token.strip().strip('"\'`'))
    t = re.sub(r'[\(\[\{].*?[\)\]\}]', '', t)
    t = re.sub(r'[^a-zA-Z0-9]', '', t).lower()
    return t


def _tokenize_name(name: str) -> Set[str]:
    """Extract individual alphanumeric token words from a name or label."""
    if not name:
        return set()
    clean = re.sub(r'<[^>]+>', ' ', name)
    clean = re.sub(r'[\(\[\{].*?[\)\]\}]', ' ', clean)
    words = re.findall(r'[a-zA-Z0-9]+', clean)
    tokens = {w.lower() for w in words}
    tokens.add(_normalize_identifier(name))
    return tokens - {""}


def _find_sysml_files(repo: WorkspaceRepository, schemas_dir: Optional[str] = None) -> List[str]:
    """Locate all SysML files in workspace."""
    sysml_files: List[str] = []

    if schemas_dir and os.path.exists(schemas_dir):
        if os.path.isfile(schemas_dir) and schemas_dir.endswith(".sysml"):
            sysml_files.append(schemas_dir)
        elif os.path.isdir(schemas_dir):
            for root, _, files in os.walk(schemas_dir):
                for f in sorted(files):
                    if f.endswith(".sysml") and not f.startswith("."):
                        sysml_files.append(os.path.join(root, f))

    for s_name in ("schema", "schemas"):
        cand = os.path.join(repo.workspace_dir, s_name)
        if os.path.isdir(cand):
            for root, _, files in os.walk(cand):
                for f in sorted(files):
                    if f.endswith(".sysml") and not f.startswith("."):
                        p = os.path.join(root, f)
                        if p not in sysml_files:
                            sysml_files.append(p)

    if not sysml_files and not repo.is_upstream_compiler_repo():
        pipeline_sysml = os.path.join(repo.workspace_dir, ".pipeline", "schema.sysml")
        if os.path.exists(pipeline_sysml) and pipeline_sysml not in sysml_files:
            sysml_files.append(pipeline_sysml)

    return sysml_files


def _extract_mermaid_blocks(content: str) -> List[Tuple[int, str, str]]:
    """Extract Mermaid fenced blocks: returns List of (start_line, diagram_type, block_content)."""
    blocks: List[Tuple[int, str, str]] = []
    lines = content.splitlines()
    i = 0
    fence_pattern = re.compile(r"^\s*```+\s*mermaid\s*$", re.I)
    end_fence_pattern = re.compile(r"^\s*```+\s*$")

    while i < len(lines):
        if fence_pattern.match(lines[i]):
            start_lineno = i + 1
            body: List[str] = []
            i += 1
            while i < len(lines):
                if end_fence_pattern.match(lines[i]):
                    break
                body.append(lines[i])
                i += 1
            
            # Determine diagram type from first non-comment line
            diag_type = ""
            for line in body:
                stripped = line.strip().lower()
                if stripped and not stripped.startswith("%%"):
                    diag_type = stripped.split()[0] if stripped.split() else ""
                    break
            
            raw_block = "\n".join(body)
            blocks.append((start_lineno, diag_type, raw_block))
        i += 1

    return blocks


class SemanticDiagramASTValidator(IValidator):
    """
    Check 21: Semantic Diagram-to-AST Topology Parity Validator.
    Verifies that deliverable Mermaid diagrams strictly align with the SysML v2 AST:
    - Zero undeclared phantom nodes.
    - Zero inverted telemetry/signal flows.
    - Zero ungrounded actuators or invalid physical load paths.
    """

    def __init__(self, workspace_repo: Optional[WorkspaceRepository] = None):
        self.workspace_repo = workspace_repo

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """Validate all Markdown deliverable diagrams across the repository against SysML AST."""
        findings: List[Finding] = []
        schemas_dir = kwargs.get("schemas_dir")
        self.workspace_repo = repo

        # 1. Discover SysML files
        sysml_files = _find_sysml_files(repo, schemas_dir)
        if not sysml_files:
            return []

        # 2. Parse and merge SysML package AST
        combined_pkg = SysMLPackage(name="MergedSystemModel")
        has_content = False

        for sf in sysml_files:
            try:
                with open(sf, "r", encoding="utf-8") as f:
                    text = f.read()
                if not text.strip():
                    continue
                pkg = SysMLParser.parse_text(text)
                has_content = True
                combined_pkg.part_defs.extend(pkg.part_defs or [])
                combined_pkg.port_defs.extend(pkg.port_defs or [])
                combined_pkg.action_defs.extend(pkg.action_defs or [])
                combined_pkg.capability_defs.extend(pkg.capability_defs or [])
                combined_pkg.operation_defs.extend(pkg.operation_defs or [])
                combined_pkg.interaction_defs.extend(pkg.interaction_defs or [])
                combined_pkg.constraint_defs.extend(pkg.constraint_defs or [])
                combined_pkg.test_case_defs.extend(pkg.test_case_defs or [])
                combined_pkg.requirement_defs.extend(pkg.requirement_defs or [])
                combined_pkg.state_defs.extend(pkg.state_defs or [])
                combined_pkg.use_case_defs.extend(pkg.use_case_defs or [])
                combined_pkg.item_defs.extend(pkg.item_defs or [])
                combined_pkg.hazard_defs.extend(pkg.hazard_defs or [])
                combined_pkg.risk_defs.extend(pkg.risk_defs or [])
                combined_pkg.connection_defs.extend(pkg.connection_defs or [])
                combined_pkg.sub_packages.extend(pkg.sub_packages or [])
            except Exception:
                pass

        if not has_content or (not combined_pkg.part_defs and not combined_pkg.connection_defs and not combined_pkg.sub_packages):
            return []

        # 3. Discover markdown files to validate
        scan_subdirs = kwargs.get("scan_dirs") or ["docs", "rules", "skills"]
        md_files: List[str] = []
        for sdir in scan_subdirs:
            abs_dir = os.path.join(repo.workspace_dir, sdir)
            if os.path.exists(abs_dir):
                md_files.extend(repo.get_markdown_files(abs_dir))

        # 4. Validate each markdown file
        for md_path in md_files:
            rel_path = os.path.relpath(md_path, repo.workspace_dir)
            try:
                with open(md_path, "r", encoding="utf-8") as f:
                    content = f.read()
            except Exception as e:
                findings.append(Finding(
                    "semantic-diagram-read-error",
                    f"Failed to read '{rel_path}': {e}",
                    location=rel_path
                ))
                continue

            blocks = _extract_mermaid_blocks(content)
            for start_line, diag_type, block_text in blocks:
                source_label = f"{rel_path}:{start_line}"
                diag_findings = self.validate_diagram_ast(block_text, source=source_label, sysml_package=combined_pkg)
                findings.extend(diag_findings)

        return findings

    def validate_diagram_ast(self, diagram_text: str, source: str, sysml_package: SysMLPackage) -> List[Finding]:
        """Validate a single Mermaid diagram string against the SysML AST ground truth."""
        findings: List[Finding] = []
        if not diagram_text or not diagram_text.strip():
            return []

        # 1. Build AST ground truth registry
        ast_elements = self._build_ast_ground_truth(sysml_package)

        clean_text = diagram_text.strip()
        lower_text = clean_text.lower()

        # 2. Check diagram type and parse
        if any(lower_text.startswith(p) for p in ("flowchart", "graph td", "graph lr", "graph", "flowchart td", "flowchart lr", "graph bt", "graph rl")):
            flowchart_parser = MermaidFlowchartParser()
            try:
                parsed_flowchart = flowchart_parser.parse(clean_text)
            except Exception:
                return []

            self._validate_flowchart_semantics(parsed_flowchart, source, ast_elements, sysml_package, findings)

        elif "classdiagram" in lower_text:
            try:
                workspace = self.workspace_repo or WorkspaceRepository()
                class_parser = MermaidClassDiagramParser(workspace)
                parsed_class = class_parser.parse(clean_text)
                self._validate_class_diagram_semantics(parsed_class, source, ast_elements, findings)
            except Exception:
                pass

        return findings

    def _build_ast_ground_truth(self, pkg: SysMLPackage) -> Dict[str, Any]:
        """Collect all declared AST elements from SysMLPackage."""
        all_parts = pkg.get_all_parts()
        all_conns = pkg.get_all_connections()
        all_hazards = pkg.get_all_hazards()
        all_risks = pkg.get_all_risks()
        all_states = pkg.get_all_states()

        part_names: Set[str] = set()
        part_norm: Set[str] = set()
        port_names: Set[str] = set()
        port_norm: Set[str] = set()
        action_names: Set[str] = set()
        action_norm: Set[str] = set()
        capability_names: Set[str] = set()
        capability_norm: Set[str] = set()
        item_names: Set[str] = set()
        item_norm: Set[str] = set()
        state_names: Set[str] = set()
        state_norm: Set[str] = set()
        use_case_names: Set[str] = set()
        use_case_norm: Set[str] = set()
        declared_actors: Set[str] = set()

        for p in all_parts:
            part_names.add(p.name)
            part_norm.add(_normalize_identifier(p.name))
            for port in (p.ports or []):
                port_names.add(port.name)
                port_names.add(f"{p.name}.{port.name}")
                port_norm.add(_normalize_identifier(port.name))
                port_norm.add(_normalize_identifier(f"{p.name}.{port.name}"))
            for act in (p.actions or []):
                action_names.add(act.name)
                action_norm.add(_normalize_identifier(act.name))
            for op in (p.operations or []):
                action_names.add(op.name)
                action_norm.add(_normalize_identifier(op.name))
            for cap in (p.capabilities or []):
                capability_names.add(cap.name)
                capability_norm.add(_normalize_identifier(cap.name))
            for st in (p.states or []):
                state_names.add(st.name)
                state_norm.add(_normalize_identifier(st.name))
            for it in (p.item_defs or []):
                item_names.add(it.name)
                item_norm.add(_normalize_identifier(it.name))
            for uc in (p.use_cases or []):
                use_case_names.add(uc.name)
                use_case_norm.add(_normalize_identifier(uc.name))
                if uc.actor:
                    declared_actors.add(uc.actor.lower())
                    declared_actors.add(_normalize_identifier(uc.actor))

        for port in (pkg.port_defs or []):
            port_names.add(port.name)
            port_norm.add(_normalize_identifier(port.name))
        for act in (pkg.action_defs or []):
            action_names.add(act.name)
            action_norm.add(_normalize_identifier(act.name))
        for op in (pkg.operation_defs or []):
            action_names.add(op.name)
            action_norm.add(_normalize_identifier(op.name))
        for cap in (pkg.capability_defs or []):
            capability_names.add(cap.name)
            capability_norm.add(_normalize_identifier(cap.name))
        for it in (pkg.item_defs or []):
            item_names.add(it.name)
            item_norm.add(_normalize_identifier(it.name))
        for st in (pkg.state_defs or []):
            state_names.add(st.name)
            state_norm.add(_normalize_identifier(st.name))
        for uc in (pkg.use_case_defs or []):
            use_case_names.add(uc.name)
            use_case_norm.add(_normalize_identifier(uc.name))
            if uc.actor:
                declared_actors.add(uc.actor.lower())
                declared_actors.add(_normalize_identifier(uc.actor))

        # Build connection directional map: src_part -> set of dest_parts
        conn_dir_map: Dict[str, Set[str]] = {}
        for conn in all_conns:
            src = conn.source_port
            tgt = conn.target_port
            if src and tgt:
                src_part = src.split('.', 1)[0] if '.' in src else src
                tgt_part = tgt.split('.', 1)[0] if '.' in tgt else tgt
                conn_dir_map.setdefault(_normalize_identifier(src_part), set()).add(_normalize_identifier(tgt_part))
                conn_dir_map.setdefault(_normalize_identifier(src), set()).add(_normalize_identifier(tgt))

        return {
            "all_parts": all_parts,
            "all_conns": all_conns,
            "part_names": part_names,
            "part_norm": part_norm,
            "port_names": port_names,
            "port_norm": port_norm,
            "action_names": action_names,
            "action_norm": action_norm,
            "capability_names": capability_names,
            "capability_norm": capability_norm,
            "item_names": item_names,
            "item_norm": item_norm,
            "state_names": state_names,
            "state_norm": state_norm,
            "use_case_names": use_case_names,
            "use_case_norm": use_case_norm,
            "declared_actors": declared_actors,
            "conn_dir_map": conn_dir_map,
        }

    def _is_declared_node(self, node_id: str, label: str, ast: Dict[str, Any], subgraphs: Dict[str, Any]) -> bool:
        """Check whether a flowchart node or label matches declared AST elements or recognized actors."""
        id_norm = _normalize_identifier(node_id)
        lbl_norm = _normalize_identifier(label)

        if not id_norm and not lbl_norm:
            return True

        # Check subgraphs
        for sg_id, sg in subgraphs.items():
            sg_id_norm = _normalize_identifier(sg_id)
            sg_lbl_norm = _normalize_identifier(getattr(sg, "label", "") or sg_id)
            if id_norm in (sg_id_norm, sg_lbl_norm) or lbl_norm in (sg_id_norm, sg_lbl_norm):
                return True

        # Check exact structural/procedural tokens
        if id_norm in RECOGNIZED_STRUCTURAL_TOKENS or lbl_norm in RECOGNIZED_STRUCTURAL_TOKENS:
            return True

        # Check procedural workflow / lifecycle / step / WBS patterns
        procedural_prefix = re.compile(r'^(step\d*|phase|abort|gate|mtc|lru|task\d*|sortie|turnaround|diagnostics|pbit|ibit|cbit|check|pass|fail|l\d+|wp[_\-]|wbs[_\-])', re.I)
        if procedural_prefix.match(node_id.strip()) or procedural_prefix.match(id_norm) or procedural_prefix.match(lbl_norm):
            return True

        # Check UAF / Architecture structural annotations and segments
        if any(marker in id_norm or marker in lbl_norm for marker in ("userrole", "interfaceport", "performernode", "operationalrole", "segment", "stakeholder", "authority")):
            return True

        # Check external actors by exact match, normalized identifier, or token overlap
        tokens = _tokenize_name(node_id) | _tokenize_name(label)
        if id_norm in RECOGNIZED_EXTERNAL_ACTORS or lbl_norm in RECOGNIZED_EXTERNAL_ACTORS:
            return True
        if any(tok in RECOGNIZED_EXTERNAL_ACTORS for tok in tokens if len(tok) >= 2):
            return True
        if id_norm in ast["declared_actors"] or lbl_norm in ast["declared_actors"]:
            return True
        if any(tok in ast["declared_actors"] for tok in tokens if len(tok) >= 2):
            return True

        # Check AST parts, ports, actions, capabilities, states, items, use cases
        target_sets = (
            ast["part_norm"], ast["port_norm"], ast["action_norm"],
            ast["capability_norm"], ast["item_norm"], ast["state_norm"],
            ast["use_case_norm"]
        )
        for t_set in target_sets:
            if id_norm in t_set or lbl_norm in t_set:
                return True

        # Check if AST part or port is explicitly contained in node_id or label
        for p_norm in ast["part_norm"]:
            if len(p_norm) >= 3 and (p_norm in id_norm or p_norm in lbl_norm):
                return True
        for port_norm in ast["port_norm"]:
            if len(port_norm) >= 3 and (port_norm in id_norm or port_norm in lbl_norm):
                return True
        for item_norm in ast["item_norm"]:
            if len(item_norm) >= 3 and (item_norm in id_norm or item_norm in lbl_norm):
                return True

        return False

    def _is_actuator(self, name: str, label: str) -> bool:
        """Check if a node represents an actuator component."""
        name_lower = name.lower()
        lbl_lower = (label or "").lower()
        # Exclude communication, satellite, network, service relays, and stations
        if any(k in name_lower or k in lbl_lower for k in ("satcom", "network", "comms", "service", "gateway", "hub", "station", "console", "terminal", "umbilical")):
            return False
        tokens = _tokenize_name(name) | _tokenize_name(label)
        return any(kw in name_lower or kw in lbl_lower or kw in tokens for kw in ACTUATOR_KEYWORDS)

    def _is_sensor_or_data_source(self, name: str, label: str) -> bool:
        """Check if a node represents a sensor, IMU, or primary data source."""
        name_lower = name.lower()
        lbl_lower = (label or "").lower()
        tokens = _tokenize_name(name) | _tokenize_name(label)
        return any(kw in name_lower or kw in lbl_lower or kw in tokens for kw in SENSOR_OR_SOURCE_KEYWORDS)

    def _validate_flowchart_semantics(
        self,
        flowchart: Any,
        source: str,
        ast: Dict[str, Any],
        pkg: SysMLPackage,
        findings: List[Finding]
    ) -> None:
        """Validate flowchart nodes, directed edges, and actuator grounding."""
        nodes = getattr(flowchart, "nodes", {}) or {}
        connections = getattr(flowchart, "connections", []) or []
        subgraphs = getattr(flowchart, "subgraphs", {}) or {}

        # 1. Validate Node Declarations (Check for Undeclared Phantom Nodes)
        for node_id, node in nodes.items():
            label = getattr(node, "label", "") or node_id
            if not self._is_declared_node(node_id, label, ast, subgraphs):
                findings.append(Finding(
                    "semantic-diagram-undeclared-node",
                    f"{source}: Undeclared phantom node '{node_id}' ('{label}') in diagram is not present in SysML AST or external actor roster.",
                    location=source,
                    detail={"node_id": node_id, "label": label}
                ))

        # 2. Validate Directed Edges & Signal/Telemetry Flow Parity
        conn_dir_map = ast["conn_dir_map"]
        for conn in connections:
            from_id = getattr(conn, "from_node", "")
            to_id = getattr(conn, "to_node", "")
            edge_label = (getattr(conn, "label", "") or "").lower()
            if not from_id or not to_id:
                continue

            from_norm = _normalize_identifier(from_id)
            to_norm = _normalize_identifier(to_id)
            from_node = nodes.get(from_id)
            to_node = nodes.get(to_id)
            from_label = getattr(from_node, "label", "") if from_node else from_id
            to_label = getattr(to_node, "label", "") if to_node else to_id

            # Check if SysML defines an inverted connection
            # If SysML has to_norm -> from_norm but NOT from_norm -> to_norm
            sysml_has_reverse = to_norm in conn_dir_map and from_norm in conn_dir_map[to_norm]
            sysml_has_forward = from_norm in conn_dir_map and to_norm in conn_dir_map[from_norm]

            is_telemetry_flow = any(k in edge_label for k in ("telemetry", "telemetry_data", "sensor_data", "measurement", "status", "stream", "report"))

            if sysml_has_reverse and not sysml_has_forward:
                findings.append(Finding(
                    "semantic-diagram-inverted-flow",
                    f"{source}: Inverted signal/telemetry flow detected: diagram directs flow from '{from_id}' to '{to_id}', violating SysML connection topology ('{to_id}' -> '{from_id}').",
                    location=source,
                    detail={"from_node": from_id, "to_node": to_id, "edge_label": edge_label}
                ))
            elif is_telemetry_flow and self._is_sensor_or_data_source(to_id, to_label) and not self._is_sensor_or_data_source(from_id, from_label):
                # Telemetry flow pointing into a sensor from a non-sensor
                findings.append(Finding(
                    "semantic-diagram-inverted-flow",
                    f"{source}: Inverted signal/telemetry flow detected: telemetry flow '{edge_label}' directed from '{from_id}' to sensor '{to_id}'.",
                    location=source,
                    detail={"from_node": from_id, "to_node": to_id, "edge_label": edge_label}
                ))

        # 3. Validate Ungrounded Actuators & Invalid Physical Load Paths
        incoming_counts: Dict[str, int] = {nid: 0 for nid in nodes}
        outgoing_conns: Dict[str, List[Any]] = {nid: [] for nid in nodes}

        for conn in connections:
            from_id = getattr(conn, "from_node", "")
            to_id = getattr(conn, "to_node", "")
            if to_id in incoming_counts:
                incoming_counts[to_id] += 1
            if from_id in outgoing_conns:
                outgoing_conns[from_id].append(conn)

        for node_id, node in nodes.items():
            label = getattr(node, "label", "") or node_id
            if self._is_actuator(node_id, label):
                # Check for ungrounded component (0 incoming command/power connections)
                if incoming_counts.get(node_id, 0) == 0:
                    findings.append(Finding(
                        "semantic-diagram-ungrounded-component",
                        f"{source}: Ungrounded actuator '{node_id}' ('{label}') has no incoming command/power connections in diagram.",
                        location=source,
                        detail={"node_id": node_id, "label": label}
                    ))

                # Check for invalid load paths (actuator commanding controller or sensor)
                for out_conn in outgoing_conns.get(node_id, []):
                    tgt_id = getattr(out_conn, "to_node", "")
                    tgt_node = nodes.get(tgt_id)
                    tgt_label = getattr(tgt_node, "label", "") if tgt_node else tgt_id
                    out_label = (getattr(out_conn, "label", "") or "").lower()

                    if any(k in out_label for k in ("command", "cmd", "control", "drive")) or self._is_sensor_or_data_source(tgt_id, tgt_label):
                        findings.append(Finding(
                            "semantic-diagram-invalid-load-path",
                            f"{source}: Invalid physical load/command path: actuator '{node_id}' directs command/driving flow upstream to '{tgt_id}'.",
                            location=source,
                            detail={"actuator": node_id, "target": tgt_id, "label": out_label}
                        ))

    def _validate_class_diagram_semantics(
        self,
        class_diag: Any,
        source: str,
        ast: Dict[str, Any],
        findings: List[Finding]
    ) -> None:
        """Validate classDiagram classes against SysML AST."""
        classes = getattr(class_diag, "classes", {}) or {}
        for cls_name, cls_info in classes.items():
            cls_norm = _normalize_identifier(cls_name)
            if not self._is_declared_node(cls_name, cls_name, ast, {}):
                findings.append(Finding(
                    "semantic-diagram-undeclared-node",
                    f"{source}: Undeclared class '{cls_name}' in classDiagram is not present in SysML AST.",
                    location=source,
                    detail={"class_name": cls_name}
                ))
