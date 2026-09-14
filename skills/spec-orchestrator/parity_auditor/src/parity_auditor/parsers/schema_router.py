import os
import re
import json
import logging
import xml.etree.ElementTree as ET
from dataclasses import dataclass, field
from typing import List, Tuple, Optional, Dict, Any, Union

try:
    import yaml
except ImportError:
    yaml = None

from .base import IParser
from .regex import RegexSchemaParser
from ..core.workspace import WorkspaceRepository

logger = logging.getLogger(__name__)

IGNORED_FILENAMES = {".gitkeep", ".DS_Store", ".gitignore"}


def _is_ignored_file(filepath: str) -> bool:
    basename = os.path.basename(filepath)
    if basename in IGNORED_FILENAMES:
        return True
    if basename.startswith("."):
        return True
    return False


class DirectionStr(str):
    """Case-insensitive string representation for port directionality (e.g. in, out, inout)."""

    def __eq__(self, other: Any) -> bool:
        if isinstance(other, str):
            return self.lower() == other.lower()
        return super().__eq__(other)

    def __ne__(self, other: Any) -> bool:
        return not self.__eq__(other)

    def __hash__(self) -> int:
        return hash(self.lower())


@dataclass
class SubsystemPort:
    """Canonical model for a subsystem port / interface."""

    name: str
    direction: str = "inout"
    type_name: str = "Port"
    doc: str = ""

    def __post_init__(self) -> None:
        if not isinstance(self.direction, DirectionStr):
            self.direction = DirectionStr(self.direction)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "direction": str(self.direction),
            "type_name": self.type_name,
            "doc": self.doc,
        }


@dataclass
class SubsystemPart:
    """Canonical model for a subsystem part definition."""

    name: str
    doc: str = ""
    ports: List[SubsystemPort] = field(default_factory=list)
    mass_kg: Optional[float] = None
    power_w: Optional[float] = None
    actions: List[Any] = field(default_factory=list)
    attributes: Dict[str, Any] = field(default_factory=dict)
    constraints: List[Any] = field(default_factory=list)

    def __post_init__(self) -> None:
        normalized_ports: List[SubsystemPort] = []
        for p in self.ports:
            if isinstance(p, SubsystemPort):
                normalized_ports.append(p)
            elif isinstance(p, dict):
                normalized_ports.append(
                    SubsystemPort(
                        name=str(p.get("name") or p.get("id") or ""),
                        direction=str(p.get("direction") or p.get("dir") or "inout"),
                        type_name=str(p.get("type_name") or p.get("type") or "Port"),
                        doc=str(p.get("doc") or p.get("description") or ""),
                    )
                )
            elif isinstance(p, (list, tuple)) and len(p) >= 1:
                p_name = str(p[0])
                p_dir = str(p[1]) if len(p) > 1 else "inout"
                p_type = str(p[2]) if len(p) > 2 else "Port"
                p_doc = str(p[3]) if len(p) > 3 else ""
                normalized_ports.append(SubsystemPort(name=p_name, direction=p_dir, type_name=p_type, doc=p_doc))
        self.ports = normalized_ports

    def to_dict(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "doc": self.doc,
            "ports": [p.to_dict() for p in self.ports],
            "mass_kg": self.mass_kg,
            "power_w": self.power_w,
            "actions": list(self.actions),
            "attributes": dict(self.attributes),
            "constraints": list(self.constraints),
        }


def _parse_float(val: Any) -> Optional[float]:
    """Parse numeric float from string or number, ignoring trailing units."""
    if val is None:
        return None
    if isinstance(val, (int, float)):
        return float(val)
    if isinstance(val, str):
        m = re.search(r"[-+]?\d*\.?\d+(?:[eE][-+]?\d+)?", val.strip())
        if m:
            try:
                return float(m.group(0))
            except (ValueError, TypeError):
                return None
    return None


def _extract_preceding_doc(text: str, pos: int) -> str:
    """Extract comment lines immediately preceding a keyword match."""
    sub = text[:pos].rstrip()
    if sub.endswith("*/"):
        start_c = sub.rfind("/*")
        if start_c != -1:
            raw = sub[start_c + 2 : -2].strip()
            if raw.startswith("doc"):
                raw = raw[3:].strip()
            lines = [line.strip().lstrip("*").strip() for line in raw.splitlines()]
            return " ".join(line for line in lines if line)
    lines = sub.splitlines()
    doc_lines: List[str] = []
    for line in reversed(lines):
        s = line.strip()
        if s.startswith("//"):
            doc_lines.append(s[2:].strip())
        else:
            break
    doc_lines.reverse()
    return " ".join(doc_lines)


