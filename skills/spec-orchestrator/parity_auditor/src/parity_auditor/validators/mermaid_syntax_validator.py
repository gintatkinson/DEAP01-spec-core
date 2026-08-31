"""Offline checker for the project's documented Mermaid syntax rules.

This is **not** a Mermaid grammar parser. It enforces the character-level rules
consolidated in ``rules/platform-independence.md``, which are the rules that have
actually broken rendering in this repository. A clean result means no *documented
rule* was violated; it does not prove the diagram renders.

That limit is deliberate. The alternative considered was calling a remote renderer,
which was rejected: a blocking gate depending on a third-party service fails when
that service is down or rate-limits, and it ships specification content to a third
party. See ``.pipeline/upstream/pipeline-tooling.md`` § *Validation Gates*.

Rules enforced
--------------
* Semicolons in ``Note`` statements and in message text. Mermaid reads ``;`` as a
  statement separator, so the remainder of the line becomes a new statement and
  collides with whatever follows. This is the fault that shipped on issue #283.
* Unclosed ```` ```mermaid ```` fences.
* Colons inside class-diagram member lines and inside note strings.
* Stereotypes on class-diagram relationship lines (``<<`` ``>>`` ``«`` ``»`` and
  their HTML entities).
* Curly braces inside class-diagram member lines.
* Empty class bodies written on a single line (``class X {}``). A class block is
  opened by ``class X {`` and closed only by a line consisting of ``}``, so the
  same-line brace never closes it; a later ``}`` pops the leaked block and the
  classes after it are silently attached to the wrong namespace. Issue #279.

Deliberately not enforced
-------------------------
* *Attribute-less classes as such.* Only the single-line spelling above is
  rejected. An ancestor container node on a schema containment path frequently has
  no attributes of its own, and the canonical Feature template in
  ``skills/schema-specification-engineering/SKILL.md`` ships one, so a blanket
  prohibition would reject this repository's own template. Semantic emptiness is
  governed elsewhere: the *isolated classes* rule in ``validators/uml.py`` covers
  classes with no relationships, and schema coverage covers unmapped nodes.
* *Participants declared before use.* Mermaid auto-declares participants on first
  reference, so requiring an explicit declaration would reject valid diagrams. A
  checker that produces false positives is worse than no checker.
* *Arrow token validity.* Mermaid accepts a large set of arrow forms; enumerating
  them invites false positives for no demonstrated benefit.
"""

import os
import re
from typing import List, Optional, Sequence, Tuple

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

FENCE = "`" * 3
_FENCE_OPEN = re.compile(r"^\s*" + FENCE + r"\s*mermaid\s*$", re.I)
_FENCE_ANY = re.compile(r"^\s*" + FENCE)

# `Note over A,B: text` / `Note left of A: text`
_NOTE = re.compile(r"^\s*note\b[^:]*:(?P<text>.*)$", re.I)
# `A->>B: text`, `A-->B: text`, `A-)B: text` and similar
_MESSAGE = re.compile(r"^\s*(?:(?:\"[^\"]*\")|\S+)\s*-{1,2}[->x)]{1,2}>?\s*(?:(?:\"[^\"]*\")|\S+)\s*:(?P<text>.*)$")
# classDiagram note forms: `note "..."` / `note for X "..."`
_CLASS_NOTE = re.compile(r'^\s*note\b.*?"(?P<text>[^"]*)"', re.I)
_RELATIONSHIP = re.compile(r"(\*--|o--|<\|--|--\|>|-->|<--|\.\.>|--)")
# `ClassA *-- ClassB : label`  -- captures whatever follows the colon.
_RELATIONSHIP_LABEL = re.compile(
    r"^\s*(?:(?:\"[^\"]*\")|\S+)\s*(?:\*--|o--|<\|--|--\|>|-->|<--|\.\.>|--)\s*(?:(?:\"[^\"]*\")|\S+)\s*:\s*(?P<label>.*)$"
)
_STEREOTYPE = re.compile(r"(<<|>>|&lt;&lt;|&gt;&gt;|\u00ab|\u00bb)")
# A member line inside `class X { ... }`.
#
# The visibility marker is followed by at most ONE space before content. This
# deliberately excludes unified-diff context lines, which preserve the original
# indentation after the diff marker (e.g. "-       }"). Several docs embed diffs
# containing mermaid blocks, and reading a diff marker as a UML visibility marker
# produced false positives on first implementation.
_MEMBER = re.compile(r"^\s*(?:[+\-#~]\s*)?(?P<rest>\S.*)$")
# Structural brace lines are not members and must never be flagged for braces.
_BRACE_ONLY = re.compile(r"^[{}\s]*$")
# `class X {}` -- an empty body opened and closed on one line. The leading `\s*`
# deliberately excludes unified-diff lines, whose `+`/`-` marker is not whitespace,
# for the same reason as _MEMBER above.
_ONE_LINE_EMPTY_CLASS = re.compile(
    r"^\s*class\s+(?:`[^`]+`|[A-Za-z0-9_.\-]+)\s*\{\s*\}(?:\s*(?:%%.*)?)$", re.I
)

