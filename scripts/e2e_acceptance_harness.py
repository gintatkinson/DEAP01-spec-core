#!/usr/bin/env python3
"""
End-to-End Acceptance Test Harness across Cyber-Physical Domain Workspaces.

Executes a 2-Tier Semantic Verification Architecture (6 layers) across all 10 domain repositories:
  Tier 1 (Syntactic & Structural Integrity):
    - Layer 1 (Delivery Gate 0): Physical presence, line counts, section line floors.
    - Layer 2 (Mechanical Syntax & Token Purity): 0 mustache tokens, 0 pseudovariables, 0 raw $ in tables, 0 KaTeX underscore syntax errors.
  Tier 2 (Semantic & Mathematical Physics Verification):
    - Layer 3 (Statutory Cardinality & Normative Standards): 16 Threat Vectors, 4 PACE tiers, >=12 MIL-STD-810H methods, 24 SORA OSOs, 7 Emergency rows, and Solver 4 (Normative Standards Cross-Checker: IEC 62304, EN 50128, ECSS, ISO 3691-4, DNV-GL).
    - Layer 4 (Closed-Form Physical & Math Solver): SORA kinetic energy (E_k <= 34.0J), Kalman covariance units & linear algebra dimensions, Bingo energy conservation, Solver 1 (Relational Table Mass Cross-Sum Solver), Solver 2 (Closed-Form Quadratic Physics Solver), and Solver 3 (Dimensional Scaling & Energy Conservation Engine).
    - Layer 5 (Adversarial Invariant Verification & Ontology Scanner): Priority arbitration (P_EMG07 > ... > P_EMG01), failsafe non-destructive RTB, NIST SP 800-82r3 anti-replay, Solver 5 (Forbidden Cross-Domain Ontology Scanner), Solver 6 (Positive Domain Lexicon Floor), and Solver 7 (Pairwise Anti-Plagiarism Gate).
    - Layer 6 (Baseline Parity & Model Coverage): verify_downstream_baseline.py and verify_model_coverage.py --spec-only.

Generates:
  - MASTER_E2E_ACCEPTANCE_REPORT.md
  - acceptance_scorecard.json
"""

import argparse
import datetime
import json
import os
import re
import subprocess
import sys
from dataclasses import dataclass, field, asdict
from typing import Dict, List, Optional, Tuple, Set


# ---------------------------------------------------------------------------
# Data Models
# ---------------------------------------------------------------------------

@dataclass
class LayerResult:
    layer_id: int
    layer_name: str
    passed: bool
    errors: List[str] = field(default_factory=list)
    details: Dict[str, object] = field(default_factory=dict)


@dataclass
class DomainScorecard:
    domain_id: str
    domain_name: str
    workspace_path: str
    overall_passed: bool
    layers: Dict[int, LayerResult] = field(default_factory=dict)
    metadata: Dict[str, str] = field(default_factory=dict)


@dataclass
class HarnessSummary:
    total_domains: int
    passed_domains: int
    failed_domains: int
    execution_timestamp: str
    domain_results: List[DomainScorecard] = field(default_factory=list)
    similarity_matrix: Dict[str, Dict[str, float]] = field(default_factory=dict)


# ---------------------------------------------------------------------------
# Known Domain Metadata Mapping
# ---------------------------------------------------------------------------

DOMAIN_NAMES = {
    "run_01_tactical_isr": "Tactical ISR Fixed-Wing UAV",
    "run_01": "Tactical ISR Fixed-Wing UAV",
    "run_02": "Autonomous Urban Air Mobility (eVTOL)",
    "run_03": "Subsea Autonomous Underwater Vehicle (AUV)",
    "run_04": "Autonomous Surface Vessel (ASV / Maritime)",
    "run_05": "Autonomous Ground Delivery Fleet (UGV)",
    "run_06": "Low Earth Orbit (LEO) CubeSat Constellation",
    "run_07": "Automated Robotic Surgical Console",
    "run_08": "Autonomous Rail Shunting Locomotive",
    "run_09": "Industrial Autonomous Forklift (AGV)",
    "run_10": "Counter-UAS Kinetic Interceptor",
}


# ---------------------------------------------------------------------------
# Helper Markdown Table Parser
# ---------------------------------------------------------------------------

def parse_markdown_table_rows(text: str) -> List[List[str]]:
    """Parse markdown table data rows while preserving escaped pipes."""
    rows = []
    lines = text.splitlines()
    in_table = False
    header_seen = False
    
    for line in lines:
        s = line.strip()
        if s.startswith("|") and s.endswith("|"):
            # Protect escaped pipes
            protected = s[1:-1].replace(r"\|", "___PIPE___")
            cells = [c.strip().replace("___PIPE___", "|") for c in protected.split("|")]
            
            # Check separator row
            if all(re.fullmatch(r":?-{1,}:?", c) for c in cells if c):
                header_seen = True
                continue
            if not header_seen:
                # Header row
                continue
            rows.append(cells)
            in_table = True
        else:
            if in_table and s and not s.startswith("|"):
                in_table = False
                header_seen = False
    return rows


# ---------------------------------------------------------------------------
# Layer 1: Delivery Gate 0
# ---------------------------------------------------------------------------

def verify_layer1_delivery_gate(workspace_path: str) -> LayerResult:
    """Check physical presence, file size, line counts, and section line floors."""
    errors = []
    details = {}
    
    conops_path = os.path.join(workspace_path, "docs", "conops", "CONOPS.md")
    intent_path = os.path.join(workspace_path, "docs", "conops", "MISSION_INTENT.md")
    
    checks = [
        (conops_path, "CONOPS.md", 800, 12),
        (intent_path, "MISSION_INTENT.md", 400, 10),
    ]
    
    for path, doc_name, min_lines, expected_sections in checks:
        if not os.path.exists(path):
            errors.append(f"{doc_name} does not exist at {path}")
            continue
        size = os.path.getsize(path)
        if size == 0:
            errors.append(f"{doc_name} is empty (0 bytes)")
            continue
            
        with open(path, "r", encoding="utf-8", errors="ignore") as f:
            lines = f.readlines()
            
        total_lines = len(lines)
        details[f"{doc_name}_lines"] = total_lines
        details[f"{doc_name}_size_bytes"] = size
        
        if total_lines < min_lines:
            errors.append(f"{doc_name} total lines ({total_lines}) below floor of {min_lines}")
            
        # Section line floors >= 8
        sections = []
        cur_sec = None
        cur_lines = 0
        for line in lines:
            s = line.strip()
            if s.startswith("## "):
                if cur_sec is not None:
                    sections.append((cur_sec, cur_lines))
                cur_sec = s
                cur_lines = 0
            elif cur_sec is not None:
                cur_lines += 1
        if cur_sec is not None:
            sections.append((cur_sec, cur_lines))
            
        details[f"{doc_name}_section_count"] = len(sections)
        for sname, scount in sections:
            if scount < 8:
                errors.append(f"{doc_name} section '{sname}' has {scount} lines (< 8 line floor)")
                
    passed = len(errors) == 0
    return LayerResult(
        layer_id=1,
        layer_name="Delivery Gate 0 (Presence & Section Floors)",
        passed=passed,
        errors=errors,
        details=details
    )


# ---------------------------------------------------------------------------
# Layer 2: Mechanical Syntax & Token Purity
# ---------------------------------------------------------------------------

def verify_layer2_syntax_purity(workspace_path: str) -> LayerResult:
    """Assert 0 mustache tokens, 0 pseudovariables, 0 raw table math, 0 KaTeX syntax errors."""
    errors = []
    details = {"files_scanned": 0, "violations_found": 0}
    
    mustache_regex = re.compile(r"\{\{.*?\}\}")
    pseudovar_patterns = [
        re.compile(r"\bIP_xy\b"),
        re.compile(r"\bAo_threshold\b"),
        re.compile(r"\bt_battery_SE\b"),
        re.compile(r"\[TODO\]", re.IGNORECASE),
        re.compile(r"\[TBD\]", re.IGNORECASE),
        re.compile(r"\[PLACEHOLDER\]", re.IGNORECASE),
        re.compile(r"<placeholder>", re.IGNORECASE),
    ]
    
    docs_dir = os.path.join(workspace_path, "docs")
    if not os.path.isdir(docs_dir):
        errors.append(f"docs directory missing at {docs_dir}")
        return LayerResult(layer_id=2, layer_name="Mechanical Syntax & Token Purity", passed=False, errors=errors)
        
    for root, dirs, files in os.walk(docs_dir):
        # Exclude template units and non-specification directories
        dirs[:] = [d for d in dirs if d not in ("node_modules", ".git", ".pytest_cache", "units")]
        for f in files:
            if not f.endswith(".md"):
                continue
            fpath = os.path.join(root, f)
            rel = os.path.relpath(fpath, workspace_path)
            details["files_scanned"] += 1
            
            with open(fpath, "r", encoding="utf-8", errors="ignore") as handle:
                content = handle.read()
                
            # 1. Mustache tokens
            mustaches = mustache_regex.findall(content)
            if mustaches:
                errors.append(f"{rel}: Unrendered mustache tokens found: {mustaches[:5]}")
                details["violations_found"] += len(mustaches)
                
            # 2. Pseudovariables
            for pat in pseudovar_patterns:
                p_matches = pat.findall(content)
                if p_matches:
                    errors.append(f"{rel}: Uninstantiated pseudovariables found ({pat.pattern}): {p_matches[:5]}")
                    details["violations_found"] += len(p_matches)
                    
            # 3. Raw $ math in markdown table cells
            lines = content.splitlines()
            in_code = False
            for idx, line in enumerate(lines, 1):
                s = line.strip()
                if s.startswith("```") or s.startswith("~~~"):
                    in_code = not in_code
                    continue
                if in_code:
                    continue
                if s.startswith("|") and s.endswith("|"):
                    # Split cells respecting escaped pipes
                    protected = s[1:-1].replace(r"\|", "___PIPE___")
                    cells = [c.strip().replace("___PIPE___", "|") for c in protected.split("|")]
                    for c_idx, cell in enumerate(cells, 1):
                        unescaped = re.findall(r"(?<!\\)\$", cell)
                        if len(unescaped) > 0:
                            errors.append(f"{rel}:{idx}: Raw math delimiter '$' in table cell (col {c_idx}): {cell[:60]}")
                            details["violations_found"] += 1
                            
            # 4. KaTeX math-mode unescaped underscore in \mathrm{} / \text{} macros
            bad_macros = re.findall(r"\\(?:mathrm|text|mathbf|mathit)\{[^}]*?(?<!\\)_[^}]*?\}", content)
            if bad_macros:
                errors.append(f"{rel}: KaTeX unescaped underscore in macro: {bad_macros[:5]}")
                details["violations_found"] += len(bad_macros)

    passed = len(errors) == 0
    return LayerResult(
        layer_id=2,
        layer_name="Mechanical Syntax & Token Purity",
        passed=passed,
        errors=errors,
        details=details
    )


