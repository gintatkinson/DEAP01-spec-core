//! Reproducer test helpers for GitHub Issue #698 (`compile-sysml` KaTeX & MATLAB operator lowering).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C verification
//! objectives for `crates/compile-sysml/tests/reproducer_e_tests.rs` (refs #698).
//!
//! ## 2. Invariants & Mathematical Properties
//! - Every `$$ ... $$` display-math block in `07_RTA_ARCHITECTURE.md` and `STPA_MATRIX.md` lowers
//!   SysML v2 operators (`and`, `or`, `not`, `implies`, `xor`, `>=`, `<=`, `!=`, `==`, `%`) to
//!   canonical KaTeX tokens (`\land`, `\lor`, `\lnot`, `\implies`, `\oplus`, `\ge`, `\le`, `\neq`,
//!   `=`, `\bmod`), escapes snake_case identifiers in `\mathrm{...}`, maps whitespace-only
//!   expressions to `\mathrm{true}`, contains zero unescaped `%`, and preserves balanced braces.
//! - Every `sldv.assert(` statement across `07_RTA_ARCHITECTURE.md`, `STPA_MATRIX.md`, and
//!   `SLDV_FORMAL_PROOFS.m` lowers SysML v2 operators to valid MATLAB constructs (`&&`, `||`, `~`,
//!   `~=`, `xor(`, `mod(`, `(true)`) with zero raw SysML operators or empty `()` predicates.
//!
//! ## 3. Preconditions
//! - Pure in-memory `SafetyLoweringPkg` AST fixture.
//!
//! ## 4. Postconditions
//! - All KaTeX and MATLAB operator lowering assertions hold.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(A \cdot L)$ single linear scan over generated Markdown and MATLAB buffers.
//! - Space Complexity: $O(A \cdot L)$ bounded in-memory `BTreeMap<String, String>` storage.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Validates `transpile_safety_suite` output.
//! - State: Pure in-memory evaluation with zero shared mutable state.
//! - Action: Executes with zero panics or `.unwrap()` calls.
//! - Resource: Reclaimed at scope exit.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-STPA-KATEX-OPERATOR-LOWERING], [REQ-STPA-MATLAB-SLDV-OPERATOR-LOWERING] | SysML v2: VerificationCaseDef(reproducer_e_tests)

use deap_core::sysml_cst::{ActionDef, CalcDef, ConstraintDef, PackageDef, PartDef};

