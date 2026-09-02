#!/usr/bin/env python3
"""
Transcript Audit Process Gate (`tests/test_prompt_discipline.py`).

Audits session transcripts (transcript.jsonl or mock transcript streams) for strict
prompt discipline:
1. Extracts prompt text of every subagent in `Subagents` for all `invoke_subagent` tool calls.
2. Validates extracted prompts against `lint_prompt_text()` and `validate_subagent_preflight()`
   from `scripts/lint_subagent_prompt.py`.
3. Asserts that non-compliant dispatches (summarized prompts, missing `view_file` on `SKILL.md`,
   missing repository classification, leading steering, forbidden issue close commands, or
   scope violations) raise assertion failures.
4. Ensures clean transcripts, multi-subagent turns, and empty transcripts are audited accurately.
"""

import io
import json
import os
import re
import sys
import tempfile
import unittest
from typing import Any, Dict, List, Optional, Tuple, Union

# Ensure repository root is on sys.path
TEST_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_ROOT = os.path.abspath(os.path.join(TEST_DIR, ".."))
if PROJECT_ROOT not in sys.path:
    sys.path.insert(0, PROJECT_ROOT)

from scripts.lint_subagent_prompt import lint_prompt_text, validate_subagent_preflight


def _corpus_requirements() -> List[str]:
    """
    Parse the six Requirement cells of the pre-flight checklist body verbatim
    from rules/subagent-dispatch-standards.md (mirrors the corpora pattern in
    tests/test_prompt_linter.py).
    """
    rules_path = os.path.join(PROJECT_ROOT, "rules", "subagent-dispatch-standards.md")
    with open(rules_path, "r", encoding="utf-8") as f:
        lines = f.read().splitlines()
    requirements = []
    for line in lines:
        if re.match(r"^\|\s*\d+\.\s", line):
            cells = [c.strip() for c in line.split("|")]
            if len(cells) > 3 and cells[2]:
                requirements.append(cells[2])
    return requirements


def _extract_prompts_from_value(value: Any) -> List[str]:
    """
    Recursively inspects arbitrary JSON-decoded data structures to locate all
    `invoke_subagent` tool calls and extract all subagent prompt strings.
    """
    prompts: List[str] = []

    if isinstance(value, dict):
        # Determine tool name if this object represents a tool call / tool invocation
        tool_name = (
            value.get("name")
            or value.get("tool_name")
            or value.get("tool")
            or value.get("action")
            or value.get("toolName")
        )

        if tool_name == "invoke_subagent":
            # Extract arguments / parameters dictionary
            args = (
                value.get("parameters")
                or value.get("arguments")
                or value.get("input")
                or value.get("args")
                or value.get("action_input")
                or value
            )
            if isinstance(args, str):
                try:
                    args = json.loads(args)
                except Exception:
                    pass

            if isinstance(args, dict):
                subagents = args.get("Subagents") or args.get("subagents")
                if isinstance(subagents, list):
                    for sub in subagents:
                        if isinstance(sub, dict):
                            p = (
                                sub.get("prompt")
                                or sub.get("Prompt")
                                or sub.get("prompt_text")
                                or sub.get("task_prompt")
                                or sub.get("content")
                                or sub.get("instructions")
                            )
                            if p is not None:
                                prompts.append(str(p))
                        elif isinstance(sub, str):
                            prompts.append(sub)
                else:
                    # Single subagent direct prompt parameter
                    direct_p = (
                        args.get("prompt")
                        or args.get("Prompt")
                        or args.get("prompt_text")
                        or args.get("task_prompt")
                    )
                    if direct_p is not None:
                        prompts.append(str(direct_p))

            # Stop recursion into this tool call's immediate parameters since it was processed
            return prompts

        # Recurse into all nested values
        for v in value.values():
            prompts.extend(_extract_prompts_from_value(v))

    elif isinstance(value, list):
        for item in value:
            prompts.extend(_extract_prompts_from_value(item))

    return prompts


