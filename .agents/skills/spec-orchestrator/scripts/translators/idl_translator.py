#!/usr/bin/env python3
"""
OMG IDL to SysML v2 Translator
Parses OMG IDL modules, structs, interfaces, typedefs, enums into SysML v2 AST.
"""

import re
from typing import List, Optional
import sys
import os

# Handle both direct execution and package import
try:
    from sysmlv2_ast import SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef
except ImportError:
    from skills.spec_orchestrator.scripts.sysmlv2_ast import (
        SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef
    )


class IDLTranslator:
    def __init__(self):
        pass

    def translate(self, content: str, default_name: str = "IDL_Package") -> SysMLPackage:
        pkg_name = default_name
        module_match = re.search(r'\bmodule\s+([a-zA-Z0-9_\-]+)\s*\{', content)
        if module_match:
            pkg_name = module_match.group(1)

        pkg = SysMLPackage(name=pkg_name, doc="Translated from OMG IDL schema")

        # Parse structs
        struct_pattern = re.compile(r'\bstruct\s+([a-zA-Z0-9_\-]+)\s*\{([^}]+)\};', re.MULTILINE | re.DOTALL)
        for match in struct_pattern.finditer(content):
            s_name = match.group(1)
            s_body = match.group(2)
            attrs = []
            field_pattern = re.compile(r'^\s*([a-zA-Z0-9_\-\<\>:]+)\s+([a-zA-Z0-9_\-]+)\s*;', re.MULTILINE)
            for f_match in field_pattern.finditer(s_body):
                f_type = f_match.group(1).strip()
                f_name = f_match.group(2).strip()
                attrs.append(AttributeDef(name=f_name, type_name=f_type))
            pkg.part_defs.append(PartDef(name=s_name, doc="IDL Struct", attributes=attrs))

        # Parse interfaces
        interface_pattern = re.compile(r'\binterface\s+([a-zA-Z0-9_\-]+)\s*\{([^}]+)\};', re.MULTILINE | re.DOTALL)
        for match in interface_pattern.finditer(content):
            if_name = match.group(1)
            if_body = match.group(2)
            actions = []
            op_pattern = re.compile(r'^\s*(?:[a-zA-Z0-9_\-\:]+)\s+([a-zA-Z0-9_\-]+)\s*\(([^)]*)\)\s*;', re.MULTILINE)
            for op_match in op_pattern.finditer(if_body):
                op_name = op_match.group(1).strip()
                params_raw = op_match.group(2).strip()
                in_params = []
                out_params = []
                if params_raw:
                    param_items = [p.strip() for p in params_raw.split(',') if p.strip()]
                    for p in param_items:
                        p_parts = p.split()
                        if len(p_parts) >= 3 and p_parts[0] in ('in', 'out', 'inout'):
                            direction, p_type, p_n = p_parts[0], p_parts[1], p_parts[2]
                            attr = AttributeDef(name=p_n, type_name=p_type)
                            if direction == 'out':
                                out_params.append(attr)
                            else:
                                in_params.append(attr)
                        elif len(p_parts) >= 2:
                            p_type, p_n = p_parts[0], p_parts[1]
                            in_params.append(AttributeDef(name=p_n, type_name=p_type))
                actions.append(ActionDef(name=op_name, in_params=in_params, out_params=out_params))
            pkg.part_defs.append(PartDef(name=if_name, doc="IDL Interface", actions=actions))

        # Parse typedefs
        typedef_pattern = re.compile(r'^\s*typedef\s+([a-zA-Z0-9_\-\<\>:]+)\s+([a-zA-Z0-9_\-]+)\s*;', re.MULTILINE)
        for match in typedef_pattern.finditer(content):
            t_type = match.group(1).strip()
            t_name = match.group(2).strip()
            pkg.attribute_defs.append(AttributeDef(name=t_name, type_name=t_type, doc="IDL Typedef"))

        return pkg
