"""
Validator that programmatically checks subagent dispatch prompts for the mandatory
governance preamble. The required clause set is derived at runtime, verbatim, from
rules/subagent-dispatch-standards.md rather than hardcoded, so a payload that
paraphrases or weakens a requirement is distinguishable from one that carries it
whole. Explicit waiver/negation language fails regardless of what else is present.
"""

import os
import re
from typing import List, Optional, Tuple

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

_EXPLICIT_WAIVER_PHRASES = (
    "is hereby waived",
    "hereby waived",
    "is waived",
    "strictly optional",
    "at your discretion",
    "requirement does not apply",
    "is optional and may be skipped",
)


def _default_mandate_path() -> Optional[str]:
    """Locate rules/subagent-dispatch-standards.md by walking up from this module."""
    current = os.path.dirname(os.path.abspath(__file__))
    while True:
        candidate = os.path.join(current, "rules", "subagent-dispatch-standards.md")
        if os.path.isfile(candidate):
            return candidate
        parent = os.path.dirname(current)
        if parent == current:
            return None
        current = parent


def load_mandate_requirements(
    rules_path: Optional[str] = None,
) -> Tuple[Optional[List[str]], Optional[str]]:
    """
    Parse the six pre-flight checklist Requirement cells verbatim from the rules corpus.

    Returns:
        (requirements, error) where requirements is the list of verbatim Requirement
        strings, or None with a fail-closed error message when the corpus file is
        unreadable or contains no parseable Requirements at gate time.
    """
    rules_path = rules_path or _default_mandate_path()
    if not rules_path:
        return None, "mandatory rules corpus not found: rules/subagent-dispatch-standards.md"
    try:
        with open(rules_path, "r", encoding="utf-8") as f:
            rules_text = f.read()
    except OSError as exc:
        return None, f"mandatory rules corpus unreadable at gate time: {rules_path} ({exc})"
    requirements = []
    for line in rules_text.splitlines():
        if re.match(r"^\|\s*\d+\.\s", line):
            cells = [c.strip() for c in line.split("|")]
            if len(cells) > 3 and cells[2]:
                requirements.append(cells[2])
    if not requirements:
        return None, f"mandatory rules corpus contains no parseable pre-flight Requirements: {rules_path}"
    return requirements, None


MANDATORY_PREAMBLE_MARKERS = load_mandate_requirements()[0] or []


def validate_dispatch_prompt(
    prompt_text: str, rules_path: Optional[str] = None
) -> List[str]:
    """
    Programmatically checks subagent dispatch prompt against the pre-flight requirements.

    Requirement strings are read at runtime from rules/subagent-dispatch-standards.md
    (whitespace-normalized verbatim containment), so no new governance text is
    hardcoded here. Explicit waiver or negation language is rejected even when every
    requirement string is present. Returns a list of violations; an empty list
    indicates clean compliance.
    """
    requirements, mandate_error = load_mandate_requirements(rules_path)
    if mandate_error:
        return [mandate_error]
    if not prompt_text:
        return list(requirements)
    normalized_prompt = re.sub(r"\s+", " ", prompt_text).strip()
    violations = [
        "Subagent dispatch prompt is missing mandatory pre-flight requirement verbatim "
        f"from rules/subagent-dispatch-standards.md: '{requirement}'"
        for requirement in requirements
        if re.sub(r"\s+", " ", requirement).strip() not in normalized_prompt
    ]
    lowered = normalized_prompt.lower()
    for phrase in _EXPLICIT_WAIVER_PHRASES:
        if phrase in lowered:
            violations.append(
                f"Subagent dispatch prompt contains waiver/negation language: '{phrase}'"
            )
    return violations


class DispatchPreambleValidator(IValidator):
    """
    Validator enforcing that all subagent dispatch prompts carry the mandatory governance
    preamble requirements from rules/subagent-dispatch-standards.md, and verifying skill
    files document prompt preamble rules and subagent failure protocol.
    """

    def validate_prompt(self, prompt_text: str) -> List[str]:
        """Convenience helper to validate prompt text directly."""
        return validate_dispatch_prompt(prompt_text)

    def _prompt_rules_path(self, repo: WorkspaceRepository) -> Optional[str]:
        workspace_rules = os.path.join(
            repo.workspace_dir, "rules", "subagent-dispatch-standards.md"
        )
        if os.path.isfile(workspace_rules):
            return workspace_rules
        return None

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        errors: List[Finding] = []

        prompt_text = kwargs.get("prompt_text")
        if prompt_text is not None:
            violations = validate_dispatch_prompt(
                prompt_text, rules_path=self._prompt_rules_path(repo)
            )
            for violation in violations:
                rule_id = (
                    "subagent-dispatch-preamble-negation"
                    if "waiver/negation language" in violation
                    else "subagent-dispatch-preamble-missing"
                )
                errors.append(
                    Finding(
                        rule_id=rule_id,
                        message=violation,
                        location="subagent_dispatch_prompt",
                    )
                )

        workspace_dir = repo.workspace_dir
        skill_rel = os.path.join("skills", "feature-driven-implementation", "SKILL.md")
        skill_abs = os.path.join(workspace_dir, skill_rel)

        if not os.path.isfile(skill_abs):
            skill_rel = os.path.join(".agents", "skills", "feature-driven-implementation", "SKILL.md")
            skill_abs = os.path.join(workspace_dir, skill_rel)

        if os.path.isfile(skill_abs):
            try:
                with open(skill_abs, "r", encoding="utf-8") as f:
                    content = f.read()

                if "governance preamble" not in content.lower():
                    errors.append(
                        Finding(
                            rule_id="dispatch-preamble-documentation-missing",
                            message="skills/feature-driven-implementation/SKILL.md missing mandatory subagent governance preamble section.",
                            location=skill_rel,
                        )
                    )
                if "two consecutive failures" not in content.lower():
                    errors.append(
                        Finding(
                            rule_id="subagent-failure-protocol-missing",
                            message="skills/feature-driven-implementation/SKILL.md missing mandatory subagent failure protocol documentation.",
                            location=skill_rel,
                        )
                    )
            except Exception as e:
                errors.append(
                    Finding(
                        rule_id="skill-read-error",
                        message=f"Failed to read skill file {skill_rel}: {e}",
                        location=skill_rel,
                    )
                )

        return errors
