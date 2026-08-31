"""
Validator that programmatically checks subagent dispatch prompts for mandatory
governance preamble markers (constitution, Zero-Mocking, 3-Layer DoD, RED-GREEN-REFACTOR,
build/test commands, view_file mandate).
"""

import os
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

MANDATORY_PREAMBLE_MARKERS = [
    "Adopt the feature-driven-implementation skill",
    "view_file on skills/feature-driven-implementation/SKILL.md as step 1",
    "Section 1.9 Zero-Mocking Live Persistence Mandate",
    "3-Layer Definition of Done (DoD)",
    "RED-GREEN-REFACTOR",
    "flutter analyze (0 issues), flutter test (all pass)",
]


def validate_dispatch_prompt(prompt_text: str) -> List[str]:
    """
    Programmatically checks subagent dispatch prompt for mandatory preamble markers.
    Returns a list of missing markers. Empty list indicates clean compliance.
    """
    if not prompt_text:
        return list(MANDATORY_PREAMBLE_MARKERS)
    missing = [marker for marker in MANDATORY_PREAMBLE_MARKERS if marker not in prompt_text]
    return missing


class DispatchPreambleValidator(IValidator):
    """
    Validator enforcing that all subagent dispatch prompts carry mandatory governance preambles,
    and verifying skill files document prompt preamble rules and subagent failure protocol.
    """

    def validate_prompt(self, prompt_text: str) -> List[str]:
        """Convenience helper to validate prompt text directly."""
        return validate_dispatch_prompt(prompt_text)

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        errors: List[Finding] = []

        prompt_text = kwargs.get("prompt_text")
        if prompt_text is not None:
            missing = validate_dispatch_prompt(prompt_text)
            for marker in missing:
                errors.append(
                    Finding(
                        rule_id="subagent-dispatch-preamble-missing",
                        description=f"Subagent dispatch prompt is missing mandatory governance marker: '{marker}'",
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
                            description="skills/feature-driven-implementation/SKILL.md missing mandatory subagent governance preamble section.",
                            location=skill_rel,
                        )
                    )
                if "two consecutive failures" not in content.lower():
                    errors.append(
                        Finding(
                            rule_id="subagent-failure-protocol-missing",
                            description="skills/feature-driven-implementation/SKILL.md missing mandatory subagent failure protocol documentation.",
                            location=skill_rel,
                        )
                    )
            except Exception as e:
                errors.append(
                    Finding(
                        rule_id="skill-read-error",
                        description=f"Failed to read skill file {skill_rel}: {e}",
                        location=skill_rel,
                    )
                )

        return errors