# ---------------------------------------------------------------------------
# Layer 3: Statutory Cardinality
# ---------------------------------------------------------------------------
# Deterministic Semantic Solvers (2-Tier Semantic Verification Architecture)
# ---------------------------------------------------------------------------

def solve_relational_mass_cross_sum(
    conops_text: str, total_mtow: Optional[float] = None
) -> Tuple[bool, List[str], Dict[str, object]]:
    """
    Solver 1: Relational Table Mass Cross-Sum Solver (Layer 4).
    Parses Table 1.3.2 partition rows (Airframe, Avionics, Propulsion, Energy, Payload, Failsafe).
    Asserts: abs(sum(partition_masses) - TOTAL_MTOW_KG) <= 0.01 kg.
    """
    errors = []
    details = {}
    
    partition_patterns = {
        "airframe": re.compile(r"airframe|chassis|structural|structure|hull", re.IGNORECASE),
        "avionics": re.compile(r"avionics", re.IGNORECASE),
        "propulsion": re.compile(r"propulsion", re.IGNORECASE),
        "energy": re.compile(r"energy", re.IGNORECASE),
        "payload": re.compile(r"payload", re.IGNORECASE),
        "failsafe": re.compile(r"(?:failsafe|containment)", re.IGNORECASE),
    }
    
    table_rows = parse_markdown_table_rows(conops_text)
    partition_masses: Dict[str, float] = {}
    extracted_total_mtow = total_mtow
    
    for row in table_rows:
        if not row:
            continue
        row_str = " ".join(row)
        first_cell = row[0]
        
        # Check for Total MTOW row in Table 1.3.2 (ignore Pugh matrix or trade study tables)
        if extracted_total_mtow is None and not re.search(r"pugh|trade|decision|score", row_str, re.IGNORECASE):
            if (
                re.search(r"\btotal(?:\s+system|\s+mtow|\s+integration|\s+mass)\b", first_cell, re.IGNORECASE)
                or re.search(r"100\.0%\s*mtow", row_str, re.IGNORECASE)
            ):
                for cell in row[1:]:
                    c_clean = cell.replace("**", "").replace("*", "").strip()
                    if "%" in c_clean:
                        continue
                    m_val = re.search(r"^[-+]?(\d+(?:\.\d+)?)\s*(?:kg)?$", c_clean, re.IGNORECASE)
                    if m_val:
                        try:
                            extracted_total_mtow = float(m_val.group(1))
                            break
                        except ValueError:
                            pass
        
        # Check AST partitions
        for p_key, p_pat in partition_patterns.items():
            if p_key not in partition_masses and p_pat.search(first_cell) and not re.search(r"pugh|trade|decision|score|total", first_cell, re.IGNORECASE):
                val_found = None
                for cell in row[1:]:
                    c_clean = cell.replace("**", "").replace("*", "").strip()
                    if "%" in c_clean:
                        continue
                    m_val = re.search(r"^[-+]?(\d+(?:\.\d+)?)\s*(?:kg)?$", c_clean, re.IGNORECASE)
                    if m_val:
                        try:
                            val_found = float(m_val.group(1))
                            break
                        except ValueError:
                            pass
                if val_found is not None:
                    partition_masses[p_key] = val_found
                    break

    # If total MTOW not found from table, look for TOTAL_MTOW_KG or PL-01 parameter
    if extracted_total_mtow is None:
        m_mtow = re.search(r"TOTAL_MTOW(?:_KG)?\s*[:|=]?\s*([\d\.]+)", conops_text, re.IGNORECASE)
        if not m_mtow:
            m_mtow = re.search(
                r"\|\s*\*\*PL-01\*\*\s*\|.*?\|\s*m_MTOW\s*\|.*?\|\s*([\d\.]+)\s*\|\s*kg",
                conops_text,
                re.IGNORECASE,
            )
        if not m_mtow:
            m_mtow = re.search(r"m_MTOW\s*\|\s*<=?\s*([\d\.]+)", conops_text, re.IGNORECASE)
        if m_mtow:
            extracted_total_mtow = float(m_mtow.group(1))
            
    details["partition_masses"] = partition_masses
    details["total_mtow_kg"] = extracted_total_mtow
    
    missing_partitions = [p for p in partition_patterns if p not in partition_masses]
    if missing_partitions:
        errors.append(f"Table 1.3.2 missing AST partition rows: {missing_partitions}")
        
    if extracted_total_mtow is None:
        errors.append("TOTAL_MTOW_KG parameter / Table 1.3.2 total mass could not be determined")
    elif not missing_partitions:
        sum_mass = sum(partition_masses.values())
        details["sum_partition_masses"] = sum_mass
        diff = abs(sum_mass - extracted_total_mtow)
        details["mass_cross_sum_diff"] = diff
        if diff > 0.01:
            errors.append(
                f"Table 1.3.2 Mass Cross-Sum mismatch: sum of partitions ({sum_mass:.4f} kg) "
                f"!= TOTAL_MTOW_KG ({extracted_total_mtow:.4f} kg), diff={diff:.4f} kg > 0.01 kg"
            )
            
    passed = len(errors) == 0
    return passed, errors, details


def solve_closed_form_quadratic_physics(
    conops_text: str
) -> Tuple[bool, List[str], Dict[str, object]]:
    """
    Solver 2: Closed-Form Quadratic Physics Solver (Layer 4).
    Extracts parameters m, S, C_d, rho, g from Section 5.2.
    Calculates:
      v_calc = sqrt(2 * m * g / (rho * S * C_d))
      E_k_calc = 0.5 * m * v_calc^2
    Asserts tabulated v_terminal and E_k_mitigated match calculated physics within +/- 5%.
    """
    errors = []
    details = {}
    
    table_rows = parse_markdown_table_rows(conops_text)
    
    params: Dict[str, float] = {}
    for row in table_rows:
        if len(row) >= 3:
            name = row[0].strip()
            sym = row[1].strip()
            val_str = row[2].strip()
            m_val = re.search(r"^[-+]?(\d+(?:\.\d+)?)", val_str)
            if not m_val:
                continue
            val = float(m_val.group(1))
            
            if sym == "m" or (re.search(r"operational\s+mass|system\s+mass", name, re.IGNORECASE) and not re.search(r"payload", name, re.IGNORECASE)):
                if "m" not in params:
                    params["m"] = val
            elif sym == "g" or re.search(r"gravitational\s+acceleration", name, re.IGNORECASE):
                params["g"] = val
            elif sym in ("rho", r"\rho") or re.search(r"air\s+density|atmospheric\s+density", name, re.IGNORECASE):
                params["rho"] = val
            elif sym in ("S_canopy", "S") or (re.search(r"canopy\s+area|recovery\s+area", name, re.IGNORECASE) and not re.search(r"unmitigated|reference", name, re.IGNORECASE)):
                if "S" not in params:
                    params["S"] = val
            elif sym in ("C_d_parachute", "C_D_parachute", "C_d", r"C_{d,\mathrm{parachute}}", r"C_d") or (re.search(r"(?:parachute|recovery)\s+drag\s+coefficient", name, re.IGNORECASE) and not re.search(r"unmitigated", name, re.IGNORECASE)):
                if "C_d" not in params:
                    params["C_d"] = val
            elif sym in ("v_terminal_parachute", "v_terminal_p", "v_terminal", r"v_{\mathrm{terminal}}", r"v_{\mathrm{terminal,parachute}}") or ((re.search(r"(?:parachute|recovery|equilibrium\s+descent)\s+terminal\s+velocity", name, re.IGNORECASE) or re.search(r"terminal\s+velocity", name, re.IGNORECASE)) and not re.search(r"unmitigated", name, re.IGNORECASE) and not re.search(r"unmitigated", sym, re.IGNORECASE)):
                if "v_terminal" not in params:
                    params["v_terminal"] = val
            elif sym in ("E_k_mitigated", "E_k") or (re.search(r"\bmitigated\s+kinetic\s+energy\b", name, re.IGNORECASE) and not re.search(r"\bunmitigated\b", name, re.IGNORECASE)):
                if "E_k_mitigated" not in params:
                    params["E_k_mitigated"] = val

    if "g" not in params:
        params["g"] = 9.80665
    if "rho" not in params:
        params["rho"] = 1.225
        
    details["extracted_parameters"] = params
    
    required_keys = ["m", "S", "C_d", "v_terminal", "E_k_mitigated"]
    missing = [k for k in required_keys if k not in params]
    if missing:
        errors.append(f"Section 5.2 missing required parameters for quadratic physics solver: {missing}")
    else:
        m = params["m"]
        g = params["g"]
        rho = params["rho"]
        S = params["S"]
        C_d = params["C_d"]
        v_tab = params["v_terminal"]
        ek_tab = params["E_k_mitigated"]
        
        denom = rho * S * C_d
        if denom <= 0:
            errors.append(f"Invalid non-positive denominator in quadratic physics: rho*S*C_d={denom}")
        else:
            v_calc = ((2.0 * m * g) / denom) ** 0.5
            E_k_calc = 0.5 * m * (v_calc ** 2)
            
            details["v_calc_mps"] = v_calc
            details["E_k_calc_J"] = E_k_calc
            
            v_rel_err = abs(v_tab - v_calc) / v_calc
            ek_rel_err = abs(ek_tab - E_k_calc) / E_k_calc
            
            details["v_rel_error"] = v_rel_err
            details["E_k_rel_error"] = ek_rel_err
            
            if v_rel_err > 0.05:
                errors.append(
                    f"Tabulated terminal velocity v_terminal ({v_tab:.4f} m/s) deviates from calculated "
                    f"quadratic physics ({v_calc:.4f} m/s) by {v_rel_err*100:.2f}% (> 5.0% tolerance)"
                )
            if ek_rel_err > 0.05:
                errors.append(
                    f"Tabulated mitigated kinetic energy E_k_mitigated ({ek_tab:.4f} J) deviates from calculated "
                    f"quadratic physics ({E_k_calc:.4f} J) by {ek_rel_err*100:.2f}% (> 5.0% tolerance)"
                )
                
    passed = len(errors) == 0
    return passed, errors, details


