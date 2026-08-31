#!/usr/bin/env python3
"""
OpenAPI 3.0/3.1 to SysML v2 Translator
Parses OpenAPI schemas (JSON/YAML), paths, parameters, components into SysML v2 AST.
"""

import json
import re
from typing import List, Optional, Dict, Any
import sys
import os

try:
    from sysmlv2_ast import SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef
except ImportError:
    from skills.spec_orchestrator.scripts.sysmlv2_ast import (
        SysMLPackage, PartDef, PortDef, AttributeDef, ActionDef
    )


class OpenAPITranslator:
    def __init__(self):
        pass

    def _parse_raw(self, content: str) -> Dict[str, Any]:
        content_clean = content.strip()
        if content_clean.startswith('{') or content_clean.startswith('['):
            try:
                return json.loads(content_clean)
            except Exception:
                pass
        
        # Simple fallback parsing for basic JSON-like or structured dictionary
        try:
            import yaml
            return yaml.safe_load(content)
        except Exception:
            pass

        # Manual minimal fallback for title, paths, schemas if yaml library not installed
        data = {"info": {"title": "OpenAPI_Package"}, "paths": {}, "components": {"schemas": {}}}
        title_match = re.search(r'title:\s*["\']?([^"\'\n]+)["\']?', content)
        if title_match:
            data["info"]["title"] = title_match.group(1).strip()
        return data

    def translate(self, content: str, default_name: str = "OpenAPI_Package") -> SysMLPackage:
        spec = self._parse_raw(content)

        pkg_name = spec.get("info", {}).get("title", default_name)
        # Sanitize package name
        pkg_name = re.sub(r'[^a-zA-Z0-9_]', '_', pkg_name).strip('_') or default_name

        pkg = SysMLPackage(name=pkg_name, doc="Translated from OpenAPI 3.0/3.1 schema")

        # Parse components.schemas
        schemas = spec.get("components", {}).get("schemas", {})
        for schema_name, schema_obj in schemas.items():
            if not isinstance(schema_obj, dict):
                continue
            clean_s_name = re.sub(r'[^a-zA-Z0-9_]', '_', schema_name)
            attrs = []
            props = schema_obj.get("properties", {})
            if isinstance(props, dict):
                for p_name, p_obj in props.items():
                    p_type = "String"
                    if isinstance(p_obj, dict):
                        p_type = p_obj.get("type", p_obj.get("$ref", "String"))
                    attrs.append(AttributeDef(name=p_name, type_name=str(p_type)))
            pkg.part_defs.append(PartDef(name=clean_s_name, doc="OpenAPI Schema", attributes=attrs))

        # Parse paths / operations
        paths = spec.get("paths", {})
        actions = []
        if isinstance(paths, dict):
            for path_str, path_obj in paths.items():
                if not isinstance(path_obj, dict):
                    continue
                for method in ("get", "post", "put", "delete", "patch", "options", "head"):
                    if method in path_obj:
                        op_obj = path_obj[method]
                        if isinstance(op_obj, dict):
                            op_id = op_obj.get("operationId")
                            if not op_id:
                                clean_path = re.sub(r'[^a-zA-Z0-9]', '_', path_str).strip('_')
                                op_id = f"{method}_{clean_path}"
                            summary = op_obj.get("summary", f"{method.upper()} {path_str}")
                            actions.append(ActionDef(name=op_id, doc=summary))

        if actions:
            pkg.part_defs.append(PartDef(name=f"{pkg_name}_Endpoints", doc="OpenAPI Operations", actions=actions))

        return pkg
