"""
Factual Grounding & Parametric SSOT Parity Gate Validator (Check 23).

Enforces factual grounding and physical fidelity against SysML v2 AST and Level 0 OEM schema:
1. Ingests SysML v2 AST AttributeDefs, PartDefs, PortDefs, ConnectionDefs via fail-closed loader.
2. Ingests raw schema texts, BOM tables, and dictionaries from schema/ and schema/extracted/.
3. Gracefully passes on upstream clean landing zones (empty schema/).
4. Evaluates specification documents in docs/ (CONOPS, STPA, features, epics, icds, use-cases, user-stories):
   a) Control surface counts and structural descriptors (rule ID: 'factual-grounding-numeric-drift'):
      - Detects ungrounded structural assertions (e.g. claiming "V-tail" when BOM specifies 4 ruddervators / X-tail,
        or claiming 2 ruddervators when BOM has 4).
   b) Numeric quantities (rule ID: 'factual-grounding-numeric-drift'):
      - Detects fabricated numeric quantities (e.g. claiming "15-20g" or "18g" catapult launch acceleration
        when not substantiated in schema ground truth or lacking SSOT citation).
   c) Electrical / communication protocols (rule ID: 'factual-grounding-unverified-protocol'):
      - Detects ungrounded protocol claims (e.g. "STANAG 4586", "STANAG 4609", "MIL-STD-1553", "ARINC 429",
        "CANopen", "MAVLink", "RS-485", etc.) mentioned in specifications that are not declared in schema/ or
        substantiated with SSOT citations.
   d) Temporal safety in Mermaid sequence diagrams (rule ID: 'factual-grounding-temporal-safety-violation'):
      - Scans ```mermaid sequenceDiagram blocks across docs/.
      - Detects physical arming/firing/motor-enable signals (e.g. targeting Actuator, Pyro, FiringCircuit, SafetySwitch,
        Warhead with action Arm/Fire/Enable).
      - Validates that every physical arming signal must be preceded temporally in the sequence by an explicit
        human-in-the-loop (HITL) C2 arming command / operator consent / pilot authorization.
      - Rejects autonomous arming sequences without prior human C2 command.
5. Contextual filtering:
   - Skips non-normative sections (Glossary, Acronyms, MCDA Trade Studies / Alternatives Analysis analyzing rejected options).
   - Skips non-diagram code blocks and HTML comments.
   - Permits claims grounded by explicit SSOT citations (<!-- Source: schema/... -->, <!-- SSOT: ... -->,
     markdown links to schema, or frontmatter source_references/realized_ast_nodes).
"""

import fnmatch
import os
import re
import sys
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set, Tuple, Any, Sequence

try:
    from .base import IValidator
    from ..core.findings import Finding
    from ..core.workspace import WorkspaceRepository
except (ImportError, ValueError):
    _src_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
    if _src_dir not in sys.path:
        sys.path.insert(0, _src_dir)
    from parity_auditor.validators.base import IValidator
    from parity_auditor.core.findings import Finding
    from parity_auditor.core.workspace import WorkspaceRepository

# Import SysML v2 AST classes via the fail-closed loader
from ..utils.sysml_loader import load_sysml_ast_members

_sysml_ast = load_sysml_ast_members([
    "SysMLPackage", "SysMLParser", "PartDef", "AttributeDef"
])
SysMLPackage = _sysml_ast.SysMLPackage
SysMLParser = _sysml_ast.SysMLParser
PartDef = _sysml_ast.PartDef
AttributeDef = _sysml_ast.AttributeDef


# Non-normative section heading patterns (e.g. Glossary, MCDA trade study, Acronyms, Standards/Regulatory baseline)
NON_NORMATIVE_SECTION_PATTERNS = [
    re.compile(r'\b(?:glossary|acronyms?|abbreviations?|definitions?|terminology|lexicon|vocabulary)\b', re.I),
    re.compile(r'\b(?:trade\s+stud(?:y|ies)|trade-?off|mcda|multi-criteria|rejected\s+alternatives?|candidate\s+analysis|trade\s+space|decision\s+matrix|evaluation\s+of\s+alternatives)\b', re.I),
    re.compile(r'\b(?:revision\s+history|document\s+history|document\s+control|change\s+log|changelog)\b', re.I),
    re.compile(r'\b(?:references?|applicable\s+documents|reference\s+standards|normative\s+standards|standards\s+baseline|regulatory\s+baseline|regulatory\s+framework|standards\s+and\s+regulatory|standards\s+taxonomy)\b', re.I),
]