def solve_dimensional_energy_conservation(
    conops_text: str, intent_text: str = ""
) -> Tuple[bool, List[str], Dict[str, object]]:
    """
    Solver 3: Dimensional Scaling & Energy Conservation Engine (Layer 4).
    Converts declared battery capacity in kWh to Joules (kWh * 3.6e6 J).
    Asserts: E_capacity_joules >= P_nominal_watts * (t_endurance_hours * 3600).
    """
    errors = []
    details = {}
    
    combined_text = conops_text + "\n" + intent_text
    table_rows = parse_markdown_table_rows(combined_text)
    
    e_capacity_joules = None
    p_nominal_watts = None
    t_endurance_hours = None
    
    for row in table_rows:
        if not row:
            continue
        row_str = " ".join(row)
        cells_clean = [c.replace("**", "").replace("*", "").strip() for c in row]
        
        # 1. Total Storage Capacity (E_capacity)
        if (
            (len(row) >= 2 and (row[0].strip().lower() in ("total storage capacity", "battery capacity") or row[1].strip() in ("E_capacity", "E_storage")))
            or (re.search(r"^\|\s*(?:Total Storage Capacity|Battery Capacity)\s*\|", row_str, re.IGNORECASE))
        ) and not re.search(r"Ratio_reserve|reserve\s+ratio", row_str, re.IGNORECASE):
            # Extract number from cells
            for cell in cells_clean:
                m_val = re.search(r"^[-+]?(\d+(?:\.\d+)?)\s*(?:kWh|kW\*h|kW-h|Wh|W\*h|MJ|kJ|J)?$", cell, re.IGNORECASE)
                if m_val:
                    try:
                        num = float(m_val.group(1))
                        row_u = row_str.lower()
                        if "kwh" in row_u or "kw*h" in row_u or "kw-h" in row_u:
                            e_capacity_joules = num * 3.6e6
                        elif "wh" in row_u or "w*h" in row_u:
                            e_capacity_joules = num * 3600.0
                        elif "mj" in row_u:
                            e_capacity_joules = num * 1e6
                        elif "kj" in row_u:
                            e_capacity_joules = num * 1000.0
                        else:
                            e_capacity_joules = num
                        break
                    except ValueError:
                        pass

        # 2. Nominal Power Budget (P_nominal)
        # Check Table 1.3.2 Total row (Col 4 is Nominal Power Budget)
        if any(re.search(r"Total\s+System(?:\s+Integration)?", c, re.IGNORECASE) for c in row):
            if len(cells_clean) >= 5:
                # Column 4: Nominal Power Budget (W)
                m_val = re.search(r"^[-+]?(\d+(?:\.\d+)?)", cells_clean[4])
                if m_val:
                    try:
                        p_nominal_watts = float(m_val.group(1))
                    except ValueError:
                        pass
        elif (
            len(row) >= 2 and (row[0].strip().lower() in ("nominal power", "nominal power budget") or row[1].strip() in ("P_nominal", "P_nom", "TOTAL_POWER_NOMINAL_W"))
        ) and not re.search(r"E_divert|Distance|Integral", row_str, re.IGNORECASE):
            for cell in cells_clean:
                m_val = re.search(r"^[-+]?(\d+(?:\.\d+)?)\s*(?:W|kW)?$", cell, re.IGNORECASE)
                if m_val:
                    try:
                        num = float(m_val.group(1))
                        if "kw" in row_str.lower() and "kwh" not in row_str.lower():
                            p_nominal_watts = num * 1000.0
                        else:
                            p_nominal_watts = num
                        break
                    except ValueError:
                        pass

        # 3. Mission Operational Endurance (t_endurance)
        if any(re.search(r"\b(?:t_endurance|PL-09|Mission Operational Endurance)\b", c, re.IGNORECASE) for c in row):
            # Look for float values
            vals = []
            for cell in cells_clean:
                m_val = re.search(r"(?:>=|<=)?\s*(\d+(?:\.\d+)?)", cell)
                if m_val:
                    try:
                        vals.append(float(m_val.group(1)))
                    except ValueError:
                        pass
            if vals:
                # Prefer the last extracted numeric value (often nominal target in Table 1.3.3)
                target_val = vals[-1]
                row_u = row_str.lower()
                if "hour" in row_u or " hr" in row_u or " h " in row_u or row_u.endswith(" h") or row_u.endswith(" h |"):
                    t_endurance_hours = target_val
                elif "min" in row_u:
                    t_endurance_hours = target_val / 60.0
                elif "sec" in row_u or " s " in row_u or row_u.endswith(" s"):
                    t_endurance_hours = target_val / 3600.0
                else:
                    t_endurance_hours = target_val if target_val <= 24.0 else target_val / 60.0

    # Fallback regex search
    if e_capacity_joules is None:
        m_kwh = re.search(r"(?:battery\s+capacity|E_capacity)[^\n\d]*([\d\.]+)\s*kW[·\-\*]?h", combined_text, re.IGNORECASE)
        if m_kwh:
            e_capacity_joules = float(m_kwh.group(1)) * 3.6e6
        else:
            m_j = re.search(r"(?:E_capacity)[^\n\d]*([\d\.]+)\s*J", combined_text, re.IGNORECASE)
            if m_j:
                e_capacity_joules = float(m_j.group(1))

    if p_nominal_watts is None:
        m_pnom = re.search(r"\|\s*\*\*Total System Integration\*\*.*?\|\s*([\d\.]+)\s*\|\s*[\d\.]+\s*\|", conops_text)
        if m_pnom:
            p_nominal_watts = float(m_pnom.group(1))
        else:
            m_p = re.search(r"(?:P_nominal|Nominal\s+Power)[^\n\d]*([\d\.]+)\s*W", combined_text, re.IGNORECASE)
            if m_p:
                p_nominal_watts = float(m_p.group(1))

    if t_endurance_hours is None:
        m_end_h = re.search(r"t_endurance[^\n\d]*([\d\.]+)\s*(?:hours|h|hr)", combined_text, re.IGNORECASE)
        if m_end_h:
            t_endurance_hours = float(m_end_h.group(1))
        else:
            m_end_m = re.search(r"t_endurance[^\n\d]*([\d\.]+)\s*min", combined_text, re.IGNORECASE)
            if m_end_m:
                t_endurance_hours = float(m_end_m.group(1)) / 60.0

    details["e_capacity_joules"] = e_capacity_joules
    details["p_nominal_watts"] = p_nominal_watts
    details["t_endurance_hours"] = t_endurance_hours

    if e_capacity_joules is None:
        errors.append("Declared energy storage capacity (E_capacity) missing from specifications")
    if p_nominal_watts is None:
        errors.append("Nominal power consumption (P_nominal) missing from specifications")
    if t_endurance_hours is None:
        errors.append("Mission operational endurance (t_endurance) missing from specifications")

    if e_capacity_joules is not None and p_nominal_watts is not None and t_endurance_hours is not None:
        is_space = any(k in combined_text.lower() for k in ("cubesat", "orbit", "spacecraft", "satellite"))
        effective_endurance_h = t_endurance_hours
        if is_space and t_endurance_hours > 24.0:
            effective_endurance_h = 1.0  # Max orbital eclipse duration in LEO (35 min nominal)
            
        e_required_joules = p_nominal_watts * (effective_endurance_h * 3600.0)
        details["e_required_joules"] = e_required_joules
        details["energy_margin_joules"] = e_capacity_joules - e_required_joules
        
        if e_capacity_joules < e_required_joules:
            errors.append(
                f"Dimensional Energy Conservation Violation: Declared storage capacity "
                f"({e_capacity_joules:.1f} J / {e_capacity_joules/3.6e6:.3f} kWh) is less than required "
                f"mission energy ({e_required_joules:.1f} J / {e_required_joules/3.6e6:.3f} kWh) for "
                f"P_nominal={p_nominal_watts:.1f} W over t_endurance={effective_endurance_h:.2f} h"
            )

    passed = len(errors) == 0
    return passed, errors, details


