"""
Factual Grounding & Parametric SSOT Parity Gate Validator (Check 23).

Enforces factual grounding and physical fidelity against SysML v2 AST and Level 0 OEM schema:
1. Ingests SysML v2 AST AttributeDefs, PartDefs, PortDefs, ConnectionDefs via fail-closed loader.
2. Ingests raw schema texts, BOM tables, and dictionaries from schema/ and schema/extracted/.
3. Gracefully passes on upstream clean landing zones (empty schema/).
4. Evaluates specification documents in docs/ (CONOPS, STPA, features, epics, icds, use-cases, user-stories):
   a) Structural assertions and descriptors (rule ID: 'factual-grounding-numeric-drift'):
      - Detects ungrounded structural assertions and component count drift against schema ground truth.
      - Detects structural descriptor and configuration drift against schema ground truth.
   b) Numeric quantities and limits (rule ID: 'factual-grounding-numeric-drift'):
      - Detects fabricated numeric quantities and limit violations against schema ground truth.
   c) Electrical / communication protocols (rule ID: 'factual-grounding-unverified-protocol'):
      - Detects ungrounded protocol claims (e.g. "STANAG 4586", "STANAG 4609", "MIL-STD-1553", "ARINC 429",
        "CANopen", "MAVLink", "RS-485", etc.) mentioned in specifications that are not declared in schema/ or
        substantiated with SSOT citations.
   d) Temporal safety in Mermaid sequence diagrams (rule ID: 'factual-grounding-temporal-safety-violation'):
      - Scans ```mermaid sequenceDiagram blocks across docs/.
      - Detects physical arming/firing/motor-enable signals (e.g. targeting Actuator, Pyro, FiringCircuit, SafetySwitch,
        Safety-critical actuator with action Arm/Fire/Enable).
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
from typing import Dict, List, Optional, Set, Tuple, Any, Sequence, Union

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
    "actuator", "pyro", "firingcircuit", "safetyswitch", "motor",
    "igniter", "armingdevice", "payloadrelease", "laser",
    "powerstage", "highvoltage", "emitter",
    "squib", "propulsionenable", "armswitch", "firingunit",
    "safearm", "safeandarm", "initiator", "booster", "payloadbay", "ejector"
}

# Physical arming/firing action tokens in sequence diagrams
PHYSICAL_ARMING_ACTION_PATTERNS = [
    re.compile(r'\b(?:arm|arm_all|arming|arm_circuit|arm_system|arm_pyro|arm_device|arm_motor)\b', re.I),
    re.compile(r'\b(?:fire|firing|fire_pulse|fire_squib|fire_pyro|fire_circuit|detonate|detonation)\b', re.I),
    re.compile(r'\b(?:ignite|ignition|ignite_motor|start_ignition|motor_enable|propulsion_enable|enable_motor|enable_firing|enable_high_voltage)\b', re.I),
    re.compile(r'\b(?:deploy_payload|release_payload|eject_payload|payload_release)\b', re.I),
    re.compile(r'\b(?:activate_pyro|activate_initiator|activate_power_stage)\b', re.I),
]

# Explicit prohibition/negation tokens on arming actions (e.g. disarm, safe, inhibit, abort)
DISARM_ACTION_PATTERNS = [
    re.compile(r'\b(?:disarm|disarming|safe|safing|inhibit|inhibiting|abort|aborting|disable|deactivate)\b', re.I),
]

# Human C2 / HITL participant tokens in sequence diagrams
HITL_SENDER_TOKENS = {
    "operator", "operators", "pilot", "pilots", "human", "commander",
    "supervisor", "controller", "technician", "safetyofficer", "user",
    "c2", "gcsoperator", "gcs", "groundcontrolstation", "groundstation"
}

# HITL C2 arming / consent / authorization action patterns
HITL_CONSENT_ACTION_PATTERNS = [
    re.compile(r'\b(?:arm|arm_command|command_arm|send_arm_cmd|arm_switch_on|arm_request|authorize_arm|request_arm|arm_confirm|confirm_arm)\b', re.I),
    re.compile(r'\b(?:consent|operator_consent|pilot_consent|grant_consent|consent_granted|consent_token|consent_key)\b', re.I),
    re.compile(r'\b(?:authorize|authorization|authorize_fire|authorize_launch|permit_fire|permit_arm|fire_permission|command_fire|fire_command|send_fire_cmd)\b', re.I),
    re.compile(r'\b(?:confirm_engagement|engage_command|manual_arm|manual_consent|c2_arm|c2_fire)\b', re.I),
]

# Generic AST tokens that represent structural/modeling boilerplate, units, or non-hardware prose
NON_HARDWARE_GENERIC_TOKENS: Set[str] = {
    "time", "step", "state", "range", "message", "sign", "mode", "level",
    "feed", "status", "report", "frame", "int", "real", "scalar", "code",
    "gate", "link", "bus", "item", "port", "doc", "package", "model",
    "attribute", "connection", "part", "def",
    "stage", "off", "phase", "order", "rate", "value", "val", "type",
    "count", "qty", "quantity", "number", "num", "size", "index", "flag",
    "id", "name", "data", "info", "param", "parameter", "config", "configuration",
    "integer", "boolean", "string", "float", "double", "true", "false",
}


# Architectural tracer tags, requirement codes, and signal/interface prefixes
TRACER_AND_SIGNAL_PREFIXES: Tuple[str, ...] = (
    "SIG-", "REQ-", "CONN-", "FEAT-", "US-", "UC-", "EPIC-", "OP-",
    "SC-", "RULE-", "TEST-", "OSO-", "UCA-", "HAZ-", "SAF-", "OBL-",
    "INT-", "EXT-", "SYS-", "SW-", "HW-", "ICD-", "SPEC-", "DOC-",
)


def _is_tracer_or_signal_identifier(token: str) -> bool:
    """
    Checks if a token represents an architectural tracer tag, requirement code,
    signal identifier, or interface tag (e.g. SIG-ESAD, REQ-01, CONN-BATTERY, FEAT-01).
    Prevents such identifiers from being falsely classified as compound structural descriptors
    (such as 'v-tail', 'abc-tail', 'X-tail').
    """
    if not token:
        return False
    tok_upper = token.upper()
    if any(tok_upper.startswith(p) for p in TRACER_AND_SIGNAL_PREFIXES):
        return True
    m = re.match(r'^([A-Z0-9_]+)-', token)
    if m:
        prefix = m.group(1)
        # Exclude single-letter uppercase prefixes (e.g. 'V-tail', 'X-tail', 'T-tail')
        # which represent valid geometric structural shape descriptors rather than tracer tags.
        if len(prefix) >= 2 or prefix.isdigit():
            return True
    return False


def _get_enclosing_hyphenated_token(line: str, start: int, end: int) -> str:
    """Extracts the full hyphenated identifier surrounding a regex match span."""
    left = start
    while left > 0 and (line[left - 1].isalnum() or line[left - 1] in "_-"):
        left -= 1
    right = end
    while right < len(line) and (line[right].isalnum() or line[right] in "_-"):
        right += 1
    return line[left:right]


def _normalize_name(name: str) -> str:
    """Normalize identifier by removing non-alphanumeric characters and lowercasing."""
    if not name:
        return ""
    return re.sub(r'[^a-zA-Z0-9]', '', str(name)).lower()


def _tokenize_identifier(ident: str) -> List[str]:
    """Splits an identifier by camelCase, snake_case, kebab-case, or spaces into lowercase words."""
    if not ident:
        return []
    s = re.sub(r'[-_./:]', ' ', str(ident))
    s = re.sub(r'([a-z0-9])([A-Z])', r'\1 \2', s)
    return [t.lower() for t in s.split() if t.strip()]


# Suffixes and tokens for physical count targets and exclusions
COUNT_TARGET_INCLUSIONS: Tuple[str, ...] = (
    "count", "qty", "quantity", "surfaces", "channels", "actuators"
)
COUNT_TARGET_EXCLUSION_SUFFIXES: Tuple[str, ...] = (
    "register", "registers", "code", "codes", "index", "indices",
    "offset", "offsets", "id", "ids", "status", "statuses",
    "state", "states", "mask", "masks", "threshold", "thresholds",
    "baud", "chars", "bits"
)

# Suffixes and tokens for configuration targets and exclusions
CONFIG_TARGET_INCLUSIONS: Tuple[str, ...] = (
    "configuration", "config", "layout", "arrangement", "topology", "architecture", "type"
)
CONFIG_TARGET_EXCLUSION_SUFFIXES: Tuple[str, ...] = (
    "name", "names", "title", "titles", "description", "descriptions",
    "doc", "docs", "note", "notes", "ref", "refs", "poly", "polys",
    "init", "vector", "vectors", "standard", "standards",
    "baseline", "baselines", "variants", "variant", "camera", "cameras"
)

# Standard and protocol numbers that must never be treated as physical component counts
STANDARD_PROTOCOL_NUMBERS: Set[int] = {
    485, 422, 232, 429, 661, 653, 818, 825, 664,
    1553, 1760, 4586, 4609, 7085, 4187, 461, 6016, 188, 220
}

STANDARD_NAME_PREFIX_PATTERN = re.compile(
    r'\b(?:RS|EIA|TIA|MIL-STD|MIL-HDBK|MIL-SPEC|MIL|STANAG|ARINC|DO|IEEE|ISO|DEF-STAN)[-_ ]*$',
    re.I
)

STANDARD_NAME_SUFFIX_PATTERN = re.compile(
    r'^\s*(?:bus|commands?|protocol|protocols|standard|standards|spec|specs|specification|specifications|transceiver|transceivers|interface|interfaces|link|links|port|ports|serial)\b',
    re.I
)

STANDARD_TOKEN_PATTERN = re.compile(
    r'\b(?:RS|EIA|TIA|MIL(?:-STD|-HDBK|-SPEC)?|STANAG|ARINC|DO|IEEE|ISO)[-_ ]*\d+',
    re.I
)


def _is_count_target(name: str) -> bool:
    """
    Checks if an attribute name represents a genuine physical count target.
    An integer attribute is ONLY a physical count target if its normalized name contains or
    ends with count, qty, quantity, surfaces, channels, or actuators.
    EXCLUDES attributes that represent registers, bitmasks, status codes, state indices, or thresholds
    (e.g. ending in register, code, index, offset, id, status, state, mask, threshold, baud, chars, bits).
    """
    if not name:
        return False
    name_norm = _normalize_name(name)
    if not name_norm:
        return False
    tokens = _tokenize_identifier(name)
    if tokens:
        last_tok = tokens[-1].lower()
        if last_tok in COUNT_TARGET_EXCLUSION_SUFFIXES:
            return False
    if any(name_norm.endswith(ex) for ex in COUNT_TARGET_EXCLUSION_SUFFIXES):
        return False
    return any(inc in name_norm for inc in COUNT_TARGET_INCLUSIONS)


def _is_config_target(name: str) -> bool:
    """
    Checks if an attribute name represents a genuine configuration target.
    A string attribute is ONLY a configuration target if its normalized name contains or ends with
    configuration, config, layout, arrangement, topology, architecture, or type.
    EXCLUDES metadata strings (e.g. ending in name, title, description, doc, note, ref, poly, init,
    vector, standard, baseline, variants, camera).
    """
    if not name:
        return False
    name_norm = _normalize_name(name)
    if not name_norm:
        return False
    tokens = _tokenize_identifier(name)
    if tokens:
        last_tok = tokens[-1].lower()
        if last_tok in CONFIG_TARGET_EXCLUSION_SUFFIXES:
            return False
    if any(name_norm.endswith(ex) for ex in CONFIG_TARGET_EXCLUSION_SUFFIXES):
        return False
    return any(inc in name_norm for inc in CONFIG_TARGET_INCLUSIONS)


def _is_protocol_or_standard_number(line: str, start: int, end: int, num_val: int) -> bool:
    """
    Determines if a matched number represents a communication protocol, military/civil standard,
    or interface number rather than a physical component count.
    Excludes numbers preceded or followed by standard names (e.g. RS-485, RS485, EIA-485,
    485 bus, 485 Commands, MIL-STD-461, STANAG 4187, 4187).
    """
    if num_val in STANDARD_PROTOCOL_NUMBERS:
        return True

    preceding = line[:start]
    if STANDARD_NAME_PREFIX_PATTERN.search(preceding):
        return True

    following = line[end:]
    if STANDARD_NAME_SUFFIX_PATTERN.search(following):
        return True

    enclosing = _get_enclosing_hyphenated_token(line, start, end)
    if STANDARD_TOKEN_PATTERN.search(enclosing):
        return True

    return False


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


# ISO/IEC 80000 and SysML v2 ISQ Recognized Physical Measurement Units
# In conformance with ISO/IEC 80000 and SysML v2 ISQ / MeasurementUnits metamodel:
# (Length, Mass, Time, Pressure, Voltage, Power, Frequency, Velocity, Acceleration, Plane Angle, Current, Energy, Force, etc.)
ISO_80000_PHYSICAL_UNITS: Dict[str, str] = {
    # Length (ISO 80000-3)
    "m": "m", "meter": "m", "meters": "m", "metre": "m", "metres": "m",
    "km": "km", "kilometer": "km", "kilometers": "km",
    "cm": "cm", "centimeter": "cm", "centimeters": "cm",
    "mm": "mm", "millimeter": "mm", "millimeters": "mm",
    "um": "um", "micrometer": "um", "micrometers": "um", "micron": "um", "microns": "um",
    "nm": "nm", "nanometer": "nm", "nanometers": "nm",

    # Mass (ISO 80000-4)
    "kg": "kg", "kilogram": "kg", "kilograms": "kg",
    "g": "g", "gram": "g", "grams": "g",
    "mg": "mg", "milligram": "mg", "milligrams": "mg",
    "ug": "ug", "microgram": "ug", "micrograms": "ug",
    "t": "t", "tonne": "t", "tonnes": "t",

    # Time (ISO 80000-3)
    "s": "s", "sec": "s", "secs": "s", "second": "s", "seconds": "s",
    "ms": "ms", "millisecond": "ms", "milliseconds": "ms",
    "us": "us", "microsecond": "us", "microseconds": "us",
    "ns": "ns", "nanosecond": "ns", "nanoseconds": "ns",
    "min": "min", "minute": "min", "minutes": "min",
    "hr": "hr", "hrs": "hr", "hour": "hr", "hours": "hr",
    "d": "d", "day": "d", "days": "d",

    # Pressure & Stress (ISO 80000-4)
    "pa": "pa", "pascal": "pa", "pascals": "pa",
    "kpa": "kpa", "kilopascal": "kpa", "kilopascals": "kpa",
    "mpa": "mpa", "megapascal": "mpa", "megapascals": "mpa",
    "hpa": "hpa", "hectopascal": "hpa",
    "bar": "bar", "mbar": "mbar", "millibar": "mbar",
    "psi": "psi",
    "n/m^2": "pa", "n/m2": "pa",

    # Voltage & Electric Potential (ISO 80000-6)
    "v": "v", "volt": "v", "volts": "v",
    "mv": "mv", "millivolt": "mv", "millivolts": "mv",
    "kv": "kv", "kilovolt": "kv", "kilovolts": "kv",
    "uv": "uv", "microvolt": "uv",

    # Electric Current (ISO 80000-6)
    "a": "a", "amp": "a", "amps": "a", "ampere": "a", "amperes": "a",
    "ma": "ma", "milliamp": "ma", "milliamps": "ma",
    "ka": "ka", "kiloamp": "ka",

    # Power (ISO 80000-6)
    "w": "w", "watt": "w", "watts": "w",
    "kw": "kw", "kilowatt": "kw", "kilowatts": "kw",
    "mw": "mw", "milliwatt": "mw", "milliwatts": "mw",
    "gw": "gw", "gigawatt": "gw",

    # Frequency (ISO 80000-3)
    "hz": "hz", "hertz": "hz",
    "khz": "khz", "kilohertz": "khz",
    "mhz": "mhz", "megahertz": "mhz",
    "ghz": "ghz", "gigahertz": "ghz",
    "1/s": "hz", "s^-1": "hz",

    # Velocity (ISO 80000-3)
    "m/s": "m/s", "mps": "m/s", "m*s^-1": "m/s", "m s^-1": "m/s",
    "km/h": "km/h", "kph": "km/h",
    "knot": "knot", "knots": "knot",

    # Acceleration (ISO 80000-3)
    "m/s^2": "m/s^2", "m/s2": "m/s^2", "mps2": "m/s^2", "m*s^-2": "m/s^2",
    "g": "g", "g-load": "g", "gload": "g",

    # Plane Angle (ISO 80000-3)
    "deg": "deg", "degree": "deg", "degrees": "deg", "deg_ang": "deg",
    "rad": "rad", "radian": "rad", "radians": "rad",
    "mrad": "mrad", "milliradian": "mrad",

    # Energy & Work (ISO 80000-5)
    "j": "j", "joule": "j", "joules": "j",
    "kj": "kj", "kilojoule": "kj",
    "wh": "wh", "watt-hour": "wh", "kwh": "kwh", "kilowatt-hour": "kwh",

    # Force (ISO 80000-4)
    "n": "n", "newton": "n", "newtons": "n",
    "kn": "kn", "kilonewton": "kn",

    # Electric Resistance & Impedance (ISO 80000-6)
    "ohm": "ohm", "ohms": "ohm", "kohm": "kohm", "mohm": "mohm",

    # Capacitance (ISO 80000-6)
    "f": "f", "farad": "f", "uf": "uf", "nf": "nf", "pf": "pf",

    # Dimensionless Ratio / Percentage
    "%": "%", "pct": "%", "percent": "%",
}

# Atomic compound identifiers and standards citations
ATOMIC_IDENTIFIER_PATTERN = re.compile(
    r'\b[A-Za-z0-9]+(?:[-/_][A-Za-z0-9]+)+\b'
)
STANDARDS_CITATION_PATTERN = re.compile(
    r'\b(?:DO|ARP|MIL|STD|ASTM|ISO|IEC|IEEE|STANAG|ARINC|RTCA|SAE|DEF-STAN)[-_ ]*[0-9]+[A-Za-z0-9]*\b',
    re.I
)


def _is_numeric_range_or_quantity(tok: str) -> bool:
    """
    Checks if a compound token like '15-20' or '15-20g' is actually a numeric range/quantity
    rather than an indivisible reference symbol / standard citation.
    """
    m = re.match(r'^(\d+(?:\.\d+)?)\s*-\s*(\d+(?:\.\d+)?)([a-zA-Z/%^]*)$', tok)
    if m:
        suffix = m.group(3).strip().lower()
        if not suffix or suffix in ISO_80000_PHYSICAL_UNITS:
            return True
    return False


def _get_protected_spans(line: str) -> List[Tuple[int, int]]:
    """
    Identifies spans of atomic compound identifiers and standards citations
    that must be protected from being fragmented into numeric scalars or unit abbreviations.
    """
    spans: List[Tuple[int, int]] = []

    for m in STANDARDS_CITATION_PATTERN.finditer(line):
        spans.append((m.start(), m.end()))

    for m in ATOMIC_IDENTIFIER_PATTERN.finditer(line):
        tok = m.group(0)
        if _is_numeric_range_or_quantity(tok):
            continue
        spans.append((m.start(), m.end()))

    if not spans:
        return []
    spans.sort(key=lambda s: s[0])
    merged: List[Tuple[int, int]] = [spans[0]]
    for cur_s, cur_e in spans[1:]:
        prev_s, prev_e = merged[-1]
        if cur_s <= prev_e:
            merged[-1] = (prev_s, max(prev_e, cur_e))
        else:
            merged.append((cur_s, cur_e))
    return merged


def _mask_spans(line: str, spans: List[Tuple[int, int]]) -> str:
    """Replaces characters within specified spans with spaces while preserving string length."""
    if not spans:
        return line
    chars = list(line)
    for s, e in spans:
        for i in range(max(0, s), min(e, len(chars))):
            chars[i] = ' '
    return "".join(chars)


def _extract_unit(val_str: str, name_tokens: Optional[List[str]] = None) -> str:
    """Extracts ISO/IEC 80000 recognized physical measurement unit from value string or name tokens."""
    if val_str:
        m = re.search(r'[-+]?\d+(?:\.\d+)?\s*\[?([a-zA-Z/%^]+)\]?', str(val_str))
        if m:
            cand = m.group(1).lower()
            canon = ISO_80000_PHYSICAL_UNITS.get(cand)
            if canon:
                return canon

    if name_tokens:
        last_tok = name_tokens[-1].lower()
        if last_tok in ISO_80000_PHYSICAL_UNITS:
            return ISO_80000_PHYSICAL_UNITS[last_tok]
        if last_tok == "mps":
            return "m/s"
        if last_tok == "pct":
            return "%"
        if "g" in name_tokens and any(t in name_tokens for t in ("load", "accel", "acceleration", "limit")):
            return "g"
    return ""

def _is_lower_bound_name(name: str, tokens: Optional[List[str]] = None) -> bool:
    """
    Checks if attribute name represents a lower bound:
    Attributes whose names contain min, low, or floor (and not max) are lower bounds.
    """
    if not name:
        return False
    name_l = name.lower()
    if "max" in name_l:
        return False

    toks = tokens if tokens is not None else _tokenize_identifier(name)
    toks_l = [t.lower() for t in toks]
    for t in toks_l:
        if (
            t in ("min", "minimum", "low", "lower", "floor")
            or t.startswith("min")
            or (t.startswith("low") and t not in ("load", "loads"))
        ):
            return True

    for kw in ("min", "low", "floor"):
        if kw in name_l:
            if kw == "min" and any(fp in name_l for fp in ("nominal", "aluminum", "terminal")):
                continue
            if kw == "low" and any(fp in name_l for fp in ("flow", "blower", "slow")):
                continue
            return True

    return False


def _is_upper_bound_name(name: str, tokens: Optional[List[str]] = None) -> bool:
    """
    Checks if attribute name represents an upper bound:
    Attributes whose names contain max, high, limit, ceiling, bound are upper bounds.
    """
    if not name:
        return False
    name_l = name.lower()
    toks = tokens if tokens is not None else _tokenize_identifier(name)
    toks_l = [t.lower() for t in toks]
    for t in toks_l:
        if (
            t in ("max", "maximum", "high", "higher", "limit", "limits", "ceiling", "bound", "bounds")
            or t.startswith("max")
            or t.startswith("limit")
        ):
            return True
    for kw in ("max", "high", "limit", "ceiling", "bound"):
        if kw in name_l:
            return True
    return False


def _property_token_matches(prop_tok: str, text_tok: str) -> bool:
    """Checks if a property token matches a token found in specification text."""
    if not prop_tok or not text_tok:
        return False
    prop_l = prop_tok.lower()
    text_l = text_tok.lower()
    if prop_l == text_l:
        return True
    # Stemming / engineering synonyms for bounds and dimensions:
    lower_bound_words = ("min", "minimum", "minimal", "floor", "low", "lower")
    upper_bound_words = ("max", "maximum", "maximal", "ceiling", "high", "higher", "peak")
    if (prop_l in lower_bound_words or prop_l.startswith("min") or (prop_l.startswith("low") and prop_l not in ("load", "loads"))) and \
       (text_l in lower_bound_words or text_l.startswith("min") or (text_l.startswith("low") and text_l not in ("load", "loads"))):
        return True
    if (prop_l in upper_bound_words or prop_l.startswith("max")) and \
       (text_l in upper_bound_words or text_l.startswith("max")):
        return True
    if prop_l in ("temp", "temperature") and text_l in ("temp", "temperature"):
        return True
    if prop_l in ("freq", "frequency") and text_l in ("freq", "frequency"):
        return True
    if prop_l in ("width", "wide") and text_l in ("width", "wide"):
        return True
    if prop_l in ("length", "long") and text_l in ("length", "long"):
        return True
    if prop_l in ("height", "tall") and text_l in ("height", "tall"):
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


@dataclass
class ScopedNumericLimit:
    """Scoped numeric limit extracted from schema AST with component ownership."""
    key: str
    limit_val: float
    unit: str
    bound_type: str  # "lower" | "upper"
    owner: Optional[str] = None  # e.g. "esad", "battery", "airframe"
    meaningful_tokens: List[str] = field(default_factory=list)


@dataclass
class SchemaGroundTruth:
    """Consolidated Ground Truth extracted from SysML AST and schema markdown."""
    structural_attributes: Dict[str, Union[int, str]] = field(default_factory=dict)
    numeric_limits: Dict[str, Tuple[float, str]] = field(default_factory=dict)  # map normalized key -> (limit_val, unit)
    numeric_bound_types: Dict[str, str] = field(default_factory=dict)  # map key -> "lower" | "upper"
    scoped_numeric_limits: List[ScopedNumericLimit] = field(default_factory=list)
    attributes: Dict[str, Any] = field(default_factory=dict)
    declared_protocols: Set[str] = field(default_factory=set)
    raw_schema_text: str = ""
    source_files: List[str] = field(default_factory=list)
    has_concrete_schema: bool = False
    declared_ast_nodes: Set[str] = field(default_factory=set)
    declared_parts: Set[str] = field(default_factory=set)


GroundTruth = SchemaGroundTruth



class FactualGroundingValidator(IValidator):
    """
    Factual Grounding & Parametric SSOT Parity Gate Validator (Check 23).
    """

    def __init__(self, workspace_repo: Optional[WorkspaceRepository] = None, **kwargs):
        self.workspace_repo = workspace_repo

    def _register_numeric_limit(
        self,
        gt: SchemaGroundTruth,
        name: str,
        limit_val: float,
        unit: str,
        bound_type: str,
        owner: Optional[str] = None
    ) -> None:
        """Registers a numeric limit into SchemaGroundTruth with component-scoped metadata."""
        name_norm = _normalize_name(name)
        gt.numeric_limits[name_norm] = (limit_val, unit)
        gt.numeric_bound_types[name_norm] = bound_type

        tokens = _tokenize_identifier(name)
        meaningful = [t for t in tokens if t not in ("limit", "value", "val", "real", "float")]

        canon_unit = ISO_80000_PHYSICAL_UNITS.get(unit.lower(), unit.lower()) if unit else ""
        unit_tokens = set()
        if unit:
            unit_tokens.add(unit.lower())
            unit_tokens.update(_tokenize_identifier(unit.lower()))

        prop_tokens = []
        for t in tokens:
            if t in ("limit", "value", "val", "real", "float", "scalar"):
                continue
            if t in unit_tokens:
                continue
            if canon_unit and ISO_80000_PHYSICAL_UNITS.get(t) == canon_unit:
                continue
            prop_tokens.append(t)
        if not prop_tokens:
            prop_tokens = [t for t in tokens if t not in ("value", "val", "real", "float")]

        gt.scoped_numeric_limits.append(ScopedNumericLimit(
            key=name_norm,
            limit_val=limit_val,
            unit=unit,
            bound_type=bound_type,
            owner=owner,
            meaningful_tokens=prop_tokens
        ))

        if len(tokens) > 1:
            if meaningful:
                alias1 = "".join(meaningful)
                alias2 = "_".join(meaningful)
                gt.numeric_limits[alias1] = (limit_val, unit)
                gt.numeric_limits[alias2] = (limit_val, unit)
                gt.numeric_bound_types[alias1] = bound_type
                gt.numeric_bound_types[alias2] = bound_type
                if owner:
                    owner_alias = f"{owner}_{alias1}"
                    gt.numeric_limits[owner_alias] = (limit_val, unit)
                    gt.numeric_bound_types[owner_alias] = bound_type
                gt.scoped_numeric_limits.append(ScopedNumericLimit(
                    key=alias1,
                    limit_val=limit_val,
                    unit=unit,
                    bound_type=bound_type,
                    owner=owner,
                    meaningful_tokens=prop_tokens
                ))

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

            # a) Evaluate structural assertions & descriptors
            findings.extend(self._validate_structural_assertions(content, rel_path, gt))

            # b) Evaluate numeric quantities & limits
            findings.extend(self._validate_numeric_assertions(content, rel_path, gt))

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
                self._extract_from_sysml(text, gt)

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

                self._extract_from_markdown(text, os.path.relpath(mf, workspace_dir), gt)
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
            gt.structural_attributes
            or gt.numeric_limits
            or gt.attributes
            or gt.declared_protocols
        )
        if not has_concrete:
            gt.has_concrete_schema = False

        return gt

    def _extract_sysml_attributes_from_block(
        self,
        text: str,
        gt: SchemaGroundTruth,
        owner: Optional[str] = None
    ) -> None:
        attr_pattern = re.compile(
            r'\battribute\s+(?:def\s+)?([a-zA-Z0-9_]+)(?:\s*:\s*([a-zA-Z0-9_<>:]+))?\s*=\s*([^;]+);'
        )
        for match in attr_pattern.finditer(text):
            name = match.group(1).strip()
            type_str = match.group(2).strip() if match.group(2) else ""
            raw_val = match.group(3).strip()
            val_clean = raw_val.strip('"\'`')

            name_norm = _normalize_name(name)
            gt.attributes[name_norm] = val_clean
            gt.declared_ast_nodes.add(name_norm)
            gt.declared_ast_nodes.add(_normalize_name(val_clean))

            tokens = _tokenize_identifier(name)
            for t in tokens:
                gt.declared_ast_nodes.add(t)
            unit = _extract_unit(raw_val, tokens)
            scalar = _extract_numeric_scalar(raw_val)

            # Categorize based on type (Integer/Real) and unit:
            is_integer_type = type_str.lower() in ("integer", "int", "count", "natural", "cardinal")
            is_integer_val = scalar is not None and (
                is_integer_type or (not "." in val_clean and val_clean.isdigit() and not unit)
            )

            if is_integer_val:
                int_val = int(scalar)
                if _is_count_target(name) and int_val not in STANDARD_PROTOCOL_NUMBERS:
                    gt.structural_attributes[name_norm] = int_val
                    # Register base entity root (stripping count suffixes)
                    root_tokens = [t for t in tokens if t not in ("count", "qty", "quantity", "number", "num", "actuators", "surfaces", "channels")]
                    if root_tokens:
                        root_key = "".join(root_tokens)
                        gt.structural_attributes[root_key] = int_val
            elif (scalar is None or type_str.lower() in ("string", "str")) and _is_config_target(name):
                gt.structural_attributes[name_norm] = val_clean
                root_tokens = [t for t in tokens if t not in ("configuration", "config", "type", "mode", "layout", "geometry", "architecture", "topology", "arrangement")]
                if root_tokens:
                    root_key = "".join(root_tokens)
                    gt.structural_attributes[root_key] = val_clean

            # Check numeric limits / quantities:
            is_real_type = type_str.lower() in ("real", "float", "double", "scalar")
            has_limit_tokens = any(t in tokens for t in ("limit", "max", "maximum", "min", "minimum", "low", "lower", "floor", "ceiling", "high", "load", "accel", "bound", "threshold", "capacity"))
            if scalar is not None and (is_real_type or unit or has_limit_tokens):
                limit_val = float(scalar)
                bound_type = "lower" if _is_lower_bound_name(name, tokens) else "upper"
                self._register_numeric_limit(gt, name, limit_val, unit, bound_type, owner=owner)

    def _extract_from_sysml(self, text: str, gt: SchemaGroundTruth) -> None:
        """
        Generic AST extraction for SysML attribute definitions with component scoping:
        Ingests ANY typed attribute into gt.structural_attributes and/or gt.numeric_limits
        with owning component tracking.
        """
        # Ingest part definitions directly from SysML text with their bodies
        part_pattern = re.compile(r'\bpart\s+(?:def\s+)?([a-zA-Z0-9_]+)\s*\{([^}]*)\}', re.DOTALL)
        part_spans: List[Tuple[int, int]] = []
        for match in part_pattern.finditer(text):
            part_spans.append((match.start(), match.end()))
            pname = match.group(1).strip()
            pname_norm = _normalize_name(pname)
            pbody = match.group(2)
            if pname_norm:
                gt.declared_parts.add(pname_norm)
                gt.declared_ast_nodes.add(pname_norm)
                for tok in _tokenize_identifier(pname):
                    gt.declared_ast_nodes.add(tok)
                    if tok not in NON_HARDWARE_GENERIC_TOKENS and len(tok) >= 3:
                        gt.declared_parts.add(tok)
            self._extract_sysml_attributes_from_block(pbody, gt, owner=pname_norm)

        # Ingest package-level attributes (outside part defs)
        top_level_text = _mask_spans(text, part_spans)
        self._extract_sysml_attributes_from_block(top_level_text, gt, owner=None)

        # Ingest any remaining part declarations without block braces
        bare_part_pattern = re.compile(r'\bpart\s+(?:def\s+)?([a-zA-Z0-9_]+)\b')
        for match in bare_part_pattern.finditer(text):
            pname = match.group(1).strip()
            pname_norm = _normalize_name(pname)
            if pname_norm:
                gt.declared_parts.add(pname_norm)
                gt.declared_ast_nodes.add(pname_norm)
                for tok in _tokenize_identifier(pname):
                    gt.declared_ast_nodes.add(tok)
                    if tok not in NON_HARDWARE_GENERIC_TOKENS and len(tok) >= 3:
                        gt.declared_parts.add(tok)

    def _ingest_sysml_package(self, pkg: Any, gt: SchemaGroundTruth) -> None:
        """Recursively ingests elements from a parsed SysMLPackage using generic AST extraction."""
        if not pkg:
            return

        # Ingest attribute_defs
        for attr in getattr(pkg, "attribute_defs", []) or getattr(pkg, "attributes", []) or []:
            name = getattr(attr, "name", "")
            type_str = getattr(attr, "type_name", None) or getattr(attr, "type", "") or ""
            val_str = getattr(attr, "default_value", None) or getattr(attr, "doc", "") or ""
            if name and val_str:
                stmt = f"attribute {name} : {type_str} = {val_str};"
                self._extract_sysml_attributes_from_block(stmt, gt, owner=None)

        # Ingest part_defs
        for part in getattr(pkg, "part_defs", []) or getattr(pkg, "parts", []) or []:
            pname = getattr(part, "name", "")
            pname_norm = _normalize_name(pname)
            if pname:
                gt.declared_parts.add(pname_norm)
                gt.declared_ast_nodes.add(pname_norm)
                for tok in _tokenize_identifier(pname):
                    gt.declared_ast_nodes.add(tok)
                    if tok not in NON_HARDWARE_GENERIC_TOKENS and len(tok) >= 3:
                        gt.declared_parts.add(tok)
            for port in getattr(part, "ports", []) or []:
                port_name = getattr(port, "name", "")
                if port_name:
                    gt.declared_ast_nodes.add(_normalize_name(port_name))
            for attr in getattr(part, "attributes", []) or getattr(part, "attribute_defs", []) or []:
                aname = getattr(attr, "name", "")
                aname_norm = _normalize_name(aname)
                aval = getattr(attr, "default_value", None) or ""
                ascalar = _extract_numeric_scalar(aval)
                if _is_count_target(aname) and ascalar is not None and int(ascalar) not in STANDARD_PROTOCOL_NUMBERS:
                    gt.structural_attributes[pname_norm] = int(ascalar)
                elif aname and aval:
                    stmt = f"attribute {aname} = {aval};"
                    self._extract_sysml_attributes_from_block(stmt, gt, owner=pname_norm)

        # Ingest sub_packages
        for nested in getattr(pkg, "sub_packages", []) or getattr(pkg, "packages", []) or []:
            self._ingest_sysml_package(nested, gt)

    def _extract_from_markdown(self, text: str, rel_path: str, gt: SchemaGroundTruth) -> None:
        """
        Generic extraction from markdown:
        Ingests ANY table row `| Property | Value |` or bullet `Property: Value`
        into gt.structural_attributes and/or gt.numeric_limits without hardcoded keywords.
        """
        lines = text.splitlines()
        current_heading = ""
        current_owner: Optional[str] = None
        for line in lines:
            line_str = line.strip()
            if not line_str:
                continue

            m_h = re.match(r'^(#{1,6})\s+(.+)$', line_str)
            if m_h:
                current_heading = m_h.group(2).strip()
                h_norm = _normalize_name(current_heading)
                current_owner = None
                for part in gt.declared_parts:
                    if part in h_norm:
                        current_owner = part
                        break
                continue

            # Check for Markdown table rows: | Property | Value | [Desc] |
            if line_str.startswith("|") and line_str.endswith("|"):
                cells = [c.strip() for c in line_str.split("|")[1:-1]]
                if len(cells) >= 2:
                    k, v = cells[0], cells[1]
                    # Skip table header and separator rows
                    if k.startswith(":") or k.startswith("-") or k.lower() in (
                        "component", "property", "parameter", "item", "attribute", "name", "field"
                    ):
                        continue

                    row_owner = current_owner
                    k_norm = _normalize_name(k)
                    for part in gt.declared_parts:
                        if part in k_norm:
                            row_owner = part
                            break

                    # If cell 1 is a type (e.g. Integer, Real, String), value is cell 2
                    type_hint = ""
                    if len(cells) >= 3 and cells[1].lower() in ("integer", "int", "real", "float", "string", "boolean"):
                        type_hint = cells[1]
                        v = cells[2]

                    clean_v = v.strip('"\'`')
                    gt.attributes[k_norm] = clean_v
                    gt.declared_ast_nodes.add(k_norm)
                    gt.declared_ast_nodes.add(_normalize_name(clean_v))

                    tokens = _tokenize_identifier(k)
                    for t in tokens:
                        gt.declared_ast_nodes.add(t)
                    unit = _extract_unit(clean_v, tokens)
                    scalar = _extract_numeric_scalar(clean_v)

                    # 1. Integer count structural attributes:
                    is_int = scalar is not None and (
                        type_hint.lower() in ("integer", "int") or
                        (clean_v.isdigit() and not unit)
                    )

                    if is_int:
                        int_val = int(scalar)
                        if _is_count_target(k) and int_val not in STANDARD_PROTOCOL_NUMBERS:
                            gt.structural_attributes[k_norm] = int_val
                            root_tokens = [t for t in tokens if t not in ("actuators", "count", "quantity", "qty", "surfaces", "channels")]
                            if root_tokens:
                                gt.structural_attributes["".join(root_tokens)] = int_val
                                singular = root_tokens[-1].rstrip("s")
                                gt.structural_attributes["".join(root_tokens[:-1] + [singular])] = int_val
                            for t in tokens:
                                if t not in NON_HARDWARE_GENERIC_TOKENS and len(t) >= 3:
                                    gt.declared_parts.add(t)
                    elif (scalar is None or type_hint.lower() in ("string", "str")) and _is_config_target(k):
                        gt.structural_attributes[k_norm] = clean_v
                        root_tokens = [t for t in tokens if t not in ("configuration", "config", "type", "mode", "layout", "geometry", "bus", "architecture", "topology", "arrangement")]
                        if root_tokens:
                            gt.structural_attributes["".join(root_tokens)] = clean_v

                    # 2. Numeric limits:
                    has_limit_tokens = any(t in tokens for t in ("limit", "max", "maximum", "min", "minimum", "low", "lower", "floor", "ceiling", "high", "load", "accel", "bound", "threshold", "capacity"))
                    if scalar is not None and (unit or has_limit_tokens or type_hint.lower() in ("real", "float")):
                        limit_val = float(scalar)
                        bound_type = "lower" if _is_lower_bound_name(k, tokens) else "upper"
                        self._register_numeric_limit(gt, k, limit_val, unit, bound_type, owner=row_owner)

                    # 3. If 3rd cell (Description) contains compound configuration descriptors, extract them generically
                    if len(cells) >= 3:
                        desc = cells[2]
                        for m_desc in re.finditer(r'\b([a-zA-Z0-9]+(?:-[a-zA-Z0-9]+)?[- ][a-zA-Z0-9]+)\s+(?:arrangement|configuration|layout|geometry)\b', desc, re.I):
                            cfg_val = m_desc.group(1).strip()
                            cfg_tokens = _tokenize_identifier(cfg_val)
                            if len(cfg_tokens) >= 2:
                                noun = cfg_tokens[-1]
                                if not any(noun.endswith(ex) for ex in CONFIG_TARGET_EXCLUSION_SUFFIXES):
                                    gt.structural_attributes[noun] = cfg_val

            # Check for bullet points: - Key: Value
            m_bullet = re.match(r'^[-*]\s*([a-zA-Z0-9_\s]+)\s*[:=]\s*([^;\n]+)$', line_str)
            if m_bullet:
                k = m_bullet.group(1).strip()
                v = m_bullet.group(2).strip()
                clean_v = v.strip('"\'`')
                k_norm = _normalize_name(k)
                gt.attributes[k_norm] = clean_v
                gt.declared_ast_nodes.add(k_norm)
                gt.declared_ast_nodes.add(_normalize_name(clean_v))

                tokens = _tokenize_identifier(k)
                for t in tokens:
                    gt.declared_ast_nodes.add(t)
                unit = _extract_unit(clean_v, tokens)
                scalar = _extract_numeric_scalar(clean_v)
                if scalar is not None and clean_v.isdigit() and not unit:
                    int_val = int(scalar)
                    if _is_count_target(k) and int_val not in STANDARD_PROTOCOL_NUMBERS:
                        gt.structural_attributes[k_norm] = int_val
                        root_tokens = [t for t in tokens if t not in ("count", "qty", "quantity", "actuators", "surfaces", "channels")]
                        if root_tokens:
                            gt.structural_attributes["".join(root_tokens)] = int_val
                        for t in tokens:
                            if t not in NON_HARDWARE_GENERIC_TOKENS and len(t) >= 3:
                                gt.declared_parts.add(t)
                elif scalar is not None and (unit or any(t in tokens for t in ("limit", "max", "min", "minimum", "low", "lower", "floor", "ceiling", "high", "load", "accel", "bound", "threshold"))):
                    limit_val = float(scalar)
                    bound_type = "lower" if _is_lower_bound_name(k, tokens) else "upper"
                    self._register_numeric_limit(gt, k, limit_val, unit, bound_type, owner=current_owner)
                elif scalar is None and _is_config_target(k):
                    gt.structural_attributes[k_norm] = clean_v
                    root_tokens = [t for t in tokens if t not in ("configuration", "config", "type", "mode", "layout", "geometry", "architecture", "topology", "arrangement")]
                    if root_tokens:
                        gt.structural_attributes["".join(root_tokens)] = clean_v

    # Backward-compatibility alias
    _ingest_schema_markdown = _extract_from_markdown

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

    def _validate_structural_assertions(
        self,
        content: str,
        rel_path: str,
        gt: SchemaGroundTruth
    ) -> List[Finding]:
        """
        Validates structural component counts and configuration descriptors against schema ground truth.
        Emits Finding('factual-grounding-numeric-drift', ...).
        """
        findings: List[Finding] = []
        lines = content.splitlines()

        current_heading = "Header"
        is_normative = True
        in_code_block = False

        count_targets: Dict[str, int] = {}
        config_targets: Dict[str, str] = {}

        for k, v in gt.structural_attributes.items():
            if isinstance(v, int):
                if v not in STANDARD_PROTOCOL_NUMBERS and not any(k.endswith(ex) for ex in COUNT_TARGET_EXCLUSION_SUFFIXES):
                    count_targets[k] = v
            elif isinstance(v, str) and len(v) >= 2:
                if not any(k.endswith(ex) for ex in CONFIG_TARGET_EXCLUSION_SUFFIXES):
                    config_targets[k] = v

        # Collect structural nouns strictly from declared config_targets:
        # (e.g. tail from tailConfiguration, empennage from empennageConfiguration, wing from wingConfiguration, chassis from chassisConfig)
        # Do NOT add decomposed tokens from arbitrary part def names (power, segment, fiber, sensor, launch)
        structural_nouns: Set[str] = set()

        for k, v in config_targets.items():
            # Second token / noun of compound value (e.g. "X-tail" -> "tail", "swept-wing" -> "wing", "delta wing" -> "wing")
            v_toks = _tokenize_identifier(str(v))
            if len(v_toks) >= 2:
                noun_val = v_toks[-1].lower()
                if noun_val not in NON_HARDWARE_GENERIC_TOKENS and len(noun_val) >= 3:
                    structural_nouns.add(noun_val)
            # Attribute noun from key (e.g. "tailConfiguration" -> "tail", "empennageConfiguration" -> "empennage", "chassisConfig" -> "chassis", "wingConfiguration" -> "wing")
            k_toks = [t for t in _tokenize_identifier(k) if t not in ("configuration", "config", "type", "mode", "layout", "geometry", "architecture", "topology", "arrangement")]
            for tok in k_toks:
                tok_clean = tok.lower()
                for suffix in ("configuration", "config", "layout", "geometry", "arrangement", "topology", "architecture", "type"):
                    if tok_clean.endswith(suffix) and len(tok_clean) > len(suffix):
                        tok_clean = tok_clean[:-len(suffix)]
                        break
                if tok_clean not in NON_HARDWARE_GENERIC_TOKENS and len(tok_clean) >= 3:
                    structural_nouns.add(tok_clean)

        pat_compound_desc = (
            re.compile(
                r'\b([a-zA-Z0-9]+-(?:' + '|'.join(re.escape(n) for n in sorted(structural_nouns, key=len, reverse=True)) + r'))\b',
                re.I
            )
            if structural_nouns
            else None
        )

        WORD_NUMBERS = {
            "one": 1, "two": 2, "three": 3, "four": 4, "five": 5,
            "six": 6, "seven": 7, "eight": 8, "nine": 9, "ten": 10,
            "single": 1, "dual": 2, "twin": 2, "triple": 3, "quad": 4, "octo": 8
        }

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

            # 1. Check integer count assertions
            matched_count_entities: Set[str] = set()
            for entity_key, expected_count in count_targets.items():
                if len(entity_key) < 3 or entity_key in matched_count_entities:
                    continue

                pattern = re.compile(
                    r'\b(?:(?:RS|EIA|TIA|MIL-STD|MIL-HDBK|MIL-SPEC|STANAG|ARINC|DO|IEEE|ISO)[- ]?)?'
                    r'(\d+|one|two|three|four|five|six|seven|eight|nine|ten|single|dual|twin|triple|quad)\s*(?:x\s*)?'
                    r'((?:[a-zA-Z-]+\s+){0,2})' + re.escape(entity_key) + r'(?:e?s|\b)',
                    re.I
                )
                for m_count in pattern.finditer(line_str):
                    num_word = m_count.group(1).lower()
                    claimed_count = int(num_word) if num_word.isdigit() else WORD_NUMBERS.get(num_word)
                    if claimed_count is None:
                        continue

                    # Protocol / Standard Number Exclusions:
                    num_start = m_count.start(1)
                    num_end = m_count.end(1)
                    if _is_protocol_or_standard_number(line_str, num_start, num_end, claimed_count):
                        continue

                    if claimed_count != expected_count:
                        claimed_text = m_count.group(0).strip()
                        plural_suffix = "es" if entity_key.endswith("s") else "s"
                        findings.append(Finding(
                            "factual-grounding-numeric-drift",
                            f"{rel_path}:{lineno_1idx}: Ungrounded structural assertion '{claimed_text}' contradicts schema ground truth ({expected_count} {entity_key}{plural_suffix}) in {', '.join(gt.source_files) or 'schema/'}.",
                            location=f"{rel_path}:{lineno_1idx}",
                            detail={"file": rel_path, "line": lineno_1idx, "claimed": claimed_text, "expected": expected_count}
                        ))
                        matched_count_entities.add(entity_key)
                        break

            # 2. Check configuration descriptor drift
            reported_descriptors_on_line: Set[str] = set()
            for entity_key, expected_cfg in config_targets.items():
                if len(expected_cfg) < 3 or expected_cfg.upper() in gt.declared_protocols:
                    continue

                cfg_norm = _normalize_name(expected_cfg)
                m_cfg_parts = re.match(r'^([a-zA-Z0-9]+)[- ]([a-zA-Z0-9]+)$', expected_cfg)
                if m_cfg_parts:
                    cfg_prefix = m_cfg_parts.group(1)
                    cfg_noun = m_cfg_parts.group(2)

                    pat_desc = re.compile(
                        r'\b([a-zA-Z0-9]+(?:-[a-zA-Z0-9]+)?[- ]' + re.escape(cfg_noun) + r')\b',
                        re.I
                    )
                    for m_desc in pat_desc.finditer(line_str):
                        desc_claimed = m_desc.group(1).strip()
                        if desc_claimed in reported_descriptors_on_line:
                            continue
                        enclosing_tok = _get_enclosing_hyphenated_token(line_str, m_desc.start(), m_desc.end())
                        if _is_tracer_or_signal_identifier(desc_claimed) or _is_tracer_or_signal_identifier(enclosing_tok):
                            continue
                        desc_claimed_norm = _normalize_name(desc_claimed)
                        if desc_claimed_norm != cfg_norm:
                            if cfg_norm not in _normalize_name(line_str):
                                findings.append(Finding(
                                    "factual-grounding-numeric-drift",
                                    f"{rel_path}:{lineno_1idx}: Structural descriptor '{desc_claimed}' contradicts schema ground truth ({expected_cfg}) in {', '.join(gt.source_files) or 'schema/'}.",
                                    location=f"{rel_path}:{lineno_1idx}",
                                    detail={"file": rel_path, "line": lineno_1idx, "descriptor": desc_claimed, "expected": expected_cfg}
                                ))
                                reported_descriptors_on_line.add(desc_claimed)
                                break

            # 3. Closed-world structural descriptor resolution: check candidate compound descriptors
            if pat_compound_desc:
                for m_desc in pat_compound_desc.finditer(line_str):
                    desc_claimed = m_desc.group(1).strip()
                    if desc_claimed in reported_descriptors_on_line:
                        continue
                    if desc_claimed.upper() in gt.declared_protocols or any(p.upper() == desc_claimed.upper() for p in RECOGNIZED_PROTOCOLS):
                        continue
                    enclosing_tok = _get_enclosing_hyphenated_token(line_str, m_desc.start(), m_desc.end())
                    if _is_tracer_or_signal_identifier(desc_claimed) or _is_tracer_or_signal_identifier(enclosing_tok):
                        continue
                    desc_claimed_norm = _normalize_name(desc_claimed)
                    if not desc_claimed_norm or desc_claimed_norm.isdigit():
                        continue

                    # Ensure generic hyphenated English phrases (e.g. real-time, valid-range, sign-off, multi-mode, two-step, single-stage, three-state)
                    # do not get flagged unless their noun specifically matches a declared physical subsystem or configuration attribute
                    parts = desc_claimed.split('-')
                    noun = parts[-1].lower() if parts else ""
                    if noun in NON_HARDWARE_GENERIC_TOKENS or noun not in structural_nouns:
                        continue

                    is_declared = (
                        desc_claimed_norm in gt.declared_ast_nodes
                        or any(desc_claimed_norm == _normalize_name(v) for v in gt.structural_attributes.values() if isinstance(v, str))
                        or any(desc_claimed_norm == k for k in gt.structural_attributes.keys())
                        or desc_claimed_norm in _normalize_name(gt.raw_schema_text)
                    )
                    if not is_declared:
                        findings.append(Finding(
                            "factual-grounding-numeric-drift",
                            f"{rel_path}:{lineno_1idx}: Ungrounded structural descriptor '{desc_claimed}' is not declared in schema ground truth or AST nodes in {', '.join(gt.source_files) or 'schema/'}.",
                            location=f"{rel_path}:{lineno_1idx}",
                            detail={"file": rel_path, "line": lineno_1idx, "descriptor": desc_claimed}
                        ))
                        reported_descriptors_on_line.add(desc_claimed)

        return findings

    _validate_structural_descriptors = _validate_structural_assertions

    def _validate_numeric_assertions(
        self,
        content: str,
        rel_path: str,
        gt: SchemaGroundTruth
    ) -> List[Finding]:
        """
        Validates numeric quantities and loads against declared limits in schema ground truth.
        Enforces ISO/IEC 80000 & SysML v2 ISQ dimensional quantity typing, atomic identifier lexing,
        fail-closed unitless attribute handling, component-scoped contextual binding, and
        property token specificity for same-unit attributes with IEEE 754 float tolerance.
        Emits Finding('factual-grounding-numeric-drift', ...).
        """
        findings: List[Finding] = []
        lines = content.splitlines()

        current_heading = "Header"
        is_normative = True
        in_code_block = False

        if not gt.numeric_limits and not gt.scoped_numeric_limits:
            return []

        # Collect scoped numeric limits
        scoped_limits: List[ScopedNumericLimit] = []
        if gt.scoped_numeric_limits:
            seen_sigs: Set[Tuple[float, str, str, Optional[str], Tuple[str, ...]]] = set()
            for sl in gt.scoped_numeric_limits:
                sig = (sl.limit_val, sl.unit, sl.bound_type, sl.owner, tuple(sl.meaningful_tokens))
                if sig not in seen_sigs:
                    seen_sigs.add(sig)
                    scoped_limits.append(sl)
        else:
            seen_sigs = set()
            for k, (limit, unit) in gt.numeric_limits.items():
                tokens = _tokenize_identifier(k)
                canon_unit = ISO_80000_PHYSICAL_UNITS.get(unit.lower(), unit.lower()) if unit else ""
                unit_tokens = set()
                if unit:
                    unit_tokens.add(unit.lower())
                    unit_tokens.update(_tokenize_identifier(unit.lower()))
                prop_tokens = []
                for t in tokens:
                    if t in ("limit", "value", "val", "real", "float", "scalar"):
                        continue
                    if t in unit_tokens:
                        continue
                    if canon_unit and ISO_80000_PHYSICAL_UNITS.get(t) == canon_unit:
                        continue
                    prop_tokens.append(t)
                if not prop_tokens:
                    prop_tokens = [t for t in tokens if t not in ("value", "val", "real", "float")]
                bound_type = gt.numeric_bound_types.get(k)
                if not bound_type:
                    bound_type = "lower" if _is_lower_bound_name(k, prop_tokens) else "upper"
                sig = (limit, unit, bound_type, None, tuple(prop_tokens))
                if sig in seen_sigs:
                    continue
                seen_sigs.add(sig)
                scoped_limits.append(ScopedNumericLimit(
                    key=k,
                    limit_val=limit,
                    unit=unit,
                    bound_type=bound_type,
                    owner=None,
                    meaningful_tokens=prop_tokens
                ))

        numeric_pattern = re.compile(
            r'\b(\d+(?:\.\d+)?(?:\s*-\s*\d+(?:\.\d+)?)?)\s*(%|[a-zA-Z/][a-zA-Z0-9/%^*_-]*\b)'
        )

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

            # (a) Atomic Identifier Lexing: protect compound designations and citations
            protected_spans = _get_protected_spans(line_str)
            search_line = _mask_spans(line_str, protected_spans)

            line_tokens = _tokenize_identifier(line_str)
            heading_tokens = _tokenize_identifier(current_heading)
            reported_claims_on_line: Set[str] = set()

            # (b) ISO/IEC 80000 Physical Dimensional Typing:
            # An engineering numeric claim exists if and only if paired with recognized physical unit
            for match in numeric_pattern.finditer(search_line):
                val_range_str = match.group(1).strip()
                unit_raw = match.group(2).strip()
                claimed_str = line_str[match.start():match.end()].strip()

                if claimed_str in reported_claims_on_line:
                    continue

                canon_unit = ISO_80000_PHYSICAL_UNITS.get(unit_raw.lower())
                if canon_unit is None:
                    # Non-dimensional prose: words that are not physical units are ignored
                    continue

                numbers = [float(n) for n in re.findall(r'\d+(?:\.\d+)?', val_range_str)]
                if not numbers:
                    continue
                max_claimed = max(numbers)
                min_claimed = min(numbers)

                for metric in scoped_limits:
                    if not metric.unit:
                        # (c) Fail-closed: attributes without declared physical units do not match physical unit claims
                        continue

                    metric_canon_unit = ISO_80000_PHYSICAL_UNITS.get(metric.unit.lower(), metric.unit.lower())
                    if canon_unit != metric_canon_unit:
                        continue

                    # (d) Component-Scoped Contextual Binding & Property Token Specificity
                    if metric.owner:
                        owner_toks = _tokenize_identifier(metric.owner)
                        owner_in_line = (metric.owner in _normalize_name(line_str) or any(t in line_tokens for t in owner_toks if t not in NON_HARDWARE_GENERIC_TOKENS))
                        owner_in_heading = (metric.owner in _normalize_name(current_heading) or any(t in heading_tokens for t in owner_toks if t not in NON_HARDWARE_GENERIC_TOKENS))

                        # If line explicitly targets another declared component and does not mention this owner, skip
                        other_parts = {p for p in gt.declared_parts if p != metric.owner and (p in line_tokens or p in _normalize_name(line_str))}
                        if other_parts and not owner_in_line:
                            continue
                        if not (owner_in_line or owner_in_heading):
                            continue

                        # Property Token Specificity:
                        # If a component defines multiple attributes sharing the same physical unit,
                        # match the numeric quantity against the attribute whose property tokens match the line / local context.
                        same_owner_unit_metrics = [
                            m for m in scoped_limits
                            if m.owner == metric.owner and ISO_80000_PHYSICAL_UNITS.get(m.unit.lower(), m.unit.lower()) == canon_unit
                        ]
                        distinct_tokens = {tuple(m.meaningful_tokens) for m in same_owner_unit_metrics}
                        if len(distinct_tokens) > 1:
                            clause_start = 0
                            for sep_match in re.finditer(r'[,;|]|\b(?:and|or)\b', line_str[:match.start()]):
                                clause_start = sep_match.end()
                            clause_end = len(line_str)
                            m_end = re.search(r'[,;|]|\b(?:and|or)\b', line_str[match.end():])
                            if m_end:
                                clause_end = match.end() + m_end.start()
                            local_clause = line_str[clause_start:clause_end]
                            local_tokens = _tokenize_identifier(local_clause)

                            def _metric_score(m: ScopedNumericLimit) -> int:
                                if not m.meaningful_tokens:
                                    return 0
                                c_matches = sum(
                                    1 for pt in m.meaningful_tokens
                                    if pt in _normalize_name(local_clause) or any(_property_token_matches(pt, lt) for lt in local_tokens)
                                )
                                l_matches = sum(
                                    1 for pt in m.meaningful_tokens
                                    if pt in _normalize_name(line_str) or any(_property_token_matches(pt, lt) for lt in line_tokens)
                                )
                                return c_matches * 10 + l_matches

                            best_score = max((_metric_score(m) for m in same_owner_unit_metrics), default=0)
                            if best_score == 0:
                                continue
                            if _metric_score(metric) < best_score:
                                continue
                    else:
                        token_matches = any(
                            any(_property_token_matches(t, lt) for lt in line_tokens)
                            for t in metric.meaningful_tokens
                        )
                        if not token_matches and metric.unit != "g":
                            continue

                        if metric.unit == "g" and not token_matches:
                            g_context = any(t in line_tokens for t in ("launch", "load", "accel", "acceleration", "gload", "rail", "profile", "catapult"))
                            if not g_context:
                                continue

                    if metric.bound_type == "lower":
                        if min_claimed < metric.limit_val - 1e-6:
                            findings.append(Finding(
                                "factual-grounding-numeric-drift",
                                f"{rel_path}:{lineno_1idx}: Fabricated numeric quantity '{claimed_str}' falls below schema ground truth lower bound ({metric.limit_val:.1f}{metric.unit}) in {', '.join(gt.source_files) or 'schema/'}.",
                                location=f"{rel_path}:{lineno_1idx}",
                                detail={
                                    "file": rel_path,
                                    "line": lineno_1idx,
                                    "claimed": claimed_str,
                                    "ground_truth_limit": metric.limit_val,
                                    "ground_truth_lower_bound": metric.limit_val,
                                    "bound_type": "lower",
                                    "unit": metric.unit
                                }
                            ))
                            reported_claims_on_line.add(claimed_str)
                            break
                    else:
                        if max_claimed > metric.limit_val + 1e-6:
                            findings.append(Finding(
                                "factual-grounding-numeric-drift",
                                f"{rel_path}:{lineno_1idx}: Fabricated numeric quantity '{claimed_str}' exceeds schema ground truth limit ({metric.limit_val:.1f}{metric.unit}) in {', '.join(gt.source_files) or 'schema/'}.",
                                location=f"{rel_path}:{lineno_1idx}",
                                detail={
                                    "file": rel_path,
                                    "line": lineno_1idx,
                                    "claimed": claimed_str,
                                    "ground_truth_limit": metric.limit_val,
                                    "bound_type": "upper",
                                    "unit": metric.unit
                                }
                            ))
                            reported_claims_on_line.add(claimed_str)
                            break

            # (c) Fail-Closed Unitless Attributes evaluation:
            for metric in scoped_limits:
                if metric.unit:
                    continue  # Only evaluate unitless attributes here

                # Strictly require an explicit attribute identifier match in local statement
                has_explicit_id = (
                    metric.key in _normalize_name(line_str)
                    or (len(metric.meaningful_tokens) >= 2 and all(t in line_tokens for t in metric.meaningful_tokens))
                )
                if not has_explicit_id:
                    continue

                if metric.owner:
                    owner_toks = _tokenize_identifier(metric.owner)
                    owner_in_line = (metric.owner in _normalize_name(line_str) or any(t in line_tokens for t in owner_toks if t not in NON_HARDWARE_GENERIC_TOKENS))
                    owner_in_heading = (metric.owner in _normalize_name(current_heading) or any(t in heading_tokens for t in owner_toks if t not in NON_HARDWARE_GENERIC_TOKENS))
                    if not (owner_in_line or owner_in_heading):
                        continue

                for m_num in re.finditer(r'\b(\d+(?:\.\d+)?)\b', search_line):
                    num_str = m_num.group(1)
                    if num_str in reported_claims_on_line:
                        continue
                    # Check if followed by a physical unit
                    post_text = search_line[m_num.end():m_num.end() + 10].strip()
                    m_u = re.match(r'^([a-zA-Z/%^]+)', post_text)
                    if m_u and m_u.group(1).lower() in ISO_80000_PHYSICAL_UNITS:
                        continue

                    num_val = float(num_str)
                    if _is_protocol_or_standard_number(line_str, m_num.start(), m_num.end(), int(num_val)):
                        continue

                    if metric.bound_type == "lower":
                        if num_val < metric.limit_val - 1e-6:
                            findings.append(Finding(
                                "factual-grounding-numeric-drift",
                                f"{rel_path}:{lineno_1idx}: Fabricated numeric quantity '{num_str}' falls below schema ground truth lower bound ({metric.limit_val:.1f}) in {', '.join(gt.source_files) or 'schema/'}.",
                                location=f"{rel_path}:{lineno_1idx}",
                                detail={"file": rel_path, "line": lineno_1idx, "claimed": num_str, "ground_truth_limit": metric.limit_val, "bound_type": "lower"}
                            ))
                            reported_claims_on_line.add(num_str)
                            break
                    else:
                        if num_val > metric.limit_val + 1e-6:
                            findings.append(Finding(
                                "factual-grounding-numeric-drift",
                                f"{rel_path}:{lineno_1idx}: Fabricated numeric quantity '{num_str}' exceeds schema ground truth limit ({metric.limit_val:.1f}) in {', '.join(gt.source_files) or 'schema/'}.",
                                location=f"{rel_path}:{lineno_1idx}",
                                detail={"file": rel_path, "line": lineno_1idx, "claimed": num_str, "ground_truth_limit": metric.limit_val, "bound_type": "upper"}
                            ))
                            reported_claims_on_line.add(num_str)
                            break

        return findings

    _validate_numeric_quantities = _validate_numeric_assertions

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