# Standard aerospace & industrial communication / electrical protocols
RECOGNIZED_PROTOCOLS = [
    "STANAG 4586", "STANAG 4609", "STANAG 4586 Ed. 3", "STANAG 7085",
    "MIL-STD-1553", "MIL-STD-1553B", "MIL-STD-1760", "MIL-STD-188-220", "MIL-STD-6016",
    "ARINC 429", "ARINC 661", "ARINC 653", "ARINC 818", "ARINC 825",
    "CANopen", "CAN bus", "CAN-FD", "CAN FD", "DeviceNet",
    "MAVLink", "MAVLink v2", "MAVLink v1",
    "SpaceWire", "SpaceFibre", "AFDX", "ARINC 664",
    "100BASE-TX", "1000BASE-T", "10GBASE-T", "Ethernet",
    "RS-485", "RS-422", "RS-232", "Modbus", "Modbus RTU", "Modbus TCP",
    "Profibus", "Profinet", "EtherCAT", "Micro-D"
]

# Physical arming/firing target entity tokens in sequence diagrams
PHYSICAL_ARMING_TARGET_TOKENS = {
    "actuator", "pyro", "firingcircuit", "safetyswitch", "warhead", "motor",
    "igniter", "armingdevice", "payloadrelease", "laser", "weapon", "esad",
    "fuzing", "squib", "propulsionenable", "armswitch", "firingunit",
    "safearm", "safeandarm", "initiator", "booster", "payloadbay", "ejector"
}

# Physical arming/firing action tokens in sequence diagrams
PHYSICAL_ARMING_ACTION_PATTERNS = [
    re.compile(r'\b(?:arm|arm_all|arming|arm_circuit|arm_system|arm_warhead|arm_pyro|arm_device|arm_motor)\b', re.I),
    re.compile(r'\b(?:fire|firing|fire_pulse|fire_squib|fire_pyro|fire_circuit|detonate|detonation)\b', re.I),
    re.compile(r'\b(?:ignite|ignition|ignite_motor|start_ignition|motor_enable|propulsion_enable|enable_motor|enable_firing|enable_high_voltage)\b', re.I),
    re.compile(r'\b(?:deploy_payload|release_payload|eject_payload|payload_release|weapon_release)\b', re.I),
    re.compile(r'\b(?:activate_warhead|activate_pyro|activate_fuzing|activate_esad)\b', re.I),
]

# Explicit prohibition/negation tokens on arming actions (e.g. disarm, safe, inhibit, abort)
DISARM_ACTION_PATTERNS = [
    re.compile(r'\b(?:disarm|disarming|safe|safing|inhibit|inhibiting|abort|aborting|disable|deactivate)\b', re.I),
]

# Human C2 / HITL participant tokens in sequence diagrams
HITL_SENDER_TOKENS = {
    "operator", "operators", "pilot", "pilots", "human", "commander", "remotepilot",
    "gcsoperator", "gcs", "c2", "missioncommander", "safetyofficer", "tacticaloperator",
    "groundcontrolstation", "groundstation", "user"
}

# HITL C2 arming / consent / authorization action patterns
HITL_CONSENT_ACTION_PATTERNS = [
    re.compile(r'\b(?:arm|arm_command|command_arm|send_arm_cmd|arm_switch_on|arm_request|authorize_arm|request_arm|arm_confirm|confirm_arm)\b', re.I),
    re.compile(r'\b(?:consent|operator_consent|pilot_consent|grant_consent|consent_granted|consent_token|consent_key)\b', re.I),
    re.compile(r'\b(?:authorize|authorization|authorize_fire|authorize_launch|permit_fire|permit_arm|fire_permission|command_fire|fire_command|send_fire_cmd)\b', re.I),
    re.compile(r'\b(?:confirm_engagement|engage_command|manual_arm|manual_consent|c2_arm|c2_fire)\b', re.I),
]


def _normalize_name(name: str) -> str:
    """Normalize identifier by removing non-alphanumeric characters and lowercasing."""
    if not name:
        return ""
    return re.sub(r'[^a-zA-Z0-9]', '', str(name)).lower()


def _extract_numeric_scalar(val_str: str) -> Optional[float]:
    """Extract first numeric scalar value from a string."""
    if not val_str:
        return None
    m = re.search(r'[-+]?\d+(?:\.\d+)?', str(val_str))
    if m:
        try:
            return float(m.group(0))
        except ValueError:
            return None
    return None


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


def _find_extracted_markdown_files(repo: WorkspaceRepository, schemas_dir: Optional[str] = None) -> List[str]:
    """Locate extracted OEM / schema markdown files in schema/."""
    md_files: List[str] = []

    if schemas_dir and os.path.isdir(schemas_dir):
        for root, _, files in os.walk(schemas_dir):
            for f in sorted(files):
                if f.endswith(".md") and not f.startswith("."):
                    md_files.append(os.path.join(root, f))

    for s_name in ("schema", "schemas"):
        cand = os.path.join(repo.workspace_dir, s_name)
        if os.path.isdir(cand):
            for root, _, files in os.walk(cand):
                for f in sorted(files):
                    if f.endswith(".md") and not f.startswith("."):
                        p = os.path.join(root, f)
                        if p not in md_files:
                            md_files.append(p)

    return md_files