_DIAGRAM_KINDS = (
    "sequencediagram", "classdiagram", "statediagram", "statediagram-v2", 
    "graph", "flowchart", "erdiagram", "gantt", "pie", "gitgraph", "c4context"
)

MERMAID_SEQUENCE_RESERVED_KEYWORDS = {
    "link",
    "links",
    "actor",
    "participant",
    "loop",
    "opt",
    "alt",
    "rect",
    "note",
    "end",
    "par",
    "and",
    "critical",
    "option",
    "break",
    "activate",
    "deactivate",
    "autonumber",
    "box",
    "create",
    "destroy",
}

_SEQUENCE_PARTICIPANT = re.compile(
    r"^\s*(?:participant|actor)\s+(?P<alias>\"[^\"]+\"|[^\s:]+)",
    re.I
)

# No directories are excluded from the default scan.
#
# An earlier revision skipped `decisions/` and `designs/` to keep the linter green on
# historical records. That was wrong: downstream output is a diagnostic instrument,
# and excluding directories suppresses the signal it exists to provide -- the
# exclusion concealed 13 confirmed non-rendering diagrams behind a green check.
# Findings in disposable content are the measurement, not a problem to manage.

SHAPE_PAIRS = {
    '[': ']',
    '[[': ']]',
    '[(': ')]',
    '([': '])',
    '[/': '/]',
    '[\\': '\\]',
    '{{': '}}',
    '{': '}',
    '(': ')',
    '((': '))',
    '>': ']',
}

def validate_mermaid_node_label_quoting(line: str) -> List[str]:
    unquoted = []
    if re.match(r"^\s*(subgraph|class|note|style|linkStyle|click)\b", line, re.I):
        return []
    
    for match in re.finditer(r"\b([a-zA-Z0-9_]+)\s*(\[\[|\[\(|\(\[|\[\/|\[\\|\{\{|\{|\[|\(\(|\(|>)", line):
        opener = match.group(2)
        closer = SHAPE_PAIRS[opener]
        
        start_idx = match.end()
        end_idx = line.find(closer, start_idx)
        if end_idx != -1:
            label = line[start_idx:end_idx].strip()
            if not label:
                continue
            if not (label.startswith('"') and label.endswith('"')):
                if re.search(r"[/:()\[\]]", label):
                    unquoted.append(label)
    return unquoted

def validate_mermaid_subgraph_title_quoting(line: str) -> Optional[str]:
    match = re.match(r"^\s*subgraph\s+(.*)", line, re.I)
    if match:
        title = match.group(1).strip()
        bracket_match = re.match(r"^[a-zA-Z0-9_]+\s*\[(.*?)\]\s*$", title)
        if bracket_match:
            title = bracket_match.group(1).strip()
        
        if not (title.startswith('"') and title.endswith('"')):
            if " " in title or "-" in title:
                return title
    return None

