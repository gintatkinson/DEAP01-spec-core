"""
Gate 30: Architecture Viewpoint & Diagram Completeness Validator.
Governing Standards: DoDAF 2.02 / OMG UAF v2.0 / ISO/IEC/IEEE 29148 / MIL-STD-882E / STPA / SORA.

Enforces fail-closed validation of the 11 canonical architecture diagrams across the 5 viewpoints:
1. Operational Viewpoint (OV):
   - Diagram 1 (OV-1 System Context)
   - Diagram 2 (OV-2 Operational Node Connectivity)
   - Diagram 6 (OV-5b Operational Activity Functional Flow)
   - Diagram 7 (OV-6c Scenario Lifeline Sequences)
2. System Viewpoint (SV):
   - Diagram 3 (SV-1 Super-System Segment Allocation)
   - Diagram 4 (SV-2 Subsystem Interconnect & Bus Architecture / IBD)
   - Diagram 8 (SV-10b Master System Lifecycle State Machine)
3. Interface Viewpoint (ICD):
   - Diagram 5 (N^2 Physical Interface Matrix)
4. Safety Viewpoint (STPA):
   - Diagram 9 (Hierarchical Safety Control Structure)
   - Diagram 10 (Run-Time Assurance RTA Statechart)
5. Spatial / Environmental Viewpoint (SORA):
   - Diagram 11 (4D Spatial Volume & GRB Containment)
"""

import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Set, Tuple, Union

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.workspace import WorkspaceRepository, extract_metadata_from_content
    from .mermaid_syntax_validator import check_mermaid_text
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.workspace import WorkspaceRepository, extract_metadata_from_content
    from parity_auditor.validators.mermaid_syntax_validator import check_mermaid_text


# Rule IDs
RULE_CORPUS_MISSING = "architecture-viewpoint-corpus-missing"
RULE_DIAGRAM_MISSING = "architecture-viewpoint-missing"
RULE_UNCLOSED_FENCE = "architecture-viewpoint-unclosed-fence"
RULE_EMPTY_DIAGRAM = "architecture-viewpoint-empty-diagram"
RULE_SYNTAX_ERROR = "architecture-viewpoint-syntax-error"
RULE_INCOMPLETE_DIAGRAM = "architecture-viewpoint-incomplete"
RULE_AST_UNBOUND = "architecture-viewpoint-ast-unbound"

ALLOWED_DIAGRAM_KINDS = {
    "graph", "flowchart", "sequencediagram", "statediagram", "statediagram-v2",
    "classdiagram", "erdiagram", "gantt", "pie", "gitgraph", "c4context",
    "journey", "mindmap", "timeline", "quadrantchart", "xychart", "block-beta",
    "packet-beta", "kanban", "architecture-beta"
}


# =============================================================================
# Canonical 11 Architecture Diagrams Registry
# =============================================================================

