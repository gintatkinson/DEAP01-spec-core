"""
Cross-Document Diagram Parity Validator (Check 25).

Enforces strict 1:1 parity between Mermaid architecture diagrams replicated across
specification and executive deliverable documents:
1. Extracts and normalizes Mermaid architecture diagrams from `docs/conops/CONOPS.md`
   (and other ConOps specifications) and `docs/reports/PHASE_1_EXECUTIVE_ENGINEERING_DELIVERABLES.md`
   (or other executive reports in `docs/reports/` and `docs/management/`).
2. Validates graph structures:
   - Subgraphs (identifiers, labels, and assigned node memberships).
   - Nodes (component identifiers and shape/label semantics).
   - Embedded Port Attributes (bulleted port declarations: port name, direction, and data type).
   - Connection Links (source, destination, directionality, connection IDs, and payload labels).
3. Emits fatal error findings ('cross-document-diagram-disparity') if any disparity in nodes,
   ports, edges, or subgraphs is detected between source specifications and executive deliverables.
"""

import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple, Union

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.workspace import WorkspaceRepository
    from ..parsers.mermaid import MermaidFlowchartParser
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.workspace import WorkspaceRepository
    from parity_auditor.parsers.mermaid import MermaidFlowchartParser


RULE_ID = "cross-document-diagram-disparity"


def _normalize_text(s: Optional[str]) -> str:
    """Normalize text by stripping quotes, html breaks, whitespace, and case."""
    if not s:
        return ""
    t = re.sub(r'<[^>]+>', ' ', s)
    t = re.sub(r'\s+', ' ', t).strip().strip('"\'`')
    return t


def _normalize_identifier(token: str) -> str:
    """Normalize identifier by stripping formatting and punctuation."""
    if not token:
        return ""
    t = re.sub(r'<[^>]+>', ' ', token.strip().strip('"\'`'))
    t = re.sub(r'[\(\[\{].*?[\)\]\}]', '', t)
    t = re.sub(r'[^a-zA-Z0-9_]', '', t)
    return t.lower()


def _extract_mermaid_blocks(content: str, rel_path: str) -> List[Dict[str, Any]]:
    """Extract Mermaid fenced blocks with start lines and preceding header context."""
    blocks: List[Dict[str, Any]] = []
    lines = content.splitlines()
    i = 0
    fence_pattern = re.compile(r"^\s*```+\s*mermaid\s*$", re.I)
    end_fence_pattern = re.compile(r"^\s*```+\s*$")
    last_heading = ""

    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        if stripped.startswith("#"):
            last_heading = stripped.lstrip("#").strip()

        if fence_pattern.match(line):
            start_lineno = i + 1
            body: List[str] = []
            i += 1
            while i < len(lines):
                if end_fence_pattern.match(lines[i]):
                    break
                body.append(lines[i])
                i += 1

            diag_type = ""
            for bline in body:
                b_stripped = bline.strip().lower()
                if b_stripped and not b_stripped.startswith("%%"):
                    diag_type = b_stripped.split()[0] if b_stripped.split() else ""
                    break

            raw_block = "\n".join(body)
            blocks.append({
                "start_line": start_lineno,
                "diag_type": diag_type,
                "raw_block": raw_block,
                "heading": last_heading,
                "source": f"{rel_path}:{start_lineno}",
                "rel_path": rel_path,
            })
        i += 1

    return blocks


def _parse_ports_from_label(label: str) -> Dict[str, Dict[str, str]]:
    """Extract embedded port declarations from node label text.

    Supported patterns:
    - • c2Uplink (OUT: C2Commands)
    - • c2Downlink (IN: C2Telemetry)
    - • rfl (INOUT: RFLink)
    - - portName (DIRECTION: Type)
    - * portName (DIRECTION: Type)
    - portName (DIRECTION: Type)
    """
    ports: Dict[str, Dict[str, str]] = {}
    if not label:
        return ports

    # Normalize HTML breaks into newlines
    normalized_label = re.sub(r'<br\s*/?>', '\n', label, flags=re.I)

    # Pattern for bulleted or parenthesized port declarations
    pattern_with_dir = re.compile(
        r'(?:[•\-\*]|\b)\s*([a-zA-Z0-9_]+)\s*\(\s*(IN|OUT|INOUT|in|out|inout)\s*:\s*([a-zA-Z0-9_]+)\s*\)'
    )
    for m in pattern_with_dir.finditer(normalized_label):
        port_name = m.group(1).strip()
        direction = m.group(2).strip().upper()
        port_type = m.group(3).strip()
        norm_key = port_name.lower()
        ports[norm_key] = {
            "name": port_name,
            "direction": direction,
            "data_type": port_type,
            "raw": m.group(0).strip(),
        }

    # Pattern for port without explicit direction (e.g., • portName: Type)
    pattern_without_dir = re.compile(
        r'(?:[•\-\*])\s*([a-zA-Z0-9_]+)\s*:\s*([a-zA-Z0-9_]+)'
    )
    for m in pattern_without_dir.finditer(normalized_label):
        port_name = m.group(1).strip()
        port_type = m.group(2).strip()
        norm_key = port_name.lower()
        if norm_key not in ports:
            ports[norm_key] = {
                "name": port_name,
                "direction": "UNSPECIFIED",
                "data_type": port_type,
                "raw": m.group(0).strip(),
            }

    return ports


