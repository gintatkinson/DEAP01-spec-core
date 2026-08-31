#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""
Unit tests for KaTeX AST and Mathematical Rendering Integrity Validator.

Validates detection of:
1. Dangling binary operators (/, +, -, *, ^, _ before whitespace, \\quad, \\\\, EOL).
2. Unescaped underscores inside \\text{...}.
3. Embedded physical unit macros inside display math equations.
4. Mismatched delimiters and unclosed alignment environments.
5. Clean, valid symbolic mathematical formulations passing without errors.
"""

import os
import sys
import tempfile
import pytest

# Ensure parity_auditor is on sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PARITY_AUDITOR_DIR = os.path.abspath(os.path.join(SCRIPT_DIR, ".."))
SPEC_ORCH_DIR = os.path.abspath(os.path.join(PARITY_AUDITOR_DIR, ".."))
PROJECT_ROOT = os.path.abspath(os.path.join(SPEC_ORCH_DIR, "..", ".."))
PARITY_AUDITOR_SRC = os.path.join(PARITY_AUDITOR_DIR, "src")

for p in (PARITY_AUDITOR_SRC, PROJECT_ROOT):
    if p not in sys.path:
        sys.path.insert(0, p)

from parity_auditor.core.workspace import WorkspaceRepository
from parity_auditor.validators.katex_validator import KatexValidator, check_katex_text
from parity_auditor.core.findings import Finding


def test_detects_dangling_slash_operators():
    """Verify detection of dangling / binary operators before whitespace, spacing macros, and newlines."""
    bad_md = r"""
# Feature Specification

$$
\begin{aligned}
\dot{\psi} &= \frac{g \tan(\phi)}{V} = \frac{9.80665 \cdot 1.0}{31.0} = 0.3163 \approx 18.12^\circ/ \\
\left| \frac{d\delta_{elevon,L}}{dt} \right| &\le 150.0^\circ/, \quad \left| \frac{d\delta_{elevon,R}}{dt} \right| \le 150.0^\circ/
\end{aligned}
$$

Inline test: $2.5^\circ/ \quad$ and $150.0^\circ/$.
"""
    findings = check_katex_text(bad_md, source="bad_slash.md")
    rule_ids = [f.rule_id for f in findings]
    assert "katex-dangling-binary-operator" in rule_ids

    # Confirm all dangling slash instances were caught
    dangling_findings = [f for f in findings if f.rule_id == "katex-dangling-binary-operator"]
    assert len(dangling_findings) >= 4
    assert any("18.12" in str(f) or "/" in str(f) for f in dangling_findings)


def test_detects_other_dangling_operators():
    """Verify detection of dangling +, -, *, ^, _ operators before delimiters and newlines."""
    bad_md = r"""
$$
\begin{aligned}
a &= b + \\
c &= d * \quad e \\
x &= y_ \\
z &= w^ \quad
\end{aligned}
$$
"""
    findings = check_katex_text(bad_md, source="bad_ops.md")
    dangling_findings = [f for f in findings if f.rule_id == "katex-dangling-binary-operator"]
    assert len(dangling_findings) >= 4


def test_detects_unescaped_underscores_in_text():
    """Verify detection of unescaped underscores inside \\text{...} in display and inline math."""
    bad_md = r"""
# Test Feature

$$
\begin{aligned}
T_{\text{cycle}} &= T_{\text{yaw_disturbance}} + 1.2 \\
\text{bad_identifier_name} &= 5.0
\end{aligned}
$$

Inline equation: $\text{innovation_gate_failed} = 1$.
"""
    findings = check_katex_text(bad_md, source="bad_underscores.md")
    underscore_findings = [f for f in findings if f.rule_id == "katex-unescaped-underscore-in-text"]
    assert len(underscore_findings) >= 3
    assert any("yaw_disturbance" in str(f) for f in underscore_findings)
    assert any("bad_identifier_name" in str(f) for f in underscore_findings)
    assert any("innovation_gate_failed" in str(f) for f in underscore_findings)


def test_detects_embedded_physical_unit_macros():
    """Verify detection of embedded physical unit macros like \\text{ ms}, \\text{ kg}, \\text{ m}, \\text{ bar} inside display math."""
    bad_md = r"""
