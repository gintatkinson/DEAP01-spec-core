#!/usr/bin/env python3
"""
SysML v2 Abstract Syntax Tree (AST) Data Models & Parser

Provides Canonical SysML v2 AST elements:
- AttributeDef: Defines attributes (data elements / primitive types)
- PortDef: Defines ports (flow / interaction interfaces)
- ActionDef: Defines actions / methods
- SysMLOperationDef: Defines operations with typed signatures and direction
- SysMLCapabilityDef: Defines system/subsystem capability specifications
- SysMLInteractionDef: Defines interaction sequences (lifelines, messages, triggers)
- SysMLConstraintDef: Defines invariants and assertions (assert constraint / constraint def)
- SysMLTestCaseDef: Defines test case definitions with subject and verification links
- RequirementDef / SysMLRequirementDef: Defines requirement specifications
- StateDef / SysMLStateDef: Defines statechart / state machine definitions
- UseCaseDef / SysMLUseCaseDef: Defines formal use case definitions
- ItemDef / SysMLItemDef: Defines data item / payload definitions
- PartDef / SysMLPart: Defines structural components (parts / blocks)
- SysMLPackage: Top-level or nested SysML v2 package container
- SysMLParser: Textual SysML v2 parser into canonical AST
"""

import os
import re
from dataclasses import dataclass, field
from typing import List, Optional, Dict, Any, Union


@dataclass
class AttributeDef:
    name: str
    type_name: str = "String"
    doc: str = ""
    default_value: Optional[str] = None

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        doc_str = f"{pad}doc /* {self.doc} */\n" if self.doc else ""
        def_str = f" = {self.default_value}" if self.default_value is not None else ""
        return f"{doc_str}{pad}attribute {self.name} : {self.type_name}{def_str};"


@dataclass
class PortDef:
    name: str
    type_name: str = "Port"
    direction: str = "inout"
    doc: str = ""

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        doc_str = f"{pad}doc /* {self.doc} */\n" if self.doc else ""
        dir_prefix = f"{self.direction} " if self.direction and self.direction != "inout" else ""
        return f"{doc_str}{pad}{dir_prefix}port {self.name} : {self.type_name};"


@dataclass
class ActionDef:
    name: str
    doc: str = ""
    in_params: List[AttributeDef] = field(default_factory=list)
    out_params: List[AttributeDef] = field(default_factory=list)

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        doc_str = f"{pad}doc /* {self.doc} */\n" if self.doc else ""
        all_params = []
        for p in (self.in_params or []):
            all_params.append(f"in {p.name} : {p.type_name}")
        for p in (self.out_params or []):
            all_params.append(f"out {p.name} : {p.type_name}")
        params_str = f"({', '.join(all_params)})" if all_params else ""
        return f"{doc_str}{pad}action {self.name}{params_str};"


@dataclass
class SysMLOperationDef:
    name: str
    direction: str = "inout"
    param_type: str = "String"
    return_type: Optional[str] = None
    doc: str = ""
    parameters: List[AttributeDef] = field(default_factory=list)

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        doc_str = f"{pad}doc /* {self.doc} */\n" if self.doc else ""
        if self.parameters:
            param_strs = []
            for p in self.parameters:
                p_dir = getattr(p, "default_value", None) or self.direction
                if p_dir not in ("in", "out", "inout"):
                    p_dir = "in"
                param_strs.append(f"{p_dir} {p.name} : {p.type_name}")
            params_header = f"({', '.join(param_strs)})"
        elif self.param_type and self.param_type != "None":
            params_header = f"({self.direction} param : {self.param_type})"
        else:
            params_header = "()"
        ret_str = f" : {self.return_type}" if self.return_type else ""
        return f"{doc_str}{pad}operation {self.name}{params_header}{ret_str};"


@dataclass
class SysMLCapabilityDef:
    name: str
    description: str = ""
    subsystem: str = ""
    package_ref: str = ""
    doc: str = ""

    def __post_init__(self):
        if not self.description and self.doc:
            self.description = self.doc
        elif not self.doc and self.description:
            self.doc = self.description
        if not self.package_ref and self.subsystem:
            self.package_ref = self.subsystem
        elif not self.subsystem and self.package_ref:
            self.subsystem = self.package_ref

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        lines = []
        doc_val = self.doc or self.description
        if doc_val:
            lines.append(f"{pad}doc /* {doc_val} */")
        lines.append(f"{pad}capability def {self.name} {{")
        subsys = self.subsystem or self.package_ref
        if subsys:
            lines.append(f"{pad}    subsystem {subsys};")
        lines.append(f"{pad}}}")
        return "\n".join(lines)


@dataclass
class SysMLInteractionDef:
    name: str
    lifelines: List[str] = field(default_factory=list)
    messages: List[str] = field(default_factory=list)
    triggers: List[str] = field(default_factory=list)
    doc: str = ""

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        lines = []
        if self.doc:
            lines.append(f"{pad}doc /* {self.doc} */")
        lines.append(f"{pad}interaction {self.name} {{")
        for ll in (self.lifelines or []):
            lines.append(f"{pad}    lifeline {ll};")
        for msg in (self.messages or []):
            lines.append(f"{pad}    message {msg};")
        for trg in (self.triggers or []):
            lines.append(f"{pad}    trigger {trg};")
        lines.append(f"{pad}}}")
        return "\n".join(lines)


@dataclass
class SysMLConstraintDef:
    name: str
    expression: str = ""
    parameters: List[str] = field(default_factory=list)
    is_assertion: bool = False
    doc: str = ""

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        lines = []
        if self.doc:
            lines.append(f"{pad}doc /* {self.doc} */")
        kw = "assert constraint" if self.is_assertion else "constraint def"
        params_str = f"({', '.join(self.parameters)})" if self.parameters else ""
        if self.expression:
            lines.append(f"{pad}{kw} {self.name}{params_str} {{")
            lines.append(f"{pad}    {self.expression};")
            lines.append(f"{pad}}}")
        else:
            lines.append(f"{pad}{kw} {self.name}{params_str};")
        return "\n".join(lines)