def _strip_nested_part_defs(body: str) -> str:
    """Strip nested part def blocks to avoid assigning child ports to parent parts."""
    pattern = re.compile(r"part\s+(?:def\s+)?([a-zA-Z0-9_]+)\s*\{")
    pos = 0
    res: List[str] = []
    last_end = 0
    while pos < len(body):
        m = pattern.search(body, pos)
        if not m:
            res.append(body[last_end:])
            break
        res.append(body[last_end : m.start()])
        start = m.end()
        depth = 1
        i = start
        while i < len(body) and depth > 0:
            if body[i] == "{":
                depth += 1
            elif body[i] == "}":
                depth -= 1
            i += 1
        last_end = i
        pos = i
    return "".join(res)


def _extract_from_sysml(content: str) -> List[SubsystemPart]:
    """Extract SubsystemPart objects from SysML v2 source."""
    parts: List[SubsystemPart] = []
    pattern = re.compile(r"(?:(?:doc\s*/\*|\/\*)\s*(.*?)\*\/\s*)?part\s+(?:def\s+)?([a-zA-Z0-9_]+)\s*\{")
    pos = 0
    seen_names = set()

    while pos < len(content):
        m = pattern.search(content, pos)
        if not m:
            break
        pre_doc = (m.group(1) or "").strip()
        if pre_doc.startswith("doc"):
            pre_doc = pre_doc[3:].strip()
        if not pre_doc:
            pre_doc = _extract_preceding_doc(content, m.start())
        name = m.group(2)
        start = m.end()
        depth = 1
        i = start
        while i < len(content) and depth > 0:
            if content[i] == "{":
                depth += 1
            elif content[i] == "}":
                depth -= 1
            i += 1
        raw_body = content[start : i - 1]
        pos = m.end()

        # Check for doc inside body
        inner_doc_m = re.search(r"(?:doc\s*/\*|\/\*)\s*(.*?)\*\/", raw_body, re.DOTALL)
        doc = ""
        if inner_doc_m:
            raw_id = inner_doc_m.group(1).strip()
            if raw_id.startswith("doc"):
                raw_id = raw_id[3:].strip()
            doc = raw_id
        if not doc:
            doc = pre_doc

        clean_body = _strip_nested_part_defs(raw_body)

        # Ports
        ports: List[SubsystemPort] = []
        port_pat = re.compile(
            r"(?:(?:doc\s*/\*|\/\*)\s*(.*?)\*\/\s*)?(?:(in|out|inout)\s+)?port\s+(?:def\s+)?([a-zA-Z0-9_]+)(?:\s*:\s*([a-zA-Z0-9_]+))?",
            re.DOTALL,
        )
        for pm in port_pat.finditer(clean_body):
            p_doc = (pm.group(1) or "").strip()
            if p_doc.startswith("doc"):
                p_doc = p_doc[3:].strip()
            p_dir = pm.group(2) or "inout"
            p_name = pm.group(3)
            p_type = pm.group(4) or "Port"
            ports.append(SubsystemPort(name=p_name, direction=DirectionStr(p_dir), type_name=p_type, doc=p_doc))

        # Attributes
        attributes: Dict[str, Any] = {}
        attr_pat = re.compile(r"\b(?:attribute\s+)?([a-zA-Z0-9_]+)\s*(?::\s*([a-zA-Z0-9_]+))?\s*=\s*([^;]+);")
        for am in attr_pat.finditer(clean_body):
            attr_name = am.group(1)
            attr_val = am.group(3).strip().strip("\"'")
            attributes[attr_name] = attr_val

        mass_kg: Optional[float] = None
        for k, v in attributes.items():
            if k.lower() in ("mass_kg", "mass", "weight", "weight_kg"):
                mass_kg = _parse_float(v)
                break
        if mass_kg is None:
            m_match = re.search(r"\b(?:attribute\s+)?mass(?:_kg)?\s*(?::\s*[^=;]+)?=\s*([0-9\.]+)", clean_body)
            if m_match:
                mass_kg = float(m_match.group(1))

        power_w: Optional[float] = None
        for k, v in attributes.items():
            if k.lower() in ("power_w", "power", "wattage"):
                power_w = _parse_float(v)
                break
        if power_w is None:
            p_match = re.search(r"\b(?:attribute\s+)?power(?:_w)?\s*(?::\s*[^=;]+)?=\s*([0-9\.]+)", clean_body)
            if p_match:
                power_w = float(p_match.group(1))

        actions = re.findall(r"\b(?:action\s+(?:def\s+)?|perform\s+)([a-zA-Z0-9_]+)", clean_body)
        constraints = re.findall(r"\b(?:assert\s+constraint|constraint\s+(?:def\s+)?)([a-zA-Z0-9_]+)", clean_body)

        parts.append(
            SubsystemPart(
                name=name,
                doc=doc,
                ports=ports,
                mass_kg=mass_kg,
                power_w=power_w,
                actions=actions,
                attributes=attributes,
                constraints=constraints,
            )
        )
        seen_names.add(name)

    # Statement-style part def <name>;
    stmt_pat = re.compile(r"(?:(?:doc\s*/\*|\/\*)\s*(.*?)\*\/\s*)?part\s+(?:def\s+)?([a-zA-Z0-9_]+)\s*;")
    for sm in stmt_pat.finditer(content):
        s_name = sm.group(2)
        if s_name not in seen_names:
            s_doc = (sm.group(1) or "").strip()
            if s_doc.startswith("doc"):
                s_doc = s_doc[3:].strip()
            if not s_doc:
                s_doc = _extract_preceding_doc(content, sm.start())
            parts.append(SubsystemPart(name=s_name, doc=s_doc))
            seen_names.add(s_name)

    return parts


