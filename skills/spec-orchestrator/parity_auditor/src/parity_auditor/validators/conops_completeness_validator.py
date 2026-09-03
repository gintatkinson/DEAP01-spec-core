r"""
Gate 26: ConOps & Mission Intent Completeness Validator Engine (ISO 29148 / NATO STANAG 4586 / OMG UAF).

Enforces:
1. ConopsCompletenessValidator:
   12 Mandatory Sections for Concept of Operations (CONOPS.md):
   - 1. Scope & System Identification
   - 2. Normative Standards & Regulatory Baseline
   - 3. Current Situation & Deficiency Analysis (Predecessors)
   - 4. Operational Justification & Priority Matrix (Trade-Offs)
   - 5. Operational Modes & Lifecycle Stages (\Phi_{lifecycle})
   - 6. 4D Operational Volume & SORA Ground Risk Buffer (GRB) Mathematics
   - 7. OMG UAF Operational Activity Taxonomy (with Gate 24 tags)
   - 8. Operational Information Exchange (Op-Tx) Matrix
   - 9. Operational Environments & Physical Constraints
   - 10. Multi-Threaded Operational Scenarios
   - 11. Maintenance & Sustainment Concepts (O/I/D Maintenance)
   - 12. 7-Row Emergency Decision & Contingency Matrix (EMG-01..07)

2. MissionIntentCompletenessValidator:
   10 Mandatory Sections for Mission Intent (MISSION_INTENT.md):
   - 1. Commander's Intent & Operational Objectives
   - 2. Mission Essential Task List (METL MET-01..N)
   - 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics
   - 4. Multi-Domain Operational Threat & Contested Environment Matrix
   - 5. PACE C2 Link Communications Plan (Primary, Alternate, Contingency, Emergency)
   - 6. Rules of Engagement (ROE) & Weapon/Sensor Interlocks
   - 7. Airspace Deconfliction & U-space Dynamic Geo-Zones
   - 8. Go/No-Go Decision Matrix
   - 9. Bingo Energy Mathematics & Secondary Divert Protocols (Statutory Reserve \ge 20%)
   - 10. Gate 24 MissionTask Traceability Tags

3. Pure Abstract Systems Engineering Archetypes across templates, models, and solvers.
"""

import math
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
    from ..parsers.research_inventory import parse_research_inventory
    from .coverage_digest_validator import _normalize_obligation_id, _parse_obligation_tags
    from .obligation_witness_validator import _parse_witness_tags
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.workspace import WorkspaceRepository, extract_metadata_from_content
    from parity_auditor.parsers.research_inventory import parse_research_inventory
    from parity_auditor.validators.coverage_digest_validator import _normalize_obligation_id, _parse_obligation_tags
    from parity_auditor.validators.obligation_witness_validator import _parse_witness_tags


# =============================================================================
# Mathematical Helpers
# =============================================================================

def calculate_sora_grb_radius(
    h_max_m: float,
    theta_impact_deg: float = 45.0,
    v_wind_max_mps: float = 15.0,
    g: float = 9.80665,
    d_glide_max_m: float = 0.0,
) -> float:
    """
    Computes minimum theoretical SORA Ground Risk Buffer radius in accordance with
    JARUS SORA v2.5 Annex B:
        R_GRB = h_max * tan(theta_impact) + v_wind_max * sqrt(2 * h_max / g) + d_glide_max
    """
    if h_max_m <= 0:
        return 0.0
    theta_rad = math.radians(theta_impact_deg)
    t_fall = math.sqrt((2.0 * h_max_m) / g)
    r_grb = (h_max_m * math.tan(theta_rad)) + (v_wind_max_mps * t_fall) + d_glide_max_m
    return r_grb


def calculate_bingo_energy_reserve_ratio(
    total_capacity_j: float,
    reserve_energy_j: float,
) -> float:
    """
    Computes the statutory energy reserve ratio:
        ratio = reserve_energy_j / total_capacity_j
    Statutory requirement: ratio >= 0.20 (20%).
    """
    if total_capacity_j <= 0:
        return 0.0
    return reserve_energy_j / total_capacity_j


def _extract_bingo_energy_parameters(sec9_content: str) -> Tuple[Optional[float], Optional[float]]:
    """
    Extracts total capacity E_capacity and statutory reserve E_reserve from Section 9.
    Uses table-aware extraction first to prevent false triggers on inequality constraints
    (e.g., E_reserve >= 0.20 * E_capacity). Reference Fixes #130.
    """
    capacity_val: Optional[float] = None
    reserve_val: Optional[float] = None

    sec9_tables, _ = _parse_commonmark_tables(sec9_content)
    for tbl in sec9_tables:
        for row in tbl:
            symbol = row.get("symbol", "").strip()
            param_name = (row.get("energy_parameter") or row.get("parameter") or row.get("name") or "").strip()
            val_str = (row.get("value") or row.get("val") or "").strip()

            m_val = re.search(r'([0-9]+(?:\.[0-9]+)?)', val_str)
            num_val: Optional[float] = None
            if m_val:
                try:
                    num_val = float(m_val.group(1))
                except ValueError:
                    pass

            if num_val is not None:
                if re.search(r'\bE[_\s]*capacity\b', symbol, re.IGNORECASE) or re.search(r'\btotal\s+(?:storage\s+|pack\s+|battery\s+)?capacity\b', param_name, re.IGNORECASE):
                    capacity_val = num_val
                elif re.search(r'\bE[_\s]*reserve\b', symbol, re.IGNORECASE) or re.search(r'\b(?:mandatory\s+)?statutory\s+reserve\b', param_name, re.IGNORECASE) or (re.search(r'\breserve\s+energy\b', param_name, re.IGNORECASE) and not re.search(r'ratio|percent|rule|constraint', param_name, re.IGNORECASE)):
                    reserve_val = num_val

    # Fallback to regex on text if not extracted from tables (stripping display math formulas to avoid matching 0.20 in E_reserve >= 0.20 * E_capacity)
    if capacity_val is None or reserve_val is None:
        clean_text = re.sub(r'\$\$[\s\S]*?\$\$', '', sec9_content)

        if capacity_val is None:
            m_cap = re.search(r'E[_\s]*capacity\s*[:=\|]?\s*([0-9]+(?:\.[0-9]+)?)', clean_text, re.IGNORECASE)
            if not m_cap:
                m_cap = re.search(r'(?:total\s+capacity|pack\s+capacity|battery\s+capacity)\s*[:=\|]?\s*([0-9]+(?:\.[0-9]+)?)', clean_text, re.IGNORECASE)
            if m_cap:
                try:
                    capacity_val = float(m_cap.group(1))
                except ValueError:
                    pass

        if reserve_val is None:
            m_res = re.search(r'E[_\s]*reserve\s*[:=\|]?\s*([0-9]+(?:\.[0-9]+)?)', clean_text, re.IGNORECASE)
            if not m_res:
                m_res = re.search(r'(?:statutory\s+reserve|reserve\s+energy)\s*[:=\|]?\s*([0-9]+(?:\.[0-9]+)?)', clean_text, re.IGNORECASE)
            if m_res:
                try:
                    reserve_val = float(m_res.group(1))
                except ValueError:
                    pass

    return capacity_val, reserve_val


def _extract_sora_parameters(sec6_content: str) -> Tuple[Optional[float], Optional[float], float, Optional[float]]:
    """
    Extracts h_max, v_wind, theta_impact, and R_GRB from Section 6.
    Uses table-aware extraction first. Reference Fixes #130.
    """
    h_max_val: Optional[float] = None
    v_wind_val: Optional[float] = None
    theta_val: float = 45.0
    r_grb_val: Optional[float] = None

    sec6_tables, _ = _parse_commonmark_tables(sec6_content)
    for tbl in sec6_tables:
        for row in tbl:
            symbol = row.get("symbol", "").strip()
            param_name = (row.get("parameter") or row.get("name") or "").strip()
            val_str = (row.get("value") or row.get("val") or "").strip()

            m_val = re.search(r'([0-9]+(?:\.[0-9]+)?)', val_str)
            num_val: Optional[float] = None
            if m_val:
                try:
                    num_val = float(m_val.group(1))
                except ValueError:
                    pass

            if num_val is not None:
                if re.search(r'\bh[_\s]*max\b', symbol, re.IGNORECASE) or re.search(r'\bmax(?:imum)?\s+altitude\b', param_name, re.IGNORECASE):
                    h_max_val = num_val
                elif re.search(r'\bv[_\s]*wind(?:[_\s]*max)?\b', symbol, re.IGNORECASE) or re.search(r'\bmax(?:imum)?\s+wind\b', param_name, re.IGNORECASE):
                    v_wind_val = num_val
                elif re.search(r'\btheta[_\s]*impact\b', symbol, re.IGNORECASE) or re.search(r'\bimpact\s+angle\b', param_name, re.IGNORECASE):
                    theta_val = num_val
                elif re.search(r'\bR[_\s]*GRB\b', symbol, re.IGNORECASE) or re.search(r'\bground\s+risk\s+buffer\b', param_name, re.IGNORECASE):
                    r_grb_val = num_val

    # Fallback to regex
    if h_max_val is None:
        m_h = re.search(r'h[_\s]*max[^\d]*([0-9]+(?:\.[0-9]+)?)', sec6_content, re.IGNORECASE)
        if not m_h:
            m_h = re.search(r'(?:max(?:imum)?\s+altitude|operating\s+ceiling)[^\d]*([0-9]+(?:\.[0-9]+)?)', sec6_content, re.IGNORECASE)
        if m_h:
            try:
                h_max_val = float(m_h.group(1))
            except ValueError:
                pass

    if v_wind_val is None:
        m_w = re.search(r'v[_\s]*wind(?:[_\s]*max)?[^\d]*([0-9]+(?:\.[0-9]+)?)', sec6_content, re.IGNORECASE)
        if not m_w:
            m_w = re.search(r'(?:max(?:imum)?\s+wind|wind\s+limit|wind\s+speed)[^\d]*([0-9]+(?:\.[0-9]+)?)', sec6_content, re.IGNORECASE)
        if m_w:
            try:
                v_wind_val = float(m_w.group(1))
            except ValueError:
                pass

    if r_grb_val is None:
        m_r = re.search(r'R[_\s]*GRB[^\d]*([0-9]+(?:\.[0-9]+)?)', sec6_content, re.IGNORECASE)
        if not m_r:
            m_r = re.search(r'(?:ground\s+risk\s+buffer\s+radius|buffer\s+radius)[^\d]*([0-9]+(?:\.[0-9]+)?)', sec6_content, re.IGNORECASE)
        if m_r:
            try:
                r_grb_val = float(m_r.group(1))
            except ValueError:
                pass

    return h_max_val, v_wind_val, theta_val, r_grb_val


# =============================================================================
# Template Placeholder Detection (Gate 26, Fixes #142)
# =============================================================================

TEMPLATE_PLACEHOLDER_REGEX = re.compile(r"\{\{[A-Za-z0-9_]+\}\}")


def _find_unresolved_template_placeholders(content: str) -> List[Tuple[int, str]]:
    r"""
    Finds all unresolved template placeholder tokens matching r"\{\{[A-Za-z0-9_]+\}\}"
    in the given content.
    Returns list of (line_number, placeholder_token) tuples.
    """
    results: List[Tuple[int, str]] = []
    for idx, line in enumerate(content.splitlines(), start=1):
        for match in TEMPLATE_PLACEHOLDER_REGEX.finditer(line):
            results.append((idx, match.group(0)))
    return results


# =============================================================================
# Domain Models for ConOps & Mission Intent
# =============================================================================

@dataclass
class OperationalVolume4D:
    nominal_waypoints: List[str] = field(default_factory=list)
    min_altitude_meters: float = 0.0
    max_altitude_meters: float = 120.0
    contingency_margin_meters: float = 50.0
    sora_grb_radius_meters: float = 200.0

    def is_volume_bounded(self) -> bool:
        return self.max_altitude_meters > self.min_altitude_meters and self.sora_grb_radius_meters > 0


@dataclass
class SoraRiskBuffer:
    max_altitude_agl: float = 120.0
    impact_angle_degrees: float = 45.0
    max_wind_speed_mps: float = 15.0
    free_fall_time_seconds: float = 4.95
    terminal_velocity_mps: float = 25.0
    impact_kinetic_energy_joules: float = 1562.5
    sora_grc_classification: str = "GRC-3"


@dataclass
class EmergencyDecisionRow:
    trigger_id: str
    contingency_trigger_name: str
    detection_mechanism: str
    automated_containment_action: str
    failsafe_recovery_state: str
    max_response_time_seconds: float
    human_in_the_loop_role: str


@dataclass
class UafActivityEntry:
    activity_id: str
    activity_name: str
    description: str
    allocation_tag: str


@dataclass
class OpTxExchangeEntry:
    exchange_id: str
    source_node: str
    dest_node: str
    information_item: str
    data_rate: str
    max_latency: str
    criticality: str


@dataclass
class ConopsDocumentModel:
    scope: str = ""
    normative_standards: List[str] = field(default_factory=list)
    current_situation: str = ""
    justification: str = ""
    operational_modes: List[str] = field(default_factory=list)
    operational_volume: Optional[OperationalVolume4D] = None
    sora_risk_buffer: Optional[SoraRiskBuffer] = None
    activity_taxonomy: List[UafActivityEntry] = field(default_factory=list)
    op_tx_matrix: List[OpTxExchangeEntry] = field(default_factory=list)
    operational_environments: str = ""
    scenarios: List[str] = field(default_factory=list)
    maintenance_concepts: str = ""
    emergency_matrix: List[EmergencyDecisionRow] = field(default_factory=list)