CANONICAL_DIAGRAMS: Dict[str, Dict[str, Any]] = {
    # 1. Operational Viewpoint (OV)
    "OV-1": {
        "id": "OV-1",
        "diagram_num": 1,
        "name": "System Context",
        "viewpoint": "Operational Viewpoint (OV)",
        "standards": "DoDAF 2.02 / OMG UAF v2.0 / ISO/IEC/IEEE 29148 §6.4.3",
        "expected_types": {"graph", "flowchart"},
        "min_nodes": 2,
        "governing_keywords": [r"\bOV-1\b", r"\bOV1\b", r"\bSystem\s+Context\b", r"\bOperational\s+Context\b"],
        "default_location": "docs/conops/CONOPS.md",
        "description": "System boundary / core system node interacting with external actors, operators, and operational environment",
    },
    "OV-2": {
        "id": "OV-2",
        "diagram_num": 2,
        "name": "Operational Node Connectivity",
        "viewpoint": "Operational Viewpoint (OV)",
        "standards": "DoDAF 2.02 / OMG UAF v2.0",
        "expected_types": {"graph", "flowchart"},
        "min_nodes": 2,
        "governing_keywords": [r"\bOV-2\b", r"\bOV2\b", r"\bOperational\s+Node\s+Connectivity\b", r"\bNode\s+Connectivity\b", r"\bOp-Tx\b"],
        "default_location": "docs/conops/CONOPS.md",
        "description": "Operational nodes with directional information exchanges (Op-Tx telemetry, commands, video, coordination)",
    },
    "OV-5b": {
        "id": "OV-5b",
        "diagram_num": 6,
        "name": "Operational Activity Functional Flow",
        "viewpoint": "Operational Viewpoint (OV)",
        "standards": "DoDAF 2.02 / OMG UAF v2.0",
        "expected_types": {"graph", "flowchart", "statediagram", "statediagram-v2"},
        "min_nodes": 2,
        "governing_keywords": [r"\bOV-5b\b", r"\bOV5b\b", r"\bOperational\s+Activity\b", r"\bFunctional\s+Flow\b", r"\bActivity\s+Functional\s+Flow\b", r"\bActivity\s+Flow\b"],
        "default_location": "docs/conops/CONOPS.md",
        "description": "Operational activities (OA-01..N or named activities) showing functional sequence and flow",
    },
    "OV-6c": {
        "id": "OV-6c",
        "diagram_num": 7,
        "name": "Scenario Lifeline Sequences",
        "viewpoint": "Operational Viewpoint (OV)",
        "standards": "DoDAF 2.02 / OMG UAF v2.0",
        "expected_types": {"sequencediagram"},
        "min_nodes": 2,  # min participants
        "governing_keywords": [r"\bOV-6c\b", r"\bOV6c\b", r"\bScenario\s+Lifeline\b", r"\bScenario\s+Sequence\b", r"\bOperational\s+Scenario\b", r"\bLifeline\s+Sequence\b"],
        "default_location": "docs/conops/CONOPS.md",
        "description": "Multi-threaded operational scenario sequence diagram with lifelines and message exchanges",
    },

    # 2. System Viewpoint (SV)
    "SV-1": {
        "id": "SV-1",
        "diagram_num": 3,
        "name": "Super-System Segment Allocation",
        "viewpoint": "System Viewpoint (SV)",
        "standards": "DoDAF 2.02 / OMG UAF v2.0 / IEEE 1362 §5.3",
        "expected_types": {"graph", "flowchart"},
        "min_nodes": 2,
        "requires_subgraphs": True,
        "governing_keywords": [r"\bSV-1\b", r"\bSV1\b", r"\bSuper-System\s+Segment\b", r"\bSegment\s+Allocation\b", r"\bSuper-System\s+Architecture\b"],
        "default_location": "docs/conops/CONOPS.md",
        "description": "Operational segments (subgraphs) allocating physical subsystems and computational nodes",
    },
    "SV-2": {
        "id": "SV-2",
        "diagram_num": 4,
        "name": "Subsystem Interconnect & Bus Architecture / IBD",
        "viewpoint": "System Viewpoint (SV)",
        "standards": "DoDAF 2.02 / SysML v2 IBD / OMG UAF v2.0",
        "expected_types": {"graph", "flowchart"},
        "min_nodes": 2,
        "governing_keywords": [r"\bSV-2\b", r"\bSV2\b", r"\bSubsystem\s+Interconnect\b", r"\bBus\s+Architecture\b", r"\bIBD\b", r"\bInternal\s+Block\s+Diagram\b"],
        "default_location": "docs/conops/CONOPS.md",
        "description": "Subsystem interconnects, bus topologies (CAN, Ethernet, RS-485, Serial, SPI), and port bindings",
    },
    "SV-10b": {
        "id": "SV-10b",
        "diagram_num": 8,
        "name": "Master System Lifecycle State Machine",
        "viewpoint": "System Viewpoint (SV)",
        "standards": "DoDAF 2.02 / SysML State Machine / ISO/IEC/IEEE 15288",
        "expected_types": {"statediagram", "statediagram-v2", "graph", "flowchart"},
        "min_nodes": 3,
        "governing_keywords": [r"\bSV-10b\b", r"\bSV10b\b", r"\bMaster\s+System\s+Lifecycle\b", r"\bLifecycle\s+State\s+Machine\b", r"\bSystem\s+Lifecycle\s+State\b", r"\bLifecycle\s+Modes\b", r"\bOperational\s+State\s+Space\b"],
        "default_location": "docs/conops/CONOPS.md",
        "description": "Operational lifecycle states/modes (Init, Standby, Active, Nominal, Degraded, Emergency, Shutdown) and transitions",
    },

    # 3. Interface Viewpoint (ICD)
    "ICD-N2": {
        "id": "ICD-N2",
        "diagram_num": 5,
        "name": "N^2 Physical Interface Matrix",
        "viewpoint": "Interface Viewpoint (ICD)",
        "standards": "ISO/IEC/IEEE 29148 / NASA SE Handbook / OMG UAF Res-Cn",
        "expected_types": {"graph", "flowchart", "matrix_table"},
        "min_nodes": 2,
        "governing_keywords": [r"\bN\^2\b", r"\bN2\b", r"\bPhysical\s+Interface\s+Matrix\b", r"\bSubsystem\s+Interface\s+Matrix\b", r"\bSubsystem\s+Interaction\s+Matrix\b"],
        "default_location": "docs/interfaces/ICD_01_SYSTEM_INTERFACE_MATRIX.md",
        "description": "Canonical N^2 physical/subsystem interface matrix (diagonal subsystems, off-diagonal interfaces) or Mermaid connectivity grid",
    },

    # 4. Safety Viewpoint (STPA)
    "STPA-CONTROL": {
        "id": "STPA-CONTROL",
        "diagram_num": 9,
        "name": "Hierarchical Safety Control Structure",
        "viewpoint": "Safety Viewpoint (STPA)",
        "standards": "STPA (Leveson) / MIL-STD-882E Task 201",
        "expected_types": {"graph", "flowchart"},
        "min_nodes": 3,
        "governing_keywords": [r"\bHierarchical\s+(?:Safety\s+)?Control\s+Structure\b", r"\bControl\s+Structure\s+Topology\b", r"\bSTPA\s+Control\s+Structure\b"],
        "default_location": "docs/safety/STPA_MATRIX.md",
        "description": "Hierarchical control structure: controllers, actuators, controlled process (plant), sensors, downward control and upward feedback",
    },
    "STPA-RTA": {
        "id": "STPA-RTA",
        "diagram_num": 10,
        "name": "Run-Time Assurance RTA Statechart",
        "viewpoint": "Safety Viewpoint (STPA)",
        "standards": "ASTM F3269-17 / DO-178C / DO-331",
        "expected_types": {"statediagram", "statediagram-v2", "graph", "flowchart"},
        "min_nodes": 2,
        "governing_keywords": [r"\bRTA\b", r"\bRun-Time\s+Assurance\b", r"\bSafety\s+Statecharts?\b", r"\bStateflow\s+Synthesis\b", r"\bASTM\s+F3269\b"],
        "default_location": "docs/safety/STPA_MATRIX.md",
        "description": "Run-Time Assurance switching/monitoring architecture: Nominal, Intervention, Recovery, Safe State",
    },

    # 5. Spatial / Environmental Viewpoint (SORA)
    "SORA-4D": {
        "id": "SORA-4D",
        "diagram_num": 11,
        "name": "4D Spatial Volume & GRB Containment",
        "viewpoint": "Spatial / Environmental Viewpoint (SORA)",
        "standards": "JARUS SORA v2.5 Annex B / EASA SORA",
        "expected_types": {"graph", "flowchart"},
        "min_nodes": 2,
        "governing_keywords": [r"\b4D\s+Spatial\s+Volume\b", r"\bSpatial\s+Volume\b", r"\bGround\s+Risk\s+Buffer\b", r"\bGRB\s+Containment\b", r"\bSORA\s+4D\b", r"\bContainment\s+Volume\b"],
        "default_location": "docs/conops/CONOPS.md",
        "description": "4D spatial volumes: Flight Geography, Contingency Volume, Operational Volume, Ground Risk Buffer (GRB), 1:1 Rule",
    },
}