def extract_subagent_prompts(
    transcript_source: Union[str, io.IOBase, List[Any]]
) -> List[str]:
    """
    Parses a session transcript source and extracts prompt text for every subagent
    found in `invoke_subagent` tool calls.

    Supported sources:
    - Path to a .jsonl file on disk
    - Raw JSONL string or newline-delimited stream
    - IOBase stream (e.g. io.StringIO)
    - List of dicts or list of JSON strings
    """
    prompts: List[str] = []

    if isinstance(transcript_source, list):
        for item in transcript_source:
            if isinstance(item, str):
                item = item.strip()
                if not item:
                    continue
                try:
                    parsed = json.loads(item)
                    prompts.extend(_extract_prompts_from_value(parsed))
                except Exception:
                    pass
            elif isinstance(item, (dict, list)):
                prompts.extend(_extract_prompts_from_value(item))
        return prompts

    lines: List[str] = []
    if isinstance(transcript_source, io.IOBase):
        lines = transcript_source.readlines()
    elif isinstance(transcript_source, str):
        if os.path.isfile(transcript_source):
            with open(transcript_source, "r", encoding="utf-8") as f:
                lines = f.readlines()
        else:
            lines = transcript_source.splitlines()

    for line_num, line in enumerate(lines, 1):
        line = line.strip()
        if not line:
            continue
        try:
            entry = json.loads(line)
            extracted = _extract_prompts_from_value(entry)
            prompts.extend(extracted)
        except json.JSONDecodeError:
            # Non-JSON transcript lines are ignored
            continue

    return prompts


class TranscriptAuditReport:
    """Encapsulates the findings of a transcript prompt discipline audit."""

    def __init__(
        self,
        passed: bool,
        total_subagents: int,
        violations: List[str],
        prompts_checked: List[str],
    ):
        self.passed = passed
        self.total_subagents = total_subagents
        self.violations = violations
        self.prompts_checked = prompts_checked

    def __repr__(self) -> str:
        status = "PASSED" if self.passed else "FAILED"
        return (
            f"<TranscriptAuditReport status={status} total_subagents={self.total_subagents} "
            f"violations_count={len(self.violations)}>"
        )


def audit_transcript(
    transcript_source: Union[str, io.IOBase, List[Any]],
    assert_valid: bool = False,
) -> TranscriptAuditReport:
    """
    Audits all subagent prompts in a transcript against:
    1. `validate_subagent_preflight()`: repository classification, step 1 view_file, no leading steering, no truncation.
    2. `lint_prompt_text()`: single-item scope, defect filing (gh/glab create), PROCEED token, no issue closures.

    If `assert_valid=True`, raises `AssertionError` if any violation is detected.
    Returns a `TranscriptAuditReport`.
    """
    prompts = extract_subagent_prompts(transcript_source)
    violations: List[str] = []

    for idx, prompt_text in enumerate(prompts, 1):
        # 1. Subagent self-rejection pre-flight gate
        preflight_passed, preflight_reason = validate_subagent_preflight(prompt_text)
        if not preflight_passed:
            violations.append(
                f"[Subagent #{idx} Pre-Flight Gate Violation]: {preflight_reason}"
            )

        # 2. Subagent prompt payload linter
        lint_errors = lint_prompt_text(prompt_text)
        for err in lint_errors:
            violations.append(f"[Subagent #{idx} Lint Violation]: {err}")

    passed = len(violations) == 0
    report = TranscriptAuditReport(
        passed=passed,
        total_subagents=len(prompts),
        violations=violations,
        prompts_checked=prompts,
    )

    if assert_valid and not passed:
        failure_msg = (
            f"Transcript prompt discipline audit FAILED with {len(violations)} violation(s) "
            f"across {len(prompts)} subagent dispatch(es):\n"
            + "\n".join(f"  - {v}" for v in violations)
        )
        raise AssertionError(failure_msg)

    return report


def assert_transcript_compliant(
    transcript_source: Union[str, io.IOBase, List[Any]]
) -> TranscriptAuditReport:
    """Helper assertion function enforcing transcript compliance."""
    return audit_transcript(transcript_source, assert_valid=True)


