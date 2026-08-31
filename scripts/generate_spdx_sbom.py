#!/usr/bin/env python3
"""SPDX 2.3 Software Bill of Materials (SBOM) JSON generator.

Reads ``app_flutter/pubspec.yaml`` for Dart/Flutter dependencies and
``requirements.txt`` for Python dependencies, then emits a single SPDX 2.3
JSON document to *stdout* (or to the file given by ``--output``).

Realises: sbom.md / GenerateSpdxSbom

Usage::

    python3 scripts/generate_spdx_sbom.py                     # stdout
    python3 scripts/generate_spdx_sbom.py -o sbom.spdx.json   # file
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import yaml


# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_SPDX_VERSION = "SPDX-2.3"
_DATA_LICENSE = "CC0-1.0"
_SPDX_ID_DOC = "SPDXRef-DOCUMENT"


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _spdx_ref(name: str) -> str:
    """Return a valid SPDX identifier from *name*.

    Replaces characters that are illegal in SPDX identifiers (anything that
    is not ``[a-zA-Z0-9.-]``) with a hyphen.
    """
    sanitised = re.sub(r"[^a-zA-Z0-9.\-]", "-", name)
    return f"SPDXRef-{sanitised}"


def _parse_pubspec(path: Path) -> list[dict[str, Any]]:
    """Parse ``pubspec.yaml`` and return a list of SPDX package dicts.

    Only concrete (non-SDK) dependencies and dev_dependencies are included.
    SDK entries (``flutter``, ``flutter_test``, ``integration_test``) are
    skipped because they are part of the Flutter SDK, not separate packages.

    Args:
        path: Absolute or relative path to the ``pubspec.yaml`` file.

    Returns:
        A list of SPDX package dictionaries, one per dependency.

    Raises:
        FileNotFoundError: If *path* does not exist.
        yaml.YAMLError: If the YAML content is malformed.
    """
    content = yaml.safe_load(path.read_text(encoding="utf-8"))
    packages: list[dict[str, Any]] = []

    for section in ("dependencies", "dev_dependencies"):
        deps = content.get(section, {}) or {}
        for name, version_spec in deps.items():
            # Skip SDK dependencies (value is a map with 'sdk' key).
            if isinstance(version_spec, dict) and "sdk" in version_spec:
                continue
            version_str = str(version_spec) if version_spec else "NOASSERTION"
            pkg = {
                "SPDXID": _spdx_ref(name),
                "name": name,
                "versionInfo": version_str,
                "downloadLocation": f"https://pub.dev/packages/{name}",
                "filesAnalyzed": False,
                "supplier": "NOASSERTION",
                "primaryPackagePurpose": "LIBRARY",
            }
            packages.append(pkg)

    return packages


def _parse_requirements(path: Path) -> list[dict[str, Any]]:
    """Parse ``requirements.txt`` and return a list of SPDX package dicts.

    Lines starting with ``#`` and blank lines are ignored.  Version
    specifiers (``>=``, ``==``, etc.) are preserved as-is in ``versionInfo``.

    Args:
        path: Absolute or relative path to ``requirements.txt``.

    Returns:
        A list of SPDX package dictionaries, one per dependency.

    Raises:
        FileNotFoundError: If *path* does not exist.
    """
    packages: list[dict[str, Any]] = []
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        # Split on the first version specifier symbol.
        match = re.match(r"([A-Za-z0-9_\-]+)(.*)", line)
        if not match:
            continue
        name = match.group(1)
        version_str = match.group(2).strip() or "NOASSERTION"
        pkg = {
            "SPDXID": _spdx_ref(f"py-{name}"),
            "name": name,
            "versionInfo": version_str,
            "downloadLocation": f"https://pypi.org/project/{name}/",
            "filesAnalyzed": False,
            "supplier": "NOASSERTION",
            "primaryPackagePurpose": "LIBRARY",
        }
        packages.append(pkg)
    return packages


def generate_sbom(
    pubspec_path: Path,
    requirements_path: Path,
    *,
    document_name: str = "DEAP-spec-core",
    document_namespace: str | None = None,
) -> dict[str, Any]:
    """Build a complete SPDX 2.3 JSON document dictionary.

    Args:
        pubspec_path: Path to ``pubspec.yaml``.
        requirements_path: Path to ``requirements.txt``.
        document_name: Human-readable name for the SPDX document.
        document_namespace: Unique URI namespace.  Auto-generated when
            *None*.

    Returns:
        A dictionary conforming to SPDX 2.3 JSON schema.
    """
    if document_namespace is None:
        document_namespace = (
            f"https://spdx.org/spdxdocs/{document_name}-{uuid.uuid4()}"
        )

    packages: list[dict[str, Any]] = []
    if pubspec_path.exists():
        packages.extend(_parse_pubspec(pubspec_path))
    if requirements_path.exists():
        packages.extend(_parse_requirements(requirements_path))

    relationships: list[dict[str, str]] = []
    for pkg in packages:
        relationships.append(
            {
                "spdxElementId": _SPDX_ID_DOC,
                "relatedSpdxElement": pkg["SPDXID"],
                "relationshipType": "DESCRIBES",
            }
        )

    now = datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    sbom: dict[str, Any] = {
        "spdxVersion": _SPDX_VERSION,
        "dataLicense": _DATA_LICENSE,
        "SPDXID": _SPDX_ID_DOC,
        "name": document_name,
        "documentNamespace": document_namespace,
        "creationInfo": {
            "created": now,
            "creators": ["Tool: generate_spdx_sbom.py"],
            "licenseListVersion": "3.19",
        },
        "packages": packages,
        "relationships": relationships,
    }
    return sbom


# ---------------------------------------------------------------------------
# CLI entry-point
# ---------------------------------------------------------------------------


def main(argv: list[str] | None = None) -> None:
    """CLI entry-point for SBOM generation.

    Args:
        argv: Command-line arguments (defaults to ``sys.argv[1:]``).
    """
    parser = argparse.ArgumentParser(
        description="Generate SPDX 2.3 SBOM JSON from project manifests.",
    )
    parser.add_argument(
        "--pubspec",
        type=Path,
        default=Path("app_flutter/pubspec.yaml"),
        help="Path to pubspec.yaml (default: app_flutter/pubspec.yaml).",
    )
    parser.add_argument(
        "--requirements",
        type=Path,
        default=Path("requirements.txt"),
        help="Path to requirements.txt (default: requirements.txt).",
    )
    parser.add_argument(
        "-o",
        "--output",
        type=Path,
        default=None,
        help="Output file path.  Prints to stdout when omitted.",
    )
    parser.add_argument(
        "--name",
        default="DEAP-spec-core",
        help="SPDX document name.",
    )
    args = parser.parse_args(argv)

    sbom = generate_sbom(
        pubspec_path=args.pubspec,
        requirements_path=args.requirements,
        document_name=args.name,
    )
    json_str = json.dumps(sbom, indent=2) + "\n"

    if args.output:
        args.output.write_text(json_str, encoding="utf-8")
    else:
        sys.stdout.write(json_str)


if __name__ == "__main__":
    main()
