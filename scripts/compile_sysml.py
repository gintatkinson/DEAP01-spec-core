#!/usr/bin/env python3
"""
SysML v2 Compiler, STPA Safety Constraints & RTA Compiler & Textual Model Serializer

Compiles and parses SysML v2 textual models into structured AST representations,
extracting all 6 core model constructs (packages, parts, attributes, ports,
actions, capabilities, operations, interactions, constraints/assertions,
test cases, requirements, states, use cases, items).

Implements STPA-to-SysML compilation: parses STPA Unsafe Control Actions (UCAs)
and FMECA failure modes, compiling them into formal SysML v2 `constraint def` and
`assert constraint` expressions for Run-Time Assurance (RTA) mathematical verification
with Simulink Design Verifier (SLDV) and Embedded Coder synthesis.

Implements Closed-Loop Bidirectional Synchronization (--reverse-sync):
Extracts Use Cases, User Stories, Features, Epics, and Safety Matrices from markdown
specifications into canonical SysML v2 AST nodes, merging them deterministically into
the SysML Single Source of Truth (.pipeline/schema.sysml) and regenerating .pipeline/schema-digest.json.

Usage:
    python3 scripts/compile_sysml.py <file.sysml>
    python3 scripts/compile_sysml.py --stpa <stpa_file.md>
    python3 scripts/compile_sysml.py --reverse-sync [--docs docs/] [--schema schema/DEAP_MODEL.sysml] [--out .pipeline/schema.sysml] [--digest .pipeline/schema-digest.json]
"""

import sys
import json
import os
import re
import hashlib
import argparse
import tempfile
from typing import Dict, List, Any, Optional, Tuple, Union

# Ensure spec-orchestrator scripts are on sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.dirname(SCRIPT_DIR)
SPEC_SCRIPTS_DIR = os.path.join(PROJECT_ROOT, "skills", "spec-orchestrator", "scripts")
if SPEC_SCRIPTS_DIR not in sys.path:
    sys.path.insert(0, SPEC_SCRIPTS_DIR)

try:
    import yaml
except ImportError:
    yaml = None

try:
    from sysmlv2_ast import (
        SysMLParser,
        SysMLPackage,
        SysMLConstraintDef,
        PartDef,
        SysMLPart,
        AttributeDef,
        PortDef,
        ActionDef,
        SysMLOperationDef,
        SysMLCapabilityDef,
        SysMLInteractionDef,
        SysMLTestCaseDef,
        RequirementDef,
        StateDef,
        UseCaseDef,
        ItemDef,
    )
except ImportError:
    SysMLParser = None
    SysMLPackage = None
    SysMLConstraintDef = None
    PartDef = None
    SysMLPart = None
    AttributeDef = None
    PortDef = None
    ActionDef = None
    SysMLOperationDef = None
    SysMLCapabilityDef = None
    SysMLInteractionDef = None
    SysMLTestCaseDef = None
    RequirementDef = None
    StateDef = None
    UseCaseDef = None
    ItemDef = None


def parse_stpa_ucas(content: str) -> List[Dict[str, Any]]:
    """
    Parses STPA Unsafe Control Actions (UCAs) from markdown tables, structured text,
    or 4-guide-word specification matrices (MIL-STD-882E / STPA / SORA).

    Returns:
        List of dicts representing parsed UCAs with fields:
        - id: UCA identifier (e.g. 'UCA-UAS-01' or synthesized 'UCA_Engage_Autonomous_RTL_Not_providing_1')
        - controller: Controlling element (e.g. 'Flight Controller')
        - control_action: Action commanded (e.g. 'Engage Autonomous RTL')
        - category: STPA UCA guide word (e.g. 'Not providing', 'Providing', 'Too late', 'Stopped too soon')
        - context: Environmental Context / Trigger condition
        - hazard: Associated System Hazard (e.g. 'H-1, H-5')
        - constraint: Formal safety constraint (e.g. 'SC-01, SC-06')
        - severity: Severity level (e.g. 'Catastrophic')
        - sail: SORA SAIL level (e.g. 'SAIL IV-VI')
    """
    ucas = []

    def _is_table_separator(line: str) -> bool:
        s = line.strip()
        if not s.startswith("|"):
            return False
        inner = s.replace("|", "").strip()
        return bool(inner and set(inner) <= {"-", ":", " "} and "-" in inner)

    # Pattern 1: Markdown table row with explicit UCA ID
    # | UCA ID | Controller | Control Action | STPA UCA Category | Context | Hazard | Severity | SAIL |
    row_pattern = re.compile(
        r'\|\s*(?:\*\*)?(UCA(?:-[A-Za-z0-9_]+)?-\d+)(?:\*\*)?\s*\|'
        r'\s*([^|]+)\s*\|'
        r'\s*([^|]+)\s*\|'
        r'\s*([^|]+)\s*\|'
        r'\s*([^|]+)\s*\|'
        r'\s*([^|]+)\s*\|'
        r'\s*([^|]+)\s*\|'
        r'(?:\s*([^|\n]+)\s*\|)?'
    )

    for match in row_pattern.finditer(content):
        uca_id = match.group(1).strip()
        controller = match.group(2).strip()
        control_action = match.group(3).strip()
        category = match.group(4).strip().strip('*')
        context = match.group(5).strip()
        hazard = match.group(6).strip().strip('*')
        severity = match.group(7).strip()
        sail = match.group(8).strip() if match.group(8) else ""

        ucas.append({
            "id": uca_id,
            "controller": controller,
            "control_action": control_action,
            "category": category,
            "context": context,
            "hazard": hazard,
            "constraint": "",
            "severity": severity,
            "sail": sail
        })

    # Pattern 2: 4-Guide-Word STPA Taxonomy Matrix (MIL-STD-882E / SORA)
    # | Control Action | Guide Word | Context / State | Resulting Hazard | Safety Constraint |
    lines = content.splitlines()
    in_guideword_table = False
    active_action = ""
    action_counter = {}

    for line in lines:
        stripped = line.strip()
        if not stripped.startswith("|"):
            in_guideword_table = False
            continue

        if _is_table_separator(stripped):
            continue

        lower = stripped.lower()
        if "control action" in lower and "guide word" in lower:
            in_guideword_table = True
            continue

        if in_guideword_table:
            cols = [c.strip() for c in stripped.strip("|").split("|")]
            if len(cols) >= 4:
                col_action = cols[0].strip().strip("*")
                col_guideword = cols[1].strip().strip("*")
                col_context = cols[2].strip()
                col_hazard = cols[3].strip().strip("*")
                col_constraint = cols[4].strip().strip("*") if len(cols) > 4 else ""

                if col_action and not col_action.startswith("-"):
                    active_action = col_action

                # Verify guide word is recognized STPA guide word
                gw_lower = col_guideword.lower()
                is_valid_gw = any(g in gw_lower for g in [
                    "not providing", "providing", "too early", "too late",
                    "out of order", "stopped too soon", "applied too long"
                ])

                if is_valid_gw and (col_context or col_hazard):
                    clean_action = _sanitize_id(active_action or "SafetyAction")
                    clean_gw = _sanitize_id(col_guideword)
                    key = f"{clean_action}_{clean_gw}"
                    action_counter[key] = action_counter.get(key, 0) + 1
                    synthesized_id = f"UCA_{clean_action}_{clean_gw}_{action_counter[key]}"

                    if not any(u["id"] == synthesized_id for u in ucas):
                        ucas.append({
                            "id": synthesized_id,
                            "controller": "FlightSafetyController",
                            "control_action": active_action,
                            "category": col_guideword,
                            "context": col_context,
                            "hazard": col_hazard,
                            "constraint": col_constraint,
                            "severity": "Catastrophic",
                            "sail": "SAIL II-IV"
                        })

    # Pattern 3: Generic UCA extraction fallback (e.g. list items or headings)
    if not ucas:
        generic_pattern = re.compile(r'\b(UCA(?:-[A-Za-z0-9_]+)?-\d+)\b')
        for match in generic_pattern.finditer(content):
            uid = match.group(1)
            if not any(u["id"] == uid for u in ucas):
                ucas.append({
                    "id": uid,
                    "controller": "SafetyController",
                    "control_action": "SystemSafetyAction",
                    "category": "UnsafeControlAction",
                    "context": "OperationalBoundExceeded",
                    "hazard": "H_System_Hazard",
                    "constraint": "SC_Safety_Constraint",
                    "severity": "Critical",
                    "sail": "SafetyLevel_High"
                })

    return ucas