def _parse_subsystem_dict(d: Dict[str, Any], default_name: str = "") -> Optional[SubsystemPart]:
    """Parse a single dictionary into a SubsystemPart."""
    name = str(d.get("name") or d.get("id") or d.get("subsystem") or d.get("component") or default_name or "").strip()
    if not name:
        return None
    doc = str(d.get("doc") or d.get("description") or d.get("desc") or d.get("summary") or "").strip()
    mass_val = d.get("mass_kg") if d.get("mass_kg") is not None else d.get("mass")
    if mass_val is None:
        mass_val = d.get("weight_kg") if d.get("weight_kg") is not None else d.get("weight")
    mass_kg = _parse_float(mass_val)

    power_val = d.get("power_w") if d.get("power_w") is not None else d.get("power")
    if power_val is None:
        power_val = d.get("wattage")
    power_w = _parse_float(power_val)

    ports: List[SubsystemPort] = []
    raw_ports = d.get("ports") or d.get("interfaces") or []
    if isinstance(raw_ports, list):
        for p in raw_ports:
            if isinstance(p, dict):
                p_name = str(p.get("name") or p.get("id") or "").strip()
                if not p_name:
                    continue
                p_dir = str(p.get("direction") or p.get("dir") or p.get("flow") or "inout").strip()
                p_type = str(p.get("type_name") or p.get("type") or p.get("data_type") or "Port").strip()
                p_doc = str(p.get("doc") or p.get("description") or "").strip()
                ports.append(SubsystemPort(name=p_name, direction=DirectionStr(p_dir), type_name=p_type, doc=p_doc))
            elif isinstance(p, str):
                p_clean = p.strip()
                if not p_clean:
                    continue
                dir_m = re.search(r"\((in|out|inout|in/out)\)", p_clean, re.IGNORECASE)
                if dir_m:
                    p_dir = "inout" if dir_m.group(1).lower() in ("inout", "in/out") else dir_m.group(1).lower()
                    p_clean = p_clean[: dir_m.start()] + p_clean[dir_m.end() :]
                else:
                    p_dir = "inout"
                if ":" in p_clean:
                    pn, pt = p_clean.split(":", 1)
                    p_name = pn.strip()
                    p_type = pt.strip()
                else:
                    p_name = p_clean.strip()
                    p_type = "Port"
                ports.append(SubsystemPort(name=p_name, direction=DirectionStr(p_dir), type_name=p_type))
    elif isinstance(raw_ports, dict):
        for pk, pv in raw_ports.items():
            pk_name = str(pk).strip()
            if isinstance(pv, dict):
                p_dir = str(pv.get("direction") or pv.get("dir") or "inout").strip()
                p_type = str(pv.get("type_name") or pv.get("type") or "Port").strip()
                p_doc = str(pv.get("doc") or pv.get("description") or "").strip()
                ports.append(SubsystemPort(name=pk_name, direction=DirectionStr(p_dir), type_name=p_type, doc=p_doc))
            elif isinstance(pv, str):
                pv_clean = pv.strip()
                if pv_clean.lower() in ("in", "out", "inout"):
                    ports.append(SubsystemPort(name=pk_name, direction=DirectionStr(pv_clean), type_name="Port"))
                else:
                    ports.append(SubsystemPort(name=pk_name, direction=DirectionStr("inout"), type_name=pv_clean))

    actions = list(d.get("actions") or d.get("operations") or d.get("methods") or [])
    attributes = dict(d.get("attributes") or d.get("properties") or {})
    constraints = list(d.get("constraints") or d.get("invariants") or [])

    return SubsystemPart(
        name=name,
        doc=doc,
        ports=ports,
        mass_kg=mass_kg,
        power_w=power_w,
        actions=actions,
        attributes=attributes,
        constraints=constraints,
    )