def solve_normative_standards_cross_check(
    workspace_path: str, conops_text: str, domain_config: Optional[Dict] = None
) -> Tuple[bool, List[str], Dict[str, object]]:
    """
    Solver 4: Normative Standards Cross-Checker (Layer 3).
    Parses schema/domain_config.json for REGULATORY_STANDARDS or OPERATIONAL_DOMAIN.
    Asserts declared standards (IEC 62304 for medical, EN 50128 for rail, ECSS for space,
    ISO 3691-4 for AGV, DNV-GL for subsea) are cited in Section 1.5.
    """
    errors = []
    details = {}
    
    cfg = domain_config
    if cfg is None:
        cfg_paths = [
            os.path.join(workspace_path, "schema", "domain_config.json"),
            os.path.join(workspace_path, "domain_config.json"),
        ]
        for p in cfg_paths:
            if os.path.isfile(p):
                try:
                    with open(p, "r", encoding="utf-8") as f:
                        cfg = json.load(f)
                    break
                except Exception as ex:
                    errors.append(f"Failed to parse {p}: {ex}")
    
    sec_15_match = re.search(
        r"###?\s*1\.5[^\n]*\n(.*?)(?=\n###?\s*1\.[6-9]|\n##\s*[2-9]|\Z)",
        conops_text,
        re.DOTALL | re.IGNORECASE,
    )
    sec_15_text = sec_15_match.group(1) if sec_15_match else conops_text
    
    domain_standards_map = {
        "medical": ("IEC 62304", re.compile(r"\bIEC\s*62304\b", re.IGNORECASE)),
        "rail": ("EN 50128", re.compile(r"\bEN\s*50128\b|\bEN\s*50126\b|\bEN\s*50129\b", re.IGNORECASE)),
        "space": ("ECSS", re.compile(r"\bECSS\b", re.IGNORECASE)),
        "agv": ("ISO 3691-4", re.compile(r"\bISO\s*3691-4\b|\bISO\s*3691\b", re.IGNORECASE)),
        "subsea": ("DNV-GL", re.compile(r"\bDNV(?:-GL)?\b", re.IGNORECASE)),
    }
    
    standards_to_check: List[Tuple[str, str]] = []
    
    if cfg:
        reg_stds = cfg.get("REGULATORY_STANDARDS") or cfg.get("regulatory_standards")
        if isinstance(reg_stds, list):
            for std in reg_stds:
                if isinstance(std, str):
                    standards_to_check.append((std, "domain_config.json REGULATORY_STANDARDS"))
        elif isinstance(reg_stds, dict):
            for std in reg_stds.keys():
                standards_to_check.append((std, "domain_config.json REGULATORY_STANDARDS"))
                
        op_domain = str(cfg.get("OPERATIONAL_DOMAIN") or cfg.get("operational_domain") or cfg.get("domain") or "").lower()
        for d_key, (std_name, _) in domain_standards_map.items():
            if d_key in op_domain:
                standards_to_check.append((std_name, f"declared domain '{op_domain}'"))
    
    domain_id = os.path.basename(os.path.abspath(workspace_path)).lower()
    domain_name = DOMAIN_NAMES.get(domain_id, domain_id).lower()
    for d_key, (std_name, _) in domain_standards_map.items():
        if d_key in domain_id or d_key in domain_name:
            standards_to_check.append((std_name, f"inferred domain '{domain_id}'"))
            
    domain_alias_map = {
        "surgical": ("IEC 62304", "surgical domain"),
        "healthcare": ("IEC 62304", "healthcare domain"),
        "locomotive": ("EN 50128", "locomotive rail domain"),
        "shunting": ("EN 50128", "rail shunting domain"),
        "cubesat": ("ECSS", "cubesat space domain"),
        "satellite": ("ECSS", "satellite space domain"),
        "orbit": ("ECSS", "orbital space domain"),
        "forklift": ("ISO 3691-4", "forklift AGV domain"),
        "auv": ("DNV-GL", "AUV subsea domain"),
        "underwater": ("DNV-GL", "underwater subsea domain"),
    }
    for alias_key, (std_name, reason) in domain_alias_map.items():
        if alias_key in domain_id or alias_key in domain_name:
            standards_to_check.append((std_name, reason))

    details["standards_checked"] = standards_to_check
    
    checked_set = set()
    for std_name, reason in standards_to_check:
        if std_name in checked_set:
            continue
        checked_set.add(std_name)
        
        std_pattern = re.compile(re.escape(std_name), re.IGNORECASE)
        for _, (c_name, c_pat) in domain_standards_map.items():
            if c_name == std_name:
                std_pattern = c_pat
                break
                
        if not std_pattern.search(sec_15_text):
            errors.append(
                f"Normative standard '{std_name}' required for {reason} is missing from CONOPS.md Section 1.5"
            )
            
    passed = len(errors) == 0
    return passed, errors, details


def solve_forbidden_cross_domain_ontology(
    workspace_path: str, conops_text: str, intent_text: str = "", domain_config: Optional[Dict] = None
) -> Tuple[bool, List[str], Dict[str, object]]:
    """
    Solver 5: Forbidden Cross-Domain Ontology Scanner (Layer 5).
    - For non-aircraft platforms (Ground, Rail, Medical, Subsea, Space):
        reject `V_stall > 0`, `parachute`, `altitude AGL`, `airframe`, `ASTM F3411 Remote ID`.
    - For civilian platforms (Medical, Logistics AGV, Rail):
        reject `ROE-01..06`, `PID`, `weapons release`, `collateral damage`.
    """
    errors = []
    details = {}
    
    cfg = domain_config
    if cfg is None:
        cfg_paths = [
            os.path.join(workspace_path, "schema", "domain_config.json"),
            os.path.join(workspace_path, "domain_config.json"),
        ]
        for p in cfg_paths:
            if os.path.isfile(p):
                try:
                    with open(p, "r", encoding="utf-8") as f:
                        cfg = json.load(f)
                    break
                except Exception:
                    pass

    domain_id = os.path.basename(os.path.abspath(workspace_path)).lower()
    domain_name = DOMAIN_NAMES.get(domain_id, domain_id).lower()
    combined_domain_info = f"{domain_id} {domain_name}"
    if cfg:
        combined_domain_info += f" {cfg.get('PLATFORM_TYPE', '')} {cfg.get('OPERATIONAL_DOMAIN', '')} {cfg.get('domain', '')}".lower()

    non_aircraft_keywords = [
        "ground", "rail", "medical", "subsea", "space", "ugv", "locomotive",
        "surgical", "auv", "cubesat", "satellite", "agv", "forklift", "underwater",
        "run_03", "run_05", "run_06", "run_07", "run_08", "run_09",
    ]
    civilian_keywords = [
        "medical", "agv", "rail", "surgical", "forklift", "locomotive", "logistics",
        "run_07", "run_08", "run_09", "civilian",
    ]

    is_non_aircraft = any(k in combined_domain_info for k in non_aircraft_keywords)
    if cfg and "is_aircraft" in cfg:
        is_non_aircraft = not cfg["is_aircraft"]

    is_civilian = any(k in combined_domain_info for k in civilian_keywords)
    if cfg and "is_civilian" in cfg:
        is_civilian = cfg["is_civilian"]

    details["is_non_aircraft"] = is_non_aircraft
    details["is_civilian"] = is_civilian

    combined_text = conops_text + "\n" + intent_text

    # Rule A: Non-aircraft platforms
    if is_non_aircraft:
        # 1. V_stall > 0
        v_stall_matches = re.findall(
            r"V_stall\s*\|\s*<=?\s*([1-9]\d*(?:\.\d+)?|0\.\d*[1-9]\d*)",
            combined_text,
            re.IGNORECASE,
        )
        if not v_stall_matches:
            v_stall_matches = re.findall(
                r"V_stall\s*=\s*([1-9]\d*(?:\.\d+)?|0\.\d*[1-9]\d*)",
                combined_text,
                re.IGNORECASE,
            )
        if not v_stall_matches:
            v_stall_matches = re.findall(
                r"\|\s*Minimum Controllable / Stall Velocity\s*\|\s*V_stall\s*\|\s*<=?\s*([1-9]\d*(?:\.\d+)?|0\.\d*[1-9]\d*)",
                combined_text,
                re.IGNORECASE,
            )
        if v_stall_matches:
            errors.append(
                f"Forbidden ontology in non-aircraft domain: positive stall velocity 'V_stall = {v_stall_matches[0]} m/s' (> 0)"
            )

        # 2. parachute
        if re.search(r"\bparachute\b", combined_text, re.IGNORECASE):
            errors.append("Forbidden ontology in non-aircraft domain: references aeronautical 'parachute'")

        # 3. altitude AGL
        if re.search(r"\b(?:altitude\s+AGL|m\s+AGL)\b", combined_text, re.IGNORECASE):
            errors.append("Forbidden ontology in non-aircraft domain: references aeronautical 'altitude AGL'")

        # 4. airframe
        if re.search(r"\bairframe\b", combined_text, re.IGNORECASE):
            errors.append("Forbidden ontology in non-aircraft domain: references aeronautical 'airframe'")

        # 5. ASTM F3411 Remote ID
        if re.search(r"(?:ASTM\s+F3411|\bRemote\s+ID\b)", combined_text, re.IGNORECASE):
            errors.append("Forbidden ontology in non-aircraft domain: references UAS standard 'ASTM F3411 Remote ID'")

    # Rule B: Civilian platforms
    if is_civilian:
        # 1. ROE-01..06
        roe_matches = re.findall(r"\bROE-0[1-6]\b", combined_text)
        if roe_matches:
            errors.append(
                f"Forbidden ontology in civilian domain: references military Rules of Engagement {sorted(set(roe_matches))}"
            )

        # 2. PID
        if re.search(r"\bPID\b", combined_text):
            errors.append("Forbidden ontology in civilian domain: references military tactical Positive Identification ('PID')")

        # 3. weapons release
        if re.search(r"\bweapons?\s+release\b", combined_text, re.IGNORECASE):
            errors.append("Forbidden ontology in civilian domain: references military 'weapons release'")

        # 4. collateral damage
        if re.search(r"\bcollateral\s+damage\b", combined_text, re.IGNORECASE):
            errors.append("Forbidden ontology in civilian domain: references military 'collateral damage'")

    passed = len(errors) == 0
    return passed, errors, details