def parse_fmeca_modes(content: str) -> List[Dict[str, Any]]:
    """
    Parses FMECA failure modes from markdown tables or specification text,
    supporting both qualitative RPN tables and multi-mode quantitative MIL-STD-1629A tables.
    """
    fmecas = []

    def _is_table_separator(line: str) -> bool:
        s = line.strip()
        if not s.startswith("|"):
            return False
        inner = s.replace("|", "").strip()
        return bool(inner and set(inner) <= {"-", ":", " "} and "-" in inner)

    # Pattern 1: Classic explicit FMECA ID row
    # | FMECA-ID | Component | Failure Mode | Effect | Mitigation |
    row_pattern = re.compile(
        r'\|\s*(?:\*\*)?(FMECA(?:-[A-Za-z0-9_]+)?-\d+)(?:\*\*)?\s*\|'
        r'\s*([^|]+)\s*\|'
        r'\s*([^|]+)\s*\|'
        r'\s*([^|]+)\s*\|'
        r'(?:\s*([^|\n]+)\s*\|)?'
    )

    for match in row_pattern.finditer(content):
        fmeca_id = match.group(1).strip()
        component = match.group(2).strip()
        failure_mode = match.group(3).strip()
        effect = match.group(4).strip()
        mitigation = match.group(5).strip() if match.group(5) else ""

        fmecas.append({
            "id": fmeca_id,
            "component": component,
            "failure_mode": failure_mode,
            "effect": effect,
            "mitigation": mitigation,
            "is_quantitative": False
        })

    # Pattern 2: Multi-Mode Quantitative / Qualitative FMECA Table with Context Inheritance
    lines = content.splitlines()
    in_fmeca_table = False
    is_quantitative = False
    active_component = ""
    comp_mode_count = {}

    for line in lines:
        stripped = line.strip()
        if not stripped.startswith("|"):
            in_fmeca_table = False
            continue

        if _is_table_separator(stripped):
            continue

        lower = stripped.lower()
        if "failure mode" in lower or "failure mode & mechanism" in lower or "failure mechanism" in lower:
            # Check if this is an explicit FMECA-ID table header
            if "fmeca-id" in lower or "fmeca id" in lower:
                in_fmeca_table = False
                continue
            in_fmeca_table = True
            is_quantitative = "lambda" in lower or "alpha" in lower or "c_m" in lower or "criticality" in lower
            continue

        if in_fmeca_table:
            cols = [c.strip() for c in stripped.strip("|").split("|")]
            if len(cols) >= 3:
                raw_comp = cols[0].strip().strip("*")
                if raw_comp and not raw_comp.startswith("-"):
                    active_component = raw_comp

                failure_mode = cols[1].strip() if len(cols) > 1 else ""

                # Skip header-like rows or empty mode rows
                if not failure_mode or "failure mode" in failure_mode.lower() or "failure mechanism" in failure_mode.lower():
                    continue

                clean_comp = _sanitize_id(active_component or "Component")
                comp_mode_count[clean_comp] = comp_mode_count.get(clean_comp, 0) + 1
                mode_idx = comp_mode_count[clean_comp]
                synthesized_id = f"FMECA_{clean_comp}_Mode_{mode_idx}"

                if is_quantitative and len(cols) >= 8:
                    def _parse_float(val_str: str) -> float:
                        try:
                            clean_val = re.sub(r'[^0-9eE\.\-]', '', val_str)
                            return float(clean_val) if clean_val else 0.0
                        except ValueError:
                            return 0.0

                    lambda_p = _parse_float(cols[2])
                    alpha = _parse_float(cols[3])
                    beta = _parse_float(cols[4])
                    c_m = _parse_float(cols[5])
                    c_r = _parse_float(cols[6])
                    severity = cols[7].strip()
                    mitigation = cols[8].strip() if len(cols) > 8 else ""

                    fmecas.append({
                        "id": synthesized_id,
                        "component": active_component,
                        "failure_mode": failure_mode,
                        "lambda_p": lambda_p,
                        "alpha": alpha,
                        "beta": beta,
                        "c_m": c_m,
                        "c_r": c_r,
                        "severity": severity,
                        "mitigation": mitigation,
                        "is_quantitative": True
                    })
                else:
                    # Qualitative / Standard columns
                    effect = cols[2].strip() if len(cols) > 2 else ""
                    mitigation = cols[-1].strip() if len(cols) > 3 else ""
                    fmecas.append({
                        "id": synthesized_id,
                        "component": active_component,
                        "failure_mode": failure_mode,
                        "effect": effect,
                        "mitigation": mitigation,
                        "is_quantitative": False
                    })

    # Pattern 3: Generic fallback
    if not fmecas:
        generic_pattern = re.compile(r'\b(FMECA(?:-[A-Za-z0-9_]+)?-\d+)\b')
        for match in generic_pattern.finditer(content):
            fid = match.group(1)
            if not any(f["id"] == fid for f in fmecas):
                fmecas.append({
                    "id": fid,
                    "component": "GenericComponent",
                    "failure_mode": "GenericFailureMode",
                    "effect": "DegradedOperation",
                    "mitigation": "RedundantSwitchover",
                    "is_quantitative": False
                })

    return fmecas


def _sanitize_id(identifier: str) -> str:
    """Converts hyphens and non-alphanumeric chars into clean underscores for SysML IDs."""
    if not identifier:
        return ""
    sanitized = re.sub(r'[^a-zA-Z0-9_]', '_', identifier)
    sanitized = re.sub(r'_+', '_', sanitized).strip('_')
    return sanitized


def _derive_formal_rta_expression(uca: Dict[str, Any]) -> str:
    """
    Synthesizes a mathematically verifiable formal assertion predicate expression
    from an STPA UCA context, guide word, and control action.
    """
    uca_id = uca.get("id", "")
    context = uca.get("context", "")
    action = uca.get("control_action", "")
    category = uca.get("category", "").lower()

    # Clean LaTeX math formatting
    clean_ctx = re.sub(r'[\$\_{}]', '', context)
    clean_ctx = re.sub(r'\\text\{([^}]*)\}', r'\1', clean_ctx)
    clean_ctx = re.sub(r'\\mathbf\{([^}]*)\}', r'\1', clean_ctx)
    clean_ctx = re.sub(r'\\mu', 'micro', clean_ctx)
    clean_ctx = re.sub(r'\\le', '<=', clean_ctx)
    clean_ctx = re.sub(r'\\ge', '>=', clean_ctx)

    if "not providing" in category:
        if "c2" in clean_ctx.lower() or "loss" in clean_ctx.lower() or "link" in clean_ctx.lower():
            return "c2LinkLossDuration < 30.0"
        elif "pressure" in clean_ctx.lower() or "bar" in clean_ctx.lower():
            return "railPressure >= 13.0"
        elif "distance" in clean_ctx.lower() or "boundary" in clean_ctx.lower():
            return "distanceToBoundary >= 50.0"
        else:
            return "systemCommandIssued == true"
    elif "providing" in category:
        if "flare" in clean_ctx.lower() or "agl" in clean_ctx.lower():
            return "altitudeAGL > 2.0"
        elif "cruise" in clean_ctx.lower():
            return "flightPhase != Cruise"
        else:
            return "boundaryInBounds == true"
    elif "too late" in category:
        if "soc" in clean_ctx.lower() or "battery" in clean_ctx.lower():
            return "batterySoC >= 0.20"
        elif "velocity" in clean_ctx.lower() or "m/s" in clean_ctx.lower() or "v =" in clean_ctx.lower():
            return "boundaryCrossVelocity <= 31.0"
        else:
            return "reactionLatency <= maxAllowedLatency"
    elif "stopped too soon" in category:
        if "altitude" in clean_ctx.lower():
            return "transitAltitude >= safeTransitAltitude"
        else:
            return "commandHoldDuration >= minRequiredDuration"
    elif "applied too long" in category:
        return "turnHoldDuration <= maxTurnDuration"
    else:
        return "systemParameter <= maxThreshold"


def compile_uca_to_constraint(uca: Dict[str, Any]) -> Any:
    """
    Compiles a parsed UCA into a formal SysMLConstraintDef AST node configured
    as an `assert constraint` for Run-Time Assurance (RTA) mathematical verification.
    """
    uca_id = uca["id"]
    clean_id = _sanitize_id(uca_id)
    name = f"Assert_{clean_id}"
    expression = _derive_formal_rta_expression(uca)

    doc = (
        f"STPA RTA Safety Invariant for {uca_id} | Action: {uca.get('control_action', '')} | "
        f"Guide Word: {uca.get('category', '')} | Hazard: {uca.get('hazard', '')} | "
        f"Constraint: {uca.get('constraint', '')} | Severity: {uca.get('severity', '')}"
    )

    if SysMLConstraintDef:
        return SysMLConstraintDef(
            name=name,
            expression=expression,
            is_assertion=True,
            doc=doc
        )
    return {
        "name": name,
        "expression": expression,
        "is_assertion": True,
        "doc": doc
    }


def compile_fmeca_to_constraint(fmeca: Dict[str, Any]) -> Any:
    """
    Compiles a parsed FMECA failure mode into a formal SysMLConstraintDef AST node.
    """
    fmeca_id = fmeca["id"]
    clean_id = _sanitize_id(fmeca_id)
    name = f"Constraint_{clean_id}"
    comp = _sanitize_id(fmeca.get("component", "Component"))
    expression = f"{comp}_healthStatus == Normal"

    if fmeca.get("is_quantitative"):
        doc = (
            f"MIL-STD-1629A FMECA Invariant for {fmeca_id} | Component: {fmeca.get('component', '')} | "
            f"Failure Mode: {fmeca.get('failure_mode', '')} | lambda_p: {fmeca.get('lambda_p', 0.0)}/10^6 hr | "
            f"alpha: {fmeca.get('alpha', 0.0)} | beta: {fmeca.get('beta', 0.0)} | "
            f"Cm: {fmeca.get('c_m', 0.0):.2e} | Cr: {fmeca.get('c_r', 0.0):.2e} | Mitigation: {fmeca.get('mitigation', '')}"
        )
    else:
        doc = f"FMECA Safety Invariant for {fmeca_id} | Failure Mode: {fmeca.get('failure_mode', '')}"

    if SysMLConstraintDef:
        return SysMLConstraintDef(
            name=name,
            expression=expression,
            is_assertion=False,
            doc=doc
        )
    return {
        "name": name,
        "expression": expression,
        "is_assertion": False,
        "doc": doc
    }
