#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Fail-closed regression test for the SysML v2 AST loader (refs #76).

Defect: importing ``parity_auditor.validators`` without
``skills/spec-orchestrator/scripts`` pre-seeded on sys.path made the three-tier
import cascade dead-end, inserted a nonexistent ``parity_auditor/scripts`` dir
into sys.path, and silently bound ``SysMLParser`` (and sibling AST classes) to
None. Checks 18-23 then ran against an empty model and reported a clean exit 0.

This test spawns a pristine interpreter whose PYTHONPATH contains ONLY the
parity_auditor src directory and demands one of two acceptable outcomes:

  * exit 0 with every validator module carrying a real (non-None) SysMLParser;
  * a clean non-zero exit whose diagnostic names the missing module
    (``sysmlv2_ast``) via an ImportError.

Exit 0 with a half-loaded validator silently carrying ``SysMLParser is None``
is the defective behavior this test exists to detect, and fails the assertion.
"""

import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
PARITY_SRC = os.path.abspath(os.path.join(HERE, "..", "src"))

# Executed in a pristine subprocess. sys.path[0] is the cwd, which the runner
# points at an empty temp dir so no repository layout can leak in through ''.
_SUBPROCESS_CODE = """
import os
import sys

sys_path_before = list(sys.path)

from parity_auditor.validators import cardinality_validator, uml, icd_completeness_validator

for name, module in (
    ("cardinality_validator", cardinality_validator),
    ("uml", uml),
    ("icd_completeness_validator", icd_completeness_validator),
):
    if getattr(module, "SysMLParser", None) is None:
        sys.exit("FAIL_HALF_LOADED: %s.SysMLParser is None" % name)

inserted = [p for p in sys.path if p not in sys_path_before]
bad_paths = [p for p in inserted if not (p == "" or os.path.isdir(p))]
if bad_paths:
    sys.exit("FAIL_POLLUTED_SYSPATH: %r" % bad_paths)

print("PARSER_OK")
"""


def _env_with_only_src_path():
    env = os.environ.copy()
    env["PYTHONPATH"] = PARITY_SRC
    return env


def test_sysml_loader_fail_closed_isolated_import():
    """Importing the validators with only parity src on sys.path must never
    yield a half-loaded module; it resolves the AST module or raises a clean
    ImportError naming ``sysmlv2_ast``."""
    with tempfile.TemporaryDirectory() as tmp:
        proc = subprocess.run(
            [sys.executable, "-c", _SUBPROCESS_CODE],
            capture_output=True,
            text=True,
            cwd=tmp,
            env=_env_with_only_src_path(),
        )
    combined = proc.stdout + proc.stderr

    if proc.returncode == 0:
        assert "PARSER_OK" in proc.stdout, (
            "subprocess exited 0 but did not confirm a resolved parser:\n%s" % combined
        )
        assert "FAIL_HALF_LOADED" not in combined
        assert "FAIL_POLLUTED_SYSPATH" not in combined
        return

    # Clean-failure branch: the diagnostic must name the missing module and be
    # an ImportError, never a bare AssemblyError trace from a half-loaded module.
    assert "sysmlv2_ast" in combined, (
        "subprocess failed without naming the missing SysML AST module:\n%s" % combined
    )
    assert "ImportError" in combined, (
        "subprocess failed without a clean ImportError diagnostic:\n%s" % combined
    )