def _parse_diagram_structure(block_dict: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Parse a flowchart/graph Mermaid diagram into normalized structural entities."""
    raw_block = block_dict["raw_block"]
    clean_text = raw_block.strip()
    lower_text = clean_text.lower()

    if not any(lower_text.startswith(p) for p in ("flowchart", "graph td", "graph lr", "graph", "flowchart td", "flowchart lr", "graph bt", "graph rl")):
        return None

    parser = MermaidFlowchartParser()
    try:
        parsed = parser.parse(clean_text)
    except Exception:
        return None

    nodes_dict: Dict[str, Any] = {}
    for nid, node in (getattr(parsed, "nodes", {}) or {}).items():
        label = getattr(node, "label", "") or nid
        ports = _parse_ports_from_label(label)
        first_line = label.split('<br')[0].split('\n')[0].strip()
        nodes_dict[nid] = {
            "id": nid,
            "norm_id": _normalize_identifier(nid),
            "label": label,
            "title": first_line,
            "shape": getattr(node, "shape", "unknown"),
            "subgraph": getattr(node, "subgraph", None),
            "ports": ports,
        }

    subgraphs_dict: Dict[str, Any] = {}
    for sid, sg in (getattr(parsed, "subgraphs", {}) or {}).items():
        s_label = getattr(sg, "label", "") or sid
        subgraphs_dict[sid] = {
            "id": sid,
            "norm_id": _normalize_identifier(sid),
            "label": s_label,
            "norm_label": _normalize_text(s_label),
            "parent": getattr(sg, "parent", None),
            "nodes": set(getattr(sg, "nodes", []) or []),
        }

    # Populate subgraphs with node assignments
    for nid, ndata in nodes_dict.items():
        sg_id = ndata["subgraph"]
        if sg_id and sg_id in subgraphs_dict:
            subgraphs_dict[sg_id]["nodes"].add(nid)

    connections_list: List[Dict[str, Any]] = []
    for conn in (getattr(parsed, "connections", []) or []):
        from_id = getattr(conn, "from_node", "")
        to_id = getattr(conn, "to_node", "")
        label = getattr(conn, "label", "") or ""
        style = getattr(conn, "style", "solid_arrow")
        if from_id and to_id:
            # Extract CONN-xx id if present
            conn_id_match = re.search(r'\b(CONN-[0-9]+(?:-[0-9]+)?)\b', label, re.I)
            conn_id = conn_id_match.group(1).upper() if conn_id_match else None
            connections_list.append({
                "from_node": from_id,
                "to_node": to_id,
                "norm_from": _normalize_identifier(from_id),
                "norm_to": _normalize_identifier(to_id),
                "style": style,
                "label": label.strip(),
                "norm_label": _normalize_text(label),
                "conn_id": conn_id,
            })

    return {
        "source": block_dict["source"],
        "rel_path": block_dict["rel_path"],
        "start_line": block_dict["start_line"],
        "heading": block_dict["heading"],
        "nodes": nodes_dict,
        "subgraphs": subgraphs_dict,
        "connections": connections_list,
        "node_ids": set(nodes_dict.keys()),
        "norm_node_ids": {ndata["norm_id"] for ndata in nodes_dict.values()},
    }


def _is_sv1_diagram(diag_struct: Dict[str, Any]) -> bool:
    """Determine if a diagram is specifically an SV-1 System Interface Block Diagram."""
    heading = (diag_struct.get("heading") or "").lower()
    if any(k in heading for k in ("sv-1", "system interface block", "system interface matrix", "system functional & logical", "system functional and logical")):
        return True

    # Check for presence of core SV-1 architectural node set
    norm_nodes = diag_struct.get("norm_node_ids", set())
    core_sv1_tokens = {"gcs", "obc", "esad", "wh", "act", "prop", "sens", "rta", "cat", "gsr", "ant"}
    overlap = len(norm_nodes & core_sv1_tokens)
    return overlap >= 7


def _compute_node_overlap(d1: Dict[str, Any], d2: Dict[str, Any]) -> float:
    """Compute Jaccard similarity of normalized node IDs between two diagrams."""
    nodes1 = d1.get("norm_node_ids", set())
    nodes2 = d2.get("norm_node_ids", set())
    if not nodes1 or not nodes2:
        return 0.0
    intersection = len(nodes1 & nodes2)
    union = len(nodes1 | nodes2)
    return intersection / union if union > 0 else 0.0


def _compare_diagrams(
    ref_diag: Dict[str, Any],
    target_diag: Dict[str, Any],
    diag_name: str = "System Architecture Diagram"
) -> List[Finding]:
    """Perform exhaustive structural parity comparison between reference and target diagrams."""
    findings: List[Finding] = []
    ref_src = ref_diag["source"]
    tgt_src = target_diag["source"]

    ref_nodes = ref_diag["nodes"]
    tgt_nodes = target_diag["nodes"]

    # 1. Compare Nodes
    ref_node_keys = set(ref_nodes.keys())
    tgt_node_keys = set(tgt_nodes.keys())

    missing_in_target = ref_node_keys - tgt_node_keys
    extra_in_target = tgt_node_keys - ref_node_keys

    for missing_node in sorted(missing_in_target):
        findings.append(Finding(
            RULE_ID,
            f"{tgt_src}: Disparity in {diag_name}: Missing node '{missing_node}' present in reference {ref_src}.",
            location=tgt_src,
            detail={"node": missing_node, "disparity_kind": "missing_node", "reference": ref_src}
        ))

    for extra_node in sorted(extra_in_target):
        findings.append(Finding(
            RULE_ID,
            f"{tgt_src}: Disparity in {diag_name}: Extra undeclared node '{extra_node}' not present in reference {ref_src}.",
            location=tgt_src,
            detail={"node": extra_node, "disparity_kind": "extra_node", "reference": ref_src}
        ))

    # 2. Compare Embedded Port Attributes for Common Nodes
    common_nodes = ref_node_keys & tgt_node_keys
    for nid in sorted(common_nodes):
        ref_ndata = ref_nodes[nid]
        tgt_ndata = tgt_nodes[nid]

        ref_ports = ref_ndata.get("ports", {})
        tgt_ports = tgt_ndata.get("ports", {})

        ref_port_keys = set(ref_ports.keys())
        tgt_port_keys = set(tgt_ports.keys())

        missing_ports = ref_port_keys - tgt_port_keys
        extra_ports = tgt_port_keys - ref_port_keys

        for pkey in sorted(missing_ports):
            pinfo = ref_ports[pkey]
            findings.append(Finding(
                RULE_ID,
                f"{tgt_src}: Disparity in {diag_name}: Node '{nid}' missing port '{pinfo['name']}' ({pinfo['direction']}: {pinfo['data_type']}) declared in reference {ref_src}.",
                location=tgt_src,
                detail={"node": nid, "port": pinfo["name"], "disparity_kind": "missing_port", "reference": ref_src}
            ))

        for pkey in sorted(extra_ports):
            pinfo = tgt_ports[pkey]
            findings.append(Finding(
                RULE_ID,
                f"{tgt_src}: Disparity in {diag_name}: Node '{nid}' has extra undeclared port '{pinfo['name']}' ({pinfo['direction']}: {pinfo['data_type']}) not present in reference {ref_src}.",
                location=tgt_src,
                detail={"node": nid, "port": pinfo["name"], "disparity_kind": "extra_port", "reference": ref_src}
            ))

        # Check port attribute modifications
        common_ports = ref_port_keys & tgt_port_keys
        for pkey in sorted(common_ports):
            r_p = ref_ports[pkey]
            t_p = tgt_ports[pkey]

            if r_p["direction"] != t_p["direction"]:
                findings.append(Finding(
                    RULE_ID,
                    f"{tgt_src}: Disparity in {diag_name}: Node '{nid}' port '{r_p['name']}' direction modified ({t_p['direction']} in report vs {r_p['direction']} in reference {ref_src}).",
                    location=tgt_src,
                    detail={"node": nid, "port": r_p["name"], "disparity_kind": "port_direction_mismatch"}
                ))

            if r_p["data_type"].lower() != t_p["data_type"].lower():
                findings.append(Finding(
                    RULE_ID,
                    f"{tgt_src}: Disparity in {diag_name}: Node '{nid}' port '{r_p['name']}' data type modified ('{t_p['data_type']}' in report vs '{r_p['data_type']}' in reference {ref_src}).",
                    location=tgt_src,
                    detail={"node": nid, "port": r_p["name"], "disparity_kind": "port_type_mismatch"}
                ))

    # 3. Compare Subgraphs
    ref_subgraphs = ref_diag["subgraphs"]
    tgt_subgraphs = target_diag["subgraphs"]

    ref_sg_keys = set(ref_subgraphs.keys())
    tgt_sg_keys = set(tgt_subgraphs.keys())

    missing_sgs = ref_sg_keys - tgt_sg_keys
    extra_sgs = tgt_sg_keys - ref_sg_keys

    for missing_sg in sorted(missing_sgs):
        findings.append(Finding(
            RULE_ID,
            f"{tgt_src}: Disparity in {diag_name}: Missing subgraph '{missing_sg}' present in reference {ref_src}.",
            location=tgt_src,
            detail={"subgraph": missing_sg, "disparity_kind": "missing_subgraph", "reference": ref_src}
        ))

    for extra_sg in sorted(extra_sgs):
        findings.append(Finding(
            RULE_ID,
            f"{tgt_src}: Disparity in {diag_name}: Extra undeclared subgraph '{extra_sg}' not present in reference {ref_src}.",
            location=tgt_src,
            detail={"subgraph": extra_sg, "disparity_kind": "extra_subgraph", "reference": ref_src}
        ))

    # 4. Compare Connections / Links (Edges)
    ref_conns = ref_diag["connections"]
    tgt_conns = target_diag["connections"]

    # Build edge lookup: (norm_from, norm_to) -> conn dict
    ref_edge_map: Dict[Tuple[str, str], List[Dict[str, Any]]] = {}
    for c in ref_conns:
        ref_edge_map.setdefault((c["norm_from"], c["norm_to"]), []).append(c)

    tgt_edge_map: Dict[Tuple[str, str], List[Dict[str, Any]]] = {}
    for c in tgt_conns:
        tgt_edge_map.setdefault((c["norm_from"], c["norm_to"]), []).append(c)

    ref_edges = set(ref_edge_map.keys())
    tgt_edges = set(tgt_edge_map.keys())

    missing_edges = ref_edges - tgt_edges
    extra_edges = tgt_edges - ref_edges

    for src_norm, tgt_norm in sorted(missing_edges):
        r_conn = ref_edge_map[(src_norm, tgt_norm)][0]
        findings.append(Finding(
            RULE_ID,
            f"{tgt_src}: Disparity in {diag_name}: Missing connection link '{r_conn['from_node']}' -> '{r_conn['to_node']}' present in reference {ref_src}.",
            location=tgt_src,
            detail={"from": r_conn["from_node"], "to": r_conn["to_node"], "disparity_kind": "missing_connection", "reference": ref_src}
        ))

    for src_norm, tgt_norm in sorted(extra_edges):
        t_conn = tgt_edge_map[(src_norm, tgt_norm)][0]
        findings.append(Finding(
            RULE_ID,
            f"{tgt_src}: Disparity in {diag_name}: Extra connection link '{t_conn['from_node']}' -> '{t_conn['to_node']}' not present in reference {ref_src}.",
            location=tgt_src,
            detail={"from": t_conn["from_node"], "to": t_conn["to_node"], "disparity_kind": "extra_connection", "reference": ref_src}
        ))

    # Check connection labels on matching edges
    common_edges = ref_edges & tgt_edges
    for edge_key in sorted(common_edges):
        r_conns = ref_edge_map[edge_key]
        t_conns = tgt_edge_map[edge_key]

        # If edge has connection ID or descriptive label, check for parity
        r_conn = r_conns[0]
        t_conn = t_conns[0]

        r_label = r_conn["label"]
        t_label = t_conn["label"]

        if r_conn.get("conn_id") and t_conn.get("conn_id"):
            if r_conn["conn_id"] != t_conn["conn_id"]:
                findings.append(Finding(
                    RULE_ID,
                    f"{tgt_src}: Disparity in {diag_name}: Connection link '{t_conn['from_node']}' -> '{t_conn['to_node']}' ID mismatch ('{t_conn['conn_id']}' in report vs '{r_conn['conn_id']}' in reference {ref_src}).",
                    location=tgt_src,
                    detail={"from": t_conn["from_node"], "to": t_conn["to_node"], "disparity_kind": "connection_id_mismatch"}
                ))
        elif r_label and t_label:
            norm_r = _normalize_text(r_label)
            norm_t = _normalize_text(t_label)
            if norm_r != norm_t:
                findings.append(Finding(
                    RULE_ID,
                    f"{tgt_src}: Disparity in {diag_name}: Connection link '{t_conn['from_node']}' -> '{t_conn['to_node']}' label mismatch: '{t_label}' vs reference '{r_label}'.",
                    location=tgt_src,
                    detail={"from": t_conn["from_node"], "to": t_conn["to_node"], "disparity_kind": "connection_label_mismatch"}
                ))

    return findings


def validate_cross_document_diagram_parity(workspace_path: Union[str, Path]) -> List[str]:
    """Validate cross-document Mermaid diagram parity across the workspace repository.

    Extracts architecture diagrams from docs/conops/ and compares them against
    executive deliverable reports in docs/reports/ and docs/management/.
    """
    root_dir = Path(workspace_path).resolve()
    findings: List[Finding] = []

    # 1. Discover ConOps documents
    conops_candidates: List[Path] = []
    conops_dir = root_dir / "docs" / "conops"
    if conops_dir.is_dir():
        for p in conops_dir.rglob("*.md"):
            if not p.name.startswith("."):
                conops_candidates.append(p)
    conops_root_cand = root_dir / "docs" / "CONOPS.md"
    if conops_root_cand.is_file() and conops_root_cand not in conops_candidates:
        conops_candidates.append(conops_root_cand)

    # 2. Discover Executive Deliverable / Report documents
    report_candidates: List[Path] = []
    for sdir in ("reports", "management"):
        target_dir = root_dir / "docs" / sdir
        if target_dir.is_dir():
            for p in target_dir.rglob("*.md"):
                if not p.name.startswith(".") and "defects" not in p.parts:
                    report_candidates.append(p)

    if not conops_candidates or not report_candidates:
        return []

    # 3. Extract and parse diagrams from ConOps documents
    conops_diagrams: List[Dict[str, Any]] = []
    for c_path in conops_candidates:
        try:
            rel_p = str(c_path.relative_to(root_dir))
            text = c_path.read_text(encoding="utf-8")
            blocks = _extract_mermaid_blocks(text, rel_p)
            for b in blocks:
                struct = _parse_diagram_structure(b)
                if struct:
                    conops_diagrams.append(struct)
        except Exception:
            pass

    if not conops_diagrams:
        return []

    # Identify primary ConOps SV-1 / System Interface Diagram
    conops_sv1_diag = next((d for d in conops_diagrams if _is_sv1_diagram(d)), None)

    # 4. Extract, parse, and compare diagrams in Report documents
    for r_path in report_candidates:
        try:
            rel_p = str(r_path.relative_to(root_dir))
            text = r_path.read_text(encoding="utf-8")
            blocks = _extract_mermaid_blocks(text, rel_p)
            for b in blocks:
                struct = _parse_diagram_structure(b)
                if not struct:
                    continue

                # Check if this report diagram is an SV-1 / System Interface Diagram
                if _is_sv1_diagram(struct):
                    if conops_sv1_diag:
                        diag_findings = _compare_diagrams(
                            conops_sv1_diag,
                            struct,
                            diag_name="DoDAF SV-1 System Interface Block Diagram"
                        )
                        findings.extend(diag_findings)
                    continue

                # Check for high node overlap with any ConOps diagram
                best_match = None
                best_score = 0.0
                for c_diag in conops_diagrams:
                    score = _compute_node_overlap(c_diag, struct)
                    if score > best_score:
                        best_score = score
                        best_match = c_diag

                if best_match and best_score >= 0.85 and len(struct["node_ids"]) >= 5:
                    diag_findings = _compare_diagrams(
                        best_match,
                        struct,
                        diag_name=f"Architecture Diagram ('{struct.get('heading', 'Context')}')"
                    )
                    findings.extend(diag_findings)

        except Exception:
            pass

    return [str(f) for f in findings]


class CrossDocumentDiagramParityValidator(IValidator):
    """
    Check 25: Cross-Document Diagram Parity Gate Validator.
    Ensures 1:1 parity between Mermaid architecture diagrams replicated across
    ConOps specifications and executive reports.
    """

    def __init__(self, workspace_repo: Optional[WorkspaceRepository] = None):
        self.workspace_repo = workspace_repo

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """Execute validation across repo workspace."""
        self.workspace_repo = repo
        res = validate_cross_document_diagram_parity(repo.workspace_dir)
        return [Finding(RULE_ID, msg, location=msg.split(":", 2)[0] if ":" in msg else "") for msg in res]
