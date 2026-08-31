"""The one specification-title normaliser, shared by every gate that protects it.

`reconcile_backlog.py` addresses tracker issues by normalised title whenever a spec has
no canonical `issue_id` yet (#314/#316 demoted that path to a warning-only fallback but
did not remove it). Two validators exist to stop that key colliding — the #318 title
uniqueness gate and `SyncValidator` — and each carried its own copy of the normaliser.
They had drifted: `sync_validator`'s copy lacked the guard that keeps the original title
when prefix-stripping would empty it, and additionally folded `_` to a space. A gate that
collides in a different space from the consumer it protects is worse than no gate — the
looser copy misses collisions the reconciler will make, the stricter one invents
collisions it will not, and `sync_validator`'s copy managed both at once.

**Why the reconciler is the definition site, and this module only re-exports.**

The dependency has to point somewhere, and the two ends are not symmetric:

* `reconcile_backlog.py` is a standalone script. `.pipeline/upstream/pipeline-tooling.md`
  § *Platform & Stack* says `skills/*/scripts/` SHOULD be standard-library only, and
  `AGENTS.md` § *Backlog Reconciliation Mandate* requires it to run before every merge —
  including in a downstream repository that has never installed this package. Making it
  import `parity_auditor` would make a mandated pre-merge step fail on a missing
  third-party install.
* `parity_auditor` is an installed package whose gates only ever run against a checkout
  of the repository they audit, and whose own tests already reach the scripts directory
  by path. It can carry the locator; the script cannot carry the dependency.

It is also the correct direction on the merits. The reconciler is the *consumer* of the
key: its normalisation defines the collision space, and the gates exist to describe that
space exactly. If the two disagree, the reconciler is right by definition.

The re-export is by reference, not by copy, so the identity is checked by
`tests/test_normalize_title_unified_issue318.py` and cannot drift again.

The locator raises rather than substituting a local implementation. A fallback copy is
the defect this module exists to remove: a gate silently normalising differently from
the reconciler is precisely the failure, and it is much better caught at import.
"""

import importlib.util
import os
import sys

_SCRIPT_NAME = "reconcile_backlog.py"

# From src/parity_auditor/utils/ up to skills/spec-orchestrator/, then into scripts/.
_PACKAGE_RELATIVE_SCRIPT = os.path.abspath(
    os.path.join(os.path.dirname(__file__), "..", "..", "..", "..", "scripts", _SCRIPT_NAME)
)


def _load_reconciler():
    """The `reconcile_backlog` module, however it is reachable from here.

    Importing it is inert: the module defines functions and constants only, and guards
    its entry point behind `if __name__ == "__main__"`.
    """
    module = sys.modules.get("reconcile_backlog")
    if module is not None:
        return module
    try:  # already on sys.path — how this package's own tests reach it
        import reconcile_backlog  # noqa: F401

        return sys.modules["reconcile_backlog"]
    except ImportError:
        pass

    if os.path.isfile(_PACKAGE_RELATIVE_SCRIPT):
        spec = importlib.util.spec_from_file_location(
            "reconcile_backlog", _PACKAGE_RELATIVE_SCRIPT
        )
        if spec and spec.loader:
            module = importlib.util.module_from_spec(spec)
            sys.modules["reconcile_backlog"] = module
            spec.loader.exec_module(module)
            return module

    raise ImportError(
        f"Cannot locate {_SCRIPT_NAME}, which defines the specification-title "
        "normaliser these gates share. Looked on sys.path and at "
        f"{_PACKAGE_RELATIVE_SCRIPT}. Falling back to a local copy is deliberately not "
        "done: a gate that normalises differently from the reconciler it protects "
        "reports collisions that do not exist and misses the ones that do."
    )


# Bound by reference, never re-implemented. `normalize_spec_title is
# reconcile_backlog.normalize_title` is the property the regression test asserts, and it
# is what makes a future change to the reconciler reach both gates automatically.
normalize_spec_title = _load_reconciler().normalize_title

__all__ = ["normalize_spec_title"]