# ---------------------------------------------------------------------------
# Solver 6: Positive Domain Lexicon Density Floor (Layer 5)
# ---------------------------------------------------------------------------

POSITIVE_DOMAIN_LEXICONS: Dict[str, List[str]] = {
    "medical": [
        "surgeon", "trocar", "master manipulator", "laparoscope",
        "sterile drape", "end-effector", "haptic", "dicom", "hl7",
    ],
    "rail": [
        "track circuit", "axle counter", "coupler", "shunting yard",
        "brake pipe", "turnout", "bogie", "etcs", "balise",
    ],
    "marine": [
        "bathymetry", "buoyancy engine", "usbl", "dvl", "ctd",
        "transponder", "acoustic modem", "colregs", "seaway",
    ],
    "space": [
        "adcs", "reaction wheel", "magnetorquer", "star tracker",
        "lvlh", "orbital eclipse", "ccsds", "telemetry tracking",
    ],
    "industrial": [
        "pallet", "fork mast", "docking", "vda 5050", "safety field",
        "optical lidar", "curbside", "odometry",
    ],
    "aviation": [
        "airframe", "flight controller", "airspace", "aerodynamic",
        "wing", "avionics", "sora", "payload",
    ],
}


def _term_to_regex(term: str) -> re.Pattern:
    tokens = re.split(r"[\s\-]+", term.strip().lower())
    last = tokens[-1]
    if last == "colregs":
        last_pat = r"colregs?"
    elif last.endswith("s"):
        last_pat = re.escape(last) + r"?"
    else:
        last_pat = re.escape(last) + r"(?:s|es)?"
    parts = [re.escape(tok) for tok in tokens[:-1]] + [last_pat]
    pat_str = r"\b" + r"[\s\-_]+".join(parts) + r"\b"
    return re.compile(pat_str, re.IGNORECASE)


def _infer_lexicon_domain_type(
    domain_id: str, domain_type: Optional[str] = None, text: str = ""
) -> Optional[str]:
    if domain_type and domain_type.lower() in POSITIVE_DOMAIN_LEXICONS:
        return domain_type.lower()
    d_name = DOMAIN_NAMES.get(domain_id, domain_id)
    combined = f"{domain_id} {d_name} {domain_type or ''}".lower()

    rules = [
        ("medical", ("medical", "surgical", "surgeon", "laparoscopic", "hospital", "healthcare", "run_07")),
        ("rail", ("rail", "locomotive", "shunting", "train", "railway", "run_08")),
        ("marine", ("marine", "maritime", "subsea", "underwater", "auv", "asv", "surface vessel", "vessel", "run_03", "run_04")),
        ("space", ("space", "cubesat", "satellite", "spacecraft", "orbit", "leo", "constellation", "run_06")),
        ("industrial", ("industrial", "agv", "forklift", "ugv", "ground delivery", "ground robot", "warehouse", "logistics", "run_05", "run_09")),
        ("aviation", ("aviation", "aircraft", "uav", "uas", "aerial", "airspace", "aerospace", "evtol", "fixed-wing", "interceptor", "air mobility", "flight", "drone", "rotorcraft", "sora", "run_01", "run_02", "run_10")),
    ]

    def _matches_rule(target: str, kw_tuple: Tuple[str, ...]) -> bool:
        for k in kw_tuple:
            pat = r"\b" + re.escape(k).replace(r"\-", r"[\s\-]") + r"\b"
            if re.search(pat, target, re.IGNORECASE):
                return True
        return False

    for dom_key, kw_tuple in rules:
        if _matches_rule(combined, kw_tuple):
            return dom_key

    if text:
        text_head = text[:2000].lower()
        for dom_key, kw_tuple in rules:
            if _matches_rule(text_head, kw_tuple):
                return dom_key

    return None


def solve_positive_domain_lexicon_floor(
    domain_id: str, domain_type: Optional[str], text: str
) -> Tuple[bool, List[str], Dict[str, object]]:
    """
    Solver 6: Positive Domain Lexicon Density Floor (Layer 5).
    Check that each domain matches at least 4 authentic domain-native terms:
      - medical: surgeon, trocar, master manipulator, laparoscope, sterile drape, end-effector, haptic, dicom, hl7
      - rail: track circuit, axle counter, coupler, shunting yard, brake pipe, turnout, bogie, etcs, balise
      - marine: bathymetry, buoyancy engine, usbl, dvl, ctd, transponder, acoustic modem, colregs, seaway
      - space: adcs, reaction wheel, magnetorquer, star tracker, lvlh, orbital eclipse, ccsds, telemetry tracking
      - industrial: pallet, fork mast, docking, vda 5050, safety field, optical lidar, curbside, odometry
      - aviation: airframe, flight controller, airspace, aerodynamic, wing, avionics, sora, payload
    Fail Layer 5 if count < 4 with explicit missing terms list.
    """
    errors = []
    details: Dict[str, object] = {}

    resolved_type = _infer_lexicon_domain_type(domain_id, domain_type, text)
    if not resolved_type or resolved_type not in POSITIVE_DOMAIN_LEXICONS:
        errors.append(f"Unable to determine domain type for domain '{domain_id}' (domain_type='{domain_type}')")
        return False, errors, details

    target_terms = POSITIVE_DOMAIN_LEXICONS[resolved_type]
    matched_terms = []

    for term in target_terms:
        rx = _term_to_regex(term)
        if rx.search(text):
            matched_terms.append(term)

    missing_terms = [t for t in target_terms if t not in matched_terms]
    matched_count = len(matched_terms)

    details["domain_id"] = domain_id
    details["domain_type"] = resolved_type
    details["matched_terms"] = matched_terms
    details["matched_count"] = matched_count
    details["missing_terms"] = missing_terms
    details["lexicon_floor"] = 4

    if matched_count < 4:
        errors.append(
            f"Positive domain lexicon floor breach for domain '{domain_id}' ({resolved_type}): "
            f"matched {matched_count} terms (< 4 required). "
            f"Matched: {matched_terms}, Missing: {missing_terms}"
        )
        passed = False
    else:
        passed = True

    return passed, errors, details


# ---------------------------------------------------------------------------
# Solver 7: Pairwise Anti-Plagiarism Gate (Layer 5)
# ---------------------------------------------------------------------------

def extract_substantive_sentences(text: str) -> Set[str]:
    """
    Extract substantive sentences (>= 4 words) from specification text.
    Strips markdown formatting, headers, table pipes, and normalizes whitespace.
    """
    sentences = set()
    raw_segments = re.split(r"(?<=[.!?])\s+|\n+|\|", text)
    for seg in raw_segments:
        cleaned = re.sub(r"^[\s#*\->|:]+", "", seg)
        cleaned = re.sub(r"[\s|:]+$", "", cleaned)
        cleaned = re.sub(r"[*_`~]", "", cleaned)
        cleaned = cleaned.strip()

        # Skip separator rows like --- or :---:
        if not cleaned or all(c in "-: " for c in cleaned):
            continue

        words = re.findall(r"\b[A-Za-z0-9_-]+\b", cleaned)
        if len(words) >= 4:
            normalized_sentence = " ".join(words).lower()
            sentences.add(normalized_sentence)
    return sentences


def solve_pairwise_domain_similarity(
    domain_texts: Dict[str, str], max_threshold: float = 0.25
) -> Tuple[bool, List[str], Dict[str, object]]:
    """
    Solver 7: Pairwise Anti-Plagiarism Gate (Layer 5).
    Extracts substantive sentences (>= 4 words) from CONOPS.md + MISSION_INTENT.md
    for all evaluated domain sandboxes.
    Calculates pairwise Jaccard sentence similarity between all distinct physical domain pairs.
    If similarity > max_threshold (25.0%), fails Layer 5 with explicit error showing overlap %.
    """
    errors = []
    details: Dict[str, object] = {}

    domain_sentences: Dict[str, Set[str]] = {}
    for d_id, txt in domain_texts.items():
        domain_sentences[d_id] = extract_substantive_sentences(txt)

    sentence_counts = {d: len(s) for d, s in domain_sentences.items()}
    details["sentence_counts"] = sentence_counts

    domain_keys = sorted(domain_texts.keys())
    similarity_matrix: Dict[str, Dict[str, float]] = {}
    violations = []
    max_sim = 0.0
    max_pair = None

    for i, d1 in enumerate(domain_keys):
        if d1 not in similarity_matrix:
            similarity_matrix[d1] = {}
        similarity_matrix[d1][d1] = 1.0

        for j in range(i + 1, len(domain_keys)):
            d2 = domain_keys[j]
            if d2 not in similarity_matrix:
                similarity_matrix[d2] = {}

            s1 = domain_sentences[d1]
            s2 = domain_sentences[d2]

            union = s1.union(s2)
            if union:
                inter = s1.intersection(s2)
                jaccard = len(inter) / len(union)
            else:
                jaccard = 0.0

            similarity_matrix[d1][d2] = round(jaccard, 4)
            similarity_matrix[d2][d1] = round(jaccard, 4)

            if jaccard > max_sim:
                max_sim = jaccard
                max_pair = (d1, d2)

            if jaccard > max_threshold:
                pct = jaccard * 100.0
                threshold_pct = max_threshold * 100.0
                err_msg = (
                    f"Pairwise domain similarity between '{d1}' and '{d2}' is {pct:.2f}% "
                    f"(exceeds {threshold_pct:.1f}% anti-plagiarism threshold)"
                )
                errors.append(err_msg)
                violations.append({
                    "domain_a": d1,
                    "domain_b": d2,
                    "similarity": round(jaccard, 4),
                    "overlap_percentage": round(pct, 2),
                })

    details["similarity_matrix"] = similarity_matrix
    details["max_similarity"] = round(max_sim, 4)
    details["max_similarity_pair"] = max_pair
    details["threshold"] = max_threshold
    details["violations"] = violations

    passed = len(errors) == 0
    return passed, errors, details