def _extract_from_dict_or_list(data: Any) -> List[SubsystemPart]:
    """Traverse JSON/YAML structured data to extract SubsystemPart collections."""
    parts: List[SubsystemPart] = []
    if isinstance(data, list):
        for item in data:
            if isinstance(item, dict):
                p = _parse_subsystem_dict(item)
                if p:
                    parts.append(p)
    elif isinstance(data, dict):
        for key in ("subsystems", "components", "parts", "modules", "lrus", "nodes"):
            if key in data:
                val = data[key]
                if isinstance(val, list):
                    for item in val:
                        if isinstance(item, dict):
                            p = _parse_subsystem_dict(item)
                            if p:
                                parts.append(p)
                elif isinstance(val, dict):
                    for k, v in val.items():
                        if isinstance(v, dict):
                            p = _parse_subsystem_dict(v, default_name=k)
                            if p:
                                parts.append(p)
        if not parts:
            p = _parse_subsystem_dict(data)
            if p and (p.mass_kg is not None or p.power_w is not None or p.ports or p.actions or p.doc):
                parts.append(p)
            else:
                for k, v in data.items():
                    if isinstance(v, dict) and any(
                        x in v for x in ("ports", "interfaces", "mass", "mass_kg", "power", "power_w", "actions")
                    ):
                        p = _parse_subsystem_dict(v, default_name=k)
                        if p:
                            parts.append(p)
    return parts


def _extract_from_yaml(content: str) -> List[SubsystemPart]:
    """Extract SubsystemPart objects from YAML content."""
    if yaml is not None:
        try:
            data = yaml.safe_load(content)
            return _extract_from_dict_or_list(data)
        except Exception:
            pass
    try:
        data = json.loads(content)
        return _extract_from_dict_or_list(data)
    except Exception:
        return []


def _extract_from_json(content: str) -> List[SubsystemPart]:
    """Extract SubsystemPart objects from JSON content."""
    try:
        data = json.loads(content)
        return _extract_from_dict_or_list(data)
    except Exception as exc:
        logger.debug("Failed to parse JSON content: %s", exc)
        return []


def _extract_from_proto(text: str) -> List[SubsystemPart]:
    """Extract SubsystemPart objects from Protobuf messages and services."""
    parts: List[SubsystemPart] = []

    # Parse messages
    msg_pat = re.compile(r"\bmessage\s+([a-zA-Z0-9_]+)\s*\{", re.DOTALL)
    for m in msg_pat.finditer(text):
        name = m.group(1)
        doc = _extract_preceding_doc(text, m.start())
        start = m.end()
        depth = 1
        i = start
        while i < len(text) and depth > 0:
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
            i += 1
        body = text[start : i - 1]
        ports: List[SubsystemPort] = []
        attributes: Dict[str, Any] = {}
        field_pat = re.compile(
            r"\s*(?:repeated|optional|required)?\s*([a-zA-Z0-9_\.]+)\s+([a-zA-Z0-9_]+)\s*=\s*\d+\s*;", re.DOTALL
        )
        for fm in field_pat.finditer(body):
            f_doc = _extract_preceding_doc(body, fm.start())
            f_type = fm.group(1)
            f_name = fm.group(2)
            attributes[f_name] = f_type
            ports.append(SubsystemPort(name=f_name, direction=DirectionStr("inout"), type_name=f_type, doc=f_doc))

        mass_kg: Optional[float] = None
        m_match = re.search(r"mass(?:_kg)?\s*[:=]\s*([0-9\.]+)", doc, re.IGNORECASE)
        if m_match:
            mass_kg = float(m_match.group(1))

        power_w: Optional[float] = None
        p_match = re.search(r"power(?:_w)?\s*[:=]\s*([0-9\.]+)", doc, re.IGNORECASE)
        if p_match:
            power_w = float(p_match.group(1))

        parts.append(
            SubsystemPart(
                name=name, doc=doc, ports=ports, mass_kg=mass_kg, power_w=power_w, attributes=attributes
            )
        )

    # Parse services
    srv_pat = re.compile(r"\bservice\s+([a-zA-Z0-9_]+)\s*\{", re.DOTALL)
    for m in srv_pat.finditer(text):
        name = m.group(1)
        doc = _extract_preceding_doc(text, m.start())
        start = m.end()
        depth = 1
        i = start
        while i < len(text) and depth > 0:
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
            i += 1
        body = text[start : i - 1]
        ports = []
        actions = []
        rpc_pat = re.compile(
            r"\brpc\s+([a-zA-Z0-9_]+)\s*\(\s*(?:stream\s+)?([a-zA-Z0-9_\.]+)\s*\)\s*returns\s*\(\s*(?:stream\s+)?([a-zA-Z0-9_\.]+)\s*\)\s*;",
            re.DOTALL,
        )
        for rm in rpc_pat.finditer(body):
            r_doc = _extract_preceding_doc(body, rm.start())
            r_name = rm.group(1)
            in_t = rm.group(2)
            out_t = rm.group(3)
            actions.append(r_name)
            ports.append(
                SubsystemPort(name=r_name, direction=DirectionStr("inout"), type_name=f"{in_t}->{out_t}", doc=r_doc)
            )
        parts.append(SubsystemPart(name=name, doc=doc, ports=ports, actions=actions))

    return parts


