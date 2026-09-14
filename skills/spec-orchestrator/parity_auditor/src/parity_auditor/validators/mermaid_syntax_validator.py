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
* Commas and slashes inside quoted Mermaid labels in ``graph``, ``flowchart``,
  ``stateDiagram``, and ``stateDiagram-v2``. Issue #200.
* Universal Mermaid Visual Ergonomics & Layout Invariants (Issue #274):
  - Rule E1: Horizontal flow prohibition (``flowchart LR`` / ``graph LR`` prohibited
    when node count > 2 or any label > 25 characters; mandate ``flowchart TD`` or ``flowchart TB``).
  - Rule E2: Mandatory node label line-wrapping (single lines inside node labels
    exceeding 35 characters must be wrapped with ``<br/>``, ignoring ``<b>`` / ``</b>``).
  - Rule E3: Mandatory ``direction TB`` on subgraphs (diagrams with >= 2 subgraphs or
    subgraphs with >= 4 sibling nodes must declare explicit ``direction TB`` or ``direction TD``).
  - Rule E4: Universal Option 3 compact block standard (architecture/interface diagrams
    SV-1, ICD, STPA must use Option 3 compact blocks with bulleted port attributes
    ``• port (DIR)`` instead of exploded child port nodes or subgraphs).

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
from typing import List, Optional, Sequence, Set, Tuple

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

_QUOTED_LABEL_FORBIDDEN = (",", "/")


def validate_mermaid_quoted_label_content(line: str) -> List[Tuple[str, List[str]]]:
    """Scan quoted strings for forbidden comma and slash characters (ignoring HTML tags)."""
    offending: List[Tuple[str, List[str]]] = []
    for quoted in re.finditer(r'"[^"]*"', line):
        inner = quoted.group(0)
        # Strip allowed HTML formatting tags (e.g. <br/>, <br>, <b>, </b>, <i>, </i>) before scanning for forbidden characters
        clean_inner = re.sub(r"</?[a-zA-Z0-9_-]+\s*/?>", "", inner)
        conflict = [ch for ch in _QUOTED_LABEL_FORBIDDEN if ch in clean_inner]
        if conflict:
            offending.append((inner, conflict))
    return offending


def parse_line_nodes(line: str) -> List[Tuple[str, str]]:
    """Extract (node_id, raw_label) pairs from node shape definitions on a line."""
    line_strip = line.strip()
    if not line_strip or line_strip.startswith("%%"):
        return []
    if re.match(r"^\s*(?:subgraph|direction|end|class|style|linkStyle|click|classDef)\b", line, re.I):
        return []

    # Strip edge labels between |...| first to avoid treating bracketed text in edge labels as node shapes
    clean_line = re.sub(r"\|[^|]*\|", "", line)

    nodes: List[Tuple[str, str]] = []
    idx = 0
    while idx < len(clean_line):
        match = re.search(
            r"(?<![a-zA-Z0-9_\-\&])([a-zA-Z0-9_]+)\s*(\[\[|\[\(|\(\[|\[\/|\[\\|\{\{|\{|\[|\(\(|\(|>)",
            clean_line[idx:],
        )
        if not match:
            break

        nid = match.group(1)
        opener = match.group(2)
        closer = SHAPE_PAIRS.get(opener, "]")

        start_idx = idx + match.end()
        if clean_line[start_idx:].lstrip().startswith('"'):
            q_start = clean_line.find('"', start_idx)
            q_end = clean_line.find('"', q_start + 1)
            if q_end != -1:
                end_idx = clean_line.find(closer, q_end)
            else:
                end_idx = clean_line.find(closer, start_idx)
        else:
            end_idx = clean_line.find(closer, start_idx)

        if end_idx != -1:
            raw_label = clean_line[start_idx:end_idx].strip()
            if nid.lower() not in ("subgraph", "direction", "end", "class", "style", "linkstyle", "click", "tb", "td", "lr", "rl", "bt"):
                nodes.append((nid, raw_label))
            idx = end_idx + len(closer)
        else:
            idx = start_idx
    return nodes


def validate_mermaid_node_label_quoting(line: str) -> List[str]:
    unquoted = []
    if re.match(r"^\s*(subgraph|class|note|style|linkStyle|click)\b", line, re.I):
        return []

    for node_id, label in parse_line_nodes(line):
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
        bracket_match = re.match(r"^([a-zA-Z0-9_]+)\s*\[(.*?)\]\s*$", title)
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
    arrows = sorted([
        "==>>", "==>", "-->>", "-->", "->>", "->",
        "<<==", "<==", "<<--", "<--", "<<-", "<-",
        "-.->", "<-.-", "<-->", "<==>", "<->"
    ], key=len, reverse=True)
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


def extract_node_ids_from_line(line: str) -> Set[str]:
    """Extract node identifiers declared or referenced on a single diagram line."""
    nodes: Set[str] = set()
    line_strip = line.strip()
    if not line_strip or line_strip.startswith("%%"):
        return nodes
    if re.match(r"^\s*(?:subgraph|direction|end|class|style|linkStyle|click|classDef)\b", line, re.I):
        return nodes

    # 1. Node shape definitions
    for nid, _ in parse_line_nodes(line):
        nodes.add(nid)

    # 2. Node connections: e.g. A --> B, A -->|label| B, GCS <-->|"..."| FCS
    clean_line = re.sub(r'"[^"]*"', '""', line)
    for match in re.finditer(
        r"\b([a-zA-Z0-9_]+)\s*(?:<[=-]+>|<-+>|<=+>|--[->x)]+>|-->|->>|->|==>|==>>|-.->|-.-|---|===|\.\.>|<--|<==|<<-)\s*(?:\|[^|]*\|\s*)?([a-zA-Z0-9_]+)\b",
        clean_line,
    ):
        src, dst = match.group(1), match.group(2)
        if src.lower() not in ("subgraph", "direction", "end", "class", "style", "linkstyle", "click", "tb", "td", "lr", "rl", "bt"):
            nodes.add(src)
        if dst.lower() not in ("subgraph", "direction", "end", "class", "style", "linkstyle", "click", "tb", "td", "lr", "rl", "bt"):
            nodes.add(dst)

    return nodes


def extract_mermaid_nodes_and_labels(body: Sequence[str]) -> Tuple[Set[str], List[Tuple[int, str, str]]]:
    """Extract all unique node IDs and (line_offset, node_id, raw_label) pairs from a flowchart/graph body."""
    all_node_ids: Set[str] = set()
    node_labels: List[Tuple[int, str, str]] = []

    for offset, line in enumerate(body):
        line_strip = line.strip()
        if not line_strip or line_strip.startswith("%%"):
            continue
        if re.match(r"^\s*(?:direction|end|class|style|linkStyle|click|classDef)\b", line, re.I):
            continue
        if re.match(r"^\s*subgraph\b", line, re.I):
            continue

        for nid, raw_label in parse_line_nodes(line):
            all_node_ids.add(nid)
            node_labels.append((offset, nid, raw_label))

        for nid in extract_node_ids_from_line(line):
            all_node_ids.add(nid)

    return all_node_ids, node_labels


def validate_mermaid_horizontal_flow(first_line: str, body: Sequence[str]) -> Optional[Tuple[int, int]]:
    """Rule E1: Flag error if diagram type is flowchart LR or graph LR when node count > 2 or any label > 25 chars.

    Returns (node_count, max_label_length) if violated, else None.
    """
    if not re.match(r"^\s*(?:flowchart|graph)\s+LR\b", first_line, re.IGNORECASE):
        return None

    nodes, labels = extract_mermaid_nodes_and_labels(body)
    max_label_len = 0
    for _, _, raw_label in labels:
        clean = raw_label.strip()
        if clean.startswith('"') and clean.endswith('"'):
            clean = clean[1:-1]
        for seg in re.split(r"<br\s*/?>", clean, flags=re.I):
            seg_clean = re.sub(r"</?b>", "", seg, flags=re.I).strip()
            if len(seg_clean) > max_label_len:
                max_label_len = len(seg_clean)

    if len(nodes) > 2 or max_label_len > 25:
        return len(nodes), max_label_len
    return None


def validate_mermaid_node_label_line_wrapping(body: Sequence[str]) -> List[Tuple[int, int, str]]:
    """Rule E2: Flag error if any single line inside a node label exceeds 35 chars without <br/> wrapping.

    Returns list of (line_offset, clean_line_length, clean_line_content).
    """
    violations: List[Tuple[int, int, str]] = []
    _, labels = extract_mermaid_nodes_and_labels(body)
    for offset, _, raw_label in labels:
        clean = raw_label.strip()
        if clean.startswith('"') and clean.endswith('"'):
            clean = clean[1:-1]
        for seg in re.split(r"<br\s*/?>", clean, flags=re.I):
            seg_clean = re.sub(r"</?b>", "", seg, flags=re.I).strip()
            if len(seg_clean) > 35:
                violations.append((offset, len(seg_clean), seg_clean))
    return violations


def validate_mermaid_subgraph_direction(body: Sequence[str]) -> List[Tuple[int, str]]:
    """Rule E3: Flag error if diagram contains >= 2 subgraphs or >= 4 sibling nodes inside a subgraph without direction TB/TD.

    Returns list of (line_offset, subgraph_name).
    """
    violations: List[Tuple[int, str]] = []
    subgraph_stack: List[dict] = []
    subgraphs_list: List[dict] = []

    for offset, line in enumerate(body):
        line_strip = line.strip()
        if not line_strip or line_strip.startswith("%%"):
            continue

        sg_match = re.match(r"^\s*subgraph\s+(.*)", line, re.I)
        if sg_match:
            raw_title = sg_match.group(1).strip()
            bracket_match = re.match(r"^([a-zA-Z0-9_]+)\s*\[(.*?)\]\s*$", raw_title)
            if bracket_match:
                name = bracket_match.group(1).strip()
            else:
                name = raw_title.strip('"')
            sg_info = {
                "offset": offset,
                "name": name,
                "has_direction": False,
                "nodes": set(),
            }
            subgraphs_list.append(sg_info)
            subgraph_stack.append(sg_info)
            continue

        if re.match(r"^\s*direction\s+(?:TB|TD)\b", line, re.I):
            if subgraph_stack:
                subgraph_stack[-1]["has_direction"] = True
            continue

        if re.match(r"^\s*end\b", line, re.I):
            if subgraph_stack:
                subgraph_stack.pop()
            continue

        if subgraph_stack:
            line_nodes = extract_node_ids_from_line(line)
            for nid in line_nodes:
                subgraph_stack[-1]["nodes"].add(nid)

    total_subgraphs = len(subgraphs_list)
    for sg in subgraphs_list:
        if (total_subgraphs >= 2 or len(sg["nodes"]) >= 4) and not sg["has_direction"]:
            violations.append((sg["offset"], sg["name"]))

    return violations


def validate_mermaid_option3_compact_blocks(body: Sequence[str], source: str = "") -> List[Tuple[int, str]]:
    """Rule E4: Flag error if an architecture/interface diagram (SV-1, ICD, STPA) uses exploded port nodes/subgraphs.

    Returns list of (line_offset, line_content).
    """
    full_text = "\n".join(body)
    is_arch = (
        bool(re.search(r"(sv-?1|icd|stpa|conops|system_interface|interface_matrix)", source, re.I))
        or bool(re.search(r"\b(?:SV-?1|ICD|STPA|System\s+Interface|Interface\s+Matrix|DoDAF\s+SV-1)\b", full_text, re.I))
    )
    if not is_arch:
        return []

    violations: List[Tuple[int, str]] = []
    for offset, line in enumerate(body):
        line_strip = line.strip()
        if not line_strip or line_strip.startswith("%%"):
            continue

        # Exploded port subgraph
        sg_match = re.match(r"^\s*subgraph\s+(?:\"[^\"]+\"|([a-zA-Z0-9_]+))(?:\s*\[(.*?)\])?", line, re.I)
        if sg_match:
            sg_id = (sg_match.group(1) or "").lower()
            sg_title = (sg_match.group(2) or "").lower()
            if sg_id.startswith("port_") or sg_id.startswith("port-") or "ports" in sg_id or "ports" in sg_title:
                violations.append((offset, line_strip))
            continue

        # Exploded port node
        for node_id, raw_label in parse_line_nodes(line):
            if re.match(r"^(?:PORT|port)[_-]", node_id):
                violations.append((offset, line_strip))
                break

            clean_label = raw_label.strip('"').strip()
            if (clean_label.startswith("PORT_") or clean_label.startswith("PORT-") or clean_label.startswith("port_") or clean_label.startswith("port-")) and "•" not in clean_label:
                violations.append((offset, line_strip))
                break

    return violations


def validate_mermaid_layout_ergonomics(
    start: int,
    body: Sequence[str],
    kind: str,
    source: str = "<input>",
) -> List[Finding]:
    """Validate visual ergonomics and horizontal layout gates (Rules E1-E4).

    (a) Rule E1 (Orientation & Horizontal Sprawl): For graph and flowchart, flag unconstrained
        LR or RL orientation when total link count (--> or ---) exceeds 4 without nested
        subgraph partitioning (mermaid-ergonomics-horizontal-sprawl).
    (b) Rule E2 (Node Label Width & Wrapping): Flag node labels containing unbroken text lines
        exceeding 35 characters without <br/> or newline line breaks (mermaid-ergonomics-unbroken-label).
    (c) Rule E3 (Subgraph Layout Direction): When subgraphs are declared within graph or
        flowchart, flag subgraphs missing explicit internal direction TB declarations
        (mermaid-subgraph-direction-tb-mandated).
    (d) Rule E4 (Universal Option 3 Compact Block Standard): Architecture/interface diagrams
        must use Option 3 compact blocks (mermaid-option3-compact-block-mandated).
    """
    errors: List[Finding] = []
    if kind not in ("graph", "flowchart"):
        return errors

    first_line_content = ""
    header_lineno = start
    for offset, line in enumerate(body):
        line_strip = line.strip()
        if not line_strip or line_strip.startswith("%%"):
            continue
        first_line_content = line_strip.lower()
        header_lineno = start + offset + 1
        break

    # Rule E1 (Orientation & Horizontal Sprawl)
    is_lr_or_rl = False
    orientation = ""
    for offset, line in enumerate(body):
        line_strip = line.strip()
        if not line_strip or line_strip.startswith("%%"):
            continue
        m = re.match(r"^\s*(?:flowchart|graph)\s+(LR|RL)\b", line_strip, re.I)
        if m:
            is_lr_or_rl = True
            orientation = m.group(1).upper()
            header_lineno = start + offset + 1
            break
        if re.match(r"^\s*(?:flowchart|graph)\b", line_strip, re.I):
            header_lineno = start + offset + 1
            for sub_offset in range(offset + 1, len(body)):
                sub_line = body[sub_offset].strip()
                if not sub_line or sub_line.startswith("%%"):
                    continue
                if re.match(r"^\s*subgraph\b", sub_line, re.I):
                    break
                dir_m = re.match(r"^\s*direction\s+(LR|RL)\b", sub_line, re.I)
                if dir_m:
                    is_lr_or_rl = True
                    orientation = dir_m.group(1).upper()
                    header_lineno = start + sub_offset + 1
                    break
            break

    total_links = sum(
        len(re.findall(r"-->|---", l))
        for l in body
        if not l.strip().startswith("%%")
    )
    has_subgraphs = any(
        re.match(r"^\s*subgraph\b", l.strip(), re.I)
        for l in body
        if not l.strip().startswith("%%")
    )

    if is_lr_or_rl and total_links > 4 and not has_subgraphs:
        errors.append(Finding(
            "mermaid-ergonomics-horizontal-sprawl",
            f"{source}:{header_lineno}: flowchart/graph uses '{orientation}' orientation with {total_links} links "
            f"without nested subgraph partitioning. Use 'TD' or 'TB' orientation or insert explicit subgraphs to prevent horizontal sprawl.",
            location=f"{source}",
        ))

    # Also check horizontal layout constraint (node count > 2 or label > 25)
    h_flow = validate_mermaid_horizontal_flow(first_line_content, body)
    if h_flow:
        node_cnt, max_lbl = h_flow
        errors.append(Finding(
            "mermaid-horizontal-flow-prohibited",
            f"{source}:{header_lineno}: horizontal layout ({first_line_content!r}) is prohibited for diagrams with more than 2 nodes or labels exceeding 25 characters (found {node_cnt} nodes, max label length {max_lbl}). Mandate 'flowchart TD' or 'flowchart TB'.",
            location=f"{source}",
        ))

    # Rule E2 (Node Label Width & Wrapping)
    _, labels = extract_mermaid_nodes_and_labels(body)
    for offset, _, raw_label in labels:
        clean = raw_label.strip()
        if clean.startswith('"') and clean.endswith('"'):
            clean = clean[1:-1]
        for seg in re.split(r"<br\s*/?>|\r?\n", clean, flags=re.I):
            seg_clean = re.sub(r"</?[a-zA-Z0-9_-]+\s*/?>", "", seg).strip()
            if len(seg_clean) > 35:
                lineno = start + offset + 1
                errors.append(Finding(
                    "mermaid-ergonomics-unbroken-label",
                    f"{source}:{lineno}: node label line exceeds 35 characters without '<br/>' or newline line breaks ({len(seg_clean)} chars: {seg_clean!r}). Break line using '<br/>' or newline.",
                    location=f"{source}",
                ))
                errors.append(Finding(
                    "mermaid-node-label-line-wrapping-mandated",
                    f"{source}:{lineno}: node label line exceeds 35 characters without '<br/>' wrapping ({len(seg_clean)} chars: {seg_clean!r}). Node labels must be wrapped with '<br/>' to maintain visual ergonomics.",
                    location=f"{source}",
                ))

    # Rule E3: Mandatory direction TB on Subgraphs
    for offset, sg_name in validate_mermaid_subgraph_direction(body):
        lineno = start + offset + 1
        errors.append(Finding(
            "mermaid-subgraph-direction-tb-mandated",
            f"{source}:{lineno}: subgraph {sg_name!r} is missing explicit 'direction TB' or 'direction TD' declaration. Subgraphs in multi-subgraph diagrams (>= 2) or with >= 4 nodes must declare 'direction TB' or 'direction TD'.",
            location=f"{source}",
        ))

    # Rule E4: Universal Option 3 Compact Block Standard
    for offset, line_content in validate_mermaid_option3_compact_blocks(body, source=source):
        lineno = start + offset + 1
        errors.append(Finding(
            "mermaid-option3-compact-block-mandated",
            f"{source}:{lineno}: exploded port node or subgraph detected ({line_content!r}). Architecture and interface diagrams (SV-1, ICD, STPA) must use Option 3 compact blocks with embedded bulleted port attributes ('• port (DIR)') instead of exploded child port nodes or subgraphs.",
            location=f"{source}",
        ))

    return errors


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

        if kind in ("graph", "flowchart"):
            errors.extend(validate_mermaid_layout_ergonomics(start, body, kind, source=source))

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

            if kind in ("graph", "flowchart", "statediagram", "statediagram-v2"):
                for label_str, conflict in validate_mermaid_quoted_label_content(line_strip):
                    for ch in conflict:
                        err_name = "comma" if ch == "," else "slash"
                        errors.append(Finding(
                            f"mermaid-quoted-label-{err_name}-forbidden",
                            f"{source}:{lineno}: '{ch}' inside a quoted Mermaid label "
                            f"({label_str!r}). GitLab's pinned Glfm renderer rejects this content "
                            f"even though Mermaid headless parses it. Replace the character "
                            f"with a dash or space, or split the label.",
                            location=f"{source}",
                        ))

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
                # wherever it appears -- inside quotes as much as outside. The quoting
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

    def validate_layout_ergonomics(
        self, start: int, body: Sequence[str], kind: str, source: str = "<input>"
    ) -> List[Finding]:
        """Validate layout ergonomics (Rules E1-E4)."""
        return validate_mermaid_layout_ergonomics(start, body, kind, source)

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
                if (
                    "docs/audits" in dirpath
                    or "docs/decisions" in dirpath
                    or "docs/designs" in dirpath
                    or "docs/architecture" in dirpath
                ):
                    continue
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
