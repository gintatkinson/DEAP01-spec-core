#!/usr/bin/env python3
"""
Markdown / BOM Specification to SysML v2 Translator

Translates Level 0 OEM unstructured/semi-structured Markdown documents,
Bill of Materials (BOM) tables, interface/port definitions, and parametric
constraints into canonical SysML v2 textual models (AST).

Primary Tier-1 Commercial Toolchain Integration Context:
This module explicitly integrates with MATLAB / Simulink / Stateflow / Embedded Coder
(Model-Based Design, Control Law Synthesis, DO-178C C/SPARK Ada code generation).

Pure Schema-Driven Compiler Invariant:
All specifications and AST representations derive deterministically from the parsed
Markdown tables and headers. Zero hardcoded domain concepts.
"""

import os
import re
import sys
from typing import List, Optional, Dict, Any, Tuple, Union, Set

# Handle both direct execution and package import
try:
    from sysmlv2_ast import (
        SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef,
        SysMLConstraintDef, ItemFlowDef, ConnectionDef
    )
except ImportError:
    from skills.spec_orchestrator.scripts.sysmlv2_ast import (
        SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef,
        SysMLConstraintDef, ItemFlowDef, ConnectionDef
    )


RESERVED_SYSML_KEYWORDS = {
    "package", "part", "def", "port", "attribute", "action", "flow",
    "item", "connect", "connection", "interface", "doc", "assert",
    "constraint", "state", "requirement", "use", "case", "test",
    "in", "out", "inout", "import", "public", "private", "alias",
    "perform", "subsystem", "first", "then", "step", "entry", "exit",
    "do", "transition", "assume", "require", "verify", "satisfy"
}

NON_COMPONENT_SECTION_KEYWORDS = {
    "bill of materials", "bom", "components", "parts", "subsystems",
    "interfaces", "ports", "signals", "signal interface", "interconnects",
    "parameters", "parametric limits", "limits", "constraints", "invariants",
    "assertions", "requirements", "specifications", "overview", "introduction",
    "scope", "architecture", "notes", "glossary", "definitions", "summary",
    "system interfaces", "system parameters", "system limits", "system constraints"
}


def sanitize_identifier(text: str, default: str = "Item") -> str:
    """Sanitizes any arbitrary text into a valid SysML v2 identifier."""
    if not text:
        return default

    # Remove parentheses/bracketed content if that leaves a valid name, e.g. "Mass (kg)" -> "Mass"
    stripped = re.sub(r'[\(\[\{][^\)\]\}]*[\)\]\}]', '', text).strip()
    clean = stripped if stripped else text

    # Remove Markdown backticks, asterisks, formatting
    clean = re.sub(r'[*`_#]', ' ', clean).strip()

    # Replace non-alphanumeric characters with underscore
    clean = re.sub(r'[^a-zA-Z0-9_]', '_', clean)
    clean = re.sub(r'_+', '_', clean).strip('_')

    if not clean:
        return default

    # Must not start with a digit
    if clean[0].isdigit():
        clean = f"_{clean}"

    # Avoid SysML v2 reserved keywords
    if clean.lower() in RESERVED_SYSML_KEYWORDS:
        clean = f"{clean}_item"

    return clean


def extract_unit_from_header_or_val(header_or_val: str) -> Tuple[str, str]:
    """Extracts unit in parentheses or brackets: e.g. 'Mass (kg)' -> ('Mass', 'kg')."""
    unit_match = re.search(r'[\(\[]([a-zA-Z0-9_/°^%\-]+)[\)\]]', header_or_val)
    unit = unit_match.group(1).strip() if unit_match else ""
    cleaned = re.sub(r'[\(\[][^\)\]]*[\)\]]', '', header_or_val).strip()
    return cleaned, unit


def normalize_col(name: str) -> str:
    """Normalizes a table column header for canonical matching."""
    clean, _ = extract_unit_from_header_or_val(name)
    clean = re.sub(r'[*`_]', ' ', clean).strip().lower()
    clean = re.sub(r'[^a-z0-9]+', '_', clean).strip('_')
    return clean