$$
\begin{aligned}
T_{\text{cycle}} &\le 1.2\text{ ms} \\
m_{\text{total}} &\le 25.0\text{ kg} \\
h_{\text{ceiling}} &= 5000.0\text{ m} \\
P_{\text{pneumatic}} &= 40.0\text{ bar} \\
SNR &= P_{rx} - N_{floor} - \text{NF} \quad [\text{dB}]
\end{aligned}
$$
"""
    findings = check_katex_text(bad_md, source="bad_units.md")
    unit_findings = [f for f in findings if f.rule_id == "katex-embedded-unit-macro"]
    assert len(unit_findings) >= 5
    assert any("ms" in str(f) for f in unit_findings)
    assert any("kg" in str(f) for f in unit_findings)
    assert any("bar" in str(f) for f in unit_findings)


def test_detects_mismatched_delimiters_and_unclosed_environments():
    """Verify detection of unclosed display math delimiters and unclosed \\begin{aligned}."""
    unclosed_aligned_md = r"""
$$
\begin{aligned}
x &= y + z
$$
"""
    findings = check_katex_text(unclosed_aligned_md, source="unclosed_aligned.md")
    assert any(f.rule_id == "katex-unclosed-environment" for f in findings)

    unclosed_display_md = r"""
# Specification

$$
\begin{aligned}
x &= y + z
\end{aligned}

Some trailing markdown without closing display math.
"""
    findings = check_katex_text(unclosed_display_md, source="unclosed_display.md")
    assert any(f.rule_id == "katex-mismatched-delimiters" for f in findings)

    forbidden_align_md = r"""
$$
\begin{align}
x &= y
\end{align}
$$
"""
    findings = check_katex_text(forbidden_align_md, source="forbidden_align.md")
    assert any(f.rule_id == "katex-forbidden-align-environment" for f in findings)

    bare_amp_md = r"""
$$
x &= y + z
$$
"""
    findings = check_katex_text(bare_amp_md, source="bare_amp.md")
    assert any(f.rule_id == "katex-bare-alignment-operator" for f in findings)

    unclosed_inline_md = r"""
Here is an unclosed inline math $x = y without closing dollar.
"""
    findings = check_katex_text(unclosed_inline_md, source="unclosed_inline.md")
    assert any(f.rule_id == "katex-mismatched-delimiters" for f in findings)


def test_detects_forbidden_math_in_tables():
    """Verify detection of forbidden $ ... $ LaTeX math delimiters inside markdown table headers and cells."""
    bad_table_md = r"""
# Table With Math

| Parameter ($S$) | Specification Value | Description |
| :--- | :--- | :--- |
| **Nominal Speed ($V_{cruise}$)** | $31.0\text{ m/s}$ | Cruise speed |
| **Stall Speed** | $V_s \le 24.0\text{ m/s}$ | Minimum speed |
| Normal Row | Clean Value | Plain text |
"""
    findings = check_katex_text(bad_table_md, source="bad_table.md")
    table_math_findings = [f for f in findings if f.rule_id == "katex-forbidden-math-in-table"]
    assert len(table_math_findings) == 3
    assert any("Parameter ($S$)" in str(f) for f in table_math_findings)
    assert any("V_{cruise}" in str(f) for f in table_math_findings)
    assert any("V_s" in str(f) for f in table_math_findings)


def test_detects_table_column_count_mismatch():
    """Verify detection of column count mismatch between table header and delimiter line."""
    mismatched_table_md = r"""
# Mismatched Table

| Col 1 | Col 2 | Col 3 |
| :--- | :--- |
| Val 1 | Val 2 | Val 3 |

| A | B |
| :--- | :--- | :--- | :--- |
| 1 | 2 |
"""
    findings = check_katex_text(mismatched_table_md, source="mismatched_table.md")
    mismatch_findings = [f for f in findings if f.rule_id == "table-column-count-mismatch"]
    assert len(mismatch_findings) == 2
    assert any("3 columns but delimiter row has 2" in str(f) for f in mismatch_findings)
    assert any("2 columns but delimiter row has 4" in str(f) for f in mismatch_findings)


def test_passes_clean_valid_symbolic_math_blocks():
    """Verify that clean, valid symbolic mathematical formulations and clean tables pass with zero findings."""
    clean_md = r"""
