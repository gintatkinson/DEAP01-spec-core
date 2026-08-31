"""KaTeX Mathematical Rendering and AST Integrity Validator.

Enforces LaTeX / KaTeX rendering integrity rules for mathematical expressions
across markdown specifications in the docs/ directory:
1. Dangling binary operators: /, +, -, *, ^, _ before whitespace, \\quad, \\\\,
   or end of line/string (e.g. 2.5^\\circ/ or 150.0^\\circ/).
2. Unescaped underscores inside \\text{...} (e.g. \\text{yaw_disturbance}; must be \\_).
3. Embedded physical unit macros inside display math equations ($$ ... $$)
   (e.g. \\text{ ms}, \\text{ kg}, \\text{ m}, \\text{ bar}).
4. Mismatched delimiters or unclosed \\begin{aligned} and alignment environments.
5. Markdown table math prohibition: strictly ban $ ... $ delimiters in table cells/headers.
6. Markdown table column count consistency: 1:1 match between header and delimiter rows.
"""

import os
import re
from typing import List, Optional, Set, Tuple

from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository

PHYSICAL_UNITS: Set[str] = {
    "ms", "us", "µs", "ns", "sec", "secs", "second", "seconds",
    "mins", "minute", "minutes", "hr", "hrs", "hour", "hours",
    "kg", "g", "mg", "t",
    "m", "cm", "mm", "km", "ft", "in", "nm", "nmi", "mi",
    "bar", "mbar", "pa", "hpa", "kpa", "mpa", "psi",
    "m/s", "mps", "m/s^2", "m/s²", "m/s2", "km/h", "knot", "knots", "kt", "kts", "rad/s", "deg/s",
    "hz", "khz", "mhz", "ghz",
    "deg", "rad",
    "v", "mv", "kv", "a", "ma", "w", "kw", "mw", "wh", "kwh", "ah", "mah",
    "db", "dbm", "dbi",
    "rpm", "fps",
    "kn", "kj"
}

ALIGNMENT_ENVIRONMENTS: Set[str] = {
    "aligned", "matrix", "bmatrix", "pmatrix", "vmatrix", "Vmatrix",
    "cases", "array", "split", "gathered", "gather", "smallmatrix", "submatrix"
}

_DANGLING_OP_RE = re.compile(
    r"(?:(?<!\^)(?P<binop>[/+*])|(?<!\^)(?P<sign>[+\-])|(?P<script>[_^_]))\s*(?:,|;)?\s*(?:\\quad|\\qquad|\\\\(?:\[[^\]]*\])?|$|(?=\$\$|\$))"
)


def _extract_text_macros(s: str) -> List[Tuple[int, str, int]]:
    """Find all \\text{...} occurrences returning (start_idx, inner_content, end_idx)."""
    results = []
    i = 0
    pattern = re.compile(r"\\text\s*\{")
    while i < len(s):
        match = pattern.search(s, i)
        if not match:
            break
        start_idx = match.start()
        brace_start = match.end() - 1
        depth = 1
        curr = brace_start + 1
        while curr < len(s) and depth > 0:
            if s[curr] == "\\":
                curr += 2
                continue
            elif s[curr] == "{":
                depth += 1
            elif s[curr] == "}":
                depth -= 1
                if depth == 0:
                    break
            curr += 1
        if depth == 0:
            inner = s[brace_start + 1 : curr]
            results.append((start_idx, inner, curr + 1))
            i = curr + 1
        else:
            i = brace_start + 1
    return results


def _mask_text_macros(s: str) -> str:
    """Mask out \\text{...} contents with replacement characters to preserve positions and avoid false dangling operators."""
    macros = _extract_text_macros(s)
    if not macros:
        return s
    chars = list(s)
    for start_idx, _, end_idx in macros:
        for idx in range(start_idx, min(end_idx, len(chars))):
            chars[idx] = "X"
    return "".join(chars)