@dataclass
class SysMLTestCaseDef:
    name: str
    subject_part: str = ""
    verified_requirements: List[str] = field(default_factory=list)
    objective: str = ""
    test_steps: List[str] = field(default_factory=list)
    doc: str = ""

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        lines = []
        if self.doc:
            lines.append(f"{pad}doc /* {self.doc} */")
        lines.append(f"{pad}test case def {self.name} {{")
        if self.subject_part:
            lines.append(f"{pad}    subject {self.subject_part};")
        for req in (self.verified_requirements or []):
            lines.append(f"{pad}    verify requirement {req};")
        if self.objective:
            lines.append(f"{pad}    objective \"{self.objective}\";")
        for step in (self.test_steps or []):
            lines.append(f"{pad}    step {step};")
        lines.append(f"{pad}}}")
        return "\n".join(lines)


@dataclass
class RequirementDef:
    name: str
    doc: str = ""
    req_id: str = ""
    text: str = ""
    assumes: List[str] = field(default_factory=list)
    requires: List[str] = field(default_factory=list)
    verified_by: List[str] = field(default_factory=list)
    satisfied_by: List[str] = field(default_factory=list)

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        lines = []
        if self.doc:
            lines.append(f"{pad}doc /* {self.doc} */")
        lines.append(f"{pad}requirement def {self.name} {{")
        if self.req_id:
            lines.append(f"{pad}    id = \"{self.req_id}\";")
        if self.text:
            lines.append(f"{pad}    doc /* {self.text} */")
        for a in (self.assumes or []):
            lines.append(f"{pad}    assume {a};")
        for r in (self.requires or []):
            lines.append(f"{pad}    require {r};")
        for v in (self.verified_by or []):
            lines.append(f"{pad}    verify by {v};")
        for s in (self.satisfied_by or []):
            lines.append(f"{pad}    satisfy by {s};")
        lines.append(f"{pad}}}")
        return "\n".join(lines)


@dataclass
class StateDef:
    name: str
    doc: str = ""
    entry_action: Optional[str] = None
    do_action: Optional[str] = None
    exit_action: Optional[str] = None
    transitions: List[str] = field(default_factory=list)

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        lines = []
        if self.doc:
            lines.append(f"{pad}doc /* {self.doc} */")
        lines.append(f"{pad}state def {self.name} {{")
        if self.entry_action:
            lines.append(f"{pad}    entry {self.entry_action};")
        if self.do_action:
            lines.append(f"{pad}    do {self.do_action};")
        if self.exit_action:
            lines.append(f"{pad}    exit {self.exit_action};")
        for t in (self.transitions or []):
            lines.append(f"{pad}    transition {t};")
        lines.append(f"{pad}}}")
        return "\n".join(lines)


@dataclass
class UseCaseDef:
    name: str
    doc: str = ""
    subject: str = ""
    actor: str = ""
    objective: str = ""
    includes: List[str] = field(default_factory=list)
    extends: List[str] = field(default_factory=list)

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        lines = []
        if self.doc:
            lines.append(f"{pad}doc /* {self.doc} */")
        lines.append(f"{pad}use case def {self.name} {{")
        if self.subject:
            lines.append(f"{pad}    subject {self.subject};")
        if self.actor:
            lines.append(f"{pad}    actor {self.actor};")
        if self.objective:
            lines.append(f"{pad}    objective \"{self.objective}\";")
        for inc in (self.includes or []):
            lines.append(f"{pad}    include {inc};")
        for ext in (self.extends or []):
            lines.append(f"{pad}    extend {ext};")
        lines.append(f"{pad}}}")
        return "\n".join(lines)


@dataclass
class ItemDef:
    name: str
    doc: str = ""
    attributes: List[AttributeDef] = field(default_factory=list)

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        lines = []
        if self.doc:
            lines.append(f"{pad}doc /* {self.doc} */")
        lines.append(f"{pad}item def {self.name} {{")
        for attr in (self.attributes or []):
            lines.append(attr.to_sysml(indent + 4))
        lines.append(f"{pad}}}")
        return "\n".join(lines)


# Type aliases for consistency
SysMLRequirementDef = RequirementDef
SysMLStateDef = StateDef
SysMLUseCaseDef = UseCaseDef
SysMLItemDef = ItemDef


@dataclass
class PartDef:
    name: str
    doc: str = ""
    attributes: List[AttributeDef] = field(default_factory=list)
    ports: List[PortDef] = field(default_factory=list)
    actions: List[ActionDef] = field(default_factory=list)
    parts: List['PartDef'] = field(default_factory=list)
    operations: List[SysMLOperationDef] = field(default_factory=list)
    capabilities: List[SysMLCapabilityDef] = field(default_factory=list)
    interactions: List[SysMLInteractionDef] = field(default_factory=list)
    constraints: List[SysMLConstraintDef] = field(default_factory=list)
    test_cases: List[SysMLTestCaseDef] = field(default_factory=list)
    states: List[StateDef] = field(default_factory=list)
    requirements: List[RequirementDef] = field(default_factory=list)
    use_cases: List[UseCaseDef] = field(default_factory=list)
    item_defs: List[ItemDef] = field(default_factory=list)

    def to_sysml(self, indent: int = 4) -> str:
        pad = " " * indent
        lines = []
        if self.doc:
            lines.append(f"{pad}doc /* {self.doc} */")
        lines.append(f"{pad}part def {self.name} {{")

        for attr in (self.attributes or []):
            lines.append(attr.to_sysml(indent + 4))
        for port in (self.ports or []):
            lines.append(port.to_sysml(indent + 4))
        for act in (self.actions or []):
            lines.append(act.to_sysml(indent + 4))
        for op in (self.operations or []):
            lines.append(op.to_sysml(indent + 4))
        for cap in (self.capabilities or []):
            lines.append(cap.to_sysml(indent + 4))
        for inter in (self.interactions or []):
            lines.append(inter.to_sysml(indent + 4))
        for c in (self.constraints or []):
            lines.append(c.to_sysml(indent + 4))
        for tc in (self.test_cases or []):
            lines.append(tc.to_sysml(indent + 4))
        for st in (self.states or []):
            lines.append(st.to_sysml(indent + 4))
        for req in (self.requirements or []):
            lines.append(req.to_sysml(indent + 4))
        for uc in (self.use_cases or []):
            lines.append(uc.to_sysml(indent + 4))
        for item in (self.item_defs or []):
            lines.append(item.to_sysml(indent + 4))
        for subpart in (self.parts or []):
            lines.append(subpart.to_sysml(indent + 4))

        lines.append(f"{pad}}}")
        return "\n".join(lines)


