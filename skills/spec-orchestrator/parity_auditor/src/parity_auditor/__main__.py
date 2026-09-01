"""
``python3 -m parity_auditor`` entry point.

Mirrors the ``parity-auditor`` console script declared in ``pyproject.toml``
(``parity_auditor.cli:main``) exactly: zero-argument invocation of
``cli.main()``, which owns environment sanitisation, exception handling, and
process exit codes.  Adding this module makes the ``python3 -m parity_auditor``
gate mandated by ``skills/feature-driven-implementation/SKILL.md`` Step 4
executable, since Python cannot run a package without it.
"""

from .cli import main

if __name__ == "__main__":
    main()
