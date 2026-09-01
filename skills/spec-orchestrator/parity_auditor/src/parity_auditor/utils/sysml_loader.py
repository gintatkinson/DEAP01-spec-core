#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Fail-closed loader for the SysML v2 AST module (``sysmlv2_ast.py``).

The AST module is a specification-tooling script that lives OUTSIDE the
``parity_auditor`` package, at ``skills/spec-orchestrator/scripts/sysmlv2_ast.py``
relative to the repository root. The historical three-tier import cascade
trapped its final ``ImportError`` and silently bound every AST class to None,
which made the SysML coverage gates (Checks 18-23) report a clean pass over an
empty model while also inserting a nonexistent directory into sys.path
(refs #76).

This loader never binds None and never mutates sys.path. It either returns the
fully-executed module or raises ``ImportError`` with a diagnostic naming the
missing file, so any validator that cannot obtain a real SysMLParser fails at
import time instead of degrading silently.

Resolution order:

1. Fast path: if ``sysmlv2_ast`` is already in ``sys.modules`` (e.g. a caller
   pre-seeded the scripts directory), return it unchanged.
2. Otherwise walk upward from ``__file__`` until
   ``<repo>/skills/spec-orchestrator/scripts/sysmlv2_ast.py`` exists and load
   it directly via ``importlib.util.spec_from_file_location`` — robust to
   install layouts and independent of any sys.path seeding.
3. Otherwise raise ImportError (fail closed).
"""

import importlib.util
import os
import sys

_SYSML_MODULE_NAME = "sysmlv2_ast"
_SYSML_FILE_NAME = "sysmlv2_ast.py"


def _find_scripts_dir(start_dir):
    """Walk upward from ``start_dir`` to the repo dir containing the real
    ``skills/spec-orchestrator/scripts/_SYSML_FILE_NAME``, or return None."""
    cur = os.path.abspath(start_dir)
    while True:
        candidate = os.path.join(cur, "skills", "spec-orchestrator", "scripts", _SYSML_FILE_NAME)
        if os.path.isfile(candidate):
            return os.path.dirname(candidate)
        parent = os.path.dirname(cur)
        if parent == cur:
            return None
        cur = parent


def load_sysml_ast_module():
    """Return the executed ``sysmlv2_ast`` module, or raise ImportError.

    Never returns None, and never falls back silently after a failure.
    """
    existing = sys.modules.get(_SYSML_MODULE_NAME)
    if existing is not None:
        return existing

    scripts_dir = _find_scripts_dir(__file__)
    if scripts_dir is None:
        raise ImportError(
            "SysML v2 AST dependency missing: could not resolve module file 'sysmlv2_ast.py' "
            "under '<repo>/skills/spec-orchestrator/scripts/'. The SysML coverage gates "
            "(Checks 18-23) refuse to run with SysMLParser unavailable."
        )

    module_path = os.path.join(scripts_dir, _SYSML_FILE_NAME)
    spec = importlib.util.spec_from_file_location(_SYSML_MODULE_NAME, module_path)
    if spec is None or spec.loader is None:
        raise ImportError(
            "SysML v2 AST dependency unloadable: '%s' exists but cannot be imported. "
            "The SysML coverage gates (Checks 18-23) refuse to run with SysMLParser "
            "unavailable." % module_path
        )

    module = importlib.util.module_from_spec(spec)
    sys.modules[_SYSML_MODULE_NAME] = module
    try:
        spec.loader.exec_module(module)
    except Exception as exc:
        sys.modules.pop(_SYSML_MODULE_NAME, None)
        raise ImportError(
            "Failed to execute SysML v2 AST module '%s': %s" % (module_path, exc)
        ) from exc
    return module


def load_sysml_ast_members(member_names):
    """Return the loaded ``sysmlv2_ast`` module for ``member_names``.

    Fails closed if any requested member does not exist on the module, so a
    validator can never continue with a silently-missing class.
    """
    module = load_sysml_ast_module()
    missing = [name for name in member_names if not hasattr(module, name)]
    if missing:
        raise ImportError(
            "SysML v2 AST module '%s' is missing required members: %s. "
            "The SysML coverage gates refuse to run without them." % (
                getattr(module, "__file__", _SYSML_FILE_NAME), ", ".join(sorted(missing)))
        )
    return module