def compile_stpa_to_ast(content: str, package_name: str = "AutonomousUAS_SafetyConstraints") -> Any:
    """
    Compiles STPA and FMECA hazard analyses into a canonical SysMLPackage AST containing
    formal `assert constraint` and `constraint def` nodes.
    """
    ucas = parse_stpa_ucas(content)
    fmecas = parse_fmeca_modes(content)

    constraints = []
    for u in ucas:
        constraints.append(compile_uca_to_constraint(u))
    for f in fmecas:
        constraints.append(compile_fmeca_to_constraint(f))

    if SysMLPackage:
        pkg = SysMLPackage(
            name=package_name,
            doc="STPA and FMECA Safety Invariants compiled for Run-Time Assurance (RTA) & SLDV Verification",
            constraint_defs=constraints
        )
        return pkg
    return {
        "package": package_name,
        "constraints": constraints
    }


def compile_stpa_to_sysml(content: str, package_name: str = "AutonomousUAS_SafetyConstraints") -> str:
    """
    Compiles STPA hazard matrices and FMECA modes into textual SysML v2 model notation.
    """
    ast_pkg = compile_stpa_to_ast(content, package_name)
    if hasattr(ast_pkg, "to_sysml"):
        return ast_pkg.to_sysml()

    lines = [f"package {package_name} {{", f"    doc /* STPA RTA Safety Invariants */"]
    for c in ast_pkg.get("constraints", []):
        kw = "assert constraint" if c.get("is_assertion") else "constraint def"
        lines.append(f"    doc /* {c.get('doc', '')} */")
        lines.append(f"    {kw} {c.get('name')} {{")
        lines.append(f"        {c.get('expression')};")
        lines.append("    }\n")
    lines.append("}")
    return "\n".join(lines)


# ==============================================================================
# MARKDOWN SPECIFICATION PARSERS (REVERSE SYNCHRONIZATION)
# ==============================================================================

def _parse_frontmatter(content: str) -> Tuple[Dict[str, Any], str]:
    """Parses YAML frontmatter from markdown content if present, returning (frontmatter_dict, body_text)."""
    fm_match = re.match(r"^---\s*\n(.*?)\n---\s*\n", content, re.DOTALL)
    if not fm_match:
        return {}, content
    fm_text = fm_match.group(1)
    body = content[fm_match.end():]
    fm_dict = {}
    if yaml is not None:
        try:
            data = yaml.safe_load(fm_text.replace('\x01', ''))
            if isinstance(data, dict):
                fm_dict = data
        except Exception:
            pass
    if not fm_dict:
        for line in fm_text.splitlines():
            if ":" in line:
                k, v = line.split(":", 1)
                k = k.strip()
                v = v.strip().strip('"').strip("'")
                if v.startswith("[") and v.endswith("]"):
                    items = [x.strip().strip('"').strip("'") for x in v[1:-1].split(",") if x.strip()]
                    fm_dict[k] = items
                else:
                    fm_dict[k] = v
    return fm_dict, body


def _to_pascal_case(text: str) -> str:
    """Converts space/hyphen/underscore-separated text into clean PascalCase identifier."""
    clean = re.sub(r'^(?:epic|feat|feature|user[- ]story|use[- ]case|us|uc)[s]?(?:[- ]*\d+\s*[:\-]?|:)\s*', '', text, flags=re.IGNORECASE)
    words = re.findall(r'[a-zA-Z0-9]+', clean)
    if not words:
        words = re.findall(r'[a-zA-Z0-9]+', text)
    return "".join(w.capitalize() for w in words)


def extract_use_cases_from_markdown(content: str, filename: str = "") -> List[Any]:
    """
    Parses Use Case markdown specification:
    Extracts Use Case name, subject, actor list, objective, include and extend references,
    constructing UseCaseDef AST nodes.
    """
    fm, body = _parse_frontmatter(content)
    use_cases = []

    # Name derivation
    name = ""
    if fm.get("use_case_def"):
        name = str(fm["use_case_def"])
    elif fm.get("use_case"):
        name = str(fm["use_case"])
    elif fm.get("name"):
        name = str(fm["name"])
    elif fm.get("title"):
        name = _to_pascal_case(str(fm["title"]))

    if not name:
        uc_def_m = re.search(r'\buse\s+case\s+(?:def\s+)?([a-zA-Z0-9_]+)', body)
        if uc_def_m:
            name = uc_def_m.group(1)
        else:
            h1_m = re.search(r'^#\s+(?:Use\s+Case\s*:\s*)?(.*)$', body, re.MULTILINE)
            if h1_m:
                name = _to_pascal_case(h1_m.group(1))
            elif filename:
                base = os.path.splitext(os.path.basename(filename))[0]
                name = _to_pascal_case(base)
            else:
                name = "AutonomousUseCase"

    name = _sanitize_id(name)
    if name and name[0].isdigit():
        name = f"UC_{name}"

    # Subject
    subject = ""
    if fm.get("subject_part"):
        subject = str(fm["subject_part"])
    elif fm.get("subject"):
        subject = str(fm["subject"])
    else:
        subj_m = re.search(r'\bSubject(?:\s+Part)?\b[\s\*:]*[:=]?[\s\*]*`?([A-Za-z0-9_]+)`?', body, re.IGNORECASE)
        if subj_m:
            subject = subj_m.group(1)
        else:
            subj_stmt = re.search(r'\bsubject\s+([a-zA-Z0-9_]+);', body)
            if subj_stmt:
                subject = subj_stmt.group(1)

    # Actor
    actor = ""
    actors_list = []
    if fm.get("actors"):
        val = fm["actors"]
        if isinstance(val, list):
            actors_list = [str(a) for a in val]
        else:
            actors_list = [str(val)]
    elif fm.get("actor"):
        actors_list = [str(fm["actor"])]
    else:
        act_m = re.search(r'\bActor(?:s)?\b[\s\*:]*[:=]?[\s\*]*(.+)', body, re.IGNORECASE)
        if act_m:
            raw_actors = act_m.group(1).strip()
            found_actors = re.findall(r'`?([A-Za-z0-9_]+)`?', raw_actors)
            actors_list = [a for a in found_actors if a.lower() not in ("actors", "actor", "none", "n", "a")]
        else:
            act_stmts = re.findall(r'\bactor\s+([a-zA-Z0-9_]+);', body)
            if act_stmts:
                actors_list = act_stmts

    if actors_list:
        actor = actors_list[0]

    # Objective
    objective = ""
    if fm.get("objective"):
        objective = str(fm["objective"])
    elif fm.get("description"):
        objective = str(fm["description"])
    else:
        obj_m = re.search(r'\b(?:Verification\s+)?Objective\b[\s\*:]*[:=]?[\s\*]*["\']?([^"\'\n\r]+)["\']?', body, re.IGNORECASE)
        if obj_m:
            objective = obj_m.group(1).strip()
        else:
            obj_stmt = re.search(r'\bobjective\s*[:=]?\s*["\']([^"\']+)["\'];', body)
            if obj_stmt:
                objective = obj_stmt.group(1).strip()

    # Includes
    includes = []
    if fm.get("includes"):
        val = fm["includes"]
        if isinstance(val, list):
            includes = [str(x) for x in val]
        else:
            includes = [str(val)]
    elif fm.get("include"):
        includes = [str(fm["include"])]

    inc_stmts = re.findall(r'\binclude\s+([a-zA-Z0-9_]+);', body)
    includes.extend(inc_stmts)
    inc_m = re.search(r'\bInclude(?:s)?\b[\s\*:]*[:=]?[\s\*]*(.+)', body, re.IGNORECASE)
    if inc_m:
        found_incs = re.findall(r'`?([A-Za-z0-9_]+)`?', inc_m.group(1))
        includes.extend([x for x in found_incs if x.lower() not in ("includes", "include", "none", "n", "a")])
    diag_incs = re.findall(r'(?:<<|&lt;&lt;|«)\s*include(?:s)?\s*(?:>>|&gt;&gt;|»)\s*\|?\s*`?([A-Za-z0-9_]+)`?', body, re.IGNORECASE)
    includes.extend(diag_incs)

    seen_inc = set()
    dedup_includes = []
    for inc in includes:
        clean_inc = _sanitize_id(inc)
        if clean_inc and clean_inc not in seen_inc:
            seen_inc.add(clean_inc)
            dedup_includes.append(clean_inc)

    # Extends
    extends = []
    if fm.get("extends"):
        val = fm["extends"]
        if isinstance(val, list):
            extends = [str(x) for x in val]
        else:
            extends = [str(val)]
    elif fm.get("extend"):
        extends = [str(fm["extend"])]

    ext_stmts = re.findall(r'\bextend\s+([a-zA-Z0-9_]+);', body)
    extends.extend(ext_stmts)
    ext_m = re.search(r'\bExtend(?:s)?\b[\s\*:]*[:=]?[\s\*]*(.+)', body, re.IGNORECASE)
    if ext_m:
        found_exts = re.findall(r'`?([A-Za-z0-9_]+)`?', ext_m.group(1))
        extends.extend([x for x in found_exts if x.lower() not in ("extends", "extend", "none", "n", "a")])
    diag_exts = re.findall(r'(?:<<|&lt;&lt;|«)\s*extend(?:s)?\s*(?:>>|&gt;&gt;|»)\s*\|?\s*`?([A-Za-z0-9_]+)`?', body, re.IGNORECASE)
    extends.extend(diag_exts)

    seen_ext = set()
    dedup_extends = []
    for ext in extends:
        clean_ext = _sanitize_id(ext)
        if clean_ext and clean_ext not in seen_ext:
            seen_ext.add(clean_ext)
            dedup_extends.append(clean_ext)

    doc = objective or (str(fm.get("title", "")) if fm else "")

    if UseCaseDef:
        uc_obj = UseCaseDef(
            name=name,
            doc=doc,
            subject=subject,
            actor=actor,
            objective=objective,
            includes=dedup_includes,
            extends=dedup_extends
        )
    else:
        uc_obj = {
            "name": name,
            "doc": doc,
            "subject": subject,
            "actor": actor,
            "objective": objective,
            "includes": dedup_includes,
            "extends": dedup_extends
        }
    use_cases.append(uc_obj)
    return use_cases