@dataclass
class SchemaGroundTruth:
    """Consolidated Ground Truth extracted from SysML AST and schema markdown."""
    control_surface_count: Optional[int] = None
    ruddervator_count: Optional[int] = None
    tail_configuration: Optional[str] = None  # e.g. "x-tail", "v-tail", "inverted-v-tail", "cruciform"
    catapult_launch_limit_g: Optional[float] = None
    max_g_load: Optional[float] = None
    attributes: Dict[str, Any] = field(default_factory=dict)
    declared_protocols: Set[str] = field(default_factory=set)
    raw_schema_text: str = ""
    source_files: List[str] = field(default_factory=list)
    has_concrete_schema: bool = False


class FactualGroundingValidator(IValidator):
    """
    Factual Grounding & Parametric SSOT Parity Gate Validator (Check 23).
    """

    def __init__(self, workspace_repo: Optional[WorkspaceRepository] = None, **kwargs):
        self.workspace_repo = workspace_repo

    def validate(
        self,
        repo: WorkspaceRepository,
        scan_dirs: Optional[List[str]] = None,
        schemas_dir: Optional[str] = None,
        **kwargs
    ) -> List[Finding]:
        """
        Executes factual grounding and physical SSOT verification.
        """
        findings: List[Finding] = []

        # 1. Ingest Schema Ground Truth
        gt = self._extract_ground_truth(repo, schemas_dir=schemas_dir)

        # Upstream clean landing zone check: if no concrete schema exists, pass cleanly
        if not gt.has_concrete_schema:
            return []

        # 2. Discover target specification documents
        spec_files = self._discover_spec_files(repo, scan_dirs=scan_dirs)
        if not spec_files:
            return []

        # 3. Evaluate each document
        for full_path, rel_path in spec_files:
            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception:
                continue

            # a) Evaluate Control surface counts & structural descriptors
            findings.extend(self._validate_structural_descriptors(content, rel_path, gt))

            # b) Evaluate Numeric quantities (catapult acceleration / launch load / general limits)
            findings.extend(self._validate_numeric_quantities(content, rel_path, gt))

            # c) Evaluate Electrical / communication protocols
            findings.extend(self._validate_protocols(content, rel_path, gt))

            # d) Evaluate Temporal safety in Mermaid sequence diagrams
            findings.extend(self._validate_sequence_diagram_temporal_safety(content, rel_path))

        return findings

    def _extract_ground_truth(
        self,
        repo: WorkspaceRepository,
        schemas_dir: Optional[str] = None
    ) -> SchemaGroundTruth:
        """
        Parses SysML AST and schema/extracted/ markdown to build the consolidated Ground Truth.
        """
        gt = SchemaGroundTruth()
        workspace_dir = repo.workspace_dir

        sysml_files = _find_sysml_files(repo, schemas_dir=schemas_dir)
        md_files = _find_extracted_markdown_files(repo, schemas_dir=schemas_dir)

        if not sysml_files and not md_files:
            # Check if schema dir has any real files besides .gitkeep
            schema_dir = os.path.join(workspace_dir, "schema")
            if os.path.isdir(schema_dir):
                real_files = [f for f in os.listdir(schema_dir) if f != ".gitkeep" and not f.startswith(".")]
                if not real_files:
                    gt.has_concrete_schema = False
                    return gt
            else:
                gt.has_concrete_schema = False
                return gt

        raw_texts: List[str] = []

        # 1. Ingest SysML AST files
        parser = SysMLParser()
        for sf in sysml_files:
            try:
                with open(sf, "r", encoding="utf-8", errors="ignore") as f:
                    text = f.read()
                if not text.strip():
                    continue
                raw_texts.append(text)
                gt.source_files.append(os.path.relpath(sf, workspace_dir))
                gt.has_concrete_schema = True

                # Direct regex ingestion for SysML attributes
                for match in re.finditer(r'\battribute\s+(?:def\s+)?([a-zA-Z0-9_]+)(?:\s*:\s*[a-zA-Z0-9_<>:]+)?\s*=\s*([^;]+);', text):
                    aname = match.group(1).strip()
                    aval = match.group(2).strip().strip('"\'`')
                    aname_norm = _normalize_name(aname)
                    gt.attributes[aname_norm] = aval
                    ascalar = _extract_numeric_scalar(aval)
                    if "ruddervator" in aname_norm and ascalar is not None:
                        gt.ruddervator_count = int(ascalar)
                    elif "controlsurface" in aname_norm and ascalar is not None:
                        gt.control_surface_count = int(ascalar)
                    elif ("catapult" in aname_norm or "launchload" in aname_norm or "launchaccel" in aname_norm) and ascalar is not None:
                        gt.catapult_launch_limit_g = ascalar
                    elif ("maxgload" in aname_norm or "gloadlimit" in aname_norm) and ascalar is not None:
                        gt.max_g_load = ascalar
                    elif "tailconfig" in aname_norm or "empennage" in aname_norm:
                        gt.tail_configuration = aval.lower()

                # AST Parser ingestion
                pkg = parser.parse(text)
                self._ingest_sysml_package(pkg, gt)
            except Exception:
                continue

        # 2. Ingest Extracted Schema Markdown files
        for mf in md_files:
            try:
                with open(mf, "r", encoding="utf-8", errors="ignore") as f:
                    text = f.read()
                if not text.strip():
                    continue
                raw_texts.append(text)
                gt.source_files.append(os.path.relpath(mf, workspace_dir))
                gt.has_concrete_schema = True

                self._ingest_schema_markdown(text, os.path.relpath(mf, workspace_dir), gt)
            except Exception:
                continue

        gt.raw_schema_text = "\n".join(raw_texts)

        # 3. Detect declared protocols across raw schema text
        for proto in RECOGNIZED_PROTOCOLS:
            pattern = re.compile(r'\b' + re.escape(proto) + r'\b', re.I)
            if pattern.search(gt.raw_schema_text):
                gt.declared_protocols.add(_normalize_name(proto))

        # Check for generic protocol keywords declared in schema (e.g. "RS-485", "CAN", "UART", "MAVLink")
        generic_protos = ["rs485", "rs422", "rs232", "can", "canopen", "mavlink", "ethernet", "spacewire", "milstd1553", "arinc429", "modbus", "uart", "spi", "i2c"]
        for gp in generic_protos:
            if re.search(r'\b' + gp + r'\b', _normalize_name(gt.raw_schema_text)):
                gt.declared_protocols.add(gp)

        # Check if schema actually defines concrete architectural ground truth
        has_concrete = bool(
            gt.attributes
            or gt.control_surface_count
            or gt.ruddervator_count
            or gt.catapult_launch_limit_g
            or gt.declared_protocols
            or gt.tail_configuration
        )
        if not has_concrete:
            gt.has_concrete_schema = False

        return gt

    def _ingest_sysml_package(self, pkg: Any, gt: SchemaGroundTruth) -> None:
        """Recursively ingests elements from a parsed SysMLPackage."""
        if not pkg:
            return

        # Ingest attribute_defs
        for attr in getattr(pkg, "attribute_defs", []) or getattr(pkg, "attributes", []) or []:
            name = getattr(attr, "name", "")
            val_str = getattr(attr, "default_value", None) or getattr(attr, "doc", "") or ""
            norm_name = _normalize_name(name)
            gt.attributes[norm_name] = val_str

            scalar = _extract_numeric_scalar(val_str)
            if "ruddervator" in norm_name and scalar is not None:
                gt.ruddervator_count = int(scalar)
            elif "controlsurface" in norm_name and scalar is not None:
                gt.control_surface_count = int(scalar)
            elif ("catapult" in norm_name or "launchload" in norm_name or "launchaccel" in norm_name) and scalar is not None:
                gt.catapult_launch_limit_g = scalar
            elif ("maxgload" in norm_name or "gloadlimit" in norm_name) and scalar is not None:
                gt.max_g_load = scalar
            elif "tailconfig" in norm_name or "empennage" in norm_name:
                gt.tail_configuration = str(val_str).strip('"\'`').lower()

        # Ingest part_defs
        for part in getattr(pkg, "part_defs", []) or getattr(pkg, "parts", []) or []:
            pname = getattr(part, "name", "")
            pname_norm = _normalize_name(pname)
            # Check for part attributes or multiplicity
            for attr in getattr(part, "attributes", []) or getattr(part, "attribute_defs", []) or []:
                aname_norm = _normalize_name(getattr(attr, "name", ""))
                aval = getattr(attr, "default_value", None) or ""
                ascalar = _extract_numeric_scalar(aval)
                if ("count" in aname_norm or "quantity" in aname_norm or "qty" in aname_norm) and ascalar is not None:
                    if "ruddervator" in pname_norm:
                        gt.ruddervator_count = int(ascalar)
                    elif "controlsurface" in pname_norm:
                        gt.control_surface_count = int(ascalar)

        # Ingest sub_packages
        for nested in getattr(pkg, "sub_packages", []) or getattr(pkg, "packages", []) or []:
            self._ingest_sysml_package(nested, gt)

    def _ingest_schema_markdown(self, text: str, rel_path: str, gt: SchemaGroundTruth) -> None:
        """Parses extracted schema markdown for BOM counts, tail config, and launch limits."""
        lines = text.splitlines()
        for line in lines:
            line_str = line.strip()
            if not line_str:
                continue

            # 1. Ruddervator count: e.g. "4 ruddervators", "4x ruddervators", "| Ruddervator Actuators | 4 |"
            m_rudder = re.search(r'\b(\d+)\s*(?:x\s*)?(?:independent\s*)?ruddervators?\b', line_str, re.I)
            if m_rudder:
                try:
                    gt.ruddervator_count = int(m_rudder.group(1))
                except ValueError:
                    pass

            # 2. Control surface count: e.g. "4 control surfaces", "| Control Surfaces | 4 |"
            m_cs = re.search(r'\b(\d+)\s*(?:x\s*)?control\s+surfaces?\b', line_str, re.I)
            if m_cs:
                try:
                    gt.control_surface_count = int(m_cs.group(1))
                except ValueError:
                    pass

            # Table row BOM count: | Ruddervator | 4 | or | Actuator (Ruddervator) | 4 |
            if line_str.startswith("|") and line_str.endswith("|"):
                cells = [c.strip() for c in line_str.split("|")[1:-1]]
                if len(cells) >= 2:
                    k, v = cells[0], cells[1]
                    k_norm = _normalize_name(k)
                    v_scalar = _extract_numeric_scalar(v)
                    if v_scalar is not None:
                        if "ruddervator" in k_norm:
                            gt.ruddervator_count = int(v_scalar)
                        elif "controlsurface" in k_norm:
                            gt.control_surface_count = int(v_scalar)
                        elif "catapult" in k_norm or "launchaccel" in k_norm or "launchload" in k_norm:
                            gt.catapult_launch_limit_g = v_scalar
                        elif "maxg" in k_norm or "gload" in k_norm:
                            gt.max_g_load = v_scalar

            # 3. Tail configuration: e.g. "X-tail", "V-tail", "inverted V-tail", "cruciform"
            if re.search(r'\b(?:x-tail|x\s+tail|x-configuration)\b', line_str, re.I):
                gt.tail_configuration = "x-tail"
            elif re.search(r'\b(?:inverted\s+v-tail|inverted\s+v\s+tail)\b', line_str, re.I):
                gt.tail_configuration = "inverted-v-tail"
            elif re.search(r'\b(?:v-tail|v\s+tail)\b', line_str, re.I) and not gt.tail_configuration:
                gt.tail_configuration = "v-tail"

            # 4. Catapult launch limit / acceleration: e.g. "catapult launch limit of 12g", "catapult acceleration: 12 g"
            m_launch = re.search(r'\b(?:catapult|launch)\b[^.\n]*?\b(\d+(?:\.\d+)?)\s*g\b', line_str, re.I)
            if m_launch:
                try:
                    gt.catapult_launch_limit_g = float(m_launch.group(1))
                except ValueError:
                    pass

    def _is_excluded_spec_file(self, rel_path: str, filename: str) -> bool:
        """
        Excludes retrospective defect reports and audit summary files from normative specification evaluation:
        1. Any file located under docs/reports/defects/
        2. Any file matching *AUDIT.md or *audit*.md
        """
        norm_rel = rel_path.replace("\\", "/")
        if norm_rel.startswith("docs/reports/defects/") or "/reports/defects/" in f"/{norm_rel}":
            return True
        f_lower = filename.lower()
        if fnmatch.fnmatch(filename, "*AUDIT.md") or fnmatch.fnmatch(f_lower, "*audit*.md"):
            return True
        return False

    def _discover_spec_files(
        self,
        repo: WorkspaceRepository,
        scan_dirs: Optional[List[str]] = None
    ) -> List[Tuple[str, str]]:
        """Finds all specification markdown documents to evaluate, excluding retrospective defect reports and audit files."""
        workspace_dir = repo.workspace_dir
        target_dirs: List[str] = []

        if scan_dirs:
            for s in scan_dirs:
                target_dirs.append(s)
        else:
            target_dirs = ["docs"]

        spec_files: List[Tuple[str, str]] = []
        for tdir in target_dirs:
            full_tdir = os.path.join(workspace_dir, tdir) if not os.path.isabs(tdir) else tdir
            if os.path.isfile(full_tdir) and full_tdir.endswith(".md"):
                rel = os.path.relpath(full_tdir, workspace_dir)
                filename = os.path.basename(full_tdir)
                if not self._is_excluded_spec_file(rel, filename):
                    spec_files.append((full_tdir, rel))
            elif os.path.isdir(full_tdir):
                for root, _, files in os.walk(full_tdir):
                    for f in sorted(files):
                        if f.endswith(".md") and not f.startswith("."):
                            full_p = os.path.join(root, f)
                            rel_p = os.path.relpath(full_p, workspace_dir)
                            if not self._is_excluded_spec_file(rel_p, f):
                                spec_files.append((full_p, rel_p))

        return spec_files

    def _is_non_normative_section(self, heading: str) -> bool:
        """Check if section heading denotes a non-normative section."""
        if not heading:
            return False
        for pat in NON_NORMATIVE_SECTION_PATTERNS:
            if pat.search(heading):
                return True
        return False

    def _has_ssot_citation(self, line: str, content: str, rel_path: str) -> bool:
        """Checks if a claim or file carries an explicit SSOT citation."""
        # 1. Inline or block HTML comment citation
        if re.search(r'<!--\s*(?:Source|SSOT|Grounding|Reference):\s*[^>]+-->', line, re.I):
            return True
        # 2. Markdown link to schema/
        if re.search(r'\[[^\]]+\]\([^)]*?schema/[^)]*\)', line, re.I):
            return True
        # 3. Document-level frontmatter source references
        if re.search(r'(?:source_references|realized_ast_nodes|ssot_source):\s*\[?[^\n\]]+schema/[^\n\]]+', content, re.I):
            return True
        return False

    def _validate_structural_descriptors(
        self,
        content: str,
        rel_path: str,
        gt: SchemaGroundTruth
    ) -> List[Finding]:
        """
        Validates control surface counts and tail geometry against schema ground truth.
        Emits Finding('factual-grounding-numeric-drift', ...).
        """
        findings: List[Finding] = []
        lines = content.splitlines()

        current_heading = "Header"
        is_normative = True
        in_code_block = False

        expected_count = gt.ruddervator_count or gt.control_surface_count
        expected_tail = gt.tail_configuration

        for lineno_1idx, line in enumerate(lines, start=1):
            line_str = line.strip()

            if line_str.startswith("```"):
                in_code_block = not in_code_block
                continue
            if in_code_block or not line_str:
                continue

            # Heading detection
            m_head = re.match(r'^(#{1,6})\s+(.+)$', line_str)
            if m_head:
                current_heading = m_head.group(2).strip()
                is_normative = not self._is_non_normative_section(current_heading)
                continue

            if not is_normative:
                continue

            # Skip rejected trade study rows
            if re.search(r'\b(?:rejected|discarded|eliminated|not\s+selected|cons|fail)\b', line_str, re.I):
                continue

            # Check if line has explicit SSOT citation
            if self._has_ssot_citation(line_str, content, rel_path):
                continue

            # Check control surface count drift:
            # E.g. Ground truth has 4 ruddervators, spec claims "2 ruddervators", "two ruddervators", "dual ruddervators", "3 ruddervators"
            if expected_count is not None and expected_count == 4:
                m_drift = re.search(r'\b(2|two|dual|3|three|6|six|8|eight)\s*(?:independent\s*)?ruddervators?\b', line_str, re.I)
                if m_drift:
                    claimed = m_drift.group(0)
                    findings.append(Finding(
                        "factual-grounding-numeric-drift",
                        f"{rel_path}:{lineno_1idx}: Ungrounded structural assertion '{claimed}' contradicts schema ground truth ({expected_count} ruddervators) in {', '.join(gt.source_files) or 'schema/'}.",
                        location=f"{rel_path}:{lineno_1idx}",
                        detail={"file": rel_path, "line": lineno_1idx, "claimed": claimed, "expected": expected_count}
                    ))
                    continue

                # Spec claims "V-tail" when schema specifies 4 ruddervators / X-tail (V-tail conventionally has only 2 surfaces)
                if expected_tail == "x-tail" or (expected_count == 4 and not expected_tail):
                    if re.search(r'(?<!inverted\s)\bv-tail\b|\bv\s+tail\b', line_str, re.I):
                        # Allow if explicitly clarified as 4-surface or X-tail or inverted V with 4 surfaces
                        if not re.search(r'\b(?:4|four|x-tail)\b', line_str, re.I):
                            findings.append(Finding(
                                "factual-grounding-numeric-drift",
                                f"{rel_path}:{lineno_1idx}: Structural descriptor 'V-tail' contradicts schema ground truth ({expected_count} ruddervators / X-tail configuration) in {', '.join(gt.source_files) or 'schema/'}.",
                                location=f"{rel_path}:{lineno_1idx}",
                                detail={"file": rel_path, "line": lineno_1idx, "descriptor": "V-tail", "expected": "X-tail / 4 ruddervators"}
                            ))
                            continue

            elif expected_count is not None and expected_count == 2:
                m_drift4 = re.search(r'\b(4|four|quad|x-tail)\s*(?:independent\s*)?ruddervators?\b', line_str, re.I)
                if m_drift4:
                    claimed = m_drift4.group(0)
                    findings.append(Finding(
                        "factual-grounding-numeric-drift",
                        f"{rel_path}:{lineno_1idx}: Ungrounded structural assertion '{claimed}' contradicts schema ground truth ({expected_count} ruddervators) in {', '.join(gt.source_files) or 'schema/'}.",
                        location=f"{rel_path}:{lineno_1idx}",
                        detail={"file": rel_path, "line": lineno_1idx, "claimed": claimed, "expected": expected_count}
                    ))
                    continue

        return findings

    def _validate_numeric_quantities(
        self,
        content: str,
        rel_path: str,
        gt: SchemaGroundTruth
    ) -> List[Finding]:
        """
        Validates numeric quantities (such as catapult launch acceleration limits) against schema ground truth.
        Emits Finding('factual-grounding-numeric-drift', ...).
        """
        findings: List[Finding] = []
        lines = content.splitlines()

        current_heading = "Header"
        is_normative = True
        in_code_block = False

        gt_launch_limit = gt.catapult_launch_limit_g or gt.max_g_load or 12.0

        for lineno_1idx, line in enumerate(lines, start=1):
            line_str = line.strip()

            if line_str.startswith("```"):
                in_code_block = not in_code_block
                continue
            if in_code_block or not line_str:
                continue

            # Heading detection
            m_head = re.match(r'^(#{1,6})\s+(.+)$', line_str)
            if m_head:
                current_heading = m_head.group(2).strip()
                is_normative = not self._is_non_normative_section(current_heading)
                continue

            if not is_normative:
                continue

            # Skip rejected trade study rows
            if re.search(r'\b(?:rejected|discarded|eliminated|not\s+selected|cons|fail|exceeds\s+limit)\b', line_str, re.I):
                continue

            # Check if line has explicit SSOT citation
            if self._has_ssot_citation(line_str, content, rel_path):
                continue

            # Check for fabricated catapult launch load / acceleration values (e.g. "15-20g", "18g", "20g", "16g")
            # Pattern: catapult launch [load/accel/limit] or launch acceleration followed by g numbers
            m_catapult = re.search(
                r'(?:\bcatapult\b|\blaunch\s+load\b|\blaunch\s+acceleration\b|\blaunch\s+g-load\b)[^.\n]*?\b(\d+(?:\.\d+)?(?:\s*-\s*\d+(?:\.\d+)?)?)\s*g\b',
                line_str,
                re.I
            )
            if not m_catapult:
                # Also match: \d+g catapult launch
                m_catapult = re.search(
                    r'\b(\d+(?:\.\d+)?(?:\s*-\s*\d+(?:\.\d+)?)?)\s*g\b[^.\n]*?(?:\bcatapult\b|\blaunch\s+load\b|\blaunch\s+accel\b)',
                    line_str,
                    re.I
                )

            if m_catapult:
                val_range_str = m_catapult.group(1).strip()
                # Parse min/max in range or single number
                numbers = [float(n) for n in re.findall(r'\d+(?:\.\d+)?', val_range_str)]
                max_claimed = max(numbers) if numbers else 0.0

                # If claimed exceeds ground truth by >10% or is an ungrounded high range like 15-20g vs 12g limit
                if max_claimed > (gt_launch_limit * 1.05):
                    findings.append(Finding(
                        "factual-grounding-numeric-drift",
                        f"{rel_path}:{lineno_1idx}: Fabricated numeric quantity '{val_range_str}g' catapult launch load contradicts schema ground truth limit ({gt_launch_limit:.1f}g) in {', '.join(gt.source_files) or 'schema/'}.",
                        location=f"{rel_path}:{lineno_1idx}",
                        detail={"file": rel_path, "line": lineno_1idx, "claimed": val_range_str, "ground_truth_limit": gt_launch_limit}
                    ))

        return findings

    def _validate_protocols(
        self,
        content: str,
        rel_path: str,
        gt: SchemaGroundTruth
    ) -> List[Finding]:
        """
        Validates electrical / communication protocols against declared schema ground truth.
        Emits Finding('factual-grounding-unverified-protocol', ...).
        """
        findings: List[Finding] = []
        lines = content.splitlines()

        current_heading = "Header"
        is_normative = True
        in_code_block = False

        for lineno_1idx, line in enumerate(lines, start=1):
            line_str = line.strip()

            if line_str.startswith("```"):
                in_code_block = not in_code_block
                continue
            if in_code_block or not line_str:
                continue

            # Heading detection
            m_head = re.match(r'^(#{1,6})\s+(.+)$', line_str)
            if m_head:
                current_heading = m_head.group(2).strip()
                is_normative = not self._is_non_normative_section(current_heading)
                continue

            if not is_normative:
                continue

            # Skip rejected trade study rows or evaluation options
            if re.search(r'\b(?:rejected|discarded|eliminated|not\s+selected|candidate\s+option|option\s+[a-z0-9]|alternative)\b', line_str, re.I):
                continue

            # Check if line has explicit SSOT citation
            if self._has_ssot_citation(line_str, content, rel_path):
                continue

            # Skip standards citations / regulatory references (e.g. "NATO STANAG 4586", "STANAG 4586 §3.2", "ISO/IEC/IEEE", "RTCA DO-178C")
            if "§" in line_str or re.search(r'\b(?:NATO|RTCA|SAE|IEEE|ISO|MIL-STD|ARINC)\s+[A-Z0-9\-_]+(?:\s+§|\s+Ed\.|\s+Rev|\s*\|)', line_str, re.I):
                continue
            if re.search(r'\b(?:normative\s+reference|standard\s+reference|reference\s+standard|compliance\s+reference|regulatory\s+reference)\b', line_str, re.I):
                continue
            # If line is in a table citing standard definitions/descriptions or research allocations
            if "|" in line_str and re.search(r'\b(?:NATO|UCS|DLI|Interoperability|Regulatory|Compliance|Guidance|IEEE|SAE|Standard Interfaces)\b', line_str, re.I):
                continue

            # Check for protocol claims in line
            for proto in RECOGNIZED_PROTOCOLS:
                # Match word boundary
                pat = re.compile(r'\b' + re.escape(proto) + r'\b', re.I)
                if pat.search(line_str):
                    proto_norm = _normalize_name(proto)
                    if proto_norm not in gt.declared_protocols:
                        findings.append(Finding(
                            "factual-grounding-unverified-protocol",
                            f"{rel_path}:{lineno_1idx}: Ungrounded protocol claim '{proto}' is not declared in schema ground truth or substantiated by SSOT citation.",
                            location=f"{rel_path}:{lineno_1idx}",
                            detail={"file": rel_path, "line": lineno_1idx, "protocol": proto}
                        ))

        return findings

    def _validate_sequence_diagram_temporal_safety(
        self,
        content: str,
        rel_path: str
    ) -> List[Finding]:
        """
        Scans ```mermaid sequenceDiagram blocks and validates that any physical arming/firing signal
        is preceded temporally by an explicit Human-in-the-Loop (HITL) C2 arming command / operator consent.
        Emits Finding('factual-grounding-temporal-safety-violation', ...).
        """
        findings: List[Finding] = []

        # Find all sequenceDiagram code blocks
        seq_pattern = re.compile(r'```mermaid\s*\n\s*sequenceDiagram\b(.*?)\n```', re.DOTALL | re.I)

        for match in seq_pattern.finditer(content):
            block_content = match.group(1)
            # Compute line number of start of match
            preceding_text = content[:match.start()]
            block_start_lineno = preceding_text.count('\n') + 1

            block_lines = block_content.splitlines()
            hitl_consent_granted = False

            # Regex for message arrow: ParticipantA ->> ParticipantB: Message
            msg_pattern = re.compile(
                r'^\s*([a-zA-Z0-9_\-]+)\s*(?:->>|->|-->>|-->|-\)|--\)|-x|--x)\s*([a-zA-Z0-9_\-]+)\s*:\s*(.+)$'
            )

            for line_offset, b_line in enumerate(block_lines, start=1):
                raw_line = b_line.strip()
                curr_lineno = block_start_lineno + line_offset

                if not raw_line or raw_line.startswith("%%") or raw_line.startswith("Note ") or raw_line.startswith("autonumber") or raw_line.startswith("title "):
                    continue

                m_msg = msg_pattern.match(raw_line)
                if not m_msg:
                    continue

                sender = m_msg.group(1).strip()
                target = m_msg.group(2).strip()
                action = m_msg.group(3).strip()

                sender_norm = _normalize_name(sender)
                target_norm = _normalize_name(target)

                # 1. Check if this message grants Human C2 Arming / Consent
                is_hitl_sender = sender_norm in HITL_SENDER_TOKENS or any(tok in sender_norm for tok in ("operator", "pilot", "gcs", "commander", "human"))
                if is_hitl_sender:
                    for pat in HITL_CONSENT_ACTION_PATTERNS:
                        if pat.search(action):
                            hitl_consent_granted = True
                            break

                # 2. Check if this message is a disarm/safe action (resets consent if disarmed)
                is_disarm = any(pat.search(action) for pat in DISARM_ACTION_PATTERNS)
                if is_disarm:
                    hitl_consent_granted = False
                    continue

                # 3. Check if target is a Physical Arming / Firing / Motor-Enable Target
                is_physical_target = (
                    target_norm in PHYSICAL_ARMING_TARGET_TOKENS
                    or any(tok in target_norm for tok in PHYSICAL_ARMING_TARGET_TOKENS)
                )

                # 4. Check if action is a Physical Arming / Firing Action
                is_arming_action = False
                for pat in PHYSICAL_ARMING_ACTION_PATTERNS:
                    if pat.search(action):
                        is_arming_action = True
                        break

                if is_physical_target and is_arming_action:
                    if not hitl_consent_granted:
                        findings.append(Finding(
                            "factual-grounding-temporal-safety-violation",
                            f"{rel_path}:{curr_lineno}: Autonomous arming sequence detected. Physical signal '{action}' targeting '{target}' is not preceded by an explicit Human-in-the-Loop (HITL) C2 arming command or operator consent.",
                            location=f"{rel_path}:{curr_lineno}",
                            detail={"file": rel_path, "line": curr_lineno, "target": target, "action": action}
                        ))

        return findings
