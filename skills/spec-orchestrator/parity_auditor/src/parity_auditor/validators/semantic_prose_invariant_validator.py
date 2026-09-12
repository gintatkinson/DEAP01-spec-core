"""
Physical Invariant Semantic Prose Gate Validator (Check 22).

Evaluates natural language narrative prose across all specification documents in docs/
against physical negative invariants declared in the SysML AST (schema/DEAP_MODEL.sysml,
schema/*.sysml, schema.sysml, schema/extracted/*.md, schema/):
1. Ingests negative attributes from AST / SysML model (e.g. recoverySystem == "No" or "None",
   chuteEnabled == 0 or false, landingGear == "No" or "None", LifecycleType == "Expendable").
2. Builds dynamic prohibited concept patterns without static domain constants.
3. Applies contextual disambiguation: strictly permits valid negative assertions (e.g.
   "Recovery system: No", "does not deploy", "no parachute recovery", "zero recovery landing",
   "landing gear is not installed", "without vehicle recovery"), while raising fatal validation
   errors on positive operational claims (e.g. "following vehicle recovery", "the UAV has landed",
   "commanded recovery landing", "deploys parachute", "executes flare maneuver and touches down on runway",
   "lowers landing gear").
4. Skips code blocks, HTML comments, and non-normative sections (e.g. glossary, acronyms, MCDA trade
   study analyzing rejected options).
5. Emits structured Finding objects with rule ID 'semantic-prose-physical-invariant-violation'.
6. Gracefully handles clean upstream landing zones (empty schema/).
"""

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


# Values indicating a disabled, absent, negative, or single-use physical characteristic
NEGATIVE_ATTRIBUTE_VALUES = {
    "no", "none", "false", "0", "disabled", "not installed", "not_installed",
    "uninstalled", "absent", "nil", "n/a", "na", "zero", "unsupported",
    "omitted", "excluded", "precluded", "prohibited", "non-existent", "nonexistent"
}

EXPENDABLE_LIFECYCLE_VALUES = {
    "expendable", "disposable", "single-use", "single_use", "singleuse",
    "one-way", "one_way", "oneway", "non-recoverable", "non_recoverable",
    "nonrecoverable", "discardable", "attrition"
}

# Non-normative section heading patterns (e.g. Glossary, MCDA trade study, Acronyms)
NON_NORMATIVE_SECTION_PATTERNS = [
    re.compile(r'\b(?:glossary|acronyms?|definitions?|terminology)\b', re.I),
    re.compile(r'\b(?:trade\s+stud(?:y|ies)|trade-?off|mcda|multi-criteria|rejected\s+alternatives?|candidate\s+analysis|trade\s+space)\b', re.I),
    re.compile(r'\b(?:revision\s+history|document\s+history|document\s+control|change\s+log)\b', re.I),
]


@dataclass
class NegativeInvariant:
    """A physical negative invariant extracted from the schema/AST."""
    attribute_name: str
    attribute_value: str
    source_file: str
    concept_domain: str  # "recovery", "parachute", "landing", "landing_gear", "expendable_lifecycle", or "custom"
    description: str


@dataclass
class ProhibitedConceptRule:
    """A rule specifying a prohibited positive operational concept pattern."""
    concept_name: str
    pattern: re.Pattern
    invariant: NegativeInvariant
    exclusion_patterns: List[re.Pattern] = field(default_factory=list)


def _normalize_name(name: str) -> str:
    """Normalize identifier for matching."""
    if not name:
        return ""
    return re.sub(r'[^a-zA-Z0-9]', '', str(name)).lower()


def _is_negative_value(val: str) -> bool:
    """Check if value string denotes a negative / disabled / absent state."""
    if not val:
        return False
    v = str(val).strip().strip('"\'`').lower()
    if v in NEGATIVE_ATTRIBUTE_VALUES:
        return True
    if re.match(r'^(?:0(?:\.0+)?|false|no|none|n/a|na|nil|disabled|absent)$', v, re.I):
        return True
    return False


def _is_expendable_lifecycle_value(val: str) -> bool:
    """Check if value string denotes an expendable / non-recoverable lifecycle."""
    if not val:
        return False
    v = str(val).strip().strip('"\'`').lower()
    v_norm = _normalize_name(v)
    if v in EXPENDABLE_LIFECYCLE_VALUES or v_norm in {
        "expendable", "disposable", "singleuse", "oneway", "nonrecoverable", "discardable", "attrition"
    }:
        return True
    return False


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


