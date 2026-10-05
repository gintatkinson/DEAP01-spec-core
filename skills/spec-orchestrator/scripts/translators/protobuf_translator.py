#!/usr/bin/env python3
"""
Protobuf to SysML v2 Translator
Parses Protobuf messages, services, enums, fields into SysML v2 AST.
"""

import re
from typing import List, Optional
import sys
import os

try:
    from sysmlv2_ast import SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef
except ImportError:
    from skills.spec_orchestrator.scripts.sysmlv2_ast import (
        SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef
    )


class ProtobufTranslator:
    def __init__(self):
        pass

    def translate(self, content: str, default_name: str = "Protobuf_Package") -> SysMLPackage:
        pkg_name = default_name
        pkg_match = re.search(r'^\s*package\s+([a-zA-Z0-9_\-\.]+)\s*;', content, re.MULTILINE)
        if pkg_match:
            pkg_name = pkg_match.group(1).replace('.', '_')

        pkg = SysMLPackage(name=pkg_name, doc="Translated from Protobuf schema")

        # Parse messages
        msg_pattern = re.compile(r'\bmessage\s+([a-zA-Z0-9_\-]+)\s*\{([^}]+)\}', re.MULTILINE | re.DOTALL)
        for match in msg_pattern.finditer(content):
            m_name = match.group(1)
            m_body = match.group(2)
            attrs = []
            field_pattern = re.compile(
                r'^\s*(?:repeated|optional|required)?\s*([a-zA-Z0-9_\-\.]+)\s+([a-zA-Z0-9_\-]+)\s*=\s*\d+\s*;',
                re.MULTILINE
            )
            for f_match in field_pattern.finditer(m_body):
                f_type = f_match.group(1).strip()
                f_name = f_match.group(2).strip()
                if f_type not in ('message', 'enum', 'service'):
                    attrs.append(AttributeDef(name=f_name, type_name=f_type))
            pkg.part_defs.append(PartDef(name=m_name, doc="Protobuf Message", attributes=attrs))

        # Parse services
        service_pattern = re.compile(r'\bservice\s+([a-zA-Z0-9_\-]+)\s*\{([^}]+)\}', re.MULTILINE | re.DOTALL)
        for match in service_pattern.finditer(content):
            s_name = match.group(1)
            s_body = match.group(2)
            actions = []
            rpc_pattern = re.compile(
                r'\brpc\s+([a-zA-Z0-9_\-]+)\s*\(\s*(?:stream\s+)?([a-zA-Z0-9_\-\.]+)\s*\)\s*returns\s*\(\s*(?:stream\s+)?([a-zA-Z0-9_\-\.]+)\s*\)',
                re.MULTILINE
            )
            for r_match in rpc_pattern.finditer(s_body):
                rpc_name = r_match.group(1).strip()
                in_type = r_match.group(2).strip()
                out_type = r_match.group(3).strip()
                actions.append(
                    ActionDef(
                        name=rpc_name,
                        in_params=[AttributeDef(name="request", type_name=in_type)],
                        out_params=[AttributeDef(name="response", type_name=out_type)]
                    )
                )
            pkg.part_defs.append(PartDef(name=s_name, doc="Protobuf Service", actions=actions))

        # Parse enums
        enum_pattern = re.compile(r'\benum\s+([a-zA-Z0-9_\-]+)\s*\{([^}]+)\}', re.MULTILINE | re.DOTALL)
        for match in enum_pattern.finditer(content):
            e_name = match.group(1)
            e_body = match.group(2)
            values = []
            val_pattern = re.compile(r'^\s*([a-zA-Z0-9_\-]+)\s*=\s*\d+\s*;', re.MULTILINE)
            for v_match in val_pattern.finditer(e_body):
                v_name = v_match.group(1).strip()
                values.append(AttributeDef(name=v_name, type_name="EnumConstant"))
            pkg.part_defs.append(PartDef(name=e_name, doc="Protobuf Enum", attributes=values))

        return pkg
