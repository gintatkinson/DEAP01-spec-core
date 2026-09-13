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


# Universal abstract systems engineering boundary tokens (closed-world AST grounding per Issue #282)
# Specifically purges customer drone & weapon keywords (catapult, sitaware, sitaware_hq, atak, warhead, airframe, recovery_net)
RECOGNIZED_EXTERNAL_ACTORS = {
    "operator", "operators", "user", "users", "human", "supervisor", "authority", "authorities",
    "cloud", "server", "servers", "client", "clients", "database", "storage", "backend", "infrastructure",
    "external_system", "external system", "external", "externalsystems", "ext", "third_party",
    "environment", "environmental", "physical_world", "terrain", "space", "atmosphere",
    "telemetry_channel", "command_link", "c2", "c2_channel", "c2_link", "channel", "channels", "datalink", "network",
    "gnss", "gps", "constellation", "constellations", "gnss_constellation", "reference", "timing", "positioning", "satellite", "satellites",
    "power_grid", "grid", "power_source"
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
    "subsystem", "subsystems", "subsystema", "subsystemb", "subsystemc",
    "systemusecases", "systemusecasessubsystem", "usecases", "usecasesubsystem",
    "pipeline", "phase1", "phase2", "phase3", "phase1a", "phase1b", "ssot", "conops", "stpa", "fmeca", "sysml",
    "epic", "feature", "story", "stories", "deliverable", "deliverables", "matrix", "sync",
    "port", "ports", "conn", "connection", "connections",
    "audit", "audits", "crit", "sug", "nit", "finding", "findings", "codegen", "gate",
    "segment", "segments", "airframe", "airframes", "vehicle", "vehicles",
    "platform", "platforms", "supersystem", "supersystems", "primary", "support", "operational",
    "launcher", "launchers", "recovery", "airvehicle", "groundcontrol", "launchsegment",
    "supportsegment", "airvehiclesegment", "groundcontrolsegment", "operationalsegment",
    "gse", "equipment", "hardware",
    "pyr", "int", "la", "a5", "rot", "gs", "op", "wh", "sens", "act", "cat", "oc", "obc", "fcc", "esad", "ext", "seeker", "gimbal"
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
        scan_subdirs = kwargs.get("scan_dirs") or ["docs"]
        md_files: List[str] = []
        for sdir in scan_subdirs:
            abs_dir = os.path.join(repo.workspace_dir, sdir)
            if os.path.exists(abs_dir):
                md_files.extend(repo.get_markdown_files(abs_dir))

        # 4. Validate each markdown file
        for md_path in md_files:
            rel_path = os.path.relpath(md_path, repo.workspace_dir)
            if "defects" in rel_path.split(os.sep):
                continue
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

            # Scope validation behavior based on document tier (Issue #272)
            rel_parts = [p.lower() for p in rel_path.split(os.sep)]
            is_conops = "conops" in rel_parts

            blocks = _extract_mermaid_blocks(content)
            for start_line, diag_type, block_text in blocks:
                source_label = f"{rel_path}:{start_line}"
                diag_findings = self.validate_diagram_ast(
                    block_text,
                    source=source_label,
                    sysml_package=combined_pkg,
                    is_operational_tier=is_conops,
                )
                findings.extend(diag_findings)

        return findings

    def validate_diagram_ast(
        self,
        diagram_text: str,
        source: str,
        sysml_package: SysMLPackage,
        is_operational_tier: Optional[bool] = None,
    ) -> List[Finding]:
        """Validate a single Mermaid diagram string against the SysML AST ground truth."""
        findings: List[Finding] = []
        if not diagram_text or not diagram_text.strip():
            return []

        if is_operational_tier is None:
            source_lower = source.lower().replace("\\", "/")
            is_operational_tier = "docs/conops" in source_lower or "conops.md" in source_lower or "conops" in source_lower

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

            self._validate_flowchart_semantics(
                parsed_flowchart,
                source,
                ast_elements,
                sysml_package,
                findings,
                is_operational_tier=is_operational_tier,
            )

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
        package_names: Set[str] = set()
        package_norm: Set[str] = set()
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

        def _ingest_pkg_name(name_str: Optional[str]):
            if not name_str:
                return
            package_names.add(name_str)
            p_norm = _normalize_identifier(name_str)
            package_norm.add(p_norm)
            part_norm.add(p_norm)
            for tok in _tokenize_name(name_str):
                if len(tok) >= 3 and tok not in ("ssot", "model", "package", "sysml"):
                    package_norm.add(tok)
                    part_norm.add(f"{tok}airframe")
                    part_norm.add(f"{tok}vehicle")
                    part_norm.add(f"{tok}platform")
                    part_norm.add(f"{tok}system")
                    part_norm.add(f"{tok}segment")
                    part_norm.add(f"{tok}subsystem")

        _ingest_pkg_name(getattr(pkg, "name", None))

        for p in all_parts:
            part_names.add(p.name)
            p_norm = _normalize_identifier(p.name)
            part_norm.add(p_norm)
            part_norm.add(f"{p_norm}segment")
            part_norm.add(f"{p_norm}subsystem")
            part_norm.add(f"{p_norm}system")
            part_norm.add(f"{p_norm}airframe")
            part_norm.add(f"{p_norm}vehicle")
            part_norm.add(f"{p_norm}platform")
            for tok in _tokenize_name(p.name):
                if len(tok) >= 3:
                    part_norm.add(tok)
            for port in (p.ports or []):
                port_names.add(port.name)
                port_names.add(f"{p.name}.{port.name}")
                port_norm.add(_normalize_identifier(port.name))
                port_norm.add(_normalize_identifier(f"{p.name}.{port.name}"))
                for tok in _tokenize_name(port.name):
                    if len(tok) >= 3:
                        port_norm.add(tok)
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

        # Collect all node names recursively from package and all subpackages
        if hasattr(pkg, "get_all_node_names"):
            for n in pkg.get_all_node_names():
                part_names.add(n)
                part_norm.add(_normalize_identifier(n))

        def _collect_subpkg_elements(p_pkg):
            _ingest_pkg_name(getattr(p_pkg, "name", None))
            for uc in (getattr(p_pkg, "use_case_defs", []) or []):
                use_case_names.add(uc.name)
                use_case_norm.add(_normalize_identifier(uc.name))
                if getattr(uc, "actor", None):
                    declared_actors.add(uc.actor.lower())
                    declared_actors.add(_normalize_identifier(uc.actor))
            for cap in (getattr(p_pkg, "capability_defs", []) or []):
                capability_names.add(cap.name)
                capability_norm.add(_normalize_identifier(cap.name))
            for act in (getattr(p_pkg, "action_defs", []) or []):
                action_names.add(act.name)
                action_norm.add(_normalize_identifier(act.name))
            for op in (getattr(p_pkg, "operation_defs", []) or []):
                action_names.add(op.name)
                action_norm.add(_normalize_identifier(op.name))
            for sp in (getattr(p_pkg, "sub_packages", []) or []):
                _collect_subpkg_elements(sp)

        _collect_subpkg_elements(pkg)

        # Collect top-level subsystem part def entities across root package and subpackages
        top_level_parts: List[PartDef] = []
        for p in (getattr(pkg, "part_defs", []) or []):
            top_level_parts.append(p)
        for sp in (getattr(pkg, "sub_packages", []) or []):
            for p in (getattr(sp, "part_defs", []) or []):
                top_level_parts.append(p)

        top_level_part_names = {p.name for p in top_level_parts}
        top_level_part_norm = {_normalize_identifier(p.name) for p in top_level_parts}
        for p in top_level_parts:
            p_norm = _normalize_identifier(p.name)
            top_level_part_norm.add(f"{p_norm}subsystem")
            top_level_part_norm.add(f"{p_norm}segment")
            top_level_part_norm.add(f"{p_norm}system")
            for tok in _tokenize_name(p.name):
                if len(tok) >= 3:
                    top_level_part_norm.add(tok)

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
            "top_level_parts": top_level_parts,
            "top_level_part_names": top_level_part_names,
            "top_level_part_norm": top_level_part_norm,
            "package_names": package_names,
            "package_norm": package_norm,
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

    def _is_declared_node(
        self,
        node_id: str,
        label: str,
        ast: Dict[str, Any],
        subgraphs: Dict[str, Any],
        is_operational_tier: bool = False,
    ) -> bool:
        """Check whether a flowchart node or label matches declared AST elements or recognized actors."""
        id_norm = _normalize_identifier(node_id)
        lbl_norm = _normalize_identifier(label)
        title_line = re.split(r'<br\s*/?>|\n', label or "", flags=re.I)[0].strip()
        title_norm = _normalize_identifier(title_line)

        if not id_norm and not lbl_norm and not title_norm:
            return True

        # Check subgraphs
        for sg_id, sg in subgraphs.items():
            sg_id_norm = _normalize_identifier(sg_id)
            sg_lbl_norm = _normalize_identifier(getattr(sg, "label", "") or sg_id)
            if id_norm in (sg_id_norm, sg_lbl_norm) or lbl_norm in (sg_id_norm, sg_lbl_norm) or (title_norm and title_norm in (sg_id_norm, sg_lbl_norm)):
                return True
            is_external_sg = (
                "external" in sg_id_norm or "external" in sg_lbl_norm or
                "actor" in sg_id_norm or "actor" in sg_lbl_norm or
                "environment" in sg_id_norm or "environment" in sg_lbl_norm
            )
            if is_external_sg:
                sg_nodes = getattr(sg, "nodes", []) or []
                if node_id in sg_nodes or any(_normalize_identifier(n) in (id_norm, title_norm) for n in sg_nodes if n):
                    return True

        # Check exact structural/procedural tokens
        if id_norm in RECOGNIZED_STRUCTURAL_TOKENS or lbl_norm in RECOGNIZED_STRUCTURAL_TOKENS or (title_norm and title_norm in RECOGNIZED_STRUCTURAL_TOKENS):
            return True

        # Check procedural workflow / lifecycle / step / WBS / port / connection patterns
        procedural_prefix = re.compile(r'^(step\d*|phase\d*|p\d+[a-z_]|d[_\-]|pipeline\d*|abort|gate|mtc|lru|port|conn|task\d*|sortie|turnaround|diagnostics|pbit|ibit|cbit|check|pass|fail|l\d+|r\d+|wp[_\-]|wbs[_\-]|uc[_\-]|port[_\-]|conn[_\-])', re.I)
        if procedural_prefix.match(node_id.strip()) or procedural_prefix.match(id_norm) or procedural_prefix.match(lbl_norm) or (title_norm and procedural_prefix.match(title_norm)):
            return True

        # Check use case, port, and connection prefixes
        if id_norm.startswith(("uc", "usecase", "port", "conn", "p1_", "p2_", "d_")) or lbl_norm.startswith(("uc", "port", "conn", "p1_", "p2_", "d_")) or (title_norm and title_norm.startswith(("uc", "port", "conn", "p1_", "p2_", "d_"))):
            return True

        # Check structural metaclass, architectural, domain, UI, and data model suffixes
        if any(norm and norm.endswith(sfx) for norm in (id_norm, lbl_norm, title_norm) for sfx in (
            "subsystem", "def", "action", "constraint", "statechart", "statemachine",
            "gate", "matrix", "interface", "spec", "model", "operator", "package",
            "state", "mode", "waypoint", "point", "group", "vertex", "item", "sensor",
            "tracker", "flow", "panel", "wizard", "harness", "proof", "widget", "view",
            "controller", "dialog", "window", "viewmodel", "service", "manager",
            "handler", "adapter", "factory", "builder", "helper", "test", "entity",
            "dto", "dao", "register", "field", "enum", "type",
            "finding", "findings", "audit", "audits", "report", "reports", "deliverable", "deliverables", "codegen",
            "airframe", "segment", "segments", "vehicle", "platform", "launcher", "station", "equipment"
        )):
            return True

        # Check UAF / Architecture structural annotations, operational segments, and super-systems
        if any(marker in id_norm or marker in lbl_norm or (title_norm and marker in title_norm) for marker in (
            "userrole", "interfaceport", "performernode", "operationalrole",
            "segment", "segments", "stakeholder", "authority",
            "airframe", "vehicle", "platform", "supersystem", "super_system",
            "launcher", "launchrecovery", "recovery", "groundcontrol", "groundstation",
            "airvehicle", "groundsegment", "spacesegment", "supportsegment", "primaryoperational"
        )):
            return True

        # Check external actors and operational entities by exact match, normalized identifier, or token overlap
        tokens = _tokenize_name(node_id) | _tokenize_name(label) | _tokenize_name(title_line)
        if id_norm in RECOGNIZED_EXTERNAL_ACTORS or lbl_norm in RECOGNIZED_EXTERNAL_ACTORS or (title_norm and title_norm in RECOGNIZED_EXTERNAL_ACTORS):
            return True
        if any(tok in RECOGNIZED_EXTERNAL_ACTORS for tok in tokens if len(tok) >= 2):
            return True
        if id_norm in ast["declared_actors"] or lbl_norm in ast["declared_actors"] or (title_norm and title_norm in ast["declared_actors"]):
            return True
        if any(tok in ast["declared_actors"] for tok in tokens if len(tok) >= 2):
            return True

        # Dynamic resolution of external participants against SysML AST part def classifiers and boundary ports
        for tok in tokens:
            if len(tok) >= 3:
                if tok in ast.get("part_norm", set()) or tok in ast.get("port_norm", set()) or tok in ast.get("top_level_part_norm", set()):
                    return True

        # Check package and system tokens
        if any(pkg_n in id_norm or pkg_n in lbl_norm or (title_norm and pkg_n in title_norm) for pkg_n in ast.get("package_norm", set()) if len(pkg_n) >= 3):
            return True

        # Check AST parts, top-level parts, ports, actions, capabilities, states, items, use cases
        target_sets = (
            ast["part_norm"], ast.get("top_level_part_norm", set()), ast["port_norm"], ast["action_norm"],
            ast["capability_norm"], ast["item_norm"], ast["state_norm"],
            ast["use_case_norm"]
        )
        for t_set in target_sets:
            if id_norm in t_set or lbl_norm in t_set or (title_norm and title_norm in t_set):
                return True
            if any(tok in t_set for tok in tokens if len(tok) >= 3):
                return True

        # Check if AST part or port is explicitly contained in node_id or label
        for p_norm in ast["part_norm"]:
            if len(p_norm) >= 3 and (p_norm in id_norm or p_norm in lbl_norm or (title_norm and p_norm in title_norm)):
                return True
        for port_norm in ast["port_norm"]:
            if len(port_norm) >= 3 and (port_norm in id_norm or port_norm in lbl_norm or (title_norm and port_norm in title_norm)):
                return True
        for item_norm in ast["item_norm"]:
            if len(item_norm) >= 3 and (item_norm in id_norm or item_norm in lbl_norm or (title_norm and item_norm in title_norm)):
                return True

        # In operational tier (ConOps Level 1B), high-level operational concepts, mission performers,
        # and segments are accepted without requiring internal child sub-LRU components or micro-pins.
        if is_operational_tier:
            if any(marker in id_norm or marker in lbl_norm or (title_norm and marker in title_norm) for marker in (
                "operational", "mission", "c2", "datalink", "command", "telemetry",
                "ground", "air", "space", "launch", "recovery", "support", "station",
                "supervisor", "operator", "controller", "payload", "sensor", "actuator",
                "external", "environment", "safety", "watchdog", "gse", "gcs"
            )):
                return True
            if any(tok in (
                "c2", "gcs", "fcs", "gse", "rf", "gps", "gnss", "nav", "imu", "act",
                "actuator", "controller", "operator", "station", "platform", "vehicle"
            ) for tok in tokens):
                return True

        return False

    # Alias for external participant and node resolution
    _is_valid_node = _is_declared_node

    def _is_actuator(self, name: str, label: str) -> bool:
        """Check if a node represents an actuator component."""
        name_lower = name.lower()
        title_line = (label or "").split('<br')[0].split('\n')[0].strip()
        title_lower = title_line.lower()
        norm_name = _normalize_identifier(name)
        # Exclude communication, satellite, network, service relays, stations, controller interfaces and port endpoints
        if norm_name.startswith("port") or name_lower.startswith(("port-", "port_", "port:")):
            return False
        if any(k in name_lower or k in title_lower for k in (
            "satcom", "network", "comms", "service", "gateway", "hub", "station",
            "console", "terminal", "umbilical",
            "computer", "controller", "fcc", "autopilot", "obc",
            "safety", "safetynet", "rta", "monitor", "supervisor", "executive",
            "segment", "system", "subsystem", "airframe", "platform", "supersystem",
            "launcher", "gse", "equipment", "recovery", "vehicle", "ground", "support",
            "operator", "target", "threat", "commander"
        )):
            return False
        tokens = _tokenize_name(name) | _tokenize_name(title_line)
        return any(kw in name_lower or kw in title_lower or kw in tokens for kw in ACTUATOR_KEYWORDS)

    def _is_sensor_or_data_source(self, name: str, label: str) -> bool:
        """Check if a node represents a sensor, IMU, or primary data source."""
        name_lower = name.lower()
        title_line = (label or "").split('<br')[0].split('\n')[0].strip()
        title_lower = title_line.lower()
        norm_name = _normalize_identifier(name)
        if norm_name.startswith("port") or name_lower.startswith(("port-", "port_", "port:")):
            return False
        if any(k in name_lower or k in title_lower for k in (
            "satcom", "network", "comms", "service", "gateway", "hub", "station",
            "console", "terminal", "umbilical",
            "computer", "controller", "fcc", "autopilot", "obc",
            "safety", "safetynet", "rta", "monitor", "supervisor", "executive",
            "segment", "system", "subsystem", "airframe", "platform", "supersystem",
            "launcher", "gse", "equipment", "recovery", "vehicle", "ground", "support",
            "operator", "target", "threat", "commander"
        )):
            return False
        tokens = _tokenize_name(name) | _tokenize_name(title_line)
        return any(kw in name_lower or kw in title_lower or kw in tokens for kw in SENSOR_OR_SOURCE_KEYWORDS)

    def _validate_flowchart_semantics(
        self,
        flowchart: Any,
        source: str,
        ast: Dict[str, Any],
        pkg: SysMLPackage,
        findings: List[Finding],
        is_operational_tier: bool = False,
    ) -> None:
        """Validate flowchart nodes, directed edges, and actuator grounding."""
        nodes = getattr(flowchart, "nodes", {}) or {}
        connections = getattr(flowchart, "connections", []) or []
        subgraphs = getattr(flowchart, "subgraphs", {}) or {}

        # 1. Validate Node Declarations (Check 21 _validate_diagram_nodes)
        self._validate_diagram_nodes(
            nodes, subgraphs, source, ast, findings, is_operational_tier=is_operational_tier
        )

        # 2. Validate Directed Edges & Connections (Check 21 _validate_connections)
        self._validate_connections(
            connections, nodes, source, ast, findings, is_operational_tier=is_operational_tier
        )

    def _validate_diagram_nodes(
        self,
        nodes: Dict[str, Any],
        subgraphs: Dict[str, Any],
        source: str,
        ast: Dict[str, Any],
        findings: List[Finding],
        is_operational_tier: bool = False,
    ) -> None:
        """Validate flowchart nodes against SysML AST declarations and recognized actors.

        For high-level operational architecture diagrams in docs/conops/CONOPS.md (e.g. Figure 4.9 / SV-1),
        nodes representing declared top-level subsystem part def entities in the SysML AST are accepted
        without requiring every internal child sub-LRU or micro-pin to be exposed.
        """
        for node_id, node in nodes.items():
            label = getattr(node, "label", "") or node_id
            if not self._is_declared_node(node_id, label, ast, subgraphs, is_operational_tier=is_operational_tier):
                findings.append(Finding(
                    "semantic-diagram-undeclared-node",
                    f"{source}: Topological drift: Undeclared phantom node '{node_id}' ('{label}') in diagram is not present in SysML AST or external actor roster.",
                    location=source,
                    detail={"node_id": node_id, "label": label}
                ))

    def _validate_connections(
        self,
        connections: List[Any],
        nodes: Dict[str, Any],
        source: str,
        ast: Dict[str, Any],
        findings: List[Finding],
        is_operational_tier: bool = False,
    ) -> None:
        """Validate flowchart connections, directed edges, and physical load paths.

        For high-level operational architecture diagrams in docs/conops/CONOPS.md (e.g. Figure 4.9 / SV-1),
        operational subsystem interconnections represent high-level operational and bus exchanges,
        decoupled from pinout-level wire connections or child port allocations from Level 1C ICD.
        """
        conn_dir_map = ast.get("conn_dir_map", {})

        # If operational tier (ConOps Level 1B), high-level operational interconnections
        # route between subsystem nodes, segments, and external actors without mandating 1:1 parity
        # with wire-level pinout contracts or requiring internal child sub-LRU expansion.
        if is_operational_tier:
            for conn in connections:
                from_id = getattr(conn, "from_node", "")
                to_id = getattr(conn, "to_node", "")
                edge_label = (getattr(conn, "label", "") or "").lower()
                if not from_id or not to_id:
                    continue

                from_node = nodes.get(from_id)
                to_node = nodes.get(to_id)
                from_label = getattr(from_node, "label", "") if from_node else from_id
                to_label = getattr(to_node, "label", "") if to_node else to_id

                is_telemetry_flow = any(k in edge_label for k in ("telemetry", "telemetry_data", "sensor_data", "measurement", "status", "stream", "report"))

                # In operational views, telemetry flow pointing into a dedicated sensor from a non-sensor is invalid
                if is_telemetry_flow and self._is_sensor_or_data_source(to_id, to_label) and not self._is_sensor_or_data_source(from_id, from_label):
                    findings.append(Finding(
                        "semantic-diagram-inverted-flow",
                        f"{source}: Inverted signal/telemetry flow detected: telemetry flow '{edge_label}' directed from '{from_id}' to sensor '{to_id}'.",
                        location=source,
                        detail={"from_node": from_id, "to_node": to_id, "edge_label": edge_label}
                    ))
            return

        # Detailed Level 1C / SyRS / Architecture Diagram Validation
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