SysMLPart = PartDef


@dataclass
class SysMLPackage:
    name: str
    doc: str = ""
    part_defs: List[PartDef] = field(default_factory=list)
    attribute_defs: List[AttributeDef] = field(default_factory=list)
    port_defs: List[PortDef] = field(default_factory=list)
    action_defs: List[ActionDef] = field(default_factory=list)
    sub_packages: List['SysMLPackage'] = field(default_factory=list)
    capability_defs: List[SysMLCapabilityDef] = field(default_factory=list)
    operation_defs: List[SysMLOperationDef] = field(default_factory=list)
    interaction_defs: List[SysMLInteractionDef] = field(default_factory=list)
    constraint_defs: List[SysMLConstraintDef] = field(default_factory=list)
    test_case_defs: List[SysMLTestCaseDef] = field(default_factory=list)
    requirement_defs: List[RequirementDef] = field(default_factory=list)
    state_defs: List[StateDef] = field(default_factory=list)
    use_case_defs: List[UseCaseDef] = field(default_factory=list)
    item_defs: List[ItemDef] = field(default_factory=list)

    def to_sysml(self, indent: int = 0) -> str:
        pad = " " * indent
        lines = []
        if self.doc:
            lines.append(f"{pad}doc /* {self.doc} */")
        lines.append(f"{pad}package {self.name} {{")

        for attr in (self.attribute_defs or []):
            lines.append(attr.to_sysml(indent + 4))
        for port in (self.port_defs or []):
            lines.append(port.to_sysml(indent + 4))
        for act in (self.action_defs or []):
            lines.append(act.to_sysml(indent + 4))
        for op in (self.operation_defs or []):
            lines.append(op.to_sysml(indent + 4))
        for cap in (self.capability_defs or []):
            lines.append(cap.to_sysml(indent + 4))
        for inter in (self.interaction_defs or []):
            lines.append(inter.to_sysml(indent + 4))
        for c in (self.constraint_defs or []):
            lines.append(c.to_sysml(indent + 4))
        for tc in (self.test_case_defs or []):
            lines.append(tc.to_sysml(indent + 4))
        for req in (self.requirement_defs or []):
            lines.append(req.to_sysml(indent + 4))
        for st in (self.state_defs or []):
            lines.append(st.to_sysml(indent + 4))
        for uc in (self.use_case_defs or []):
            lines.append(uc.to_sysml(indent + 4))
        for item in (self.item_defs or []):
            lines.append(item.to_sysml(indent + 4))
        for part in (self.part_defs or []):
            lines.append(part.to_sysml(indent + 4))
        for subpkg in (self.sub_packages or []):
            lines.append(subpkg.to_sysml(indent + 4))

        lines.append(f"{pad}}}")
        return "\n".join(lines)

    def node_counts(self) -> Dict[str, int]:
        counts = {
            "packages": 1,
            "part_defs": len(self.part_defs or []),
            "attribute_defs": len(self.attribute_defs or []),
            "port_defs": len(self.port_defs or []),
            "action_defs": len(self.action_defs or []),
            "capability_defs": len(self.capability_defs or []),
            "operation_defs": len(self.operation_defs or []),
            "interaction_defs": len(self.interaction_defs or []),
            "constraint_defs": len(self.constraint_defs or []),
            "test_case_defs": len(self.test_case_defs or []),
            "requirement_defs": len(self.requirement_defs or []),
            "state_defs": len(self.state_defs or []),
            "use_case_defs": len(self.use_case_defs or []),
            "item_defs": len(self.item_defs or []),
            "containers": len(self.part_defs or []),
            "lists": len(self.action_defs or []),
            "leaves": len(self.attribute_defs or []),
            "typedefs": 0,
            "identities": 0,
            "groupings": 0,
        }

        def _aggregate_part(p: PartDef):
            counts["attribute_defs"] += len(p.attributes or [])
            counts["port_defs"] += len(p.ports or [])
            counts["action_defs"] += len(p.actions or [])
            counts["operation_defs"] += len(p.operations or [])
            counts["capability_defs"] += len(p.capabilities or [])
            counts["interaction_defs"] += len(p.interactions or [])
            counts["constraint_defs"] += len(p.constraints or [])
            counts["test_case_defs"] += len(p.test_cases or [])
            counts["requirement_defs"] += len(p.requirements or [])
            counts["state_defs"] += len(p.states or [])
            counts["use_case_defs"] += len(p.use_cases or [])
            counts["item_defs"] += len(p.item_defs or [])
            counts["leaves"] += len(p.attributes or [])
            counts["lists"] += len(p.actions or [])
            counts["part_defs"] += len(p.parts or [])
            counts["containers"] += len(p.parts or [])
            for sub_p in (p.parts or []):
                _aggregate_part(sub_p)

        for part in (self.part_defs or []):
            _aggregate_part(part)

        for sub in (self.sub_packages or []):
            sub_counts = sub.node_counts()
            for k in counts:
                counts[k] += sub_counts.get(k, 0)
        return counts

    def get_all_node_names(self) -> List[str]:
        names = [self.name]
        for attr in (self.attribute_defs or []):
            names.append(attr.name)
        for port in (self.port_defs or []):
            names.append(port.name)
        for act in (self.action_defs or []):
            names.append(act.name)
        for op in (self.operation_defs or []):
            names.append(op.name)
        for cap in (self.capability_defs or []):
            names.append(cap.name)
        for inter in (self.interaction_defs or []):
            names.append(inter.name)
        for c in (self.constraint_defs or []):
            names.append(c.name)
        for tc in (self.test_case_defs or []):
            names.append(tc.name)
        for req in (self.requirement_defs or []):
            names.append(req.name)
        for st in (self.state_defs or []):
            names.append(st.name)
        for uc in (self.use_case_defs or []):
            names.append(uc.name)
        for item in (self.item_defs or []):
            names.append(item.name)

        def _collect_part_names(p: PartDef):
            names.append(p.name)
            for attr in (p.attributes or []):
                names.append(attr.name)
            for port in (p.ports or []):
                names.append(port.name)
            for act in (p.actions or []):
                names.append(act.name)
            for op in (p.operations or []):
                names.append(op.name)
            for cap in (p.capabilities or []):
                names.append(cap.name)
            for inter in (p.interactions or []):
                names.append(inter.name)
            for c in (p.constraints or []):
                names.append(c.name)
            for tc in (p.test_cases or []):
                names.append(tc.name)
            for req in (p.requirements or []):
                names.append(req.name)
            for st in (p.states or []):
                names.append(st.name)
            for uc in (p.use_cases or []):
                names.append(uc.name)
            for item in (p.item_defs or []):
                names.append(item.name)
            for sub_p in (p.parts or []):
                _collect_part_names(sub_p)

        for part in (self.part_defs or []):
            _collect_part_names(part)

        for sub in (self.sub_packages or []):
            names.extend(sub.get_all_node_names())

        return sorted(list(set(names)))


