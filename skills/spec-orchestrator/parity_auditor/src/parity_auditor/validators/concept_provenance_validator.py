"""
Concept Provenance & Parametric SSOT Validator.

Enforces pure schema-driven parameter provenance and numeric tolerance verification:
1. Dynamically scans schema/*.sysml and schema/extracted/*.md in the workspace.
2. Extracts attribute definitions, ports, and table key-values to build a dynamic ground-truth parameter dictionary.
3. Validates that numeric assertions in docs/conops/ and specifications match ground truth within +/- 5% tolerance.
4. Verifies specification claims have machine-resolvable source citation anchors (e.g. <!-- Source: schema/... --> or markdown links).
5. Enforces directional authority: Level 1 concept documents in docs/conops/ must derive exclusively from Level 0 OEM ground truth (schema/extracted/) and cannot cite mutable SysML models (.sysml).
6. Gracefully handles clean upstream landing zones (empty schema/).
"""

import os
import re
from dataclasses import dataclass
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


def _normalize_identifier(name: str) -> str:
    """Normalize identifier by lowercasing and stripping non-alphanumeric characters."""
    return re.sub(r'[^a-z0-9]', '', name.lower())


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


class ConceptProvenanceValidator(IValidator):
    """Pure schema-driven Concept Provenance Validator."""

    def extract_ground_truth(self, repo: WorkspaceRepository) -> Dict[str, GroundTruthParameter]:
        """
        Dynamically scans schema/ for .sysml and extracted markdown files
        to construct the dynamic ground-truth parameter dictionary.
        """
        workspace_dir = repo.workspace_dir
        schema_dir = os.path.join(workspace_dir, "schema")
        if not os.path.isdir(schema_dir):
            return {}

        params: Dict[str, GroundTruthParameter] = {}

        # 1. Scan .sysml files
        for root, _, files in os.walk(schema_dir):
            for f in files:
                if f.endswith(".sysml"):
                    filepath = os.path.join(root, f)
                    rel_path = os.path.relpath(filepath, workspace_dir)
                    self._extract_sysml_parameters(filepath, rel_path, params)

        # 2. Scan extracted markdown files in schema/extracted or schema/
        for root, _, files in os.walk(schema_dir):
            for f in files:
                if f.endswith(".md") and f != "README.md":
                    filepath = os.path.join(root, f)
                    rel_path = os.path.relpath(filepath, workspace_dir)
                    self._extract_markdown_parameters(filepath, rel_path, params)

        return params

    def _extract_sysml_parameters(self, filepath: str, rel_path: str, params: Dict[str, GroundTruthParameter]) -> None:
        """Parses attribute declarations from SysML file into params dict."""
        try:
            with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
        except Exception:
            return

        # Pattern for: attribute [def] <name> [: <Type>] = <value> [unit];
        attr_pattern = re.compile(
            r'\battribute\s+(?:def\s+)?([a-zA-Z0-9_]+)(?:\s*:\s*[a-zA-Z0-9_<>:]+)?\s*=\s*([^;]+);',
            re.MULTILINE
        )
        for match in attr_pattern.finditer(content):
            name = match.group(1).strip()
            raw_val = match.group(2).strip()

            num_match = re.search(r'([-+]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?)', raw_val)
            if num_match:
                try:
                    num_val = float(num_match.group(1))
                    unit_m = re.search(r'\[([a-zA-Z0-9_/\^°%]+)\]', raw_val)
                    unit = unit_m.group(1) if unit_m else None
                    if not unit:
                        after_num = raw_val[num_match.end():].strip()
                        unit_str = re.match(r'([a-zA-Z°%µΩ][a-zA-Z0-9_/\^°%µΩ]*)', after_num)
                        if unit_str:
                            unit = unit_str.group(1)

                    norm = _normalize_identifier(name)
                    gt = GroundTruthParameter(
                        name=name,
                        normalized_name=norm,
                        value=num_val,
                        raw_value=raw_val,
                        unit=unit,
                        source_file=rel_path
                    )
                    params[norm] = gt
                    params[name.lower()] = gt
                except ValueError:
                    pass

    def _extract_markdown_parameters(self, filepath: str, rel_path: str, params: Dict[str, GroundTruthParameter]) -> None:
        """Parses tables and key-value lists from extracted schema markdown files into params dict."""
        try:
            with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
        except Exception:
            return

        # Scan markdown tables
        lines = content.splitlines()
        for line in lines:
            line_str = line.strip()
            if line_str.startswith("|") and line_str.endswith("|"):
                parts = [p.strip() for p in line_str.split("|")[1:-1]]
                if len(parts) >= 2:
                    k_raw = parts[0]
                    v_raw = parts[1]
                    clean_k = re.sub(r'[*`]', '', k_raw).strip()
                    # Strip leading/trailing formatting underscores like __key__
                    clean_k = re.sub(r'^_+|_+$', '', clean_k)
                    num_match = re.search(r'([-+]?\d+(?:\.\d+)?(?:[eE][-+]?\d+)?)', v_raw)
                    if clean_k and num_match:
                        try:
                            num_val = float(num_match.group(1))
                            norm = _normalize_identifier(clean_k)
                            if norm and norm not in ("parameter", "property", "attribute", "key", "metric", "item"):
                                gt = GroundTruthParameter(
                                    name=clean_k,
                                    normalized_name=norm,
                                    value=num_val,
                                    raw_value=v_raw,
                                    unit=None,
                                    source_file=rel_path
                                )
                                params[norm] = gt
                                params[clean_k.lower()] = gt
                        except ValueError:
                            pass

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
        ground_truth = self.extract_ground_truth(repo)

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
            if not ground_truth:
                continue

            # Check if doc makes claims referencing ground truth parameters
            claimed_parameters: List[Tuple[GroundTruthParameter, float, int]] = []

            for lineno_1idx, line in enumerate(lines, start=1):
                line_clean = line.strip()
                if not line_clean:
                    continue

                for norm_name, gt in ground_truth.items():
                    # Avoid duplicate checks for alias keys
                    if norm_name != gt.normalized_name:
                        continue

                    # Search for pattern: <name> [: = is of at around] <num>
                    escaped_name = re.escape(gt.name)
                    escaped_norm = re.escape(norm_name)

                    # 1. Table format | Name | 1800.0 |
                    table_m = re.search(
                        r'\|\s*(?:' + escaped_name + r'|' + escaped_norm + r')\s*\|\s*([0-9]+(?:\.[0-9]+)?)\b',
                        line_clean,
                        re.IGNORECASE
                    )
                    if table_m:
                        try:
                            val = float(table_m.group(1))
                            claimed_parameters.append((gt, val, lineno_1idx))
                            continue
                        except ValueError:
                            pass

                    # 2. Key-value format: **Name**: 1800.0 or Name = 1800.0 or Name of 1800.0
                    kv_m = re.search(
                        r'(?:\*\*|\b)(?:' + escaped_name + r'|' + escaped_norm + r')(?:\*\*|\b)\s*(?:[:=]|is|of|at|around|approx(?:imately)?|—|-)\s*([0-9]+(?:\.[0-9]+)?)\b',
                        line_clean,
                        re.IGNORECASE
                    )
                    if kv_m:
                        try:
                            val = float(kv_m.group(1))
                            claimed_parameters.append((gt, val, lineno_1idx))
                            continue
                        except ValueError:
                            pass

            if claimed_parameters:
                # 1. Verify source citation anchor
                if not self._has_source_citation(content, rel_path):
                    errors.append(Finding(
                        "concept-provenance-missing-source-citation",
                        f"{rel_path}: Specification asserts schema parameters but lacks a machine-resolvable source citation anchor (e.g. '<!-- Source: schema/... -->' or Markdown link to schema file).",
                        location=rel_path
                    ))

                # 2. Verify numeric tolerance within +/- 5%
                for gt, doc_val, lineno in claimed_parameters:
                    if gt.value != 0:
                        rel_err = abs(doc_val - gt.value) / abs(gt.value)
                        if rel_err > 0.05:
                            errors.append(Finding(
                                "concept-provenance-parametric-mismatch",
                                f"{rel_path}:{lineno}: Claimed parameter '{gt.name}' = {doc_val} deviates from schema ground truth {gt.value} in {gt.source_file} by {rel_err*100:.1f}% (exceeds ±5% tolerance).",
                                location=rel_path,
                                detail={"parameter": gt.name, "claimed": doc_val, "ground_truth": gt.value, "error": rel_err}
                            ))

        # Semantic OEM provenance checks
        errors.extend(self.validate_semantic_oem_provenance(repo))

        return errors

    def validate_semantic_oem_provenance(self, repo: WorkspaceRepository) -> List[Finding]:
        """
        Scans concept markdown documents in docs/conops/ for physical hardware
        contradiction tokens against Level 0 OEM ground-truth extractions in schema/extracted/.
        """
        workspace_dir = repo.workspace_dir
        extracted_dir = os.path.join(workspace_dir, "schema", "extracted")
        if not os.path.isdir(extracted_dir):
            return []

        extracted_files: List[str] = []
        for root, _, files in os.walk(extracted_dir):
            for f in files:
                if f.endswith(".md") and f != "README.md":
                    extracted_files.append(os.path.join(root, f))

        if not extracted_files:
            return []

        extracted_texts: List[str] = []
        for ef in extracted_files:
            try:
                with open(ef, "r", encoding="utf-8", errors="ignore") as f:
                    extracted_texts.append(f.read())
            except Exception:
                continue

        if not extracted_texts:
            return []

        oem_text = "\n\n".join(extracted_texts)

        # Baseline conditions from OEM ground truth
        oem_no_recovery = bool(re.search(
            r'\brecovery(?:\s+system)?\s*[:|]?\s*(?:is\s*)?(?:no\b|none\b|false\b|n/a\b|not\s+equipped|not\s+installed)',
            oem_text,
            re.IGNORECASE
        ))
        oem_ruddervator = bool(re.search(r'\bruddervators?\b', oem_text, re.IGNORECASE))
        has_oem_pbit_11 = bool(re.search(r'(?:0x11\b[^\n]*?\bPBIT\b|\bPBIT\b[^\n]*?0x11\b)', oem_text, re.IGNORECASE))
        has_oem_exchange_10 = bool(re.search(r'(?:0x10\b[^\n]*?\b(?:EXCHANGE|KEY_EXCHANGE)\b|\b(?:EXCHANGE|KEY_EXCHANGE)\b[^\n]*?0x10\b)', oem_text, re.IGNORECASE))
        oem_esad_icd = has_oem_pbit_11 or has_oem_exchange_10

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

            lines = content.splitlines()
            for lineno, line in enumerate(lines, start=1):
                line_clean = line.strip()
                if not line_clean:
                    continue

                # a) Asserting recovery systems when OEM extraction declares Recovery system: No / None
                if oem_no_recovery:
                    is_denial = bool(re.search(
                        r'\b(?:no|none|without|never|not\s+equipped|not\s+installed|not\s+included)\b.*?\b(?:parachute|chute)\b',
                        line,
                        re.IGNORECASE
                    ))
                    if not is_denial:
                        m_rec = re.search(
                            r'\b(ballistic\s+parachute|recovery\s+parachute|parachute\s+recovery|parachute\s+system|emergency\s+parachute|pyrotechnic\s+parachute|recovery\s+chute|ballistic\s+chute|parachute)\b',
                            line,
                            re.IGNORECASE
                        )
                        if m_rec:
                            token = m_rec.group(1)
                            findings.append(Finding(
                                "semantic-oem-provenance-contradiction",
                                f"{rel_path}:{lineno}: Physical assertion ('{token}') contradicts Level 0 OEM Ground-Truth extraction baseline in schema/extracted/.",
                                location=f"{rel_path}:{lineno}",
                                detail={"file": rel_path, "line": lineno, "token": token}
                            ))
                            continue

                # b) Asserting incorrect control surface taxonomy when OEM declares ruddervator
                if oem_ruddervator:
                    is_denial = bool(re.search(
                        r'\b(?:no|none|without|never|not\s+equipped|not\s+installed|not\s+included)\b.*?\belevons?\b',
                        line,
                        re.IGNORECASE
                    ))
                    if not is_denial:
                        m_elevon = re.search(
                            r'\b(symmetrical\s+elevons?|elevon\s+actuators?|elevon\s+controls?|elevon\s+surfaces?|elevons?)\b',
                            line,
                            re.IGNORECASE
                        )
                        if m_elevon:
                            token = m_elevon.group(1)
                            findings.append(Finding(
                                "semantic-oem-provenance-contradiction",
                                f"{rel_path}:{lineno}: Physical assertion ('{token}') contradicts Level 0 OEM Ground-Truth extraction baseline in schema/extracted/.",
                                location=f"{rel_path}:{lineno}",
                                detail={"file": rel_path, "line": lineno, "token": token}
                            ))
                            continue

                # c) Asserting inverted ESAD RS-485 opcodes
                if oem_esad_icd:
                    m_inv_pbit = re.search(
                        r'\b((?:Opcode\s+)?0x10\s*[:\-—|]?\s*PBIT|PBIT\s*(?:\(0x10\)|opcode\s+0x10|[:\-—|]\s*0x10)|0x10\s+PBIT)\b',
                        line,
                        re.IGNORECASE
                    )
                    if m_inv_pbit:
                        token = m_inv_pbit.group(1)
                        findings.append(Finding(
                            "semantic-oem-provenance-contradiction",
                            f"{rel_path}:{lineno}: Physical assertion ('{token}') contradicts Level 0 OEM Ground-Truth extraction baseline in schema/extracted/.",
                            location=f"{rel_path}:{lineno}",
                            detail={"file": rel_path, "line": lineno, "token": token}
                        ))
                        continue

                    m_inv_exch = re.search(
                        r'\b((?:Opcode\s+)?0x11\s*[:\-—|]?\s*(?:Key\s+)?Exchange|(?:Key\s+)?Exchange\s*(?:\(0x11\)|opcode\s+0x11|[:\-—|]\s*0x11)|0x11\s+Exchange)\b',
                        line,
                        re.IGNORECASE
                    )
                    if m_inv_exch:
                        token = m_inv_exch.group(1)
                        findings.append(Finding(
                            "semantic-oem-provenance-contradiction",
                            f"{rel_path}:{lineno}: Physical assertion ('{token}') contradicts Level 0 OEM Ground-Truth extraction baseline in schema/extracted/.",
                            location=f"{rel_path}:{lineno}",
                            detail={"file": rel_path, "line": lineno, "token": token}
                        ))
                        continue

        return findings