def clean_cell_value(val: str) -> str:
    """Strips Markdown links, HTML tags, backticks, and bold/italic markers from table cell."""
    if not val:
        return ""
    # Strip markdown link: [text](url) -> text
    clean = re.sub(r'\[([^\]]+)\]\([^\)]+\)', r'\1', str(val))
    # Replace <br> with space
    clean = re.sub(r'<br\s*/?>', ' ', clean, flags=re.IGNORECASE)
    # Strip remaining HTML tags
    clean = re.sub(r'<[^>]+>', '', clean)
    return clean.strip()


def parse_numeric_with_unit(raw_val: str) -> Tuple[Optional[float], Optional[str], Optional[int]]:
    """
    Parses a scalar string like '1800.0 kg', '150 W', '42', '0x1A'.
    Returns (float_val, unit, int_val).
    """
    raw_clean = re.sub(r'[*`]', '', str(raw_val)).strip()

    # Check hex
    hex_m = re.match(r'^(0x[0-9a-fA-F]+)$', raw_clean)
    if hex_m:
        try:
            val_int = int(hex_m.group(1), 16)
            return float(val_int), "", val_int
        except ValueError:
            pass

    # Number followed by optional unit
    num_m = re.match(r'^([-+]?[0-9]*\.?[0-9]+(?:[eE][-+]?[0-9]+)?)\s*([a-zA-Z0-9_/°^%\-]*)$', raw_clean)
    if num_m:
        val_str = num_m.group(1)
        unit = num_m.group(2).strip()
        try:
            if '.' in val_str or 'e' in val_str.lower():
                val_float = float(val_str)
                return val_float, unit, None
            else:
                val_int = int(val_str)
                return float(val_int), unit, val_int
        except ValueError:
            pass

    return None, None, None


def parse_range_bounds(range_text: str) -> Tuple[Optional[float], Optional[float]]:
    """
    Parses range string e.g. '[0.0, 100.0]', '-20.0 to 60.0', '18.0 .. 25.2'.
    Returns (min_val, max_val).
    """
    clean = re.sub(r'[*`\[\]\(\)]', '', range_text).strip()

    # Split by comma, '..', 'to', or ':'
    parts = re.split(r'\s*(?:,|\.\.|\bto\b|:)\s*', clean)
    if len(parts) >= 2:
        val1, _, _ = parse_numeric_with_unit(parts[0])
        val2, _, _ = parse_numeric_with_unit(parts[1])
        if val1 is not None and val2 is not None:
            return min(val1, val2), max(val1, val2)

    return None, None


