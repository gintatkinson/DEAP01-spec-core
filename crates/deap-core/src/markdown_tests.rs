#![cfg(test)]
#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! Unit tests for `deap_core::markdown` token scanning, KaTeX math blocks, Mermaid validation, and CommonMark AST re-exports.
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C deterministic verification objectives for `crates/deap-core/src/markdown.rs`.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Exercises GLFM inline math stripping, `$$` display math parity, alignment environment discipline, Mermaid header/fence validation, and CommonMark AST integration.
//!
//! ## 3. Preconditions
//! - Valid UTF-8 Markdown test fixtures.
//!
//! ## 4. Postconditions
//! - Verifies exact diagnostic emission and catalog parity across `AllowedAlignmentEnv` and `ValidMermaidHeader`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ per test fixture.
//! - Space Complexity: $O(N)$ bounded test buffers.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Deterministic in-memory assertions with zero external side effects.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-0401], [REQ-0402], [REQ-0608]

#[cfg(test)]
mod tests {
    use super::*;

#[test]
fn test_glfm_inline_math_does_not_synthesize_display_math() {
    let sample = "The constraint `$``v_{lim} = 10``$` applies along with `$``a_{lim} = 2``$`.";
    let cleaned = strip_code_and_glfm(sample);
    assert!(
        !cleaned.contains("$$"),
        "GLFM math stripping must not synthesize false '$$' display delimiters! Cleaned: {:?}",
        cleaned
    );
    let errors = check_latex_katex_syntax(sample, "test.md");
    assert!(
        errors.is_empty(),
        "Expected 0 errors on valid GLFM math, got: {:?}",
        errors
    );
}

#[test]
fn test_bare_alignment_inside_aligned_passes() {
    let sample = r#"
$$
\begin{aligned}
x &= 1 \\
y &= 2
\end{aligned}
$$
"#;
    let errors = check_latex_katex_syntax(sample, "test.md");
    assert!(
        errors.is_empty(),
        "Expected 0 errors for alignment operator '&' inside \\begin{{aligned}}, got: {:?}",
        errors
    );
}

#[test]
fn test_bare_alignment_outside_aligned_fails() {
    let sample = r#"
$$
x & 1 \\
y & 2
$$
"#;
    let errors = check_latex_katex_syntax(sample, "test.md");
    assert!(
        !errors.is_empty(),
        "Expected error for bare '&' outside alignment environment"
    );
    let first_err = errors.first().map(|s| s.as_str()).unwrap_or("");
    assert!(
        first_err.contains("Bare alignment operator '&' outside alignment environment"),
        "Error was: {}",
        first_err
    );
}

const REQ_0099_SAMPLE: &str = r#"---
id: REQ-0099
title: "Base Dimension Tuple Representation & Generalized Conjugate Power Pairs"
subsystem: "Subsystem 5: 7D Physical Metrology & Abstract Flow Conservation Networks"
uuidv5: 83cab42d-eb92-56bf-91cd-0f4bc1c7a675
---

# [REQ-0099] Base Dimension Tuple Representation & Generalized Conjugate Power Pairs

| Metadata Field | Contract Specification |
| :--- | :--- |
| **Requirement ID** | `REQ-0099` |
| **Deterministic UUIDv5** | `83cab42d-eb92-56bf-91cd-0f4bc1c7a675` |
| **Subsystem** | Subsystem 5: 7D Physical Metrology & Abstract Flow Conservation Networks |
| **Complexity Class** | Class $\mathbf{P}$ ($O(1)$ coordinate check, $O(N)$ lowering) |
| **NP Frontier Anchor** | N/A (Deterministic Polynomial Foundation) |
| **Diagnostic Code Bindings** | `E0199`, `E0300`, `E0301` (per REQ-0191) |
| **Governing Standard** | IEEE 29148-2018 §8.4 / RFC 2119 / ISO 80000-1 |

---

## 1. Normative Statement

The engine SHALL represent dimensions as irreducible rational 7-tuples over $\mathbb{Q}^7$ spanning ISQ base dimensions ($M, L, T, I, \Theta, N, J$).

Lowering MUST enforce:
1. **Irreducible Exponents**: Coordinates MUST be irreducible pairs $(p, q)$ ($p \in \mathbb{Z}, q \in \mathbb{Z}^+, \gcd(|p|, q) = 1$). Denominators $q \le 0$ emit `E0300`.
2. **Dimensionless Zero**: Zero vector $\mathbf{0} \in \mathbb{Q}^7$ acts as additive identity.
3. **Conjugate Pairs**: Conservative ports (`Port_Alpha`, `Interface_Gamma`) MUST bind Across (`Across_Var`, $e$) and Through (`Through_Var`, $f$) features.
4. **Power Invariant**: Conjugate dimensions MUST satisfy $``\mathbf{d}(e) + \mathbf{d}(f) \equiv \mathbf{d}_{\text{Power}} = (1,2,-3,0,0,0,0) \in \mathbb{Q}^7``$. Violations emit `E0301`.
5. **Non-Conservative Exemption**: Signal interfaces (REQ-0106) SHALL NOT enforce the power invariant.
6. **Homogeneity**: Operations ($``Q_1 \pm Q_2``$, $``Q_1 == Q_2``$) require $``\mathbf{d}(Q_1) = \mathbf{d}(Q_2)``$; mismatches emit `E0300`.

---

## 2. Formal Invariant

Lattice invariants, power duality, and diagnostics:

```math
\begin{aligned}
\mathbf{d} &\in \mathbb{Q}^7,\; d_i = p_i/q_i,\; q_i \in \mathbb{Z}^+,\; \gcd(|p_i|, q_i) = 1,\; \mathbf{0} = (0,\dots,0) \\
\mathbf{d}_{\text{Power}} &= (1,2,-3,0,0,0,0),\; \forall C = \langle e, f \rangle \in \mathcal{C}_{\text{pair}} \implies \mathbf{d}(e) + \mathbf{d}(f) \equiv \mathbf{d}_{\text{Power}} \\
\mathbf{d}(e) + \mathbf{d}(f) &\ne \mathbf{d}_{\text{Power}} \implies \text{Diagnostic}(C, \text{E0301}) \\
q_i \le 0 &\vee (\mathbf{d}(Q_1) \ne \mathbf{d}(Q_2) \wedge Q_1 \pm Q_2) \implies \text{Diagnostic}(n, \text{E0300}) \\
\Delta_n &= \text{UUIDv5}(\text{NS}_{\text{DEAP}}, \text{FQN}(n))
\end{aligned}
```

| Symbol / Term | Domain / Formal Definition |
| :--- | :--- |
| $\mathbf{d} \in \mathbb{Q}^7$ | Irreducible rational exponent 7-tuple over ISQ |
| $``p_i, q_i``$ | Rational numerator and positive denominator ($\gcd=1$) |
| $``\mathbf{0}, \mathbf{d}_{\text{Power}}``$ | Zero vector and Power vector $(1,2,-3,0,0,0,0)$ |
| $``\mathcal{C}_{\text{pair}}, \Delta_n``$ | Conjugate pair descriptors and UUIDv5 hash |
| $e, f$ | Across effort and Through flow features |

---

## 3. Computational Complexity & Algorithmic Bounds

- **Complexity Class:** $\mathbf{P}$ (Deterministic Polynomial)
- **Asymptotic Bound:** $O(1)$ arithmetic; $``O(N_P + N_C)``$ AST traversal over $``N_P``$ ports and $``N_C``$ pairs.
- **Tractable Subclass & Frontier:** Group arithmetic in $\mathbb{Q}^7$ runs in $O(1)$, bypassing NP frontiers (NP-1--NP-8) to ground flow networks (NP-6). Work-stealing scales as $``O((N_P + N_C) / N_{\text{max\_workers}} + \log N_{\text{max\_workers}})``$.

---

## 4. Verification & Conformance Criteria (IEEE 29148 Acceptance Criteria)

### AC-01: Canonical Rational Reduction & Zero Identity
- **Given:** Rational coordinates $(2/4,6/2,-9/3,0/1,0/1,0/1,0/1)$ and dimensionless scalar.
- **When:** Lowering dimension AST nodes.
- **Then:** Engine reduces coordinates to $((1,2),(3,1),(-3,1),(0,1),(0,1),(0,1),(0,1))$ and maps scalar to $\mathbf{0}$.
- **Diagnostic:** Diagnostic sink remains empty.

### AC-02: Conservative Conjugate Power Pair Conformance
- **Given:** Interface `Interface_Gamma` with Across `Across_Var` and Through `Through_Var`.
- **When:** Validating conjugate pairs.
- **Then:** Dimensional sum evaluates to $``\mathbf{d}_{\text{Power}} = (1,2,-3,0,0,0,0)``$ with valid flag set true.
- **Diagnostic:** Diagnostic sink remains empty.

### AC-03: Conjugate Power Product Violation Rejection
- **Given:** Interface `Interface_Gamma` with Across $(1,1,-2,0,0,0,0)$ and Through $(0,0,1,0,0,0,0)$.
- **When:** Evaluating power invariant ($``\mathbf{d}_e + \mathbf{d}_f \ne \mathbf{d}_{\text{Power}}``$).
- **Then:** Lowering rejects definition with `E0301` and assigns sentinel $``\bot_{\text{pair}}``$.
- **Diagnostic:** `E0301`.

### AC-04: Static Homogeneity & Delimiter Recovery
- **Given:** Denominator $q = 0$ or mismatched addition `param_x + param_y`.
- **When:** Parsing and validation execute.
- **Then:** Compiler emits `E0300`, assigns sentinel $``\bot_{\text{dim}}``$, recovers on delimiter (`;`), and halts at $``C_{\text{max\_errors}}``$ with `E0199`.
- **Diagnostic:** `E0300`, `E0199`.

---

*Traceability: Elaborated under IEEE 29148-2018 and the DEAP Compiler Streamlined Charter.*
"#;

#[test]
fn test_schema_req_0099_zero_errors() {
    let katex_errors = check_latex_katex_syntax(REQ_0099_SAMPLE, "schema/REQ-0099.md");
    assert!(
        katex_errors.is_empty(),
        "schema/REQ-0099.md must have 0 KaTeX errors, got: {:?}",
        katex_errors
    );

    let mermaid_errors = check_mermaid_syntax(REQ_0099_SAMPLE, "schema/REQ-0099.md");
    assert!(
        mermaid_errors.is_empty(),
        "schema/REQ-0099.md must have 0 Mermaid errors, got: {:?}",
        mermaid_errors
    );
}

#[test]
fn test_mermaid_unclosed_fence_detection() {
    let sample = "```mermaid\ngraph TD\nA --> B\n";
    let errors = check_mermaid_syntax(sample, "test.md");
    assert!(!errors.is_empty(), "Should detect unclosed mermaid fence");
    assert!(errors
        .first()
        .is_some_and(|e| e.contains("unclosed ```mermaid fence")));
}

#[test]
fn test_mermaid_invalid_header_detection() {
    let sample = "```mermaid\ninvalidHeader TD\nA --> B\n```";
    let errors = check_mermaid_syntax(sample, "test.md");
    assert!(!errors.is_empty(), "Should detect invalid diagram header");
    assert!(errors
        .first()
        .is_some_and(|e| e.contains("missing or invalid Mermaid diagram header")));
}

#[test]
fn test_mermaid_unquoted_angle_bracket_detection() {
    let sample = "```mermaid\ngraph TD\nA -->|x < 5| B\n```";
    let errors = check_mermaid_syntax(sample, "test.md");
    assert!(!errors.is_empty(), "Should detect unquoted '<' character");
    assert!(errors
        .first()
        .is_some_and(|e| e.contains("unquoted '<' character")));

    let quoted_sample = "```mermaid\ngraph TD\nA -->|\"x < 5\"| B\n```";
    let clean_errors = check_mermaid_syntax(quoted_sample, "test.md");
    assert!(
        clean_errors.is_empty(),
        "Quoted angle bracket should pass, got: {:?}",
        clean_errors
    );
}

#[test]
fn test_catalog_structs_contains_and_iter() {
    let align_catalog: AllowedAlignmentEnvsCatalog = ALLOWED_ALIGNMENT_ENVS;
    assert!(align_catalog.contains(&"aligned"));
    assert!(align_catalog.contains(&"pmatrix"));
    assert!(!align_catalog.contains(&"align"));
    assert_eq!(align_catalog.iter().count(), 17);

    let mermaid_catalog: ValidMermaidHeadersCatalog = VALID_MERMAID_HEADERS;
    assert!(mermaid_catalog.contains("flowchart"));
    assert!(mermaid_catalog.contains("sequencediagram"));
    assert!(!mermaid_catalog.contains("invalid"));
    assert_eq!(mermaid_catalog.iter().count(), 18);
}

#[test]
fn test_allowed_alignment_env_enum_cardinality_order_and_catalog_parity() {
    let expected = vec![
        "aligned",
        "alignedat",
        "matrix",
        "pmatrix",
        "bmatrix",
        "Bmatrix",
        "vmatrix",
        "Vmatrix",
        "cases",
        "dcases",
        "rcases",
        "array",
        "split",
        "gathered",
        "gather",
        "subarray",
        "smallmatrix",
    ];
    assert_eq!(AllowedAlignmentEnv::ALL.len(), 17);
    let via_enum: Vec<&'static str> = AllowedAlignmentEnv::ALL
        .iter()
        .map(|e| e.as_str())
        .collect();
    assert_eq!(via_enum, expected);
    let via_catalog: Vec<&'static str> = ALLOWED_ALIGNMENT_ENVS.iter().collect();
    assert_eq!(via_catalog, expected);
    for e in AllowedAlignmentEnv::ALL {
        assert_eq!(e.as_str_ref(), e.as_str());
        assert!(ALLOWED_ALIGNMENT_ENVS.contains(&e.as_str()));
    }
    assert!(!ALLOWED_ALIGNMENT_ENVS.contains(&"align"));
    assert!(!ALLOWED_ALIGNMENT_ENVS.contains(&"ALIGNED"));
}

#[test]
fn test_valid_mermaid_header_enum_cardinality_order_and_catalog_parity() {
    let expected = vec![
        "graph",
        "flowchart",
        "sequencediagram",
        "statediagram",
        "statediagram-v2",
        "classdiagram",
        "erdiagram",
        "gantt",
        "pie",
        "gitgraph",
        "c4context",
        "journey",
        "quadrantchart",
        "xychart-beta",
        "mindmap",
        "timeline",
        "sankey-beta",
        "requirementdiagram",
    ];
    assert_eq!(ValidMermaidHeader::ALL.len(), 18);
    let via_enum: Vec<&'static str> =
        ValidMermaidHeader::ALL.iter().map(|h| h.as_str()).collect();
    assert_eq!(via_enum, expected);
    let via_catalog: Vec<&'static str> = VALID_MERMAID_HEADERS.iter().collect();
    assert_eq!(via_catalog, expected);
    for h in ValidMermaidHeader::ALL {
        assert_eq!(h.as_str_ref(), h.as_str());
        assert!(VALID_MERMAID_HEADERS.contains(h.as_str()));
    }
    assert!(!VALID_MERMAID_HEADERS.contains("sequenceDiagram"));
    assert!(!VALID_MERMAID_HEADERS.contains("architecture-beta"));
}

#[test]
fn test_markdown_catalogs_free_of_static_str_iter_evasion() {
    let source = include_str!("markdown.rs");
    let needle_into_iter = ["IntoIter<&'static", " str"].concat();
    let needle_static_slice = ["&'static [&'static", " str]"].concat();
    let hits: Vec<String> = source
        .lines()
        .enumerate()
        .filter(|(_, l)| l.contains(&needle_into_iter) || l.contains(&needle_static_slice))
        .map(|(i, l)| format!("line {}: {}", i + 1, l.trim()))
        .collect();
    assert!(
        hits.is_empty(),
        "markdown.rs still carries untyped string-array catalog evasions:\n{}",
        hits.join("\n")
    );
}

#[test]
fn test_markdown_ast_reexports_from_markdown_module() -> Result<(), String> {
    let md = "# Heading\n\n| ColA | ColB |\n| :--- | :--- |\n| 1 | 2 |\n";
    let ast: ParsedMarkdownAst = parse_markdown_ast(md);
    assert_eq!(ast.sections.len(), 1);
    assert_eq!(ast.tables.len(), 1);
    let formatted = format_markdown_cmark(md)?;
    assert!(formatted.contains("Heading"));
    Ok(())
}
}
