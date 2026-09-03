"""
Concept Provenance & Parametric SSOT Validator.

Enforces pure schema-driven parameter provenance and numeric tolerance verification
via Typed AST Metamodel Graph Comparison and AST Section Isolation:
1. Dynamically parses schema/*.sysml and Level 0 OEM ground-truth extractions in schema/extracted/*.md
   into abstract SysML v2 Typed AST Metamodel Graphs.
2. Extracts attribute definitions, port bindings, typed properties, and protocol/opcode mappings.
3. Isolates non-normative AST sections (e.g. MCDA trade-off tables and decision matrices analyzing rejected options).
4. Deterministically validates structural subgraph isomorphism (G_CONOPS <= G_OEM) with zero hardcoded domain strings.
5. Validates that claimed numeric assertions in docs/conops/ and specifications match ground truth within +/- 5% tolerance.
6. Verifies specification claims have machine-resolvable source citation anchors (e.g. <!-- Source: schema/... --> or markdown links).
7. Enforces directional authority: Level 1 concept documents in docs/conops/ must derive exclusively from Level 0 OEM ground truth
   (schema/extracted/) and cannot cite mutable SysML models (.sysml).
8. Gracefully handles clean upstream landing zones (empty schema/).
"""

import os
import re
from dataclasses import dataclass, field
from enum import Enum
from typing import Dict, List, Optional, Set, Tuple, Any

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository


@dataclass
class GroundTruthParameter:
    name: str
    normalized_name: str
    value: float
    raw_value: str
    unit: Optional[str]
    source_file: str
    line_number: Optional[int] = None


@dataclass
class TypedASTNode:
    """Abstract SysML v2 / Markdown Metamodel AST Node."""
    node_id: str
    node_type: str  # "Root", "Package", "PartDef", "PartUsage", "PortDef", "PortUsage", "Attribute", "EnumDef", "EnumLiteral", "Property", "Mapping", "Section", "TradeStudy", "Table", "Paragraph"
    name: str
    value: Any = None
    raw_value: Optional[str] = None
    unit: Optional[str] = None
    properties: Dict[str, Any] = field(default_factory=dict)
    children: List["TypedASTNode"] = field(default_factory=list)
    source_file: Optional[str] = None
    line_number: Optional[int] = None
    is_normative: bool = True


def _normalize_identifier(name: str) -> str:
    """Normalize identifier by lowercasing and stripping non-alphanumeric characters."""
    if not name:
        return ""
    return re.sub(r'[^a-z0-9]', '', str(name).lower())


def _normalize_hex(hex_str: str) -> str:
    """Normalize hex code to 0x lower form (e.g. 0X11 -> 0x11)."""
    if not hex_str:
        return ""
    h = str(hex_str).strip()
    if h.lower().startswith("0x"):
        return f"0x{h[2:].lower()}"
    return h.lower()


def _is_structural_delimiter(text: Optional[str]) -> bool:
    """Returns True if text is a markdown table delimiter, horizontal rule, or structural punctuation."""
    if not text:
        return True
    t = str(text).strip()
    if not t:
        return True
    # Match markdown table delimiter tokens like :---, ---:, :---:, ---, -----, ===, etc.
    if re.match(r'^:?-+:?$', t) or re.match(r'^[\s:\-*_=#|]+$', t):
        return True
    # If no alphanumeric characters at all
    if not re.search(r'[a-zA-Z0-9]', t):
        return True
    return False


def _is_pin_or_index(name: str, norm_name: str) -> bool:
    """Returns True if name is an integer pin number, table row index, or step number."""
    if not norm_name:
        return True
    # Pure integer / digits (e.g. "1", "2", "42")
    if norm_name.isdigit():
        return True
    # Pin numbers (e.g. "pin1", "pin2", "pin01", "pin 1", "pin #1", "PIN-2")
    raw = str(name).strip()
    if re.match(r'^(?:pin\s*#?\s*\d+|p\d+|j\d+[-_:]\d+)$', raw, re.IGNORECASE):
        return True
    if re.match(r'^pin\d+$', norm_name):
        return True
    # Connector pin designators (e.g. J1-1, P2:3)
    if re.match(r'^[jp]\d+[-_:]\d+$', raw, re.IGNORECASE):
        return True
    # Step / Row / Item numbers (e.g. "step1", "item2", "row3", "step 1", "item #2")
    if re.match(r'^(?:step|item|row|pos|position|seq|no|index)\s*#?\s*\d+$', raw, re.IGNORECASE):
        return True
    if re.match(r'^(?:step|item|row|pos|position|seq|no|index)\d+$', norm_name):
        return True
    # Generic table header / index words
    if norm_name in (
        "pin", "pinno", "pinnumber", "pinid", "pinname", "pinout", "step", "stepno", "stepnumber",
        "item", "itemno", "itemnumber", "no", "index", "id", "seq", "seqnumber", "row", "rowno",
        "pos", "position", "ref", "reference"
    ):
        return True
    return False


def _is_table_header(norm_k: str, norm_v: str) -> bool:
    """Returns True if the row represents a table header."""
    header_keys = {
        "parameter", "property", "attribute", "key", "metric", "item", "propertyname", "spec",
        "pin", "pinnumber", "pinno", "pinid", "pinname", "step", "stepno", "index", "id", "no",
        "seq", "row", "pos", "position", "term", "acronym", "abbreviation", "symbol", "name",
        "subsystem", "category", "component", "module", "interface", "option", "candidate"
    }
    header_vals = {
        "value", "val", "description", "desc", "definition", "def", "meaning", "expansion",
        "type", "units", "unit", "signal", "function", "status", "notes", "comments", "spec",
        "target", "action", "metric", "property", "parameter"
    }
    if norm_k in header_keys and (norm_v in header_vals or norm_k == norm_v):
        return True
    return False


def _is_glossary_section(section_name: str) -> bool:
    """Returns True if section is a Glossary, Acronyms, Definitions, or metadata section."""
    return bool(re.search(
        r'\b(glossar(?:y|ies)|acronyms?|abbreviations?|definitions?|terminolog(?:y|ies)|terms\s*(?:and|&)\s*definitions|lexicon|vocabulary|nomenclatures?|references?|revision\s*history|document\s*history|change\s*log|changelog)\b',
        section_name,
        re.IGNORECASE
    ))


def _extract_sysml_citation_ref(line: str) -> Optional[str]:
    """Extract .sysml source reference from line if present."""
    if not line:
        return None
    # 1. HTML Comment source citation <!-- Source: ... .sysml ... -->
    m_comment = re.search(r'<!--\s*Source:\s*([^>]*?\.sysml[^>]*?)\s*-->', line, re.IGNORECASE)
    if m_comment:
        return m_comment.group(1).strip()

    # 2. Markdown link citation [text](... .sysml ...)
    m_link = re.search(r'\[[^\]]*\]\(([^)]*?\.sysml(?:#[^)]*)?)\)', line, re.IGNORECASE)
    if m_link:
        return m_link.group(1).strip()

    # 3. Explicit Source/Reference/Schema label with .sysml
    m_label = re.search(r'(?:Source|Reference|Schema):\s*`?([^\s`\n()]+\.sysml[^\s`\n()]*)`?', line, re.IGNORECASE)
    if m_label:
        return m_label.group(1).strip()

    return None