def extract_user_story_ast(content: str, filename: str = "") -> Tuple[List[Any], List[Any]]:
    """
    Parses User Story markdown specifications:
    Extracts sequence diagram lifelines, message exchanges, and Stateflow transition triggers,
    constructing SysMLInteractionDef nodes.
    Also extracts Acceptance Criteria BDD Scenarios, constructing SysMLTestCaseDef nodes with
    `verify requirement` bindings.
    """
    fm, body = _parse_frontmatter(content)
    interactions = []
    test_cases = []

    # --- 1. Interaction Extraction ---
    inter_name = ""
    if fm.get("interaction"):
        inter_name = str(fm["interaction"])
    elif fm.get("interaction_def"):
        inter_name = str(fm["interaction_def"])
    else:
        inter_stmt = re.search(r'\binteraction\s+(?:def\s+)?([a-zA-Z0-9_]+)', body)
        if inter_stmt:
            inter_name = inter_stmt.group(1)
        elif fm.get("title"):
            inter_name = _to_pascal_case(str(fm["title"]))
        elif filename:
            base = os.path.splitext(os.path.basename(filename))[0]
            inter_name = _to_pascal_case(base)
        else:
            inter_name = "UserStoryInteraction"

    inter_name = _sanitize_id(inter_name)
    if inter_name and inter_name[0].isdigit():
        inter_name = f"Interaction_{inter_name}"

    lifelines = []
    messages = []
    triggers = []

    seq_matches = re.finditer(r'```mermaid\s*\n\s*sequenceDiagram(.*?)(?=```|\Z)', body, re.DOTALL)
    for sm in seq_matches:
        seq_text = sm.group(1)
        for line in seq_text.splitlines():
            line = line.strip()
            if not line or line.startswith('%%'):
                continue
            # Lifeline: participant / actor with alias as "alias : Classifier"
            part_as_m = re.search(r'\b(?:participant|actor)\s+([a-zA-Z0-9_]+)\s+as\s+["\']?[^:]*:\s*([a-zA-Z0-9_]+)["\']?', line)
            if part_as_m:
                cls_name = part_as_m.group(2)
                if cls_name not in lifelines:
                    lifelines.append(cls_name)
                continue
            part_simple_m = re.search(r'\b(?:participant|actor)\s+([a-zA-Z0-9_]+)', line)
            if part_simple_m:
                p_name = part_simple_m.group(1)
                if p_name not in lifelines:
                    lifelines.append(p_name)
                continue
            # Messages: a->>b: Operation(params) or a->>b: Operation
            msg_m = re.search(r'->>?[^:]*:\s*([a-zA-Z0-9_]+)(?:\(.*\))?', line)
            if msg_m:
                op_msg = msg_m.group(1).strip()
                if op_msg not in messages and op_msg.lower() not in ("status", "ack", "reply"):
                    messages.append(op_msg)

    for ll in re.findall(r'\blifeline\s+([a-zA-Z0-9_]+);?', body):
        if ll not in lifelines:
            lifelines.append(ll)
    for msg in re.findall(r'\bmessage\s+([a-zA-Z0-9_]+);?', body):
        if msg not in messages:
            messages.append(msg)
    for trg in re.findall(r'\btrigger\s+([a-zA-Z0-9_]+);?', body):
        if trg not in triggers:
            triggers.append(trg)

    if fm.get("triggers"):
        t_val = fm["triggers"]
        if isinstance(t_val, list):
            for t in t_val:
                if str(t) not in triggers:
                    triggers.append(str(t))
        else:
            if str(t_val) not in triggers:
                triggers.append(str(t_val))

    inter_doc = str(fm.get("title", "")) or "User Story Interaction Sequence"
    if lifelines or messages or triggers or fm.get("interaction"):
        if SysMLInteractionDef:
            interactions.append(SysMLInteractionDef(
                name=inter_name,
                lifelines=lifelines,
                messages=messages,
                triggers=triggers,
                doc=inter_doc
            ))
        else:
            interactions.append({
                "name": inter_name,
                "lifelines": lifelines,
                "messages": messages,
                "triggers": triggers,
                "doc": inter_doc
            })

    # --- 2. Test Case Extraction ---
    tc_name = ""
    if fm.get("test_case"):
        tc_name = str(fm["test_case"])
    elif fm.get("test_case_def"):
        tc_name = str(fm["test_case_def"])
    elif fm.get("test_cases"):
        tc_val = fm["test_cases"]
        tc_name = str(tc_val[0]) if isinstance(tc_val, list) else str(tc_val)
    else:
        tc_m = re.search(r'\b(?:SysML\s+)?Test\s+Case(?:\s+Def)?\b[\s\*:]*[:=]?[\s\*]*`?([A-Za-z0-9_]+)`?', body)
        if tc_m:
            tc_name = tc_m.group(1)
        else:
            tc_prefix_m = re.search(r'\b(TC_[A-Za-z0-9_]+)\b', body)
            if tc_prefix_m:
                tc_name = tc_prefix_m.group(1)
            elif fm.get("title"):
                tc_name = f"TC_{_to_pascal_case(str(fm['title']))}"
            elif filename:
                base = os.path.splitext(os.path.basename(filename))[0]
                tc_name = f"TC_{_to_pascal_case(base)}"
            else:
                tc_name = "TC_UserStoryVerification"

    tc_name = _sanitize_id(tc_name)

    subject_part = ""
    if fm.get("subject_part"):
        subject_part = str(fm["subject_part"])
    elif fm.get("subject"):
        subject_part = str(fm["subject"])
    else:
        subj_m = re.search(r'\bSubject(?:\s+Part)?\b[\s\*:]*[:=]?[\s\*]*`?([A-Za-z0-9_]+)`?', body, re.IGNORECASE)
        if subj_m:
            subject_part = subj_m.group(1)
        elif lifelines:
            subject_part = lifelines[0]

    verified_reqs = []
    if fm.get("verified_requirements"):
        vr_val = fm["verified_requirements"]
        if isinstance(vr_val, list):
            verified_reqs = [str(r) for r in vr_val]
        else:
            verified_reqs = [str(vr_val)]
    elif fm.get("verified_requirement"):
        verified_reqs = [str(fm["verified_requirement"])]
    elif fm.get("requirement"):
        verified_reqs = [str(fm["requirement"])]

    vr_m = re.findall(r'\bVerified\s+(?:Safety\s+)?Requirement\b[\s\*:]*[:=]?[\s\*]*`?([A-Za-z0-9_\-]+)`?', body, re.IGNORECASE)
    verified_reqs.extend(vr_m)
    vr_stmts = re.findall(r'\bverify\s+requirement\s+([a-zA-Z0-9_\-]+);?', body)
    verified_reqs.extend(vr_stmts)

    clean_reqs = []
    for r in verified_reqs:
        clean_r = _sanitize_id(r)
        if clean_r and clean_r not in clean_reqs:
            clean_reqs.append(clean_r)

    objective = ""
    if fm.get("objective"):
        objective = str(fm["objective"])
    else:
        obj_m = re.search(r'\b(?:Verification\s+)?Objective\b[\s\*:]*[:=]?[\s\*]*["\']?([^"\'\n\r]+)["\']?', body, re.IGNORECASE)
        if obj_m:
            objective = obj_m.group(1).strip()
        else:
            obj_stmt = re.search(r'\bobjective\s*[:=]?\s*["\']([^"\']+)["\'];', body)
            if obj_stmt:
                objective = obj_stmt.group(1).strip()
            else:
                bdd_m = re.search(r'(Given\b.*?\bWhen\b.*?\bThen\b[^\n\r]+)', body, re.DOTALL | re.IGNORECASE)
                if bdd_m:
                    objective = re.sub(r'[\r\n\*\#]+', ' ', bdd_m.group(1)).strip()

    test_steps = []
    steps_block_m = re.search(r'(?:\*\*|#+\s+)?Test\s+Steps(?:\*\*)?\s*[:=]?(.*?)(?=\n#|\Z)', body, re.DOTALL | re.IGNORECASE)
    if steps_block_m:
        step_items = re.findall(r'[-*]\s+(?:`?step\s+)?`?([a-zA-Z0-9_\-]+)`?', steps_block_m.group(1))
        for s in step_items:
            clean_s = _sanitize_id(s)
            if clean_s and clean_s not in test_steps and clean_s.lower() != "step":
                test_steps.append(clean_s)

    for s_stmt in re.findall(r'\bstep\s+([a-zA-Z0-9_]+);?', body):
        if s_stmt not in test_steps:
            test_steps.append(s_stmt)

    if tc_name:
        if SysMLTestCaseDef:
            test_cases.append(SysMLTestCaseDef(
                name=tc_name,
                subject_part=subject_part,
                verified_requirements=clean_reqs,
                objective=objective,
                test_steps=test_steps,
                doc=objective or f"Verification test case for {tc_name}"
            ))
        else:
            test_cases.append({
                "name": tc_name,
                "subject_part": subject_part,
                "verified_requirements": clean_reqs,
                "objective": objective,
                "test_steps": test_steps,
                "doc": objective
            })

    return interactions, test_cases