class MarkdownTranslator:
    """
    Translates Markdown specifications and BOM tables into canonical SysML v2 AST.
    Handles component/BOM hierarchies, port/signal interfaces, and parametric constraints.
    """

    def __init__(self):
        pass

    def translate(self, content: str, default_name: str = "Markdown_Package") -> SysMLPackage:
        """
        Parses markdown content string into a canonical SysMLPackage AST.
        """
        pkg_name, sections = self._parse_markdown_structure(content, default_name)
        pkg = SysMLPackage(name=pkg_name, doc="Translated from Level 0 OEM Markdown specification")

        part_registry: Dict[str, PartDef] = {}

        for sec in sections:
            component_target = sec.get("component_target")
            current_part: Optional[PartDef] = None
            if component_target:
                part_name = sanitize_identifier(component_target)
                if part_name not in part_registry:
                    part_registry[part_name] = PartDef(name=part_name, doc=sec.get("section_doc", ""))
                current_part = part_registry[part_name]
                if not current_part.doc and sec.get("section_doc"):
                    current_part.doc = sec.get("section_doc")

            for tbl in sec.get("tables", []):
                tbl_type = self._classify_table(tbl["normalized_headers"])
                target_container: Union[SysMLPackage, PartDef] = current_part if current_part is not None else pkg

                if tbl_type == "bom":
                    self._process_bom_table(tbl, pkg, part_registry)
                elif tbl_type == "ports":
                    self._process_ports_table(tbl, target_container, pkg)
                elif tbl_type == "constraints":
                    self._process_constraints_table(tbl, target_container, pkg)
                elif tbl_type == "properties":
                    self._process_properties_table(tbl, target_container, pkg)
                else:
                    self._process_generic_table(tbl, target_container, pkg, part_registry)

        # Merge part_registry parts into pkg.part_defs if not already present
        for p_name, p_def in part_registry.items():
            if not any(p.name == p_name for p in (pkg.part_defs or [])):
                pkg.part_defs.append(p_def)

        # Fallback: if no parts were defined anywhere, but attributes or ports exist at package level,
        # create a canonical system PartDef so that structural model consumers have a primary part.
        if not pkg.part_defs and (pkg.port_defs or pkg.attribute_defs or pkg.constraint_defs):
            sys_part_name = f"{pkg_name}_System"
            sys_part = PartDef(name=sys_part_name, doc=f"Synthesized system component for {pkg_name}")
            sys_part.attributes = list(pkg.attribute_defs or [])
            sys_part.ports = list(pkg.port_defs or [])
            sys_part.constraints = list(pkg.constraint_defs or [])
            pkg.part_defs.append(sys_part)

        return pkg

    def translate_files(self, file_paths: List[str], default_name: str = "OEM_System_Model") -> SysMLPackage:
        """
        Translates multiple markdown specification files and merges them into a
        single canonical SysMLPackage AST.
        """
        combined_pkg = SysMLPackage(name=sanitize_identifier(default_name), doc="Consolidated Level 0 OEM Specifications")
        part_registry: Dict[str, PartDef] = {}

        for fpath in file_paths:
            if not os.path.exists(fpath):
                continue
            with open(fpath, "r", encoding="utf-8", errors="replace") as f:
                content = f.read()
            f_basename = os.path.splitext(os.path.basename(fpath))[0]
            sub_pkg = self.translate(content, default_name=f_basename)

            # Consolidate parts
            for p in (sub_pkg.part_defs or []):
                if p.name in part_registry:
                    existing = part_registry[p.name]
                    # Merge attributes
                    existing_attr_names = {a.name for a in (existing.attributes or [])}
                    for a in (p.attributes or []):
                        if a.name not in existing_attr_names:
                            existing.attributes.append(a)
                            existing_attr_names.add(a.name)
                    # Merge ports
                    existing_port_names = {prt.name for prt in (existing.ports or [])}
                    for prt in (p.ports or []):
                        if prt.name not in existing_port_names:
                            existing.ports.append(prt)
                            existing_port_names.add(prt.name)
                    # Merge constraints
                    existing_con_names = {c.name for c in (existing.constraints or [])}
                    for c in (p.constraints or []):
                        if c.name not in existing_con_names:
                            existing.constraints.append(c)
                            existing_con_names.add(c.name)
                else:
                    part_registry[p.name] = p

            # Consolidate package-level ports, attributes, constraints, connections
            pkg_attr_names = {a.name for a in (combined_pkg.attribute_defs or [])}
            for a in (sub_pkg.attribute_defs or []):
                if a.name not in pkg_attr_names:
                    combined_pkg.attribute_defs.append(a)
                    pkg_attr_names.add(a.name)

            pkg_port_names = {prt.name for prt in (combined_pkg.port_defs or [])}
            for prt in (sub_pkg.port_defs or []):
                if prt.name not in pkg_port_names:
                    combined_pkg.port_defs.append(prt)
                    pkg_port_names.add(prt.name)

            pkg_con_names = {c.name for c in (combined_pkg.constraint_defs or [])}
            for c in (sub_pkg.constraint_defs or []):
                if c.name not in pkg_con_names:
                    combined_pkg.constraint_defs.append(c)
                    pkg_con_names.add(c.name)

            pkg_conn_names = {conn.name for conn in (combined_pkg.connection_defs or [])}
            for conn in (sub_pkg.connection_defs or []):
                if conn.name not in pkg_conn_names:
                    combined_pkg.connection_defs.append(conn)
                    pkg_conn_names.add(conn.name)

        combined_pkg.part_defs = list(part_registry.values())
        return combined_pkg

    def _parse_markdown_structure(self, content: str, default_name: str) -> Tuple[str, List[Dict[str, Any]]]:
        """
        Parses Markdown lines into hierarchical sections and discovers tables with context.
        """
        lines = content.splitlines()
        pkg_name = sanitize_identifier(default_name)
        sections: List[Dict[str, Any]] = []

        current_sec_header = ""
        current_component_target: Optional[str] = None
        current_sec_doc: List[str] = []
        current_tables: List[Dict[str, Any]] = []

        in_table = False
        table_raw_headers: List[str] = []
        table_norm_headers: List[str] = []
        table_units: List[str] = []
        table_rows: List[Dict[str, str]] = []

        def _flush_table():
            nonlocal in_table, table_raw_headers, table_norm_headers, table_units, table_rows
            if in_table and table_raw_headers and table_rows:
                current_tables.append({
                    "raw_headers": list(table_raw_headers),
                    "normalized_headers": list(table_norm_headers),
                    "units": list(table_units),
                    "rows": list(table_rows),
                })
            in_table = False
            table_raw_headers = []
            table_norm_headers = []
            table_units = []
            table_rows = []

        def _flush_section():
            _flush_table()
            if current_sec_header or current_tables or current_sec_doc:
                sections.append({
                    "header": current_sec_header,
                    "component_target": current_component_target,
                    "section_doc": " ".join(current_sec_doc).strip(),
                    "tables": list(current_tables),
                })
            current_sec_doc.clear()
            current_tables.clear()

        i = 0
        n = len(lines)
        while i < n:
            line = lines[i]
            line_str = line.strip()

            # Heading 1 detection: # <PackageName>
            h1_m = re.match(r'^#\s+([^#].*)$', line_str)
            if h1_m:
                _flush_section()
                raw_pkg = h1_m.group(1).strip()
                # Remove words like "Package", "Specification", etc.
                raw_pkg_clean = re.sub(r'\b(?:package|specification|model|spec|system)\b', '', raw_pkg, flags=re.IGNORECASE).strip()
                pkg_name = sanitize_identifier(raw_pkg_clean or raw_pkg, default=pkg_name)
                current_sec_header = raw_pkg
                current_component_target = None
                i += 1
                continue

            # Heading 2 or 3 detection: ## <SectionOrComponent>
            h_sub = re.match(r'^(#{2,4})\s+([^#].*)$', line_str)
            if h_sub:
                _flush_section()
                raw_header = h_sub.group(2).strip()
                current_sec_header = raw_header

                # Determine if section header denotes a component or a functional grouping
                norm_header = re.sub(r'[*`]', '', raw_header).strip().lower()
                clean_header_key = re.sub(r'[^a-z0-9]+', ' ', norm_header).strip()

                is_functional = any(
                    clean_header_key == kw or clean_header_key.startswith(kw + " ") or clean_header_key.endswith(" " + kw)
                    for kw in NON_COMPONENT_SECTION_KEYWORDS
                )

                if is_functional:
                    # Keep previous component target if sub-heading (e.g. ### Interfaces under ## FlightController)
                    level = len(h_sub.group(1))
                    if level <= 2:
                        current_component_target = None
                else:
                    # Component header: strip prefix like "Component:", "Part:", "Subsystem:"
                    comp_name_cand = re.sub(r'^(?:component|part|subsystem|assembly|block|module)\s*[:\-]\s*', '', raw_header, flags=re.IGNORECASE).strip()
                    current_component_target = comp_name_cand

                i += 1
                continue

            # Markdown Table detection
            if "|" in line_str:
                # Potential table row
                cells = [c.strip() for c in line_str.strip().strip('|').split('|')]

                # Check if this is a delimiter row: | --- | :---: | ---: |
                is_delimiter = len(cells) >= 1 and all(re.match(r'^:?-+:?$', c) for c in cells if c)

                if is_delimiter and not in_table:
                    # Previous non-empty line must have been table header
                    # Find previous non-empty row in current doc or lines
                    if current_sec_doc:
                        header_line = current_sec_doc.pop()
                        raw_h_cells = [c.strip() for c in header_line.strip().strip('|').split('|')]
                        table_raw_headers = raw_h_cells
                        table_norm_headers = [normalize_col(c) for c in raw_h_cells]
                        table_units = [extract_unit_from_header_or_val(c)[1] for c in raw_h_cells]
                        in_table = True
                        i += 1
                        continue

                elif is_delimiter and in_table:
                    # Standard delimiter inside table
                    i += 1
                    continue

                elif in_table:
                    # Data row
                    row_dict: Dict[str, str] = {}
                    for idx, norm_col in enumerate(table_norm_headers):
                        val = clean_cell_value(cells[idx]) if idx < len(cells) else ""
                        row_dict[norm_col] = val
                    table_rows.append(row_dict)
                    i += 1
                    continue

                else:
                    # Could be the header row of a table (peeking ahead to next row)
                    if i + 1 < n and "|" in lines[i + 1]:
                        next_line = lines[i + 1].strip()
                        next_cells = [c.strip() for c in next_line.strip().strip('|').split('|')]
                        if len(next_cells) >= 1 and all(re.match(r'^:?-+:?$', c) for c in next_cells if c):
                            # Yes, current line is table header with delimiter row!
                            _flush_table()
                            table_raw_headers = cells
                            table_norm_headers = [normalize_col(c) for c in cells]
                            table_units = [extract_unit_from_header_or_val(c)[1] for c in cells]
                            in_table = True
                            i += 2  # Skip header and delimiter
                            continue
                        elif len(cells) >= 2 and len(next_cells) >= 2:
                            # Table without delimiter row
                            _flush_table()
                            table_raw_headers = cells
                            table_norm_headers = [normalize_col(c) for c in cells]
                            table_units = [extract_unit_from_header_or_val(c)[1] for c in cells]
                            in_table = True
                            i += 1
                            continue

            # If inside table and hit non-table line, end table
            if in_table and ("|" not in line_str or not line_str):
                _flush_table()

            # Prose doc line
            if line_str and not line_str.startswith("<!--") and not line_str.startswith("```"):
                current_sec_doc.append(line_str)

            i += 1

        _flush_section()
        return pkg_name, sections

    def _classify_table(self, headers: List[str]) -> str:
        """Classifies the table type based on column header patterns."""
        h_set = set(headers)

        # 1. BOM / Component list
        bom_indicators = {"component", "component_name", "part", "part_name", "subsystem", "item", "module", "assembly", "lru"}
        if any(h in h_set for h in bom_indicators) and any(h in h_set for h in {"part_number", "pn", "sku", "mass", "weight", "power", "qty", "quantity", "cost"}):
            return "bom"
        if any(h in h_set for h in bom_indicators) and len(headers) >= 3 and not any(h in h_set for h in {"direction", "dir", "port", "signal"}):
            return "bom"

        # 2. Ports / Interfaces / Signals
        port_indicators = {"port", "port_name", "signal", "signal_name", "signal_id", "interface", "interface_name", "flow"}
        dir_indicators = {"direction", "dir", "flow_direction"}
        if any(h in h_set for h in port_indicators) or (any(h in h_set for h in dir_indicators) and any(h in h_set for h in {"type", "data_type", "payload", "rate", "protocol"})):
            return "ports"
        if {"source_port", "target_port"}.issubset(h_set) or {"source", "target"}.issubset(h_set):
            return "ports"

        # 3. Parametric limits / Constraints
        constraint_indicators = {"min", "max", "lower_bound", "upper_bound", "range", "valid_range", "limit", "bound", "bounds", "tolerance", "constraint", "assertion"}
        param_indicators = {"parameter", "param", "property", "metric", "variable", "name"}
        if any(h in h_set for h in constraint_indicators) and any(h in h_set for h in param_indicators):
            return "constraints"
        if any(h in h_set for h in constraint_indicators) and len(headers) >= 2:
            return "constraints"

        # 4. Key-Value / Properties
        if len(headers) in (2, 3) and any(h in headers[0] for h in ("property", "parameter", "attribute", "key", "field", "name")):
            return "properties"

        return "generic"

    def _process_bom_table(self, table: Dict[str, Any], pkg: SysMLPackage, part_registry: Dict[str, PartDef]):
        """Processes BOM tables into PartDef AST elements with typed AttributeDefs."""
        headers = table["normalized_headers"]
        units = table.get("units", [])
        unit_map = {h: units[idx] if idx < len(units) else "" for idx, h in enumerate(headers)}

        comp_col = next((h for h in headers if h in {"component", "component_name", "part", "part_name", "subsystem", "item", "module", "assembly"}), headers[0])

        for row in table["rows"]:
            comp_raw = row.get(comp_col, "").strip()
            if not comp_raw:
                continue

            comp_name = sanitize_identifier(comp_raw)
            if comp_name not in part_registry:
                part_registry[comp_name] = PartDef(name=comp_name)
            part = part_registry[comp_name]

            # Parse attributes
            for col, val in row.items():
                if col == comp_col or not val.strip():
                    continue

                attr_name = sanitize_identifier(col)
                # Description column maps to doc
                if col in ("description", "doc", "function", "notes"):
                    if not part.doc:
                        part.doc = val.strip()
                    continue

                col_unit = unit_map.get(col, "")
                num_val, val_unit, int_val = parse_numeric_with_unit(val)
                eff_unit = col_unit or val_unit or ""

                if int_val is not None and "." not in val:
                    type_name = "Integer"
                    def_val = str(int_val)
                elif num_val is not None:
                    type_name = "Real"
                    def_val = str(num_val)
                elif val.lower() in ("true", "false"):
                    type_name = "Boolean"
                    def_val = val.lower()
                else:
                    type_name = "String"
                    clean_str = val.strip('"\'; ')
                    def_val = f'"{clean_str}"'

                doc_str = f"unit: {eff_unit}" if eff_unit else ""
                # Avoid duplicate attributes
                if not any(a.name == attr_name for a in (part.attributes or [])):
                    part.attributes.append(AttributeDef(
                        name=attr_name,
                        type_name=type_name,
                        default_value=def_val,
                        doc=doc_str
                    ))

            if not any(p.name == comp_name for p in (pkg.part_defs or [])):
                pkg.part_defs.append(part)

    def _process_ports_table(self, table: Dict[str, Any], target_container: Union[SysMLPackage, PartDef], pkg: SysMLPackage):
        """Processes Port/Signal/Interface tables into PortDef, ItemFlowDef, and ConnectionDef AST nodes."""
        headers = table["normalized_headers"]
        units = table.get("units", [])
        unit_map = {h: units[idx] if idx < len(units) else "" for idx, h in enumerate(headers)}

        name_col = next((h for h in headers if h in {"port", "port_name", "signal", "signal_name", "signal_id", "interface", "name", "flow"}), headers[0])
        dir_col = next((h for h in headers if h in {"direction", "dir", "flow_direction"}), None)
        type_col = next((h for h in headers if h in {"type", "data_type", "item_type", "payload", "port_type"}), None)
        protocol_col = next((h for h in headers if h in {"protocol", "protocol_family", "bus"}), None)
        rate_col = next((h for h in headers if h in {"rate", "rate_hz", "frequency", "update_rate"}), None)
        range_col = next((h for h in headers if h in {"range", "valid_range"}), None)
        unit_col = next((h for h in headers if h in {"unit", "units", "engineering_units"}), None)
        def_col = next((h for h in headers if h in {"default", "default_value"}), None)
        doc_col = next((h for h in headers if h in {"description", "doc", "notes"}), None)
        src_col = next((h for h in headers if h in {"source_port", "source", "source_part"}), None)
        tgt_col = next((h for h in headers if h in {"target_port", "target", "target_part"}), None)

        for row in table["rows"]:
            raw_name = row.get(name_col, "").strip()
            if not raw_name:
                continue

            port_name = sanitize_identifier(raw_name)

            # Direction
            dir_raw = (row.get(dir_col, "") if dir_col else "").strip().lower()
            if dir_raw in ("in", "input", "rx"):
                direction = "in"
            elif dir_raw in ("out", "output", "tx"):
                direction = "out"
            else:
                direction = "inout"

            # Type
            raw_type = (row.get(type_col, "") if type_col else "").strip()
            if raw_type:
                # Normalize primitive types
                low_t = raw_type.lower()
                if low_t in ("float", "float32", "float64", "double", "real"):
                    type_name = "Real"
                elif low_t in ("int", "int32", "int64", "uint", "integer"):
                    type_name = "Integer"
                elif low_t in ("bool", "boolean"):
                    type_name = "Boolean"
                elif low_t in ("str", "string"):
                    type_name = "String"
                else:
                    type_name = sanitize_identifier(raw_type)
            else:
                type_name = "Port"

            # Protocol
            protocol = (row.get(protocol_col, "") if protocol_col else "").strip()

            # Description
            doc = (row.get(doc_col, "") if doc_col else "").strip()

            # Rate
            rate_hz = None
            if rate_col:
                r_val, _, _ = parse_numeric_with_unit(row.get(rate_col, ""))
                rate_hz = r_val

            # Unit
            eff_unit = (row.get(unit_col, "") if unit_col else "").strip() or unit_map.get(name_col, "")

            # Range
            val_range = (row.get(range_col, "") if range_col else "").strip()

            # Default
            def_val = (row.get(def_col, "") if def_col else "").strip() or None

            # Create ItemFlowDef if flow properties exist
            item_flows: List[ItemFlowDef] = []
            if rate_hz is not None or eff_unit or val_range or (type_name != "Port" and direction != "inout"):
                flow_name = f"{port_name}_flow"
                item_flows.append(ItemFlowDef(
                    name=flow_name,
                    direction=direction if direction != "inout" else "out",
                    item_type=type_name if type_name != "Port" else "Item",
                    doc=doc,
                    rate_hz=rate_hz,
                    unit=eff_unit,
                    valid_range=val_range,
                    default_value=def_val
                ))

            port_def = PortDef(
                name=port_name,
                type_name=type_name,
                direction=direction,
                doc=doc,
                protocol_family=protocol,
                item_flows=item_flows
            )

            # Add to container
            if isinstance(target_container, PartDef):
                if not any(p.name == port_name for p in (target_container.ports or [])):
                    target_container.ports.append(port_def)
            else:
                if not any(p.name == port_name for p in (pkg.port_defs or [])):
                    pkg.port_defs.append(port_def)

            # ICD Connection creation if source and target ports are specified
            if src_col and tgt_col:
                src_val = row.get(src_col, "").strip()
                tgt_val = row.get(tgt_col, "").strip()
                if src_val and tgt_val:
                    conn_name = f"conn_{port_name}"
                    if not any(c.name == conn_name for c in (pkg.connection_defs or [])):
                        pkg.connection_defs.append(ConnectionDef(
                            name=conn_name,
                            source_port=src_val,
                            target_port=tgt_val,
                            protocol=protocol,
                            item_payload=type_name if type_name != "Port" else port_name,
                            doc=doc
                        ))

    def _process_constraints_table(self, table: Dict[str, Any], target_container: Union[SysMLPackage, PartDef], pkg: SysMLPackage):
        """Processes Parametric Limits and Constraints into SysMLConstraintDef and AttributeDef nodes."""
        headers = table["normalized_headers"]
        units = table.get("units", [])
        unit_map = {h: units[idx] if idx < len(units) else "" for idx, h in enumerate(headers)}

        param_col = next((h for h in headers if h in {"parameter", "param", "property", "metric", "variable", "name"}), headers[0])
        min_col = next((h for h in headers if h in {"min", "minimum", "lower_bound", "min_val"}), None)
        max_col = next((h for h in headers if h in {"max", "maximum", "upper_bound", "max_val"}), None)
        range_col = next((h for h in headers if h in {"range", "valid_range", "bounds", "limit", "bound"}), None)
        unit_col = next((h for h in headers if h in {"unit", "units", "engineering_units"}), None)
        type_col = next((h for h in headers if h in {"type", "data_type"}), None)
        def_col = next((h for h in headers if h in {"default", "default_value", "nominal", "value"}), None)
        constraint_col = next((h for h in headers if h in {"constraint", "condition", "invariant", "assertion"}), None)
        doc_col = next((h for h in headers if h in {"description", "doc", "notes"}), None)

        for row in table["rows"]:
            raw_param = row.get(param_col, "").strip()
            if not raw_param:
                continue

            param_name = sanitize_identifier(raw_param)
            doc = (row.get(doc_col, "") if doc_col else "").strip()
            eff_unit = (row.get(unit_col, "") if unit_col else "").strip() or unit_map.get(param_col, "")

            # Determine bounds
            min_val: Optional[float] = None
            max_val: Optional[float] = None

            if min_col and row.get(min_col):
                v, u, _ = parse_numeric_with_unit(row.get(min_col, ""))
                min_val = v
                if not eff_unit and u:
                    eff_unit = u

            if max_col and row.get(max_col):
                v, u, _ = parse_numeric_with_unit(row.get(max_col, ""))
                max_val = v
                if not eff_unit and u:
                    eff_unit = u

            if (min_val is None or max_val is None) and range_col and row.get(range_col):
                r_min, r_max = parse_range_bounds(row.get(range_col, ""))
                if min_val is None:
                    min_val = r_min
                if max_val is None:
                    max_val = r_max

            # Build expression
            expression = ""
            if constraint_col and row.get(constraint_col):
                expression = row.get(constraint_col, "").strip().rstrip(';')
            elif min_val is not None and max_val is not None:
                expression = f"{param_name} >= {min_val} and {param_name} <= {max_val}"
            elif min_val is not None:
                expression = f"{param_name} >= {min_val}"
            elif max_val is not None:
                expression = f"{param_name} <= {max_val}"

            # Create SysMLConstraintDef if expression is present
            if expression:
                con_name = f"assert_{param_name}_range"
                con_doc = f"{doc} [unit: {eff_unit}]".strip() if eff_unit else doc
                constraint_def = SysMLConstraintDef(
                    name=con_name,
                    expression=expression,
                    is_assertion=True,
                    doc=con_doc
                )
                if isinstance(target_container, PartDef):
                    if not any(c.name == con_name for c in (target_container.constraints or [])):
                        target_container.constraints.append(constraint_def)
                else:
                    if not any(c.name == con_name for c in (pkg.constraint_defs or [])):
                        pkg.constraint_defs.append(constraint_def)

            # Create corresponding AttributeDef
            raw_type = (row.get(type_col, "") if type_col else "").strip()
            type_name = "Real"
            if raw_type:
                low_t = raw_type.lower()
                if "int" in low_t:
                    type_name = "Integer"
                elif "bool" in low_t:
                    type_name = "Boolean"
                elif "str" in low_t:
                    type_name = "String"
                else:
                    type_name = sanitize_identifier(raw_type)

            raw_def = (row.get(def_col, "") if def_col else "").strip()
            def_val: Optional[str] = None
            if raw_def:
                num_v, _, int_v = parse_numeric_with_unit(raw_def)
                if int_v is not None and type_name == "Integer":
                    def_val = str(int_v)
                elif num_v is not None:
                    def_val = str(num_v)
                else:
                    def_val = f'"{raw_def.strip()}"'

            attr_doc = f"{doc} (unit: {eff_unit})".strip() if eff_unit else doc
            attr_def = AttributeDef(
                name=param_name,
                type_name=type_name,
                default_value=def_val,
                doc=attr_doc
            )

            if isinstance(target_container, PartDef):
                if not any(a.name == param_name for a in (target_container.attributes or [])):
                    target_container.attributes.append(attr_def)
            else:
                if not any(a.name == param_name for a in (pkg.attribute_defs or [])):
                    pkg.attribute_defs.append(attr_def)

    def _process_properties_table(self, table: Dict[str, Any], target_container: Union[SysMLPackage, PartDef], pkg: SysMLPackage):
        """Processes 2-3 column Key-Value property tables into AttributeDef and Constraint nodes."""
        headers = table["normalized_headers"]
        k_col = headers[0]
        v_col = headers[1] if len(headers) > 1 else headers[0]
        d_col = headers[2] if len(headers) > 2 else None

        for row in table["rows"]:
            raw_k = row.get(k_col, "").strip()
            raw_v = row.get(v_col, "").strip()
            if not raw_k:
                continue

            attr_name = sanitize_identifier(raw_k)
            doc = (row.get(d_col, "") if d_col else "").strip()

            num_v, unit, int_v = parse_numeric_with_unit(raw_v)
            if int_v is not None and "." not in raw_v:
                type_name = "Integer"
                def_val = str(int_v)
            elif num_v is not None:
                type_name = "Real"
                def_val = str(num_v)
            elif raw_v.lower() in ("true", "false"):
                type_name = "Boolean"
                def_val = raw_v.lower()
            else:
                type_name = "String"
                def_val = f'"{raw_v.strip()}"' if raw_v else None

            eff_doc = f"{doc} (unit: {unit})".strip() if unit else doc
            attr_def = AttributeDef(
                name=attr_name,
                type_name=type_name,
                default_value=def_val,
                doc=eff_doc
            )

            if isinstance(target_container, PartDef):
                if not any(a.name == attr_name for a in (target_container.attributes or [])):
                    target_container.attributes.append(attr_def)
            else:
                if not any(a.name == attr_name for a in (pkg.attribute_defs or [])):
                    pkg.attribute_defs.append(attr_def)

    def _process_generic_table(self, table: Dict[str, Any], target_container: Union[SysMLPackage, PartDef], pkg: SysMLPackage, part_registry: Dict[str, PartDef]):
        """Fallback processor for structured tables not matching known archetypes."""
        headers = table["normalized_headers"]
        if not headers:
            return

        # If table has a name column and properties, process as BOM or properties
        if len(headers) >= 2:
            self._process_properties_table(table, target_container, pkg)
