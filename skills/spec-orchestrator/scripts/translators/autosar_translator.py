#!/usr/bin/env python3
"""
AUTOSAR ARXML to SysML v2 Translator
Parses ARXML packages, SW component types, ports, and runnables into SysML v2 AST.
"""

import xml.etree.ElementTree as ET
from typing import List, Optional
import sys
import os

try:
    from sysmlv2_ast import SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef
except ImportError:
    from skills.spec_orchestrator.scripts.sysmlv2_ast import (
        SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef
    )


class AUTOSARTranslator:
    def __init__(self):
        pass

    def _strip_ns(self, tag: str) -> str:
        if '}' in tag:
            return tag.split('}', 1)[1]
        return tag

    def translate(self, content: str, default_name: str = "AUTOSAR_Package") -> SysMLPackage:
        pkg_name = default_name
        try:
            root = ET.fromstring(content)
        except Exception as e:
            # Fallback if content contains syntax issue
            return SysMLPackage(name=default_name, doc=f"Parsing error: {e}")

        # Find first AR-PACKAGE short name if available
        for elem in root.iter():
            if self._strip_ns(elem.tag) == "AR-PACKAGE":
                for child in elem:
                    if self._strip_ns(child.tag) == "SHORT-NAME" and child.text:
                        pkg_name = child.text.strip()
                        break
                if pkg_name != default_name:
                    break

        pkg = SysMLPackage(name=pkg_name, doc="Translated from AUTOSAR ARXML schema")

        # Iterate components
        for elem in root.iter():
            tag = self._strip_ns(elem.tag)
            if tag.endswith("-SW-COMPONENT-TYPE"):
                comp_name = "SWComponent"
                ports = []
                actions = []
                attrs = []

                for child in elem:
                    c_tag = self._strip_ns(child.tag)
                    if c_tag == "SHORT-NAME" and child.text:
                        comp_name = child.text.strip()
                    elif c_tag == "PORTS":
                        for port_elem in child:
                            p_tag = self._strip_ns(port_elem.tag)
                            p_name = ""
                            direction = "inout"
                            if p_tag == "P-PORT-PROTOTYPE":
                                direction = "out"
                            elif p_tag == "R-PORT-PROTOTYPE":
                                direction = "in"

                            for p_child in port_elem:
                                if self._strip_ns(p_child.tag) == "SHORT-NAME" and p_child.text:
                                    p_name = p_child.text.strip()
                            if p_name:
                                ports.append(PortDef(name=p_name, direction=direction))

                    elif c_tag == "INTERNAL-BEHAVIORS":
                        for ib_elem in child.iter():
                            if self._strip_ns(ib_elem.tag) == "RUNNABLE-ENTITY":
                                r_name = ""
                                for r_child in ib_elem:
                                    if self._strip_ns(r_child.tag) == "SHORT-NAME" and r_child.text:
                                        r_name = r_child.text.strip()
                                if r_name:
                                    actions.append(ActionDef(name=r_name))

                pkg.part_defs.append(
                    PartDef(
                        name=comp_name,
                        doc=f"AUTOSAR Component ({tag})",
                        ports=ports,
                        actions=actions,
                        attributes=attrs
                    )
                )

        return pkg