class SysMLParser:
    """
    Parser for Canonical SysML v2 textual models.
    Translates textual SysML v2 packages, parts, capabilities, operations,
    interactions, constraints/assertions, test cases, requirements, states,
    actions, ports, and attributes into a canonical SysMLPackage AST.
    """

    @classmethod
    def parse_file(cls, filepath: str) -> SysMLPackage:
        if not os.path.exists(filepath):
            raise FileNotFoundError(f"SysML file not found: {filepath}")
        with open(filepath, "r", encoding="utf-8") as f:
            content = f.read()
        default_name = os.path.splitext(os.path.basename(filepath))[0]
        return cls.parse_text(content, default_name=default_name)

    @classmethod
    def parse_text(cls, content: str, default_name: str = "SysML_Model") -> SysMLPackage:
        parser = cls()
        return parser._parse(content, default_name=default_name)

    def _parse(self, content: str, default_name: str = "SysML_Model") -> SysMLPackage:
        decls = self._scan_declarations(content)
        pkg_decls = [d for d in decls if d["type"] == "block" and self._is_keyword(d["header"], "package")]

        if pkg_decls:
            primary_pkg = None
            for p_decl in pkg_decls:
                pkg_obj = self._parse_package(p_decl)
                if primary_pkg is None:
                    primary_pkg = pkg_obj
                else:
                    primary_pkg.sub_packages.append(pkg_obj)
            non_pkg_decls = [d for d in decls if not (d["type"] == "block" and self._is_keyword(d["header"], "package"))]
            if non_pkg_decls and primary_pkg is not None:
                self._populate_container(primary_pkg, non_pkg_decls)
            return primary_pkg if primary_pkg is not None else SysMLPackage(name=default_name)
        else:
            # Flat file without top-level package block
            pkg = SysMLPackage(name=default_name)
            self._populate_container(pkg, decls)
            return pkg

    def _is_keyword(self, header: str, keyword: str) -> bool:
        tokens = header.strip().split()
        return len(tokens) > 0 and tokens[0] == keyword

    def _scan_declarations(self, text: str) -> List[Dict[str, Any]]:
        decls = []
        i = 0
        n = len(text)
        doc_comment = ""

        while i < n:
            while i < n and text[i].isspace():
                i += 1
            if i >= n:
                break

            # Handle doc /* ... */ or /* ... */
            if text.startswith('/*', i):
                end_c = text.find('*/', i + 2)
                if end_c == -1:
                    break
                raw_c = text[i+2:end_c].strip()
                if raw_c.startswith('doc'):
                    raw_c = raw_c[3:].strip()
                doc_comment = raw_c
                i = end_c + 2
                continue
            elif text.startswith('//', i):
                end_c = text.find('\n', i + 2)
                if end_c == -1:
                    break
                i = end_c + 1
                continue
            elif text.startswith('doc', i) and i + 3 < n and (text[i+3].isspace() or text[i+3] in ('/', '"', "'")):
                i += 3
                while i < n and text[i].isspace():
                    i += 1
                if text.startswith('/*', i):
                    end_c = text.find('*/', i + 2)
                    if end_c != -1:
                        doc_comment = text[i+2:end_c].strip()
                        i = end_c + 2
                        continue
                elif text.startswith('"', i) or text.startswith("'", i):
                    quote_char = text[i]
                    end_c = text.find(quote_char, i + 1)
                    if end_c != -1:
                        doc_comment = text[i+1:end_c].strip()
                        i = end_c + 1
                        continue

            start_decl = i
            brace_pos = None
            semi_pos = None

            j = i
            in_paren = 0
            in_str = None
            while j < n:
                ch = text[j]
                if in_str:
                    if ch == '\\':
                        j += 2
                        continue
                    elif ch == in_str:
                        in_str = None
                else:
                    if ch in ('"', "'"):
                        in_str = ch
                    elif ch == '(':
                        in_paren += 1
                    elif ch == ')':
                        in_paren = max(0, in_paren - 1)
                    elif ch == '{' and in_paren == 0:
                        brace_pos = j
                        break
                    elif ch == ';' and in_paren == 0:
                        semi_pos = j
                        break
                j += 1

            if brace_pos is not None and (semi_pos is None or brace_pos < semi_pos):
                header = text[start_decl:brace_pos].strip()
                body_start = brace_pos + 1
                depth = 1
                k = body_start
                in_str = None
                while k < n and depth > 0:
                    ch = text[k]
                    if in_str:
                        if ch == '\\':
                            k += 2
                            continue
                        elif ch == in_str:
                            in_str = None
                    else:
                        if ch in ('"', "'"):
                            in_str = ch
                        elif ch == '/' and k + 1 < n and text[k+1] == '*':
                            end_k = text.find('*/', k + 2)
                            if end_k != -1:
                                k = end_k + 2
                                continue
                        elif ch == '/' and k + 1 < n and text[k+1] == '/':
                            end_k = text.find('\n', k + 2)
                            if end_k != -1:
                                k = end_k + 1
                                continue
                        elif ch == '{':
                            depth += 1
                        elif ch == '}':
                            depth -= 1
                            if depth == 0:
                                break
                    k += 1
                body_end = k
                body = text[body_start:body_end]
                decls.append({
                    "type": "block",
                    "header": header,
                    "body": body,
                    "doc": doc_comment,
                })
                doc_comment = ""
                i = k + 1
                while i < n and text[i].isspace():
                    i += 1
                if i < n and text[i] == ';':
                    i += 1
            elif semi_pos is not None:
                statement = text[start_decl:semi_pos].strip()
                decls.append({
                    "type": "statement",
                    "statement": statement,
                    "doc": doc_comment,
                })
                doc_comment = ""
                i = semi_pos + 1
            else:
                break

        return decls

    def _parse_package(self, decl: Dict[str, Any]) -> SysMLPackage:
        header = decl["header"]
        match = re.search(r'\bpackage\s+([a-zA-Z0-9_\-\.]+)', header)
        pkg_name = match.group(1).replace('.', '_') if match else "Package"
        pkg = SysMLPackage(name=pkg_name, doc=decl.get("doc", ""))

        body_decls = self._scan_declarations(decl["body"])
        self._populate_container(pkg, body_decls)
        return pkg

    def _populate_container(self, container: Union[SysMLPackage, PartDef], decls: List[Dict[str, Any]]) -> None:
        for d in decls:
            if d["type"] == "block":
                header = d["header"]
                doc = d.get("doc", "")

                if re.search(r'\bcapability\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    c_obj = self._parse_capability_block(d)
                    if isinstance(container, SysMLPackage):
                        container.capability_defs.append(c_obj)
                    else:
                        container.capabilities.append(c_obj)

                elif re.search(r'\binteraction\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    i_obj = self._parse_interaction_block(d)
                    if isinstance(container, SysMLPackage):
                        container.interaction_defs.append(i_obj)
                    else:
                        container.interactions.append(i_obj)

                elif re.search(r'\b(?:assert\s+constraint|constraint\s+def|constraint)\s+([a-zA-Z0-9_]+)', header):
                    con_obj = self._parse_constraint_block(d)
                    if isinstance(container, SysMLPackage):
                        container.constraint_defs.append(con_obj)
                    else:
                        container.constraints.append(con_obj)

                elif re.search(r'\btest\s+case\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    tc_obj = self._parse_test_case_block(d)
                    if isinstance(container, SysMLPackage):
                        container.test_case_defs.append(tc_obj)
                    else:
                        container.test_cases.append(tc_obj)

                elif re.search(r'\bpart\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    part_obj = self._parse_part_block(d)
                    if isinstance(container, SysMLPackage):
                        container.part_defs.append(part_obj)
                    else:
                        container.parts.append(part_obj)

                elif re.search(r'\bpackage\s+([a-zA-Z0-9_\-\.]+)', header):
                    if isinstance(container, SysMLPackage):
                        sub_pkg = self._parse_package(d)
                        container.sub_packages.append(sub_pkg)

                elif re.search(r'\baction\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    act_obj = self._parse_action_decl(header, doc)
                    if isinstance(container, SysMLPackage):
                        container.action_defs.append(act_obj)
                    else:
                        container.actions.append(act_obj)

                elif re.search(r'\boperation\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    op_obj = self._parse_operation_decl(header, doc)
                    if isinstance(container, SysMLPackage):
                        container.operation_defs.append(op_obj)
                    else:
                        container.operations.append(op_obj)

                elif re.search(r'\brequirement\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    req_obj = self._parse_requirement_block(d)
                    if isinstance(container, SysMLPackage):
                        container.requirement_defs.append(req_obj)
                    else:
                        container.requirements.append(req_obj)

                elif re.search(r'\bstate\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    state_obj = self._parse_state_block(d)
                    if isinstance(container, SysMLPackage):
                        container.state_defs.append(state_obj)
                    else:
                        container.states.append(state_obj)

                elif re.search(r'\buse\s+case\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    uc_obj = self._parse_use_case_block(d)
                    if isinstance(container, SysMLPackage):
                        container.use_case_defs.append(uc_obj)
                    else:
                        container.use_cases.append(uc_obj)

                elif re.search(r'\bitem\s+(?:def\s+)?([a-zA-Z0-9_]+)', header):
                    item_obj = self._parse_item_block(d)
                    if isinstance(container, SysMLPackage):
                        container.item_defs.append(item_obj)
                    else:
                        container.item_defs.append(item_obj)

            elif d["type"] == "statement":
                stmt = d["statement"]
                doc = d.get("doc", "")

                if re.search(r'\b(?:assert\s+constraint|constraint\s+def|constraint)\s+([a-zA-Z0-9_]+)', stmt):
                    con_obj = self._parse_constraint_stmt(stmt, doc)
                    if isinstance(container, SysMLPackage):
                        container.constraint_defs.append(con_obj)
                    else:
                        container.constraints.append(con_obj)

                elif re.search(r'\bcapability\s+(?:def\s+)?([a-zA-Z0-9_]+)', stmt):
                    m = re.search(r'\bcapability\s+(?:def\s+)?([a-zA-Z0-9_]+)', stmt)
                    cap_obj = SysMLCapabilityDef(name=m.group(1), doc=doc)
                    if isinstance(container, SysMLPackage):
                        container.capability_defs.append(cap_obj)
                    else:
                        container.capabilities.append(cap_obj)

                elif re.search(r'\b(?:operation|feature)\s+(?:def\s+)?([a-zA-Z0-9_]+)', stmt):
                    op_obj = self._parse_operation_decl(stmt, doc)
                    if isinstance(container, SysMLPackage):
                        container.operation_defs.append(op_obj)
                    else:
                        container.operations.append(op_obj)

                elif re.search(r'\baction\s+(?:def\s+)?([a-zA-Z0-9_]+)', stmt):
                    act_obj = self._parse_action_decl(stmt, doc)
                    if isinstance(container, SysMLPackage):
                        container.action_defs.append(act_obj)
                    else:
                        container.actions.append(act_obj)

                elif re.search(r'\b(?:in|out|inout)?\s*port\s+(?:def\s+)?([a-zA-Z0-9_]+)', stmt):
                    port_obj = self._parse_port_stmt(stmt, doc)
                    if isinstance(container, SysMLPackage):
                        container.port_defs.append(port_obj)
                    else:
                        container.ports.append(port_obj)

                elif re.search(r'\battribute\s+(?:def\s+)?([a-zA-Z0-9_]+)', stmt):
                    attr_obj = self._parse_attribute_stmt(stmt, doc)
                    if isinstance(container, SysMLPackage):
                        container.attribute_defs.append(attr_obj)
                    else:
                        container.attributes.append(attr_obj)

    def _parse_part_block(self, decl: Dict[str, Any]) -> PartDef:
        header = decl["header"]
        m = re.search(r'\bpart\s+(?:def\s+)?([a-zA-Z0-9_]+)', header)
        name = m.group(1) if m else "Part"
        part = PartDef(name=name, doc=decl.get("doc", ""))
        body_decls = self._scan_declarations(decl["body"])
        self._populate_container(part, body_decls)
        return part

    def _parse_capability_block(self, decl: Dict[str, Any]) -> SysMLCapabilityDef:
        header = decl["header"]
        m = re.search(r'\bcapability\s+(?:def\s+)?([a-zA-Z0-9_]+)', header)
        name = m.group(1) if m else "Capability"
        doc = decl.get("doc", "")
        description = doc
        subsystem = ""
        package_ref = ""

        body_decls = self._scan_declarations(decl["body"])
        for d in body_decls:
            if d["type"] == "statement":
                stmt = d["statement"]
                subsys_m = re.search(r'\b(?:subsystem|package|subject)\s+([a-zA-Z0-9_\-\.]+)', stmt)
                if subsys_m:
                    subsystem = subsys_m.group(1)
                    package_ref = subsys_m.group(1)
                desc_m = re.search(r'\bdescription\s*[:=]\s*["\']?([^"\']+)["\']?', stmt)
                if desc_m:
                    description = desc_m.group(1).strip()
            if d.get("doc") and not description:
                description = d["doc"]

        return SysMLCapabilityDef(
            name=name,
            description=description,
            subsystem=subsystem,
            package_ref=package_ref,
            doc=doc or description
        )

    def _parse_interaction_block(self, decl: Dict[str, Any]) -> SysMLInteractionDef:
        header = decl["header"]
        m = re.search(r'\binteraction\s+(?:def\s+)?([a-zA-Z0-9_]+)', header)
        name = m.group(1) if m else "Interaction"
        doc = decl.get("doc", "")

        lifelines = []
        messages = []
        triggers = []

        body_decls = self._scan_declarations(decl["body"])
        for d in body_decls:
            if d["type"] == "statement":
                stmt = d["statement"]
                # Lifelines
                ll_m = re.search(r'\b(?:lifeline|part|actor)\s+([a-zA-Z0-9_]+)', stmt)
                if ll_m:
                    lifelines.append(ll_m.group(1))

                # Messages / flows
                msg_m = re.search(r'\b(?:message|send|action|flow)\s+([a-zA-Z0-9_]+)', stmt)
                if msg_m:
                    messages.append(msg_m.group(1))

                # Triggers / events
                trg_m = re.search(r'\b(?:trigger|on|when|after|event)\s+([a-zA-Z0-9_]+)', stmt)
                if trg_m:
                    triggers.append(trg_m.group(1))

        return SysMLInteractionDef(
            name=name,
            lifelines=lifelines,
            messages=messages,
            triggers=triggers,
            doc=doc
        )

    def _parse_constraint_block(self, decl: Dict[str, Any]) -> SysMLConstraintDef:
        header = decl["header"]
        is_assertion = bool(re.search(r'\bassert\s+constraint\b', header))
        m = re.search(r'\b(?:assert\s+constraint|constraint\s+def|constraint)\s+([a-zA-Z0-9_]+)', header)
        name = m.group(1) if m else "Constraint"
        doc = decl.get("doc", "")

        # Extract parameters if present in header, e.g. (in x: Type)
        params = []
        p_match = re.search(r'\(([^)]*)\)', header)
        if p_match and p_match.group(1).strip():
            raw_params = p_match.group(1).strip()
            params = [p.strip() for p in raw_params.split(',') if p.strip()]

        raw_body = decl["body"].strip()
        doc_in_body = re.search(r'(?:doc\s*)?/\*(.*?)\*/', raw_body, re.DOTALL)
        if doc_in_body:
            if not doc:
                extracted_doc = doc_in_body.group(1).strip()
                if extracted_doc.startswith("doc"):
                    extracted_doc = extracted_doc[3:].strip()
                doc = extracted_doc
            raw_body = re.sub(r'(?:doc\s*)?/\*.*?\*/', '', raw_body, flags=re.DOTALL)

        raw_body = re.sub(r'//.*', '', raw_body)
        raw_body = re.sub(r'[\r\n\t]+', ' ', raw_body).strip()
        if raw_body.endswith(';'):
            raw_body = raw_body[:-1].strip()

        expression = raw_body

        return SysMLConstraintDef(
            name=name,
            expression=expression,
            parameters=params,
            is_assertion=is_assertion,
            doc=doc
        )

    def _parse_constraint_stmt(self, stmt: str, doc: str = "") -> SysMLConstraintDef:
        is_assertion = bool(re.search(r'\bassert\s+constraint\b', stmt))
        m = re.search(r'\b(?:assert\s+constraint|constraint\s+def|constraint)\s+([a-zA-Z0-9_]+)', stmt)
        name = m.group(1) if m else "Constraint"

        params = []
        p_match = re.search(r'\(([^)]*)\)', stmt)
        if p_match and p_match.group(1).strip():
            raw_params = p_match.group(1).strip()
            params = [p.strip() for p in raw_params.split(',') if p.strip()]

        expression = ""
        if ':' in stmt:
            expression = stmt.split(':', 1)[1].strip()
        elif '=' in stmt:
            expression = stmt.split('=', 1)[1].strip()

        return SysMLConstraintDef(
            name=name,
            expression=expression,
            parameters=params,
            is_assertion=is_assertion,
            doc=doc
        )

    def _parse_test_case_block(self, decl: Dict[str, Any]) -> SysMLTestCaseDef:
        header = decl["header"]
        m = re.search(r'\btest\s+case\s+(?:def\s+)?([a-zA-Z0-9_]+)', header)
        name = m.group(1) if m else "TestCase"
        doc = decl.get("doc", "")

        subject_part = ""
        verified_requirements = []
        objective = doc
        test_steps = []

        body_decls = self._scan_declarations(decl["body"])
        for d in body_decls:
            if d["type"] == "statement":
                stmt = d["statement"]
                if re.match(r'^\s*subject\b', stmt):
                    subj_m = re.search(r'^\s*subject\s+(?:part\s+)?([a-zA-Z0-9_]+)', stmt)
                    if subj_m:
                        subject_part = subj_m.group(1)

                elif re.match(r'^\s*verify\b', stmt):
                    req_m = re.search(r'^\s*verify\s+(?:requirement\s+)?([a-zA-Z0-9_\-]+)', stmt)
                    if req_m:
                        verified_requirements.append(req_m.group(1))

                elif re.match(r'^\s*objective\b', stmt):
                    obj_m = re.search(r'^\s*objective\s*[:=]?\s*["\']?([^"\']+)["\']?', stmt)
                    if obj_m:
                        objective = obj_m.group(1).strip()

                elif re.match(r'^\s*(?:step|test\s+step|action|assert)\b', stmt):
                    step_m = re.search(r'^\s*(?:step|test\s+step|action|assert)\s+([a-zA-Z0-9_\-]+)', stmt)
                    if step_m:
                        test_steps.append(step_m.group(1))

        return SysMLTestCaseDef(
            name=name,
            subject_part=subject_part,
            verified_requirements=verified_requirements,
            objective=objective,
            test_steps=test_steps,
            doc=doc
        )

    def _parse_action_decl(self, text: str, doc: str = "") -> ActionDef:
        m = re.search(r'\baction\s+(?:def\s+)?([a-zA-Z0-9_]+)', text)
        name = m.group(1) if m else "Action"
        in_params = []
        out_params = []

        p_match = re.search(r'\(([^)]*)\)', text)
        if p_match and p_match.group(1).strip():
            raw_params = p_match.group(1).strip()
            param_items = [p.strip() for p in raw_params.split(',') if p.strip()]
            for p in param_items:
                p_parts = p.split()
                if len(p_parts) >= 3 and p_parts[0] in ('in', 'out', 'inout'):
                    direction = p_parts[0]
                    # Format: in name : type
                    p_name = p_parts[1].rstrip(':')
                    p_type = p_parts[2] if len(p_parts) > 2 else "String"
                    if len(p_parts) >= 4 and p_parts[2] == ':':
                        p_type = p_parts[3]
                    attr = AttributeDef(name=p_name, type_name=p_type)
                    if direction == 'out':
                        out_params.append(attr)
                    else:
                        in_params.append(attr)
                elif len(p_parts) >= 2:
                    p_name = p_parts[0].rstrip(':')
                    p_type = p_parts[1]
                    if len(p_parts) >= 3 and p_parts[1] == ':':
                        p_type = p_parts[2]
                    in_params.append(AttributeDef(name=p_name, type_name=p_type))

        return ActionDef(name=name, doc=doc, in_params=in_params, out_params=out_params)

    def _parse_operation_decl(self, text: str, doc: str = "") -> SysMLOperationDef:
        m = re.search(r'\b(?:operation|feature)\s+(?:def\s+)?([a-zA-Z0-9_]+)', text)
        name = m.group(1) if m else "Operation"

        return_type = None
        # Check return type after ':' outside params
        after_paren = text
        p_match = re.search(r'\(([^)]*)\)', text)
        if p_match:
            after_paren = text[p_match.end():]
        ret_m = re.search(r':\s*([a-zA-Z0-9_<>:]+)', after_paren)
        if ret_m:
            return_type = ret_m.group(1).strip()

        parameters = []
        direction = "inout"
        param_type = "String"

        if p_match and p_match.group(1).strip():
            raw_params = p_match.group(1).strip()
            param_items = [p.strip() for p in raw_params.split(',') if p.strip()]
            for p in param_items:
                p_parts = p.split()
                if len(p_parts) >= 3 and p_parts[0] in ('in', 'out', 'inout'):
                    dir_val = p_parts[0]
                    p_name = p_parts[1].rstrip(':')
                    p_t = p_parts[2] if len(p_parts) > 2 else "String"
                    if len(p_parts) >= 4 and p_parts[2] == ':':
                        p_t = p_parts[3]
                    parameters.append(AttributeDef(name=p_name, type_name=p_t, default_value=dir_val))
                elif len(p_parts) >= 2:
                    p_name = p_parts[0].rstrip(':')
                    p_t = p_parts[1]
                    if len(p_parts) >= 3 and p_parts[1] == ':':
                        p_t = p_parts[2]
                    parameters.append(AttributeDef(name=p_name, type_name=p_t, default_value="in"))

            if parameters:
                param_type = parameters[0].type_name
                direction = getattr(parameters[0], "default_value", "inout")

        return SysMLOperationDef(
            name=name,
            direction=direction,
            param_type=param_type,
            return_type=return_type,
            doc=doc,
            parameters=parameters
        )

    def _parse_port_stmt(self, stmt: str, doc: str = "") -> PortDef:
        dir_m = re.search(r'\b(in|out|inout)\b', stmt)
        direction = dir_m.group(1) if dir_m else "inout"

        m = re.search(r'\bport\s+(?:def\s+)?([a-zA-Z0-9_]+)', stmt)
        name = m.group(1) if m else "Port"

        type_m = re.search(r':\s*([a-zA-Z0-9_<>:]+)', stmt)
        type_name = type_m.group(1).strip() if type_m else "Port"

        return PortDef(name=name, type_name=type_name, direction=direction, doc=doc)

    def _parse_attribute_stmt(self, stmt: str, doc: str = "") -> AttributeDef:
        m = re.search(r'\battribute\s+(?:def\s+)?([a-zA-Z0-9_]+)', stmt)
        name = m.group(1) if m else "Attribute"

        type_name = "String"
        type_m = re.search(r':\s*([a-zA-Z0-9_<>:]+)', stmt)
        if type_m:
            type_name = type_m.group(1).strip()

        default_value = None
        def_m = re.search(r'=\s*([^;]+)', stmt)
        if def_m:
            default_value = def_m.group(1).strip()

        return AttributeDef(name=name, type_name=type_name, doc=doc, default_value=default_value)

    def _parse_requirement_block(self, decl: Dict[str, Any]) -> RequirementDef:
        header = decl["header"]
        m = re.search(r'\brequirement\s+(?:def\s+)?([a-zA-Z0-9_]+)', header)
        name = m.group(1) if m else "Requirement"
        doc = decl.get("doc", "")
        req_id = ""
        text = doc

        assumes = []
        requires = []
        verified_by = []
        satisfied_by = []

        body_decls = self._scan_declarations(decl["body"])
        for d in body_decls:
            if d["type"] == "statement":
                stmt = d["statement"]
                id_m = re.search(r'\bid\s*[:=]\s*["\']?([^"\']+)["\']?', stmt)
                if id_m:
                    req_id = id_m.group(1).strip()
                asm_m = re.search(r'\bassume\s+([^;]+)', stmt)
                if asm_m:
                    assumes.append(asm_m.group(1).strip())
                req_m = re.search(r'\brequire\s+([^;]+)', stmt)
                if req_m:
                    requires.append(req_m.group(1).strip())
                vb_m = re.search(r'\bverify\s+(?:by\s+)?([^;]+)', stmt)
                if vb_m:
                    verified_by.append(vb_m.group(1).strip())
                sb_m = re.search(r'\bsatisfy\s+(?:by\s+)?([^;]+)', stmt)
                if sb_m:
                    satisfied_by.append(sb_m.group(1).strip())
            if d.get("doc") and not text:
                text = d["doc"]

        return RequirementDef(
            name=name,
            doc=doc,
            req_id=req_id,
            text=text,
            assumes=assumes,
            requires=requires,
            verified_by=verified_by,
            satisfied_by=satisfied_by
        )

    def _parse_state_block(self, decl: Dict[str, Any]) -> StateDef:
        header = decl["header"]
        m = re.search(r'\bstate\s+(?:def\s+)?([a-zA-Z0-9_]+)', header)
        name = m.group(1) if m else "State"
        doc = decl.get("doc", "")
        entry_action = None
        do_action = None
        exit_action = None
        transitions = []

        body_decls = self._scan_declarations(decl["body"])
        for d in body_decls:
            if d["type"] == "statement":
                stmt = d["statement"]
                if stmt.startswith("entry"):
                    entry_action = stmt[5:].strip()
                elif stmt.startswith("do"):
                    do_action = stmt[2:].strip()
                elif stmt.startswith("exit"):
                    exit_action = stmt[4:].strip()
                elif stmt.startswith("transition"):
                    transitions.append(stmt[10:].strip())

        return StateDef(
            name=name,
            doc=doc,
            entry_action=entry_action,
            do_action=do_action,
            exit_action=exit_action,
            transitions=transitions
        )

    def _parse_use_case_block(self, decl: Dict[str, Any]) -> UseCaseDef:
        header = decl["header"]
        m = re.search(r'\buse\s+case\s+(?:def\s+)?([a-zA-Z0-9_]+)', header)
        name = m.group(1) if m else "UseCase"
        doc = decl.get("doc", "")
        subject = ""
        actor = ""
        objective = doc
        includes = []
        extends = []

        body_decls = self._scan_declarations(decl["body"])
        for d in body_decls:
            if d["type"] == "statement":
                stmt = d["statement"]
                subj_m = re.search(r'\bsubject\s+([a-zA-Z0-9_]+)', stmt)
                if subj_m:
                    subject = subj_m.group(1)
                act_m = re.search(r'\bactor\s+([a-zA-Z0-9_]+)', stmt)
                if act_m:
                    actor = act_m.group(1)
                obj_m = re.search(r'\bobjective\s*[:=]?\s*["\']?([^"\']+)["\']?', stmt)
                if obj_m:
                    objective = obj_m.group(1).strip()
                inc_m = re.search(r'\binclude\s+([a-zA-Z0-9_]+)', stmt)
                if inc_m:
                    includes.append(inc_m.group(1))
                ext_m = re.search(r'\bextend\s+([a-zA-Z0-9_]+)', stmt)
                if ext_m:
                    extends.append(ext_m.group(1))

        return UseCaseDef(
            name=name,
            doc=doc,
            subject=subject,
            actor=actor,
            objective=objective,
            includes=includes,
            extends=extends
        )

    def _parse_item_block(self, decl: Dict[str, Any]) -> ItemDef:
        header = decl["header"]
        m = re.search(r'\bitem\s+(?:def\s+)?([a-zA-Z0-9_]+)', header)
        name = m.group(1) if m else "Item"
        doc = decl.get("doc", "")
        attributes = []

        body_decls = self._scan_declarations(decl["body"])
        for d in body_decls:
            if d["type"] == "statement":
                stmt = d["statement"]
                if re.search(r'\battribute\s+', stmt):
                    attributes.append(self._parse_attribute_stmt(stmt, d.get("doc", "")))

        return ItemDef(name=name, doc=doc, attributes=attributes)