# =============================================================================
# Helper Models and Parsers
# =============================================================================

@dataclass
class ParsedMermaidBlock:
    file_path: str
    start_line: int
    end_line: int
    heading: str
    diagram_type: str
    raw_lines: List[str]
    non_comment_lines: List[str]
    nodes: Set[str] = field(default_factory=set)
    subgraphs: List[str] = field(default_factory=list)
    edges: List[Tuple[str, str, str]] = field(default_factory=list)
    states: Set[str] = field(default_factory=set)
    transitions: List[Tuple[str, str, str]] = field(default_factory=list)
    participants: Set[str] = field(default_factory=set)
    messages: List[Tuple[str, str, str]] = field(default_factory=list)


@dataclass
class ParsedMarkdownMatrixTable:
    file_path: str
    start_line: int
    heading: str
    headers: List[str]
    subsystems: List[str]
    cell_count: int


def _extract_blocks_and_syntax_findings(content: str, rel_path: str) -> Tuple[List[ParsedMermaidBlock], List[Finding]]:
    """
    Extracts all Mermaid diagram blocks from markdown content, checking for unclosed fences,
    empty diagram blocks, and syntax violations.
    """
    blocks: List[ParsedMermaidBlock] = []
    findings: List[Finding] = []
    lines = content.splitlines()

    fence_pattern = re.compile(r"^\s*```+\s*mermaid\s*$", re.IGNORECASE)
    end_fence_pattern = re.compile(r"^\s*```+\s*$")
    heading_pattern = re.compile(r"^(#{1,6})\s+(.*)$")

    current_heading = ""
    i = 0
    num_lines = len(lines)

    while i < num_lines:
        line = lines[i]
        m_head = heading_pattern.match(line)
        if m_head:
            current_heading = m_head.group(2).strip()

        if fence_pattern.match(line):
            start_lineno = i + 1
            body: List[str] = []
            i += 1
            fence_closed = False
            while i < num_lines:
                if end_fence_pattern.match(lines[i]):
                    fence_closed = True
                    break
                body.append(lines[i])
                i += 1

            if not fence_closed:
                findings.append(Finding(
                    RULE_UNCLOSED_FENCE,
                    f"{rel_path}:{start_lineno}: Unclosed ```mermaid code fence detected (reached EOF without closing fence).",
                    location=f"{rel_path}:{start_lineno}",
                    detail={"rel_path": rel_path, "line": start_lineno}
                ))
                break

            end_lineno = i + 1

            # Inspect diagram body
            non_comment_lines: List[str] = []
            for bline in body:
                stripped = bline.strip()
                if stripped and not stripped.startswith("%%"):
                    non_comment_lines.append(stripped)

            if not non_comment_lines:
                findings.append(Finding(
                    RULE_EMPTY_DIAGRAM,
                    f"{rel_path}:{start_lineno}: Empty Mermaid diagram block (contains zero executable statements).",
                    location=f"{rel_path}:{start_lineno}",
                    detail={"rel_path": rel_path, "line": start_lineno}
                ))
                continue

            # First non-comment line must declare the diagram kind
            first_line = non_comment_lines[0]
            first_token = first_line.split()[0].lower() if first_line.split() else ""
            if first_token not in ALLOWED_DIAGRAM_KINDS:
                findings.append(Finding(
                    RULE_SYNTAX_ERROR,
                    f"{rel_path}:{start_lineno}: Invalid Mermaid diagram declaration: '{first_line}'. Expected diagram kind.",
                    location=f"{rel_path}:{start_lineno}",
                    detail={"rel_path": rel_path, "line": start_lineno, "token": first_token}
                ))
                continue

            # Check character-level syntax errors (e.g. semicolons in Notes)
            for idx, bline in enumerate(body, start=start_lineno):
                s_bline = bline.strip()
                if re.match(r"^\s*note\b", s_bline, re.IGNORECASE) and ";" in s_bline:
                    findings.append(Finding(
                        RULE_SYNTAX_ERROR,
                        f"{rel_path}:{idx}: Semicolon ';' is forbidden inside Mermaid Note statement (breaks rendering parser).",
                        location=f"{rel_path}:{idx}",
                        detail={"rel_path": rel_path, "line": idx}
                    ))

            # Parse elements inside block
            parsed = _parse_mermaid_block_elements(
                file_path=rel_path,
                start_line=start_lineno,
                end_line=end_lineno,
                heading=current_heading,
                diagram_type=first_token,
                raw_lines=body,
                non_comment_lines=non_comment_lines,
            )
            blocks.append(parsed)

        i += 1

    return blocks, findings