def validate_mermaid_angle_bracket_escaping(line: str) -> Optional[str]:
    clean_line = line.strip()
    clean_line = re.sub(r"<<[a-zA-Z0-9_\-]+>>", "", clean_line, flags=re.IGNORECASE)
    
    # Strip longest arrows first to prevent partial replacements
    arrows = [
        "==>>", "==>", "-->>", "-->", "->>", "->",
        "<<==", "<==", "<<--", "<--", "<<-", "<-",
        "-.->", "<-.-", "<-->", "<==>"
    ]
    for arrow in arrows:
        clean_line = clean_line.replace(arrow, "")
    
    in_quotes = False
    unquoted_bracket = None
    escape = False
    for char in clean_line:
        if escape:
            escape = False
            continue
        if char == '\\':
            escape = True
            continue
        if char == '"':
            in_quotes = not in_quotes
        elif not in_quotes:
            if char in ('<', '>'):
                unquoted_bracket = char
                break
    return unquoted_bracket

def _blocks(text: str) -> Tuple[List[Tuple[int, List[str], str]], List[int]]:
    """Return ``(blocks, unclosed_starts)``.

    Each block is ``(start_line_1based, body_lines, kind)``.
    """
    blocks: List[Tuple[int, List[str], str]] = []
    unclosed: List[int] = []
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        if _FENCE_OPEN.match(lines[i]):
            start = i + 1
            body: List[str] = []
            i += 1
            closed = False
            while i < len(lines):
                if _FENCE_ANY.match(lines[i]):
                    closed = True
                    break
                body.append(lines[i])
                i += 1
            kind = ""
            for line in body:
                stripped = line.strip().lower()
                if stripped and not stripped.startswith("%%"):
                    kind = next((k for k in _DIAGRAM_KINDS if stripped.startswith(k)), "")
                    break
            if closed:
                blocks.append((start, body, kind))
            else:
                unclosed.append(start)
        i += 1
    return blocks, unclosed


def _in_class_body(body: Sequence[str], idx: int) -> bool:
    """True when ``body[idx]`` sits inside a ``class X { ... }`` block."""
    depth = 0
    for line in body[:idx]:
        depth += line.count("{") - line.count("}")
    return depth > 0


