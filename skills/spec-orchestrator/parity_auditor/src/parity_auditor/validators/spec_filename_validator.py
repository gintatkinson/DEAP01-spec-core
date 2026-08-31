"""Enforces the documented specification filename convention (issue #300).

The convention was already documented and enforced by nothing — orphan documentation,
which is #289's defect class:

* ``spec-usecase-engineering/SKILL.md:62`` — ``uc-[XX]-[name].md``, "zero-padded,
  dash-separated"
* ``spec-user-story-engineering/SKILL.md:73`` — ``us-[XX]-[name].md``, likewise
* ``schema-specification-engineering/SKILL.md:39,89`` — ``epic-01-name.md``,
  ``feat-01-name.md``

Three checks per backlog directory:

1. **Ordinal uniqueness.** Two files claiming the same number make every reference to
   that number ambiguous. ``reconcile_backlog.py`` and Epic checklists address specs by
   ordinal, so a collision is a correctness problem, not a tidiness one.
2. **Format conformance** against ``<prefix>-<ordinal>-<kebab-name>.md``.
3. **Padding consistency** within a directory. The rule is internal consistency rather
   than a fixed width: a directory padded uniformly to three digits is fine, mixing two
   and three is not, because it breaks lexical sort order.
"""

import os
import re
from typing import Dict, List

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

# Directory key in backlog_directories -> the prefix its files must carry.
DIRECTORY_PREFIXES: Dict[str, str] = {
    "features": "feat",
    "epics": "epic",
    "user_stories": "us",
    "use_cases": "uc",
}

# <prefix>-<digits>-<kebab-name>.md  -- lowercase, dash-separated, at least one
# descriptive segment after the ordinal.
_NAME_RE = re.compile(r"^(?P<prefix>[a-z]+)-(?P<ordinal>\d+)-(?P<name>[a-z0-9]+(?:-[a-z0-9]+)*)\.md$")


class SpecFilenameValidator(IValidator):
    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        rules = repo.get_codebase_rules()
        backlog_dirs = rules.backlog_directories
        errors: List[str] = []

        for dir_key, expected_prefix in DIRECTORY_PREFIXES.items():
            rel = getattr(backlog_dirs, dir_key, None)
            if not rel:
                continue
            target = os.path.join(repo.workspace_dir, rel)
            if not os.path.isdir(target):
                continue

            names = sorted(
                n for n in os.listdir(target)
                if n.endswith(".md") and not n.startswith(".")
            )
            if not names:
                continue

            by_ordinal: Dict[int, List[str]] = {}
            widths: Dict[int, List[str]] = {}

            for name in names:
                match = _NAME_RE.match(name)
                if not match:
                    errors.append(Finding(
                        "spec-filename-format",
                        f"{rel}/{name}: filename does not match the documented convention "
                        f"'{expected_prefix}-<zero-padded-ordinal>-<kebab-name>.md'. "
                        f"Names must be lowercase and dash-separated."
                    , location=rel))
                    continue

                if match.group("prefix") != expected_prefix:
                    errors.append(Finding(
                        "spec-filename-directory-prefix",
                        f"{rel}/{name}: directory prefix mismatch "
                        f"- found '{match.group('prefix')}-', but files in {rel} must "
                        f"use the '{expected_prefix}-' prefix."
                    , location=rel))
                    continue

                ordinal_text = match.group("ordinal")
                by_ordinal.setdefault(int(ordinal_text), []).append(name)
                widths.setdefault(len(ordinal_text), []).append(name)

            for ordinal, colliding in sorted(by_ordinal.items()):
                if len(colliding) > 1:
                    errors.append(Finding(
                        "spec-filename-ordinal-uniqueness",
                        f"{rel}: duplicate ordinal "
                        f"- {ordinal} is claimed by {len(colliding)} files "
                        f"({', '.join(colliding)}). Ordinals must be "
                        f"unique, because backlog reconciliation and Epic checklists "
                        f"reference specifications by number."
                    , location=rel))

            if len(widths) > 1:
                summary = "; ".join(
                    f"{width} digit(s): {', '.join(sorted(files))}"
                    for width, files in sorted(widths.items())
                )
                errors.append(Finding(
                        "spec-filename-padding-consistency",
                    f"{rel}: inconsistent ordinal padding width across the directory "
                    f"({summary}). Pick one width and apply it uniformly, otherwise "
                    f"lexical ordering does not match numeric ordering."
                , location=rel))

        return errors