def _parse_parameter_string(param_str: str) -> Tuple[List[Any], List[Any], List[Any]]:
    """
    Parses parameter signatures into (in_params, out_params, all_params).
    Supports directional parameters (`in targetHeading : Float`, `Float in_targetHeading`, etc.).
    """
    in_params = []
    out_params = []
    all_params = []
    if not param_str or not param_str.strip():
        return in_params, out_params, all_params

    raw_items = [p.strip() for p in param_str.split(',') if p.strip()]
    for raw in raw_items:
        tokens = raw.split()
        direction = "in"
        p_name = ""
        p_type = "String"

        if tokens[0] in ("in", "out", "inout"):
            direction = tokens[0]
            p_name = tokens[1].rstrip(':') if len(tokens) > 1 else "param"
            if len(tokens) >= 4 and tokens[2] == ':':
                p_type = tokens[3]
            elif len(tokens) >= 3:
                p_type = tokens[2]
        elif len(tokens) >= 2:
            if tokens[1] == ':' and len(tokens) >= 3:
                p_name = tokens[0]
                p_type = tokens[2]
            elif tokens[0].endswith(':'):
                p_name = tokens[0].rstrip(':')
                p_type = tokens[1]
            else:
                p_type = tokens[0]
                p_name = tokens[1]
                if p_name.startswith('in_') or p_name.startswith('in'):
                    direction = "in"
                    p_name = re.sub(r'^in_?', '', p_name)
                elif p_name.startswith('out_') or p_name.startswith('out'):
                    direction = "out"
                    p_name = re.sub(r'^out_?', '', p_name)
        elif len(tokens) == 1:
            p_name = tokens[0]

        if AttributeDef:
            attr = AttributeDef(name=p_name, type_name=p_type, default_value=direction)
        else:
            attr = {"name": p_name, "type_name": p_type, "default_value": direction}
        all_params.append(attr)
        if direction == "out":
            out_params.append(attr)
        else:
            in_params.append(attr)

    return in_params, out_params, all_params


def extract_features_from_markdown(content: str, filename: str = "") -> List[Any]:
    """
    Parses Feature markdown specification:
    Extracts methods, action def operations, typed parameter signatures (`in`, `out`, types),
    and validation constraints, constructing SysMLOperationDef, ActionDef, and PartDef AST nodes.
    """
    fm, body = _parse_frontmatter(content)
    parts = []

    # Part name derivation
    part_name = ""
    if fm.get("schema_containers"):
        sc = fm["schema_containers"]
        if isinstance(sc, list) and sc:
            path_val = sc[0].get("path", "") if isinstance(sc[0], dict) else str(sc[0])
            part_name = path_val.split('/')[-1].split(':')[-1]
    elif fm.get("part"):
        part_name = str(fm["part"])
    elif fm.get("part_def"):
        part_name = str(fm["part_def"])

    if not part_name:
        class_m = re.search(r'class\s+([a-zA-Z0-9_]+)\s*\{', body)
        if class_m:
            part_name = class_m.group(1)
        elif fm.get("title"):
            part_name = _to_pascal_case(str(fm["title"]))
        elif filename:
            base = os.path.splitext(os.path.basename(filename))[0]
            part_name = _to_pascal_case(base)
        else:
            part_name = "FeatureComponent"

    part_name = _sanitize_id(part_name)
    doc = str(fm.get("title", "")) or f"Feature specification for {part_name}"

    attributes = []
    actions = []
    operations = []
    constraints = []

    # 1. Mermaid Class Diagram
    cd_match = re.search(r'```mermaid\s*\n\s*classDiagram(.*?)(?=```|\Z)', body, re.DOTALL)
    if cd_match:
        cd_text = cd_match.group(1)
        for line in cd_text.splitlines():
            line = line.strip()
            if not line or line.startswith('%%') or '<<' in line or 'class ' in line:
                continue
            clean_line = re.sub(r'^[+\-#~]\s*', '', line)

            if '(' in clean_line and ')' in clean_line:
                m_match = re.match(r'(?:([a-zA-Z0-9_<>:]+)\s+)?([a-zA-Z0-9_]+)\s*\(([^)]*)\)', clean_line)
                if m_match:
                    ret_type = m_match.group(1)
                    m_name = m_match.group(2)
                    params_raw = m_match.group(3)
                    in_p, out_p, all_p = _parse_parameter_string(params_raw)

                    if out_p or ret_type in (None, "void", "Boolean", "bool"):
                        if not any(getattr(a, "name", "") == m_name for a in actions):
                            if ActionDef:
                                actions.append(ActionDef(name=m_name, in_params=in_p, out_params=out_p))
                            else:
                                actions.append({"name": m_name, "in_params": in_p, "out_params": out_p})
                    else:
                        if not any(getattr(o, "name", "") == m_name for o in operations):
                            if SysMLOperationDef:
                                operations.append(SysMLOperationDef(
                                    name=m_name,
                                    return_type=ret_type,
                                    parameters=all_p
                                ))
                            else:
                                operations.append({"name": m_name, "return_type": ret_type, "parameters": all_p})
            else:
                attr_m = re.match(r'(?:([a-zA-Z0-9_<>:]+)\s+)?([a-zA-Z0-9_]+)(?:\s*[:=]\s*([a-zA-Z0-9_<>:]+))?', clean_line)
                if attr_m:
                    t1 = attr_m.group(1)
                    n1 = attr_m.group(2)
                    t2 = attr_m.group(3)
                    a_type = t2 or t1 or "String"
                    a_name = n1
                    if a_name and not any(getattr(a, "name", "") == a_name for a in attributes):
                        if AttributeDef:
                            attributes.append(AttributeDef(name=a_name, type_name=a_type))
                        else:
                            attributes.append({"name": a_name, "type_name": a_type})

    # 2. Logical Operations section
    ops_sec_m = re.search(r'#{1,4}\s+(?:\d+(?:\.\d+)*\.?\s+)?Logical\s+Operations\s*(?:&|and)?\s*Interface\s+Messages\b(.*?)(?=\n#{1,4}\s+|\Z)', body, re.DOTALL | re.IGNORECASE)
    if ops_sec_m:
        sec_text = ops_sec_m.group(1)
        for item in re.finditer(r'[-*]\s+`?[+\-#~]?\s*([a-zA-Z0-9_]+)\s*\(([^)]*)\)(?:\s*:\s*([a-zA-Z0-9_<>:]+))?`?\s*(?::\s*([^\n\r]+))?', sec_text):
            m_name = item.group(1)
            params_raw = item.group(2)
            ret_type = item.group(3)
            m_doc = item.group(4) or ""
            in_p, out_p, all_p = _parse_parameter_string(params_raw)

            if out_p or ret_type in (None, "void", "Boolean", "bool"):
                existing_act = next((a for a in actions if getattr(a, "name", "") == m_name), None)
                if not existing_act:
                    if ActionDef:
                        actions.append(ActionDef(name=m_name, in_params=in_p, out_params=out_p, doc=m_doc))
                    else:
                        actions.append({"name": m_name, "in_params": in_p, "out_params": out_p, "doc": m_doc})
                else:
                    if hasattr(existing_act, "in_params") and not existing_act.in_params and in_p:
                        existing_act.in_params = in_p
                    if hasattr(existing_act, "out_params") and not existing_act.out_params and out_p:
                        existing_act.out_params = out_p
                    if hasattr(existing_act, "doc") and not existing_act.doc and m_doc:
                        existing_act.doc = m_doc
            else:
                existing_op = next((o for o in operations if getattr(o, "name", "") == m_name), None)
                if not existing_op:
                    if SysMLOperationDef:
                        operations.append(SysMLOperationDef(
                            name=m_name,
                            return_type=ret_type,
                            parameters=all_p,
                            doc=m_doc
                        ))
                    else:
                        operations.append({"name": m_name, "return_type": ret_type, "parameters": all_p, "doc": m_doc})
                else:
                    if hasattr(existing_op, "parameters") and not existing_op.parameters and all_p:
                        existing_op.parameters = all_p
                    if hasattr(existing_op, "return_type") and not existing_op.return_type and ret_type:
                        existing_op.return_type = ret_type
                    if hasattr(existing_op, "doc") and not existing_op.doc and m_doc:
                        existing_op.doc = m_doc

    # 3. Validation & Constraints section
    val_sec_m = re.search(r'#{1,4}\s+(?:\d+(?:\.\d+)*\.?\s+)?Validation\s+(?:&|and)\s+Constraints\b(.*?)(?=\n#{1,4}\s+|\Z)', body, re.DOTALL | re.IGNORECASE)
    if val_sec_m:
        sec_text = val_sec_m.group(1)
        for item in re.finditer(r'[-*]\s+`?([a-zA-Z0-9_]+)`?\s*[:=]\s*([^\n\r]+)', sec_text):
            c_name = item.group(1).strip()
            c_expr = item.group(2).strip().rstrip(';')
            is_assert = "assert" in c_name.lower() or any(op in c_expr for op in ("<=", ">=", "==", "!=", "<", ">"))
            if not any(getattr(c, "name", "") == c_name for c in constraints):
                if SysMLConstraintDef:
                    constraints.append(SysMLConstraintDef(
                        name=c_name,
                        expression=c_expr,
                        is_assertion=is_assert,
                        doc=f"Feature validation constraint {c_name}"
                    ))
                else:
                    constraints.append({
                        "name": c_name,
                        "expression": c_expr,
                        "is_assertion": is_assert,
                        "doc": f"Feature validation constraint {c_name}"
                    })

    if PartDef:
        part_obj = PartDef(
            name=part_name,
            doc=doc,
            attributes=attributes,
            actions=actions,
            operations=operations,
            constraints=constraints
        )
    else:
        part_obj = {
            "name": part_name,
            "doc": doc,
            "attributes": attributes,
            "actions": actions,
            "operations": operations,
            "constraints": constraints
        }
    parts.append(part_obj)
    return parts