NON_PHYSICAL_PARAM_NAMES = {
    "parity", "baud", "baudrate", "bit", "bits", "stopbit", "stopbits", "databits",
    "byte", "bytes", "crc", "ack", "nack", "seq", "sequence", "command", "cmd",
    "opcode", "header", "footer", "checksum", "pbit", "cbit", "ibit", "pbitresult",
    "cbitresult", "ibitresult", "offset", "index", "address", "addr", "register",
    "reg", "port", "pin", "pinnumber", "pinname", "voltage", "current", "power",
    "frequency", "freq", "period", "interval", "rate", "gain", "baud_rate",
    "timeout", "retry", "retries", "delay", "latency", "message", "messagelength",
    "packet", "buffer", "payloadlen", "payloadlength", "version", "revision",
    "format", "encoding", "status", "flag", "flags", "error",
    "errorcode", "returncode", "result", "code", "counter", "count", "id",
    "channel", "uart", "rs485", "can", "spi", "i2c", "ethernet", "gpio",
    "baud_rate", "stop_bits", "data_bits", "bus_interval", "reply_timeout"
}

NON_PHYSICAL_WORD_TOKENS = {
    "pin", "parity", "baud", "crc", "pbit", "cbit", "ibit", "baudrate", "stopbit",
    "stopbits", "databits"
}

DOC_METADATA_PARAM_NAMES = {
    "note", "notes", "precondition", "preconditions", "postcondition", "postconditions",
    "condition", "conditions", "trigger", "triggers", "actor", "actors", "dependency",
    "dependencies", "assumption", "assumptions", "comment", "comments", "remark",
    "remarks", "requirement", "requirements", "rationale", "traceability", "reference",
    "references", "exception", "exceptions", "summary", "author", "reviewer", "version",
    "revision", "title", "name", "id", "description", "scope", "purpose", "target",
    "objective", "tag", "tags", "label", "labels", "source", "dest", "destination",
    "type", "unit", "units", "range", "domain", "format", "criterion", "criteria",
    "step", "steps", "flow", "alternative", "constraint", "constraints", "verification",
    "validation", "compliance", "mitigation", "severity", "probability", "rpn"
}

SIGNAL_STATUS_SUFFIXES = (
    "valid", "status", "state", "ready", "flag", "ack", "fault", "error",
    "mode", "signal", "feed", "value", "level", "threshold", "rate", "gain",
    "timeout", "count", "counter", "index", "offset"
)

PHYSICAL_EQUIPMENT_KEYWORDS = {
    "system", "device", "mechanism", "equipment", "hardware", "subsystem", "payload",
    "sensor", "actuator", "armor", "wing", "rotor", "engine", "motor", "propeller",
    "thruster", "turret", "hook", "float", "skid", "launcher", "catapult", "airframe",
    "uplink", "downlink", "transponder", "radar", "lidar", "altimeter", "beacon",
    "camera", "gimbal", "warhead", "chute", "parachute", "recovery", "landing",
    "gear", "undercarriage", "flotation", "arresting", "waterproofing", "shielding"
}


def _is_valid_identifier_key(k: str) -> bool:
    """Validate that key is a genuine named identifier, not a table cell, number, or metadata."""
    if not k:
        return False
    k_clean = k.strip()
    if not re.match(r'^[a-zA-Z][a-zA-Z0-9_\s\-]*$', k_clean):
        return False
    norm = _normalize_name(k_clean)
    if len(norm) < 3 or norm.isdigit():
        return False
    if norm in NON_PHYSICAL_PARAM_NAMES or norm in DOC_METADATA_PARAM_NAMES:
        return False
    words = _split_words(k_clean)
    if any(w in NON_PHYSICAL_WORD_TOKENS for w in words):
        return False
    return True


def _is_physical_feature_attribute(norm_name: str, val_clean: str) -> bool:
    """Determine if a custom attribute represents a physical hardware/capability invariant."""
    if any(norm_name.endswith(sfx) for sfx in SIGNAL_STATUS_SUFFIXES):
        return False
    # Must have physical equipment keyword or explicit installation/presence prefix/suffix
    has_physical_kw = any(kw in norm_name for kw in PHYSICAL_EQUIPMENT_KEYWORDS)
    has_presence_affix = (
        norm_name.startswith(("has", "with", "enable", "support")) or
        norm_name.endswith(("installed", "equipped", "fitted", "present", "available", "capable", "capability", "enabled"))
    )
    if not (has_physical_kw or has_presence_affix):
        return False
    if val_clean in ("0", "0.0") and not has_presence_affix:
        return False
    return True