def _parse_mermaid_block_elements(
    file_path: str,
    start_line: int,
    end_line: int,
    heading: str,
    diagram_type: str,
    raw_lines: List[str],
    non_comment_lines: List[str],
) -> ParsedMermaidBlock:
    block = ParsedMermaidBlock(
        file_path=file_path,
        start_line=start_line,
        end_line=end_line,
        heading=heading,
        diagram_type=diagram_type,
        raw_lines=raw_lines,
        non_comment_lines=non_comment_lines,
    )

    if diagram_type in ("graph", "flowchart"):
        # Extract subgraphs, nodes, and edges
        subgraph_pattern = re.compile(r"^\s*subgraph\s+(?:([A-Za-z0-9_\-]+)\s*(?:\[(.*?)\])?|(?:\"([^\"]+)\"))", re.IGNORECASE)
        edge_pattern = re.compile(r"([A-Za-z0-9_\-]+)\s*(?:-->|---|-.->|==>)\s*(?:\|[^|]*\|\s*)?([A-Za-z0-9_\-]+)")
        node_decl_pattern = re.compile(r"([A-Za-z0-9_\-]+)\s*(\[|\(|\{|\(\()(\"?[^\"\)\]\}]*\"?)(\]|\)|\}|\)\))")

        for line in non_comment_lines:
            # Subgraph
            m_sub = subgraph_pattern.match(line)
            if m_sub:
                sub_id = m_sub.group(1) or m_sub.group(3) or ""
                sub_label = m_sub.group(2) or sub_id
                block.subgraphs.append(sub_label.strip('"'))
                if sub_id and sub_id != sub_label and sub_id not in block.subgraphs:
                    block.subgraphs.append(sub_id)

            # Edges
            for m_edge in edge_pattern.finditer(line):
                src, dst = m_edge.group(1), m_edge.group(2)
                block.edges.append((src, dst, ""))
                block.nodes.add(src)
                block.nodes.add(dst)

            # Nodes
            for m_node in node_decl_pattern.finditer(line):
                nid = m_node.group(1)
                if nid.lower() not in ("subgraph", "direction", "end", "class", "style"):
                    block.nodes.add(nid)

    elif diagram_type in ("statediagram", "statediagram-v2"):
        transition_pattern = re.compile(r"([A-Za-z0-9_]+|\[\*\])\s*-->\s*([A-Za-z0-9_]+|\[\*\])(?:\s*:\s*(.*))?")
        state_decl_pattern = re.compile(r"^\s*state\s+([A-Za-z0-9_]+)", re.IGNORECASE)

        for line in non_comment_lines:
            m_trans = transition_pattern.search(line)
            if m_trans:
                src, dst = m_trans.group(1), m_trans.group(2)
                trigger = m_trans.group(3) or ""
                block.transitions.append((src, dst, trigger.strip()))
                if src != "[*]":
                    block.states.add(src)
                if dst != "[*]":
                    block.states.add(dst)

            m_state = state_decl_pattern.match(line)
            if m_state:
                block.states.add(m_state.group(1))

    elif diagram_type == "sequencediagram":
        participant_pattern = re.compile(r"^\s*(?:participant|actor)\s+(?:([A-Za-z0-9_]+)\s+as\s+.*|([A-Za-z0-9_]+))", re.IGNORECASE)
        msg_pattern = re.compile(r"([A-Za-z0-9_]+)\s*(?:->>|-->>|->|-->|-\)|--\))\s*([A-Za-z0-9_]+)\s*:\s*(.*)")

        for line in non_comment_lines:
            m_part = participant_pattern.match(line)
            if m_part:
                pid = m_part.group(1) or m_part.group(2)
                if pid:
                    block.participants.add(pid)

            m_msg = msg_pattern.search(line)
            if m_msg:
                src, dst, msg_text = m_msg.group(1), m_msg.group(2), m_msg.group(3)
                block.messages.append((src, dst, msg_text.strip()))
                block.participants.add(src)
                block.participants.add(dst)

    return block