def extract_epics_from_markdown(content: str, filename: str = "") -> List[Any]:
    """
    Parses Epic markdown specifications:
    Extracts subsystem capability allocations, constructing SysMLCapabilityDef nodes.
    """
    fm, body = _parse_frontmatter(content)
    capabilities = []

    subsystem = fm.get("package") or fm.get("subsystem") or ""

    row_pattern = re.compile(
        r'\|\s*(?:\*\*)?([A-Za-z0-9_]+)(?:\*\*)?\s*\|'
        r'\s*([^|]+)\s*\|'
        r'\s*([^|\n]+)\s*\|'
    )
    for match in row_pattern.finditer(body):
        cap_name = match.group(1).strip()
        pkg_name = match.group(2).strip()
        desc = match.group(3).strip()
        if cap_name.lower() in ("capability", "capability name", "name"):
            continue
        if not any(getattr(c, "name", "") == cap_name for c in capabilities):
            if SysMLCapabilityDef:
                capabilities.append(SysMLCapabilityDef(
                    name=cap_name,
                    subsystem=pkg_name or subsystem,
                    description=desc,
                    doc=desc
                ))
            else:
                capabilities.append({
                    "name": cap_name,
                    "subsystem": pkg_name or subsystem,
                    "description": desc,
                    "doc": desc
                })

    return capabilities


# ==============================================================================
# AST MERGING & REVERSE SYNCHRONIZATION ENGINE
# ==============================================================================

def _merge_part_into_package(pkg: Any, new_part: Any) -> None:
    """Merges a PartDef into the SysMLPackage, updating matching parts in-place or adding a new part."""
    part_name = getattr(new_part, "name", "")
    if not part_name:
        return

    def _find_part(p: Any) -> Optional[Any]:
        for part in (getattr(p, "part_defs", []) or []):
            if getattr(part, "name", "") == part_name:
                return part
            for sub_p in (getattr(part, "parts", []) or []):
                if getattr(sub_p, "name", "") == part_name:
                    return sub_p
        for sub_pkg in (getattr(p, "sub_packages", []) or []):
            found = _find_part(sub_pkg)
            if found:
                return found
        return None

    existing = _find_part(pkg)
    if existing:
        # Merge attributes
        existing_attrs = {getattr(a, "name", ""): a for a in getattr(existing, "attributes", [])}
        for attr in getattr(new_part, "attributes", []):
            a_name = getattr(attr, "name", "")
            if a_name not in existing_attrs:
                existing.attributes.append(attr)
                existing_attrs[a_name] = attr
            else:
                ex_attr = existing_attrs[a_name]
                if getattr(attr, "type_name", None) and getattr(ex_attr, "type_name", "String") == "String":
                    ex_attr.type_name = attr.type_name
                if getattr(attr, "doc", None) and not getattr(ex_attr, "doc", None):
                    ex_attr.doc = attr.doc

        # Merge ports
        existing_ports = {getattr(p, "name", ""): p for p in getattr(existing, "ports", [])}
        for port in getattr(new_part, "ports", []):
            p_name = getattr(port, "name", "")
            if p_name not in existing_ports:
                existing.ports.append(port)
                existing_ports[p_name] = port

        # Merge actions
        existing_actions = {getattr(a, "name", ""): a for a in getattr(existing, "actions", [])}
        for act in getattr(new_part, "actions", []):
            act_name = getattr(act, "name", "")
            if act_name not in existing_actions:
                existing.actions.append(act)
                existing_actions[act_name] = act
            else:
                ex_act = existing_actions[act_name]
                if hasattr(ex_act, "in_params") and not ex_act.in_params and getattr(act, "in_params", None):
                    ex_act.in_params = act.in_params
                if hasattr(ex_act, "out_params") and not ex_act.out_params and getattr(act, "out_params", None):
                    ex_act.out_params = act.out_params
                if hasattr(ex_act, "doc") and not ex_act.doc and getattr(act, "doc", ""):
                    ex_act.doc = act.doc

        # Merge operations
        existing_ops = {getattr(o, "name", ""): o for o in getattr(existing, "operations", [])}
        for op in getattr(new_part, "operations", []):
            op_name = getattr(op, "name", "")
            if op_name not in existing_ops:
                existing.operations.append(op)
                existing_ops[op_name] = op
            else:
                ex_op = existing_ops[op_name]
                if hasattr(ex_op, "parameters") and not ex_op.parameters and getattr(op, "parameters", None):
                    ex_op.parameters = op.parameters
                if hasattr(ex_op, "return_type") and not ex_op.return_type and getattr(op, "return_type", None):
                    ex_op.return_type = op.return_type
                if hasattr(ex_op, "doc") and not ex_op.doc and getattr(op, "doc", ""):
                    ex_op.doc = op.doc

        # Merge constraints
        existing_cons = {getattr(c, "name", ""): c for c in getattr(existing, "constraints", [])}
        for con in getattr(new_part, "constraints", []):
            con_name = getattr(con, "name", "")
            if con_name not in existing_cons:
                existing.constraints.append(con)
                existing_cons[con_name] = con
            else:
                ex_con = existing_cons[con_name]
                if hasattr(ex_con, "expression") and not ex_con.expression and getattr(con, "expression", ""):
                    ex_con.expression = con.expression
                if hasattr(ex_con, "doc") and not ex_con.doc and getattr(con, "doc", ""):
                    ex_con.doc = con.doc

    else:
        pkg.part_defs.append(new_part)


def _merge_use_case_into_package(pkg: Any, new_uc: Any) -> None:
    uc_name = getattr(new_uc, "name", "")
    if not uc_name:
        return

    def _find_uc(p: Any) -> Optional[Any]:
        for uc in getattr(p, "use_case_defs", []) or []:
            if getattr(uc, "name", "") == uc_name:
                return uc
        for sub in getattr(p, "sub_packages", []) or []:
            found = _find_uc(sub)
            if found:
                return found
        return None

    existing = _find_uc(pkg)
    if not existing:
        pkg.use_case_defs.append(new_uc)
    else:
        if hasattr(existing, "subject") and not existing.subject and getattr(new_uc, "subject", ""):
            existing.subject = new_uc.subject
        if hasattr(existing, "actor") and not existing.actor and getattr(new_uc, "actor", ""):
            existing.actor = new_uc.actor
        if hasattr(existing, "objective") and not existing.objective and getattr(new_uc, "objective", ""):
            existing.objective = new_uc.objective
        if hasattr(existing, "includes"):
            for inc in (getattr(new_uc, "includes", []) or []):
                if inc not in existing.includes:
                    existing.includes.append(inc)
        if hasattr(existing, "extends"):
            for ext in (getattr(new_uc, "extends", []) or []):
                if ext not in existing.extends:
                    existing.extends.append(ext)


def _merge_interaction_into_package(pkg: Any, new_inter: Any) -> None:
    inter_name = getattr(new_inter, "name", "")
    if not inter_name:
        return

    def _find_inter(p: Any) -> Optional[Any]:
        for i in getattr(p, "interaction_defs", []) or []:
            if getattr(i, "name", "") == inter_name:
                return i
        for sub in getattr(p, "sub_packages", []) or []:
            found = _find_inter(sub)
            if found:
                return found
        return None

    existing = _find_inter(pkg)
    if not existing:
        pkg.interaction_defs.append(new_inter)
    else:
        if hasattr(existing, "lifelines"):
            for ll in (getattr(new_inter, "lifelines", []) or []):
                if ll not in existing.lifelines:
                    existing.lifelines.append(ll)
        if hasattr(existing, "messages"):
            for msg in (getattr(new_inter, "messages", []) or []):
                if msg not in existing.messages:
                    existing.messages.append(msg)
        if hasattr(existing, "triggers"):
            for trg in (getattr(new_inter, "triggers", []) or []):
                if trg not in existing.triggers:
                    existing.triggers.append(trg)


def _merge_test_case_into_package(pkg: Any, new_tc: Any) -> None:
    tc_name = getattr(new_tc, "name", "")
    if not tc_name:
        return

    def _find_tc(p: Any) -> Optional[Any]:
        for t in getattr(p, "test_case_defs", []) or []:
            if getattr(t, "name", "") == tc_name:
                return t
        for sub in getattr(p, "sub_packages", []) or []:
            found = _find_tc(sub)
            if found:
                return found
        return None

    existing = _find_tc(pkg)
    if not existing:
        pkg.test_case_defs.append(new_tc)
    else:
        if hasattr(existing, "subject_part") and not existing.subject_part and getattr(new_tc, "subject_part", ""):
            existing.subject_part = new_tc.subject_part
        if hasattr(existing, "verified_requirements"):
            for req in (getattr(new_tc, "verified_requirements", []) or []):
                if req not in existing.verified_requirements:
                    existing.verified_requirements.append(req)
        if hasattr(existing, "objective") and not existing.objective and getattr(new_tc, "objective", ""):
            existing.objective = new_tc.objective
        if hasattr(existing, "test_steps"):
            for step in (getattr(new_tc, "test_steps", []) or []):
                if step not in existing.test_steps:
                    existing.test_steps.append(step)