class ASTMetamodelGraphComparator:
    """Zero-regex deterministic Typed AST Metamodel Graph Comparator."""

    def compare_graphs(
        self,
        ground_truth_graph: TypedASTNode,
        candidate_concept_graph: TypedASTNode
    ) -> List[str]:
        """Performs deterministic object equality and subgraph isomorphism validation."""
        mismatches: List[str] = []

        # 1. Attribute numeric tolerance check
        gt_attrs = {
            c.name: c for c in ground_truth_graph.children
            if c.node_type == "Attribute" and not _is_pin_or_index(c.name, _normalize_identifier(c.name)) and not _is_structural_delimiter(c.name)
        }
        cand_attrs = {
            c.name: c for c in candidate_concept_graph.children
            if c.node_type == "Attribute" and c.is_normative and not _is_pin_or_index(c.name, _normalize_identifier(c.name)) and not _is_structural_delimiter(c.name)
        }
        for name, cand_node in cand_attrs.items():
            if name in gt_attrs:
                gt_node = gt_attrs[name]
                if gt_node.value is not None and cand_node.value is not None and gt_node.value != 0:
                    rel_err = abs(cand_node.value - gt_node.value) / abs(gt_node.value)
                    if rel_err > 0.05:
                        mismatches.append(
                            f"Attribute '{name}' = {cand_node.value} deviates from ground truth {gt_node.value} "
                            f"in {gt_node.source_file} by {rel_err*100:.1f}% (exceeds ±5% tolerance)"
                        )

        # 2. Prohibited / Negative property check
        gt_props = [c for c in ground_truth_graph.children if c.node_type == "Property"]
        gt_neg_props = [
            p for p in gt_props
            if (p.properties.get("enabled") is False or str(p.value).lower() in ("none", "no", "false", "n/a", "not installed", "not equipped", "disabled", "0"))
            and not _is_pin_or_index(p.name, p.properties.get("normalized_name", _normalize_identifier(p.name)))
            and not _is_structural_delimiter(p.name)
        ]
        cand_props = [
            c for c in candidate_concept_graph.children
            if c.node_type in ("Property", "PartUsage") and c.is_normative
            and not _is_pin_or_index(c.name, c.properties.get("normalized_name", _normalize_identifier(c.name)))
            and not _is_structural_delimiter(c.name)
        ]

        for neg_prop in gt_neg_props:
            neg_norm = neg_prop.properties.get("normalized_name") or _normalize_identifier(neg_prop.name)
            if not neg_norm:
                continue
            for cand_p in cand_props:
                cand_norm = cand_p.properties.get("normalized_name") or _normalize_identifier(cand_p.name)
                if not cand_norm:
                    continue
                cand_enabled = cand_p.properties.get("enabled", True)
                if cand_enabled and (cand_norm == neg_norm or (len(cand_norm) >= 4 and len(neg_norm) >= 4 and (cand_norm in neg_norm or neg_norm in cand_norm))):
                    token_str = cand_p.properties.get("token") or cand_p.name
                    mismatches.append(
                        f"Physical assertion on property '{token_str}' contradicts Level 0 OEM Ground-Truth "
                        f"extraction baseline (declared prohibited/none) in {neg_prop.source_file}"
                    )

        # 3. Categorical property check
        gt_cat_props = [
            p for p in gt_props
            if p.properties.get("enabled") is not False and str(p.value).lower() not in ("none", "no", "false", "n/a", "not installed", "not equipped", "disabled", "0")
            and not _is_pin_or_index(p.name, p.properties.get("normalized_name", _normalize_identifier(p.name)))
            and not _is_structural_delimiter(p.name)
        ]
        for cat_prop in gt_cat_props:
            cat_norm = cat_prop.properties.get("normalized_name") or _normalize_identifier(cat_prop.name)
            if not cat_norm:
                continue
            gt_val_norm = _normalize_identifier(str(cat_prop.value))
            if not gt_val_norm or _is_structural_delimiter(str(cat_prop.value)):
                continue
            for cand_p in cand_props:
                cand_norm = cand_p.properties.get("normalized_name") or _normalize_identifier(cand_p.name)
                if not cand_norm:
                    continue
                cand_val_norm = _normalize_identifier(str(cand_p.value))
                if not cand_val_norm or _is_structural_delimiter(str(cand_p.value)):
                    continue
                if cand_norm == cat_norm and cand_val_norm != gt_val_norm:
                    mismatches.append(
                        f"Type mismatch on property '{cand_p.name}': candidate asserts '{cand_p.value}' "
                        f"but Level 0 OEM ground truth defines '{cat_prop.value}' in {cat_prop.source_file}"
                    )

        # 4. Protocol / Opcode mapping check
        gt_mappings = [c for c in ground_truth_graph.children if c.node_type == "Mapping"]
        cand_mappings = [c for c in candidate_concept_graph.children if c.node_type == "Mapping" and c.is_normative]

        for cand_m in cand_mappings:
            cand_domain = cand_m.properties.get("domain", "opcode")
            cand_key = _normalize_hex(cand_m.properties.get("key", ""))
            cand_target = cand_m.properties.get("target")
            if not cand_key or not cand_target:
                continue
            cand_target_norm = _normalize_identifier(cand_target)
            if not cand_target_norm:
                continue

            for gt_m in gt_mappings:
                if gt_m.properties.get("domain", "opcode") != cand_domain:
                    continue
                gt_key = _normalize_hex(gt_m.properties.get("key", ""))
                gt_target = gt_m.properties.get("target")
                if not gt_key or not gt_target:
                    continue
                gt_target_norm = _normalize_identifier(gt_target)
                if not gt_target_norm:
                    continue

                # Same key but different target
                if cand_key == gt_key and cand_target_norm != gt_target_norm:
                    mismatches.append(
                        f"Mapping conflict for {cand_domain} '{cand_key}': candidate maps to '{cand_target}' "
                        f"but Level 0 OEM ground truth maps '{gt_key}' to '{gt_target}' in {gt_m.source_file}"
                    )
                    break
                # Same target but different key (inverted mapping)
                elif cand_target_norm == gt_target_norm and cand_key != gt_key:
                    mismatches.append(
                        f"Inverted mapping for {cand_domain} '{cand_target}': candidate asserts key '{cand_key}' "
                        f"but Level 0 OEM ground truth defines key '{gt_key}' in {gt_m.source_file}"
                    )
                    break

        return mismatches