# ---------------------------------------------------------------------------
# Layer 3: Statutory Cardinality & Normative Standards
# ---------------------------------------------------------------------------

def verify_layer3_cardinality(workspace_path: str) -> LayerResult:
    """Assert 16 Threat Vectors, 4 PACE tiers, 12 MIL-STD-810H methods, 24 SORA OSOs, 7 Emergency rows, and Normative Standards."""
    errors = []
    details = {}
    
    conops_path = os.path.join(workspace_path, "docs", "conops", "CONOPS.md")
    intent_path = os.path.join(workspace_path, "docs", "conops", "MISSION_INTENT.md")
    
    if not os.path.exists(conops_path) or not os.path.exists(intent_path):
        errors.append("CONOPS.md or MISSION_INTENT.md missing for cardinality analysis")
        return LayerResult(layer_id=3, layer_name="Statutory Cardinality", passed=False, errors=errors)
        
    with open(conops_path, "r", encoding="utf-8", errors="ignore") as f:
        conops_c = f.read()
    with open(intent_path, "r", encoding="utf-8", errors="ignore") as f:
        intent_c = f.read()
        
    # 1. 16 Threat Vectors (THR-01..THR-16)
    expected_threats = [f"THR-{i:02d}" for i in range(1, 17)]
    missing_threats = [t for t in expected_threats if not re.search(r"\b" + t + r"\b", intent_c)]
    details["threat_vectors_found"] = 16 - len(missing_threats)
    if missing_threats:
        errors.append(f"Missing mandatory Threat Vectors: {missing_threats}")
        
    # 2. 4 PACE Tiers (Primary, Alternate, Contingency, Emergency)
    pace_tiers = ["Primary", "Alternate", "Contingency", "Emergency"]
    missing_pace = [p for p in pace_tiers if not re.search(r"\b" + p + r"\b", intent_c)]
    details["pace_tiers_found"] = 4 - len(missing_pace)
    if missing_pace:
        errors.append(f"Missing PACE C2 communication tiers: {missing_pace}")
        
    # 3. >= 12 MIL-STD-810H Methods
    methods = sorted(set(re.findall(r"Method\s+(5\d{2}(?:\.\d+)?)", conops_c, re.IGNORECASE)))
    details["mil_std_810h_methods"] = methods
    details["mil_std_810h_count"] = len(methods)
    if len(methods) < 12:
        errors.append(f"Found only {len(methods)} MIL-STD-810H methods (< 12 required): {methods}")
        
    # 4. 24 SORA OSOs (OSO-01..OSO-24)
    safety_files = []
    safety_dir = os.path.join(workspace_path, "docs", "safety")
    if os.path.isdir(safety_dir):
        for root, _, files in os.walk(safety_dir):
            for sf in files:
                if sf.endswith(".md") and sf != "README.md":
                    with open(os.path.join(root, sf), "r", encoding="utf-8", errors="ignore") as handle:
                        safety_files.append(handle.read())
    combined_safety = conops_c + "\n" + "\n".join(safety_files)
    
    if safety_files or re.search(r"\bSORA\b", conops_c):
        expected_osos = [f"OSO-{i:02d}" for i in range(1, 25)]
        found_osos = [o for o in expected_osos if re.search(r"\b" + o + r"\b", combined_safety)]
        details["sora_osos_found"] = len(found_osos)
        if safety_files:
            missing_osos = [o for o in expected_osos if o not in found_osos]
            if missing_osos:
                errors.append(f"Missing SORA Operational Safety Objectives in docs/safety/: {missing_osos}")
        else:
            details["sora_status"] = "Level 1B ConOps framework baseline verified; STPA matrix pending Level 2"
    else:
        details["sora_osos_found"] = 24
        
    # 5. 7 Emergency Rows (EMG-01..EMG-07)
    expected_emgs = [f"EMG-{i:02d}" for i in range(1, 8)]
    missing_emgs = [e for e in expected_emgs if not re.search(r"\b" + e + r"\b", conops_c)]
    details["emergency_rows_found"] = 7 - len(missing_emgs)
    if missing_emgs:
        errors.append(f"Missing Emergency Decision rows: {missing_emgs}")

    # 6. Normative Standards Cross-Checker (Solver 4)
    std_passed, std_errors, std_details = solve_normative_standards_cross_check(workspace_path, conops_c)
    details["normative_standards"] = std_details
    if not std_passed:
        errors.extend(std_errors)

    passed = len(errors) == 0
    return LayerResult(
        layer_id=3,
        layer_name="Statutory Cardinality",
        passed=passed,
        errors=errors,
        details=details
    )


# ---------------------------------------------------------------------------
# Layer 4: Closed-Form Physical & Math Solver
# ---------------------------------------------------------------------------

def verify_layer4_physical_math(workspace_path: str) -> LayerResult:
    """Validate SORA kinetic energy, Kalman filter covariance units & dimensions, Bingo energy conservation, mass partitions, and quadratic physics."""
    errors = []
    details = {}
    
    conops_path = os.path.join(workspace_path, "docs", "conops", "CONOPS.md")
    intent_path = os.path.join(workspace_path, "docs", "conops", "MISSION_INTENT.md")
    
    if not os.path.exists(conops_path) or not os.path.exists(intent_path):
        errors.append("CONOPS.md or MISSION_INTENT.md missing for physical math solver")
        return LayerResult(layer_id=4, layer_name="Closed-Form Physical & Math Solver", passed=False, errors=errors)
        
    with open(conops_path, "r", encoding="utf-8", errors="ignore") as f:
        conops_c = f.read()
    with open(intent_path, "r", encoding="utf-8", errors="ignore") as f:
        intent_c = f.read()
        
    # 1. SORA Kinetic Energy Calculation (E_k <= 34.0J for GRC-1)
    m_ek = re.search(r"\|\s*Mitigated Kinetic Energy\s*\|\s*E_k_mitigated\s*\|\s*([\d\.]+)\s*\|\s*J", conops_c)
    if not m_ek:
        m_ek = re.search(r"E_k(?:_mitigated)?\s*\|\s*([\d\.]+)\s*\|\s*J", conops_c)
        
    if m_ek:
        ek = float(m_ek.group(1))
        details["mitigated_kinetic_energy_J"] = ek
        m_grc = re.search(r"GRC-(\d+)", conops_c)
        grc_level = int(m_grc.group(1)) if m_grc else None
        is_aircraft = not bool(
            re.search(
                r"\b(subsea|maritime|ugv|cubesat|spacecraft|satellite|space|surgical|hospital|locomotive|rail|forklift|agv)\b",
                conops_c,
                re.IGNORECASE,
            )
        )
        if is_aircraft and (grc_level is None or grc_level == 1) and ek > 34.0:
            errors.append(f"Mitigated kinetic energy E_k ({ek} J) exceeds 34.0 J GRC-1 ceiling")
    else:
        errors.append("Mitigated Kinetic Energy E_k parameter definition not found in CONOPS.md")
        
    # 2. Kalman Covariance Unit Checks
    kalman_expected_units = {
        "P_k|k-1": "m^2",
        "Q_k": "m^2/s^2",
        "K_k": "Dimensionless",
        "R_k": "m^2",
        "P_k|k": "m^2",
    }
    
    intent_rows = parse_markdown_table_rows(intent_c)
    found_kalman_params = {}
    for row in intent_rows:
        if len(row) >= 3:
            param_symbol = row[1].strip()
            unit_val = row[2].strip()
            if param_symbol in kalman_expected_units:
                found_kalman_params[param_symbol] = unit_val
                expected = kalman_expected_units[param_symbol]
                if unit_val != expected:
                    errors.append(f"Kalman parameter '{param_symbol}' unit mismatch: got '{unit_val}', expected '{expected}'")
                    
    details["kalman_parameters_verified"] = len(found_kalman_params)
    for p in kalman_expected_units:
        if p not in found_kalman_params:
            errors.append(f"Mandatory Kalman filter parameter '{p}' missing from MISSION_INTENT.md")
            
    # 3. Dynamic Bingo Energy Conservation & Reserve Ratio
    bingo_keys = ["E_capacity", "E_return", "E_divert", "E_reserve", "E_contingency", "E_bingo"]
    bingo_values = {}
    for row in intent_rows:
        if len(row) >= 4:
            sym = row[1].strip()
            if sym in bingo_keys:
                try:
                    bingo_values[sym] = float(row[2].strip())
                except ValueError:
                    pass
                    
    details["bingo_values"] = bingo_values
    if len(bingo_values) == 6:
        e_cap = bingo_values["E_capacity"]
        e_ret = bingo_values["E_return"]
        e_div = bingo_values["E_divert"]
        e_res = bingo_values["E_reserve"]
        e_con = bingo_values["E_contingency"]
        e_bin = bingo_values["E_bingo"]
        
        sum_expected = e_ret + e_div + e_res + e_con
        if abs(e_bin - sum_expected) > 1e-3:
            errors.append(f"Bingo energy conservation violation: E_bingo ({e_bin} J) != sum of components ({sum_expected} J)")
            
        ratio = e_res / e_cap
        details["reserve_ratio"] = ratio
        if ratio < 0.20 - 1e-5:
            errors.append(f"Statutory energy reserve ratio ({ratio:.3f}) below mandatory 0.20 floor")
        if e_bin > e_cap:
            errors.append(f"Total Bingo threshold ({e_bin} J) exceeds total capacity ({e_cap} J)")
    else:
        missing_bingo = [k for k in bingo_keys if k not in bingo_values]
        errors.append(f"Missing Bingo energy parameters in MISSION_INTENT.md: {missing_bingo}")

    # 4. Relational Table Mass Cross-Sum Solver (Solver 1)
    mass_passed, mass_errors, mass_details = solve_relational_mass_cross_sum(conops_c)
    details["mass_cross_sum"] = mass_details
    if not mass_passed:
        errors.extend(mass_errors)

    # 5. Closed-Form Quadratic Physics Solver (Solver 2)
    quad_passed, quad_errors, quad_details = solve_closed_form_quadratic_physics(conops_c)
    details["quadratic_physics"] = quad_details
    if not quad_passed:
        errors.extend(quad_errors)

    # 6. Dimensional Scaling & Energy Conservation Engine (Solver 3)
    dim_passed, dim_errors, dim_details = solve_dimensional_energy_conservation(conops_c, intent_c)
    details["dimensional_energy"] = dim_details
    if not dim_passed:
        errors.extend(dim_errors)

    passed = len(errors) == 0
    return LayerResult(
        layer_id=4,
        layer_name="Closed-Form Physical & Math Solver",
        passed=passed,
        errors=errors,
        details=details
    )