def _extract_from_idl(text: str) -> List[SubsystemPart]:
    """Extract SubsystemPart objects from OMG IDL interfaces and component declarations."""
    parts: List[SubsystemPart] = []

    # Interfaces
    if_pat = re.compile(r"\binterface\s+([a-zA-Z0-9_]+)\s*\{", re.DOTALL)
    for m in if_pat.finditer(text):
        name = m.group(1)
        doc = _extract_preceding_doc(text, m.start())
        start = m.end()
        depth = 1
        i = start
        while i < len(text) and depth > 0:
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
            i += 1
        body = text[start : i - 1]
        ports: List[SubsystemPort] = []
        actions: List[str] = []
        attributes: Dict[str, Any] = {}

        # Operations
        op_pat = re.compile(r"\b([a-zA-Z0-9_:\<\>]+)\s+([a-zA-Z0-9_]+)\s*\(([^)]*)\)\s*;", re.DOTALL)
        for om in op_pat.finditer(body):
            ret_t = om.group(1)
            if ret_t in ("attribute", "readonly"):
                continue
            op_name = om.group(2)
            o_doc = _extract_preceding_doc(body, om.start())
            actions.append(op_name)
            ports.append(SubsystemPort(name=op_name, direction=DirectionStr("inout"), type_name=ret_t, doc=o_doc))

        # Attributes
        attr_pat = re.compile(r"\b(?:readonly\s+)?attribute\s+([a-zA-Z0-9_:\<\>]+)\s+([a-zA-Z0-9_]+)\s*;")
        for am in attr_pat.finditer(body):
            attributes[am.group(2)] = am.group(1)

        mass_kg = _parse_float(attributes.get("mass_kg") or attributes.get("mass"))
        power_w = _parse_float(attributes.get("power_w") or attributes.get("power"))

        parts.append(
            SubsystemPart(
                name=name,
                doc=doc,
                ports=ports,
                mass_kg=mass_kg,
                power_w=power_w,
                actions=actions,
                attributes=attributes,
            )
        )

    # Components
    comp_pat = re.compile(r"\bcomponent\s+([a-zA-Z0-9_]+)\s*\{", re.DOTALL)
    for m in comp_pat.finditer(text):
        name = m.group(1)
        doc = _extract_preceding_doc(text, m.start())
        start = m.end()
        depth = 1
        i = start
        while i < len(text) and depth > 0:
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
            i += 1
        body = text[start : i - 1]
        ports = []
        prov_pat = re.compile(r"\bprovides\s+([a-zA-Z0-9_:\<\>]+)\s+([a-zA-Z0-9_]+)\s*;")
        for pm in prov_pat.finditer(body):
            ports.append(SubsystemPort(name=pm.group(2), direction=DirectionStr("in"), type_name=pm.group(1)))
        uses_pat = re.compile(r"\buses\s+([a-zA-Z0-9_:\<\>]+)\s+([a-zA-Z0-9_]+)\s*;")
        for um in uses_pat.finditer(body):
            ports.append(SubsystemPort(name=um.group(2), direction=DirectionStr("out"), type_name=um.group(1)))
        port_pat = re.compile(r"\b(in|out|inout)?\s*port\s+([a-zA-Z0-9_:\<\>]+)\s+([a-zA-Z0-9_]+)\s*;")
        for ppm in port_pat.finditer(body):
            ports.append(
                SubsystemPort(
                    name=ppm.group(3),
                    direction=DirectionStr(ppm.group(1) or "inout"),
                    type_name=ppm.group(2),
                )
            )
        parts.append(SubsystemPart(name=name, doc=doc, ports=ports))

    return parts