def _extract_markdown_matrix_tables(content: str, rel_path: str) -> List[ParsedMarkdownMatrixTable]:
    """
    Extracts markdown tables that match an N^2 interaction matrix structure.
    """
    matrix_tables: List[ParsedMarkdownMatrixTable] = []
    lines = content.splitlines()
    heading_pattern = re.compile(r"^(#{1,6})\s+(.*)$")
    current_heading = ""

    i = 0
    num_lines = len(lines)

    while i < num_lines:
        line = lines[i]
        m_head = heading_pattern.match(line)
        if m_head:
            current_heading = m_head.group(2).strip()

        if line.strip().startswith("|") and line.strip().endswith("|"):
            start_lineno = i + 1
            tbl_lines: List[str] = []
            while i < num_lines and lines[i].strip().startswith("|") and lines[i].strip().endswith("|"):
                tbl_lines.append(lines[i])
                i += 1

            if len(tbl_lines) >= 3:
                # Check header
                header_cols = [c.strip().strip("*`_") for c in tbl_lines[0].split("|")[1:-1]]
                if len(header_cols) >= 3:
                    # Look for N^2 indicators: header has subsystems and rows have matching subsystems
                    first_col = header_cols[0].lower()
                    if any(k in first_col for k in ("subsystem", "node", "performer", "n^2", "matrix", "interface")):
                        row_subsystems = []
                        for rline in tbl_lines[2:]:
                            r_cols = [c.strip().strip("*`_") for c in rline.split("|")[1:-1]]
                            if r_cols:
                                row_subsystems.append(r_cols[0])

                        # Overlap between row subsystems and col subsystems
                        col_subsystems = header_cols[1:]
                        overlap = set(row_subsystems).intersection(set(col_subsystems))
                        if len(overlap) >= 2 or "n^2" in current_heading.lower() or "matrix" in current_heading.lower():
                            matrix_tables.append(ParsedMarkdownMatrixTable(
                                file_path=rel_path,
                                start_line=start_lineno,
                                heading=current_heading,
                                headers=header_cols,
                                subsystems=list(col_subsystems),
                                cell_count=len(tbl_lines) - 2,
                            ))

        i += 1

    return matrix_tables


# =============================================================================
# Diagram Classification & Matching Engine
# =============================================================================

def _matches_canonical_diagram(
    canon_id: str,
    block: ParsedMermaidBlock,
) -> bool:
    """
    Determines if a ParsedMermaidBlock matches a given canonical diagram specification.
    """
    spec = CANONICAL_DIAGRAMS[canon_id]
    heading_text = block.heading.lower()
    raw_text = "\n".join(block.raw_lines).lower()
    combined_text = f"{heading_text} {raw_text}"

    # 1. Explicit comment / diagram tag match
    # e.g., %% OV-1, %% Diagram 1, <!-- OV-1 -->
    tag_patterns = [
        rf"%%.*?\b{canon_id}\b",
        rf"%%.*?diagram\s+{spec['diagram_num']}\b",
        rf"<!--.*?\b{canon_id}\b.*?-->",
        rf"\bdiagram\s+{spec['diagram_num']}\b",
    ]
    for pat in tag_patterns:
        if re.search(pat, combined_text, re.IGNORECASE):
            return True

    # 2. Heading / Title keyword matches
    for kw in spec["governing_keywords"]:
        if re.search(kw, block.heading, re.IGNORECASE):
            return True

    # 3. Viewpoint-specific structural heuristics
    if canon_id == "OV-1":
        # System Context: must be graph/flowchart and contain context / external actor semantics
        if block.diagram_type in ("graph", "flowchart"):
            if re.search(r"\bcontext\b", combined_text) and any(
                term in combined_text for term in ("system", "external", "operator", "environment", "user", "c2")
            ):
                return True

    elif canon_id == "OV-2":
        # Node connectivity: contains Op-Tx or operational communications
        if block.diagram_type in ("graph", "flowchart"):
            if "optx" in combined_text or "op-tx" in combined_text or (
                "connectivity" in combined_text and ("node" in combined_text or "operational" in combined_text)
            ):
                return True

    elif canon_id == "SV-1":
        # Super-System Segment Allocation: subgraphs containing "segment"
        if block.diagram_type in ("graph", "flowchart"):
            segment_subgraphs = [sg for sg in block.subgraphs if "segment" in sg.lower()]
            if len(segment_subgraphs) >= 1 or "segment" in combined_text:
                if re.search(r"\bsv-?1\b|\bsegment\s+allocation\b|\bsuper-system\b", combined_text):
                    return True

    elif canon_id == "SV-2":
        # Bus Architecture / IBD: interconnects and buses
        if block.diagram_type in ("graph", "flowchart"):
            if any(bus in combined_text for bus in ("can", "ethernet", "rs-485", "rs485", "serial", "spi", "afdx", "bus")) or "port-" in combined_text:
                if re.search(r"\bsv-?2\b|\bibd\b|\binterconnect\b|\bbus\b", combined_text):
                    return True

    elif canon_id == "ICD-N2":
        # N^2 matrix represented as flowchart
        if block.diagram_type in ("graph", "flowchart"):
            if re.search(r"\bn\^?2\b|\binterface\s+matrix\b|\bsubsystem\s+interaction\b", combined_text):
                return True

    elif canon_id == "OV-5b":
        # Operational Activity Functional Flow: OA- nodes or functional flow
        if block.diagram_type in ("graph", "flowchart", "statediagram", "statediagram-v2"):
            if re.search(r"\boa-\d+\b|\boperational\s+activit", combined_text):
                return True

    elif canon_id == "OV-6c":
        # Scenario Lifeline: sequenceDiagram with scenario in heading or text
        if block.diagram_type == "sequencediagram":
            if any(kw in combined_text for kw in ("scenario", "lifeline", "ov-6c", "timeline", "interlock", "failover", "lifecycle")):
                return True

    elif canon_id == "SV-10b":
        # Master System Lifecycle State Machine: statediagram with lifecycle states
        if block.diagram_type in ("statediagram", "statediagram-v2", "graph", "flowchart"):
            if any(mode in combined_text for mode in ("nominal", "standby", "active", "degraded", "emergency", "shutdown", "init", "preflight", "lifecycle")):
                if re.search(r"\bsv-?10b\b|\blifecycle\b|\bstate\s+space\b|\bstate\s+machine\b", combined_text):
                    return True

    elif canon_id == "STPA-CONTROL":
        # Hierarchical Safety Control Structure: controller -> actuator / plant / sensor
        if block.diagram_type in ("graph", "flowchart"):
            if ("controller" in combined_text or "supervisor" in combined_text) and (
                "actuator" in combined_text or "plant" in combined_text or "sensor" in combined_text or "process" in combined_text
            ):
                if re.search(r"\bcontrol\s+structure\b|\bhierarchical\b|\bstpa\b", combined_text):
                    return True

    elif canon_id == "STPA-RTA":
        # Run-Time Assurance: RTA statechart
        if block.diagram_type in ("statediagram", "statediagram-v2", "graph", "flowchart"):
            if "rta" in combined_text or "assurance" in combined_text or "f3269" in combined_text:
                return True

    elif canon_id == "SORA-4D":
        # 4D Spatial Volume: Flight Geography, Contingency Volume, GRB
        if block.diagram_type in ("graph", "flowchart"):
            if any(term in combined_text for term in ("flight geography", "contingency volume", "operational volume", "ground risk buffer", "grb", "sora")):
                return True

    return False