# Clean Mathematical Formulations

$$
\begin{aligned}
T_{\text{cycle}} &= T_{\text{inner-rate}} + T_{\text{elevon-mixer}} \le 1.2 \\
T_{\text{frame}} &= \frac{1}{f_{\text{inner}}} = \frac{1}{250.0} = 4.0 \\
U_{\text{core}} &= \frac{T_{\text{cycle}}}{T_{\text{frame}}} \le \frac{1.2}{4.0} = 30.0\% \\
T_{\text{watchdog}} &\le 50.0
\end{aligned}
$$

$$
\begin{aligned}
\mathbf{x} &= \begin{bmatrix} \mathbf{p}^n & \mathbf{v}^n & \mathbf{q}_b^n & \mathbf{b}_a & \mathbf{b}_g \end{bmatrix}^T \in \mathbb{R}^{15} \\
\dot{\mathbf{p}}^n &= \mathbf{v}^n \\
\dot{\mathbf{v}}^n &= \mathbf{C}_b^n (\mathbf{f}^b - \mathbf{b}_a - \boldsymbol{\eta}_a) + \mathbf{g}^n \\
\dot{\mathbf{q}}_b^n &= \frac{1}{2} \boldsymbol{\Omega}(\boldsymbol{\omega}^b - \mathbf{b}_g - \boldsymbol{\eta}_g) \mathbf{q}_b^n \\
\dot{\mathbf{b}}_a &= \boldsymbol{\eta}_{ba}, \quad \dot{\mathbf{b}}_g = \boldsymbol{\eta}_{bg}
\end{aligned}
$$

$$
\begin{aligned}
\Delta \mathbf{p} &= \mathbf{p}_{landmark} - \hat{\mathbf{p}}_{uav}^- \\
\Delta \mathbf{p} &= \hat{\mathbf{p}}_{uav}^+ - \hat{\mathbf{p}}_{uav}^-
\end{aligned}
$$

Inline equations: $D_M^2 \le 9.0$, $\gamma_{gate} \le 3.0\sigma$, $\text{yaw\_disturbance}$, and $\$100.00$ currency.

| Parameter | Specification Value | Description |
| :--- | :--- | :--- |
| **Initial S** | 5 | Severity rating |
| **Delta V (ΔV)** | 0.25 V | Voltage threshold |
| **Wavelength (λ)** | 10⁻⁶ m | Infrared wavelength |
| **Temperature** | -20°C to +50°C | Operational temperature |
| **Limits** | ≥ 26 m/s, ≤ 42 m/s | Safe bounds (→ nominal) |

```python
# Code blocks containing $$ or $ should be ignored
def calculate():
    x = 1 / 2
    return "$$ not math $$"
```
"""
    findings = check_katex_text(clean_md, source="clean_math.md")
    assert len(findings) == 0, f"Expected 0 findings but got: {findings}"


def test_katex_validator_workspace_scanning():
    """Verify KatexValidator directory traversal with WorkspaceRepository."""
    with tempfile.TemporaryDirectory() as tmpdir:
        docs_dir = os.path.join(tmpdir, "docs")
        os.makedirs(docs_dir)

        # Write a clean spec
        with open(os.path.join(docs_dir, "clean.md"), "w", encoding="utf-8") as f:
            f.write(r"""
$$
\begin{aligned}
x &= y + z \\
a &= b / c
\end{aligned}
$$
""")

        # Write a defect spec
        with open(os.path.join(docs_dir, "defect.md"), "w", encoding="utf-8") as f:
            f.write(r"""
$$
\begin{aligned}
x &= 2.5^\circ/ \\
y &= \text{bad_var}
\end{aligned}
$$

| Header A | Header B |
| :--- |
| $val$ | val |
""")

        repo = WorkspaceRepository(tmpdir)
        validator = KatexValidator()
        findings = validator.validate(repo, search_dirs=[docs_dir])

        assert len(findings) >= 4
        rule_ids = {f.rule_id for f in findings}
        assert "katex-dangling-binary-operator" in rule_ids
        assert "katex-unescaped-underscore-in-text" in rule_ids
        assert "table-column-count-mismatch" in rule_ids
        assert "katex-forbidden-math-in-table" in rule_ids
        assert all(isinstance(f, Finding) for f in findings)