def _extract_negative_invariants_from_sysml(
    sysml_files: List[str], repo_root: str
) -> List[NegativeInvariant]:
    """Extract negative attributes and lifecycle invariants from SysML v2 files."""
    invariants: List[NegativeInvariant] = []

    attr_regex = re.compile(
        r'\battribute\s+(?:def\s+)?([a-zA-Z0-9_]+)(?:\s*:\s*([a-zA-Z0-9_<>:]+))?\s*=\s*([^;]+);',
        re.I
    )
    # Also capture short-form attribute assignment e.g. attribute name = value;
    short_attr_regex = re.compile(
        r'\battribute\s+([a-zA-Z0-9_]+)\s*=\s*([^;]+);',
        re.I
    )

    for sf in sysml_files:
        rel_path = os.path.relpath(sf, repo_root)
        try:
            with open(sf, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
        except Exception:
            continue

        lines = content.splitlines()
        for line in lines:
            # Strip comments
            code_part = line.split("//")[0].split("/*")[0].strip()
            if not code_part:
                continue

            matches = list(attr_regex.finditer(code_part))
            if not matches:
                matches = list(short_attr_regex.finditer(code_part))

            for match in matches:
                name = match.group(1).strip()
                val_raw = match.group(3).strip() if match.lastindex >= 3 and match.group(3) else match.group(2).strip()
                val_clean = val_raw.strip('"\'`')
                norm_name = _normalize_name(name)

                if not _is_valid_identifier_key(name):
                    continue

                # Check if this attribute is an expendable lifecycle or negative attribute
                if norm_name in ("lifecycle", "lifecycletype", "operationalmode", "systemtype", "vehicleclass", "platformtype"):
                    if _is_expendable_lifecycle_value(val_clean):
                        invariants.append(NegativeInvariant(
                            attribute_name=name,
                            attribute_value=val_clean,
                            source_file=rel_path,
                            concept_domain="expendable_lifecycle",
                            description=f"LifecycleType == '{val_clean}'"
                        ))
                elif _is_negative_value(val_clean):
                    domain = "custom"
                    if "recovery" in norm_name or "recover" in norm_name:
                        domain = "recovery"
                    elif "chute" in norm_name or "parachute" in norm_name:
                        domain = "parachute"
                    elif "landinggear" in norm_name or "undercarriage" in norm_name or ("gear" in norm_name and not any(p in norm_name for p in ["switchgear", "gearbox", "gearratio"])):
                        domain = "landing_gear"
                    elif "landing" in norm_name or "runway" in norm_name or "autoland" in norm_name or "touchdown" in norm_name:
                        domain = "landing"
                    elif not _is_physical_feature_attribute(norm_name, val_clean):
                        continue

                    invariants.append(NegativeInvariant(
                        attribute_name=name,
                        attribute_value=val_clean,
                        source_file=rel_path,
                        concept_domain=domain,
                        description=f"{name} == '{val_clean}'"
                    ))

    return invariants


def _extract_negative_invariants_from_markdown(
    md_files: List[str], repo_root: str
) -> List[NegativeInvariant]:
    """Extract negative attributes from extracted schema markdown tables and lists."""
    invariants: List[NegativeInvariant] = []

    # Table row pattern: | Key | Value | ...
    table_row_regex = re.compile(r'^\s*\|\s*([^|]+)\|\s*([^|]+)\|')
    # Key-value pattern: - Key: Value or Key: Value
    kv_regex = re.compile(r'^\s*(?:-\s*)?([a-zA-Z0-9_\s]{2,40}):\s*([^`\n]+)$')

    for mf in md_files:
        rel_path = os.path.relpath(mf, repo_root)
        try:
            with open(mf, "r", encoding="utf-8", errors="ignore") as f:
                lines = f.readlines()
        except Exception:
            continue

        for line in lines:
            line_str = line.strip()
            if not line_str or line_str.startswith("#"):
                continue

            k, v = None, None
            m_tbl = table_row_regex.match(line_str)
            if m_tbl:
                k = m_tbl.group(1).strip()
                v = m_tbl.group(2).strip()
            else:
                m_kv = kv_regex.match(line_str)
                if m_kv:
                    k = m_kv.group(1).strip()
                    v = m_kv.group(2).strip()

            if not k or not v:
                continue

            # Skip markdown table separator rows
            if re.match(r'^:?-+:?$', k) or re.match(r'^:?-+:?$', v):
                continue

            if not _is_valid_identifier_key(k):
                continue

            v_clean = v.strip('"\'`')
            norm_k = _normalize_name(k)

            if norm_k in ("lifecycle", "lifecycletype", "operationalmode", "systemtype", "vehicleclass", "platformtype"):
                if _is_expendable_lifecycle_value(v_clean):
                    invariants.append(NegativeInvariant(
                        attribute_name=k,
                        attribute_value=v_clean,
                        source_file=rel_path,
                        concept_domain="expendable_lifecycle",
                        description=f"LifecycleType == '{v_clean}'"
                    ))
            elif _is_negative_value(v_clean):
                domain = "custom"
                if "recovery" in norm_k or "recover" in norm_k:
                    domain = "recovery"
                elif "chute" in norm_k or "parachute" in norm_k:
                    domain = "parachute"
                elif "landinggear" in norm_k or "undercarriage" in norm_k or ("gear" in norm_k and not any(p in norm_k for p in ["switchgear", "gearbox", "gearratio"])):
                    domain = "landing_gear"
                elif "landing" in norm_k or "runway" in norm_k or "autoland" in norm_k or "touchdown" in norm_k:
                    domain = "landing"
                elif not _is_physical_feature_attribute(norm_k, v_clean):
                    continue

                invariants.append(NegativeInvariant(
                    attribute_name=k,
                    attribute_value=v_clean,
                    source_file=rel_path,
                    concept_domain=domain,
                    description=f"{k} == '{v_clean}'"
                ))

    return invariants


def _split_words(name: str) -> List[str]:
    """Split identifier into lowercase component words across camelCase, PascalCase, snake_case."""
    if not name:
        return []
    s = re.sub(r'([a-z0-9])([A-Z])', r'\1 \2', name)
    s = re.sub(r'([A-Z]+)([A-Z][a-z])', r'\1 \2', s)
    s = s.replace('_', ' ').replace('-', ' ')
    return [w.lower() for w in re.findall(r'[a-zA-Z0-9]+', s) if w]


def _build_prohibited_concept_rules(
    invariants: List[NegativeInvariant]
) -> List[ProhibitedConceptRule]:
    """Dynamically construct prohibited operational concept patterns from negative invariants."""
    rules: List[ProhibitedConceptRule] = []

    has_expendable = any(inv.concept_domain == "expendable_lifecycle" for inv in invariants)
    has_no_recovery = has_expendable or any(inv.concept_domain == "recovery" for inv in invariants)
    has_no_chute = has_expendable or any(inv.concept_domain == "parachute" for inv in invariants)
    has_no_landing = has_expendable or any(inv.concept_domain == "landing" for inv in invariants)
    has_no_gear = has_expendable or any(inv.concept_domain == "landing_gear" for inv in invariants)

    primary_inv = invariants[0] if invariants else NegativeInvariant("", "", "", "none", "")

    # 1. Recovery positive assertions prohibited if recoverySystem == No or Lifecycle == Expendable
    if has_no_recovery:
        rec_inv = next((i for i in invariants if i.concept_domain in ("recovery", "expendable_lifecycle")), primary_inv)
        rules.extend([
            ProhibitedConceptRule(
                concept_name="vehicle recovery",
                pattern=re.compile(
                    r'\b(?:following|after|during|upon|prior\s+to|initiates?|executes?|performs?|completes?|conducts?|commands?)\s+(?:the\s+)?(?:vehicle|aircraft|uav|uas|drone|system|payload)?\s*recovery\b',
                    re.I
                ),
                invariant=rec_inv,
            ),
            ProhibitedConceptRule(
                concept_name="commanded recovery landing",
                pattern=re.compile(
                    r'\b(?:commanded|directed|autonomous|automated|planned|manual)\s+recovery\s+landing\b',
                    re.I
                ),
                invariant=rec_inv,
            ),
            ProhibitedConceptRule(
                concept_name="recovery landing",
                pattern=re.compile(
                    r'\b(?:executes?|initiates?|performs?|conducts?|initiating|performing|commanding)\s+(?:a\s+)?recovery\s+landing\b',
                    re.I
                ),
                invariant=rec_inv,
            ),
            ProhibitedConceptRule(
                concept_name="recovered post-flight",
                pattern=re.compile(
                    r'\b(?:is|are|was|were|has\s+been|have\s+been)\s+recovered\s+(?:post-?flight|after\s+flight|safely|at\s+base)\b',
                    re.I
                ),
                invariant=rec_inv,
            ),
            ProhibitedConceptRule(
                concept_name="recovery phase / operation",
                pattern=re.compile(
                    r'\b(?:enters?|during|in)\s+(?:the\s+)?recovery\s+(?:phase|operation|sequence|mission)\b',
                    re.I
                ),
                invariant=rec_inv,
            ),
            ProhibitedConceptRule(
                concept_name="recovery team dispatch",
                pattern=re.compile(
                    r'\b(?:dispatch(?:es|ed|ing)?|notif(?:ies|ied|ying)?)\s+(?:the\s+)?recovery\s+team\b',
                    re.I
                ),
                invariant=rec_inv,
            ),
        ])

    # 2. Parachute deployment positive assertions prohibited if chuteEnabled == 0 or parachute == No
    if has_no_chute:
        chute_inv = next((i for i in invariants if i.concept_domain in ("parachute", "recovery", "expendable_lifecycle")), primary_inv)
        rules.extend([
            ProhibitedConceptRule(
                concept_name="parachute deployment",
                pattern=re.compile(
                    r'\b(?:deploys?|deploying|deployed|trigger(?:s|ed|ing)?|release(?:s|d|ing)?)\s+(?:the\s+|a\s+)?(?:ballistic\s+|emergency\s+|recovery\s+)?(?:parachute|chute|recovery\s+chute)\b',
                    re.I
                ),
                invariant=chute_inv,
            ),
            ProhibitedConceptRule(
                concept_name="parachute recovery",
                pattern=re.compile(
                    r'\b(?:via|using|through|with|performs?|initiates?|executes?)\s+(?:ballistic\s+)?parachute\s+recovery\b',
                    re.I
                ),
                invariant=chute_inv,
            ),
            ProhibitedConceptRule(
                concept_name="chute deployment sequence",
                pattern=re.compile(
                    r'\b(?:parachute|chute)\s+deployment\s+(?:sequence|phase|event|command)\b',
                    re.I
                ),
                invariant=chute_inv,
            ),
        ])

    # 3. Landing / Touchdown / Flare / Autoland positive assertions prohibited if landingGear == No or Expendable
    if has_no_landing:
        land_inv = next((i for i in invariants if i.concept_domain in ("landing", "landing_gear", "expendable_lifecycle")), primary_inv)
        rules.extend([
            ProhibitedConceptRule(
                concept_name="vehicle has landed",
                pattern=re.compile(
                    r'\b(?:the\s+)?(?:uav|uas|aircraft|vehicle|drone|system)\s+has\s+(?:landed|touched\s+down)\b',
                    re.I
                ),
                invariant=land_inv,
            ),
            ProhibitedConceptRule(
                concept_name="landed post-flight / safely",
                pattern=re.compile(
                    r'\b(?:has|have|is|are|was|were)\s+landed\s+(?:safely|post-?flight|on\s+runway|at\s+base|at\s+the\s+site)\b',
                    re.I
                ),
                invariant=land_inv,
            ),
            ProhibitedConceptRule(
                concept_name="touches down / touchdown on runway",
                pattern=re.compile(
                    r'\b(?:touches?\s+down|touchdown)(?:\s+on\s+(?:the\s+)?(?:runway|airstrip|ground|landing\s+zone|strip))?\b',
                    re.I
                ),
                invariant=land_inv,
            ),
            ProhibitedConceptRule(
                concept_name="flare maneuver",
                pattern=re.compile(
                    r'\b(?:executes?|initiates?|performs?|commands?|conducting)\s+(?:a\s+|the\s+)?flare\s+maneuver\b',
                    re.I
                ),
                invariant=land_inv,
            ),
            ProhibitedConceptRule(
                concept_name="autoland / runway landing",
                pattern=re.compile(
                    r'\b(?:engages?|initiates?|executes?|commands?|performs?|activates?)\s+(?:auto-?land|runway\s+landing|landing\s+rollout)\b',
                    re.I
                ),
                invariant=land_inv,
            ),
            ProhibitedConceptRule(
                concept_name="landing rollout",
                pattern=re.compile(
                    r'\blanding\s+rollout(?:\s+on\s+runway)?\b',
                    re.I
                ),
                invariant=land_inv,
            ),
        ])

    # 4. Landing gear extension positive assertions prohibited if landingGear == No
    if has_no_gear:
        gear_inv = next((i for i in invariants if i.concept_domain in ("landing_gear", "landing", "expendable_lifecycle")), primary_inv)
        rules.extend([
            ProhibitedConceptRule(
                concept_name="lowers landing gear",
                pattern=re.compile(
                    r'\b(?:lowers?|lowering|extends?|extending|deploy(?:s|ed|ing)?)\s+(?:the\s+)?landing\s+gear\b',
                    re.I
                ),
                invariant=gear_inv,
            ),
            ProhibitedConceptRule(
                concept_name="landing gear deployment",
                pattern=re.compile(
                    r'\blanding\s+gear\s+(?:down|deployed|deployment|extension)\b',
                    re.I
                ),
                invariant=gear_inv,
            ),
        ])

    # 5. Dynamic prohibited concepts for custom negative invariants
    for inv in invariants:
        if inv.concept_domain == "custom":
            words = _split_words(inv.attribute_name)
            if words:
                spaced_phrase = r'\s+'.join(re.escape(w) for w in words)
                joined_phrase = re.escape("".join(words))
                root_words = [re.sub(r'ing$', '', w) if len(w) > 4 else w for w in words]
                if root_words != words:
                    root_spaced = r'\s+'.join(re.escape(w) for w in root_words)
                    root_joined = re.escape("".join(root_words))
                    regex_str = rf'\b(?:{spaced_phrase}|{joined_phrase}|{root_spaced}|{root_joined})\b'
                else:
                    regex_str = rf'\b(?:{spaced_phrase}|{joined_phrase})\b'

                patt = re.compile(regex_str, re.I)
                rules.append(ProhibitedConceptRule(
                    concept_name=" ".join(words),
                    pattern=patt,
                    invariant=inv,
                ))

    return rules


def _is_valid_negative_assertion(
    sentence: str, match_span: Tuple[int, int], matched_text: str
) -> bool:
    """
    Contextual Disambiguation: Determines whether a matched phrase is part of a valid negative assertion.
    Strictly permits valid negative assertions, e.g.:
    - "Recovery system: No", "Recovery system: None", "Landing gear: None"
    - "The system does not deploy a parachute"
    - "No parachute recovery is performed"
    - "Zero recovery landing"
    - "Landing gear is not installed"
    - "Without vehicle recovery"
    - "Is not equipped with landing gear"
    - "Omits runway touchdown"
    - "Precludes parachute deployment"
    """
    s_lower = sentence.lower()
    start_pos, end_pos = match_span

    # 1. Check if the line / clause is a key-value or table column specification declaring negative status
    # e.g. "Recovery System: No", "Landing Gear: None", "| Recovery System | No |"
    # Look at suffix immediately following the match (within ~30 chars)
    suffix = s_lower[end_pos:min(len(s_lower), end_pos + 40)]
    prefix = s_lower[max(0, start_pos - 40):start_pos]

    # Key-value declaration with negative value: ": No", ": None", ": 0", ": Disabled", "| No |", etc.
    if re.match(r'^\s*[:=]\s*(?:no\b|none\b|n/a\b|false\b|0\b|disabled\b|not\s+installed\b|uninstalled\b|absent\b|nil\b)', suffix):
        return True
    if re.match(r'^\s*\|\s*(?:no\b|none\b|n/a\b|false\b|0\b|disabled\b|not\s+installed\b|uninstalled\b|absent\b|nil\b)\s*\|', suffix):
        return True

    # Suffix negation verbs: "is not installed", "is not present", "is not fitted", "is omitted", "is excluded", "is disabled"
    suffix_neg_patterns = [
        re.compile(r'^\s+(?:is|are|was|were)\s+not\s+(?:installed|present|equipped|fitted|used|supported|performed|required|applicable|available)\b'),
        re.compile(r'^\s+(?:is|are|was|were)\s+(?:omitted|excluded|precluded|prohibited|disabled|absent|unavailable|uninstalled|unsupported)\b'),
        re.compile(r'^\s+not\s+(?:installed|present|fitted|equipped|supported|applicable|performed)\b'),
        re.compile(r'^\s*,\s*(?:which\s+is\s+)?not\s+(?:installed|present|fitted|equipped|supported)\b'),
    ]
    for snp in suffix_neg_patterns:
        if snp.search(suffix):
            return True

    # 2. Check prefix negation cues preceding the matched text in the sentence / clause
    # Look at the window of words preceding the match
    words_before = re.findall(r'[a-zA-Z0-9_\-]+', prefix)
    last_words_before = words_before[-8:] if len(words_before) >= 8 else words_before
    window_before = " ".join(last_words_before).lower()

    # Preceding negation words & phrases
    preceding_neg_patterns = [
        re.compile(r'\b(?:no|zero|without|never|neither|nor|none)\s*$'),
        re.compile(r'\b(?:does|do|did|will|would|can|could|shall|may|might|must)\s+not\s+(?:have\s+|deploy\s+|execute\s+|perform\s+|conduct\s+|require\s+|use\s+|support\s+|feature\s+|include\s+)?$'),
        re.compile(r'\b(?:is|are|was|were)\s+not\s+(?:equipped\s+with|fitted\s+with|designed\s+for|intended\s+for|configured\s+for)?\s*$'),
        re.compile(r'\b(?:omits?|omitting|omitted|excludes?|excluding|excluded|precludes?|precluding|precluded|prohibits?|prohibiting|prohibited|prevents?|preventing|prevented|disallows?|disallowing|disallowed|lacks?|lacking|lacked)\s*$'),
        re.compile(r'\b(?:devoid\s+of|free\s+of|inapplicable|not\s+applicable|non-recoverable|non-recoverable\s*,?\s*omitting|uninstalled)\s*$'),
        re.compile(r'\b(?:expendable|disposable|single-use)\s+(?:system|architecture|design|vehicle|airframe)?\s*(?:with\s+no|without|omitting)\s*$'),
    ]

    for pnp in preceding_neg_patterns:
        if pnp.search(window_before) or pnp.search(prefix):
            return True

    # Check if direct preceding word in clause is "no" / "zero" / "without" / "non-"
    if last_words_before:
        if last_words_before[-1] in ("no", "zero", "without", "never", "none", "omitting", "excluding", "precluding"):
            return True
        if len(last_words_before) >= 2 and last_words_before[-2] in ("does", "do", "did", "will", "is", "are", "was", "were", "cannot") and last_words_before[-1] in ("not", "no"):
            return True

    # Check for sentence-level negation scoping the clause
    # e.g. "There is no parachute recovery", "Mission operates without vehicle recovery"
    if re.search(r'\b(?:there\s+is\s+no|there\s+are\s+no|with\s+no|with\s+zero|at\s+zero)\s+[a-zA-Z0-9_\s]{0,25}' + re.escape(matched_text.lower()), s_lower):
        return True

    # Check "does not [verb] parachute / landing"
    if re.search(r'\b(?:does|do|did|will|cannot|can\s+not)\s+not\s+[a-zA-Z0-9_\s]{0,20}' + re.escape(matched_text.lower()), s_lower):
        return True

    # Check "landing gear is not installed"
    if re.search(re.escape(matched_text.lower()) + r'\s+[a-zA-Z0-9_\s]{0,15}\bnot\s+(?:installed|present|fitted|equipped|used|supported|performed)\b', s_lower):
        return True

    return False


def _is_non_normative_heading(heading_text: str) -> bool:
    """Returns True if the markdown heading denotes a non-normative section."""
    clean_h = heading_text.strip().strip('#').strip()
    for patt in NON_NORMATIVE_SECTION_PATTERNS:
        if patt.search(clean_h):
            return True
    return False


class SemanticProseInvariantValidator(IValidator):
    """
    Check 22: Physical Invariant Semantic Prose Gate.
    Verifies that natural language narrative prose across all specification documents in docs/
    strictly respects physical negative invariants declared in the SysML AST:
    - Zero positive operational claims of prohibited concepts (recovery, landing, autoland, flare, parachute, landing gear).
    - Strictly permits valid negative assertions (e.g. "Recovery system: No", "does not deploy", "no parachute recovery").
    - Skips code blocks, HTML comments, and non-normative sections (MCDA trade study, Glossary).
    - Gracefully passes with zero errors on clean upstream landing zones.
    """

    def __init__(self, workspace_repo: Optional[WorkspaceRepository] = None):
        self.workspace_repo = workspace_repo

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        """Validate specification prose across workspace against AST negative invariants."""
        findings: List[Finding] = []
        schemas_dir = kwargs.get("schemas_dir")
        self.workspace_repo = repo

        # 1. Discover SysML and extracted markdown schema files
        sysml_files = _find_sysml_files(repo, schemas_dir)
        md_schema_files = _find_extracted_markdown_files(repo, schemas_dir)

        if not sysml_files and not md_schema_files:
            return []

        # 2. Extract negative physical invariants
        invariants: List[NegativeInvariant] = []
        invariants.extend(_extract_negative_invariants_from_sysml(sysml_files, repo.workspace_dir))
        invariants.extend(_extract_negative_invariants_from_markdown(md_schema_files, repo.workspace_dir))

        # Deduplicate invariants by concept domain and normalized attribute name
        unique_invariants: List[NegativeInvariant] = []
        seen_keys = set()
        for inv in invariants:
            key = (inv.concept_domain, _normalize_name(inv.attribute_name))
            if key not in seen_keys:
                seen_keys.add(key)
                unique_invariants.append(inv)
        invariants = unique_invariants

        # If no negative invariants are declared in the schema, return clean pass
        if not invariants:
            return []

        # 3. Build dynamic prohibited concept rules
        rules = _build_prohibited_concept_rules(invariants)
        if not rules:
            return []

        # 4. Discover markdown files in docs/ (and optional rules/ / skills/ if scan_dirs is provided)
        scan_subdirs = kwargs.get("scan_dirs") or ["docs"]
        md_files: List[str] = []
        for sdir in scan_subdirs:
            abs_dir = os.path.join(repo.workspace_dir, sdir)
            if os.path.exists(abs_dir):
                md_files.extend(repo.get_markdown_files(abs_dir))

        # 5. Audit narrative prose in each document
        for md_path in md_files:
            rel_path = os.path.relpath(md_path, repo.workspace_dir)

            try:
                with open(md_path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
            except Exception as e:
                findings.append(Finding(
                    "semantic-prose-physical-invariant-violation",
                    f"{rel_path}: Failed to read document: {e}",
                    location=rel_path
                ))
                continue

            lines = content.splitlines()
            in_code_fence = False
            in_html_comment = False
            current_section_non_normative = False

            for lineno_1idx, raw_line in enumerate(lines, start=1):
                line = raw_line.strip()

                # Handle code fence toggles
                if re.match(r'^\s*```', line):
                    in_code_fence = not in_code_fence
                    continue

                if in_code_fence:
                    continue

                # Handle HTML comments
                if "<!--" in line and "-->" in line:
                    # Single-line HTML comment -> strip it
                    line = re.sub(r'<!--.*?-->', '', line).strip()
                    if not line:
                        continue
                elif "<!--" in line:
                    in_html_comment = True
                    line = line.split("<!--")[0].strip()
                    if not line:
                        continue
                elif "-->" in line and in_html_comment:
                    in_html_comment = False
                    line = line.split("-->")[-1].strip()
                    if not line:
                        continue
                elif in_html_comment:
                    continue

                # Heading detection & non-normative section tracking (Glossary, MCDA Trade Study, etc.)
                m_heading = re.match(r'^(#{1,6})\s+(.+)$', line)
                if m_heading:
                    heading_text = m_heading.group(2)
                    current_section_non_normative = _is_non_normative_heading(heading_text)
                    continue

                if current_section_non_normative:
                    continue

                # If line is blank or structural delimiter, skip
                if not line or re.match(r'^[\s:\-*_=#|]+$', line):
                    continue

                # Split line into sentences or clauses for precision
                # Keep line as full context
                sentence_clean = raw_line.strip()
                if not sentence_clean:
                    continue

                for rule in rules:
                    for match in rule.pattern.finditer(sentence_clean):
                        matched_text = match.group(0)
                        span = match.span()

                        # Apply Contextual Disambiguation
                        if _is_valid_negative_assertion(sentence_clean, span, matched_text):
                            # Valid negative assertion (e.g. "Recovery system: No", "does not deploy parachute")
                            continue

                        # Positive operational claim violating physical invariant!
                        inv_desc = rule.invariant.description
                        findings.append(Finding(
                            "semantic-prose-physical-invariant-violation",
                            f"{rel_path}:{lineno_1idx}: Natural language prose asserts positive operational claim '{matched_text}' violating SysML physical negative invariant ({inv_desc}). Snippet: \"{sentence_clean}\"",
                            location=f"{rel_path}:{lineno_1idx}",
                            detail={
                                "file": rel_path,
                                "line": lineno_1idx,
                                "concept": matched_text,
                                "snippet": sentence_clean,
                                "invariant": inv_desc,
                            }
                        ))

        return findings