def check_mermaid_text(text: str, source: str = "<input>") -> List[str]:
    """Return one error string per documented-rule violation. Empty means clean."""
    errors: List[str] = []
    blocks, unclosed = _blocks(text)

    for start in unclosed:
        errors.append(Finding(
                    "mermaid-fence-must-be-closed",
            f"{source}:{start}: unclosed ```mermaid fence. Every diagram must be "
            f"closed with a matching {FENCE} on its own line, or the block leaks "
            f"into the surrounding document."
        , location=f"{source}"))

    for start, body, kind in blocks:
        first_line_content = ""
        header_lineno = 0
        for offset, line in enumerate(body):
            lineno = start + offset + 1
            line_strip = line.strip()
            if not line_strip or line_strip.startswith("%%"):
                continue
            first_line_content = line_strip.lower()
            header_lineno = lineno
            break
        
        if first_line_content:
            is_valid = any(first_line_content.startswith(h) for h in _DIAGRAM_KINDS)
            if not is_valid:
                errors.append(Finding(
                    "mermaid-missing-diagram-header",
                    f"{source}:{header_lineno}: missing or invalid Mermaid diagram header "
                    f"({first_line_content!r}). The first non-comment line inside a mermaid block MUST declare a valid diagram type header.",
                    location=f"{source}"
                ))
        else:
            errors.append(Finding(
                "mermaid-missing-diagram-header",
                f"{source}:{start}: missing or invalid Mermaid diagram header "
                f"(''). The first non-comment line inside a mermaid block MUST declare a valid diagram type header.",
                location=f"{source}"
            ))

        for offset, line in enumerate(body):
            lineno = start + offset + 1
            line_strip = line.strip()

            if not line_strip or line_strip.startswith("%%"):
                continue

            if line_strip.count('"') % 2 != 0:
                errors.append(Finding(
                    "mermaid-unclosed-quotes",
                    f"{source}:{lineno}: unclosed double quotes in line "
                    f"({line_strip!r}). Mermaid diagrams must have matching double quotes.",
                    location=f"{source}"
                ))

            if kind in ("graph", "flowchart", "sequencediagram", "statediagram", "statediagram-v2"):
                unquoted_bracket = validate_mermaid_angle_bracket_escaping(line_strip)
                if unquoted_bracket:
                    errors.append(Finding(
                        "mermaid-diagram-unquoted-brackets-forbidden",
                        f"{source}:{lineno}: unquoted '{unquoted_bracket}' character in Mermaid diagram "
                        f"line: '{line_strip}'. Transitions, labels, or guards containing comparison "
                        f"operators or brackets MUST be enclosed in double quotes."
                    , location=f"{source}"))

            if kind in ("graph", "flowchart"):
                unquoted_title = validate_mermaid_subgraph_title_quoting(line_strip)
                if unquoted_title:
                    errors.append(Finding(
                        "mermaid-subgraph-title-must-be-quoted",
                        f"{source}:{lineno}: unquoted subgraph title containing spaces or hyphens "
                        f"({unquoted_title!r}). Subgraph titles with spaces or hyphens MUST be enclosed in double quotes."
                    , location=f"{source}"))
                
                unquoted_labels = validate_mermaid_node_label_quoting(line_strip)
                for label in unquoted_labels:
                    errors.append(Finding(
                        "mermaid-node-label-must-be-quoted",
                        f"{source}:{lineno}: unquoted special characters in node label "
                        f"({label!r}). Node labels containing slashes, colons, parentheses, or brackets MUST be enclosed in double quotes."
                    , location=f"{source}"))

            if kind == "sequencediagram":
                part_match = _SEQUENCE_PARTICIPANT.match(line_strip)
                if part_match:
                    raw_alias = part_match.group("alias")
                    clean_alias = raw_alias.strip('"').lower()
                    if clean_alias in MERMAID_SEQUENCE_RESERVED_KEYWORDS:
                        errors.append(Finding(
                            "mermaid-reserved-keyword-as-participant-alias",
                            f"{source}:{lineno}: reserved keyword '{clean_alias}' used as sequence diagram participant alias/ID "
                            f"({line_strip!r}). Using reserved keywords like 'link', 'actor', 'participant', 'loop', 'opt', 'alt', 'rect', 'note', 'end' as participant aliases breaks Mermaid rendering. Rename the participant alias.",
                            location=f"{source}"
                        ))

            # --- semicolons in Note statements and message text ---------------
            payload: Optional[str] = None
            note = _NOTE.match(line)
            if note:
                payload = note.group("text")
            else:
                msg = _MESSAGE.match(line)
                if msg:
                    payload = msg.group("text")
            if payload and ";" in payload:
                errors.append(Finding(
                    "mermaid-no-semicolon-in-note-or-message",
                    f"{source}:{lineno}: semicolon in Mermaid Note or message text "
                    f"({line.strip()!r}). Mermaid parses ';' as a statement separator, "
                    f"so the rest of the line becomes a new statement and the diagram "
                    f"fails to render. Replace it with a comma, dash or space."
                , location=f"{source}"))

            if kind != "classdiagram":
                continue

            # --- class-diagram-only rules --------------------------------------
            if _ONE_LINE_EMPTY_CLASS.match(line):
                errors.append(Finding(
                    "mermaid-no-single-line-empty-class-body",
                    f"{source}:{lineno}: single-line empty Mermaid class body "
                    f"({line.strip()!r}). A same-line closing brace never closes the "
                    f"block, so a later '}}' pops it instead and every class after it "
                    f"is attached to the wrong namespace. Put the closing brace on its "
                    f"own line, or omit the braces entirely."
                , location=f"{source}"))

            member = _MEMBER.match(line)
            if member and not _BRACE_ONLY.match(member.group("rest")) \
                    and _in_class_body(body, offset):
                if ":" in line:
                    errors.append(Finding(
                    "mermaid-no-colon-in-class-member",
                        f"{source}:{lineno}: colon inside a Mermaid class member line "
                        f"({line.strip()!r}). Use 'ReturnType methodName(Type arg)' "
                        f"spacing instead."
                    , location=f"{source}"))
                if "{" in line or "}" in line:
                    errors.append(Finding(
                    "mermaid-no-curly-brace-in-class-member",
                        f"{source}:{lineno}: curly brace inside a Mermaid class member "
                        f"line ({line.strip()!r}). Use parentheses, e.g. "
                        f"'(default earth)'."
                    , location=f"{source}"))
                rest = member.group("rest")
                reserved_match = re.match(r"^(link|style|click|callback)\b", rest, re.I)
                if reserved_match:
                    errors.append(Finding(
                        "mermaid-reserved-keyword-in-class-member",
                        f"{source}:{lineno}: reserved keyword '{reserved_match.group(1)}' used as unquoted class member "
                        f"({line.strip()!r}). This breaks rendering. Quote the member or rename it."
                    , location=f"{source}"))

            class_note = _CLASS_NOTE.match(line)
            if class_note and ":" in class_note.group("text"):
                errors.append(Finding(
                    "mermaid-no-colon-in-note-string",
                    f"{source}:{lineno}: colon inside a Mermaid note string "
                    f"({line.strip()!r}). Colons in notes break rendering."
                , location=f"{source}"))

            if _RELATIONSHIP.search(line) and _STEREOTYPE.search(line):
                errors.append(Finding(
                    "mermaid-no-stereotype-on-relationship",
                    f"{source}:{lineno}: stereotype on a Mermaid relationship line "
                    f"({line.strip()!r}). Use a plain label such as 'references'."
                , location=f"{source}"))

            rel_label = _RELATIONSHIP_LABEL.match(line)
            if rel_label:
                label = rel_label.group("label").strip()
                # Issue #333: a colon is not a "special character that quoting fixes".
                # Mermaid treats ':' as a statement separator, so it ends the statement
                # wherever it appears — inside quotes as much as outside. The quoting
                # rule was generalised from spaces, where quoting genuinely works, and
                # the resulting label satisfied the gate while GitHub refused to render
                # it. Checked before the quoting rule so a quoted colon is still caught.
                if ":" in label:
                    errors.append(Finding(
                        "mermaid-no-colon-in-relationship-label",
                        f"{source}:{lineno}: colon inside a Mermaid relationship label "
                        f"({line.strip()!r}). Quoting does not help: Mermaid parses ':' "
                        f"as a statement separator wherever it occurs, so the renderer "
                        f"fails with \"Expecting 'NEWLINE', 'EOF', got 'LABEL'\". Drop "
                        f"the namespace prefix or replace the colon, e.g. "
                        f': "augments nw node".',
                        location=f"{source}",
                    ))
                elif label and not label.startswith('"') and re.search(r"\s", label):
                    errors.append(Finding(
                    "mermaid-relationship-label-must-be-quoted",
                        f"{source}:{lineno}: unquoted Mermaid relationship label "
                        f"containing spaces ({line.strip()!r}). Enclose it in "
                        f'double quotes, e.g. : "renders one instance".'
                    , location=f"{source}"))

    return errors


class MermaidSyntaxValidator(IValidator):
    """``IValidator`` wrapper scanning markdown trees for rule violations."""

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        search_dirs = kwargs.get("search_dirs")
        if not search_dirs:
            search_dirs = [
                os.path.join(repo.workspace_dir, "docs"),
                os.path.join(repo.workspace_dir, "rules"),
                os.path.join(repo.workspace_dir, "skills"),
            ]

        errors: List[str] = []
        for root in search_dirs:
            if not os.path.isdir(root):
                continue
            for dirpath, _dirnames, filenames in os.walk(root):
                for name in sorted(filenames):
                    if not name.endswith(".md"):
                        continue
                    path = os.path.join(dirpath, name)
                    try:
                        with open(path, "r", encoding="utf-8", errors="replace") as fh:
                            content = fh.read()
                    except OSError:
                        continue
                    if FENCE + "mermaid" not in content:
                        continue
                    rel = os.path.relpath(path, repo.workspace_dir)
                    errors.extend(check_mermaid_text(content, source=rel))
        return errors