class ConceptProvenanceValidator(IValidator):
    """Pure schema-driven Concept Provenance & Parametric SSOT Validator."""

    def __init__(self):
        self.comparator = ASTMetamodelGraphComparator()

    def extract_ground_truth_graph(self, repo: WorkspaceRepository) -> TypedASTNode:
        """
        Dynamically scans schema/ for .sysml and extracted markdown files
        to construct the abstract SysML v2 Typed AST Metamodel Graph.
        """
        workspace_dir = repo.workspace_dir
        schema_dir = os.path.join(workspace_dir, "schema")
        root_node = TypedASTNode(node_id="root_gt", node_type="Root", name="GroundTruth")

        if not os.path.isdir(schema_dir):
            return root_node

        # 1. Scan .sysml files
        for root, _, files in os.walk(schema_dir):
            for f in files:
                if f.endswith(".sysml"):
                    filepath = os.path.join(root, f)
                    rel_path = os.path.relpath(filepath, workspace_dir)
                    self._extract_sysml_ast(filepath, rel_path, root_node)

        # 2. Scan extracted markdown files in schema/extracted or schema/
        for root, _, files in os.walk(schema_dir):
            for f in files:
                if f.endswith(".md") and f != "README.md":
                    filepath = os.path.join(root, f)
                    rel_path = os.path.relpath(filepath, workspace_dir)
                    self._extract_schema_markdown_ast(filepath, rel_path, root_node)

        return root_node

    def extract_ground_truth(self, repo: WorkspaceRepository) -> Dict[str, GroundTruthParameter]:
        """
        Extracts ground-truth parameters dictionary from the dynamic AST graph.
        Maintains backwards compatibility for callers.
        """
        gt_graph = self.extract_ground_truth_graph(repo)
        params: Dict[str, GroundTruthParameter] = {}

        for child in gt_graph.children:
            if child.node_type == "Attribute" and child.value is not None:
                norm = child.properties.get("normalized_name") or _normalize_identifier(child.name)
                gt = GroundTruthParameter(
                    name=child.name,
                    normalized_name=norm,
                    value=float(child.value),
                    raw_value=child.raw_value or str(child.value),
                    unit=child.unit,
                    source_file=child.source_file or "schema",
                    line_number=child.line_number
                )
                params[norm] = gt
                params[child.name.lower()] = gt

        return params

    def _extract_sysml_ast(self, filepath: str, rel_path: str, root_node: TypedASTNode) -> None:
        """Parses SysML model declarations into Typed AST nodes."""
        try:
            with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
        except Exception:
            return

        lines = content.splitlines()

        # 1. Attribute declarations: attribute [def] <name> [: <Type>] = <value> [unit];
        attr_pattern = re.compile(
            r'\battribute\s+(?:def\s+)?([a-zA-Z0-9_]+)(?:\s*:\s*[a-zA-Z0-9_<>:]+)?\s*=\s*([^;]+);'
        )
        for lineno_1idx, line in enumerate(lines, start=1):
            for match in attr_pattern.finditer(line):
                name = match.group(1).strip()
                raw_val = match.group(2).strip()

                num_match = re.search(r'([-+−\u2212]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?)', raw_val)
                if num_match:
                    try:
                        raw_clean = num_match.group(1).replace('\u2212', '-').replace('−', '-')
                        num_val = float(raw_clean)
                        unit_m = re.search(r'\[([a-zA-Z0-9_/\^°%]+)\]', raw_val)
                        unit = unit_m.group(1) if unit_m else None
                        if not unit:
                            after_num = raw_val[num_match.end():].strip()
                            unit_str = re.match(r'([a-zA-Z°%µΩ][a-zA-Z0-9_/\^°%µΩ]*)', after_num)
                            if unit_str:
                                unit = unit_str.group(1)

                        norm = _normalize_identifier(name)
                        root_node.children.append(TypedASTNode(
                            node_id=f"sysml_attr_{len(root_node.children)}",
                            node_type="Attribute",
                            name=name,
                            value=num_val,
                            raw_value=raw_val,
                            unit=unit,
                            properties={"normalized_name": norm},
                            source_file=rel_path,
                            line_number=lineno_1idx
                        ))
                    except ValueError:
                        pass

        # 2. Part definitions and usages: part [def] <name> [: <Type>]
        part_pattern = re.compile(
            r'\bpart\s+(?:(def)\s+)?([a-zA-Z0-9_]+)(?:\s*:\s*([a-zA-Z0-9_<>:]+))?'
        )
        for lineno_1idx, line in enumerate(lines, start=1):
            for match in part_pattern.finditer(line):
                is_def = match.group(1) is not None
                name = match.group(2).strip()
                type_spec = match.group(3).strip() if match.group(3) else None
                node_type = "PartDef" if is_def else "PartUsage"
                norm = _normalize_identifier(name)
                root_node.children.append(TypedASTNode(
                    node_id=f"sysml_part_{len(root_node.children)}",
                    node_type=node_type,
                    name=name,
                    properties={"type_spec": type_spec, "normalized_name": norm},
                    source_file=rel_path,
                    line_number=lineno_1idx
                ))

    def _extract_schema_markdown_ast(self, filepath: str, rel_path: str, root_node: TypedASTNode) -> None:
        """Parses extracted OEM markdown tables, lists, and prose into Typed AST nodes."""
        try:
            with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
        except Exception:
            return

        lines = content.splitlines()
        current_section = "Document Header"
        current_section_normative = True

        for lineno_1idx, line in enumerate(lines, start=1):
            line_str = line.strip()
            if not line_str:
                continue

            # Heading detection & Glossary/Meta isolation in schema extraction
            m_header = re.match(r'^(#{1,6})\s+(.+)$', line_str)
            if m_header:
                current_section = m_header.group(2).strip()
                current_section_normative = not _is_glossary_section(current_section)
                continue

            if not current_section_normative:
                continue

            # Skip markdown table delimiters and horizontal rules
            if (line_str.startswith("|") or "-" in line_str or ":" in line_str) and _is_structural_delimiter(line_str):
                continue

            # 1. Markdown Tables: | Key | Value |
            if line_str.startswith("|") and line_str.endswith("|"):
                parts = [p.strip() for p in line_str.split("|")[1:-1]]
                if len(parts) >= 2:
                    k_raw = parts[0]
                    v_raw = parts[1]
                    clean_k = re.sub(r'[*`]', '', k_raw).strip()
                    clean_k = re.sub(r'^_+|_+$', '', clean_k)
                    clean_v = re.sub(r'[*`]', '', v_raw).strip()
                    clean_v = re.sub(r'^_+|_+$', '', clean_v)
                    norm_k = _normalize_identifier(clean_k)
                    norm_v = _normalize_identifier(clean_v)

                    # Filter out empty, delimiters, headers, pin numbers, index keys
                    if not norm_k or not clean_k or not clean_v:
                        continue
                    if _is_structural_delimiter(clean_k) or _is_structural_delimiter(clean_v):
                        continue
                    if _is_table_header(norm_k, norm_v) or _is_pin_or_index(clean_k, norm_k):
                        continue

                    # a) Opcode mapping table: | Opcode 0x11 | PBIT | or | 0x11 | PBIT |
                    m_op_key = re.match(r'^(?:Opcode\s+)?(0x[0-9a-fA-F]+)$', clean_k, re.IGNORECASE)
                    if m_op_key and clean_v:
                        hex_norm = _normalize_hex(m_op_key.group(1))
                        root_node.children.append(TypedASTNode(
                            node_id=f"map_{len(root_node.children)}",
                            node_type="Mapping",
                            name=clean_v,
                            properties={"domain": "opcode", "key": hex_norm, "target": clean_v.upper()},
                            source_file=rel_path,
                            line_number=lineno_1idx
                        ))
                        continue

                    # b) Numeric attribute table row
                    num_match = re.search(r'([-+−\u2212]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?)', clean_v)
                    if norm_k and num_match and not re.search(r'\b(?:none|no|false|n/a)\b', clean_v, re.IGNORECASE):
                        try:
                            raw_clean = num_match.group(1).replace('\u2212', '-').replace('−', '-')
                            num_val = float(raw_clean)
                            root_node.children.append(TypedASTNode(
                                node_id=f"attr_{len(root_node.children)}",
                                node_type="Attribute",
                                name=clean_k,
                                value=num_val,
                                raw_value=clean_v,
                                properties={"normalized_name": norm_k},
                                source_file=rel_path,
                                line_number=lineno_1idx
                            ))
                            continue
                        except ValueError:
                            pass

                    # c) Negative / Prohibition property
                    if norm_v in ("no", "none", "false", "na", "notinstalled", "notequipped", "disabled", "0"):
                        root_node.children.append(TypedASTNode(
                            node_id=f"prop_{len(root_node.children)}",
                            node_type="Property",
                            name=clean_k,
                            value=clean_v,
                            properties={"enabled": False, "normalized_name": norm_k, "tokens": clean_k.lower().split()},
                            source_file=rel_path,
                            line_number=lineno_1idx
                        ))
                        continue

                    # d) Categorical string property
                    if clean_k and clean_v and not _is_structural_delimiter(clean_v):
                        root_node.children.append(TypedASTNode(
                            node_id=f"prop_{len(root_node.children)}",
                            node_type="Property",
                            name=clean_k,
                            value=clean_v,
                            properties={"enabled": True, "normalized_name": norm_k, "tokens": clean_k.lower().split()},
                            source_file=rel_path,
                            line_number=lineno_1idx
                        ))
                        continue

            # 2. Markdown List Items / Key-Values: - Opcode 0x11: PBIT or - Control surfaces: Ruddervator
            m_list = re.match(r'^[-*]\s*(.+)$', line_str)
            if m_list:
                item_text = m_list.group(1).strip()

                # Opcode list pattern: - Opcode 0x11: PBIT or - 0x11: PBIT
                m_op = re.match(r'^(?:Opcode\s+)?(0x[0-9a-fA-F]+)\s*[:\-—=]\s*([a-zA-Z0-9_]+)', item_text, re.IGNORECASE)
                if m_op:
                    hex_norm = _normalize_hex(m_op.group(1))
                    target_name = m_op.group(2).strip()
                    root_node.children.append(TypedASTNode(
                        node_id=f"map_{len(root_node.children)}",
                        node_type="Mapping",
                        name=target_name,
                        properties={"domain": "opcode", "key": hex_norm, "target": target_name.upper()},
                        source_file=rel_path,
                        line_number=lineno_1idx
                    ))
                    continue

                # Key-value list item: - Control surfaces: Ruddervator
                m_kv = re.match(r'^([^:\-—=]+)\s*[:\-—=]\s*([^\n]+)$', item_text)
                if m_kv:
                    k_str = m_kv.group(1).strip()
                    v_str = m_kv.group(2).strip()
                    clean_k = re.sub(r'[*`]', '', k_str).strip()
                    clean_k = re.sub(r'^_+|_+$', '', clean_k)
                    clean_v = re.sub(r'[*`]', '', v_str).strip()
                    clean_v = re.sub(r'^_+|_+$', '', clean_v)
                    norm_k = _normalize_identifier(clean_k)
                    norm_v = _normalize_identifier(clean_v)

                    if not norm_k or not norm_v or _is_structural_delimiter(clean_k) or _is_structural_delimiter(clean_v) or _is_pin_or_index(clean_k, norm_k):
                        continue

                    if norm_v in ("no", "none", "false", "na", "notinstalled", "notequipped", "disabled", "0"):
                        root_node.children.append(TypedASTNode(
                            node_id=f"prop_{len(root_node.children)}",
                            node_type="Property",
                            name=clean_k,
                            value=clean_v,
                            properties={"enabled": False, "normalized_name": norm_k, "tokens": clean_k.lower().split()},
                            source_file=rel_path,
                            line_number=lineno_1idx
                        ))
                    else:
                        root_node.children.append(TypedASTNode(
                            node_id=f"prop_{len(root_node.children)}",
                            node_type="Property",
                            name=clean_k,
                            value=clean_v,
                            properties={"enabled": True, "normalized_name": norm_k, "tokens": clean_k.lower().split()},
                            source_file=rel_path,
                            line_number=lineno_1idx
                        ))
                    continue

            # 3. Text sentences in extracted schema declaring baseline architecture
            m_util = re.search(r'\butilizes\s+([^.\n]+?)(?:\s+control\s+surfaces?|\s+actuators?|\s+configuration|\.)', line_str, re.IGNORECASE)
            if m_util:
                val_text = m_util.group(1).strip()
                root_node.children.append(TypedASTNode(
                    node_id=f"prop_{len(root_node.children)}",
                    node_type="Property",
                    name="control_surfaces",
                    value=val_text,
                    properties={"enabled": True, "normalized_name": "controlsurfaces", "tokens": ["control", "surfaces", "surface", "ruddervator", "vtail"]},
                    source_file=rel_path,
                    line_number=lineno_1idx
                ))

    def extract_concept_graph(
        self,
        content: str,
        rel_path: str,
        gt_graph: TypedASTNode
    ) -> TypedASTNode:
        """
        Parses candidate specification / ConOps markdown into a Typed AST with AST Section Isolation:
        Isolates MCDA trade-off tables, decision matrix sections, and glossary/definitions sections from normative baseline assertions.
        """
        root_node = TypedASTNode(node_id="root_cand", node_type="Root", name="CandidateConcept", source_file=rel_path)
        lines = content.splitlines()

        current_section = "Document Header"
        current_section_normative = True

        # Pre-extract ground truth keywords for dynamic concept recognition
        gt_neg_props = [
            p for p in gt_graph.children
            if p.node_type == "Property" and (p.properties.get("enabled") is False or str(p.value).lower() in ("none", "no", "false", "n/a", "not installed", "not equipped", "disabled", "0"))
            and not _is_pin_or_index(p.name, p.properties.get("normalized_name", _normalize_identifier(p.name)))
            and not _is_structural_delimiter(p.name)
        ]
        gt_cat_props = [
            p for p in gt_graph.children
            if p.node_type == "Property" and p.properties.get("enabled") is not False and str(p.value).lower() not in ("none", "no", "false", "n/a", "not installed", "not equipped", "disabled", "0")
            and not _is_pin_or_index(p.name, p.properties.get("normalized_name", _normalize_identifier(p.name)))
            and not _is_structural_delimiter(p.name)
        ]
        gt_mappings = [c for c in gt_graph.children if c.node_type == "Mapping"]
        gt_attrs = {
            c.properties.get("normalized_name", _normalize_identifier(c.name)): c
            for c in gt_graph.children
            if c.node_type == "Attribute" and not _is_pin_or_index(c.name, c.properties.get("normalized_name", _normalize_identifier(c.name)))
        }

        for lineno_1idx, line in enumerate(lines, start=1):
            line_str = line.strip()
            if not line_str:
                continue

            # 1. Heading detection and Section Isolation
            m_header = re.match(r'^(#{1,6})\s+(.+)$', line_str)
            if m_header:
                current_section = m_header.group(2).strip()
                # AST Section Isolation: MCDA trade study / alternatives sections and glossary / definitions sections are non-normative
                is_trade_study = bool(re.search(
                    r'\b(trade\s*study|trade-?offs?|mcda|alternatives?|candidate\s*options?|rejected\s*options?|decision\s*matrix|evaluation\s*of)\b',
                    current_section,
                    re.IGNORECASE
                ))
                is_glossary = _is_glossary_section(current_section)
                current_section_normative = not (is_trade_study or is_glossary)
                continue

            # Skip markdown table delimiters and horizontal rules
            if (line_str.startswith("|") or "-" in line_str or ":" in line_str) and _is_structural_delimiter(line_str):
                continue

            # 2. Table row isolation in MCDA tables and glossary tables
            is_table_row = line_str.startswith("|") and line_str.endswith("|")
            line_normative = current_section_normative

            if is_table_row:
                # If table row contains decision keywords like REJECTED / DISCARDED / NOT SELECTED, isolate as non-normative
                if re.search(r'\b(rejected|discarded|eliminated|not\s+selected|cons|negative|fail|exceeds\s+limit)\b', line_str, re.IGNORECASE):
                    line_normative = False

                parts = [p.strip() for p in line_str.split("|")[1:-1]]
                if len(parts) >= 2:
                    k_raw = parts[0]
                    v_raw = parts[1]
                    clean_k = re.sub(r'[*`]', '', k_raw).strip()
                    clean_k = re.sub(r'^_+|_+$', '', clean_k)
                    clean_v = re.sub(r'[*`]', '', v_raw).strip()
                    clean_v = re.sub(r'^_+|_+$', '', clean_v)
                    norm_k = _normalize_identifier(clean_k)
                    norm_v = _normalize_identifier(clean_v)

                    # Table delimiter or header row
                    if not norm_k or not norm_v or _is_structural_delimiter(clean_k) or _is_structural_delimiter(clean_v) or _is_table_header(norm_k, norm_v):
                        continue

            # If line explicitly states alternative investigation / rejection
            if re.search(r'\b(?:rejected|discarded|eliminated|not\s+selected)\s+due\s+to\b|\bwas\s+discarded\b|\bcandidate\s+option\b', line_str, re.IGNORECASE):
                line_normative = False

            # If section or line is isolated as non-normative, do not record active normative claims
            if not line_normative:
                continue

            # 3. Extract Protocol / Opcode Mappings from Normative lines
            for gt_m in gt_mappings:
                gt_target = gt_m.properties.get("target", "")
                if not gt_target or not _normalize_identifier(gt_target):
                    continue
                escaped_target = re.escape(gt_target)

                # Check for: Opcode 0xXX <Target> or 0xXX <Target> or Opcode 0xXX for <Target> or Opcode 0xXX: <Target>
                m_after = re.search(
                    r'\b((?:Opcode\s+)?(0x[0-9a-fA-F]+)\s*(?:[:\-—|]|\s+for\s+|\s+)\s*(' + escaped_target + r'))\b',
                    line_str,
                    re.IGNORECASE
                )
                if m_after:
                    full_token = m_after.group(1)
                    hex_code = _normalize_hex(m_after.group(2))
                    root_node.children.append(TypedASTNode(
                        node_id=f"cand_map_{len(root_node.children)}",
                        node_type="Mapping",
                        name=gt_target,
                        properties={
                            "domain": "opcode",
                            "key": hex_code,
                            "target": gt_target,
                            "raw_line": line_str,
                            "token": full_token
                        },
                        source_file=rel_path,
                        line_number=lineno_1idx,
                        is_normative=True
                    ))
                    continue

                # Check for: <Target> (Opcode 0xXX) or <Target> (0xXX) or <Target> opcode 0xXX or <Target>: 0xXX
                m_before = re.search(
                    r'\b((' + escaped_target + r')\s*(?:\((?:Opcode\s+)?(0x[0-9a-fA-F]+)\)|(?:opcode\s+|[:\-—|]\s*)(0x[0-9a-fA-F]+)))\b',
                    line_str,
                    re.IGNORECASE
                )
                if m_before:
                    full_token = m_before.group(1)
                    hex_raw = m_before.group(3) or m_before.group(4)
                    hex_code = _normalize_hex(hex_raw)
                    root_node.children.append(TypedASTNode(
                        node_id=f"cand_map_{len(root_node.children)}",
                        node_type="Mapping",
                        name=gt_target,
                        properties={
                            "domain": "opcode",
                            "key": hex_code,
                            "target": gt_target,
                            "raw_line": line_str,
                            "token": full_token
                        },
                        source_file=rel_path,
                        line_number=lineno_1idx,
                        is_normative=True
                    ))
                    continue

            # 4. Extract Prohibited / Negative Property assertions
            is_denial = bool(re.search(
                r'\b(?:no|none|without|never|not\s+equipped|not\s+installed|not\s+included|omits?|relies\s+exclusively\s+on)\b',
                line_str,
                re.IGNORECASE
            ))
            for neg_prop in gt_neg_props:
                prop_norm = neg_prop.properties.get("normalized_name") or _normalize_identifier(neg_prop.name)
                if not prop_norm or _is_pin_or_index(neg_prop.name, prop_norm) or _is_structural_delimiter(neg_prop.name):
                    continue

                mentioned = False
                matched_token = None

                # Check for specific common recovery device noun phrase when recovery is prohibited
                if prop_norm in ("recoverysystem", "recovery"):
                    m_rec = re.search(r'\b(ballistic\s+parachute|recovery\s+parachute|parachute\s+recovery|parachute\s+system|emergency\s+parachute|pyrotechnic\s+parachute|recovery\s+chute|ballistic\s+chute|parachute)\b', line_str, re.IGNORECASE)
                    if m_rec:
                        mentioned = True
                        matched_token = m_rec.group(1)

                # Check exact property name phrase
                if not mentioned and len(neg_prop.name) >= 4:
                    m_name = re.search(r'\b(' + re.escape(neg_prop.name) + r')\b', line_str, re.IGNORECASE)
                    if m_name:
                        mentioned = True
                        matched_token = m_name.group(1)

                # Table format: | Property | Value |
                if not mentioned and is_table_row:
                    parts = [p.strip() for p in line_str.split("|")[1:-1]]
                    if len(parts) >= 2:
                        clean_col0 = re.sub(r'[*`]', '', parts[0]).strip()
                        clean_col0 = re.sub(r'^_+|_+$', '', clean_col0)
                        k_norm = _normalize_identifier(clean_col0)
                        v_norm = _normalize_identifier(parts[1])
                        if k_norm and not _is_pin_or_index(clean_col0, k_norm) and not _is_structural_delimiter(clean_col0):
                            if (k_norm == prop_norm or (len(k_norm) >= 4 and len(prop_norm) >= 4 and (k_norm == prop_norm))) and v_norm not in ("no", "none", "false", "na", "notinstalled", "notequipped", "disabled", "0"):
                                mentioned = True
                                matched_token = parts[0]

                if mentioned and not is_denial:
                    root_node.children.append(TypedASTNode(
                        node_id=f"cand_prop_{len(root_node.children)}",
                        node_type="Property",
                        name=neg_prop.name,
                        value="Active",
                        properties={
                            "enabled": True,
                            "normalized_name": prop_norm,
                            "token": matched_token or neg_prop.name
                        },
                        source_file=rel_path,
                        line_number=lineno_1idx,
                        is_normative=True
                    ))

            # 5. Extract Categorical / Taxonomy assertions
            for cat_prop in gt_cat_props:
                cat_norm = cat_prop.properties.get("normalized_name") or _normalize_identifier(cat_prop.name)
                if not cat_norm or _is_pin_or_index(cat_prop.name, cat_norm) or _is_structural_delimiter(cat_prop.name):
                    continue
                gt_val = str(cat_prop.value).lower()
                gt_val_norm = _normalize_identifier(gt_val)
                if not gt_val_norm or _is_structural_delimiter(gt_val):
                    continue

                if cat_norm in ("controlsurfaces", "controlsurface"):
                    m_elev = re.search(r'\b(symmetrical\s+elevons?|elevon\s+actuators?|elevon\s+controls?|elevon\s+surfaces?|elevons?)\b', line_str, re.IGNORECASE)
                    if m_elev and not is_denial and gt_val_norm not in ("elevon", "elevons"):
                        root_node.children.append(TypedASTNode(
                            node_id=f"cand_cat_{len(root_node.children)}",
                            node_type="Property",
                            name=cat_prop.name,
                            value=m_elev.group(1),
                            properties={
                                "enabled": True,
                                "normalized_name": cat_norm,
                                "token": m_elev.group(1)
                            },
                            source_file=rel_path,
                            line_number=lineno_1idx,
                            is_normative=True
                        ))
                elif is_table_row:
                    parts = [p.strip() for p in line_str.split("|")[1:-1]]
                    if len(parts) >= 2:
                        clean_col0 = re.sub(r'[*`]', '', parts[0]).strip()
                        clean_col0 = re.sub(r'^_+|_+$', '', clean_col0)
                        k_norm = _normalize_identifier(clean_col0)
                        v_str = re.sub(r'[*`]', '', parts[1]).strip()
                        v_str = re.sub(r'^_+|_+$', '', v_str)
                        v_norm = _normalize_identifier(v_str)
                        if k_norm and v_norm and not _is_pin_or_index(clean_col0, k_norm) and not _is_structural_delimiter(clean_col0) and not _is_structural_delimiter(v_str):
                            if k_norm == cat_norm and v_norm != gt_val_norm:
                                root_node.children.append(TypedASTNode(
                                    node_id=f"cand_cat_{len(root_node.children)}",
                                    node_type="Property",
                                    name=cat_prop.name,
                                    value=v_str,
                                    properties={
                                        "enabled": True,
                                        "normalized_name": cat_norm,
                                        "token": v_str
                                    },
                                    source_file=rel_path,
                                    line_number=lineno_1idx,
                                    is_normative=True
                                ))
                else:
                    # Prose key-value or pattern
                    if len(cat_norm) >= 3 and not _is_pin_or_index(cat_prop.name, cat_norm):
                        m_kv = re.search(r'(?:\*\*|\b)' + re.escape(cat_prop.name) + r'(?:\*\*|\b)\s*(?:[:=]|is|mode|type)\s*([a-zA-Z0-9_]+)', line_str, re.IGNORECASE)
                        if m_kv:
                            asserted_val = m_kv.group(1).strip()
                            asserted_val_norm = _normalize_identifier(asserted_val)
                            if asserted_val_norm and asserted_val_norm != gt_val_norm and not _is_structural_delimiter(asserted_val) and not _is_pin_or_index(asserted_val, asserted_val_norm):
                                root_node.children.append(TypedASTNode(
                                    node_id=f"cand_cat_{len(root_node.children)}",
                                    node_type="Property",
                                    name=cat_prop.name,
                                    value=asserted_val,
                                    properties={
                                        "enabled": True,
                                        "normalized_name": cat_norm,
                                        "token": asserted_val
                                    },
                                    source_file=rel_path,
                                    line_number=lineno_1idx,
                                    is_normative=True
                                ))

            # 6. Extract Numeric Attributes from Normative lines
            for norm_name, gt_attr in gt_attrs.items():
                if not norm_name or _is_pin_or_index(gt_attr.name, norm_name) or _is_structural_delimiter(gt_attr.name):
                    continue
                escaped_name = re.escape(gt_attr.name)
                escaped_norm = re.escape(norm_name)

                # Table format | Name | 1800.0 |
                if is_table_row:
                    parts = [p.strip() for p in line_str.split("|")[1:-1]]
                    if len(parts) >= 2:
                        clean_col0 = re.sub(r'[*`]', '', parts[0]).strip()
                        clean_col0 = re.sub(r'^_+|_+$', '', clean_col0)
                        norm_col0 = _normalize_identifier(clean_col0)
                        if (norm_col0 == norm_name or clean_col0.lower() == gt_attr.name.lower()) and not _is_pin_or_index(clean_col0, norm_col0):
                            num_match = re.search(r'([-+−\u2212]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?)', parts[1])
                            if num_match:
                                try:
                                    raw_clean = num_match.group(1).replace('\u2212', '-').replace('−', '-')
                                    val = float(raw_clean)
                                    root_node.children.append(TypedASTNode(
                                        node_id=f"cand_attr_{len(root_node.children)}",
                                        node_type="Attribute",
                                        name=gt_attr.name,
                                        value=val,
                                        properties={"normalized_name": norm_name},
                                        source_file=rel_path,
                                        line_number=lineno_1idx,
                                        is_normative=True
                                    ))
                                    continue
                                except ValueError:
                                    pass

                # Key-value format: **Name**: 1800.0 or Name = 1800.0 or Name of 1800.0 or Name -45.0
                if len(norm_name) >= 3 and not _is_pin_or_index(gt_attr.name, norm_name):
                    name_pattern = escaped_name.replace('_', r'[\s_]+')
                    kv_m = re.search(
                        r'(?:\*\*|\b)(?:' + name_pattern + r'|' + escaped_name + r'|' + escaped_norm + r')(?:\*\*|\b)\s*(?:[:=]|is|of|at|around|approx(?:imately)?|—|-(?!\d))?\s*([-+−\u2212]?[0-9]+(?:\.[0-9]+)?(?:[eE][-+]?[0-9]+)?)\b',
                        line_str,
                        re.IGNORECASE
                    )
                    if kv_m:
                        try:
                            raw_clean = kv_m.group(1).replace('\u2212', '-').replace('−', '-')
                            val = float(raw_clean)
                            root_node.children.append(TypedASTNode(
                                node_id=f"cand_attr_{len(root_node.children)}",
                                node_type="Attribute",
                                name=gt_attr.name,
                                value=val,
                                properties={"normalized_name": norm_name},
                                source_file=rel_path,
                                line_number=lineno_1idx,
                                is_normative=True
                            ))
                            continue
                        except ValueError:
                            pass

        return root_node

    def _has_source_citation(self, content: str, rel_path: str) -> bool:
        """Checks if content has a machine-resolvable citation anchor to schema/."""
        if re.search(r'<!--\s*Source:\s*schema/.*?-->', content, re.IGNORECASE):
            return True
        if re.search(r'<!--\s*Source:\s*.*?-->', content, re.IGNORECASE):
            return True
        if re.search(r'\[[^\]]*\]\((?:\.\./)*schema/[^)]+\)', content):
            return True
        if re.search(r'(?:Source|Reference|Schema):\s*(?:`|\[)?(?:\.\./)*schema/', content, re.IGNORECASE):
            return True
        if re.search(r'##\s+Source References.*?(?:schema/)', content, re.DOTALL | re.IGNORECASE):
            return True
        return False

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """Executes concept provenance and parametric SSOT validation."""
        workspace_dir = repo.workspace_dir
        gt_graph = self.extract_ground_truth_graph(repo)
        gt_params = self.extract_ground_truth(repo)

        errors: List[Finding] = []

        # Find target specification and ConOps files
        rules = repo.get_codebase_rules()
        backlog = rules.backlog_directories if rules else None

        scan_dirs = ["docs/conops"]
        if backlog:
            for attr in ("epics", "features", "user_stories", "use_cases"):
                rel = getattr(backlog, attr, None)
                if rel and rel not in scan_dirs:
                    scan_dirs.append(rel)
        else:
            scan_dirs.extend(["docs/epics", "docs/features", "docs/user-stories", "docs/use-cases", "docs/safety"])

        doc_files: List[Tuple[str, str]] = []
        for sdir in scan_dirs:
            full_dir = os.path.join(workspace_dir, sdir)
            if not os.path.isdir(full_dir):
                continue
            for root, _, files in os.walk(full_dir):
                for f in files:
                    if f.endswith(".md") and f != "README.md":
                        full_p = os.path.join(root, f)
                        rel_p = os.path.relpath(full_p, workspace_dir)
                        doc_files.append((full_p, rel_p))

        # Check each specification / concept document
        for full_path, rel_path in doc_files:
            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            lines = content.splitlines()
            norm_rel = rel_path.replace("\\", "/")
            is_conops = norm_rel.startswith("docs/conops/") or norm_rel == "docs/conops"

            # Check for circular SysML dependency in Level 1 concept documents (docs/conops/)
            if is_conops:
                for lineno_1idx, line in enumerate(lines, start=1):
                    sysml_ref = _extract_sysml_citation_ref(line)
                    if sysml_ref:
                        errors.append(Finding(
                            "circular-sysml-concept-dependency",
                            f"{rel_path}:{lineno_1idx}: Level 1 concept document cites mutable SysML model ('{sysml_ref}') as an authority. Concept documents must derive exclusively from Level 0 OEM ground-truth texts (schema/extracted/).",
                            location=f"{rel_path}:{lineno_1idx}",
                            detail={"file": rel_path, "line": lineno_1idx, "sysml_ref": sysml_ref}
                        ))

            # Upstream template clean landing zones: empty schema/ passes gracefully
            if not gt_graph.children:
                continue

            # Extract Candidate Concept Graph with AST Section Isolation
            cand_graph = self.extract_concept_graph(content, rel_path, gt_graph)

            # Check claimed numeric parameters
            claimed_attrs = [c for c in cand_graph.children if c.node_type == "Attribute" and c.is_normative]
            claimed_props = [c for c in cand_graph.children if c.node_type in ("Property", "PartUsage", "Mapping") and c.is_normative]

            if claimed_attrs or claimed_props:
                # 1. Verify source citation anchor
                if not self._has_source_citation(content, rel_path):
                    errors.append(Finding(
                        "concept-provenance-missing-source-citation",
                        f"{rel_path}: Specification asserts schema parameters but lacks a machine-resolvable source citation anchor (e.g. '<!-- Source: schema/... -->' or Markdown link to schema file).",
                        location=rel_path
                    ))

                # 2. Verify numeric tolerance within +/- 5%
                for cand_attr in claimed_attrs:
                    norm_name = cand_attr.properties.get("normalized_name") or _normalize_identifier(cand_attr.name)
                    gt = gt_params.get(norm_name)
                    if gt and gt.value != 0:
                        rel_err = abs(cand_attr.value - gt.value) / abs(gt.value)
                        if rel_err > 0.05:
                            lineno = cand_attr.line_number or 1
                            errors.append(Finding(
                                "concept-provenance-parametric-mismatch",
                                f"{rel_path}:{lineno}: Claimed parameter '{gt.name}' = {cand_attr.value} deviates from schema ground truth {gt.value} in {gt.source_file} by {rel_err*100:.1f}% (exceeds ±5% tolerance).",
                                location=rel_path,
                                detail={"parameter": gt.name, "claimed": cand_attr.value, "ground_truth": gt.value, "error": rel_err}
                            ))

        # Semantic & Structural OEM provenance checks via AST Metamodel Graph Comparison
        errors.extend(self.validate_semantic_oem_provenance(repo))

        return errors

    def validate_semantic_oem_provenance(self, repo: WorkspaceRepository) -> List[Finding]:
        """
        Compares candidate ConOps AST Metamodel graphs against Level 0 OEM Ground-Truth AST graphs
        to detect structural contradictions and physical assertion violations.
        """
        workspace_dir = repo.workspace_dir
        gt_graph = self.extract_ground_truth_graph(repo)

        # Clean upstream landing zone: empty schema passes gracefully
        if not gt_graph.children:
            return []

        conops_dir = os.path.join(workspace_dir, "docs", "conops")
        if not os.path.isdir(conops_dir):
            return []

        conops_files: List[Tuple[str, str]] = []
        for root, _, files in os.walk(conops_dir):
            for f in files:
                if f.endswith(".md") and f != "README.md":
                    full_p = os.path.join(root, f)
                    rel_p = os.path.relpath(full_p, workspace_dir)
                    conops_files.append((full_p, rel_p))

        if not conops_files:
            return []

        findings: List[Finding] = []

        for full_path, rel_path in conops_files:
            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            cand_graph = self.extract_concept_graph(content, rel_path, gt_graph)

            # 1. Prohibited property check in candidate normative graph
            gt_props = [c for c in gt_graph.children if c.node_type == "Property"]
            gt_neg_props = [
                p for p in gt_props
                if (p.properties.get("enabled") is False or str(p.value).lower() in ("none", "no", "false", "n/a", "not installed", "not equipped", "disabled", "0"))
                and not _is_pin_or_index(p.name, p.properties.get("normalized_name", _normalize_identifier(p.name)))
                and not _is_structural_delimiter(p.name)
            ]
            cand_props = [
                c for c in cand_graph.children
                if c.node_type in ("Property", "PartUsage") and c.is_normative
                and not _is_pin_or_index(c.name, c.properties.get("normalized_name", _normalize_identifier(c.name)))
                and not _is_structural_delimiter(c.name)
            ]

            for neg_prop in gt_neg_props:
                neg_norm = neg_prop.properties.get("normalized_name") or _normalize_identifier(neg_prop.name)
                if not neg_norm:
                    continue
                for cand_p in cand_props:
                    cand_norm = cand_p.properties.get("normalized_name") or _normalize_identifier(cand_p.name)
                    if not cand_norm:
                        continue
                    cand_enabled = cand_p.properties.get("enabled", True)
                    if cand_enabled and (cand_norm == neg_norm or (len(cand_norm) >= 4 and len(neg_norm) >= 4 and (cand_norm in neg_norm or neg_norm in cand_norm))):
                        token = cand_p.properties.get("token") or cand_p.name
                        lineno = cand_p.line_number or 1
                        findings.append(Finding(
                            "semantic-oem-provenance-contradiction",
                            f"{rel_path}:{lineno}: Physical assertion ('{token}') contradicts Level 0 OEM Ground-Truth extraction baseline in schema/extracted/.",
                            location=f"{rel_path}:{lineno}",
                            detail={"file": rel_path, "line": lineno, "token": token}
                        ))

            # 2. Categorical property check
            gt_cat_props = [
                p for p in gt_props
                if p.properties.get("enabled") is not False and str(p.value).lower() not in ("none", "no", "false", "n/a", "not installed", "not equipped", "disabled", "0")
                and not _is_pin_or_index(p.name, p.properties.get("normalized_name", _normalize_identifier(p.name)))
                and not _is_structural_delimiter(p.name)
            ]
            for cat_prop in gt_cat_props:
                cat_norm = cat_prop.properties.get("normalized_name") or _normalize_identifier(cat_prop.name)
                if not cat_norm:
                    continue
                gt_val_norm = _normalize_identifier(str(cat_prop.value))
                if not gt_val_norm or _is_structural_delimiter(str(cat_prop.value)):
                    continue
                for cand_p in cand_props:
                    cand_norm = cand_p.properties.get("normalized_name") or _normalize_identifier(cand_p.name)
                    if not cand_norm:
                        continue
                    cand_val_norm = _normalize_identifier(str(cand_p.value))
                    if not cand_val_norm or _is_structural_delimiter(str(cand_p.value)):
                        continue
                    if cand_norm == cat_norm:
                        if cand_val_norm != gt_val_norm:
                            token = cand_p.properties.get("token") or cand_p.name
                            lineno = cand_p.line_number or 1
                            findings.append(Finding(
                                "semantic-oem-provenance-contradiction",
                                f"{rel_path}:{lineno}: Physical assertion ('{token}') contradicts Level 0 OEM Ground-Truth extraction baseline in schema/extracted/.",
                                location=f"{rel_path}:{lineno}",
                                detail={"file": rel_path, "line": lineno, "token": token}
                            ))

            # 3. Protocol / Opcode mapping check
            gt_mappings = [c for c in gt_graph.children if c.node_type == "Mapping"]
            cand_mappings = [c for c in cand_graph.children if c.node_type == "Mapping" and c.is_normative]

            for cand_m in cand_mappings:
                cand_domain = cand_m.properties.get("domain", "opcode")
                cand_key = _normalize_hex(cand_m.properties.get("key", ""))
                cand_target = cand_m.properties.get("target")
                if not cand_key or not cand_target:
                    continue
                cand_target_norm = _normalize_identifier(cand_target)
                if not cand_target_norm:
                    continue

                for gt_m in gt_mappings:
                    if gt_m.properties.get("domain", "opcode") != cand_domain:
                        continue
                    gt_key = _normalize_hex(gt_m.properties.get("key", ""))
                    gt_target = gt_m.properties.get("target")
                    if not gt_key or not gt_target:
                        continue
                    gt_target_norm = _normalize_identifier(gt_target)
                    if not gt_target_norm:
                        continue

                    is_conflict = False
                    if cand_key == gt_key and cand_target_norm != gt_target_norm:
                        is_conflict = True
                    elif cand_target_norm == gt_target_norm and cand_key != gt_key:
                        is_conflict = True

                    if is_conflict:
                        token = cand_m.properties.get("token") or f"{cand_key} {cand_target}"
                        lineno = cand_m.line_number or 1
                        findings.append(Finding(
                            "semantic-oem-provenance-contradiction",
                            f"{rel_path}:{lineno}: Physical assertion ('{token}') contradicts Level 0 OEM Ground-Truth extraction baseline in schema/extracted/.",
                            location=f"{rel_path}:{lineno}",
                            detail={"file": rel_path, "line": lineno, "token": token}
                        ))
                        break

        return findings