def _validate_display_block(
    display_lines: List[Tuple[int, str]],
    source: str,
    errors: List[Finding],
) -> None:
    """Validate a multi-line or single-line $$ ... $$ display math block."""
    env_stack: List[Tuple[str, int]] = []

    for lineno, raw_line in display_lines:
        line = raw_line.strip()
        if not line:
            continue

        # Environment balance check
        for m in re.finditer(r"\\(begin|end)\{([^}]+)\}", line):
            action, env_name = m.group(1), m.group(2)
            if env_name in ("align", "align*"):
                errors.append(Finding(
                    "katex-forbidden-align-environment",
                    f"{source}:{lineno}: top-level '\\begin{{{env_name}}}' environment inside display math. "
                    f"Use '\\begin{{aligned}}' instead.",
                    location=f"{source}:{lineno}"
                ))
            if action == "begin":
                env_stack.append((env_name, lineno))
            elif action == "end":
                if env_stack and env_stack[-1][0] == env_name:
                    env_stack.pop()
                else:
                    errors.append(Finding(
                        "katex-unclosed-environment",
                        f"{source}:{lineno}: unmatched '\\end{{{env_name}}}' without preceding '\\begin{{{env_name}}}'.",
                        location=f"{source}:{lineno}"
                    ))

        # Bare alignment operator & check
        current_env = env_stack[-1][0] if env_stack else None
        if "&" in line and (not current_env or current_env not in ALIGNMENT_ENVIRONMENTS):
            errors.append(Finding(
                "katex-bare-alignment-operator",
                f"{source}:{lineno}: bare alignment operator '&' outside of alignment environment "
                f"('aligned', 'matrix', 'cases', etc.).",
                location=f"{source}:{lineno}"
            ))

        # \\text{...} macro validations (unescaped underscore and physical unit macros)
        for _, inner, _ in _extract_text_macros(line):
            if re.search(r"(?<!\\)_", inner):
                errors.append(Finding(
                    "katex-unescaped-underscore-in-text",
                    f"{source}:{lineno}: unescaped underscore in '\\text{{{inner}}}'. "
                    f"Underscores in \\text{{}} must be escaped as '\\_' to prevent KaTeX math mode parser errors.",
                    location=f"{source}:{lineno}"
                ))
            cand = inner.strip().lstrip("[").rstrip("]").strip().lower()
            if cand in PHYSICAL_UNITS:
                errors.append(Finding(
                    "katex-embedded-unit-macro",
                    f"{source}:{lineno}: embedded physical unit macro '\\text{{{inner}}}' inside display math equation. "
                    f"Physical units must not be embedded in display math formulas.",
                    location=f"{source}:{lineno}"
                ))

        # Dangling operator validation line-by-line
        code_part = line.split("%", 1)[0]
        masked = _mask_text_macros(code_part)
        m = _DANGLING_OP_RE.search(masked)
        if m:
            errors.append(Finding(
                "katex-dangling-binary-operator",
                f"{source}:{lineno}: dangling binary operator '{m.group(0).strip()}' before whitespace, "
                f"spacing macro, newline or end-of-line: '{line}'. Operators must have valid right-hand operands.",
                location=f"{source}:{lineno}"
            ))

    for env_name, env_lineno in env_stack:
        errors.append(Finding(
            "katex-unclosed-environment",
            f"{source}:{env_lineno}: unclosed '\\begin{{{env_name}}}' environment. Expected matching '\\end{{{env_name}}}'.",
            location=f"{source}:{env_lineno}"
        ))


def _validate_inline_math_line(
    line: str,
    lineno: int,
    source: str,
    errors: List[Finding],
) -> None:
    """Validate inline $ ... $ math expressions on a single markdown line."""
    clean_line = re.sub(r"`[^`]*`", "", line)
    escaped_line = re.sub(r"\\\$", "  ", clean_line)

    dollar_count = escaped_line.count("$")
    if dollar_count % 2 != 0:
        errors.append(Finding(
            "katex-mismatched-delimiters",
            f"{source}:{lineno}: unclosed inline math delimiter '$' on line. Inline math must be closed on the same line.",
            location=f"{source}:{lineno}"
        ))
        return

    for m in re.finditer(r"\$([^$]+)\$", escaped_line):
        content = m.group(1)
        for _, inner, _ in _extract_text_macros(content):
            if re.search(r"(?<!\\)_", inner):
                errors.append(Finding(
                    "katex-unescaped-underscore-in-text",
                    f"{source}:{lineno}: unescaped underscore in '\\text{{{inner}}}' in inline math expression. "
                    f"Escape as '\\_'.",
                    location=f"{source}:{lineno}"
                ))
        masked = _mask_text_macros(content)
        d_m = _DANGLING_OP_RE.search(masked)
        if d_m:
            errors.append(Finding(
                "katex-dangling-binary-operator",
                f"{source}:{lineno}: dangling binary operator '{d_m.group(0).strip()}' in inline math expression: '${content}$'.",
                location=f"{source}:{lineno}"
            ))


def _split_table_cells(line: str) -> List[str]:
    """Split a markdown table row into trimmed cell contents, respecting backtick code spans."""
    stripped = line.strip()
    if stripped.startswith("|"):
        stripped = stripped[1:]
    if stripped.endswith("|") and not stripped.endswith(r"\|"):
        stripped = stripped[:-1]

    # Mask code spans so pipes inside backticks do not split columns
    masked = re.sub(r"`[^`]*`", lambda m: "X" * len(m.group(0)), stripped)

    split_indices = [-1]
    for m in re.finditer(r"(?<!\\)\|", masked):
        split_indices.append(m.start())
    split_indices.append(len(stripped))

    cells = []
    for i in range(len(split_indices) - 1):
        start = split_indices[i] + 1
        end = split_indices[i + 1]
        cells.append(stripped[start:end].strip())
    return cells


def _is_delimiter_row(line: str) -> bool:
    """Return True if the line matches a markdown table delimiter row (e.g. | :--- | ---: |)."""
    stripped = line.strip()
    if not stripped or "|" not in stripped:
        return False
    cells = _split_table_cells(stripped)
    if not cells:
        return False
    delimiter_cell_re = re.compile(r"^:?-+:?$")
    return all(delimiter_cell_re.match(cell) is not None for cell in cells)