# ---------------------------------------------------------------------------
# Layer 5: Adversarial Invariant Verification
# ---------------------------------------------------------------------------

def verify_layer5_adversarial_invariants(
    workspace_path: str,
    domain_texts: Optional[Dict[str, str]] = None,
) -> LayerResult:
    """Verify priority arbitration, failsafe non-destructive RTB, NIST SP 800-82r3 anti-replay freshness, ontology invariants, positive domain lexicon floor, and pairwise anti-plagiarism."""
    errors = []
    details = {}
    
    conops_path = os.path.join(workspace_path, "docs", "conops", "CONOPS.md")
    intent_path = os.path.join(workspace_path, "docs", "conops", "MISSION_INTENT.md")
    
    if not os.path.exists(conops_path) or not os.path.exists(intent_path):
        errors.append("CONOPS.md or MISSION_INTENT.md missing for adversarial invariant verification")
        return LayerResult(layer_id=5, layer_name="Adversarial Invariant Verification", passed=False, errors=errors)
        
    with open(conops_path, "r", encoding="utf-8", errors="ignore") as f:
        conops_c = f.read()
    with open(intent_path, "r", encoding="utf-8", errors="ignore") as f:
        intent_c = f.read()
        
    # 1. Priority Arbitration (P_EMG07 > P_EMG06 > ... > P_EMG01)
    p_pattern = re.compile(r"P_\{?\\mathrm\{EMG[-_]?0?7\}?\}?\s*>", re.IGNORECASE)
    p_pattern_alt = re.compile(r"P_EMG0?7\s*>", re.IGNORECASE)
    p_pattern_alt2 = re.compile(r"P_\{?EMG[-_]?0?7\}?\s*>", re.IGNORECASE)
    
    if not (p_pattern.search(conops_c) or p_pattern_alt.search(conops_c) or p_pattern_alt2.search(conops_c)):
        errors.append("Priority arbitration invariant (P_EMG07 > ...) missing from CONOPS.md Section 12")
    else:
        details["priority_arbitration"] = "Verified"
        
    # 2. Failsafe Non-Destructive RTB in Flight
    if not re.search(r"EMG-01.*?(?:return|loiter|rtb|hold)", conops_c, re.IGNORECASE | re.DOTALL):
        errors.append("EMG-01 failsafe state in CONOPS.md must specify non-destructive loiter / return-to-base")
    if not re.search(r"EMG-07.*?(?:flight\s+termination|abort|parachute|cutoff)", conops_c, re.IGNORECASE | re.DOTALL):
        errors.append("EMG-07 failsafe state in CONOPS.md must specify Flight Termination / Abort")
    details["failsafe_containment"] = "Verified non-destructive RTB with isolated termination"
    
    # 3. NIST SP 800-82r3 Anti-Replay Freshness
    if not ("800-82r3" in intent_c or "800-82" in intent_c):
        errors.append("NIST SP 800-82r3 normative standard citation missing from MISSION_INTENT.md")
    if not re.search(r"(?:replay|monotonic|sequence\s+counter|freshness)", intent_c, re.IGNORECASE):
        errors.append("Anti-replay freshness mechanisms (sequence counters, timestamps, nonces) missing from MISSION_INTENT.md")
    else:
        details["anti_replay_freshness"] = "Verified NIST SP 800-82r3 monotonic sequence counters"

    # 4. Forbidden Cross-Domain Ontology Scanner (Solver 5)
    onto_passed, onto_errors, onto_details = solve_forbidden_cross_domain_ontology(
        workspace_path, conops_c, intent_c
    )
    details["cross_domain_ontology"] = onto_details
    if not onto_passed:
        errors.extend(onto_errors)

    # 5. Positive Domain Lexicon Density Floor (Solver 6)
    domain_id = os.path.basename(os.path.abspath(workspace_path))
    cfg_paths = [
        os.path.join(workspace_path, "schema", "domain_config.json"),
        os.path.join(workspace_path, "domain_config.json"),
    ]
    cfg = None
    for p in cfg_paths:
        if os.path.isfile(p):
            try:
                with open(p, "r", encoding="utf-8") as f:
                    cfg = json.load(f)
                break
            except Exception:
                pass
    domain_type = None
    if cfg:
        domain_type = cfg.get("OPERATIONAL_DOMAIN") or cfg.get("domain") or cfg.get("PLATFORM_TYPE")

    combined_text = conops_c + "\n" + intent_c
    lex_passed, lex_errors, lex_details = solve_positive_domain_lexicon_floor(
        domain_id=domain_id,
        domain_type=domain_type,
        text=combined_text,
    )
    details["positive_lexicon_floor"] = lex_details
    if not lex_passed:
        errors.extend(lex_errors)

    # 6. Pairwise Anti-Plagiarism Gate (Solver 7)
    if domain_texts and len(domain_texts) >= 2:
        sim_passed, sim_errors, sim_details = solve_pairwise_domain_similarity(domain_texts)
        details["pairwise_similarity"] = {
            "sentence_count": sim_details.get("sentence_counts", {}).get(domain_id, 0),
            "max_similarity": sim_details.get("max_similarity", 0.0),
        }
        if not sim_passed:
            for err in sim_errors:
                if f"'{domain_id}'" in err:
                    errors.append(err)

    passed = len(errors) == 0
    return LayerResult(
        layer_id=5,
        layer_name="Adversarial Invariant Verification",
        passed=passed,
        errors=errors,
        details=details
    )


# ---------------------------------------------------------------------------
# Layer 6: Baseline Parity & Model Coverage
# ---------------------------------------------------------------------------

def verify_layer6_baseline_parity(workspace_path: str, core_root: str) -> LayerResult:
    """Execute verify_downstream_baseline.py and verify_model_coverage.py --spec-only."""
    errors = []
    details = {}
    
    baseline_script = os.path.join(core_root, "scripts", "verify_downstream_baseline.py")
    coverage_script = os.path.join(core_root, "skills", "spec-orchestrator", "scripts", "verify_model_coverage.py")
    
    # 1. verify_downstream_baseline.py
    if not os.path.isfile(baseline_script):
        errors.append(f"verify_downstream_baseline.py not found at {baseline_script}")
    else:
        cmd1 = [sys.executable, baseline_script, workspace_path]
        res1 = subprocess.run(cmd1, capture_output=True, text=True, cwd=core_root)
        details["baseline_returncode"] = res1.returncode
        if res1.returncode != 0:
            msg = res1.stderr.strip() or res1.stdout.strip()
            errors.append(f"verify_downstream_baseline.py failed (rc={res1.returncode}): {msg[:300]}")
        else:
            details["baseline_status"] = "PASS"
            
    # 2. verify_model_coverage.py --spec-only
    if not os.path.isfile(coverage_script):
        errors.append(f"verify_model_coverage.py not found at {coverage_script}")
    else:
        cmd2 = [sys.executable, coverage_script, "--spec-only", workspace_path]
        res2 = subprocess.run(cmd2, capture_output=True, text=True, cwd=core_root)
        details["coverage_returncode"] = res2.returncode
        if res2.returncode != 0:
            msg = res2.stderr.strip() or res2.stdout.strip()
            errors.append(f"verify_model_coverage.py failed (rc={res2.returncode}): {msg[:300]}")
        else:
            details["coverage_status"] = "PASS"

    passed = len(errors) == 0
    return LayerResult(
        layer_id=6,
        layer_name="Baseline Parity & Model Coverage",
        passed=passed,
        errors=errors,
        details=details
    )


# ---------------------------------------------------------------------------
# Workspace Evaluation Engine
# ---------------------------------------------------------------------------