def _extract_from_arxml(content: str) -> List[SubsystemPart]:
    """Extract SubsystemPart objects from AUTOSAR XML SW component definitions."""
    try:
        root = ET.fromstring(content)
    except Exception as exc:
        logger.debug("Failed to parse ARXML XML: %s", exc)
        return []

    def strip_ns(tag: str) -> str:
        return tag.split("}", 1)[1] if "}" in tag else tag

    parts: List[SubsystemPart] = []
    for elem in root.iter():
        tag = strip_ns(elem.tag)
        if tag.endswith("-SW-COMPONENT-TYPE") or tag == "SW-COMPONENT-PROTOTYPE":
            name = ""
            doc = ""
            ports: List[SubsystemPort] = []
            actions: List[str] = []
            mass_kg: Optional[float] = None
            power_w: Optional[float] = None

            for child in elem:
                c_tag = strip_ns(child.tag)
                if c_tag == "SHORT-NAME" and child.text:
                    name = child.text.strip()
                elif c_tag == "DESC":
                    doc = "".join(child.itertext()).strip()
                elif "MASS" in c_tag and child.text:
                    mass_kg = _parse_float(child.text.strip())
                elif "POWER" in c_tag and child.text:
                    power_w = _parse_float(child.text.strip())
                elif c_tag == "PORTS":
                    for port_elem in child:
                        p_tag = strip_ns(port_elem.tag)
                        if p_tag == "P-PORT-PROTOTYPE":
                            direction = "out"
                        elif p_tag == "R-PORT-PROTOTYPE":
                            direction = "in"
                        elif p_tag == "PR-PORT-PROTOTYPE":
                            direction = "inout"
                        else:
                            direction = "inout"

                        p_name = ""
                        p_type = "Port"
                        p_doc = ""
                        for p_child in port_elem:
                            pctag = strip_ns(p_child.tag)
                            if pctag == "SHORT-NAME" and p_child.text:
                                p_name = p_child.text.strip()
                            elif "INTERFACE" in pctag and p_child.text:
                                p_type = p_child.text.strip().rstrip("/").split("/")[-1]
                            elif pctag == "DESC":
                                p_doc = "".join(p_child.itertext()).strip()
                        if p_name:
                            ports.append(
                                SubsystemPort(
                                    name=p_name,
                                    direction=DirectionStr(direction),
                                    type_name=p_type,
                                    doc=p_doc,
                                )
                            )
                elif c_tag == "INTERNAL-BEHAVIORS":
                    for ib_elem in child.iter():
                        if strip_ns(ib_elem.tag) == "RUNNABLE-ENTITY":
                            for r_child in ib_elem:
                                if strip_ns(r_child.tag) == "SHORT-NAME" and r_child.text:
                                    actions.append(r_child.text.strip())
            if name:
                parts.append(
                    SubsystemPart(
                        name=name,
                        doc=doc,
                        ports=ports,
                        mass_kg=mass_kg,
                        power_w=power_w,
                        actions=actions,
                    )
                )
    return parts