# =============================================================================
# ArchitectureViewpointValidator Class (Gate 30)
# =============================================================================

class ArchitectureViewpointValidator(IValidator):
    """
    Gate 30 Validator enforcing presence, completeness, and syntax integrity
    for the 11 canonical architecture diagrams across 5 viewpoints.
    """

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        findings: List[Finding] = []
        workspace_dir = repo.workspace_dir
        is_upstream = repo.is_upstream_compiler_repo()
        allow_missing_specs: bool = kwargs.get("allow_missing_specs", False)
        spec_only: bool = kwargs.get("spec_only", False)

        # 1. Discover relevant specification files
        target_dirs = [
            os.path.join(workspace_dir, "docs", "conops"),
            os.path.join(workspace_dir, "docs", "interfaces"),
            os.path.join(workspace_dir, "docs", "safety"),
            os.path.join(workspace_dir, "docs", "architecture"),
        ]

        spec_files: List[str] = []
        for tdir in target_dirs:
            if os.path.isdir(tdir):
                for root, _, files in os.walk(tdir):
                    for f in sorted(files):
                        if f.endswith(".md") and not f.startswith("."):
                            spec_files.append(os.path.join(root, f))

        # Check root CONOPS.md or STPA_MATRIX.md if present
        root_conops = os.path.join(workspace_dir, "CONOPS.md")
        if os.path.isfile(root_conops):
            spec_files.append(root_conops)

        # Upstream Clean Landing Zone Invariant:
        # In upstream template repos, if landing zones are clean (.gitkeep/README.md), exit 0.
        if is_upstream:
            concrete_specs = [
                sf for sf in spec_files
                if not os.path.basename(sf).lower() in ("readme.md", ".gitkeep")
                and "template" not in os.path.basename(sf).lower()
            ]
            if not concrete_specs:
                return []

        # If downstream and no specification documents found
        if not spec_files:
            if allow_missing_specs:
                return []
            return [Finding(
                RULE_CORPUS_MISSING,
                "Architecture specification corpus is missing in workspace ('docs/conops', 'docs/interfaces', 'docs/safety').",
                location="docs/conops",
                detail={"target_dirs": target_dirs}
            )]

        # 2. Extract and syntax-check all Mermaid blocks and matrix tables
        all_blocks: List[ParsedMermaidBlock] = []
        all_matrix_tables: List[ParsedMarkdownMatrixTable] = []

        for fpath in spec_files:
            rel_path = os.path.relpath(fpath, workspace_dir)
            try:
                with open(fpath, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception as e:
                findings.append(Finding(
                    RULE_SYNTAX_ERROR,
                    f"Failed to read specification file '{rel_path}': {e}",
                    location=rel_path,
                    detail={"error": str(e)}
                ))
                continue

            # Extract Mermaid blocks & syntax findings
            blocks, syntax_findings = _extract_blocks_and_syntax_findings(content, rel_path)
            findings.extend(syntax_findings)
            all_blocks.extend(blocks)

            # Extract N^2 markdown tables
            matrix_tables = _extract_markdown_matrix_tables(content, rel_path)
            all_matrix_tables.extend(matrix_tables)

        # 3. Classify detected diagrams against the 11 canonical diagrams
        matched_diagrams: Dict[str, List[ParsedMermaidBlock]] = {k: [] for k in CANONICAL_DIAGRAMS}
        matched_tables: Dict[str, List[ParsedMarkdownMatrixTable]] = {k: [] for k in CANONICAL_DIAGRAMS}

        for block in all_blocks:
            for canon_id in CANONICAL_DIAGRAMS:
                if _matches_canonical_diagram(canon_id, block):
                    matched_diagrams[canon_id].append(block)

        for tbl in all_matrix_tables:
            # Table matches N^2 matrix
            matched_tables["ICD-N2"].append(tbl)

        # 4. Validate Completeness & Degeneracy for each canonical diagram
        for canon_id, spec in CANONICAL_DIAGRAMS.items():
            diagram_blocks = matched_diagrams[canon_id]
            table_matches = matched_tables[canon_id]

            if not diagram_blocks and not table_matches:
                findings.append(Finding(
                    RULE_DIAGRAM_MISSING,
                    f"Missing canonical architecture diagram: Diagram {spec['diagram_num']} ({canon_id}: {spec['name']}) across {spec['viewpoint']} per {spec['standards']}.",
                    location=spec["default_location"],
                    detail={"diagram_id": canon_id, "diagram_num": spec["diagram_num"], "viewpoint": spec["viewpoint"]}
                ))
                continue

            # If diagram blocks matched, assert completeness
            for block in diagram_blocks:
                b_loc = f"{block.file_path}:{block.start_line}"

                # Diagram type conformance
                if block.diagram_type not in spec["expected_types"] and "matrix_table" not in spec["expected_types"]:
                    findings.append(Finding(
                        RULE_INCOMPLETE_DIAGRAM,
                        f"{b_loc}: Diagram {canon_id} ({spec['name']}) declaration '{block.diagram_type}' invalid; expected one of {sorted(list(spec['expected_types']))}.",
                        location=b_loc,
                        detail={"diagram_id": canon_id, "diagram_type": block.diagram_type}
                    ))

                # Node / Participant count check
                if block.diagram_type == "sequencediagram":
                    if len(block.participants) < spec["min_nodes"] or len(block.messages) < 1:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) sequence diagram is degenerate (must have at least {spec['min_nodes']} participants and message sequences; found {len(block.participants)} participants, {len(block.messages)} messages).",
                            location=b_loc,
                            detail={"diagram_id": canon_id, "participants": len(block.participants), "messages": len(block.messages)}
                        ))
                elif block.diagram_type in ("statediagram", "statediagram-v2"):
                    if len(block.states) < spec["min_nodes"]:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) state machine is degenerate (must declare at least {spec['min_nodes']} distinct states; found {len(block.states)}).",
                            location=b_loc,
                            detail={"diagram_id": canon_id, "states": len(block.states)}
                        ))
                else:
                    if len(block.nodes) < spec["min_nodes"]:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) is degenerate (must contain at least {spec['min_nodes']} connected nodes; found {len(block.nodes)}).",
                            location=b_loc,
                            detail={"diagram_id": canon_id, "nodes": len(block.nodes)}
                        ))

                # Diagram 3 (SV-1) Segment Subgraph Invariant
                if canon_id == "SV-1":
                    segment_subgraphs = [sg for sg in block.subgraphs if "segment" in sg.lower()]
                    if not segment_subgraphs and not block.subgraphs:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) lacks operational segment subgraphs allocating physical subsystems.",
                            location=b_loc,
                            detail={"diagram_id": canon_id, "subgraphs": block.subgraphs}
                        ))

                    bidi_edges = [l.strip() for l in block.non_comment_lines if re.search(r"<-->|<==>", l)]
                    if bidi_edges:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) contains {len(bidi_edges)} bidirectional links ('<-->') triggering Dagre rank collapse onto a single horizontal line (violates Rule E3). Use directed flows ('-->') instead.",
                            location=b_loc,
                            detail={"diagram_id": canon_id, "bidi_edges": bidi_edges[:3]}
                        ))
                    tier_subgraphs = [sg for sg in block.subgraphs if re.match(r"^Tier_\d+\b", sg, re.I)]
                    if len(tier_subgraphs) > 1:
                        has_tier_spacer = any("~~~" in l for l in block.non_comment_lines)
                        if not has_tier_spacer:
                            findings.append(Finding(
                                RULE_INCOMPLETE_DIAGRAM,
                                f"{b_loc}: Diagram {canon_id} ({spec['name']}) declares {len(tier_subgraphs)} tier subgraphs without vertical rank spacers ('~~~') between tiers (violates Rule E3 max 3-column vertical tier partitioning).",
                                location=b_loc,
                                detail={"diagram_id": canon_id, "tier_subgraphs": tier_subgraphs}
                            ))

                # Diagram 9 (STPA-CONTROL) Closed-Loop Invariant
                if canon_id == "STPA-CONTROL":
                    if len(block.edges) < 2:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) lacks downward control actions and upward sensor feedback paths.",
                            location=b_loc,
                            detail={"diagram_id": canon_id, "edges": len(block.edges)}
                        ))

                # Diagram 11 (SORA-4D) Containment Volume Invariant
                if canon_id == "SORA-4D":
                    raw_lower = "\n".join(block.raw_lines).lower()
                    has_containment = any(t in raw_lower for t in ("geography", "contingency", "operational", "buffer", "grb", "adjacent"))
                    if not has_containment:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) lacks SORA containment volume definitions (Flight Geography, Contingency Volume, Operational Volume, GRB).",
                            location=b_loc,
                            detail={"diagram_id": canon_id}
                        ))

            # If table matched (for ICD-N2), verify matrix structure
            if canon_id == "ICD-N2" and not diagram_blocks and table_matches:
                for tbl in table_matches:
                    if len(tbl.subsystems) < spec["min_nodes"]:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{tbl.file_path}:{tbl.start_line}: N^2 Physical Interface Matrix table must have at least {spec['min_nodes']} subsystems; found {len(tbl.subsystems)}.",
                            location=f"{tbl.file_path}:{tbl.start_line}",
                            detail={"diagram_id": canon_id, "subsystems": len(tbl.subsystems)}
                        ))

        return findings

    def validate_markdown_content(self, content: str, rel_path: str = "docs/conops/CONOPS.md", target_diagrams: Optional[Set[str]] = None) -> List[Finding]:
        """
        Direct content validation helper for unit tests and semantic verification scripts.
        """
        findings: List[Finding] = []
        blocks, syntax_findings = _extract_blocks_and_syntax_findings(content, rel_path)
        findings.extend(syntax_findings)

        matrix_tables = _extract_markdown_matrix_tables(content, rel_path)

        matched_diagrams: Dict[str, List[ParsedMermaidBlock]] = {k: [] for k in CANONICAL_DIAGRAMS}
        matched_tables: Dict[str, List[ParsedMarkdownMatrixTable]] = {k: [] for k in CANONICAL_DIAGRAMS}

        for block in blocks:
            for canon_id in CANONICAL_DIAGRAMS:
                if target_diagrams is not None and canon_id not in target_diagrams:
                    continue
                if _matches_canonical_diagram(canon_id, block):
                    matched_diagrams[canon_id].append(block)

        for tbl in matrix_tables:
            matched_tables["ICD-N2"].append(tbl)

        for canon_id, spec in CANONICAL_DIAGRAMS.items():
            if target_diagrams is not None and canon_id not in target_diagrams:
                continue
            d_blocks = matched_diagrams[canon_id]
            t_matches = matched_tables[canon_id]

            if not d_blocks and not t_matches:
                findings.append(Finding(
                    RULE_DIAGRAM_MISSING,
                    f"Missing canonical architecture diagram: Diagram {spec['diagram_num']} ({canon_id}: {spec['name']}) across {spec['viewpoint']} per {spec['standards']}.",
                    location=rel_path,
                    detail={"diagram_id": canon_id, "diagram_num": spec["diagram_num"]}
                ))
                continue

            for block in d_blocks:
                b_loc = f"{block.file_path}:{block.start_line}"
                if block.diagram_type not in spec["expected_types"] and "matrix_table" not in spec["expected_types"]:
                    findings.append(Finding(
                        RULE_INCOMPLETE_DIAGRAM,
                        f"{b_loc}: Diagram {canon_id} ({spec['name']}) declaration '{block.diagram_type}' invalid; expected one of {sorted(list(spec['expected_types']))}.",
                        location=b_loc
                    ))

                if block.diagram_type == "sequencediagram":
                    if len(block.participants) < spec["min_nodes"] or len(block.messages) < 1:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) sequence diagram is degenerate (must have at least {spec['min_nodes']} participants and message sequences).",
                            location=b_loc
                        ))
                elif block.diagram_type in ("statediagram", "statediagram-v2"):
                    if len(block.states) < spec["min_nodes"]:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) state machine is degenerate (must declare at least {spec['min_nodes']} distinct states; found {len(block.states)}).",
                            location=b_loc
                        ))
                else:
                    if len(block.nodes) < spec["min_nodes"]:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) is degenerate (must contain at least {spec['min_nodes']} connected nodes; found {len(block.nodes)}).",
                            location=b_loc
                        ))

                if canon_id == "SV-1":
                    segment_subgraphs = [sg for sg in block.subgraphs if "segment" in sg.lower()]
                    if not segment_subgraphs and not block.subgraphs:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) lacks operational segment subgraphs allocating physical subsystems.",
                            location=b_loc,
                            detail={"diagram_id": canon_id, "subgraphs": block.subgraphs}
                        ))

                    bidi_edges = [l.strip() for l in block.non_comment_lines if re.search(r"<-->|<==>", l)]
                    if bidi_edges:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) contains {len(bidi_edges)} bidirectional links ('<-->') triggering Dagre rank collapse onto a single horizontal line (violates Rule E3). Use directed flows ('-->') instead.",
                            location=b_loc,
                            detail={"diagram_id": canon_id, "bidi_edges": bidi_edges[:3]}
                        ))
                    tier_subgraphs = [sg for sg in block.subgraphs if re.match(r"^Tier_\d+\b", sg, re.I)]
                    if len(tier_subgraphs) > 1:
                        has_tier_spacer = any("~~~" in l for l in block.non_comment_lines)
                        if not has_tier_spacer:
                            findings.append(Finding(
                                RULE_INCOMPLETE_DIAGRAM,
                                f"{b_loc}: Diagram {canon_id} ({spec['name']}) declares {len(tier_subgraphs)} tier subgraphs without vertical rank spacers ('~~~') between tiers (violates Rule E3 max 3-column vertical tier partitioning).",
                                location=b_loc,
                                detail={"diagram_id": canon_id, "tier_subgraphs": tier_subgraphs}
                            ))

                if canon_id == "STPA-CONTROL":
                    if len(block.edges) < 2:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) lacks downward control actions and upward sensor feedback paths.",
                            location=b_loc
                        ))

                if canon_id == "SORA-4D":
                    raw_lower = "\n".join(block.raw_lines).lower()
                    has_containment = any(t in raw_lower for t in ("geography", "contingency", "operational", "buffer", "grb", "adjacent"))
                    if not has_containment:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{b_loc}: Diagram {canon_id} ({spec['name']}) lacks SORA containment volume definitions.",
                            location=b_loc
                        ))

            if canon_id == "ICD-N2" and not d_blocks and t_matches:
                for tbl in t_matches:
                    if len(tbl.subsystems) < spec["min_nodes"]:
                        findings.append(Finding(
                            RULE_INCOMPLETE_DIAGRAM,
                            f"{tbl.file_path}:{tbl.start_line}: N^2 Physical Interface Matrix table must have at least {spec['min_nodes']} subsystems; found {len(tbl.subsystems)}.",
                            location=f"{tbl.file_path}:{tbl.start_line}"
                        ))

        return findings
