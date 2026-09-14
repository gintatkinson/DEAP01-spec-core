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
    "SysMLPackage", "SysMLParser", "PartDef", "AttributeDef", "ItemDef"
])
SysMLPackage = _sysml_ast.SysMLPackage
SysMLParser = _sysml_ast.SysMLParser
PartDef = _sysml_ast.PartDef
AttributeDef = _sysml_ast.AttributeDef
ItemDef = _sysml_ast.ItemDef


# Non-normative section heading patterns (e.g. Glossary, MCDA trade study, Acronyms, Standards/Regulatory baseline)
NON_NORMATIVE_SECTION_PATTERNS = [
    re.compile(r'\b(?:glossary|acronyms?|abbreviations?|definitions?|terminology|lexicon|vocabulary)\b', re.I),
    re.compile(r'\b(?:trade\s+stud(?:y|ies)|trade-?off|mcda|multi-criteria|rejected\s+alternatives?|candidate\s+analysis|trade\s+space|decision\s+matrix|evaluation\s+of\s+alternatives)\b', re.I),
    re.compile(r'\b(?:revision\s+history|document\s+history|document\s+control|change\s+log|changelog)\b', re.I),
    re.compile(r'\b(?:references?|applicable\s+documents|reference\s+standards|normative\s+standards|standards\s+baseline|regulatory\s+baseline|regulatory\s+framework|standards\s+and\s+regulatory|standards\s+taxonomy)\b', re.I),
    re.compile(r'\b(?:dual-track\s+mbd|simulation\s+deliverables|digital\s+twin(?:\s+engine)?|test\s+coverage|matlab\s*/?\s*simulink(?:\s+synthesis)?)\b', re.I),
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
    "Profibus", "Profinet", "EtherCAT", "Micro-D",
    # Digital ESC & motor protocols
    "DShot", "DShot150", "DShot300", "DShot600", "DShot1200",
    "ProShot", "ProShot1000", "MultiShot", "OneShot", "OneShot125", "OneShot42",
    # Serial receiver & telemetry protocols
    "CRSF", "Crossfire", "SBUS", "S.BUS", "IBUS", "FPort", "F.Port", "DSM2", "DSMX",
    # Pulse & signal protocols
    "PWM", "PPM",
]

# Epistemic tier annotation pattern for declared engineering decisions and TBD parameters
EPISTEMIC_EXEMPTION_PATTERN = re.compile(
    r'\[TIER-(?:3|4)(?::\s*[^\]]+)?\]',
    re.I
)


def _has_epistemic_exemption(line: str) -> bool:
    """
    Checks if a line contains an epistemic tier exemption ([TIER-3: DESIGN] or [TIER-4: TBD]).
    Tier 3 (Design Decisions) and Tier 4 (TBD/Unspecified) are acknowledged engineering
    decisions rather than fabricated OEM claims.
    """
    return bool(EPISTEMIC_EXEMPTION_PATTERN.search(line))


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
    "record", "document", "register", "entry", "section", "table", "launch",
    "check", "test", "coverage", "solver", "simulation", "twin", "ground",
}