def _extract_from_markdown(text: str) -> List[SubsystemPart]:
    """Extract SubsystemPart objects from Markdown tables declaring subsystems and ports."""
    def is_port_col(h: str) -> bool:
        return any(x in h for x in ("port", "interface", "signal")) or h in ("io", "inputs", "outputs", "inout")

    parts: List[SubsystemPart] = []
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i].strip()
        if line.startswith("|") and line.endswith("|"):
            headers = [c.strip() for c in line.split("|")[1:-1]]
            h_lowers = [re.sub(r"[^a-zA-Z0-9]", "", h.lower()) for h in headers]
            name_idx = None
            for idx, h in enumerate(h_lowers):
                if any(x in h for x in ("subsystem", "component", "part", "module", "lru", "unit")) or h == "name":
                    name_idx = idx
                    break
            if name_idx is not None and i + 1 < len(lines):
                sep_line = lines[i + 1].strip()
                if sep_line.startswith("|") and "-" in sep_line:
                    desc_idx = next(
                        (
                            idx
                            for idx, h in enumerate(h_lowers)
                            if any(x in h for x in ("desc", "purpose", "scope", "function", "role", "doc", "summary"))
                        ),
                        None,
                    )
                    mass_idx = next(
                        (idx for idx, h in enumerate(h_lowers) if any(x in h for x in ("mass", "weight"))), None
                    )
                    power_idx = next(
                        (idx for idx, h in enumerate(h_lowers) if any(x in h for x in ("power", "watt"))), None
                    )
                    ports_idx = next((idx for idx, h in enumerate(h_lowers) if is_port_col(h)), None)

                    j = i + 2
                    while j < len(lines):
                        row_line = lines[j].strip()
                        if not (row_line.startswith("|") and row_line.endswith("|")):
                            break
                        cols = [c.strip() for c in row_line.split("|")[1:-1]]
                        if name_idx < len(cols):
                            raw_name = cols[name_idx]
                            clean_name = re.sub(r"\[([^\]]+)\]\([^\)]+\)", r"\1", raw_name)
                            clean_name = re.sub(r"[*_`]", "", clean_name).strip()
                            if clean_name and not clean_name.startswith("---"):
                                doc = cols[desc_idx] if desc_idx is not None and desc_idx < len(cols) else ""
                                doc = re.sub(r"\[([^\]]+)\]\([^\)]+\)", r"\1", doc)
                                doc = re.sub(r"[*_`]", "", doc).strip()

                                mass_val = None
                                if mass_idx is not None and mass_idx < len(cols):
                                    mass_val = _parse_float(cols[mass_idx])

                                power_val = None
                                if power_idx is not None and power_idx < len(cols):
                                    power_val = _parse_float(cols[power_idx])

                                ports: List[SubsystemPort] = []
                                if ports_idx is not None and ports_idx < len(cols):
                                    raw_ports = cols[ports_idx]
                                    raw_port_items = re.split(r"(?:<br\s*/?>|[,;\n])", raw_ports)
                                    for pi in raw_port_items:
                                        p_clean = re.sub(r"[•\*\-`]", "", pi).strip()
                                        if not p_clean:
                                            continue
                                        dir_m = re.search(r"\((in|out|inout|in/out)\)", p_clean, re.IGNORECASE)
                                        if dir_m:
                                            p_dir = (
                                                "inout"
                                                if dir_m.group(1).lower() in ("inout", "in/out")
                                                else dir_m.group(1).lower()
                                            )
                                            p_clean = p_clean[: dir_m.start()] + p_clean[dir_m.end() :]
                                        else:
                                            colon_dir_m = re.search(r":\s*(in|out|inout)\b", p_clean, re.IGNORECASE)
                                            if colon_dir_m:
                                                p_dir = colon_dir_m.group(1).lower()
                                                p_clean = p_clean[: colon_dir_m.start()]
                                            else:
                                                p_dir = "inout"
                                        if ":" in p_clean:
                                            pn, pt = p_clean.split(":", 1)
                                            p_name = pn.strip()
                                            p_type = pt.strip()
                                        else:
                                            p_name = p_clean.strip()
                                            p_type = "Port"
                                        p_name = re.sub(r"[^a-zA-Z0-9_]", "", p_name)
                                        if p_name:
                                            ports.append(
                                                SubsystemPort(
                                                    name=p_name,
                                                    direction=DirectionStr(p_dir),
                                                    type_name=p_type,
                                                )
                                            )
                                parts.append(
                                    SubsystemPart(
                                        name=clean_name,
                                        doc=doc,
                                        ports=ports,
                                        mass_kg=mass_val,
                                        power_w=power_val,
                                    )
                                )
                        j += 1
                    i = j
                    continue
        i += 1
    return parts


def _extract_from_content(content: str, ext: str = "", filepath: str = "") -> List[SubsystemPart]:
    """Route textual schema content to the appropriate format extractor."""
    content_stripped = content.strip()
    if not content_stripped:
        return []

    ext_clean = ext.lower()
    if ext_clean in (".sysml", ".kerml"):
        return _extract_from_sysml(content)
    elif ext_clean in (".yaml", ".yml"):
        return _extract_from_yaml(content)
    elif ext_clean == ".json":
        return _extract_from_json(content)
    elif ext_clean == ".proto":
        return _extract_from_proto(content)
    elif ext_clean == ".idl":
        return _extract_from_idl(content)
    elif ext_clean == ".arxml":
        return _extract_from_arxml(content)
    elif ext_clean in (".md", ".markdown"):
        return _extract_from_markdown(content)

    # Content-based heuristic detection when extension is missing or generic
    if content_stripped.startswith("<?xml") or "<AUTOSAR" in content_stripped or "<AR-PACKAGE" in content_stripped:
        return _extract_from_arxml(content)
    if 'syntax = "proto' in content_stripped or 'syntax="proto' in content_stripped or re.search(r"\b(?:message|service)\s+[a-zA-Z0-9_]+\s*\{", content_stripped):
        return _extract_from_proto(content)
    if re.search(r"\bmodule\s+[a-zA-Z0-9_]+\s*\{", content_stripped) and re.search(r"\b(?:interface|component)\s+[a-zA-Z0-9_]+", content_stripped):
        return _extract_from_idl(content)
    if re.search(r"\bpart\s+(?:def\s+)?[a-zA-Z0-9_]+\s*\{", content_stripped):
        return _extract_from_sysml(content)
    if content_stripped.startswith("{") or content_stripped.startswith("["):
        res = _extract_from_json(content)
        if res:
            return res
    if "|" in content_stripped and any(l.strip().startswith("|") for l in content_stripped.splitlines()):
        res = _extract_from_markdown(content)
        if res:
            return res

    # Fallback to YAML/JSON traversal
    return _extract_from_yaml(content)