def _merge_constraint_into_package(pkg: Any, new_con: Any) -> None:
    con_name = getattr(new_con, "name", "")
    if not con_name:
        return

    def _find_con(p: Any) -> Optional[Any]:
        for c in getattr(p, "constraint_defs", []) or []:
            if getattr(c, "name", "") == con_name:
                return c
        for sub in getattr(p, "sub_packages", []) or []:
            found = _find_con(sub)
            if found:
                return found
        return None

    existing = _find_con(pkg)
    if not existing:
        pkg.constraint_defs.append(new_con)
    else:
        if hasattr(existing, "expression") and not existing.expression and getattr(new_con, "expression", ""):
            existing.expression = new_con.expression
        if hasattr(existing, "doc") and not existing.doc and getattr(new_con, "doc", ""):
            existing.doc = new_con.doc


def _merge_capability_into_package(pkg: Any, new_cap: Any) -> None:
    cap_name = getattr(new_cap, "name", "")
    if not cap_name:
        return

    def _find_cap(p: Any) -> Optional[Any]:
        for c in getattr(p, "capability_defs", []) or []:
            if getattr(c, "name", "") == cap_name:
                return c
        for sub in getattr(p, "sub_packages", []) or []:
            found = _find_cap(sub)
            if found:
                return found
        return None

    existing = _find_cap(pkg)
    if not existing:
        pkg.capability_defs.append(new_cap)
    else:
        if hasattr(existing, "subsystem") and not existing.subsystem and getattr(new_cap, "subsystem", ""):
            existing.subsystem = new_cap.subsystem
        if hasattr(existing, "description") and not existing.description and getattr(new_cap, "description", ""):
            existing.description = new_cap.description
        if hasattr(existing, "doc") and not existing.doc and getattr(new_cap, "doc", ""):
            existing.doc = new_cap.doc


def _atomic_write_file(filepath: str, content: str) -> None:
    """Atomically writes string content to target file using NamedTemporaryFile and os.replace."""
    abs_path = os.path.abspath(filepath)
    dir_name = os.path.dirname(abs_path)
    os.makedirs(dir_name, exist_ok=True)
    with tempfile.NamedTemporaryFile("w", dir=dir_name, encoding="utf-8", delete=False) as tf:
        temp_name = tf.name
        tf.write(content)
        tf.flush()
        os.fsync(tf.fileno())
    os.replace(temp_name, abs_path)


def _atomic_write_json(filepath: str, data: Any, indent: int = 2) -> None:
    """Atomically writes JSON data to target file using NamedTemporaryFile and os.replace."""
    abs_path = os.path.abspath(filepath)
    dir_name = os.path.dirname(abs_path)
    os.makedirs(dir_name, exist_ok=True)
    with tempfile.NamedTemporaryFile("w", dir=dir_name, encoding="utf-8", delete=False) as tf:
        temp_name = tf.name
        json.dump(data, tf, indent=indent)
        tf.write("\n")
        tf.flush()
        os.fsync(tf.fileno())
    os.replace(temp_name, abs_path)


def reverse_sync_specs_to_sysml(
    docs_dir: str = "docs",
    schema_path: Optional[str] = None,
    output_path: Optional[str] = None,
    digest_path: Optional[str] = None,
    allow_schema_overwrite: bool = False,
) -> Tuple[Any, Dict[str, Any]]:
    """
    Executes Closed-Loop Reverse Synchronization from markdown specification corpus
    into the canonical SysML v2 AST Single Source of Truth (.pipeline/schema.sysml).

    Parses:
    - docs/use-cases/UC-*.md -> UseCaseDef
    - docs/user-stories/US-*.md -> SysMLInteractionDef & SysMLTestCaseDef
    - docs/features/FEAT-*.md -> PartDef, SysMLOperationDef, ActionDef, SysMLConstraintDef
    - docs/epics/EPIC-*.md -> SysMLCapabilityDef
    - docs/safety/STPA_MATRIX.md -> SysMLConstraintDef (assert constraint)

    Merges all extracted constructs non-destructively into matching packages/parts,
    serializes the updated SysML v2 textual model, and recomputes the cryptographic schema digest.

    Parameters:
        docs_dir: Path to directory containing markdown specifications.
        schema_path: Optional path to base input SysML schema model.
        output_path: Destination path for compiled SysML model (default: .pipeline/schema.sysml).
        digest_path: Destination path for schema digest JSON (default: .pipeline/schema-digest.json).
        allow_schema_overwrite: If False (default), prevents modifying or overwriting schema_path in place.
    """
    if not output_path:
        output_path = os.path.join(PROJECT_ROOT, ".pipeline", "schema.sysml")
    if not digest_path:
        digest_path = os.path.join(PROJECT_ROOT, ".pipeline", "schema-digest.json")

    # Load existing package model from schema_path if provided
    pkg = None
    if schema_path:
        if not os.path.exists(schema_path):
            raise FileNotFoundError(f"Base schema file not found: {schema_path}")
        if not allow_schema_overwrite and os.path.abspath(schema_path) == os.path.abspath(output_path):
            raise RuntimeError(
                f"In-place overwrite of base input schema '{schema_path}' is prohibited when allow_schema_overwrite=False."
            )
        if SysMLParser is None:
            raise RuntimeError("SysMLParser is not available to parse base schema.")
        try:
            pkg = SysMLParser.parse_file(schema_path)
        except Exception as exc:
            raise RuntimeError(f"Failed to parse base schema '{schema_path}': {exc}") from exc
        if pkg is None:
            raise RuntimeError(f"Failed to parse base schema '{schema_path}': parser returned None.")
    elif output_path and os.path.exists(output_path):
        if SysMLParser:
            try:
                pkg = SysMLParser.parse_file(output_path)
            except Exception:
                pkg = None

    if pkg is None:
        root_name = "AutonomousUAS_SSOT"
        if schema_path:
            root_name = os.path.splitext(os.path.basename(schema_path))[0]
        if SysMLPackage:
            pkg = SysMLPackage(name=root_name, doc="Single Source of Truth for Autonomous UAS Infrastructure Safety")
        else:
            pkg = {
                "name": root_name,
                "part_defs": [],
                "constraint_defs": [],
                "use_case_defs": [],
                "interaction_defs": [],
                "test_case_defs": [],
                "capability_defs": [],
            }

    # Resolve docs directory path
    if not os.path.isabs(docs_dir):
        if os.path.exists(docs_dir):
            resolved_docs_dir = os.path.abspath(docs_dir)
        else:
            resolved_docs_dir = os.path.join(PROJECT_ROOT, docs_dir)
    else:
        resolved_docs_dir = docs_dir

    if not os.path.exists(resolved_docs_dir):
        print(f"[SysML v2 Reverse-Sync] Warning: Docs directory '{resolved_docs_dir}' does not exist.")
    else:
        for root, _, files in os.walk(resolved_docs_dir):
            for file in sorted(files):
                if not file.endswith(".md") or file.startswith("."):
                    continue
                filepath = os.path.join(root, file)
                try:
                    with open(filepath, "r", encoding="utf-8") as f:
                        content = f.read()
                except Exception as exc:
                    print(f"[SysML v2 Reverse-Sync] Warning: Failed to read '{filepath}': {exc}")
                    continue

                rel_dir = os.path.relpath(root, resolved_docs_dir).lower()

                # Use Cases
                if "use-cases" in rel_dir or "use_cases" in rel_dir or file.lower().startswith("uc-") or file.lower().startswith("uc_"):
                    uc_list = extract_use_cases_from_markdown(content, file)
                    for uc in uc_list:
                        _merge_use_case_into_package(pkg, uc)

                # User Stories
                elif "user-stories" in rel_dir or "user_stories" in rel_dir or file.lower().startswith("us-") or file.lower().startswith("us_"):
                    inters, tcs = extract_user_story_ast(content, file)
                    for inter in inters:
                        _merge_interaction_into_package(pkg, inter)
                    for tc in tcs:
                        _merge_test_case_into_package(pkg, tc)

                # Features
                elif "features" in rel_dir or file.lower().startswith("feat-") or file.lower().startswith("feat_"):
                    feat_parts = extract_features_from_markdown(content, file)
                    for part in feat_parts:
                        _merge_part_into_package(pkg, part)

                # Epics
                elif "epics" in rel_dir or file.lower().startswith("epic-") or file.lower().startswith("epic_"):
                    ep_caps = extract_epics_from_markdown(content, file)
                    for cap in ep_caps:
                        _merge_capability_into_package(pkg, cap)

                # Safety & STPA
                if "safety" in rel_dir or "UCA-" in content or "FMECA-" in content:
                    ucas = parse_stpa_ucas(content)
                    for u in ucas:
                        _merge_constraint_into_package(pkg, compile_uca_to_constraint(u))
                    fmecas = parse_fmeca_modes(content)
                    for fm_item in fmecas:
                        _merge_constraint_into_package(pkg, compile_fmeca_to_constraint(fm_item))

    # Serialize SysML textual model with atomic write semantics
    sysml_text = pkg.to_sysml() if hasattr(pkg, "to_sysml") else ""
    _atomic_write_file(output_path, sysml_text)

    if allow_schema_overwrite and schema_path and os.path.exists(schema_path) and os.path.abspath(schema_path) != os.path.abspath(output_path):
        _atomic_write_file(schema_path, sysml_text)

    # Compute digest and node counts
    with open(output_path, "rb") as f:
        content_bytes = f.read()
    sha256_hash = hashlib.sha256(content_bytes).hexdigest()
    total_lines = len(content_bytes.decode("utf-8", errors="replace").splitlines())
    node_counts = pkg.node_counts() if hasattr(pkg, "node_counts") else {}
    schema_nodes = pkg.get_all_node_names() if hasattr(pkg, "get_all_node_names") else []

    digest_data = {
        "sha256": sha256_hash,
        "total_lines": total_lines,
        "node_counts": node_counts,
        "schema_nodes": schema_nodes
    }

    # Write digest JSON with atomic write semantics
    _atomic_write_json(digest_path, digest_data, indent=2)

    print(f"[SysML v2 Reverse-Sync] Successfully synchronized specs from '{resolved_docs_dir}' -> '{output_path}'")
    print(f"[SysML v2 Reverse-Sync] Schema digest updated at '{digest_path}' (SHA-256: {sha256_hash[:12]}...)")

    return pkg, digest_data


