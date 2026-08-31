#!/usr/bin/env python3
"""
SysML v2 Universal Ingestion Engine (CLI Entrypoint)

Translates heterogeneous specification schemas (OMG IDL, AUTOSAR ARXML,
Protobuf, OpenAPI 3.0/3.1, and native SysML v2) into canonical SysML v2 textual models and
generates `.pipeline/schema-digest.json`.

Usage:
    python3 sysmlv2_ingest.py --schema <path> [--format <type>] [--out <output.sysml>] [--digest <path>]
"""

import argparse
import hashlib
import json
import os
import sys
from typing import Tuple, Dict, Any

# Ensure local script directory is on sys.path for relative imports
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

try:
    from sysmlv2_ast import SysMLPackage, SysMLParser
    from translators.idl_translator import IDLTranslator
    from translators.autosar_translator import AUTOSARTranslator
    from translators.protobuf_translator import ProtobufTranslator
    from translators.openapi_translator import OpenAPITranslator
except ImportError:
    from skills.spec_orchestrator.scripts.sysmlv2_ast import SysMLPackage, SysMLParser
    from skills.spec_orchestrator.scripts.translators.idl_translator import IDLTranslator
    from skills.spec_orchestrator.scripts.translators.autosar_translator import AUTOSARTranslator
    from skills.spec_orchestrator.scripts.translators.protobuf_translator import ProtobufTranslator
    from skills.spec_orchestrator.scripts.translators.openapi_translator import OpenAPITranslator


def detect_format(schema_path: str, content: str) -> str:
    ext = os.path.splitext(schema_path)[1].lower()
    if ext == ".sysml":
        return "sysml"
    elif ext == ".idl":
        return "idl"
    elif ext in (".arxml", ".xml"):
        return "autosar"
    elif ext == ".proto":
        return "protobuf"
    elif ext in (".json", ".yaml", ".yml"):
        return "openapi"

    # Content-based detection
    if "part def " in content or "package " in content or "capability def " in content or "requirement def " in content:
        return "sysml"
    elif "module " in content or "interface " in content or "struct " in content:
        return "idl"
    elif "<AUTOSAR" in content or "<AR-PACKAGE" in content:
        return "autosar"
    elif "syntax =" in content or "message " in content:
        return "protobuf"
    elif "openapi" in content or "swagger" in content or '"paths":' in content:
        return "openapi"

    return "idl"


def ingest_schema(
    schema_path: str,
    format_type: str = "auto",
    output_path: str = ".pipeline/schema.sysml",
    digest_path: str = ".pipeline/schema-digest.json"
) -> Tuple[SysMLPackage, Dict[str, Any]]:
    if not os.path.exists(schema_path):
        raise FileNotFoundError(f"Schema file not found: {schema_path}")

    with open(schema_path, "rb") as f:
        content_bytes = f.read()

    sha256_hash = hashlib.sha256(content_bytes).hexdigest()
    content_text = content_bytes.decode("utf-8", errors="replace")
    total_lines = len(content_text.splitlines())

    if not format_type or format_type == "auto":
        format_type = detect_format(schema_path, content_text)

    fmt = format_type.lower()
    file_basename = os.path.splitext(os.path.basename(schema_path))[0]

    if fmt in ("sysml", "sysmlv2", "sysml_v2"):
        pkg = SysMLParser.parse_text(content_text, default_name=file_basename)
    elif fmt in ("idl", "omg_idl"):
        translator = IDLTranslator()
        pkg = translator.translate(content_text, default_name=file_basename)
    elif fmt in ("autosar", "arxml"):
        translator = AUTOSARTranslator()
        pkg = translator.translate(content_text, default_name=file_basename)
    elif fmt in ("protobuf", "proto"):
        translator = ProtobufTranslator()
        pkg = translator.translate(content_text, default_name=file_basename)
    elif fmt in ("openapi", "json", "yaml"):
        translator = OpenAPITranslator()
        pkg = translator.translate(content_text, default_name=file_basename)
    else:
        translator = IDLTranslator()
        pkg = translator.translate(content_text, default_name=file_basename)

    sysml_text = pkg.to_sysml()

    # Write SysML v2 output
    os.makedirs(os.path.dirname(os.path.abspath(output_path)), exist_ok=True)
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(sysml_text)

    # Compute node counts & digest
    node_counts = pkg.node_counts()
    schema_nodes = pkg.get_all_node_names()

    digest_data = {
        "sha256": sha256_hash,
        "total_lines": total_lines,
        "node_counts": node_counts,
        "schema_nodes": schema_nodes
    }

    # Write digest JSON
    os.makedirs(os.path.dirname(os.path.abspath(digest_path)), exist_ok=True)
    with open(digest_path, "w", encoding="utf-8") as f:
        json.dump(digest_data, f, indent=2)

    print(f"[SysML v2 Ingestion] Successfully ingested {schema_path} ({format_type}) -> {output_path}")
    print(f"[SysML v2 Ingestion] Schema digest generated at {digest_path}")

    return pkg, digest_data


def main():
    parser = argparse.ArgumentParser(description="SysML v2 Universal Ingestion Engine CLI")
    parser.add_argument("schema_pos", nargs="?", default=None, help="Path to input schema file (positional)")
    parser.add_argument("--schema", required=False, default=None, help="Path to input schema file")
    parser.add_argument("--format", default="auto", help="Schema format (sysml, idl, autosar, protobuf, openapi, auto)")
    parser.add_argument("--out", default=".pipeline/schema.sysml", help="Path to output .sysml file")
    parser.add_argument("--digest", default=".pipeline/schema-digest.json", help="Path to output digest JSON")
    args = parser.parse_args()

    schema_path = args.schema or args.schema_pos
    if not schema_path:
        parser.error("Must specify a schema file via positional argument or --schema")

    ingest_schema(
        schema_path=schema_path,
        format_type=args.format,
        output_path=args.out,
        digest_path=args.digest
    )


if __name__ == "__main__":
    main()