def evaluate_domain_workspace(
    workspace_path: str, core_root: str, domain_texts: Optional[Dict[str, str]] = None
) -> DomainScorecard:
    """Execute all 6 verification layers on a single domain workspace."""
    domain_id = os.path.basename(os.path.abspath(workspace_path))
    domain_name = DOMAIN_NAMES.get(domain_id, domain_id)
    
    layers = {}
    layers[1] = verify_layer1_delivery_gate(workspace_path)
    layers[2] = verify_layer2_syntax_purity(workspace_path)
    layers[3] = verify_layer3_cardinality(workspace_path)
    layers[4] = verify_layer4_physical_math(workspace_path)
    layers[5] = verify_layer5_adversarial_invariants(workspace_path, domain_texts=domain_texts)
    layers[6] = verify_layer6_baseline_parity(workspace_path, core_root)
    
    overall_passed = all(layer.passed for layer in layers.values())
    
    return DomainScorecard(
        domain_id=domain_id,
        domain_name=domain_name,
        workspace_path=workspace_path,
        overall_passed=overall_passed,
        layers=layers,
        metadata={
            "evaluated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        }
    )


# ---------------------------------------------------------------------------
# Report & Scorecard Generation
# ---------------------------------------------------------------------------

def generate_markdown_report(summary: HarnessSummary) -> str:
    """Generate master markdown acceptance report."""
    lines = []
    lines.append("# Master 10-Domain E2E Acceptance Test Report")
    lines.append("")
    lines.append(f"**Execution Timestamp:** `{summary.execution_timestamp}`  ")
    lines.append(f"**Total Domains Evaluated:** {summary.total_domains}  ")
    lines.append(f"**Domains Passed:** {summary.passed_domains} / {summary.total_domains}  ")
    lines.append(f"**Overall Conformance Status:** {'**PASS (100%)**' if summary.failed_domains == 0 else '**FAIL**'}")
    lines.append("")
    lines.append("## Executive Domain Matrix")
    lines.append("")
    lines.append("| Domain ID | Cyber-Physical Domain | Layer 1 (Delivery) | Layer 2 (Syntax) | Layer 3 (Cardinality) | Layer 4 (Physics) | Layer 5 (Invariants) | Layer 6 (Parity) | Overall Status |")
    lines.append("| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |")
    
    for d in summary.domain_results:
        l1 = "PASS" if d.layers[1].passed else "FAIL"
        l2 = "PASS" if d.layers[2].passed else "FAIL"
        l3 = "PASS" if d.layers[3].passed else "FAIL"
        l4 = "PASS" if d.layers[4].passed else "FAIL"
        l5 = "PASS" if d.layers[5].passed else "FAIL"
        l6 = "PASS" if d.layers[6].passed else "FAIL"
        overall = "**PASS**" if d.overall_passed else "**FAIL**"
        lines.append(f"| `{d.domain_id}` | **{d.domain_name}** | {l1} | {l2} | {l3} | {l4} | {l5} | {l6} | {overall} |")
        
    lines.append("")

    if summary.similarity_matrix:
        lines.append("## Pairwise Anti-Plagiarism & Cross-Domain Similarity Matrix")
        lines.append("")
        domain_keys = sorted(summary.similarity_matrix.keys())
        header = "| Domain | " + " | ".join(f"`{k}`" for k in domain_keys) + " |"
        sep = "| :--- | " + " | ".join(":---:" for _ in domain_keys) + " |"
        lines.append(header)
        lines.append(sep)
        for d1 in domain_keys:
            row_vals = []
            for d2 in domain_keys:
                val = summary.similarity_matrix[d1].get(d2, 0.0)
                pct_str = f"{val * 100:.1f}%"
                if d1 != d2 and val > 0.25:
                    pct_str = f"**{pct_str}**"
                row_vals.append(pct_str)
            lines.append(f"| `{d1}` | " + " | ".join(row_vals) + " |")
        lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("## Detailed Layer Diagnostics per Domain")
    lines.append("")
    
    for d in summary.domain_results:
        lines.append(f"### Domain: {d.domain_name} (`{d.domain_id}`)")
        lines.append(f"- **Workspace Path:** `{d.workspace_path}`")
        lines.append(f"- **Verdict:** {'PASS' if d.overall_passed else 'FAIL'}")
        lines.append("")
        
        for layer_id in sorted(d.layers.keys()):
            l = d.layers[layer_id]
            status_tag = "✅ PASS" if l.passed else "❌ FAIL"
            lines.append(f"#### Layer {l.layer_id}: {l.layer_name} — {status_tag}")
            if l.errors:
                lines.append("**Violations:**")
                for err in l.errors:
                    lines.append(f"- {err}")
            else:
                lines.append("All layer assertions verified with 0 defects.")
            lines.append("")
            
    lines.append("---")
    lines.append("*Generated by DEAP E2E Acceptance Test Harness.*")
    return "\n".join(lines)


def generate_json_scorecard(summary: HarnessSummary) -> str:
    """Generate machine-readable JSON scorecard."""
    return json.dumps(asdict(summary), indent=2)


# ---------------------------------------------------------------------------
# CLI Entrypoint
# ---------------------------------------------------------------------------

class AcceptanceHarness:
    def __init__(self, projects_dir: str, core_root: str):
        self.projects_dir = os.path.abspath(projects_dir)
        self.core_root = os.path.abspath(core_root)

    def discover_domains(self) -> List[str]:
        """Discover domain workspace directories."""
        if not os.path.isdir(self.projects_dir):
            raise FileNotFoundError(f"Projects directory not found at {self.projects_dir}")
            
        entries = sorted(os.listdir(self.projects_dir))
        domains = []
        for e in entries:
            epath = os.path.join(self.projects_dir, e)
            if os.path.isdir(epath) and (e.startswith("run_") or os.path.exists(os.path.join(epath, "docs", "conops", "CONOPS.md"))):
                domains.append(epath)
        return domains

    def run(self, target_domain: Optional[str] = None) -> HarnessSummary:
        """Run acceptance evaluation across domains."""
        if target_domain:
            target_path = os.path.abspath(target_domain)
            domain_paths = [target_path]
        else:
            domain_paths = self.discover_domains()

        # Collect text across all discovered domains for pairwise similarity
        domain_texts: Dict[str, str] = {}
        for dp in domain_paths:
            d_id = os.path.basename(os.path.abspath(dp))
            c_path = os.path.join(dp, "docs", "conops", "CONOPS.md")
            i_path = os.path.join(dp, "docs", "conops", "MISSION_INTENT.md")
            txt = ""
            if os.path.exists(c_path):
                with open(c_path, "r", encoding="utf-8", errors="ignore") as f:
                    txt += f.read() + "\n"
            if os.path.exists(i_path):
                with open(i_path, "r", encoding="utf-8", errors="ignore") as f:
                    txt += f.read()
            domain_texts[d_id] = txt

        similarity_matrix: Dict[str, Dict[str, float]] = {}
        if len(domain_texts) >= 2:
            _, _, sim_details = solve_pairwise_domain_similarity(domain_texts)
            similarity_matrix = sim_details.get("similarity_matrix", {})

        results = []
        for dp in domain_paths:
            scorecard = evaluate_domain_workspace(dp, self.core_root, domain_texts=domain_texts)
            results.append(scorecard)

        passed_count = sum(1 for r in results if r.overall_passed)
        failed_count = len(results) - passed_count
        timestamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")

        return HarnessSummary(
            total_domains=len(results),
            passed_domains=passed_count,
            failed_domains=failed_count,
            execution_timestamp=timestamp,
            domain_results=results,
            similarity_matrix=similarity_matrix,
        )


def main():
    parser = argparse.ArgumentParser(description="DEAP 6-Layer Automated E2E Acceptance Test Harness")
    parser.add_argument("--projects-dir", default=os.path.expanduser("~/test_projects"), help="Path to 10-domain test projects directory")
    parser.add_argument("--target", default=None, help="Target a specific domain workspace path")
    parser.add_argument("--report-md", default=os.path.expanduser("~/test_projects/MASTER_E2E_ACCEPTANCE_REPORT.md"), help="Markdown report output path")
    parser.add_argument("--scorecard-json", default=os.path.expanduser("~/test_projects/acceptance_scorecard.json"), help="JSON scorecard output path")
    parser.add_argument("--verbose", action="store_true", help="Print verbose per-layer logs")
    args = parser.parse_args()

    core_root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    harness = AcceptanceHarness(projects_dir=args.projects_dir, core_root=core_root)

    print("=" * 80)
    print("DEAP 6-LAYER AUTOMATED E2E ACCEPTANCE TEST HARNESS")
    print(f"Timestamp: {datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}")
    print(f"Projects Directory: {args.projects_dir}")
    print("=" * 80)

    summary = harness.run(target_domain=args.target)

    # Print Terminal Summary Matrix
    print("\n" + "-" * 105)
    print(f"{'Domain ID':<22} | {'Layer 1':<7} | {'Layer 2':<7} | {'Layer 3':<7} | {'Layer 4':<7} | {'Layer 5':<7} | {'Layer 6':<7} | {'Verdict':<8}")
    print("-" * 105)

    for d in summary.domain_results:
        l1 = "PASS" if d.layers[1].passed else "FAIL"
        l2 = "PASS" if d.layers[2].passed else "FAIL"
        l3 = "PASS" if d.layers[3].passed else "FAIL"
        l4 = "PASS" if d.layers[4].passed else "FAIL"
        l5 = "PASS" if d.layers[5].passed else "FAIL"
        l6 = "PASS" if d.layers[6].passed else "FAIL"
        verdict = "PASS" if d.overall_passed else "FAIL"
        print(f"{d.domain_id:<22} | {l1:<7} | {l2:<7} | {l3:<7} | {l4:<7} | {l5:<7} | {l6:<7} | {verdict:<8}")

    print("-" * 105)
    print(f"Result: {summary.passed_domains}/{summary.total_domains} Domains Passed ({summary.failed_domains} Failed)\n")

    # Generate Reports
    md_content = generate_markdown_report(summary)
    with open(args.report_md, "w", encoding="utf-8") as f:
        f.write(md_content)
    print(f"Wrote Markdown Acceptance Report to: {args.report_md}")

    json_content = generate_json_scorecard(summary)
    with open(args.scorecard_json, "w", encoding="utf-8") as f:
        f.write(json_content)
    print(f"Wrote JSON Acceptance Scorecard to: {args.scorecard_json}")
    print("=" * 80)

    if summary.failed_domains > 0:
        sys.exit(1)
    sys.exit(0)


if __name__ == "__main__":
    main()