@dataclass
class METLTaskEntry:
    task_id: str
    task_name: str
    condition_statement: str
    standard_metric: str
    verification_method: str
    gate24_allocation_tag: str


@dataclass
class MoEMoPMetricEntry:
    metric_id: str
    metric_type: str
    metric_name: str
    mathematical_formula: str
    threshold_value: float
    objective_value: float
    measurement_unit: str


@dataclass
class PaceC2LinkPlan:
    primary_link: str = ""
    alternate_link: str = ""
    contingency_link: str = ""
    emergency_link: str = ""
    heartbeat_timeout_seconds: float = 5.0


@dataclass
class BingoEnergyModel:
    total_capacity_joules: float = 500000.0
    return_energy_joules: float = 150000.0
    divert_energy_joules: float = 60000.0
    reserve_energy_joules: float = 100000.0
    contingency_energy_joules: float = 40000.0
    bingo_threshold_joules: float = 350000.0
    primary_divert_waypoint: str = "DIVERT_ALPHA"
    secondary_divert_waypoint: str = "DIVERT_BRAVO"


@dataclass
class MissionIntentDocumentModel:
    commanders_intent: str = ""
    metl_roster: List[METLTaskEntry] = field(default_factory=list)
    moe_mop_metrics: List[MoEMoPMetricEntry] = field(default_factory=list)
    threat_matrix: List[Dict[str, str]] = field(default_factory=list)
    pace_link_plan: Optional[PaceC2LinkPlan] = None
    roe_rules: List[str] = field(default_factory=list)
    airspace_zones: str = ""
    go_no_go_matrix: List[Dict[str, str]] = field(default_factory=list)
    bingo_energy_model: Optional[BingoEnergyModel] = None
    gate24_task_tags: List[str] = field(default_factory=list)


@dataclass
class ValidationReport:
    gate_id: str = "Gate-26"
    status: str = "PASSED"
    total_checks: int = 0
    passed_checks: int = 0
    findings: List[Finding] = field(default_factory=list)
    conops_coverage: float = 1.0
    mission_intent_coverage: float = 1.0


# =============================================================================
# Parsing Utilities
# =============================================================================

def _extract_markdown_sections_list(text: str) -> List[Tuple[str, int, str]]:
    """
    Extracts top-level and H2 sections from markdown preserving list order and duplicate headers.
    Returns list of tuples: (section_heading, line_number, section_content).
    """
    sections: List[Tuple[str, int, str]] = []
    lines = text.splitlines()
    current_heading = ""
    current_line = 1
    current_chunk: List[str] = []
    in_code_block = False

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("```"):
            in_code_block = not in_code_block
            current_chunk.append(line)
            continue
        if in_code_block:
            current_chunk.append(line)
            continue

        m = re.match(r'^(#{1,2})\s+(.+)$', stripped)
        if m:
            if current_heading:
                sections.append((current_heading, current_line, "\n".join(current_chunk)))
            current_heading = m.group(2).strip()
            current_line = idx
            current_chunk = []
        else:
            current_chunk.append(line)

    if current_heading:
        sections.append((current_heading, current_line, "\n".join(current_chunk)))

    return sections


def _extract_markdown_sections(text: str) -> Dict[str, Tuple[int, str]]:
    """
    Extracts top-level and H2 sections from markdown.
    Returns mapping of section_heading -> (line_number, section_content).
    """
    sections_list = _extract_markdown_sections_list(text)
    sections: Dict[str, Tuple[int, str]] = {}
    for heading, line_no, content in sections_list:
        if heading not in sections:
            sections[heading] = (line_no, content)
    return sections


def _parse_commonmark_tables(text: str) -> Tuple[List[List[Dict[str, str]]], List[int]]:
    """
    Parses CommonMark tables from markdown text.
    Returns list of tables and list of malformed table line numbers.
    """
    tables: List[List[Dict[str, str]]] = []
    malformed_lines: List[int] = []
    lines = text.splitlines()
    i = 0

    while i < len(lines):
        line = lines[i].strip()
        if line.startswith("|") and line.endswith("|"):
            header_line = line
            if i + 1 < len(lines):
                sep_line = lines[i + 1].strip()
                if re.match(r"^\|(\s*:?-+:?\s*\|)+$", sep_line):
                    # Replace escaped pipes
                    clean_hdr_line = header_line.replace(r"\|", "\x00")
                    raw_headers = [col.replace("\x00", "|").strip() for col in clean_hdr_line.split("|")[1:-1]]
                    clean_headers = [re.sub(r'[*`_]', '', col).strip() for col in raw_headers]
                    norm_headers = [re.sub(r'[\s\-/]+', '_', h.lower()).strip('_') for h in clean_headers]
                    
                    rows: List[Dict[str, str]] = []
                    j = i + 2
                    is_malformed = False

                    while j < len(lines):
                        row_line = lines[j].strip()
                        if not (row_line.startswith("|") and row_line.endswith("|")):
                            if row_line.startswith("|") and not row_line.endswith("|"):
                                malformed_lines.append(j + 1)
                                is_malformed = True
                            break
                        clean_row_line = row_line.replace(r"\|", "\x00")
                        cols = [col.replace("\x00", "|").strip() for col in clean_row_line.split("|")[1:-1]]
                        if len(cols) != len(norm_headers):
                            # Allow trailing empty columns but flag significant mismatch
                            if abs(len(cols) - len(norm_headers)) > 1:
                                malformed_lines.append(j + 1)
                        row_dict: Dict[str, str] = {}
                        for idx_h, h in enumerate(norm_headers):
                            val = cols[idx_h] if idx_h < len(cols) else ""
                            row_dict[h] = val.strip("`").strip()
                        rows.append(row_dict)
                        j += 1

                    if not is_malformed:
                        tables.append(rows)
                    i = j
                    continue
                else:
                    if "|" in sep_line and not re.match(r"^\|(\s*:?-+:?\s*\|)+$", sep_line):
                        malformed_lines.append(i + 2)
        elif line.startswith("|") and not line.endswith("|"):
            malformed_lines.append(i + 1)
        i += 1

    return tables, malformed_lines


def _find_matching_sections_all(
    sections_list: List[Tuple[str, int, str]],
    sec_num: int,
    canonical_name: str,
    aliases: List[str],
) -> List[Tuple[str, int, str]]:
    """
    Finds all sections matching section number or aliases from a sections list.
    Preserves all matching occurrences to allow duplicate section detection.
    """
    matches: List[Tuple[str, int, str]] = []
    num_pattern = rf'^(?:section\s+)?{sec_num}[.\s:\-—]'
    any_num_pattern = r'^(?:section\s+)?[0-9]+[.\s:\-—]'

    for heading, line_no, content in sections_list:
        h_clean = heading.lower().strip()
        if re.search(num_pattern, h_clean):
            matches.append((heading, line_no, content))
        elif not re.search(any_num_pattern, h_clean):
            for alias in aliases:
                if re.search(rf'\b{re.escape(alias.lower())}\b', h_clean):
                    matches.append((heading, line_no, content))
                    break
    return matches


def _find_matching_section(
    sections: Union[Dict[str, Tuple[int, str]], List[Tuple[str, int, str]]],
    sec_num: int,
    canonical_name: str,
    aliases: List[str],
) -> Optional[Tuple[str, int, str]]:
    """
    Finds a section in the parsed sections dictionary or list by matching section number or aliases.
    Prioritizes direct section number prefix match before alias fallback.
    """
    if isinstance(sections, list):
        matches = _find_matching_sections_all(sections, sec_num, canonical_name, aliases)
        return matches[0] if matches else None

    # Pass 1: Direct prefix match: e.g. "1. Scope", "## 1.", "1 - Scope", "Section 1:"
    num_pattern = rf'^(?:section\s+)?{sec_num}[.\s:\-—]'
    for heading, (line_no, content) in sections.items():
        h_clean = heading.lower().strip()
        if re.search(num_pattern, h_clean):
            return heading, line_no, content

    # Pass 2: Check alias matches for headings without section numbers
    any_num_pattern = r'^(?:section\s+)?[0-9]+[.\s:\-—]'
    for heading, (line_no, content) in sections.items():
        h_clean = heading.lower().strip()
        if not re.search(any_num_pattern, h_clean):
            for alias in aliases:
                if re.search(rf'\b{re.escape(alias.lower())}\b', h_clean):
                    return heading, line_no, content

    return None


# =============================================================================
# Mermaid Block and Table Schema Validation Helpers (Fixes #114, #130)
# =============================================================================

VALID_MERMAID_TYPES = (
    "flowchart",
    "graph",
    "sequenceDiagram",
    "stateDiagram",
    "stateDiagram-v2",
    "classDiagram",
    "erDiagram",
    "gantt",
    "pie",
    "journey",
    "gitGraph",
    "quadrantChart",
    "mindmap",
    "timeline",
    "zenuml",
    "sankey-beta",
    "C4Context",
    "C4Container",
    "C4Component",
    "C4Dynamic",
    "C4Deployment",
    "requirementDiagram",
    "architecture-beta",
    "packet-beta",
    "kanban",
    "block-beta",
)


def _validate_mermaid_integrity(content: str, rel_path: str, prefix: str = "conops") -> List[Finding]:
    """
    Validates Mermaid code block syntax, recognized diagram types, and fence closing integrity.
    """
    findings: List[Finding] = []
    lines = content.splitlines()
    in_code = False
    code_lang = ""
    code_start_line = 0
    code_lines: List[str] = []

    for idx, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("```"):
            if not in_code:
                in_code = True
                code_lang = stripped[3:].strip().lower()
                code_start_line = idx
                code_lines = []
            else:
                # Closing fence
                if code_lang == "mermaid":
                    block_body = "\n".join(code_lines).strip()
                    if not block_body:
                        findings.append(Finding(
                            f"{prefix}-mermaid-malformed",
                            f"Empty Mermaid diagram block detected at line {code_start_line} in '{rel_path}'.",
                            location=f"{rel_path}:{code_start_line}",
                            detail={"line": code_start_line, "file": rel_path},
                        ))
                    else:
                        body_lines = [l.strip() for l in block_body.splitlines() if l.strip() and not l.strip().startswith("%%")]
                        if not body_lines:
                            findings.append(Finding(
                                f"{prefix}-mermaid-malformed",
                                f"Mermaid diagram block at line {code_start_line} in '{rel_path}' contains only comments or whitespace.",
                                location=f"{rel_path}:{code_start_line}",
                                detail={"line": code_start_line, "file": rel_path},
                            ))
                        else:
                            first_stmt = body_lines[0]
                            matched_type = any(
                                re.match(rf'^{re.escape(dt)}\b', first_stmt, re.IGNORECASE)
                                for dt in VALID_MERMAID_TYPES
                            )
                            if not matched_type:
                                findings.append(Finding(
                                    f"{prefix}-mermaid-malformed",
                                    f"Mermaid diagram block at line {code_start_line} in '{rel_path}' has unrecognized or invalid diagram type '{first_stmt}'.",
                                    location=f"{rel_path}:{code_start_line}",
                                    detail={"line": code_start_line, "statement": first_stmt, "file": rel_path},
                                ))
                in_code = False
                code_lang = ""
                code_start_line = 0
                code_lines = []
        elif in_code:
            code_lines.append(line)

    if in_code and code_lang == "mermaid":
        findings.append(Finding(
            f"{prefix}-mermaid-unclosed",
            f"Unclosed Mermaid code block detected starting at line {code_start_line} in '{rel_path}'.",
            location=f"{rel_path}:{code_start_line}",
            detail={"line": code_start_line, "file": rel_path},
        ))

    return findings


MANDATORY_CONOPS_TABLE_SCHEMAS: Dict[int, Dict[str, Any]] = {
    2: {
        "name": "Normative Standards Baseline",
        "required_columns": [
            ("standard_id", ["standard_id", "standard", "id"]),
            ("issuing_body", ["issuing_body", "issuing_org", "issuing_organization", "body", "org", "organization"]),
            ("title", ["title", "title_baseline", "title_baseline_description", "standard_title"]),
            ("applicable_clauses", ["applicable_clauses", "applicable_clauses_focus_area", "clauses", "clause"]),
        ],
    },
    6: {
        "name": "SORA 4D Volume & GRB Parameters",
        "required_columns": [
            ("parameter", ["parameter", "param", "parameter_name", "name"]),
            ("symbol", ["symbol", "sym"]),
            ("value", ["value", "val"]),
            ("units", ["units", "unit"]),
            ("description", ["description", "desc"]),
        ],
    },
    7: {
        "name": "OMG UAF Activity Taxonomy",
        "required_columns": [
            ("activity_id", ["activity_id", "activity", "oa_id", "id"]),
            ("activity_name", ["activity_name", "name"]),
            ("description", ["description", "desc"]),
            ("gate_24_allocation_tag", ["gate_24_allocation_tag", "allocation_tag", "allocation", "gate24_allocation_tag"]),
        ],
    },
    8: {
        "name": "Op-Tx Information Exchange Matrix",
        "required_columns": [
            ("exchange_id", ["exchange_id", "exchange", "optx_id", "id"]),
            ("source_node", ["source_node", "source", "src_node", "src"]),
            ("destination_node", ["destination_node", "dest_node", "destination", "dest", "target_node"]),
            ("information_item", ["information_item", "information_element", "item", "info_item", "message"]),
            ("data_rate", ["data_rate", "throughput", "rate", "frequency"]),
            ("max_latency", ["max_latency", "latency", "max_latency_ms", "latency_limit"]),
            ("criticality", ["criticality", "criticality_level", "dal", "safety_criticality"]),
        ],
    },
    11: {
        "name": "O/I/D Maintenance Hierarchy",
        "required_columns": [
            ("maintenance_level", ["maintenance_level", "maintenance_tier", "tier", "level", "organizational_level"]),
            ("scope", ["scope", "maintenance_tasks_work_scope", "tasks", "work_scope", "description"]),
            ("tooling_equipment", ["tooling_equipment", "tooling", "interval_trigger", "turnaround_time", "interval", "qualification", "equipment"]),
        ],
    },
    12: {
        "name": "Emergency Decision Matrix",
        "required_columns": [
            ("trigger_id", ["trigger_id", "trigger", "emg_id", "id"]),
            ("contingency_trigger", ["contingency_trigger", "contingency_trigger_name", "trigger_event", "contingency", "event"]),
            ("detection_mechanism", ["detection_mechanism", "detection", "mechanism"]),
            ("automated_containment_action", ["automated_containment_action", "containment_action", "containment", "action"]),
            ("failsafe_state", ["failsafe_state", "primary_failsafe_state", "recovery_state", "failsafe_recovery_state"]),
            ("max_response_time", ["max_response_time", "response_time", "latency_deadline", "max_latency"]),
            ("hitl_role", ["hitl_role", "authority_role", "hitl_authority_role", "human_in_the_loop_role", "role"]),
        ],
    },
}