STOP_WORDS_AND_DETERMINERS: Set[str] = {
    "the", "a", "an", "this", "that", "these", "those", "each", "every",
    "all", "both", "either", "neither", "one", "two", "three", "four", "five",
    "six", "seven", "eight", "nine", "ten", "first", "second", "third",
    "between", "on", "in", "at", "to", "for", "with", "from", "by", "about",
    "into", "through", "during", "before", "after", "above", "below", "up",
    "down", "out", "off", "over", "under", "again", "further", "then", "once",
    "here", "there", "when", "where", "why", "how", "and", "or", "but", "if",
    "while", "as", "of", "not", "no", "nor", "too", "very", "can", "will",
    "just", "should", "now", "per", "via", "multi", "single", "dual", "twin",
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
    s = re.sub(r'[^a-zA-Z0-9]', ' ', str(ident))
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
    EXCLUDES generic non-hardware targets such as bare type, record, document, metadata, etc.
    """
    if not name:
        return False
    name_norm = _normalize_name(name)
    if not name_norm or name_norm in ("type", "record", "document", "metadata", "section", "spec", "spectype", "table", "item"):
        return False
    tokens = _tokenize_identifier(name)
    if not tokens:
        return False
    last_tok = tokens[-1].lower()
    if last_tok in CONFIG_TARGET_EXCLUSION_SUFFIXES:
        return False
    if any(name_norm.endswith(ex) for ex in CONFIG_TARGET_EXCLUSION_SUFFIXES):
        return False
    # Require at least one non-config subject token (bare 'type' or 'config' without a subject noun is not a hardware config target)
    non_config_tokens = [t for t in tokens if t not in ("configuration", "config", "layout", "arrangement", "topology", "architecture", "type")]
    if not non_config_tokens:
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

    # Information & Data Rate (ISO/IEC 80000-13)
    "bps": "bps", "bit/s": "bps", "bits/s": "bps",
    "kbps": "kbps", "kbit/s": "kbps",
    "mbps": "mbps", "mbit/s": "mbps",
    "gbps": "gbps", "gbit/s": "gbps",
    "baud": "baud", "bd": "baud",
}

VEHICLE_CLASSIFIER_TOKENS: Set[str] = {
    "uav", "uas", "drone", "vehicle", "aircraft", "system", "subsystem",
    "assembled", "overall", "total", "general", "standard", "default",
    "device", "component", "module", "airframe", "platform", "item",
}


# Atomic compound identifiers and standards citations
ATOMIC_IDENTIFIER_PATTERN = re.compile(
    r'\b[A-Za-z0-9]+(?:[-/_][A-Za-z0-9]+)+\b'
)
STANDARDS_CITATION_PATTERN = re.compile(
    r'\b(?:DO|ARP|MIL|STD|ASTM|ISO|IEC|IEEE|STANAG|ARINC|RTCA|SAE|DEF-STAN)[-_ ]*[0-9]+[A-Za-z0-9]*\b',
    re.I
)
# Procedural and task identifiers (e.g. Task 202, Method 514.8, Phase 1, Clause 4.2)
PROCEDURAL_IDENTIFIER_PATTERN = re.compile(
    r'\b(?:Task|Method|Methodology|Phase|Clause)\s+\d+(?:\.\d+)*\b',
    re.I
)
MODEL_AND_DESIGNATION_PATTERN = re.compile(
    r'\b(?:Avenger|Model|Mk|Mark|Block|Lot|Type|Group|Class|Option|Figure|Table|Section|Clause)\s+\d+(?:\.\d+)*\b',
    re.I
)
SECTION_AND_TRACER_PATTERN = re.compile(
    r'(?:§\s*\d+(?:\.\d+)*|\b\d+\.\d+(?:\.\d+)+\b|\b(?:SIG|REQ|CONN|FEAT|US|UC|EPIC|RULE|TEST|OSO|UCA|HAZ|SAF)-[A-Za-z0-9_]+(?:\.\.[A-Za-z0-9_]+)?\b)',
    re.I
)


def _is_numeric_range_or_quantity(tok: str) -> bool:
    """
    Checks if a compound token like '15-20', '15-20g', or 'm/s' is actually a numeric range/quantity
    or physical unit rather than an indivisible reference symbol / standard citation.
    """
    if tok.lower() in ISO_80000_PHYSICAL_UNITS:
        return True
    m = re.match(r'^(\d+(?:\.\d+)?)\s*-\s*(\d+(?:\.\d+)?)([a-zA-Z/%^]*)$', tok)
    if m:
        suffix = m.group(3).strip().lower()
        if not suffix or suffix in ISO_80000_PHYSICAL_UNITS:
            return True
    return False


def _normalize_math_block(inner: str) -> str:
    """Normalizes the interior content of a KaTeX math expression ($...$ or $$...$$)."""
    s = inner
    # 1. En-dashes / em-dashes / hyphens inside \text{...} or raw LaTeX
    s = re.sub(r'\\text\{--+\}', '-', s)
    s = re.sub(r'\\text\{-\}', '-', s)
    s = re.sub(r'--+', '-', s)
    # 2. Degree symbol \circ, ^{\circ}, ^\circ
    s = re.sub(r'\^?\s*\{?\\circ\}?', ' deg', s)
    # 3. Relational and mathematical operators (\ge, \le, \sim, \approx, \pm, \times, \cdot)
    s = re.sub(r'\\(?:ge|geq)\b', '>=', s)
    s = re.sub(r'\\(?:le|leq)\b', '<=', s)
    s = re.sub(r'\\(?:sim|approx)\b', '~', s)
    s = re.sub(r'\\pm\b', '+/-', s)
    s = re.sub(r'\\times\b', '*', s)
    s = re.sub(r'\\cdot\b', '*', s)
    # 4. Text and font wrappers: \text{...}, \mathrm{...}, etc.
    s = re.sub(r'\\(?:text|mathrm|operatorname|mathbf|mathit)\{([^}]*)\}', r' \1 ', s)
    # 5. Remove remaining LaTeX command backslashes
    s = re.sub(r'\\[a-zA-Z]+', '', s)
    # 6. Remove braces, backslashes, carets
    s = s.replace('{', '').replace('}', '').replace('\\', '').replace('^', '')
    # 7. Normalize range hyphens e.g. 13 - 14 -> 13-14
    s = re.sub(r'(\d+)\s*-\s*(\d+)', r'\1-\2', s)
    # 8. Normalize relational operators spacing e.g. >=50 -> >= 50
    s = re.sub(r'([><]=?)\s*(\d)', r'\1 \2', s)
    # 9. Normalize whitespace
    s = re.sub(r'\s+', ' ', s).strip()
    return s


def _normalize_katex_math_expressions(text: str) -> str:
    """
    Normalizes KaTeX/LaTeX math expressions into plain-text engineering representations.
    Translates inline ($...$) and display ($$...$$) math expressions:
    - \\text{--} or \\text{-} -> -
    - \\text{([a-zA-Z/%^*_-]+)} -> \\1
    - \\ge, \\le, \\sim, \\approx, \\pm -> >=, <=, ~, ~, +/-
    - \\circ -> deg
    - Strips math delimiters ($) and braces so that:
      '$13\\text{--}14\\text{ bar}$' -> '13-14 bar'
      '$h \\ge 50\\text{ m}$' -> 'h >= 50 m'
      '$r = 300\\text{ m}$' -> 'r = 300 m'
      '$\\le 55\\text{ m/s}$' -> '<= 55 m/s'
      '$15^\\circ$' -> '15 deg'
    """
    if '$' not in text:
        return text
    # Display math $$ ... $$
    text = re.sub(r'\$\$(.*?)\$\$', lambda m: _normalize_math_block(m.group(1)), text, flags=re.DOTALL)
    # Inline math $ ... $
    text = re.sub(r'(?<!\\)\$(.*?)(?<!\\)\$', lambda m: _normalize_math_block(m.group(1)), text)
    return text


def _get_protected_spans(line: str) -> List[Tuple[int, int]]:
    """
    Identifies spans of atomic compound identifiers, procedural identifiers, and standards citations
    that must be protected from being fragmented into numeric scalars or unit abbreviations.
    """
    spans: List[Tuple[int, int]] = []

    for m in PROCEDURAL_IDENTIFIER_PATTERN.finditer(line):
        spans.append((m.start(), m.end()))

    for m in MODEL_AND_DESIGNATION_PATTERN.finditer(line):
        spans.append((m.start(), m.end()))

    for m in SECTION_AND_TRACER_PATTERN.finditer(line):
        spans.append((m.start(), m.end()))

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


def _extract_numeric_range(val_str: str) -> Optional[Tuple[float, float]]:
    """
    Extracts lower and upper numeric bounds from a range string (e.g. '4.4 - 5.0 GHz', '13-14 bar', '49–50 V', '60 km / 90 km', '[0,1800]').
    Returns (min_val, max_val) or None if not a range.
    """
    if not val_str:
        return None
    s = str(val_str).strip()
    if re.search(r'\b\d{4}[-/]\d{2}[-/]\d{2}\b', s):
        return None

    # 1. Bracketed mathematical intervals e.g. [0, 1800], [0,1800], [0 ; 2^8-1], [0; 255]
    m_bracket = re.search(r'\[\s*([-+]?\d+(?:\.\d+)?)\s*[,;\u2013\u2014\-]\s*([^\]]+)\]', s)
    if m_bracket:
        try:
            v1 = float(m_bracket.group(1))
            upper_raw = m_bracket.group(2).strip()
            m_pow = re.search(r'2\^(\d+)(?:\s*-\s*1)?', upper_raw)
            if m_pow:
                exp = int(m_pow.group(1))
                v2 = float((1 << exp) - 1 if '- 1' in upper_raw or '-1' in upper_raw else (1 << exp))
            else:
                m_num = re.search(r'[-+]?\d+(?:\.\d+)?', upper_raw)
                if m_num:
                    v2 = float(m_num.group(0))
                else:
                    return None
            return (min(v1, v2), max(v1, v2))
        except (ValueError, OverflowError):
            pass

    # 2. General range patterns e.g. '4.4 - 5.0 GHz', '13-14 bar', '49–50 V', '60 km / 90 km', '60km/180km'
    m = re.search(
        r'([-+]?\d+(?:\.\d+)?)\s*(?:[a-zA-Z/%^]+)?\s*(?:[\-\u2013\u2014/,;]|to|or|\.{2,3})\s*([-+]?\d+(?:\.\d+)?)',
        s
    )
    if m:
        try:
            v1 = float(m.group(1))
            v2 = float(m.group(2))
            return (min(v1, v2), max(v1, v2))
        except ValueError:
            return None
    return None


def _extract_unit(val_str: str, name_tokens: Optional[List[str]] = None) -> str:
    """Extracts ISO/IEC 80000 recognized physical measurement unit from value string or name tokens."""
    if val_str:
        for m in re.finditer(r'[-+]?\d+(?:\.\d+)?\s*\[?([a-zA-Z/%^]+)\]?', str(val_str)):
            cand = m.group(1).lower()
            canon = ISO_80000_PHYSICAL_UNITS.get(cand)
            if canon:
                return canon

    if name_tokens:
        last_tok = name_tokens[-1].lower()
        if last_tok in ("ms", "mps"):
            if any(t in name_tokens for t in ("speed", "wind", "airspeed", "velocity", "cruise", "stall", "dive", "horizontal", "vertical", "rate")):
                return "m/s"
            if last_tok == "mps":
                return "m/s"
            return "ms"
        if last_tok in ISO_80000_PHYSICAL_UNITS:
            return ISO_80000_PHYSICAL_UNITS[last_tok]
        if last_tok == "pct":
            return "%"
        if "g" in name_tokens and any(t in name_tokens for t in ("load", "accel", "acceleration", "limit")):
            return "g"
    return ""


def _is_nominal_name(name: str, tokens: Optional[List[str]] = None) -> bool:
    """Checks if attribute represents a nominal setpoint rather than an upper/lower bound."""
    if not name:
        return False
    toks = tokens if tokens is not None else _tokenize_identifier(name)
    toks_l = [t.lower() for t in toks]
    return any(t in ("nom", "nominal") for t in toks_l) or "nom" in name.lower()


def _is_lower_bound_name(name: str, tokens: Optional[List[str]] = None) -> bool:
    """
    Checks if attribute name represents a lower bound:
    Attributes whose names contain min, low, or floor (and not max) are lower bounds.
    """
    if not name or _is_nominal_name(name, tokens):
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
    if not name or _is_nominal_name(name, tokens):
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
    if prop_l in ("respond", "response", "responding") and text_l in ("respond", "response", "responding"):
        return True
    if prop_l in ("rotator", "rotation", "rotate", "azimuth") and text_l in ("rotator", "rotation", "rotate", "azimuth"):
        return True
    # Plural and verb inflection stemming (e.g. abandon/abandons/abandoned, wait/waits, command/commands)
    if prop_l.rstrip('s') == text_l.rstrip('s') and len(prop_l.rstrip('s')) >= 3:
        return True
    if len(prop_l) >= 4 and len(text_l) >= 4:
        if (text_l.startswith(prop_l) or prop_l.startswith(text_l)) and abs(len(prop_l) - len(text_l)) <= 3:
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
    declared_frequencies: Set[str] = field(default_factory=set)


GroundTruth = SchemaGroundTruth


class MechanicalSectionSlicer:
    """
    Parses Markdown AST headers and slices text under specific section locators (§X.Y.Z or Header Titles).
    Verifies that claimed tokens exist in the exact section text slice before validating citations.
    """
    def __init__(self, workspace_dir: str):
        self.workspace_dir = workspace_dir
        self._cache: Dict[str, Dict[str, str]] = {}
        self._raw_cache: Dict[str, str] = {}

    def _resolve_path(self, file_rel_path: str) -> Optional[str]:
        if not file_rel_path:
            return None
        full_path = os.path.join(self.workspace_dir, file_rel_path) if not os.path.isabs(file_rel_path) else file_rel_path
        if os.path.isfile(full_path):
            return full_path
        if not file_rel_path.startswith("schema/"):
            cand = os.path.join(self.workspace_dir, "schema", file_rel_path)
            if os.path.isfile(cand):
                return cand
        elif file_rel_path.startswith("schema/"):
            cand = os.path.join(self.workspace_dir, file_rel_path[len("schema/"):])
            if os.path.isfile(cand):
                return cand
        if not file_rel_path.startswith("docs/"):
            cand = os.path.join(self.workspace_dir, "docs", file_rel_path)
            if os.path.isfile(cand):
                return cand
            docs_dir = os.path.join(self.workspace_dir, "docs")
            if os.path.isdir(docs_dir):
                for root, _, files in os.walk(docs_dir):
                    if file_rel_path in files:
                        return os.path.join(root, file_rel_path)
        return None

    def get_file_text(self, file_rel_path: str) -> Optional[str]:
        full_path = self._resolve_path(file_rel_path)
        if not full_path:
            return None
        if full_path not in self._raw_cache:
            try:
                with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                    self._raw_cache[full_path] = f.read()
            except Exception:
                return None
        return self._raw_cache.get(full_path)

    def _index_sections(self, full_path: str) -> Dict[str, str]:
        text = self.get_file_text(full_path)
        if text is None:
            return {}
        lines = text.splitlines(keepends=True)
        sections: Dict[str, str] = {}

        headers: List[Tuple[int, int, str]] = []  # (line_idx, level, heading_text)
        for idx, line in enumerate(lines):
            m = re.match(r'^(#{1,6})\s+(.+)$', line.strip())
            if m:
                level = len(m.group(1))
                heading = m.group(2).strip()
                headers.append((idx, level, heading))

        if not headers:
            sections["root"] = "".join(lines)
            return sections

        if headers[0][0] > 0:
            sections["root"] = "".join(lines[:headers[0][0]])

        for i, (start_line, level, heading) in enumerate(headers):
            end_line = len(lines)
            for next_idx in range(i + 1, len(headers)):
                next_start, next_level, _ = headers[next_idx]
                if next_level <= level:
                    end_line = next_start
                    break
            sec_text = "".join(lines[start_line:end_line])

            heading_lower = heading.lower().strip()
            sections[heading_lower] = sec_text

            m_sec = re.search(r'§?\s*(\d+(?:\.\d+)*)', heading)
            if m_sec:
                sec_num = m_sec.group(1).strip()
                sections[sec_num] = sec_text
                sections[f"§{sec_num}"] = sec_text
                sections[f"section {sec_num}"] = sec_text
                sections[f"clause {sec_num}"] = sec_text

            slug = re.sub(r'[^a-z0-9]', '', heading_lower)
            if slug:
                sections[slug] = sec_text

            words_only = re.sub(r'^[§0-9.\-:\s]+', '', heading).strip().lower()
            if words_only and words_only != heading_lower:
                sections[words_only] = sec_text
                words_slug = re.sub(r'[^a-z0-9]', '', words_only)
                if words_slug:
                    sections[words_slug] = sec_text

        return sections

    def slice_section(self, file_rel_path: str, section_locator: str) -> Optional[str]:
        full_path = self._resolve_path(file_rel_path)
        if not full_path:
            return None
        if full_path not in self._cache:
            self._cache[full_path] = self._index_sections(full_path)
        sections = self._cache[full_path]

        norm_loc = section_locator.strip("§ #").lower()
        if norm_loc in sections:
            return sections[norm_loc]

        m_sec = re.search(r'(\d+(?:\.\d+)*)', section_locator)
        if m_sec and m_sec.group(1) in sections:
            return sections[m_sec.group(1)]

        slug = re.sub(r'[^a-z0-9]', '', section_locator.lower())
        if slug in sections:
            return sections[slug]

        for k, v in sections.items():
            if norm_loc and (norm_loc in k or k in norm_loc):
                return v

        for k, v in sections.items():
            if slug and (slug in k or k in slug):
                return v

        if "generalsafety" in slug or "safety" in slug:
            for k, v in sections.items():
                if "safety" in k or "warning" in k:
                    return v

        return None

    def verify_claimed_tokens(
        self,
        file_rel_path: str,
        section_locator: Optional[str],
        tokens: List[str]
    ) -> Tuple[bool, List[str], str]:
        full_path = self._resolve_path(file_rel_path)
        if not full_path:
            return False, tokens, f"Cited file '{file_rel_path}' not found in workspace."

        if section_locator:
            text_slice = self.slice_section(file_rel_path, section_locator)
            if text_slice is None:
                return False, tokens, f"Section '{section_locator}' not found in '{file_rel_path}'."
        else:
            text_slice = self.get_file_text(file_rel_path)
            if text_slice is None:
                return False, tokens, f"Could not read content from '{file_rel_path}'."

        if not tokens:
            return True, [], ""

        text_slice = _normalize_katex_math_expressions(text_slice)
        norm_slice = text_slice.lower()
        condensed_slice = re.sub(r'[^a-z0-9]', '', norm_slice)

        missing: List[str] = []
        for tok in tokens:
            tok_clean = tok.strip().lower()
            if not tok_clean:
                continue

            if tok_clean in norm_slice:
                continue

            tok_condensed = re.sub(r'[^a-z0-9]', '', tok_clean)
            if tok_condensed and tok_condensed in condensed_slice:
                continue

            m_q = re.match(r'^(\d+(?:\.\d+)?)\s*([a-zA-Z/%^]+)$', tok.strip())
            if m_q:
                num_s = m_q.group(1)
                unit_s = m_q.group(2)
                int_s = str(int(float(num_s))) if float(num_s).is_integer() else num_s
                q_pat = re.compile(r'\b' + re.escape(int_s) + r'(?:\.0+)?\s*' + re.escape(unit_s) + r'\b', re.I)
                if q_pat.search(norm_slice):
                    continue

            parts = [p for p in re.split(r'[^a-z0-9]', tok_clean) if p]
            if len(parts) > 1 and all(p in norm_slice for p in parts):
                continue

            missing.append(tok)

        if missing:
            loc_desc = f"section '{section_locator}'" if section_locator else f"file '{file_rel_path}'"
            return False, missing, f"Tokens {missing} missing from {loc_desc} in '{file_rel_path}'."

        return True, [], ""


class FactualGroundingValidator(IValidator):
    """
    Factual Grounding & Parametric SSOT Parity Gate Validator (Check 23).
    """

    def __init__(self, workspace_repo: Optional[WorkspaceRepository] = None, **kwargs):
        self.workspace_repo = workspace_repo
        self._section_slicer: Optional[MechanicalSectionSlicer] = None
        if workspace_repo:
            self._section_slicer = MechanicalSectionSlicer(workspace_repo.workspace_dir)
        self._citation_fraud_findings: List[Finding] = []
        self._seen_fraud_sigs: Set[Tuple[str, str, str]] = set()

    _normalize_katex_math_expressions = staticmethod(_normalize_katex_math_expressions)

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
        tokens = _tokenize_identifier(name)
        if _is_nominal_name(name, tokens):
            bound_type = "nominal"

        name_norm = _normalize_name(name)
        gt.numeric_limits[name_norm] = (limit_val, unit)
        gt.numeric_bound_types[name_norm] = bound_type

        meaningful = [t for t in tokens if t not in ("limit", "value", "val", "real", "float")]

        canon_unit = ISO_80000_PHYSICAL_UNITS.get(unit.lower(), unit.lower()) if unit else ""
        unit_tokens = set()
        if unit:
            unit_tokens.add(unit.lower())
            unit_tokens.update(_tokenize_identifier(unit.lower()))

        GENERIC_PROPERTY_TOKENS: Set[str] = {
            "mode", "type", "configuration", "config", "spec", "item", "parameter",
            "attribute", "field", "record", "document", "category", "target", "source",
        }

        prop_tokens = []
        for t in tokens:
            if t in ("limit", "value", "val", "real", "float", "scalar", "number", "num"):
                continue
            if t in unit_tokens:
                continue
            if canon_unit and ISO_80000_PHYSICAL_UNITS.get(t) == canon_unit:
                continue
            if t in VEHICLE_CLASSIFIER_TOKENS:
                continue
            if t in GENERIC_PROPERTY_TOKENS and len(tokens) > 1:
                continue
            prop_tokens.append(t)
        if not prop_tokens:
            prop_tokens = [t for t in tokens if t not in ("value", "val", "real", "float") and t not in unit_tokens]

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
        self.workspace_repo = repo
        self._section_slicer = MechanicalSectionSlicer(repo.workspace_dir)
        self._citation_fraud_findings = []
        self._seen_fraud_sigs = set()

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

        # Include mechanically detected citation fraud findings
        findings.extend(self._citation_fraud_findings)

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
            else:
                proto_norm = _normalize_name(proto)
                if proto_norm and re.search(r'\b' + re.escape(proto_norm) + r'\b', gt.raw_schema_text, re.I):
                    gt.declared_protocols.add(proto_norm)

        # Check for generic protocol keywords declared in schema (e.g. "RS-485", "CAN", "UART", "MAVLink")
        generic_protos = ["rs485", "rs422", "rs232", "can", "canopen", "mavlink", "ethernet", "spacewire", "milstd1553", "arinc429", "modbus", "uart", "spi", "i2c"]
        for gp in generic_protos:
            if re.search(r'\b' + gp + r'\b', gt.raw_schema_text, re.I) or gp in _normalize_name(gt.raw_schema_text):
                gt.declared_protocols.add(gp)

        # 4. Detect declared frequencies / execution rates across raw schema text & numeric limits
        for m in re.finditer(r'\b(\d+(?:\.\d+)?)\s*(Hz|kHz|MHz|GHz)\b', gt.raw_schema_text, re.I):
            val_f = float(m.group(1))
            unit_str = m.group(2).lower()
            if unit_str == "khz":
                val_f *= 1000.0
            gt.declared_frequencies.add(str(int(val_f) if val_f.is_integer() else val_f))
            gt.declared_frequencies.add(f"{int(val_f) if val_f.is_integer() else val_f} hz")
            gt.declared_frequencies.add(f"{int(val_f) if val_f.is_integer() else val_f}hz")

        for k, (limit, unit) in gt.numeric_limits.items():
            if unit and unit.lower() in ("hz", "khz"):
                val_f = limit * (1000.0 if unit.lower() == "khz" else 1.0)
                gt.declared_frequencies.add(str(int(val_f) if val_f.is_integer() else val_f))
                gt.declared_frequencies.add(f"{int(val_f) if val_f.is_integer() else val_f} hz")
                gt.declared_frequencies.add(f"{int(val_f) if val_f.is_integer() else val_f}hz")

        # Check if schema actually defines concrete architectural ground truth
        has_concrete = bool(
            gt.structural_attributes
            or gt.numeric_limits
            or gt.attributes
            or gt.declared_protocols
            or gt.declared_frequencies
        )
        if not has_concrete:
            gt.has_concrete_schema = False

        return gt

    def _extract_sysml_attributes_from_block(
        self,
        text: str,
        gt: SchemaGroundTruth,
        owner: Optional[str] = None,
        is_item_def: bool = False
    ) -> None:
        if not is_item_def:
            item_pattern = re.compile(r'\bitem\s+(?:def\s+)?([a-zA-Z0-9_]+)\s*\{([^}]*)\}', re.DOTALL)
            item_spans: List[Tuple[int, int]] = []
            for match in item_pattern.finditer(text):
                item_spans.append((match.start(), match.end()))
                iname = match.group(1).strip()
                iname_norm = _normalize_name(iname)
                ibody = match.group(2)
                if iname_norm:
                    gt.declared_ast_nodes.add(iname_norm)
                    for tok in _tokenize_identifier(iname):
                        gt.declared_ast_nodes.add(tok)
                self._extract_sysml_attributes_from_block(ibody, gt, owner=iname_norm, is_item_def=True)

            bare_item_pattern = re.compile(r'\bitem\s+(?:def\s+)?([a-zA-Z0-9_]+)\b')
            for match in bare_item_pattern.finditer(text):
                iname = match.group(1).strip()
                iname_norm = _normalize_name(iname)
                if iname_norm:
                    gt.declared_ast_nodes.add(iname_norm)
                    for tok in _tokenize_identifier(iname):
                        gt.declared_ast_nodes.add(tok)

            if item_spans:
                text = _mask_spans(text, item_spans)

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

            # Signal message payload fields in item defs are data structure fields, NOT physical system operational limits or counts.
            if is_item_def:
                continue

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
                if _is_nominal_name(name, tokens):
                    bound_type = "nominal"
                elif _is_lower_bound_name(name, tokens):
                    bound_type = "lower"
                else:
                    bound_type = "upper"
                self._register_numeric_limit(gt, name, limit_val, unit, bound_type, owner=owner)


    def _extract_from_sysml(self, text: str, gt: SchemaGroundTruth) -> None:
        """
        Generic AST extraction for SysML attribute definitions with component scoping:
        Ingests ANY typed attribute into gt.structural_attributes and/or gt.numeric_limits
        with owning component tracking.
        Differentiates physical/logical component definitions (part def) and package constraints
        from signal message payload item definitions (item def).
        """
        # 1. Ingest item definitions (signal message payload fields - excluded from numeric limits)
        item_pattern = re.compile(r'\bitem\s+(?:def\s+)?([a-zA-Z0-9_]+)\s*\{([^}]*)\}', re.DOTALL)
        item_spans: List[Tuple[int, int]] = []
        for match in item_pattern.finditer(text):
            item_spans.append((match.start(), match.end()))
            iname = match.group(1).strip()
            iname_norm = _normalize_name(iname)
            ibody = match.group(2)
            if iname_norm:
                gt.declared_ast_nodes.add(iname_norm)
                for tok in _tokenize_identifier(iname):
                    gt.declared_ast_nodes.add(tok)
            self._extract_sysml_attributes_from_block(ibody, gt, owner=iname_norm, is_item_def=True)

        bare_item_pattern = re.compile(r'\bitem\s+(?:def\s+)?([a-zA-Z0-9_]+)\b')
        for match in bare_item_pattern.finditer(text):
            iname = match.group(1).strip()
            iname_norm = _normalize_name(iname)
            if iname_norm:
                gt.declared_ast_nodes.add(iname_norm)
                for tok in _tokenize_identifier(iname):
                    gt.declared_ast_nodes.add(tok)

        # 2. Ingest part definitions directly from SysML text with their bodies
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
            self._extract_sysml_attributes_from_block(pbody, gt, owner=pname_norm, is_item_def=False)

        # 3. Ingest package-level attributes (outside part defs AND outside item defs)
        top_level_text = _mask_spans(text, part_spans + item_spans)
        self._extract_sysml_attributes_from_block(top_level_text, gt, owner=None, is_item_def=False)

        # 4. Ingest any remaining part declarations without block braces
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

        # Ingest attribute_defs (package constraints)
        for attr in getattr(pkg, "attribute_defs", []) or getattr(pkg, "attributes", []) or []:
            name = getattr(attr, "name", "")
            type_str = getattr(attr, "type_name", None) or getattr(attr, "type", "") or ""
            val_str = getattr(attr, "default_value", None) or getattr(attr, "doc", "") or ""
            if name and val_str:
                stmt = f"attribute {name} : {type_str} = {val_str};"
                self._extract_sysml_attributes_from_block(stmt, gt, owner=None, is_item_def=False)

        # Ingest item_defs (signal / message payload definitions - excluded from numeric limits)
        for item in getattr(pkg, "item_defs", []) or getattr(pkg, "items", []) or []:
            iname = getattr(item, "name", "")
            iname_norm = _normalize_name(iname)
            if iname:
                gt.declared_ast_nodes.add(iname_norm)
                for tok in _tokenize_identifier(iname):
                    gt.declared_ast_nodes.add(tok)
            for attr in getattr(item, "attributes", []) or getattr(item, "attribute_defs", []) or []:
                aname = getattr(attr, "name", "")
                type_str = getattr(attr, "type_name", None) or getattr(attr, "type", "") or ""
                val_str = getattr(attr, "default_value", None) or getattr(attr, "doc", "") or ""
                if aname:
                    aname_norm = _normalize_name(aname)
                    gt.declared_ast_nodes.add(aname_norm)
                    for tok in _tokenize_identifier(aname):
                        gt.declared_ast_nodes.add(tok)
                    if val_str:
                        stmt = f"attribute {aname} : {type_str} = {val_str};"
                        self._extract_sysml_attributes_from_block(stmt, gt, owner=iname_norm, is_item_def=True)

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
            for item in getattr(part, "item_defs", []) or getattr(part, "items", []) or []:
                iname = getattr(item, "name", "")
                iname_norm = _normalize_name(iname)
                if iname:
                    gt.declared_ast_nodes.add(iname_norm)
                    for tok in _tokenize_identifier(iname):
                        gt.declared_ast_nodes.add(tok)
                for attr in getattr(item, "attributes", []) or getattr(item, "attribute_defs", []) or []:
                    aname = getattr(attr, "name", "")
                    type_str = getattr(attr, "type_name", None) or getattr(attr, "type", "") or ""
                    val_str = getattr(attr, "default_value", None) or getattr(attr, "doc", "") or ""
                    if aname:
                        aname_norm = _normalize_name(aname)
                        gt.declared_ast_nodes.add(aname_norm)
                        for tok in _tokenize_identifier(aname):
                            gt.declared_ast_nodes.add(tok)
                        if val_str:
                            stmt = f"attribute {aname} : {type_str} = {val_str};"
                            self._extract_sysml_attributes_from_block(stmt, gt, owner=iname_norm, is_item_def=True)
            for attr in getattr(part, "attributes", []) or getattr(part, "attribute_defs", []) or []:
                aname = getattr(attr, "name", "")
                aname_norm = _normalize_name(aname)
                aval = getattr(attr, "default_value", None) or ""
                ascalar = _extract_numeric_scalar(aval)
                if _is_count_target(aname) and ascalar is not None and int(ascalar) not in STANDARD_PROTOCOL_NUMBERS:
                    gt.structural_attributes[pname_norm] = int(ascalar)
                elif aname and aval:
                    stmt = f"attribute {aname} = {aval};"
                    self._extract_sysml_attributes_from_block(stmt, gt, owner=pname_norm, is_item_def=False)

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
                    if part in h_norm or part.rstrip('s') == h_norm.rstrip('s') or (len(part) >= 4 and len(h_norm) >= 4 and (part.startswith(h_norm) or h_norm.startswith(part))):
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
                        "component", "property", "parameter", "item", "attribute", "name", "field", "data"
                    ):
                        continue

                    # Reject markdown table keys that are pure integers or digits (e.g. connector pin numbers 7, 8, 12 in pinout tables)
                    # or keys that start with numbers/ranges (e.g. 'Min. 5 Byte', '10 m') or shorter than 3 alphabetic characters.
                    clean_k = k.strip("*_`[] \t")
                    if clean_k.isdigit() or sum(1 for c in clean_k if c.isalpha()) < 3:
                        continue
                    if re.match(r'^(?:min\.?|max\.?)?\s*\d+', clean_k, re.I):
                        continue

                    row_owner = current_owner
                    k_norm = _normalize_name(k)
                    for part in gt.declared_parts:
                        if part in k_norm:
                            row_owner = part
                            break

                    # If cell 1 is a type (e.g. Integer, Real, String, UINT16, FLOAT32), value is cell 2 or range column
                    type_hint = ""
                    unit_override = ""
                    type_pattern = re.compile(r'^(?:u?int\d*|float\d*|byte|char|short|long|double|boolean|bool|string|real|integer|int|float)$', re.I)
                    if len(cells) >= 3 and (type_pattern.match(cells[1].strip().lower()) or cells[1].lower() in ("integer", "int", "real", "float", "string", "boolean")):
                        type_hint = cells[1].strip()
                        # If table has Range column (e.g. | Data | Type | Offset | Range | Unit | Description |)
                        if len(cells) >= 5:
                            v = cells[3]
                            clean_v = v.strip('"\'`')
                            if cells[4].strip() not in ("-", "None", ""):
                                unit_override = cells[4].strip()
                        else:
                            v = cells[2]
                            clean_v = v.strip('"\'`')
                    else:
                        clean_v = v.strip('"\'`')

                    gt.attributes[k_norm] = clean_v
                    gt.declared_ast_nodes.add(k_norm)
                    gt.declared_ast_nodes.add(_normalize_name(clean_v))

                    tokens = _tokenize_identifier(k)
                    for t in tokens:
                        gt.declared_ast_nodes.add(t)

                    unit = unit_override or _extract_unit(clean_v, tokens)
                    if type_pattern.match(clean_v.lower()):
                        scalar = None
                    else:
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
                    is_protocol_data_field = any(
                        k_norm.endswith(sfx) for sfx in (
                            "flag", "flags", "mask", "masks", "byte", "bytes", "result",
                            "code", "codes", "status", "statuses", "offset", "offsets",
                            "packet", "header", "crc", "checksum", "version"
                        )
                    )
                    if not is_protocol_data_field:
                        has_limit_tokens = any(t in tokens for t in ("limit", "max", "maximum", "min", "minimum", "low", "lower", "floor", "ceiling", "high", "load", "accel", "bound", "threshold", "capacity"))
                        num_range = _extract_numeric_range(clean_v)
                        if num_range is not None and (unit or has_limit_tokens or type_hint.lower() in ("real", "float")):
                            min_val, max_val = num_range
                            self._register_numeric_limit(gt, k, min_val, unit, "lower", owner=row_owner)
                            self._register_numeric_limit(gt, k, max_val, unit, "upper", owner=row_owner)
                        elif scalar is not None and (unit or has_limit_tokens or type_hint.lower() in ("real", "float")):
                            limit_val = float(scalar)
                            if _is_nominal_name(k, tokens):
                                bound_type = "nominal"
                            elif _is_lower_bound_name(k, tokens):
                                bound_type = "lower"
                            else:
                                bound_type = "upper"
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
                num_range = _extract_numeric_range(clean_v)
                if num_range is not None and (unit or any(t in tokens for t in ("limit", "max", "min", "minimum", "low", "lower", "floor", "ceiling", "high", "load", "accel", "bound", "threshold"))):
                    min_val, max_val = num_range
                    self._register_numeric_limit(gt, k, min_val, unit, "lower", owner=current_owner)
                    self._register_numeric_limit(gt, k, max_val, unit, "upper", owner=current_owner)
                elif scalar is not None and (unit or any(t in tokens for t in ("limit", "max", "min", "minimum", "low", "lower", "floor", "ceiling", "high", "load", "accel", "bound", "threshold"))):
                    limit_val = float(scalar)
                    if _is_nominal_name(k, tokens):
                        bound_type = "nominal"
                    elif _is_lower_bound_name(k, tokens):
                        bound_type = "lower"
                    else:
                        bound_type = "upper"
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

    def _extract_citation_target(self, line: str) -> Tuple[Optional[str], Optional[str]]:
        """
        Extracts target file path and optional section locator from a citation in line.
        Returns (file_path, section_locator).
        """
        # 1. HTML comment e.g. <!-- Source: schema/a5-user-manual-2.md §7.2.3 -->
        m_comment = re.search(
            r'<!--\s*(?:Source|SSOT|Grounding|Reference):\s*([^\s>]+)(?:\s+([^>]+?))?\s*-->',
            line,
            re.I
        )
        if m_comment:
            target = m_comment.group(1).strip()
            loc = m_comment.group(2).strip() if m_comment.group(2) else None
            if '#' in target and not loc:
                parts = target.split('#', 1)
                target = parts[0]
                loc = parts[1]
            return target, loc

        # 2. Markdown link e.g. [User Manual §7.2.3](schema/a5-user-manual-2.md) or [Manual](schema/a5-user-manual-2.md#723)
        m_link = re.search(r'\[([^\]]*)\]\(([^)]*?(?:schema|docs)/[^)]*)\)', line, re.I)
        if m_link:
            link_text = m_link.group(1).strip()
            link_target = m_link.group(2).strip()
            loc = None
            if '#' in link_target:
                parts = link_target.split('#', 1)
                link_target = parts[0]
                loc = parts[1]
            if not loc and ('§' in link_text or re.search(r'\b\d+(?:\.\d+)+\b', link_text)):
                loc = link_text
            return link_target, loc

        # 3. Path in prose or table e.g. `schema/a5-user-manual-2.md` §7.2.3 or docs/conops/CONOPS.md §7.1
        m_path = re.search(
            r'(?:^|[\s`\'"(\[<|])(?:\.\.?/)?((?:schema|docs)/[a-zA-Z0-9_./\-]+\.[a-zA-Z0-9]+)(?:#([a-zA-Z0-9_\-]+))?',
            line,
            re.I
        )
        if m_path:
            target = m_path.group(1).strip()
            loc = m_path.group(2).strip() if m_path.group(2) else None
            m_sec = re.search(r'§\s*([0-9]+(?:\.[0-9]+)*)', line)
            if m_sec:
                loc = m_sec.group(0).strip()
            return target, loc

        # 4. Bare filename in schema e.g. a5-user-manual-2.md §7.2.3
        m_bare = re.search(
            r'\b([a-zA-Z0-9_\-]+\.(?:md|sysml))\b(?:\s+(?:§\s*([0-9]+(?:\.[0-9]+)*)|#\s*([a-zA-Z0-9_\-]+)))?',
            line,
            re.I
        )
        if m_bare:
            fn = m_bare.group(1)
            loc = m_bare.group(2) or m_bare.group(3)
            if loc:
                loc = f"§{loc}" if m_bare.group(2) else loc
            return fn, loc

        return None, None

    def _extract_candidate_tokens(self, line: str) -> List[str]:
        """Extracts candidate technical tokens (quantities with units, protocols) from line."""
        clean = _normalize_katex_math_expressions(line)
        clean = re.sub(r'<!--.*?-->', '', clean)
        clean = re.sub(r'\[([^\]]*)\]\([^)]*?(?:schema|docs)/[^)]*\)', r'\1', clean)
        clean = re.sub(r'(?:^|[\s`\'"(\[<|])(?:\.\.?/)?(?:schema|docs)/[a-zA-Z0-9_./#:\-]+', '', clean)
        clean = re.sub(r'§\s*\d+(?:\.\d+)*', '', clean)
        clean = re.sub(r'\[TIER-[0-9][^\]]*\]', '', clean)

        tokens: List[str] = []

        # 1. Numeric quantities with units (e.g. "50 Hz", "400 Hz", "12g", "25 kg", "13-14 bar")
        for m in re.finditer(r'\b(\d+(?:\.\d+)?(?:\s*[-\u2013\u2014]\s*\d+(?:\.\d+)?)?)\s*(%|[a-zA-Z/][a-zA-Z0-9/%^*_-]*\b)', clean):
            cand_tok = clean[m.start():m.end()].strip()
            unit_part = m.group(2).strip().lower()
            if unit_part in ISO_80000_PHYSICAL_UNITS or unit_part == "%":
                tokens.append(cand_tok)

        # 2. Recognized protocols (e.g. "PWM", "DShot600", "RS-485", "MAVLink")
        for proto in sorted(RECOGNIZED_PROTOCOLS, key=len, reverse=True):
            if re.search(r'\b' + re.escape(proto) + r'\b', clean, re.I):
                if proto not in tokens:
                    tokens.append(proto)

        return tokens

    def _is_frequency_declared(self, val_f: float, freq_token: str, gt: SchemaGroundTruth) -> bool:
        """Checks if a frequency quantity is declared in schema ground truth."""
        if not gt.has_concrete_schema:
            return True
        int_val = int(val_f) if val_f.is_integer() else None
        candidates = {str(val_f), str(int_val)} if int_val is not None else {str(val_f)}
        for c in candidates:
            if c in gt.declared_frequencies or f"{c} hz" in gt.declared_frequencies or f"{c}hz" in gt.declared_frequencies:
                return True

        for k, (limit, unit) in gt.numeric_limits.items():
            if unit and unit.lower() in ("hz", "khz"):
                lim_f = limit * (1000.0 if unit.lower() == "khz" else 1.0)
                if abs(lim_f - val_f) < 1e-6:
                    return True

        for sl in gt.scoped_numeric_limits:
            if sl.unit and sl.unit.lower() in ("hz", "khz"):
                lim_f = sl.limit_val * (1000.0 if sl.unit.lower() == "khz" else 1.0)
                if abs(lim_f - val_f) < 1e-6:
                    return True

        if gt.raw_schema_text:
            num_pattern = re.escape(str(int_val if int_val is not None else val_f))
            if re.search(r'\b' + num_pattern + r'\s*Hz\b', gt.raw_schema_text, re.I):
                return True

        return False

    def _is_token_in_ground_truth(self, tok: str, gt: Optional[SchemaGroundTruth]) -> bool:
        """Verifies whether a token is declared, derived, or grounded in SchemaGroundTruth."""
        if not gt:
            return False

        t_clean = tok.strip()
        t_norm = _normalize_name(t_clean)

        # 1. Direct matches in AST identifiers, protocols, and parts
        if (
            t_norm in gt.declared_protocols
            or t_norm in gt.declared_ast_nodes
            or t_norm in gt.declared_parts
            or (t_clean.lower() in gt.raw_schema_text.lower())
        ):
            return True

        # 2. Check if frequency is declared
        m_freq = re.match(r'^(\d+(?:\.\d+)?)\s*(hz|khz|mhz|ghz)$', t_clean, re.I)
        if m_freq:
            val_f = float(m_freq.group(1))
            unit_str = m_freq.group(2).lower()
            if unit_str == "khz":
                val_f *= 1000.0
            elif unit_str == "mhz":
                val_f *= 1000000.0
            elif unit_str == "ghz":
                val_f *= 1000000000.0
            if self._is_frequency_declared(val_f, t_clean, gt):
                return True

        # 3. Check numeric quantities with units (e.g. "12g", "25 kg", "12.0 g", "13-14 bar")
        m_quant = re.match(r'^(\d+(?:\.\d+)?(?:\s*[-\u2013\u2014]\s*\d+(?:\.\d+)?)?)\s*([a-zA-Z/%^]+)$', t_clean)
        if m_quant:
            val_range_str = m_quant.group(1)
            unit_str = m_quant.group(2).strip().lower()
            canon_unit = ISO_80000_PHYSICAL_UNITS.get(unit_str, unit_str)
            numbers = [float(n) for n in re.findall(r'\d+(?:\.\d+)?', val_range_str)]
            if not numbers:
                return False

            def _is_num_in_gt(val_f: float) -> bool:
                # Check gt.numeric_limits
                for key, (lim_val, lim_unit) in gt.numeric_limits.items():
                    if abs(lim_val - val_f) < 1e-6:
                        lim_unit_norm = lim_unit.lower() if lim_unit else ""
                        lim_canon = ISO_80000_PHYSICAL_UNITS.get(lim_unit_norm, lim_unit_norm)
                        if lim_canon == canon_unit or lim_unit_norm == unit_str:
                            return True
                        if unit_str in ("g", "kg", "m", "s", "v", "a", "w") and (unit_str in key or key.endswith(unit_str)):
                            return True

                # Check gt.scoped_numeric_limits
                for sl in gt.scoped_numeric_limits:
                    if abs(sl.limit_val - val_f) < 1e-6:
                        sl_unit_norm = sl.unit.lower() if sl.unit else ""
                        sl_canon = ISO_80000_PHYSICAL_UNITS.get(sl_unit_norm, sl_unit_norm)
                        if sl_canon == canon_unit or sl_unit_norm == unit_str:
                            return True
                        if unit_str in ("g", "kg", "m", "s", "v", "a", "w") and (unit_str in sl.key or sl.key.endswith(unit_str)):
                            return True

                # Check gt.structural_attributes
                if val_f.is_integer():
                    int_v = int(val_f)
                    for k, v in gt.structural_attributes.items():
                        if v == int_v:
                            return True

                # Check raw schema text for scalar value + unit/property occurrence
                num_s = str(int(val_f) if val_f.is_integer() else val_f)
                if re.search(r'\b' + re.escape(num_s) + r'(?:\.0+)?\b', gt.raw_schema_text) and (
                    re.search(r'\b' + re.escape(unit_str) + r'\b', gt.raw_schema_text, re.I)
                    or re.search(r'[A-Za-z]' + re.escape(unit_str) + r'\b', gt.raw_schema_text)
                ):
                    return True
                return False

            if all(_is_num_in_gt(n) for n in numbers):
                return True

        return False

    def _extract_all_citations(self, line: str) -> List[Tuple[str, Optional[str]]]:
        """
        Extracts all target file paths and optional section locators from citations in line.
        Returns list of (file_path, section_locator).
        """
        results: List[Tuple[str, Optional[str]]] = []
        seen = set()

        # 1. HTML comment e.g. <!-- Source: schema/a5-user-manual-2.md §7.2.3 -->
        for m in re.finditer(
            r'<!--\s*(?:Source|SSOT|Grounding|Reference):\s*([^\s>]+)(?:\s+([^>]+?))?\s*-->',
            line,
            re.I
        ):
            target = m.group(1).strip()
            loc = m.group(2).strip() if m.group(2) else None
            if '#' in target and not loc:
                parts = target.split('#', 1)
                target = parts[0]
                loc = parts[1]
            pair = (target, loc)
            if pair not in seen:
                seen.add(pair)
                results.append(pair)

        # 2. Markdown link e.g. [User Manual §7.2.3](schema/a5-user-manual-2.md) or [Manual](schema/a5-user-manual-2.md#723)
        for m in re.finditer(r'\[([^\]]*)\]\(([^)]*?(?:schema|docs)/[^)]*)\)', line, re.I):
            link_text = m.group(1).strip()
            link_target = m.group(2).strip()
            loc = None
            if '#' in link_target:
                parts = link_target.split('#', 1)
                link_target = parts[0]
                loc = parts[1]
            if not loc and ('§' in link_text or re.search(r'\b\d+(?:\.\d+)+\b', link_text)):
                loc = link_text
            pair = (link_target, loc)
            if pair not in seen:
                seen.add(pair)
                results.append(pair)

        # 3. Path in prose or table e.g. `schema/a5-user-manual-2.md` §7.2.3
        for m in re.finditer(
            r'(?:^|[\s`\'"(\[<|])(?:\.\.?/)?((?:schema|docs)/[a-zA-Z0-9_./\-]+\.[a-zA-Z0-9]+)(?:#([a-zA-Z0-9_\-]+))?((?:\s*[,;]?\s*§\s*[^`\'",\)\n;]+)*)',
            line,
            re.I
        ):
            target = m.group(1).strip()
            loc = m.group(2).strip() if m.group(2) else None
            extra_secs = m.group(3) if m.group(3) else ""
            found_locs = re.findall(r'§\s*([^`\'",\)\n;]+)', extra_secs)
            if found_locs:
                for fl in found_locs:
                    clean_fl = fl.strip()
                    sec_tag = f"§{clean_fl}" if not clean_fl.startswith("§") else clean_fl
                    pair = (target, sec_tag)
                    if pair not in seen:
                        seen.add(pair)
                        results.append(pair)
            else:
                pair = (target, loc)
                if pair not in seen:
                    seen.add(pair)
                    results.append(pair)

        # 4. Bare filename in schema e.g. a5-user-manual-2.md §7.2.3
        for m in re.finditer(
            r'\b([a-zA-Z0-9_\-]+\.(?:md|sysml))\b(?:\s+(?:§\s*([0-9]+(?:\.[0-9]+)*)|#\s*([a-zA-Z0-9_\-]+)))?',
            line,
            re.I
        ):
            fn = m.group(1)
            loc = m.group(2) or m.group(3)
            if loc:
                loc = f"§{loc}" if m.group(2) else loc
            if not any(r[0].endswith(fn) for r in results):
                pair = (fn, loc)
                if pair not in seen:
                    seen.add(pair)
                    results.append(pair)

        return results

    def _has_ssot_citation(
        self,
        line: str,
        content: str,
        rel_path: str,
        gt: Optional[SchemaGroundTruth] = None,
        candidate_tokens: Optional[List[str]] = None,
        lineno: int = 1,
        claim_text: str = ""
    ) -> bool:
        """
        Checks if a claim carries a valid, verified SSOT citation.
        Performs mechanical markdown section text slicing when section locators are specified.
        Supports compound statements citing multiple sources across clauses.
        Emits Finding('factual-grounding-citation-fraud', ...) if claimed tokens are missing from cited sections.
        Document-level frontmatter references do NOT exempt lines from validation.
        """
        citations = self._extract_all_citations(line)
        if not citations:
            target_file, locator = self._extract_citation_target(line)
            if target_file:
                citations.append((target_file, locator))
            elif gt and gt.source_files:
                for sf in gt.source_files:
                    bname = os.path.basename(sf)
                    if len(bname) >= 5 and (bname in line or sf in line):
                        target_file = sf
                        m_sec = re.search(r'§\s*([0-9]+(?:\.[0-9]+)*)', line)
                        locator = m_sec.group(0).strip() if m_sec else None
                        citations.append((target_file, locator))
                        break

        if not citations:
            return False

        # Ensure slicer is initialized
        if self._section_slicer is None:
            ws_dir = self.workspace_repo.workspace_dir if self.workspace_repo else os.getcwd()
            self._section_slicer = MechanicalSectionSlicer(ws_dir)

        reported_claim = claim_text.strip() if claim_text else line.strip()

        # Check all cited files exist
        for target_file, locator in citations:
            resolved = self._section_slicer._resolve_path(target_file)
            if not resolved or not os.path.isfile(resolved):
                fraud_finding = Finding(
                    "factual-grounding-citation-fraud",
                    f"{rel_path}:{lineno}: Critical citation fraud: cited source file '{target_file}' does not exist in workspace.",
                    location=f"{rel_path}:{lineno}",
                    detail={
                        "file": rel_path,
                        "line": lineno,
                        "claimed": reported_claim,
                        "target_file": target_file,
                        "reason": f"Cited source file '{target_file}' does not exist in workspace."
                    }
                )
                sig = (fraud_finding.rule_id, fraud_finding.location, fraud_finding.detail.get("reason", ""))
                if sig not in self._seen_fraud_sigs:
                    self._seen_fraud_sigs.add(sig)
                    self._citation_fraud_findings.append(fraud_finding)
                return False

        tokens = candidate_tokens if candidate_tokens is not None else self._extract_candidate_tokens(line)

        # If no tokens to verify, verify that cited sections exist
        if not tokens:
            for target_file, locator in citations:
                if locator and not re.match(r'^L\d+', locator, re.I):
                    ok, missing, reason = self._section_slicer.verify_claimed_tokens(target_file, locator, [])
                    if not ok:
                        msg = f"{rel_path}:{lineno}: Critical citation fraud: claim '{reported_claim}' cites '{target_file} {locator}': {reason}"
                        fraud_finding = Finding(
                            "factual-grounding-citation-fraud",
                            msg,
                            location=f"{rel_path}:{lineno}",
                            detail={
                                "file": rel_path,
                                "line": lineno,
                                "claimed": reported_claim,
                                "target_file": target_file,
                                "section": locator,
                                "missing_tokens": [],
                                "reason": reason
                            }
                        )
                        sig = (fraud_finding.rule_id, fraud_finding.location, reason)
                        if sig not in self._seen_fraud_sigs:
                            self._seen_fraud_sigs.add(sig)
                            self._citation_fraud_findings.append(fraud_finding)
                        return False
            return True

        # When tokens are present, verify that each token is substantiated by at least one cited section/file
        remaining_tokens = list(tokens)
        for target_file, locator in citations:
            loc = locator if (locator and not re.match(r'^L\d+', locator, re.I)) else None
            verified: List[str] = []
            for tok in remaining_tokens:
                ok, _, _ = self._section_slicer.verify_claimed_tokens(target_file, loc, [tok])
                if ok:
                    verified.append(tok)
            for tok in verified:
                remaining_tokens.remove(tok)
            if not remaining_tokens:
                break

        if not remaining_tokens:
            return True

        # Check remaining tokens against ground truth
        truly_missing: List[str] = []
        for tok in remaining_tokens:
            if not self._is_token_in_ground_truth(tok, gt):
                truly_missing.append(tok)

        if truly_missing:
            target_file, locator = citations[0]
            loc_str = f" {locator}" if locator else ""
            msg = f"{rel_path}:{lineno}: Critical citation fraud: claim '{reported_claim}' cites '{target_file}{loc_str}', but token(s) {truly_missing} do not exist within cited sources."
            fraud_finding = Finding(
                "factual-grounding-citation-fraud",
                msg,
                location=f"{rel_path}:{lineno}",
                detail={
                    "file": rel_path,
                    "line": lineno,
                    "claimed": reported_claim,
                    "target_file": target_file,
                    "section": locator,
                    "missing_tokens": truly_missing,
                    "reason": "Token(s) missing from cited sections"
                }
            )
            sig = (fraud_finding.rule_id, fraud_finding.location, str(truly_missing))
            if sig not in self._seen_fraud_sigs:
                self._seen_fraud_sigs.add(sig)
                self._citation_fraud_findings.append(fraud_finding)
            return False

        return True

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
        non_normative_depth: Optional[int] = None
        is_normative = True
        in_code_block = False
        in_frontmatter = False

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

        current_citation: Optional[str] = None

        for lineno_1idx, line in enumerate(lines, start=1):
            line_str = line.strip()

            if lineno_1idx == 1 and line_str == "---":
                in_frontmatter = True
                continue
            if in_frontmatter:
                if line_str == "---":
                    in_frontmatter = False
                continue

            if line_str.startswith("```"):
                in_code_block = not in_code_block
                continue
            if in_code_block or not line_str:
                if not line_str:
                    current_citation = None
                continue

            # Heading detection
            m_head = re.match(r'^(#{1,6})\s+(.+)$', line_str)
            if m_head:
                current_citation = None
                level = len(m_head.group(1))
                current_heading = m_head.group(2).strip()
                if non_normative_depth is not None and level <= non_normative_depth:
                    non_normative_depth = None
                if self._is_non_normative_section(current_heading):
                    non_normative_depth = level
                is_normative = (non_normative_depth is None)
                continue

            if not is_normative:
                continue

            # Track block/paragraph citation comments (e.g. <!-- Source: ... -->)
            if re.search(r'<!--\s*(?:Source|SSOT|Grounding|Reference):\s*[^>]+-->', line_str, re.I):
                current_citation = line_str
                if re.match(r'^\s*<!--\s*(?:Source|SSOT|Grounding|Reference):\s*[^>]+-->\s*$', line_str, re.I):
                    continue

            # Skip rejected trade study rows
            if re.search(r'\b(?:rejected|discarded|eliminated|not\s+selected|cons|fail)\b', line_str, re.I):
                continue

            # Skip lines with epistemic exemptions ([TIER-3: DESIGN], [TIER-4: TBD])
            if _has_epistemic_exemption(line_str):
                continue

            line_str = _normalize_katex_math_expressions(line_str)

            # Check if line has explicit SSOT citation (inline or block)
            citation_to_check = line_str if self._extract_citation_target(line_str)[0] else current_citation
            if citation_to_check:
                candidate_tokens = self._extract_candidate_tokens(line_str)
                if self._has_ssot_citation(citation_to_check, content, rel_path, gt, candidate_tokens=candidate_tokens, lineno=lineno_1idx, claim_text=line_str):
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
                m_cfg_parts = re.match(r'^(.+)[- ]([a-zA-Z0-9]+)$', expected_cfg)
                if m_cfg_parts:
                    cfg_prefix = m_cfg_parts.group(1).strip()
                    cfg_noun = m_cfg_parts.group(2).strip()

                    pat_desc = re.compile(
                        r'\b([a-zA-Z0-9]+(?:-[a-zA-Z0-9]+)?[- ]' + re.escape(cfg_noun) + r')\b',
                        re.I
                    )
                    for m_desc in pat_desc.finditer(line_str):
                        desc_claimed = m_desc.group(1).strip()
                        if desc_claimed in reported_descriptors_on_line:
                            continue
                        first_word = desc_claimed.split()[0].split('-')[0].lower()
                        if first_word in STOP_WORDS_AND_DETERMINERS or first_word.isdigit():
                            continue
                        enclosing_tok = _get_enclosing_hyphenated_token(line_str, m_desc.start(), m_desc.end())
                        if _is_tracer_or_signal_identifier(desc_claimed) or _is_tracer_or_signal_identifier(enclosing_tok):
                            continue
                        desc_claimed_norm = _normalize_name(desc_claimed)
                        if desc_claimed_norm == cfg_norm or desc_claimed_norm in cfg_norm or cfg_norm in desc_claimed_norm:
                            continue
                        if first_word in gt.declared_parts or first_word in gt.declared_ast_nodes:
                            continue
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
        non_normative_depth: Optional[int] = None
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
            r'\b(\d+(?:\.\d+)?(?:\s*[-\u2013\u2014]\s*\d+(?:\.\d+)?)?)\s*(%|[a-zA-Z/][a-zA-Z0-9/%^*_-]*\b)'
        )

        current_citation: Optional[str] = None

        in_frontmatter = False

        for lineno_1idx, line in enumerate(lines, start=1):
            line_str = line.strip()

            if lineno_1idx == 1 and line_str == "---":
                in_frontmatter = True
                continue
            if in_frontmatter:
                if line_str == "---":
                    in_frontmatter = False
                continue

            if line_str.startswith("```"):
                in_code_block = not in_code_block
                continue
            if in_code_block or not line_str:
                if not line_str:
                    current_citation = None
                continue

            # Heading detection
            m_head = re.match(r'^(#{1,6})\s+(.+)$', line_str)
            if m_head:
                current_citation = None
                level = len(m_head.group(1))
                current_heading = m_head.group(2).strip()
                if non_normative_depth is not None and level <= non_normative_depth:
                    non_normative_depth = None
                if self._is_non_normative_section(current_heading):
                    non_normative_depth = level
                is_normative = (non_normative_depth is None)
                continue

            if not is_normative:
                continue

            # Track block/paragraph citation comments (e.g. <!-- Source: ... -->)
            if re.search(r'<!--\s*(?:Source|SSOT|Grounding|Reference):\s*[^>]+-->', line_str, re.I):
                current_citation = line_str
                if re.match(r'^\s*<!--\s*(?:Source|SSOT|Grounding|Reference):\s*[^>]+-->\s*$', line_str, re.I):
                    continue

            # Skip rejected trade study rows
            if re.search(r'\b(?:rejected|discarded|eliminated|not\s+selected|cons|fail|exceeds\s+limit)\b', line_str, re.I):
                continue

            # Skip lines documenting registered pending arbitration items
            if re.search(r'\bpending\s+arbitration\b', line_str, re.I):
                continue

            # Skip lines with epistemic exemptions ([TIER-3: DESIGN], [TIER-4: TBD])
            if _has_epistemic_exemption(line_str):
                continue

            line_str = _normalize_katex_math_expressions(line_str)

            # Check if line has explicit SSOT citation (inline or block)
            citation_to_check = line_str if self._extract_citation_target(line_str)[0] else current_citation
            if citation_to_check:
                candidate_tokens = self._extract_candidate_tokens(line_str)
                if self._has_ssot_citation(citation_to_check, content, rel_path, gt, candidate_tokens=candidate_tokens, lineno=lineno_1idx, claim_text=line_str):
                    continue

            # (a) Atomic Identifier Lexing: protect compound designations and citations
            protected_spans = _get_protected_spans(line_str)
            search_line = _mask_spans(line_str, protected_spans)

            line_tokens = _tokenize_identifier(line_str)
            heading_tokens = _tokenize_identifier(current_heading)
            reported_claims_on_line: Set[str] = set()

            sep_pattern = re.compile(r'[,;|]|<br\s*/?>|\.\s+|\b(?:and|or|while|whereas|with)\b', re.I)
            clause_spans: List[Tuple[int, int]] = []
            last_end = 0
            for sm in sep_pattern.finditer(line_str):
                if sm.start() > last_end:
                    clause_spans.append((last_end, sm.start()))
                last_end = sm.end()
            if last_end < len(line_str):
                clause_spans.append((last_end, len(line_str)))

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

                match_clause_idx = -1
                for idx, (c_start, c_end) in enumerate(clause_spans):
                    if c_start <= match.start() and match.end() <= c_end:
                        match_clause_idx = idx
                        break

                if match_clause_idx == -1:
                    c_start = 0
                    for sm in sep_pattern.finditer(line_str[:match.start()]):
                        c_start = sm.end()
                    c_end = len(line_str)
                    sm_end = sep_pattern.search(line_str[match.end():])
                    if sm_end:
                        c_end = match.end() + sm_end.start()
                    local_clause = line_str[c_start:c_end]
                    adjacent_clauses = [local_clause]
                else:
                    local_clause = line_str[clause_spans[match_clause_idx][0]:clause_spans[match_clause_idx][1]]
                    adjacent_clauses = [local_clause]
                    if match_clause_idx > 0:
                        adjacent_clauses.append(line_str[clause_spans[match_clause_idx - 1][0]:clause_spans[match_clause_idx - 1][1]])
                    if match_clause_idx < len(clause_spans) - 1:
                        adjacent_clauses.append(line_str[clause_spans[match_clause_idx + 1][0]:clause_spans[match_clause_idx + 1][1]])

                local_tokens = _tokenize_identifier(local_clause)

                # Skip software simulation engine, digital twin, or CI harness statements (e.g. 250 Hz digital twin simulation engine)
                if re.search(r'\b(?:simulation\s+engine|digital\s+twin|discrete\s+solver|dual-track\s+mbd)\b', local_clause, re.I):
                    continue

                candidate_metrics: List[ScopedNumericLimit] = []
                for metric in scoped_limits:
                    if not metric.unit:
                        # (c) Fail-closed: attributes without declared physical units do not match physical unit claims
                        continue
                    if metric.bound_type == "nominal":
                        continue

                    metric_canon_unit = ISO_80000_PHYSICAL_UNITS.get(metric.unit.lower(), metric.unit.lower())
                    if canon_unit != metric_canon_unit:
                        continue

                    # (d) Component-Scoped Contextual Binding
                    owner_in_clause = False
                    owner_in_line = False
                    owner_in_heading = False
                    if metric.owner:
                        owner_toks = _tokenize_identifier(metric.owner)
                        metric_owner_norm = _normalize_name(metric.owner)

                        owner_in_clause = (
                            metric_owner_norm in _normalize_name(local_clause)
                            or re.search(r'\b' + re.escape(metric.owner) + r'\b', local_clause, re.I) is not None
                            or any(t == metric.owner for t in local_tokens if t not in NON_HARDWARE_GENERIC_TOKENS)
                            or (len(owner_toks) >= 2 and all(any(_property_token_matches(ot, lt) for lt in local_tokens) for ot in owner_toks if ot not in NON_HARDWARE_GENERIC_TOKENS))
                        )

                        owner_in_line = (
                            owner_in_clause
                            or metric_owner_norm in _normalize_name(line_str)
                            or re.search(r'\b' + re.escape(metric.owner) + r'\b', line_str, re.I) is not None
                            or any(t == metric.owner for t in line_tokens if t not in NON_HARDWARE_GENERIC_TOKENS)
                            or (len(owner_toks) >= 2 and all(any(_property_token_matches(ot, lt) for lt in line_tokens) for ot in owner_toks if ot not in NON_HARDWARE_GENERIC_TOKENS))
                        )

                        owner_in_heading = (
                            metric_owner_norm in _normalize_name(current_heading)
                            or re.search(r'\b' + re.escape(metric.owner) + r'\b', current_heading, re.I) is not None
                            or any(t == metric.owner for t in heading_tokens if t not in NON_HARDWARE_GENERIC_TOKENS)
                            or (len(owner_toks) >= 2 and all(any(_property_token_matches(ot, ht) for ht in heading_tokens) for ot in owner_toks if ot not in NON_HARDWARE_GENERIC_TOKENS))
                        )

                        # Helper to check if a declared part name is an actual separate component
                        def _is_other_part(p: str, target_str: str, target_toks: List[str]) -> bool:
                            p_norm = _normalize_name(p)
                            if not p_norm or len(p_norm) < 3:
                                return False
                            if p_norm == metric_owner_norm or p_norm in metric_owner_norm or metric_owner_norm in p_norm:
                                return False
                            if any(t in owner_toks for t in _tokenize_identifier(p)):
                                return False
                            if p in NON_HARDWARE_GENERIC_TOKENS or p_norm in NON_HARDWARE_GENERIC_TOKENS:
                                return False
                            if p in target_toks or re.search(r'\b' + re.escape(p) + r'\b', target_str, re.I):
                                return True
                            if len(p) >= 5 and p_norm in _normalize_name(target_str):
                                return True
                            return False

                        other_parts_in_clause = {p for p in gt.declared_parts if _is_other_part(p, local_clause, local_tokens)}
                        if other_parts_in_clause and not owner_in_clause:
                            continue

                        other_parts_in_line = {p for p in gt.declared_parts if _is_other_part(p, line_str, line_tokens)}
                        if other_parts_in_line and not owner_in_line:
                            continue

                        strong_token_match = (
                            len(metric.meaningful_tokens) >= 2 and
                            all(any(_property_token_matches(pt, lt) for lt in line_tokens) for pt in metric.meaningful_tokens)
                        )

                        if not (owner_in_line or owner_in_heading or strong_token_match):
                            continue

                    # Property Token Specificity
                    token_matches = any(
                        any(_property_token_matches(t, lt) for lt in line_tokens)
                        for t in metric.meaningful_tokens
                    )
                    if not token_matches:
                        if metric.owner and owner_in_line:
                            token_matches = True
                        elif metric.unit == "g":
                            g_context = any(t in line_tokens for t in ("launch", "load", "accel", "acceleration", "gload", "rail", "profile", "catapult"))
                            if not g_context:
                                continue
                            token_matches = True
                        else:
                            continue

                    candidate_metrics.append(metric)

                if not candidate_metrics:
                    continue

                # Proximity Clause Matching:
                # When multiple metrics of the same unit exist on a line, match the numeric quantity
                # against the metric whose meaningful property token appears in the nearest adjacent phrase or clause
                # (e.g. Wingspan: 1.8 m matches wingspanM, not lengthM which appears later in Length: 1.6 m).
                if len(candidate_metrics) > 1:
                    distinct_tokens = {tuple(m.meaningful_tokens) for m in candidate_metrics}
                    if len(distinct_tokens) > 1:
                        def _metric_proximity_score(m: ScopedNumericLimit) -> float:
                            if not m.meaningful_tokens:
                                return 0.0
                            c_matches = sum(
                                1 for pt in m.meaningful_tokens
                                if pt in _normalize_name(local_clause) or any(_property_token_matches(pt, lt) for lt in local_tokens)
                            )
                            adj_matches = 0
                            if c_matches == 0 and len(adjacent_clauses) > 1:
                                prec_clause = adjacent_clauses[1]
                                prec_tokens = _tokenize_identifier(prec_clause)
                                adj_matches = sum(
                                    1 for pt in m.meaningful_tokens
                                    if pt in _normalize_name(prec_clause) or any(_property_token_matches(pt, lt) for lt in prec_tokens)
                                )
                            l_matches = sum(
                                1 for pt in m.meaningful_tokens
                                if pt in _normalize_name(line_str) or any(_property_token_matches(pt, lt) for lt in line_tokens)
                            )
                            if l_matches == 0:
                                return 0.0

                            c_ratio = c_matches / len(m.meaningful_tokens)
                            adj_ratio = adj_matches / len(m.meaningful_tokens)
                            l_ratio = l_matches / len(m.meaningful_tokens)

                            min_dist = float('inf')
                            preceding_bonus = 0.0
                            for m_tok in re.finditer(r'\b[a-zA-Z0-9_-]+\b', line_str):
                                tok_word = m_tok.group(0)
                                tok_tokens = _tokenize_identifier(tok_word)
                                if any(any(_property_token_matches(pt, tt) for tt in tok_tokens) for pt in m.meaningful_tokens) or \
                                   any((len(pt) >= 3 and _normalize_name(pt) in _normalize_name(tok_word)) for pt in m.meaningful_tokens):
                                    w_start, w_end = m_tok.start(), m_tok.end()
                                    is_preceding = False
                                    if w_end <= match.start():
                                        dist = float(match.start() - w_end)
                                        is_preceding = True
                                    elif w_start >= match.end():
                                        dist = float(w_start - match.end())
                                    else:
                                        dist = 0.0
                                        is_preceding = True
                                    if dist < min_dist:
                                        min_dist = dist
                                        preceding_bonus = 1.0 if is_preceding else 0.0

                            return (
                                c_matches * 10000.0 +
                                c_ratio * 5000.0 +
                                adj_matches * 100.0 +
                                adj_ratio * 50.0 +
                                l_matches * 10.0 +
                                preceding_bonus * 5.0 +
                                (1000.0 / (1.0 + min_dist))
                            )

                        best_score = max((_metric_proximity_score(m) for m in candidate_metrics), default=0.0)
                        if best_score > 0.0:
                            candidate_metrics = [m for m in candidate_metrics if _metric_proximity_score(m) >= best_score - 1e-6]
                        else:
                            candidate_metrics = []

                for metric in candidate_metrics:

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
                if metric.bound_type == "nominal":
                    continue

                # Strictly require an explicit attribute identifier match in local statement
                has_explicit_id = (
                    metric.key in _normalize_name(line_str)
                    or (len(metric.meaningful_tokens) >= 2 and all(t in line_tokens for t in metric.meaningful_tokens))
                )
                if not has_explicit_id:
                    continue

                if metric.owner:
                    owner_toks = _tokenize_identifier(metric.owner)
                    owner_in_line = (
                        re.search(r'\b' + re.escape(metric.owner) + r'\b', line_str, re.I) is not None
                        or any(t == metric.owner or t.startswith(metric.owner) for t in line_tokens if t not in NON_HARDWARE_GENERIC_TOKENS)
                    )
                    owner_in_heading = (
                        re.search(r'\b' + re.escape(metric.owner) + r'\b', current_heading, re.I) is not None
                        or any(t == metric.owner or t.startswith(metric.owner) for t in heading_tokens if t not in NON_HARDWARE_GENERIC_TOKENS)
                    )
                    if not (owner_in_line or owner_in_heading):
                        continue

                for m_num in re.finditer(r'\b(\d+(?:\.\d+)?)\b', search_line):
                    num_str = m_num.group(1)
                    if num_str in reported_claims_on_line:
                        continue

                    # Check if number is the lower bound of a dimensioned range (e.g. '49' in '49–50 V')
                    post_text_range = search_line[m_num.end():m_num.end() + 15]
                    if re.match(r'^\s*[-\u2013\u2014]\s*\d+(?:\.\d+)?\s*[a-zA-Z/%^]', post_text_range):
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
        non_normative_depth: Optional[int] = None
        is_normative = True
        in_code_block = False
        in_frontmatter = False

        current_citation: Optional[str] = None

        for lineno_1idx, line in enumerate(lines, start=1):
            line_str = line.strip()

            if lineno_1idx == 1 and line_str == "---":
                in_frontmatter = True
                continue
            if in_frontmatter:
                if line_str == "---":
                    in_frontmatter = False
                continue

            if line_str.startswith("```"):
                in_code_block = not in_code_block
                continue
            if in_code_block or not line_str:
                if not line_str:
                    current_citation = None
                continue

            # Heading detection
            m_head = re.match(r'^(#{1,6})\s+(.+)$', line_str)
            if m_head:
                current_citation = None
                level = len(m_head.group(1))
                current_heading = m_head.group(2).strip()
                if non_normative_depth is not None and level <= non_normative_depth:
                    non_normative_depth = None
                if self._is_non_normative_section(current_heading):
                    non_normative_depth = level
                is_normative = (non_normative_depth is None)
                continue

            if not is_normative:
                continue

            # Track block/paragraph citation comments (e.g. <!-- Source: ... -->)
            if re.search(r'<!--\s*(?:Source|SSOT|Grounding|Reference):\s*[^>]+-->', line_str, re.I):
                current_citation = line_str
                if re.match(r'^\s*<!--\s*(?:Source|SSOT|Grounding|Reference):\s*[^>]+-->\s*$', line_str, re.I):
                    continue

            # Skip rejected trade study rows or evaluation options
            if re.search(r'\b(?:rejected|discarded|eliminated|not\s+selected|candidate\s+option|option\s+[a-z0-9]|alternative)\b', line_str, re.I):
                continue

            # Skip lines with epistemic exemptions ([TIER-3: DESIGN], [TIER-4: TBD])
            if _has_epistemic_exemption(line_str):
                continue

            line_str = _normalize_katex_math_expressions(line_str)

            # Check if line has explicit SSOT citation (inline or block)
            citation_to_check = line_str if self._extract_citation_target(line_str)[0] else current_citation
            if citation_to_check:
                candidate_tokens = self._extract_candidate_tokens(line_str)
                if self._has_ssot_citation(citation_to_check, content, rel_path, gt, candidate_tokens=candidate_tokens, lineno=lineno_1idx, claim_text=line_str):
                    continue

            # Skip standards citations / regulatory references (e.g. "NATO STANAG 4586", "STANAG 4586 §3.2", "ISO/IEC/IEEE", "RTCA DO-178C")
            if re.search(r'\b(?:NATO|RTCA|SAE|IEEE|ISO|MIL-STD|ARINC)\s+[A-Z0-9\-_]+(?:\s+§|\s+Ed\.|\s+Rev|\s*\|)', line_str, re.I):
                continue
            if re.search(r'\b(?:normative\s+reference|standard\s+reference|reference\s+standard|compliance\s+reference|regulatory\s+reference)\b', line_str, re.I):
                continue
            # If line is in a table citing standard definitions/descriptions or research allocations
            if "|" in line_str and re.search(r'\b(?:NATO|UCS|DLI|Interoperability|Regulatory|Compliance|Guidance|IEEE|SAE|Standard Interfaces)\b', line_str, re.I):
                continue

            # Check for protocol claims in line
            reported_protocols_on_line: Set[str] = set()
            for proto in sorted(RECOGNIZED_PROTOCOLS, key=len, reverse=True):
                # Match word boundary
                pat = re.compile(r'\b' + re.escape(proto) + r'\b', re.I)
                if pat.search(line_str):
                    proto_norm = _normalize_name(proto)
                    if proto_norm in reported_protocols_on_line:
                        continue
                    if proto_norm not in gt.declared_protocols:
                        findings.append(Finding(
                            "factual-grounding-unverified-protocol",
                            f"{rel_path}:{lineno_1idx}: Ungrounded protocol claim '{proto}' is not declared in schema ground truth or substantiated by SSOT citation.",
                            location=f"{rel_path}:{lineno_1idx}",
                            detail={"file": rel_path, "line": lineno_1idx, "protocol": proto}
                        ))
                        reported_protocols_on_line.add(proto_norm)

            # Check for ungrounded execution rates (e.g. "400 Hz inner loop", "30 Hz GUI", "100 Hz outer loop")
            if not re.search(r'\b(?:simulation\s+engine|digital\s+twin|discrete\s+solver|dual-track\s+mbd|sitl\s+integrator)\b', line_str, re.I):
                m_freq = re.search(r'\b(\d+(?:\.\d+)?)\s*(Hz|kHz)\b', line_str, re.I)
                m_rate_context = re.search(
                    r'\b(?:inner\s+loop|outer\s+loop|gui|display|telemetry|control\s+loop|rate\s+loop|attitude\s+loop|servo\s+rate|refresh\s+rate|update\s+rate|execution\s+rate|pid\s+loop|task\s+rate|loop\s+rate|loop)\b',
                    line_str,
                    re.I
                )
                if m_freq and m_rate_context:
                    for m_all_freq in re.finditer(r'\b(\d+(?:\.\d+)?)\s*(Hz|kHz)\b', line_str, re.I):
                        freq_val = float(m_all_freq.group(1))
                        if m_all_freq.group(2).lower() == "khz":
                            freq_val *= 1000.0
                        freq_token = m_all_freq.group(0)
                        is_declared = self._is_frequency_declared(freq_val, freq_token, gt)
                        if not is_declared:
                            findings.append(Finding(
                                "factual-grounding-unverified-protocol",
                                f"{rel_path}:{lineno_1idx}: Ungrounded execution rate claim '{freq_token}' ({m_rate_context.group(0)}) is not declared in schema ground truth or substantiated by SSOT citation.",
                                location=f"{rel_path}:{lineno_1idx}",
                                detail={
                                    "file": rel_path,
                                    "line": lineno_1idx,
                                    "claimed": freq_token,
                                    "context": m_rate_context.group(0)
                                }
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