def _has_unescaped_dollar(line: str) -> bool:
    """Return True if line contains an unescaped dollar sign outside inline code spans."""
    clean_line = re.sub(r"`[^`]*`", "", line)
    escaped_line = re.sub(r"\\\$", "  ", clean_line)
    return "$" in escaped_line


def check_katex_text(text: str, source: str = "<input>") -> List[Finding]:
    """Scan markdown text for LaTeX / KaTeX mathematical formatting violations and table integrity."""
    errors: List[Finding] = []
    lines = text.splitlines()
    in_code_fence = False
    fence_marker = ""
    in_display_math = False
    display_start_lineno = 0
    display_lines: List[Tuple[int, str]] = []
    in_table = False
    delim_linenos: Set[int] = set()

    for idx, line in enumerate(lines):
        lineno = idx + 1
        stripped = line.strip()

        # Fenced code block handling
        if stripped.startswith("```") or stripped.startswith("~~~"):
            marker = stripped[:3]
            if not in_code_fence:
                in_code_fence = True
                fence_marker = marker
            elif marker == fence_marker:
                in_code_fence = False
            in_table = False
            continue

        if in_code_fence:
            continue

        # Display math $$ handling
        if not in_table and "$$" in line:
            parts = line.split("$$")
            if len(parts) >= 3 and not in_display_math:
                # Single-line display math block: $$ content $$
                block_content = parts[1]
                _validate_display_block([(lineno, block_content)], source, errors)
                continue
            elif not in_display_math:
                in_display_math = True
                display_start_lineno = lineno
                after = line.split("$$", 1)[1]
                display_lines = [(lineno, after)]
                continue
            else:
                in_display_math = False
                before = line.split("$$", 1)[0]
                display_lines.append((lineno, before))
                _validate_display_block(display_lines, source, errors)
                display_lines = []
                continue

        if in_display_math:
            display_lines.append((lineno, line))
            continue

        # Markdown Table handling
        if idx in delim_linenos:
            # Current line is the delimiter line of a table
            if _has_unescaped_dollar(line):
                errors.append(Finding(
                    "katex-forbidden-math-in-table",
                    f"{source}:{lineno}: forbidden LaTeX math delimiter '$' inside table delimiter row: '{stripped}'. "
                    f"Markdown tables must use plain text and Unicode symbols instead.",
                    location=f"{source}:{lineno}"
                ))
            continue

        # Check if current line starts a table (header followed by delimiter row)
        if stripped and "|" in stripped and idx + 1 < len(lines) and _is_delimiter_row(lines[idx + 1]):
            in_table = True
            delim_linenos.add(idx + 1)
            header_cells = _split_table_cells(line)
            delim_cells = _split_table_cells(lines[idx + 1])

            if len(header_cells) != len(delim_cells):
                errors.append(Finding(
                    "table-column-count-mismatch",
                    f"{source}:{lineno + 1}: table column count mismatch: header has {len(header_cells)} columns "
                    f"but delimiter row has {len(delim_cells)} columns. Header and delimiter column counts must match 1:1.",
                    location=f"{source}:{lineno + 1}"
                ))

            if _has_unescaped_dollar(line):
                errors.append(Finding(
                    "katex-forbidden-math-in-table",
                    f"{source}:{lineno}: forbidden LaTeX math delimiter '$' inside table header: '{stripped}'. "
                    f"Markdown tables must use plain text and Unicode symbols instead.",
                    location=f"{source}:{lineno}"
                ))
            continue

        if in_table:
            if not stripped or "|" not in stripped:
                in_table = False
            else:
                if _has_unescaped_dollar(line):
                    errors.append(Finding(
                        "katex-forbidden-math-in-table",
                        f"{source}:{lineno}: forbidden LaTeX math delimiter '$' inside table row: '{stripped}'. "
                        f"Markdown tables must use plain text and Unicode symbols instead.",
                        location=f"{source}:{lineno}"
                    ))
                continue

        # Inline math checks for non-display, non-table math lines
        _validate_inline_math_line(line, lineno, source, errors)

    if in_display_math:
        errors.append(Finding(
            "katex-mismatched-delimiters",
            f"{source}:{display_start_lineno}: unclosed display math block '$$'. Every display math block must be closed with matching '$$'.",
            location=f"{source}:{display_start_lineno}"
        ))

    return errors


class KatexValidator(IValidator):
    """Validator that scans markdown files in docs/ for LaTeX / KaTeX rendering integrity."""

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[Finding]:
        search_dirs = kwargs.get("search_dirs")
        if not search_dirs:
            search_dirs = [
                os.path.join(repo.workspace_dir, "docs"),
            ]

        errors: List[Finding] = []
        for root in search_dirs:
            if not os.path.isdir(root):
                continue
            for dirpath, _dirnames, filenames in os.walk(root):
                for name in sorted(filenames):
                    if not name.endswith(".md") or name.startswith("."):
                        continue
                    path = os.path.join(dirpath, name)
                    try:
                        with open(path, "r", encoding="utf-8", errors="replace") as fh:
                            content = fh.read()
                    except OSError:
                        continue
                    rel = os.path.relpath(path, repo.workspace_dir)
                    errors.extend(check_katex_text(content, source=rel))
        return errors