def _validate_table_schema(
    tables: List[List[Dict[str, str]]],
    schema_spec: Dict[str, Any],
) -> Tuple[bool, List[str], List[str]]:
    """
    Validates if at least one table in tables matches the required columns schema.
    Returns (is_valid, missing_columns, found_headers).
    """
    if not tables:
        return False, [req[0] for req in schema_spec["required_columns"]], []

    for tbl in tables:
        if not tbl:
            continue
        row_keys = list(tbl[0].keys())
        missing_for_this_table: List[str] = []
        for col_name, aliases in schema_spec["required_columns"]:
            matched = False
            for k in row_keys:
                k_norm = k.lower().replace(" ", "_").replace("-", "_")
                if any(alias == k_norm or alias in k_norm or k_norm in alias for alias in aliases):
                    matched = True
                    break
            if not matched:
                missing_for_this_table.append(col_name)

        if not missing_for_this_table:
            return True, [], row_keys

    # If none matched completely, report the missing columns from the first table
    first_tbl_keys = list(tables[0][0].keys()) if tables and tables[0] else []
    missing_from_first: List[str] = []
    for col_name, aliases in schema_spec["required_columns"]:
        matched = False
        for k in first_tbl_keys:
            k_norm = k.lower().replace(" ", "_").replace("-", "_")
            if any(alias == k_norm or alias in k_norm or k_norm in alias for alias in aliases):
                matched = True
                break
        if not matched:
            missing_from_first.append(col_name)

    return False, missing_from_first, first_tbl_keys


# =============================================================================
# Gate 26: ConopsCompletenessValidator
# =============================================================================