class TestTranscriptAuditProcessGate(unittest.TestCase):
    """
    Unit test suite verifying the Transcript Audit Process Gate for subagent dispatches.
    """

    def setUp(self):
        checklist = "\n".join(f"- {row}" for row in _corpus_requirements())
        self.valid_prompt_1 = f"""You are an isolated Subagent for DEAP01-spec-core.
Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target: scripts/dispatch_subagent.py

Mandatory Instructions:
1. Step 1: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` before executing any actions.
2. Implement FEAT-01 according to specification.
3. Report any defects using `gh issue create` or `glab issue create`.
4. Run validation checks.

Normative Pre-Flight Checklist (verbatim from rules/subagent-dispatch-standards.md):
{checklist}

PROCEED
"""
        self.valid_prompt_2 = f"""You are an isolated Validator Subagent for DEAP01-spec-core.
Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target: docs/designs/feat-01-solution.md

Instructions:
1. Step 1: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your very first step.
2. Verify that all structural identifiers in FEAT-02 match codebase definitions.
3. File issues using `gh issue create` and `glab issue create` if discrepancies found.

Normative Pre-Flight Checklist (verbatim from rules/subagent-dispatch-standards.md):
{checklist}

PROCEED
"""

    def test_clean_compliant_transcript_passes(self):
        """Clean compliant transcript with valid subagent dispatches passes audit."""
        transcript_line = json.dumps({
            "type": "tool_call",
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [
                    {
                        "role": "Micro-Task Implementer",
                        "prompt": self.valid_prompt_1,
                    }
                ]
            }
        })

        report = audit_transcript(transcript_line, assert_valid=True)
        self.assertTrue(report.passed)
        self.assertEqual(report.total_subagents, 1)
        self.assertEqual(report.violations, [])

    def test_dirty_transcript_with_summarized_prompt_fails(self):
        """Dirty transcript with summarized prompt fails with assertion error."""
        dirty_prompt = """Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Role: Implementer [summarized]
1. Step 1: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md`.
Task: Implement FEAT-01.
Record defects with `gh issue create` and `glab issue create`.
PROCEED
"""
        transcript_line = json.dumps({
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [
                    {"role": "implementer", "prompt": dirty_prompt}
                ]
            }
        })

        with self.assertRaises(AssertionError) as ctx:
            audit_transcript(transcript_line, assert_valid=True)

        self.assertIn("truncation/summarization detected", str(ctx.exception))

    def test_dirty_transcript_missing_view_file_on_skill_fails(self):
        """Subagent prompt missing view_file on SKILL.md as step 1 fails audit."""
        dirty_prompt = """Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target: FEAT-01
If defects found, record with `gh issue create` and `glab issue create`.
PROCEED
"""
        transcript_line = json.dumps({
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [
                    {"prompt": dirty_prompt}
                ]
            }
        })

        with self.assertRaises(AssertionError) as ctx:
            audit_transcript(transcript_line, assert_valid=True)

        self.assertIn("missing view_file directive on SKILL.md", str(ctx.exception))

    def test_dirty_transcript_missing_repository_classification_fails(self):
        """Subagent prompt missing repository classification fails audit."""
        dirty_prompt = """Target: FEAT-01
1. Step 1: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` before taking actions.
Record defects with `gh issue create` and `glab issue create`.
PROCEED
"""
        transcript_line = json.dumps({
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [
                    {"prompt": dirty_prompt}
                ]
            }
        })

        with self.assertRaises(AssertionError) as ctx:
            audit_transcript(transcript_line, assert_valid=True)

        self.assertIn("missing repository classification", str(ctx.exception))

    def test_dirty_transcript_leading_steering_fails(self):
        """Subagent prompt with leading line-level steering ahead of Step 1 fails audit."""
        dirty_prompt = """Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target: scripts/dispatch_subagent.py

Replace line 40 with `DEFAULT_TYPE = "custom_worker"`.
Here is the code to apply:
```python
x = 1
```

1. Step 1: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` as your first step.
Record defects with `gh issue create` and `glab issue create`.
PROCEED
"""
        transcript_line = json.dumps({
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [
                    {"prompt": dirty_prompt}
                ]
            }
        })

        with self.assertRaises(AssertionError) as ctx:
            audit_transcript(transcript_line, assert_valid=True)

        self.assertIn("leading line-level steering detected", str(ctx.exception))

    def test_dirty_transcript_forbidden_issue_close_fails(self):
        """Subagent prompt containing forbidden gh/glab issue close directives fails audit."""
        dirty_prompt = f"""{self.valid_prompt_1}
After implementation, run `gh issue close 42`.
"""
        transcript_line = json.dumps({
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [{"prompt": dirty_prompt}]
            }
        })

        with self.assertRaises(AssertionError) as ctx:
            audit_transcript(transcript_line, assert_valid=True)

        self.assertIn("forbidden issue closure command", str(ctx.exception))

    def test_dirty_transcript_missing_proceed_token_fails(self):
        """Subagent prompt missing PROCEED authorization token fails audit."""
        dirty_prompt = """You are an isolated Subagent for DEAP01-spec-core.
Repository Classification: UPSTREAM_SPEC_CORE_COMPILER
Target: scripts/dispatch_subagent.py

Mandatory Instructions:
1. Step 1: Execute `view_file` on `skills/feature-driven-implementation/SKILL.md` before executing any actions.
2. Implement FEAT-01 according to specification.
3. Report any defects using `gh issue create` or `glab issue create`.
"""
        transcript_line = json.dumps({
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [{"prompt": dirty_prompt}]
            }
        })

        with self.assertRaises(AssertionError) as ctx:
            audit_transcript(transcript_line, assert_valid=True)

        self.assertIn("PROCEED", str(ctx.exception))

    def test_dirty_transcript_multi_item_scope_fails(self):
        """Subagent prompt targeting multiple Epics/Features violates micro-task scope."""
        dirty_prompt = f"""{self.valid_prompt_1}
Also implement FEAT-02 and FEAT-03 in the same batch.
"""
        transcript_line = json.dumps({
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [{"prompt": dirty_prompt}]
            }
        })

        with self.assertRaises(AssertionError) as ctx:
            audit_transcript(transcript_line, assert_valid=True)

        self.assertTrue(
            "micro-task scope" in str(ctx.exception)
            or "multiple Features detected" in str(ctx.exception)
        )

    def test_transcript_multiple_subagents_single_turn_all_valid_passes(self):
        """Transcript with multiple valid subagents in a single turn validates and passes."""
        transcript_entry = {
            "type": "tool_call",
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [
                    {
                        "role": "Micro-Task Implementer",
                        "prompt": self.valid_prompt_1,
                    },
                    {
                        "role": "Validator Subagent",
                        "prompt": self.valid_prompt_2,
                    },
                ]
            },
        }

        report = audit_transcript(json.dumps(transcript_entry), assert_valid=True)
        self.assertTrue(report.passed)
        self.assertEqual(report.total_subagents, 2)
        self.assertEqual(report.violations, [])

    def test_transcript_multiple_subagents_single_turn_one_dirty_fails(self):
        """Transcript with multiple subagents in a single turn where one is dirty fails."""
        dirty_prompt = """Role: Implementer
1. Just change line 10
PROCEED
"""
        transcript_entry = {
            "type": "tool_call",
            "name": "invoke_subagent",
            "parameters": {
                "Subagents": [
                    {
                        "role": "Micro-Task Implementer",
                        "prompt": self.valid_prompt_1,  # VALID
                    },
                    {
                        "role": "Bad Implementer",
                        "prompt": dirty_prompt,          # DIRTY
                    },
                ]
            },
        }

        with self.assertRaises(AssertionError) as ctx:
            audit_transcript(json.dumps(transcript_entry), assert_valid=True)

        self.assertIn("Subagent #2", str(ctx.exception))

    def test_empty_transcript_passes(self):
        """Empty transcript stream, empty string, or whitespace-only lines pass audit."""
        # Empty string
        report1 = audit_transcript("", assert_valid=True)
        self.assertTrue(report1.passed)
        self.assertEqual(report1.total_subagents, 0)
        self.assertEqual(report1.violations, [])

        # Whitespace-only string
        report2 = audit_transcript("   \n\n\t  \n", assert_valid=True)
        self.assertTrue(report2.passed)
        self.assertEqual(report2.total_subagents, 0)

        # Empty list
        report3 = audit_transcript([], assert_valid=True)
        self.assertTrue(report3.passed)
        self.assertEqual(report3.total_subagents, 0)

        # Non-tool call entries only
        non_subagent_lines = [
            json.dumps({"type": "message", "role": "user", "content": "Hello"}),
            json.dumps({"type": "tool_call", "name": "view_file", "parameters": {"AbsolutePath": "/path/SKILL.md"}}),
            json.dumps({"type": "tool_call", "name": "run_command", "parameters": {"CommandLine": "git status"}}),
        ]
        report4 = audit_transcript("\n".join(non_subagent_lines), assert_valid=True)
        self.assertTrue(report4.passed)
        self.assertEqual(report4.total_subagents, 0)

    def test_transcript_file_on_disk(self):
        """Audit properly parses and validates a physical transcript.jsonl file on disk."""
        lines = [
            json.dumps({"event": "session_start", "timestamp": "2026-09-01T00:00:00Z"}),
            json.dumps({
                "type": "tool_call",
                "name": "invoke_subagent",
                "parameters": {
                    "Subagents": [{"role": "Worker 1", "prompt": self.valid_prompt_1}]
                }
            }),
            json.dumps({"event": "subagent_finished", "status": "success"}),
            json.dumps({
                "type": "tool_call",
                "name": "invoke_subagent",
                "parameters": {
                    "Subagents": [{"role": "Worker 2", "prompt": self.valid_prompt_2}]
                }
            }),
        ]

        with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as tf:
            tf.write("\n".join(lines))
            tf_path = tf.name

        try:
            report = audit_transcript(tf_path, assert_valid=True)
            self.assertTrue(report.passed)
            self.assertEqual(report.total_subagents, 2)
            self.assertEqual(len(report.prompts_checked), 2)
        finally:
            if os.path.exists(tf_path):
                os.remove(tf_path)

    def test_mock_stream_transcript(self):
        """Audit correctly handles io.StringIO stream objects."""
        transcript_stream = io.StringIO(
            json.dumps({
                "action": "invoke_subagent",
                "action_input": {
                    "Subagents": [{"prompt": self.valid_prompt_1}]
                }
            })
            + "\n"
        )

        report = audit_transcript(transcript_stream, assert_valid=True)
        self.assertTrue(report.passed)
        self.assertEqual(report.total_subagents, 1)

    def test_openai_style_tool_calls_format(self):
        """Audit correctly parses OpenAI-style tool_calls transcript format."""
        openai_entry = {
            "role": "assistant",
            "tool_calls": [
                {
                    "id": "call_123",
                    "type": "function",
                    "function": {
                        "name": "invoke_subagent",
                        "arguments": json.dumps({
                            "Subagents": [{"prompt": self.valid_prompt_1}]
                        }),
                    },
                }
            ],
        }

        report = audit_transcript(json.dumps(openai_entry), assert_valid=True)
        self.assertTrue(report.passed)
        self.assertEqual(report.total_subagents, 1)

    def test_claude_style_tool_use_format(self):
        """Audit correctly parses Claude-style tool_use transcript format."""
        claude_entry = {
            "type": "tool_use",
            "id": "toolu_01",
            "name": "invoke_subagent",
            "input": {
                "Subagents": [{"prompt": self.valid_prompt_2}]
            },
        }

        report = audit_transcript(json.dumps(claude_entry), assert_valid=True)
        self.assertTrue(report.passed)
        self.assertEqual(report.total_subagents, 1)


if __name__ == "__main__":
    unittest.main()