def extract_subsystem_parts(target: Union[str, List[str]]) -> List[SubsystemPart]:
    """Accepts a filepath, list of filepaths, or search directory and extracts all valid SubsystemPart objects."""
    if isinstance(target, (list, tuple, set)):
        results: List[SubsystemPart] = []
        for item in target:
            results.extend(extract_subsystem_parts(item))
        return results

    if not isinstance(target, str):
        return []

    target_str = target.strip()
    if not target_str:
        return []

    # If target is a directory on disk
    if os.path.isdir(target):
        supported_exts = {".sysml", ".yaml", ".yml", ".json", ".proto", ".idl", ".arxml", ".md", ".markdown"}
        subsystem_files: List[str] = []
        for root, dirs, files in os.walk(target):
            dirs[:] = [d for d in dirs if not d.startswith(".") and d not in ("node_modules", "__pycache__")]
            for f in files:
                if _is_ignored_file(f):
                    continue
                ext = os.path.splitext(f)[1].lower()
                if ext in supported_exts:
                    subsystem_files.append(os.path.join(root, f))
        subsystem_files.sort()
        results = []
        for filepath in subsystem_files:
            results.extend(extract_subsystem_parts(filepath))
        return results

    # If target is a file on disk
    if os.path.isfile(target):
        if _is_ignored_file(target):
            return []
        try:
            with open(target, "r", encoding="utf-8", errors="replace") as f:
                content = f.read()
        except Exception as exc:
            logger.warning("Failed to read schema file '%s': %s", target, exc)
            return []
        ext = os.path.splitext(target)[1].lower()
        return _extract_from_content(content, ext=ext, filepath=target)

    # If target is raw text content
    if "\n" in target or target_str.startswith(
        ("{", "[", "<?xml", "<AUTOSAR", "package", "part ", "module", "syntax", "#", "|")
    ):
        return _extract_from_content(target, ext="", filepath="")

    return []


class SchemaRouter(IParser):
    def __init__(self, workspace_repo: WorkspaceRepository, parsers: Optional[List[IParser]] = None):
        self.workspace_repo = workspace_repo
        if parsers is not None:
            self._parsers: List[IParser] = list(parsers)
        else:
            self._parsers: List[IParser] = [RegexSchemaParser(workspace_repo)]

    def register(self, parser: IParser, prepend: bool = False):
        if prepend:
            self._parsers.insert(0, parser)
        else:
            self._parsers.append(parser)

    def can_parse(self, filepath: str) -> bool:
        return any(parser.can_parse(filepath) for parser in self._parsers)

    def parse(self, filepath: str) -> Tuple[Optional[str], Dict[str, str]]:
        for parser in self._parsers:
            if parser.can_parse(filepath):
                return parser.parse(filepath)
        if _is_ignored_file(filepath):
            return None, {}
        ext = os.path.splitext(filepath)[1].lower()
        logger.warning(
            "Extensible schema parser not yet implemented for extension '%s' in %s",
            ext,
            os.path.basename(filepath),
        )
        return os.path.basename(filepath), {}

    def extract_subsystem_parts(self, target: Union[str, List[str]]) -> List[SubsystemPart]:
        """Expose extract_subsystem_parts as a SchemaRouter instance method."""
        return extract_subsystem_parts(target)


def parse_schema_file(
    filepath: str,
    repo: Optional[WorkspaceRepository] = None,
    router: Optional[SchemaRouter] = None,
) -> Tuple[Optional[str], Dict[str, str]]:
    if router is not None:
        return router.parse(filepath)
    if repo is None:
        repo = WorkspaceRepository()
    router = SchemaRouter(repo)
    return router.parse(filepath)


__all__ = [
    "SchemaRouter",
    "SubsystemPort",
    "SubsystemPart",
    "extract_subsystem_parts",
    "parse_schema_file",
]