class ConopsCompletenessValidator(IValidator):
    """
    Quality Gate 26: ConOps Completeness Validator.
    Enforces 12 mandatory sections, SORA 4D Volume & GRB Math, UAF Activity taxonomy,
    Op-Tx exchange matrix, O/I/D maintenance, and 7-Row Emergency Decision determinism.
    """

    MANDATORY_SECTIONS: List[Dict[str, Any]] = [
        {"num": 1, "title": "Scope & System Identification", "aliases": ["scope", "system identification", "system boundary"]},
        {"num": 2, "title": "Normative Standards & Regulatory Baseline", "aliases": ["normative standards", "regulatory baseline", "applicable documents", "standards"]},
        {"num": 3, "title": "Current Situation & Deficiency Analysis", "aliases": ["current situation", "deficiency analysis", "predecessor", "predecessors", "deficiencies"]},
        {"num": 4, "title": "Operational Justification & Priority Matrix", "aliases": ["operational justification", "priority matrix", "justification", "trade-off", "trade-offs", "priorities"]},
        {"num": 5, "title": "Operational Modes & Lifecycle Stages", "aliases": ["operational modes", "lifecycle stages", "lifecycle", "modes", "stages", "phi_lifecycle"]},
        {"num": 6, "title": "4D Operational Volume & SORA Ground Risk Buffer Mathematics", "aliases": ["4d operational volume", "ground risk buffer", "sora", "operational volume", "grb math", "grb"]},
        {"num": 7, "title": "OMG UAF Operational Activity Taxonomy", "aliases": ["operational activity taxonomy", "uaf operational activities", "operational activities", "uaf activities", "oa-"]},
        {"num": 8, "title": "Operational Information Exchange (Op-Tx) Matrix", "aliases": ["operational information exchange", "op-tx matrix", "op-tx table", "information exchange", "op-tx"]},
        {"num": 9, "title": "Operational Environments & Constraints", "aliases": ["operational environments", "physical constraints", "constraints", "environmental constraints"]},
        {"num": 10, "title": "Multi-Threaded Operational Scenarios", "aliases": ["operational scenarios", "scenarios", "multi-threaded", "operational threads"]},
        {"num": 11, "title": "Maintenance & Sustainment Concepts", "aliases": ["maintenance & sustainment", "maintenance concepts", "sustainment", "o/i/d maintenance", "maintenance", "o-level"]},
        {"num": 12, "title": "7-Row Emergency Decision & Contingency Matrix", "aliases": ["emergency decision", "contingency matrix", "7-row emergency", "emergency matrix", "emergency decision & contingency matrix", "emg-"]},
    ]

    CANONICAL_EMERGENCY_TRIGGERS: List[str] = [
        "EMG-01",  # Lost C2 Link
        "EMG-02",  # GNSS Navigation Loss
        "EMG-03",  # Propulsion / Power Failure
        "EMG-04",  # Critical Sensor Fault
        "EMG-05",  # Geofence Breach / Airspace Conflict
        "EMG-06",  # Structural / Actuation Anomaly
        "EMG-07",  # Flight Termination Command
    ]

    MANDATORY_EMERGENCY_SUBSECTIONS: List[Dict[str, Any]] = [
        {
            "num": "12.1",
            "title": "Failsafe State Transition Semantics & Timing Guarantees",
            "aliases": ["12.1", "failsafe state transition", "transition semantics", "timing guarantees", "priority arbitration"],
        },
        {
            "num": "12.2",
            "title": "Deterministic Emergency Statechart & State Machine",
            "aliases": ["12.2", "emergency statechart", "emergency state machine", "deterministic emergency statechart", "state machine", "statechart"],
        },
        {
            "num": "12.3",
            "title": "Degraded Modes & Fallback Hierarchy",
            "aliases": ["12.3", "degraded modes", "fallback hierarchy", "degradation modes", "multi-tier fallback"],
        },
        {
            "num": "12.4",
            "title": "Human-in-the-Loop (HITL) Authority & Override Protocols",
            "aliases": ["12.4", "human-in-the-loop", "hitl authority", "override protocols", "hitl role", "human authority"],
        },
        {
            "num": "12.5",
            "title": "Autonomous Divert & Secondary Recovery Protocols",
            "aliases": ["12.5", "autonomous divert", "secondary recovery", "divert protocols", "return-to-base", "rtb"],
        },
        {
            "num": "12.6",
            "title": "Post-Emergency Containment, Latching & Reset Procedures",
            "aliases": ["12.6", "post-emergency containment", "latching & reset", "reset procedures", "ground reset", "containment & reset"],
        },
    ]

    def __init__(self, strict_sora_math: bool = True) -> None:
        self.strict_sora_math = strict_sora_math

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        findings: List[Finding] = []
        workspace_dir = repo.workspace_dir
        conops_dir = os.path.join(workspace_dir, "docs", "conops")

        if not os.path.isdir(conops_dir):
            if repo.is_upstream_compiler_repo():
                return []
            return [Finding(
                "conops-corpus-missing",
                "ConOps corpus directory 'docs/conops' is missing in workspace.",
                location="docs/conops",
            )]

        # Find CONOPS file(s)
        conops_files: List[str] = []
        for root, _, files in os.walk(conops_dir):
            for f in sorted(files):
                if f.upper().startswith("CONOPS") and f.endswith(".md") and "TEMPLATE" not in f.upper():
                    conops_files.append(os.path.join(root, f))

        if not conops_files:
            if repo.is_upstream_compiler_repo():
                return []
            return [Finding(
                "conops-corpus-missing",
                "No valid ConOps specification document ('CONOPS*.md') found in 'docs/conops'.",
                location="docs/conops",
            )]

        for c_file in conops_files:
            rel_path = os.path.relpath(c_file, workspace_dir)
            try:
                with open(c_file, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception as e:
                findings.append(Finding(
                    "conops-read-error",
                    f"Failed to read ConOps document '{rel_path}': {e}",
                    location=rel_path,
                ))
                continue

            findings.extend(self._validate_conops_text(content, rel_path, repo=repo))

        return findings

    def _validate_conops_text(self, content: str, rel_path: str, repo: Optional[WorkspaceRepository] = None) -> List[Finding]:
        findings: List[Finding] = []
        sections_list = _extract_markdown_sections_list(content)
        sections = _extract_markdown_sections(content)
        tables, malformed_lines = _parse_commonmark_tables(content)

        # Check for allocated obligations from RESEARCH_INVENTORY.md (Gate 26, Issue #109)
        if "TEMPLATE" not in rel_path.upper():
            inventory_file = None
            if repo is not None:
                inventory_file = os.path.join(repo.workspace_dir, "docs", "research", "RESEARCH_INVENTORY.md")
            else:
                candidate = os.path.join("docs", "research", "RESEARCH_INVENTORY.md")
                if os.path.isfile(candidate):
                    inventory_file = candidate

            if inventory_file and os.path.isfile(inventory_file):
                try:
                    with open(inventory_file, "r", encoding="utf-8") as inv_f:
                        inv_doc = parse_research_inventory(inv_f.read())

                    # Extract document's realized/witnessed tags
                    doc_tags = set(_parse_witness_tags(content) + _parse_obligation_tags(content))
                    fm = extract_metadata_from_content(content)
                    if fm:
                        for key in ("obligations", "normative_obligations", "allocated_obligations", "obligation_id"):
                            val = fm.get(key)
                            if isinstance(val, list):
                                for item in val:
                                    norm = _normalize_obligation_id(str(item))
                                    if norm:
                                        doc_tags.add(norm)
                            elif isinstance(val, str):
                                norm = _normalize_obligation_id(val)
                                if norm:
                                    doc_tags.add(norm)

                    # Find all obligations allocated to CONOPS in clause allocations
                    for alloc in inv_doc.clause_allocations:
                        if alloc.population_id and alloc.downstream_spec_file:
                            spec_target = alloc.downstream_spec_file.strip("`* ")
                            target_base = os.path.basename(spec_target).upper()
                            is_conops_target = (
                                target_base.startswith("CONOPS")
                                or os.path.normpath(spec_target) == os.path.normpath(rel_path)
                            )
                            if is_conops_target:
                                norm_id = _normalize_obligation_id(alloc.population_id)
                                if norm_id not in doc_tags:
                                    findings.append(Finding(
                                        "conops-obligation-unwitnessed",
                                        f"ConOps document '{rel_path}' is missing mandatory witness/realization tag for allocated obligation '{norm_id}' ({alloc.clause_citation or alloc.standard_id}).",
                                        location=rel_path,
                                        detail={"obligation_id": norm_id, "file": rel_path, "standard_id": alloc.standard_id},
                                    ))
                except Exception:
                    pass

        # Check for un-substituted template placeholders (Gate 26, Fixes #142)
        if "TEMPLATE" not in rel_path.upper():
            unresolved = _find_unresolved_template_placeholders(content)
            if unresolved:
                unresolved_tags = [tag for _, tag in unresolved]
                unique_tags = sorted(list(set(unresolved_tags)))
                line_numbers = sorted(list(set(line for line, _ in unresolved)))
                first_line = unresolved[0][0]
                findings.append(Finding(
                    "conops-unresolved-template-placeholders",
                    f"ConOps specification '{rel_path}' contains {len(unresolved)} unresolved template placeholder token(s): {', '.join(unique_tags)} at line(s) {', '.join(str(l) for l in line_numbers)}.",
                    location=f"{rel_path}:{first_line}",
                    detail={
                        "severity": "CRITICAL",
                        "unresolved_tags": unique_tags,
                        "line_numbers": line_numbers,
                        "placeholders": [f"{tag} (line {line})" for line, tag in unresolved],
                        "file": rel_path,
                    },
                ))

        # Check density, tables, and Mermaid structures for non-templates (Gate 26, Fixes #130)
        if "TEMPLATE" not in rel_path.upper():
            findings.extend(self._validate_conops_density_and_structures(content, rel_path, tables=tables))

        # Check for malformed tables
        for m_line in malformed_lines:
            findings.append(Finding(
                "conops-table-malformed",
                f"Malformed CommonMark table or broken row formatting detected at line {m_line} in '{rel_path}'.",
                location=f"{rel_path}:{m_line}",
            ))

        # Extract content sections (excluding TOC and top-level H1 title)
        content_sections: List[Tuple[str, int, str]] = []
        raw_lines = content.splitlines()
        for h, l_num, c in sections_list:
            if re.match(r'^(?:table\s+of\s+contents|toc)$', h.strip(), re.IGNORECASE):
                continue
            if 1 <= l_num <= len(raw_lines):
                line_str = raw_lines[l_num - 1].strip()
                if line_str.startswith("# ") and not line_str.startswith("## "):
                    continue
            content_sections.append((h, l_num, c))

        # Check Table of Contents Completeness for non-templates (Fixes #148)
        if "TEMPLATE" not in rel_path.upper():
            toc_matches = [
                s for s in sections_list
                if re.match(r'^(?:table\s+of\s+contents|toc)$', s[0].strip(), re.IGNORECASE)
            ]
            if not toc_matches:
                findings.append(Finding(
                    "conops-toc-missing",
                    f"ConOps specification '{rel_path}' is missing mandatory Table of Contents ('## Table of Contents').",
                    location=rel_path,
                    detail={"severity": "CRITICAL", "file": rel_path},
                ))
            else:
                toc_heading, toc_line, toc_chunk = toc_matches[0]
                missing_toc_sections: List[Tuple[int, str]] = []
                for req in self.MANDATORY_SECTIONS:
                    sec_num = req["num"]
                    sec_title = req["title"]
                    sec_aliases = req["aliases"]

                    num_pattern_text = rf'\[(?:section\s+)?{sec_num}[.\s:\-—]'
                    num_pattern_anchor = rf'\(#(?:section-)?{sec_num}[-_]'
                    found_in_toc = bool(
                        re.search(num_pattern_text, toc_chunk, re.IGNORECASE)
                        or re.search(num_pattern_anchor, toc_chunk, re.IGNORECASE)
                    )
                    if not found_in_toc:
                        for alias in sec_aliases:
                            if re.search(rf'\[[^\]]*{re.escape(alias)}[^\]]*\]', toc_chunk, re.IGNORECASE):
                                found_in_toc = True
                                break

                    if not found_in_toc:
                        missing_toc_sections.append((sec_num, sec_title))

                if missing_toc_sections:
                    missing_str = ", ".join(f"Section {n} ('{t}')" for n, t in missing_toc_sections)
                    findings.append(Finding(
                        "conops-toc-incomplete",
                        f"Table of Contents in '{rel_path}' is incomplete; missing {len(missing_toc_sections)} mandatory section(s): {missing_str}.",
                        location=f"{rel_path}:{toc_line}",
                        detail={
                            "severity": "CRITICAL",
                            "missing_sections": [n for n, _ in missing_toc_sections],
                            "file": rel_path,
                        },
                    ))

        # Check for Duplicate Section Headers (Fixes #148)
        reported_duplicate_lines: Set[int] = set()
        for req in self.MANDATORY_SECTIONS:
            sec_num = req["num"]
            sec_title = req["title"]
            sec_aliases = req["aliases"]
            matches = _find_matching_sections_all(content_sections, sec_num, sec_title, sec_aliases)
            if len(matches) > 1:
                for dup_heading, dup_line, _ in matches[1:]:
                    reported_duplicate_lines.add(dup_line)
                    findings.append(Finding(
                        "conops-duplicate-section-header",
                        f"Duplicate section header detected for Section {sec_num} ('{dup_heading}') at line {dup_line} in '{rel_path}'.",
                        location=f"{rel_path}:{dup_line}",
                        detail={
                            "severity": "CRITICAL",
                            "section_number": sec_num,
                            "heading": dup_heading,
                            "line": dup_line,
                            "file": rel_path,
                        },
                    ))

        seen_h2_headers: Dict[str, Tuple[int, str]] = {}
        for h, l_num, _ in content_sections:
            h_norm = re.sub(r'[*`_]', '', h).strip().lower()
            if h_norm in seen_h2_headers:
                if l_num not in reported_duplicate_lines:
                    reported_duplicate_lines.add(l_num)
                    findings.append(Finding(
                        "conops-duplicate-section-header",
                        f"Duplicate section header detected ('{h}') at line {l_num} in '{rel_path}'.",
                        location=f"{rel_path}:{l_num}",
                        detail={
                            "severity": "CRITICAL",
                            "heading": h,
                            "line": l_num,
                            "file": rel_path,
                        },
                    ))
            else:
                seen_h2_headers[h_norm] = (l_num, h)

        # Check Exact Section Cardinality (Fixes #148)
        if len(content_sections) != 12:
            findings.append(Finding(
                "conops-section-cardinality-mismatch",
                f"ConOps specification '{rel_path}' has section cardinality mismatch (found {len(content_sections)} section(s); expected exactly 12 sections).",
                location=rel_path,
                detail={
                    "severity": "CRITICAL",
                    "expected_sections": 12,
                    "actual_sections": len(content_sections),
                    "file": rel_path,
                },
            ))

        # Check for 12 Mandatory Sections
        matched_sections: Dict[int, Tuple[str, int, str]] = {}
        for req in self.MANDATORY_SECTIONS:
            sec_num = req["num"]
            title = req["title"]
            aliases = req["aliases"]
            res = _find_matching_section(content_sections, sec_num, title, aliases)
            if res:
                matched_sections[sec_num] = res
            else:
                findings.append(Finding(
                    "conops-section-missing",
                    f"Mandatory ConOps Section {sec_num} ('{title}') is missing or empty in '{rel_path}'.",
                    location=rel_path,
                    detail={"section_number": sec_num, "section_title": title},
                ))

        # Check for Hollow Sections failing line floor (Fixes #114, #130)
        if "TEMPLATE" not in rel_path.upper():
            for sec_num, (h_title, s_line, s_chunk) in matched_sections.items():
                non_empty = [l for l in s_chunk.splitlines() if l.strip()]
                if len(non_empty) < 8:
                    findings.append(Finding(
                        "conops-section-hollow",
                        f"Section {sec_num} ('{h_title}') in '{rel_path}' is hollow / below minimum line floor ({len(non_empty)} lines; minimum required: 8 lines).",
                        location=f"{rel_path}:{s_line}",
                        detail={"section_number": sec_num, "line_count": len(non_empty), "minimum_floor": 8, "file": rel_path},
                    ))

        # Check Mandatory Table Schemas (Fixes #114, #130)
        if "TEMPLATE" not in rel_path.upper():
            findings.extend(self._validate_conops_table_schemas(content, rel_path, matched_sections))

        # Section 4: Operational Justification & Pugh Decision Matrix with S_j(w) Validation (Fixes #130)
        if 4 in matched_sections and "TEMPLATE" not in rel_path.upper():
            _, sec4_line, sec4_content = matched_sections[4]
            findings.extend(self._validate_pugh_decision_matrix(content, rel_path, sec4_content, sec4_line))

        # Section 6: SORA 4D Volume & GRB Math Validation
        if 6 in matched_sections:
            _, sec6_line, sec6_content = matched_sections[6]
            h_max_val, v_wind_val, theta_val, r_grb_val = _extract_sora_parameters(sec6_content)

            if h_max_val is not None and v_wind_val is not None and r_grb_val is not None and self.strict_sora_math:
                r_calc = calculate_sora_grb_radius(h_max_m=h_max_val, theta_impact_deg=theta_val, v_wind_max_mps=v_wind_val)
                if r_grb_val < (r_calc - 1.0):
                    findings.append(Finding(
                        "conops-sora-grb-underdimensioned",
                        f"Ground Risk Buffer radius R_GRB = {r_grb_val:.1f} m in Section 6 is under-dimensioned against JARUS SORA v2.5 theoretical minimum {r_calc:.1f} m (for h_max={h_max_val} m, v_wind={v_wind_val} m/s).",
                        location=f"{rel_path}:{sec6_line}",
                        detail={"declared_r_grb": r_grb_val, "minimum_r_grb": r_calc},
                    ))

        # Section 10: Multi-Threaded Operational Scenarios Timeline Steps (Fixes #114, #130)
        if 10 in matched_sections and "TEMPLATE" not in rel_path.upper():
            _, sec10_line, sec10_content = matched_sections[10]
            findings.extend(self._validate_scenario_timeline_steps(content, rel_path, sec10_content, sec10_line))

        # Section 12: 7-Row Emergency Decision Matrix Validation
        if 12 in matched_sections:
            _, sec12_line, sec12_content = matched_sections[12]
            sec12_tables, _ = _parse_commonmark_tables(sec12_content)
            
            emergency_rows: List[Dict[str, str]] = []
            if sec12_tables:
                emergency_rows = sec12_tables[0]

            found_triggers: Set[str] = set()
            if sec12_tables:
                for r in sec12_tables[0]:
                    # Find trigger token like EMG-01
                    full_row_str = " ".join(r.values())
                    m_emg = re.search(r'(EMG-0*[1-7])', full_row_str, re.IGNORECASE)
                    if m_emg:
                        found_triggers.add(m_emg.group(1).upper().replace("-0", "-0" if len(m_emg.group(1).split("-")[1]) == 2 else "-0"))
                    for trig in self.CANONICAL_EMERGENCY_TRIGGERS:
                        if trig in full_row_str.upper():
                            found_triggers.add(trig)
            else:
                # Fallback regex over raw section text if table missing
                for trig in self.CANONICAL_EMERGENCY_TRIGGERS:
                    if re.search(rf'\b{trig}\b', sec12_content, re.IGNORECASE):
                        found_triggers.add(trig)

            if len(found_triggers) < 7:
                missing_triggers = [t for t in self.CANONICAL_EMERGENCY_TRIGGERS if t not in found_triggers]
                findings.append(Finding(
                    "conops-emergency-matrix-incomplete",
                    f"Section 12 Emergency Decision Matrix has {len(found_triggers)}/7 canonical emergency triggers; missing: {', '.join(missing_triggers)}.",
                    location=f"{rel_path}:{sec12_line}",
                    detail={"found_triggers": list(found_triggers), "missing_triggers": missing_triggers},
                ))

            # Validate Section 12 depth (subsections 12.1..12.6 + statechart)
            findings.extend(self._validate_emergency_matrix_depth(content, rel_path, sec12_content, sec12_line))

        # Check Mermaid Diagram Integrity (Fixes #114, #130)
        findings.extend(_validate_mermaid_integrity(content, rel_path, prefix="conops"))

        return findings

    def _validate_conops_table_schemas(
        self,
        content: str,
        rel_path: str,
        matched_sections: Dict[int, Tuple[str, int, str]],
    ) -> List[Finding]:
        """
        Validates mandatory table column schemas across ConOps sections (Fixes #114, #130).
        """
        findings: List[Finding] = []
        for sec_num, schema_spec in MANDATORY_CONOPS_TABLE_SCHEMAS.items():
            if sec_num in matched_sections:
                _, sec_line, sec_content = matched_sections[sec_num]
                sec_tables, _ = _parse_commonmark_tables(sec_content)
                is_valid, missing_cols, found_keys = _validate_table_schema(sec_tables, schema_spec)
                if not is_valid:
                    findings.append(Finding(
                        "conops-table-schema-invalid",
                        f"Table in Section {sec_num} ('{schema_spec['name']}') in '{rel_path}' is missing required column schema: {', '.join(missing_cols)}.",
                        location=f"{rel_path}:{sec_line}",
                        detail={
                            "section": sec_num,
                            "table_name": schema_spec["name"],
                            "missing_columns": missing_cols,
                            "found_columns": found_keys,
                            "file": rel_path,
                        },
                    ))
        return findings

    def _validate_scenario_timeline_steps(
        self,
        content: str,
        rel_path: str,
        sec10_content: str,
        sec10_line: int,
    ) -> List[Finding]:
        """
        Validates timeline step count in Section 10 (Multi-Threaded Operational Scenarios) (Fixes #114, #130):
        - SCN-01 / Scenario 1 >= 8 steps (nominal lifecycle thread)
        - SCN-02 / Scenario 2 >= 6 steps
        - SCN-03 / Scenario 3 >= 6 steps
        """
        findings: List[Finding] = []
        
        # Split Section 10 into scenario sub-blocks by headers
        scenario_blocks = re.split(r'(?=^#{2,4}\s+)', sec10_content, flags=re.MULTILINE)

        def _count_steps(block_text: str) -> int:
            # 1. Check table rows
            tbls, _ = _parse_commonmark_tables(block_text)
            table_steps = 0
            for tbl in tbls:
                step_rows = 0
                for row in tbl:
                    first_val = list(row.values())[0] if row else ""
                    step_val = row.get("step_number") or row.get("step") or row.get("step_num") or first_val
                    clean_step = re.sub(r'[*`_]', '', str(step_val)).strip()
                    if re.match(r'^[0-9]+$', clean_step) or re.match(r'^step\s*[0-9]+', clean_step, re.IGNORECASE):
                        step_rows += 1
                table_steps = max(table_steps, step_rows)

            # 2. Check bullet points / lines with Step X
            step_matches = re.findall(
                r'[-*]\s+\*?\*?Step\s*([0-9]+)',
                block_text,
                re.IGNORECASE,
            )
            return max(table_steps, len(step_matches))

        checked_scenarios = {
            "SCN-01": (8, r'\b(?:SCN-01|Scenario\s+1\b|Nominal)', "Scenario 1 (SCN-01)"),
            "SCN-02": (6, r'\b(?:SCN-02|Scenario\s+2\b)', "Scenario 2 (SCN-02)"),
            "SCN-03": (6, r'\b(?:SCN-03|Scenario\s+3\b)', "Scenario 3 (SCN-03)"),
        }

        for scn_key, (min_steps, scn_pattern, scn_title) in checked_scenarios.items():
            for block in scenario_blocks:
                header_m = re.match(r'^#{2,4}\s+(.+)$', block.strip(), flags=re.MULTILINE)
                block_header = header_m.group(1) if header_m else block[:100]
                if re.search(scn_pattern, block_header, re.IGNORECASE):
                    cnt = _count_steps(block)
                    if cnt < min_steps:
                        findings.append(Finding(
                            "conops-scenario-steps-truncated",
                            f"{scn_title} in '{rel_path}' has {cnt} lifecycle step(s); minimum required: {min_steps} steps.",
                            location=f"{rel_path}:{sec10_line}",
                            detail={"scenario": scn_key, "step_count": cnt, "min_required": min_steps, "file": rel_path},
                        ))
                    break

        return findings

    def _validate_conops_density_and_structures(
        self,
        content: str,
        rel_path: str,
        tables: Optional[List[List[Dict[str, str]]]] = None,
    ) -> List[Finding]:
        """
        Validates structural density, markdown tables, and Mermaid diagrams in ConOps (Fixes #130):
        1. Minimum line count floor: >= 800 lines (emits 'conops-density-insufficient' if violated).
        2. Formal markdown tables: >= 8 tables (emits 'conops-tables-insufficient' if violated).
        3. Mermaid diagrams: >= 3 diagrams, verifying presence of 'flowchart TB',
           'sequenceDiagram', and 'stateDiagram-v2' (emits 'conops-mermaid-diagrams-insufficient' if violated).
        """
        findings: List[Finding] = []

        # 1. Line count floor (>= 800 lines)
        line_count = len(content.splitlines())
        if line_count < 800:
            findings.append(Finding(
                "conops-density-insufficient",
                f"ConOps specification '{rel_path}' has insufficient line density ({line_count} lines; minimum required: 800 lines).",
                location=rel_path,
                detail={"line_count": line_count, "min_required_lines": 800, "file": rel_path},
            ))

        # 2. Formal markdown tables (>= 8 tables)
        if tables is None:
            tables, _ = _parse_commonmark_tables(content)
        table_count = len(tables)
        if table_count < 8:
            findings.append(Finding(
                "conops-tables-insufficient",
                f"ConOps specification '{rel_path}' contains {table_count} markdown table(s); minimum required: 8 formal tables.",
                location=rel_path,
                detail={"table_count": table_count, "min_required_tables": 8, "file": rel_path},
            ))

        # 3. Mermaid diagrams (>= 3 diagrams, verifying flowchart TB, sequenceDiagram, stateDiagram-v2)
        mermaid_blocks = re.findall(r'```(?:mermaid)?\s*\n([\s\S]*?)```', content, re.IGNORECASE)
        mermaid_matches = [
            b for b in mermaid_blocks
            if re.search(r'\b(?:flowchart|graph|sequenceDiagram|stateDiagram(?:-v2)?|classDiagram|erDiagram|gantt|pie|journey|gitGraph|quadrantChart|mindmap|timeline|zenuml|C4Context)\b', b, re.IGNORECASE)
        ]
        raw_mermaid_fences = len(re.findall(r'```mermaid\b', content, re.IGNORECASE))
        total_diagrams = max(len(mermaid_matches), raw_mermaid_fences)

        has_flowchart_tb = bool(
            re.search(r'flowchart\s+TB\b', content, re.IGNORECASE)
            or re.search(r'graph\s+TB\b', content, re.IGNORECASE)
        )
        has_sequence_diagram = bool(re.search(r'sequenceDiagram\b', content, re.IGNORECASE))
        has_state_diagram = bool(
            re.search(r'stateDiagram-v2\b', content, re.IGNORECASE)
            or re.search(r'stateDiagram\b', content, re.IGNORECASE)
        )

        missing_diagrams: List[str] = []
        if not has_flowchart_tb:
            missing_diagrams.append("flowchart TB")
        if not has_sequence_diagram:
            missing_diagrams.append("sequenceDiagram")
        if not has_state_diagram:
            missing_diagrams.append("stateDiagram-v2")

        if total_diagrams < 3 or missing_diagrams:
            findings.append(Finding(
                "conops-mermaid-diagrams-insufficient",
                f"ConOps specification '{rel_path}' has insufficient Mermaid diagrams ({total_diagrams}/3 minimum required; missing: {', '.join(missing_diagrams) if missing_diagrams else 'total count < 3'}).",
                location=rel_path,
                detail={"diagram_count": total_diagrams, "min_required_diagrams": 3, "missing_types": missing_diagrams, "file": rel_path},
            ))

        return findings

    def _validate_pugh_decision_matrix(
        self,
        content: str,
        rel_path: str,
        sec4_content: str,
        sec4_line: int,
    ) -> List[Finding]:
        """
        Validates Section 4 Operational Justification & Priority Matrix (Fixes #130):
        1. Mandatory Pugh decision matrix table evaluating candidate architectures against criteria and weights.
        2. Mandatory LaTeX sensitivity equation S_j(w) in display math block.
        """
        findings: List[Finding] = []

        # Check for Pugh decision matrix
        has_pugh_keyword = bool(
            re.search(r'\bpugh\b', sec4_content, re.IGNORECASE)
            or re.search(r'\bpugh\b', content, re.IGNORECASE)
        )
        sec4_tables, _ = _parse_commonmark_tables(sec4_content)
        has_decision_table = False
        for tbl in sec4_tables:
            for row in tbl:
                keys_and_vals = " ".join(list(row.keys()) + list(row.values())).lower()
                if (
                    "weight" in keys_and_vals
                    or "score" in keys_and_vals
                    or "criterion" in keys_and_vals
                    or "criteria" in keys_and_vals
                    or "datum" in keys_and_vals
                    or "baseline" in keys_and_vals
                ):
                    has_decision_table = True
                    break
            if has_decision_table:
                break

        if not (has_pugh_keyword and (has_decision_table or sec4_tables)):
            findings.append(Finding(
                "conops-pugh-matrix-missing",
                f"Section 4 Operational Justification & Priority Matrix is missing mandatory Pugh decision matrix in '{rel_path}'.",
                location=f"{rel_path}:{sec4_line}",
                detail={"file": rel_path, "section": 4},
            ))

        # Check for LaTeX sensitivity equation S_j(w)
        has_sensitivity_formula = bool(
            re.search(r'S[_\s]*\{?j\}?\s*(?:\([^\)]+\)|\[[^\]]+\])', sec4_content)
            or re.search(r'S[_\s]*\{?j\}?\s*(?:\([^\)]+\)|\[[^\]]+\])', content)
        )
        has_math_block = bool(
            re.search(r'\$\$[\s\S]*?S[_\s]*\{?j\}?[\s\S]*?\$\$', sec4_content)
            or re.search(r'\$\$[\s\S]*?S[_\s]*\{?j\}?[\s\S]*?\$\$', content)
        )

        if not (has_sensitivity_formula and has_math_block):
            findings.append(Finding(
                "conops-pugh-sensitivity-missing",
                f"Section 4 Operational Justification & Priority Matrix is missing mandatory LaTeX sensitivity equation S_j(w) for Pugh decision analysis in '{rel_path}'.",
                location=f"{rel_path}:{sec4_line}",
                detail={"file": rel_path, "section": 4},
            ))

        return findings

    def _validate_emergency_matrix_depth(
        self,
        content: str,
        rel_path: str,
        sec12_content: str,
        sec12_line: int,
    ) -> List[Finding]:
        """
        Validates depth of Section 12 (7-Row Emergency Decision & Contingency Matrix):
        1. Mandatory subsections 12.1 through 12.6 presence.
        2. Mandatory Mermaid statechart diagram in Section 12.
        """
        findings: List[Finding] = []
        found_subsections: Set[str] = set()
        missing_subsections: List[str] = []

        heading_matches = re.findall(r'^(#{2,4})\s+(.+)$', sec12_content, re.MULTILINE)
        headings_in_sec12 = [h[1].strip() for h in heading_matches]

        for req in self.MANDATORY_EMERGENCY_SUBSECTIONS:
            s_num = req["num"]
            s_title = req["title"]
            s_aliases = req["aliases"]
            matched = False

            # Direct regex match for subsection header (e.g. ### 12.1)
            num_pattern = rf'###\s+{re.escape(s_num)}\b'
            if re.search(num_pattern, sec12_content, re.IGNORECASE):
                matched = True
            else:
                for h in headings_in_sec12:
                    h_lower = h.lower()
                    if s_num in h_lower:
                        matched = True
                        break
                    for alias in s_aliases:
                        if alias.lower() in h_lower:
                            matched = True
                            break
                    if matched:
                        break

            if matched:
                found_subsections.add(s_num)
            else:
                missing_subsections.append(f"{s_num} ({s_title})")

        if missing_subsections:
            findings.append(Finding(
                "conops-emergency-depth-missing",
                f"Section 12 Emergency Decision Matrix is missing required depth subsection(s): {', '.join(missing_subsections)} in '{rel_path}'.",
                location=f"{rel_path}:{sec12_line}",
                detail={"missing_subsections": missing_subsections, "found_subsections": list(found_subsections)},
            ))

        # Check for Mermaid Statechart Diagram in Section 12
        has_statechart = bool(
            re.search(r'```(?:mermaid)?\s*\n\s*(?:stateDiagram|stateDiagram-v2)\b', sec12_content, re.IGNORECASE)
            or (re.search(r'```mermaid', sec12_content, re.IGNORECASE) and re.search(r'-->', sec12_content))
        )
        if not has_statechart:
            findings.append(Finding(
                "conops-emergency-statechart-missing",
                f"Section 12 Emergency Decision Matrix (Subsection 12.2) is missing mandatory Mermaid statechart diagram (stateDiagram-v2) in '{rel_path}'.",
                location=f"{rel_path}:{sec12_line}",
            ))

        return findings

    def synthesize_canonical_template(self, output_path: Union[str, Path]) -> bool:
        """Synthesizes domain-neutral CONOPS_CANONICAL_TEMPLATE.md."""
        res_path = Path(__file__).resolve().parents[4] / "resources" / "CONOPS_CANONICAL_TEMPLATE.md"
        if res_path.is_file():
            template_text = res_path.read_text(encoding="utf-8")
        else:
            template_text = r"""| Attribute | Value |
| :--- | :--- |
| **Title** | Concept of Operations (ConOps): {{SYSTEM_IDENTIFIER}} |
| **Version** | {{DOCUMENT_VERSION}} |
| **Date** | {{DOCUMENT_DATE}} |

# Concept of Operations (ConOps): {{SYSTEM_IDENTIFIER}}

## 1. Scope & System Identification
- **System Identifier:** `{{SYSTEM_IDENTIFIER}}`
- **Operational Domain:** `{{OPERATIONAL_DOMAIN}}`
- **Operational Boundaries:** {{OPERATIONAL_BOUNDARIES}}
- **Stakeholder Roster:** {{STAKEHOLDER_ROSTER}}

## 2. Normative Standards & Regulatory Baseline
| Standard ID | Issuing Body | Title / Baseline | Applicable Clauses |
| :--- | :--- | :--- | :--- |
| ISO/IEC/IEEE 29148:2018 | ISO/IEEE | Systems and Software Engineering — Requirements Engineering | §6.4.2 ConOps & §6.4.3 OpsCon |
| OMG UAF v1.2 / v2.0 | OMG | Unified Architecture Framework | Operational Domain (Op-*) |
| NATO STANAG 4586 | NATO | Standard Interfaces of Autonomous Control Systems | Interoperability Profiles |
| JARUS SORA v2.5 | JARUS | Specific Operations Risk Assessment | Annex B (Ground Risk & GRB) |
| RTCA DO-178C / DO-254 | RTCA | Software and Electronic Hardware Considerations | Safety Assurance |

## 3. Current Situation & Deficiency Analysis (Predecessors)
- **Current Operational Baseline:** {{CURRENT_OPERATIONAL_BASELINE}}
- **Operational Deficiencies:** {{OPERATIONAL_DEFICIENCIES}}

## 4. Operational Justification & Priority Matrix (Trade-Offs)
- **Mission Drivers & Value Proposition:** {{MISSION_DRIVERS_AND_VALUE_PROPOSITION}}
- **Trade-Off Analysis:** {{TRADE_OFF_ANALYSIS}}

## 5. Operational Modes & Lifecycle Stages
Formal operational lifecycle stages across $\Phi_{\mathrm{lifecycle}}$:
- **Phase_Startup:** {{PHASE_STARTUP_DESCRIPTION}}
- **Phase_NominalExecution:** {{PHASE_NOMINAL_EXECUTION_DESCRIPTION}}
- **Phase_DegradedMode:** {{PHASE_DEGRADED_MODE_DESCRIPTION}}
- **Phase_ContingencyFailsafe:** {{PHASE_CONTINGENCY_FAILSAFE_DESCRIPTION}}
- **Phase_SecureShutdown:** {{PHASE_SECURE_SHUTDOWN_DESCRIPTION}}
- **Phase_MaintenanceMode:** {{PHASE_MAINTENANCE_MODE_DESCRIPTION}}

## 6. 4D Operational Volume & SORA Ground Risk Buffer Mathematics
$$
\begin{aligned}
V_{\mathrm{4D}} &= V_{\mathrm{SpatialGeometry}} \cup V_{\mathrm{ContingencyVolume}} \cup V_{\mathrm{GRB}} \\
R_{\mathrm{GRB}} &= h_{\mathrm{max}} \cdot \tan(\theta_{\mathrm{impact}}) + v_{\mathrm{wind,max}} \cdot \sqrt{\frac{2 h_{\mathrm{max}}}{g}} + d_{\mathrm{glide,max}}
\end{aligned}
$$

| Parameter | Symbol | Value | Units | Description |
| :--- | :--- | :--- | :--- | :--- |
| Max Altitude / Ceiling | h_max | {{H_MAX_M}} | m | Maximum operating ceiling above reference surface |
| Impact Angle | theta_impact | {{THETA_IMPACT_DEG}} | deg | Worst-case operational trajectory impact angle |
| Max Wind Speed | v_wind_max | {{V_WIND_MAX_MPS}} | m/s | Maximum operational wind speed limit |
| Gravitational Accel | g | {{G_ACCEL_MPS2}} | m/s^2 | Standard gravitational acceleration constant |
| Maximum Glide Distance | d_glide_max | {{D_GLIDE_MAX_M}} | m | Maximum unpowered lateral displacement margin |
| Ground Risk Buffer Radius | R_GRB | {{R_GRB_METERS}} | m | Declared ground risk buffer containment radius |
| Terminal Velocity | v_terminal | {{V_TERMINAL_MPS}} | m/s | Estimated unpowered descent terminal velocity |
| Impact Kinetic Energy | E_impact | {{E_IMPACT_JOULES}} | J | Kinetic energy at operational boundary impact |

## 7. OMG UAF Operational Activity Taxonomy
| Activity ID | Activity Name | Description | Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- |
| OA-01 | {{OA_ACTIVITY_NAME}} | {{OA_DESCRIPTION}} | `/// OperationalAllocation: [OA-01]` |

## 8. Operational Information Exchange (Op-Tx) Matrix
| Exchange ID | Source Node | Destination Node | Information Item | Data Rate | Max Latency | Criticality |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| OpTx-01 | {{OPTX_SOURCE_NODE}} | {{OPTX_DEST_NODE}} | {{OPTX_INFO_ITEM}} | {{OPTX_DATA_RATE}} | {{OPTX_MAX_LATENCY}} | {{OPTX_CRITICALITY}} |

## 9. Operational Environments & Constraints
- **Ambient Temperature:** {{AMBIENT_TEMPERATURE_RANGE}}
- **Environmental Ingress:** {{ENVIRONMENTAL_INGRESS_RATING}}
- **Electromagnetic / RF Environment:** {{RF_ENVIRONMENT_CONSTRAINTS}}
- **Physical Spatial Constraints:** {{PHYSICAL_SPATIAL_CONSTRAINTS}}

## 10. Multi-Threaded Operational Scenarios
- **Scenario 1 (Nominal Execution):** {{SCENARIO_NOMINAL_THREAD}}
- **Scenario 2 (Degraded Mode & Mitigation):** {{SCENARIO_DEGRADED_THREAD}}
- **Scenario 3 (Contingency Recovery):** {{SCENARIO_CONTINGENCY_THREAD}}

## 11. Maintenance & Sustainment Concepts (O/I/D Maintenance)
- **O-Level (Organizational):** {{O_LEVEL_MAINTENANCE_DESCRIPTION}}
- **I-Level (Intermediate):** {{I_LEVEL_MAINTENANCE_DESCRIPTION}}
- **D-Level (Depot):** {{D_LEVEL_MAINTENANCE_DESCRIPTION}}

## 12. 7-Row Emergency Decision & Contingency Matrix
| Trigger ID | Contingency Trigger | Detection Mechanism | Automated Containment Action | Failsafe State | Max Response Time | HITL Role |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `EMG-01` | {{EMG_TRIGGER_NAME}} | {{EMG_DETECTION_MECHANISM}} | {{EMG_CONTAINMENT_ACTION}} | `{{EMG_FAILSAFE_STATE}}` | {{EMG_MAX_RESPONSE_TIME}} | {{EMG_HITL_ROLE}} |
| `EMG-02` | {{EMG_TRIGGER_NAME}} | {{EMG_DETECTION_MECHANISM}} | {{EMG_CONTAINMENT_ACTION}} | `{{EMG_FAILSAFE_STATE}}` | {{EMG_MAX_RESPONSE_TIME}} | {{EMG_HITL_ROLE}} |
| `EMG-03` | {{EMG_TRIGGER_NAME}} | {{EMG_DETECTION_MECHANISM}} | {{EMG_CONTAINMENT_ACTION}} | `{{EMG_FAILSAFE_STATE}}` | {{EMG_MAX_RESPONSE_TIME}} | {{EMG_HITL_ROLE}} |
| `EMG-04` | {{EMG_TRIGGER_NAME}} | {{EMG_DETECTION_MECHANISM}} | {{EMG_CONTAINMENT_ACTION}} | `{{EMG_FAILSAFE_STATE}}` | {{EMG_MAX_RESPONSE_TIME}} | {{EMG_HITL_ROLE}} |
| `EMG-05` | {{EMG_TRIGGER_NAME}} | {{EMG_DETECTION_MECHANISM}} | {{EMG_CONTAINMENT_ACTION}} | `{{EMG_FAILSAFE_STATE}}` | {{EMG_MAX_RESPONSE_TIME}} | {{EMG_HITL_ROLE}} |
| `EMG-06` | {{EMG_TRIGGER_NAME}} | {{EMG_DETECTION_MECHANISM}} | {{EMG_CONTAINMENT_ACTION}} | `{{EMG_FAILSAFE_STATE}}` | {{EMG_MAX_RESPONSE_TIME}} | {{EMG_HITL_ROLE}} |
| `EMG-07` | {{EMG_TRIGGER_NAME}} | {{EMG_DETECTION_MECHANISM}} | {{EMG_CONTAINMENT_ACTION}} | `{{EMG_FAILSAFE_STATE}}` | {{EMG_MAX_RESPONSE_TIME}} | {{EMG_HITL_ROLE}} |

### 12.1 Failsafe State Transition Semantics & Timing Guarantees
$$
\begin{aligned}
P_{\mathrm{EMG-07}} > P_{\mathrm{EMG-03}} > P_{\mathrm{EMG-05}} > P_{\mathrm{EMG-06}} > P_{\mathrm{EMG-04}} > P_{\mathrm{EMG-02}} > P_{\mathrm{EMG-01}}
\end{aligned}
$$

- **Priority Invariant:** Higher priority contingency triggers preempt lower priority states unconditionally.
- **Deterministic Timing:** Maximum detection-to-actuation latency $t_{\mathrm{resp}} \le \tau_{\mathrm{deadline}}$ across all triggers.
- **Fail-Safe Retention:** Non-reentrant emergency containment locks until authorized manual ground reset.

### 12.2 Deterministic Emergency Statechart & State Machine
```mermaid
stateDiagram-v2
    [*] --> Phase_Startup
    Phase_Startup --> Phase_NominalExecution : BIT_Pass
    Phase_NominalExecution --> Degraded_SensorFailsafe : EMG_04_SensorFault
    Phase_NominalExecution --> Contingency_LostLinkReturn : EMG_01_LostC2
    Phase_NominalExecution --> Contingency_DeadReckoning : EMG_02_GNSSLoss
    Phase_NominalExecution --> Contingency_ResourceDivert : EMG_03_PowerDepletion
    Phase_NominalExecution --> Contingency_GeofenceContainment : EMG_05_GeofenceBreach
    Phase_NominalExecution --> Contingency_PrecautionaryHalt : EMG_06_StructuralAnomaly
    Phase_NominalExecution --> Emergency_SafeStateTermination : EMG_07_AbortCommand
    Degraded_SensorFailsafe --> Contingency_LostLinkReturn : LinkTimeout
    Contingency_LostLinkReturn --> Phase_SecureShutdown : SafeContainment
    Contingency_DeadReckoning --> Phase_SecureShutdown : SafeContainment
    Contingency_ResourceDivert --> Phase_SecureShutdown : SafeContainment
    Contingency_GeofenceContainment --> Contingency_ResourceDivert : ContainmentHold
    Contingency_PrecautionaryHalt --> Phase_SecureShutdown : SafeStop
    Emergency_SafeStateTermination --> Phase_SecureShutdown : ImpactSafe
    Phase_SecureShutdown --> [*]
```

### 12.3 Degraded Modes & Fallback Hierarchy
- **Tier 1 (Nominal Execution):** Full multi-sensor fusion, dual-channel C2 links, and nominal envelope margins.
- **Tier 2 (Degraded Sensor Mode):** Single-sensor failure activates secondary observer and dead reckoning.
- **Tier 3 (Contingency Link Mode):** Loss of primary C2 link triggers autonomous hold and return sequence.
- **Tier 4 (Emergency Containment Mode):** Unrecoverable fault triggers ballistic containment deploy or instant power cutoff.

### 12.4 Human-in-the-Loop (HITL) Authority & Override Protocols
- **Supervisory Authority:** Operator retains positive manual override capability via independent emergency link.
- **Dual-Consent Authentication:** Critical emergency termination (`EMG-07`) requires two-operator verified consent keys.
- **Interlock Inhibit:** Safety computer rejects manual commands that violate dynamic geofence containment limits.

### 12.5 Autonomous Divert & Secondary Recovery Protocols
- **Primary Recovery:** Designated nominal operational site or recovery zone.
- **Secondary Divert Sites:** Pre-surveyed alternate recovery coordinates evaluated dynamically against Bingo energy.
- **Terrain Clearance:** All emergency divert trajectories maintain minimum statutory boundary separation.

### 12.6 Post-Emergency Containment, Latching & Reset Procedures
- **Safety Lockout:** Emergency shutdown latches all actuators and high-voltage buses in de-energized safe states.
- **Non-Volatile Blackbox Offload:** Diagnostic fault logs, sensor telemetry, and watchdog stack traces are securely written to non-volatile flash.
- **Authorized Ground Clearance:** Physical inspection and signed maintenance clearance required before clearing failsafe lock.
"""
        out_p = Path(output_path)
        out_p.parent.mkdir(parents=True, exist_ok=True)
        out_p.write_text(template_text, encoding="utf-8")
        return True


# =============================================================================
# Gate 26: MissionIntentCompletenessValidator
# =============================================================================

class MissionIntentCompletenessValidator(IValidator):
    """
    Quality Gate 26: Mission Intent Completeness Validator.
    Enforces 10 mandatory sections, METL roster MET-01..N with Gate 24 allocation tags,
    MoE/MoP metrics, Threat/EW matrix, PACE C2 plan, ROE interlocks, dynamic geo-zones,
    Go/No-Go matrix, and Bingo Energy mathematics with >= 20% statutory reserve.
    """

    MANDATORY_SECTIONS: List[Dict[str, Any]] = [
        {"num": 1, "title": "Commander's Intent & Operational Objectives", "aliases": ["commander's intent", "operational objectives", "purpose", "mission intent"]},
        {"num": 2, "title": "Mission Essential Task List (METL)", "aliases": ["mission essential task list", "metl", "met-", "essential tasks"]},
        {"num": 3, "title": "Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics", "aliases": ["measures of effectiveness", "measures of performance", "moe", "mop", "moe/mop", "metrics"]},
        {"num": 4, "title": "Multi-Domain Operational Threat & Contested Environment Matrix", "aliases": ["threat", "multi-domain threat", "electronic warfare", "ew matrix", "cyber environment", "threat matrix", "contested environment"]},
        {"num": 5, "title": "PACE C2 Link Communications Plan", "aliases": ["pace c2", "pace plan", "pace communications plan", "c2 link communications plan", "pace"]},
        {"num": 6, "title": "Rules of Engagement (ROE) & Weapon/Sensor Interlocks", "aliases": ["rules of engagement", "roe", "weapon/sensor interlocks", "sensor interlocks", "roe interlocks", "interlocks"]},
        {"num": 7, "title": "Airspace Deconfliction & U-space Dynamic Geo-Zones", "aliases": ["airspace deconfliction", "u-space", "geo-zones", "dynamic geo-zones", "airspace", "geofence"]},
        {"num": 8, "title": "Go/No-Go Decision Matrix", "aliases": ["go/no-go", "go-no-go", "go / no-go decision matrix", "go / no-go matrix", "gng-"]},
        {"num": 9, "title": "Bingo Energy Mathematics & Secondary Divert Protocols", "aliases": ["bingo energy", "secondary divert", "divert protocols", "bingo energy mathematics", "bingo", "divert"]},
        {"num": 10, "title": "Gate 24 MissionTask Traceability Tags", "aliases": ["gate 24", "missiontask traceability", "traceability tags", "allocation tags", "operationalallocation"]},
    ]

    MANDATORY_THREAT_DOMAINS: List[Dict[str, Any]] = [
        {"name": "Kinetic", "pattern": r'\b(?:kinetic|thr-kin|ballistic|projectile|collision)\b'},
        {"name": "Mechanical", "pattern": r'\b(?:mechanical|structural|thr-mec|actuator\s+jam|flutter)\b'},
        {"name": "Power/Thermal", "pattern": r'\b(?:power/thermal|power\s*/\s*thermal|power\s+and\s+thermal|power|thermal|thr-pwr|thr-thm)\b'},
        {"name": "Environmental", "pattern": r'\b(?:environmental|atmospheric|weather|icing|precipitation|gust|thr-env)\b'},
        {"name": "EW", "pattern": r'\b(?:ew\b|electronic\s+warfare|electromagnetic(?:\s*/\s*rf)?|rf\s+jamming|gnss\s+jamming|thr-ew|thr-ewc)\b'},
        {"name": "Cyber", "pattern": r'\b(?:cyber\b|cybersecurity|data\s+integrity|packet\s+injection|firmware\s+tampering|thr-cyb)\b'},
        {"name": "Optical", "pattern": r'\b(?:optical|laser\s+blinding|dazzling|camera\s+saturation|thr-opt)\b'},
        {"name": "Signature", "pattern": r'\b(?:signature|acoustic|infrared|rcs\b|radar\s+cross-section|thr-sig|thr-ac)\b'},
        {"name": "Human Factors", "pattern": r'\b(?:human\s+factors|human|operator\s+fatigue|pilot|input\s+disparity|thr-hum)\b'},
        {"name": "CBRN", "pattern": r'\b(?:cbrn\b|chemical|biological|radiological|nuclear|toxic|hazardous\s+contamination|thr-cbrn)\b'},
    ]

    def __init__(self, strict_bingo_math: bool = True) -> None:
        self.strict_bingo_math = strict_bingo_math

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        findings: List[Finding] = []
        workspace_dir = repo.workspace_dir
        conops_dir = os.path.join(workspace_dir, "docs", "conops")

        if not os.path.isdir(conops_dir):
            if repo.is_upstream_compiler_repo():
                return []
            return [Finding(
                "mission-intent-corpus-missing",
                "Mission Intent corpus directory 'docs/conops' is missing in workspace.",
                location="docs/conops",
            )]

        mission_files: List[str] = []
        for root, _, files in os.walk(conops_dir):
            for f in sorted(files):
                if ("MISSION_INTENT" in f.upper() or "MISSIONINTENT" in f.upper()) and f.endswith(".md") and "TEMPLATE" not in f.upper():
                    mission_files.append(os.path.join(root, f))

        if not mission_files:
            if repo.is_upstream_compiler_repo():
                return []
            return [Finding(
                "mission-intent-corpus-missing",
                "No valid Mission Intent specification document ('MISSION_INTENT*.md') found in 'docs/conops'.",
                location="docs/conops",
            )]

        for m_file in mission_files:
            rel_path = os.path.relpath(m_file, workspace_dir)
            try:
                with open(m_file, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception as e:
                findings.append(Finding(
                    "mission-read-error",
                    f"Failed to read Mission Intent document '{rel_path}': {e}",
                    location=rel_path,
                ))
                continue

            findings.extend(self._validate_mission_text(content, rel_path, repo=repo))

        return findings

    def _validate_mission_text(self, content: str, rel_path: str, repo: Optional[WorkspaceRepository] = None) -> List[Finding]:
        findings: List[Finding] = []
        sections_list = _extract_markdown_sections_list(content)
        sections = _extract_markdown_sections(content)
        tables, malformed_lines = _parse_commonmark_tables(content)

        # Check for allocated obligations from RESEARCH_INVENTORY.md (Gate 26, Issue #109)
        if "TEMPLATE" not in rel_path.upper():
            inventory_file = None
            if repo is not None:
                inventory_file = os.path.join(repo.workspace_dir, "docs", "research", "RESEARCH_INVENTORY.md")
            else:
                candidate = os.path.join("docs", "research", "RESEARCH_INVENTORY.md")
                if os.path.isfile(candidate):
                    inventory_file = candidate

            if inventory_file and os.path.isfile(inventory_file):
                try:
                    with open(inventory_file, "r", encoding="utf-8") as inv_f:
                        inv_doc = parse_research_inventory(inv_f.read())

                    # Extract document's realized/witnessed tags
                    doc_tags = set(_parse_witness_tags(content) + _parse_obligation_tags(content))
                    fm = extract_metadata_from_content(content)
                    if fm:
                        for key in ("obligations", "normative_obligations", "allocated_obligations", "obligation_id"):
                            val = fm.get(key)
                            if isinstance(val, list):
                                for item in val:
                                    norm = _normalize_obligation_id(str(item))
                                    if norm:
                                        doc_tags.add(norm)
                            elif isinstance(val, str):
                                norm = _normalize_obligation_id(val)
                                if norm:
                                    doc_tags.add(norm)

                    # Find all obligations allocated to MISSION_INTENT in clause allocations
                    for alloc in inv_doc.clause_allocations:
                        if alloc.population_id and alloc.downstream_spec_file:
                            spec_target = alloc.downstream_spec_file.strip("`* ")
                            target_base = os.path.basename(spec_target).upper()
                            is_mission_target = (
                                "MISSION_INTENT" in target_base
                                or "MISSIONINTENT" in target_base
                                or os.path.normpath(spec_target) == os.path.normpath(rel_path)
                            )
                            if is_mission_target:
                                norm_id = _normalize_obligation_id(alloc.population_id)
                                if norm_id not in doc_tags:
                                    findings.append(Finding(
                                        "mission-intent-obligation-unwitnessed",
                                        f"Mission Intent document '{rel_path}' is missing mandatory witness/realization tag for allocated obligation '{norm_id}' ({alloc.clause_citation or alloc.standard_id}).",
                                        location=rel_path,
                                        detail={"obligation_id": norm_id, "file": rel_path, "standard_id": alloc.standard_id},
                                    ))
                except Exception:
                    pass

        # Check for un-substituted template placeholders (Gate 26, Fixes #142)
        if "TEMPLATE" not in rel_path.upper():
            unresolved = _find_unresolved_template_placeholders(content)
            if unresolved:
                unresolved_tags = [tag for _, tag in unresolved]
                unique_tags = sorted(list(set(unresolved_tags)))
                line_numbers = sorted(list(set(line for line, _ in unresolved)))
                first_line = unresolved[0][0]
                findings.append(Finding(
                    "mission-unresolved-template-placeholders",
                    f"Mission Intent specification '{rel_path}' contains {len(unresolved)} unresolved template placeholder token(s): {', '.join(unique_tags)} at line(s) {', '.join(str(l) for l in line_numbers)}.",
                    location=f"{rel_path}:{first_line}",
                    detail={
                        "severity": "CRITICAL",
                        "unresolved_tags": unique_tags,
                        "line_numbers": line_numbers,
                        "placeholders": [f"{tag} (line {line})" for line, tag in unresolved],
                        "file": rel_path,
                    },
                ))

        # Check line density floor for non-templates (Gate 26, Fixes #130)
        if "TEMPLATE" not in rel_path.upper():
            line_count = len(content.splitlines())
            if line_count < 400:
                findings.append(Finding(
                    "mission-density-insufficient",
                    f"Mission Intent specification '{rel_path}' has insufficient line density ({line_count} lines; minimum required: 400 lines).",
                    location=rel_path,
                    detail={"line_count": line_count, "min_required_lines": 400, "file": rel_path},
                ))

        # Check for malformed tables
        for m_line in malformed_lines:
            findings.append(Finding(
                "mission-table-malformed",
                f"Malformed CommonMark table or broken row formatting detected at line {m_line} in '{rel_path}'.",
                location=f"{rel_path}:{m_line}",
            ))

        # Extract content sections (excluding TOC and top-level H1 title)
        content_sections: List[Tuple[str, int, str]] = []
        raw_lines = content.splitlines()
        for h, l_num, c in sections_list:
            if re.match(r'^(?:table\s+of\s+contents|toc)$', h.strip(), re.IGNORECASE):
                continue
            if 1 <= l_num <= len(raw_lines):
                line_str = raw_lines[l_num - 1].strip()
                if line_str.startswith("# ") and not line_str.startswith("## "):
                    continue
            content_sections.append((h, l_num, c))

        # Check Table of Contents Completeness for non-templates (Fixes #148)
        if "TEMPLATE" not in rel_path.upper():
            toc_matches = [
                s for s in sections_list
                if re.match(r'^(?:table\s+of\s+contents|toc)$', s[0].strip(), re.IGNORECASE)
            ]
            if not toc_matches:
                findings.append(Finding(
                    "mission-toc-missing",
                    f"Mission Intent specification '{rel_path}' is missing mandatory Table of Contents ('## Table of Contents').",
                    location=rel_path,
                    detail={"severity": "CRITICAL", "file": rel_path},
                ))
            else:
                toc_heading, toc_line, toc_chunk = toc_matches[0]
                missing_toc_sections: List[Tuple[int, str]] = []
                for req in self.MANDATORY_SECTIONS:
                    sec_num = req["num"]
                    sec_title = req["title"]
                    sec_aliases = req["aliases"]

                    num_pattern_text = rf'\[(?:section\s+)?{sec_num}[.\s:\-—]'
                    num_pattern_anchor = rf'\(#(?:section-)?{sec_num}[-_]'
                    found_in_toc = bool(
                        re.search(num_pattern_text, toc_chunk, re.IGNORECASE)
                        or re.search(num_pattern_anchor, toc_chunk, re.IGNORECASE)
                    )
                    if not found_in_toc:
                        for alias in sec_aliases:
                            if re.search(rf'\[[^\]]*{re.escape(alias)}[^\]]*\]', toc_chunk, re.IGNORECASE):
                                found_in_toc = True
                                break

                    if not found_in_toc:
                        missing_toc_sections.append((sec_num, sec_title))

                if missing_toc_sections:
                    missing_str = ", ".join(f"Section {n} ('{t}')" for n, t in missing_toc_sections)
                    findings.append(Finding(
                        "mission-toc-incomplete",
                        f"Table of Contents in '{rel_path}' is incomplete; missing {len(missing_toc_sections)} mandatory section(s): {missing_str}.",
                        location=f"{rel_path}:{toc_line}",
                        detail={
                            "severity": "CRITICAL",
                            "missing_sections": [n for n, _ in missing_toc_sections],
                            "file": rel_path,
                        },
                    ))

        # Check for Duplicate Section Headers (Fixes #148)
        reported_duplicate_lines: Set[int] = set()
        for req in self.MANDATORY_SECTIONS:
            sec_num = req["num"]
            sec_title = req["title"]
            sec_aliases = req["aliases"]
            matches = _find_matching_sections_all(content_sections, sec_num, sec_title, sec_aliases)
            if len(matches) > 1:
                for dup_heading, dup_line, _ in matches[1:]:
                    reported_duplicate_lines.add(dup_line)
                    findings.append(Finding(
                        "mission-duplicate-section-header",
                        f"Duplicate section header detected for Section {sec_num} ('{dup_heading}') at line {dup_line} in '{rel_path}'.",
                        location=f"{rel_path}:{dup_line}",
                        detail={
                            "severity": "CRITICAL",
                            "section_number": sec_num,
                            "heading": dup_heading,
                            "line": dup_line,
                            "file": rel_path,
                        },
                    ))

        seen_h2_headers: Dict[str, Tuple[int, str]] = {}
        for h, l_num, _ in content_sections:
            h_norm = re.sub(r'[*`_]', '', h).strip().lower()
            if h_norm in seen_h2_headers:
                if l_num not in reported_duplicate_lines:
                    reported_duplicate_lines.add(l_num)
                    findings.append(Finding(
                        "mission-duplicate-section-header",
                        f"Duplicate section header detected ('{h}') at line {l_num} in '{rel_path}'.",
                        location=f"{rel_path}:{l_num}",
                        detail={
                            "severity": "CRITICAL",
                            "heading": h,
                            "line": l_num,
                            "file": rel_path,
                        },
                    ))
            else:
                seen_h2_headers[h_norm] = (l_num, h)

        # Check Exact Section Cardinality (Fixes #148)
        if len(content_sections) != 10:
            findings.append(Finding(
                "mission-section-cardinality-mismatch",
                f"Mission Intent specification '{rel_path}' has section cardinality mismatch (found {len(content_sections)} section(s); expected exactly 10 sections).",
                location=rel_path,
                detail={
                    "severity": "CRITICAL",
                    "expected_sections": 10,
                    "actual_sections": len(content_sections),
                    "file": rel_path,
                },
            ))

        # Check for 10 Mandatory Sections
        matched_sections: Dict[int, Tuple[str, int, str]] = {}
        for req in self.MANDATORY_SECTIONS:
            sec_num = req["num"]
            title = req["title"]
            aliases = req["aliases"]
            res = _find_matching_section(content_sections, sec_num, title, aliases)
            if res:
                matched_sections[sec_num] = res
            else:
                findings.append(Finding(
                    "mission-section-missing",
                    f"Mandatory Mission Intent Section {sec_num} ('{title}') is missing or empty in '{rel_path}'.",
                    location=rel_path,
                    detail={"section_number": sec_num, "section_title": title},
                ))

        # Section 2 & 10: METL Roster & Gate 24 Allocation Traceability (Theorem 3)
        if 2 in matched_sections:
            _, sec2_line, sec2_content = matched_sections[2]
            declared_met_tasks: Set[str] = set()

            # 1. Parse tables if present in Section 2
            sec2_tables, _ = _parse_commonmark_tables(sec2_content)
            for tbl in sec2_tables:
                for row in tbl:
                    task_val = (
                        row.get("task_id")
                        or row.get("id")
                        or row.get("task")
                        or row.get("metl_id")
                        or row.get("met_id")
                    )
                    if not task_val and row:
                        first_k = list(row.keys())[0]
                        task_val = row[first_k]
                    if task_val:
                        clean_val = re.sub(r'[*`_]', '', task_val).strip()
                        m = re.search(r'\b(MET-0*[0-9]+)\b', clean_val, re.IGNORECASE)
                        if m:
                            num = int(m.group(1).split("-")[1])
                            declared_met_tasks.add(f"MET-{num:02d}")

            # 2. Extract declared MET tasks from text lines / bullet points
            met_task_matches = re.finditer(
                r'\b(MET-0*[0-9]+)\b',
                sec2_content,
                re.IGNORECASE,
            )
            for m in met_task_matches:
                num = int(m.group(1).split("-")[1])
                declared_met_tasks.add(f"MET-{num:02d}")

            sorted_tasks = sorted(list(declared_met_tasks))

            # Check allocation tags across the document
            for task_id in sorted_tasks:
                # Look for '/// OperationalAllocation: [ ... task_id ... ]' or '/// OperationalAllocation: task_id'
                tag_pattern = rf'///\s*OperationalAllocation\s*:\s*(?:\[[^\]]*(?<![A-Za-z0-9_-]){re.escape(task_id)}(?![A-Za-z0-9_-])[^\]]*\]|(?<![A-Za-z0-9_-]){re.escape(task_id)}(?![A-Za-z0-9_-]))'
                if not re.search(tag_pattern, content, re.IGNORECASE):
                    findings.append(Finding(
                        "mission-metl-unallocated",
                        f"Mission Essential Task '{task_id}' declared in Section 2 has no valid Gate 24 allocation tag ('/// OperationalAllocation: [{task_id}]') in '{rel_path}'.",
                        location=f"{rel_path}:{sec2_line}",
                        detail={"task_id": task_id},
                    ))

        # Section 4: Multi-Domain Operational Threat Matrix Density (10 domains)
        if 4 in matched_sections:
            _, sec4_line, sec4_content = matched_sections[4]
            findings.extend(self._validate_threat_matrix_density(content, rel_path, sec4_content, sec4_line))

        # Section 9: Bingo Energy Mathematics & Reserve Ratio Validation (Fixes #130)
        if 9 in matched_sections:
            _, sec9_line, sec9_content = matched_sections[9]
            capacity_val, reserve_val = _extract_bingo_energy_parameters(sec9_content)

            if capacity_val is not None and reserve_val is not None and self.strict_bingo_math:
                ratio = calculate_bingo_energy_reserve_ratio(total_capacity_j=capacity_val, reserve_energy_j=reserve_val)
                if ratio < 0.199:  # Strict 20% minimum statutory reserve
                    findings.append(Finding(
                        "mission-bingo-reserve-insufficient",
                        f"Statutory reserve energy E_reserve = {reserve_val:.1f} J is {ratio*100.0:.1f}% of total capacity {capacity_val:.1f} J in Section 9, violating mandatory 20.0% statutory reserve threshold.",
                        location=f"{rel_path}:{sec9_line}",
                        detail={"capacity_joules": capacity_val, "reserve_joules": reserve_val, "reserve_ratio": ratio},
                    ))

        # Check Mermaid Diagram Integrity (Fixes #114, #130)
        findings.extend(_validate_mermaid_integrity(content, rel_path, prefix="mission"))

        return findings

    def _validate_threat_matrix_density(
        self,
        content: str,
        rel_path: str,
        sec4_content: str,
        sec4_line: int,
    ) -> List[Finding]:
        """
        Validates density of Section 4 (Multi-Domain Operational Threat Matrix):
        Ensures all 10 canonical operational domains are covered.
        """
        findings: List[Finding] = []
        sec4_tables, _ = _parse_commonmark_tables(sec4_content)
        found_domains: Set[str] = set()

        def _scan_domain(text: str) -> None:
            for domain_spec in self.MANDATORY_THREAT_DOMAINS:
                d_name = domain_spec["name"]
                d_pattern = domain_spec["pattern"]
                if re.search(d_pattern, text, re.IGNORECASE):
                    found_domains.add(d_name)

        if sec4_tables:
            for tbl in sec4_tables:
                for row in tbl:
                    row_str = " ".join(row.values())
                    _scan_domain(row_str)

        # Also scan raw section content
        _scan_domain(sec4_content)

        missing_domains = [d["name"] for d in self.MANDATORY_THREAT_DOMAINS if d["name"] not in found_domains]
        if missing_domains:
            findings.append(Finding(
                "mission-threat-domain-missing",
                f"Section 4 Multi-Domain Operational Threat Matrix is missing required threat domain(s): {', '.join(missing_domains)} in '{rel_path}'.",
                location=f"{rel_path}:{sec4_line}",
                detail={"missing_domains": missing_domains, "found_domains": list(found_domains)},
            ))

        return findings

    def synthesize_canonical_template(self, output_path: Union[str, Path]) -> bool:
        """Synthesizes domain-neutral MISSION_INTENT_CANONICAL_TEMPLATE.md."""
        res_path = Path(__file__).resolve().parents[4] / "resources" / "MISSION_INTENT_CANONICAL_TEMPLATE.md"
        if res_path.is_file():
            template_text = res_path.read_text(encoding="utf-8")
        else:
            template_text = r"""| Attribute | Value |
| :--- | :--- |
| **Title** | Tactical Mission Intent & Execution Plan: {{MISSION_SYSTEM_NAME}} |
| **Version** | {{DOCUMENT_VERSION}} |
| **Date** | {{DOCUMENT_DATE}} |

# Tactical Mission Intent & Execution Plan: {{MISSION_SYSTEM_NAME}}

## 1. Commander's Intent & Operational Objectives
- **Operational Purpose:** {{OPERATIONAL_PURPOSE}}
- **Key Tasks:** {{KEY_MISSION_TASKS}}
- **End State:** {{MISSION_END_STATE}}

## 2. Mission Essential Task List (METL)
| Task ID | Task Name | Condition Statement | Standard Metric | Verification Method | Gate 24 Allocation Tag |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `MET-01` | {{MET_TASK_NAME}} | {{MET_CONDITION}} | {{MET_STANDARD}} | {{MET_VERIFICATION}} | `/// OperationalAllocation: [MET-01]` |

## 3. Measures of Effectiveness (MoE) & Measures of Performance (MoP) Metrics
| Metric ID | Metric Type | Metric Name | Formulation / Equation | Threshold | Objective | Unit |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| MoE-01 | MoE | {{MOE_NAME}} | {{MOE_EQUATION}} | {{MOE_THRESHOLD}} | {{MOE_OBJECTIVE}} | {{MOE_UNIT}} |
| MoP-01 | MoP | {{MOP_NAME}} | {{MOP_EQUATION}} | {{MOP_THRESHOLD}} | {{MOP_OBJECTIVE}} | {{MOP_UNIT}} |

## 4. Multi-Domain Operational Threat & Contested Environment Matrix
| Threat ID | Threat Domain | Threat Vector | Technical Description | Severity | Detection Mechanism | Autonomous Mitigation Rule | Public Clause Citation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `THR-KIN-01` | Kinetic | {{THR_KIN_VECTOR}} | {{THR_KIN_DESCRIPTION}} | Critical | Proximity lidar / vision bounding box | Execute evasive lateral displacement maneuver | MIL-STD-882E §4.3 |
| `THR-MEC-01` | Mechanical | {{THR_MEC_VECTOR}} | {{THR_MEC_DESCRIPTION}} | Critical | Actuator telemetry / vibration monitor | Reconfigure dynamic control allocation matrix | MIL-STD-882E §4.3 |
| `THR-PWR-01` | Power/Thermal | {{THR_PWR_VECTOR}} | {{THR_PWR_DESCRIPTION}} | Critical | BMS thermistor array / current sensor | Isolate faulted module and initiate divert | MIL-STD-882E §4.3 |
| `THR-ENV-01` | Environmental | {{THR_ENV_VECTOR}} | {{THR_ENV_DESCRIPTION}} | High | Pitot air data / temperature sensor | Transition to high-stability penetration mode | MIL-STD-810H Method 514.8 |
| `THR-EWC-01` | EW | {{THR_EW_VECTOR}} | {{THR_EW_DESCRIPTION}} | High | RAIM alert / SNR degradation | Switch frequency-hopping channel / alternate PACE | STANAG 4586 §3.2 |
| `THR-CYB-01` | Cyber | {{THR_CYB_VECTOR}} | {{THR_CYB_DESCRIPTION}} | Critical | Cryptographic HMAC validation failure | Drop unauthorized frames, cycle crypto keys | NIST SP 800-82r3 §5.2 |
| `THR-OPT-01` | Optical | {{THR_OPT_VECTOR}} | {{THR_OPT_DESCRIPTION}} | High | Optical sensor saturation / dazzle detector | Shutter sensor and switch to secondary modality | MIL-STD-882E §4.3 |
| `THR-SIG-01` | Signature | {{THR_SIG_VECTOR}} | {{THR_SIG_DESCRIPTION}} | Medium | Acoustic / emission monitor | Reduce rotor RPM and optimize acoustic signature | MIL-STD-882E §4.3 |
| `THR-HUM-01` | Human Factors | {{THR_HUM_VECTOR}} | {{THR_HUM_DESCRIPTION}} | High | Command rate disparity / syntax validator | Sanitize input commands and enforce interlocks | ISO/IEC/IEEE 29148 §6.4 |
| `THR-CBRN-01` | CBRN | {{THR_CBRN_VECTOR}} | {{THR_CBRN_DESCRIPTION}} | High | Particulate / chemical sensor threshold | Seal enclosure air intake and route clear of plume | MIL-STD-810H Method 509.7 |

## 5. PACE C2 Link Communications Plan
| PACE Tier | Link Medium | Frequency Band | Nominal Data Rate | Heartbeat Timeout | Priority / Role |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Primary** | {{PACE_PRIMARY_MEDIUM}} | {{PACE_PRIMARY_BAND}} | {{PACE_PRIMARY_DATA_RATE}} | {{PACE_PRIMARY_TIMEOUT}} | {{PACE_PRIMARY_ROLE}} |
| **Alternate** | {{PACE_ALTERNATE_MEDIUM}} | {{PACE_ALTERNATE_BAND}} | {{PACE_ALTERNATE_DATA_RATE}} | {{PACE_ALTERNATE_TIMEOUT}} | {{PACE_ALTERNATE_ROLE}} |
| **Contingency** | {{PACE_CONTINGENCY_MEDIUM}} | {{PACE_CONTINGENCY_BAND}} | {{PACE_CONTINGENCY_DATA_RATE}} | {{PACE_CONTINGENCY_TIMEOUT}} | {{PACE_CONTINGENCY_ROLE}} |
| **Emergency** | {{PACE_EMERGENCY_MEDIUM}} | {{PACE_EMERGENCY_BAND}} | {{PACE_EMERGENCY_DATA_RATE}} | {{PACE_EMERGENCY_TIMEOUT}} | {{PACE_EMERGENCY_ROLE}} |

## 6. Rules of Engagement (ROE) & Weapon/Sensor Interlocks
- **ROE-01:** {{ROE_RULE_STATEMENT}}

## 7. Airspace Deconfliction & U-space Dynamic Geo-Zones
- **Primary Boundary Perimeter:** {{PRIMARY_BOUNDARY_PERIMETER}}
- **Dynamic Exclusion Zones:** {{DYNAMIC_EXCLUSION_ZONES}}
- **Separation Minima:** {{SEPARATION_MINIMA}}

## 8. Go/No-Go Decision Matrix
| Check ID | Phase | Parameter / Check | Threshold Condition | Sensor / Mechanism | Go / No-Go Action |
| :--- | :--- | :--- | :--- | :--- | :--- |
| GNG-01 | {{GNG_PHASE}} | {{GNG_PARAMETER}} | {{GNG_THRESHOLD}} | {{GNG_MECHANISM}} | {{GNG_ACTION}} |

## 9. Bingo Energy Mathematics & Secondary Divert Protocols
$$
\begin{aligned}
E_{\mathrm{bingo}}(t) &= E_{\mathrm{return}}(\mathbf{p}(t), \mathbf{p}_{\mathrm{dest}}) + E_{\mathrm{divert}}(\mathbf{p}_{\mathrm{dest}}, \mathbf{p}_{\mathrm{alt}}) + E_{\mathrm{reserve}} + E_{\mathrm{contingency}} \\
E_{\mathrm{reserve}} &\ge 0.20 \cdot E_{\mathrm{capacity}}
\end{aligned}
$$

| Energy Parameter | Symbol | Value | Units | Constraint Rule |
| :--- | :--- | :--- | :--- | :--- |
| Total Storage Capacity | E_capacity | {{E_CAPACITY_JOULES}} | J | Total nominal energy storage capacity |
| Return Transit Energy | E_return | {{E_RETURN_JOULES}} | J | Energy required for primary return trajectory |
| Secondary Divert Energy | E_divert | {{E_DIVERT_JOULES}} | J | Energy required to divert to secondary recovery site |
| Mandatory Statutory Reserve | E_reserve | {{E_RESERVE_JOULES}} | J | Statutory reserve threshold (E_reserve >= 0.20 * E_capacity) |
| Contingency Buffer | E_contingency | {{E_CONTINGENCY_JOULES}} | J | Dynamic operational contingency energy reserve |
| Total Bingo Threshold | E_bingo | {{E_BINGO_THRESHOLD_JOULES}} | J | Critical return threshold condition |

## 10. Gate 24 MissionTask Traceability Tags (Allocation Tags)
- `/// OperationalAllocation: [MET-01]`
"""
        out_p = Path(output_path)
        out_p.parent.mkdir(parents=True, exist_ok=True)
        out_p.write_text(template_text, encoding="utf-8")
        return True


if __name__ == "__main__":
    repo = WorkspaceRepository()
    v1 = ConopsCompletenessValidator()
    v2 = MissionIntentCompletenessValidator()
    errs1 = v1.validate(repo)
    errs2 = v2.validate(repo)
    all_errs = errs1 + errs2
    if all_errs:
        for err in all_errs:
            print(f"[{getattr(err, 'rule_id', 'ERROR')}] {err}")
        sys.exit(1)
    else:
        print("[OK] Gate 26 (ConOps & Mission Intent Completeness): All checks passed.")
        sys.exit(0)