fn build_issue_698_safety_lowering_pkg() -> PackageDef {
    let supervisor = PartDef {
        name: "InvariantSupervisor".to_string(),
        actions: vec![ActionDef {
            name: "EvaluateGuards".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    PackageDef {
        name: "SafetyLoweringPkg".to_string(),
        part_defs: vec![supervisor],
        calc_defs: vec![
            CalcDef {
                name: "Margin_Check".to_string(),
                expression: Some(
                    "guard_active and not fault_latched implies (total_ticks % cycle_period) == 0"
                        .to_string(),
                ),
                ..Default::default()
            },
            CalcDef {
                name: "Whitespace_Calc".to_string(),
                expression: Some("   ".to_string()),
                ..Default::default()
            },
        ],
        constraint_defs: vec![ConstraintDef {
            name: "Slice_Bound".to_string(),
            expression: "direct_range_slice_count >= 0 and direct_range_slice_count <= 0 or (backup_ok xor override_flag != false)".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn assert_katex_math_block_hygiene(doc_name: &str, math_block: &str) {
    let forbidden_katex_tokens = [
        ">=",
        "<=",
        "!=",
        "==",
        "&&",
        "||",
        " and ",
        " or ",
        " not ",
        " implies ",
        " xor ",
    ];
    for forbidden in forbidden_katex_tokens {
        assert!(
            !math_block.contains(forbidden),
            "{} KaTeX math block must not contain raw SysML operator {:?}:\n$${}$$",
            doc_name,
            forbidden,
            math_block
        );
    }

    let mut prev_escaped = false;
    let mut has_unescaped_percent = false;
    let mut brace_depth: i64 = 0;
    let mut negative_brace_depth = false;
    for ch in math_block.chars() {
        if ch == '{' && !prev_escaped {
            brace_depth += 1;
        } else if ch == '}' && !prev_escaped {
            brace_depth -= 1;
            if brace_depth < 0 {
                negative_brace_depth = true;
            }
        } else if ch == '%' && !prev_escaped {
            has_unescaped_percent = true;
        }
        prev_escaped = ch == '\\' && !prev_escaped;
    }

    assert!(
        !has_unescaped_percent,
        "{} KaTeX math block must not contain unescaped '%' comment character:\n$${}$$",
        doc_name, math_block
    );
    assert!(
        !negative_brace_depth && brace_depth == 0,
        "{} KaTeX math block must have balanced '{{' and '}}' braces (final depth = {}, negative = {}):\n$${}$$",
        doc_name,
        brace_depth,
        negative_brace_depth,
        math_block
    );
}

fn assert_issue_698_matlab_assertions(rta_md: &str, stpa_md: &str, sldv_m: &str) {
    let forbidden_matlab_tokens = [
        " and ",
        " or ",
        " not ",
        " implies ",
        " xor ",
        "!=",
        "%",
        "sldv.assert( (),",
    ];

    for (doc_name, doc_content) in [
        ("07_RTA_ARCHITECTURE.md", rta_md),
        ("STPA_MATRIX.md", stpa_md),
        ("SLDV_FORMAL_PROOFS.m", sldv_m),
    ] {
        for line in doc_content.lines().filter(|l| l.contains("sldv.assert(")) {
            for forbidden in forbidden_matlab_tokens {
                assert!(
                    !line.contains(forbidden),
                    "{} sldv.assert line must not contain raw/invalid token {:?}:\n{}",
                    doc_name,
                    forbidden,
                    line
                );
            }
        }
    }

    assert!(
        sldv_m.lines().any(|l| l.contains("sldv.assert(")),
        "SLDV_FORMAL_PROOFS.m must contain sldv.assert( statements:\n{}",
        sldv_m
    );

    for expected in ["&&", "||", "~", "~=", "xor(", "mod(", "(true)"] {
        assert!(
            sldv_m.contains(expected),
            "SLDV_FORMAL_PROOFS.m must contain lowered MATLAB operator/predicate {:?}:\n{}",
            expected,
            sldv_m
        );
    }
    assert!(
        sldv_m
            .chars()
            .zip(sldv_m.chars().skip(1))
            .any(|(a, b)| a == '~' && b != '='),
        "SLDV_FORMAL_PROOFS.m must contain unary MATLAB negation '~' distinct from '~=':\n{}",
        sldv_m
    );
}

/// Executes Issue #698 checks for KaTeX and MATLAB/SLDV operator lowering across safety artifacts.
///
/// # Arguments
/// * None.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - `Ok(())` when all operator lowering checks pass.
///
/// # Complexity
/// * Time: $O(A \cdot L)$ over the generated Markdown and MATLAB buffers. Space: $O(A \cdot L)$.
///
/// # Failure Modes
/// * Propagates missing-key errors or panics with diagnostic context on operator lowering failure.
pub fn run_issue_698_katex_and_matlab_lowering_checks() -> Result<(), Box<dyn std::error::Error>> {
    let pkg = build_issue_698_safety_lowering_pkg();
    let suite = compile_sysml::stpa::safety_suite::transpile_safety_suite(&pkg, None);
    let rta_md = suite
        .get("07_RTA_ARCHITECTURE.md")
        .ok_or("Missing 07_RTA_ARCHITECTURE.md in transpiled safety suite")?;
    let stpa_md = suite
        .get("STPA_MATRIX.md")
        .ok_or("Missing STPA_MATRIX.md in transpiled safety suite")?;
    let sldv_m = suite
        .get("SLDV_FORMAL_PROOFS.m")
        .ok_or("Missing SLDV_FORMAL_PROOFS.m in transpiled safety suite")?;

    for (doc_name, doc_content) in [
        ("07_RTA_ARCHITECTURE.md", rta_md.as_str()),
        ("STPA_MATRIX.md", stpa_md.as_str()),
    ] {
        let split_count = doc_content.split("$$").count();
        assert!(
            split_count >= 3 && split_count % 2 == 1,
            "{} must contain at least one properly closed $$ ... $$ display-math block (split count = {}):\n{}",
            doc_name,
            split_count,
            doc_content
        );
        for math_block in doc_content.split("$$").skip(1).step_by(2) {
            assert_katex_math_block_hygiene(doc_name, math_block);
        }
    }

    for expected in [
        "\\land",
        "\\lor",
        "\\lnot",
        "\\implies",
        "\\oplus",
        "\\ge",
        "\\le",
        "\\neq",
        "\\bmod",
        "\\mathrm{direct\\_range\\_slice\\_count}",
        "\\mathrm{guard\\_active}",
        "\\mathrm{true}",
    ] {
        assert!(
            rta_md.contains(expected),
            "07_RTA_ARCHITECTURE.md must contain lowered KaTeX token {:?}:\n{}",
            expected,
            rta_md
        );
    }

    assert_issue_698_matlab_assertions(rta_md, stpa_md, sldv_m);
    Ok(())
}