def parse_sysml(content: str) -> Dict[str, List[str]]:
    """
    Parses SysML v2 textual model content and returns a dictionary of extracted
    AST node names across all 6 core constructs and architectural elements.
    """
    ast: Dict[str, List[str]] = {
        "packages": [],
        "part_defs": [],
        "attribute_defs": [],
        "port_defs": [],
        "action_defs": [],
        "capability_defs": [],
        "operation_defs": [],
        "interaction_defs": [],
        "constraint_defs": [],
        "test_case_defs": [],
        "requirement_defs": [],
        "state_defs": [],
        "use_case_defs": [],
        "item_defs": []
    }

    if SysMLParser is not None:
        try:
            pkg = SysMLParser.parse_text(content)

            def _extract_from_pkg(p: SysMLPackage):
                if p.name and p.name not in ast["packages"] and p.name != "SysML_Model":
                    ast["packages"].append(p.name)
                for a in p.attribute_defs:
                    if a.name not in ast["attribute_defs"]:
                        ast["attribute_defs"].append(a.name)
                for pt in p.port_defs:
                    if pt.name not in ast["port_defs"]:
                        ast["port_defs"].append(pt.name)
                for ac in p.action_defs:
                    if ac.name not in ast["action_defs"]:
                        ast["action_defs"].append(ac.name)
                for cap in p.capability_defs:
                    if cap.name not in ast["capability_defs"]:
                        ast["capability_defs"].append(cap.name)
                for op in p.operation_defs:
                    if op.name not in ast["operation_defs"]:
                        ast["operation_defs"].append(op.name)
                for it in p.interaction_defs:
                    if it.name not in ast["interaction_defs"]:
                        ast["interaction_defs"].append(it.name)
                for c in p.constraint_defs:
                    if c.name not in ast["constraint_defs"]:
                        ast["constraint_defs"].append(c.name)
                for tc in p.test_case_defs:
                    if tc.name not in ast["test_case_defs"]:
                        ast["test_case_defs"].append(tc.name)
                for r in p.requirement_defs:
                    if r.name not in ast["requirement_defs"]:
                        ast["requirement_defs"].append(r.name)
                for s in p.state_defs:
                    if s.name not in ast["state_defs"]:
                        ast["state_defs"].append(s.name)
                for uc in p.use_case_defs:
                    if uc.name not in ast["use_case_defs"]:
                        ast["use_case_defs"].append(uc.name)
                for itm in p.item_defs:
                    if itm.name not in ast["item_defs"]:
                        ast["item_defs"].append(itm.name)

                for part in p.part_defs:
                    _extract_from_part(part)

                for sub in p.sub_packages:
                    _extract_from_pkg(sub)

            def _extract_from_part(part):
                if part.name not in ast["part_defs"]:
                    ast["part_defs"].append(part.name)
                for a in part.attributes:
                    if a.name not in ast["attribute_defs"]:
                        ast["attribute_defs"].append(a.name)
                for pt in part.ports:
                    if pt.name not in ast["port_defs"]:
                        ast["port_defs"].append(pt.name)
                for ac in part.actions:
                    if ac.name not in ast["action_defs"]:
                        ast["action_defs"].append(ac.name)
                for op in part.operations:
                    if op.name not in ast["operation_defs"]:
                        ast["operation_defs"].append(op.name)
                for cap in part.capabilities:
                    if cap.name not in ast["capability_defs"]:
                        ast["capability_defs"].append(cap.name)
                for it in part.interactions:
                    if it.name not in ast["interaction_defs"]:
                        ast["interaction_defs"].append(it.name)
                for c in part.constraints:
                    if c.name not in ast["constraint_defs"]:
                        ast["constraint_defs"].append(c.name)
                for tc in part.test_cases:
                    if tc.name not in ast["test_case_defs"]:
                        ast["test_case_defs"].append(tc.name)
                for r in part.requirements:
                    if r.name not in ast["requirement_defs"]:
                        ast["requirement_defs"].append(r.name)
                for s in part.states:
                    if s.name not in ast["state_defs"]:
                        ast["state_defs"].append(s.name)
                for uc in part.use_cases:
                    if uc.name not in ast["use_case_defs"]:
                        ast["use_case_defs"].append(uc.name)
                for itm in part.item_defs:
                    if itm.name not in ast["item_defs"]:
                        ast["item_defs"].append(itm.name)
                for sub_part in part.parts:
                    _extract_from_part(sub_part)

            _extract_from_pkg(pkg)
            return ast
        except Exception:
            pass

    # Regex-based extraction fallback
    for match in re.finditer(r'\bpackage\s+([a-zA-Z0-9_\-\.]+)', content):
        name = match.group(1).replace('.', '_')
        if name not in ast["packages"]:
            ast["packages"].append(name)
    for match in re.finditer(r'\bpart\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["part_defs"]:
            ast["part_defs"].append(match.group(1))
    for match in re.finditer(r'\battribute\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["attribute_defs"]:
            ast["attribute_defs"].append(match.group(1))
    for match in re.finditer(r'\b(?:in|out|inout)?\s*port\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["port_defs"]:
            ast["port_defs"].append(match.group(1))
    for match in re.finditer(r'\baction\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["action_defs"]:
            ast["action_defs"].append(match.group(1))
    for match in re.finditer(r'\bcapability\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["capability_defs"]:
            ast["capability_defs"].append(match.group(1))
    for match in re.finditer(r'\b(?:operation|feature)\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["operation_defs"]:
            ast["operation_defs"].append(match.group(1))
    for match in re.finditer(r'\binteraction\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["interaction_defs"]:
            ast["interaction_defs"].append(match.group(1))
    for match in re.finditer(r'\b(?:assert\s+constraint|constraint\s+(?:def)?)\s+([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["constraint_defs"]:
            ast["constraint_defs"].append(match.group(1))
    for match in re.finditer(r'\btest\s+case\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["test_case_defs"]:
            ast["test_case_defs"].append(match.group(1))
    for match in re.finditer(r'\brequirement\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["requirement_defs"]:
            ast["requirement_defs"].append(match.group(1))
    for match in re.finditer(r'\bstate\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["state_defs"]:
            ast["state_defs"].append(match.group(1))
    for match in re.finditer(r'\buse\s+case\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["use_case_defs"]:
            ast["use_case_defs"].append(match.group(1))
    for match in re.finditer(r'\bitem\s+(?:def\s+)?([a-zA-Z0-9_]+)', content):
        if match.group(1) not in ast["item_defs"]:
            ast["item_defs"].append(match.group(1))

    return ast


def main():
    parser = argparse.ArgumentParser(
        description="SysML v2 Compiler, STPA Safety Constraints & Closed-Loop Reverse Synchronization Engine",
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("file", nargs="?", default=None, help="SysML v2 (.sysml) or STPA markdown file path")
    parser.add_argument("--reverse-sync", action="store_true", help="Execute closed-loop reverse synchronization from markdown specs to SysML v2 SSOT")
    parser.add_argument("--docs", "--docs-dir", dest="docs_dir", default="docs", help="Path to markdown specifications directory (default: docs)")
    parser.add_argument("--schema", "--schema-path", dest="schema_path", default=None, help="Path to base/input schema file (e.g. schema/DEAP_MODEL.sysml)")
    parser.add_argument("--out", "--output", dest="output_path", default=".pipeline/schema.sysml", help="Path to output .sysml SSOT file (default: .pipeline/schema.sysml)")
    parser.add_argument("--digest", "--digest-path", dest="digest_path", default=".pipeline/schema-digest.json", help="Path to output schema digest JSON (default: .pipeline/schema-digest.json)")
    parser.add_argument("--stpa", "--compile-stpa", action="store_true", help="Compile STPA hazard matrix to SysML constraint notation")
    parser.add_argument("--allow-schema-overwrite", action="store_true", default=False, help="Allow in-place overwrite of base input schema file")

    args = parser.parse_args()

    if args.reverse_sync:
        reverse_sync_specs_to_sysml(
            docs_dir=args.docs_dir,
            schema_path=args.schema_path,
            output_path=args.output_path,
            digest_path=args.digest_path,
            allow_schema_overwrite=args.allow_schema_overwrite,
        )
        return

    target_file = args.file
    if not target_file:
        parser.print_help()
        sys.exit(1)

    if not os.path.exists(target_file):
        print(f"Error: File not found: {target_file}")
        sys.exit(1)

    with open(target_file, 'r', encoding='utf-8') as f:
        content = f.read()

    if args.stpa or ("UCA-" in content and "package " not in content):
        print(compile_stpa_to_sysml(content))
    else:
        print(json.dumps(parse_sysml(content), indent=2))


if __name__ == '__main__':
    main()
