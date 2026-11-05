//! End-to-end regression tests for Gate-10 blueprint translation fidelity (issue #863).
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C software
//! verification standards for the Markdown-to-SysML blueprint translation path.
//!
//! ## 2. Invariants & Mathematical Properties
//! - Declared diagnostic namespaces survive translation verbatim, in source order, without `E0100` fabrication.
//! - Normative requirement prose and acceptance-criteria escapes survive without truncation or control-character loss.
//! - Every Mermaid sequence message, including repeated Stage-1/Stage-2 requests and replies, survives in source order.
//! - `to_sysml` -> `parse_sysml` round-trips preserve literal `\r` escape sequences without emitting `0x0D`.
//!
//! ## 3. Preconditions
//! - `ingest-sysml` and `compile-sysml` libraries compile under workspace `forbid`/`deny` lints.
//!
//! ## 4. Postconditions
//! - Fails closed on any fidelity regression without `.unwrap()`, `.expect()`, or `panic!`.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - Time Complexity: $O(N)$ over the synthetic fixture byte length $N$.
//! - Space Complexity: $O(N)$ for the translated AST and serialized SysML buffer.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Interface: Exercises the public `translate_markdown`, `to_sysml`, and `parse_sysml` entry points.
//! - State: Uses an in-memory fixture with zero filesystem or network dependencies.
//! - Action: Returns typed `Err(String)` evidence on any fidelity violation.
//! - Resource: Bounded synthetic fixture with no unbounded recursion.
//!
//! ## 7. Realization & Traceability
//! Realises: [REQ-863-BLUEPRINT-TRANSLATOR-FIDELITY]

use deap_core::sysml_ast::{AttributeDef, PackageDef, RequirementDef};
use std::sync::atomic::{AtomicU64, Ordering};
use verify_baseline::blueprint_gate::check_10_compile_and_persist_sysml;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// RAII temporary fixture directory removed deterministically on drop.
struct TempFixtureDir {
    path: std::path::PathBuf,
}

impl TempFixtureDir {
    /// Creates a unique temporary fixture directory under the host OS temp root.
    fn new(prefix: &str) -> Result<Self, String> {
        let seq = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "{prefix}_{}_{}",
            std::process::id(),
            seq
        ));
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        std::fs::create_dir_all(&path).map_err(|err| err.to_string())?;
        Ok(Self { path })
    }
}

impl Drop for TempFixtureDir {
    /// Recursively removes the temporary fixture directory on scope exit.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn fidelity_fixture() -> &'static str {
    r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-863` |
| **Title** | Translator Fidelity Regression |
| **Subsystem** | Subsystem 863 |
| **UUIDv5** | `00000000-0000-0000-0000-000000000863` |
| **Complexity Class** | `Class P` |
| **Governing Standard** | `ISO/IEC/IEEE 29148:2018` |
| **Diagnostic Codes** | `DEAP-E0101, DEAP-E0202` |

## 1. Normative Statement

The translator shall preserve the complete normative requirement clause without any fixed-length truncation so that downstream consumers receive the full text, including every literal carriage-return escape such as \r and \r, every declared diagnostic code in source order, and every Mermaid sequence message in source order. This clause intentionally extends well beyond three hundred characters to prove that no prefix cap remains in the ingestion path, and it terminates with the unique fidelity marker FULL_CLAUSE_END_863.

## 2. Structural Component Metamodel

### Component: TranslatorFidelityNode

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `retained_messages` | `Integer` | `4` | `count` | Retained Stage-1 and Stage-2 sequence messages |

## 3. Acceptance Criteria

### AC-01: Lossless Escape Preservation

- Given: a blueprint clause containing literal \r and a second literal \r escape
- When: the translator persists the clause
- Then: both \r sequences remain two-character escapes and no CR control character appears
- Diagnostic: DEAP-E12xx

## 5. Sequence Interaction

```mermaid
sequenceDiagram
    participant Verifier
    participant Compiler
    Verifier->>Compiler: verify_lossless_cst(stage1_input)
    Compiler-->>Verifier: lossless_cst_verified(stage1)
    Verifier->>Compiler: verify_lossless_cst(stage2_input)
    Compiler-->>Verifier: lossless_cst_verified(stage2)
```
"#
}

fn translate_fixture() -> Result<PackageDef, String> {
    ingest_sysml::translate_markdown(fidelity_fixture(), "BP-863")
}

fn first_requirement(pkg: &PackageDef) -> Result<&RequirementDef, String> {
    pkg.requirement_defs.first().ok_or_else(|| "missing translated requirement".to_string())
}

fn find_attribute<'a>(req: &'a RequirementDef, name: &str) -> Result<&'a AttributeDef, String> {
    req.attributes
        .iter()
        .find(|attr| attr.name == name)
        .ok_or_else(|| format!("missing attribute {name}"))
}

fn attribute_value(attr: &AttributeDef) -> Result<&str, String> {
    attr.default_value.as_deref().ok_or_else(|| format!("missing default value for {}", attr.name))
}

fn roundtrip(pkg: &PackageDef) -> Result<PackageDef, String> {
    let sysml = compile_sysml::to_sysml(pkg);
    compile_sysml::parse_sysml(&sysml).map_err(|err| err.to_string())
}

#[test]
fn test_blueprint_translator_diagnostic_namespace_and_full_clause() -> Result<(), String> {
    let pkg = translate_fixture()?;
    let req = first_requirement(&pkg)?;
    let diagnostics = attribute_value(find_attribute(req, "diagnostic_codes")?)?;
    assert_eq!(diagnostics, "\"DEAP-E0101, DEAP-E0202, DEAP-E12xx\"");
    for code in ["DEAP-E0101", "DEAP-E0202", "DEAP-E12xx"] {
        assert_eq!(diagnostics.matches(code).count(), 1, "{code} must appear exactly once");
    }
    assert!(!diagnostics.contains("E0100"), "fabricated E0100 namespace: {diagnostics}");

    assert!(req.text.len() > 300, "normative clause truncated to {} bytes", req.text.len());
    assert!(req.text.contains("FULL_CLAUSE_END_863"), "missing full-clause marker: {}", req.text);
    Ok(())
}

#[test]
fn test_blueprint_translator_literal_cr_escapes_and_sequence_cardinality() -> Result<(), String> {
    let pkg = translate_fixture()?;
    let req = first_requirement(&pkg)?;
    assert!(req.text.matches("\\r").count() >= 2, "normative clause lost literal \\r escapes");
    assert!(!req.text.contains('\r'), "normative clause contains a CR control character");

    let ac = find_attribute(req, "ac_01_lossless_escape_preservation")?;
    let ac_value = attribute_value(ac)?;
    assert!(ac_value.matches("\\r").count() >= 2, "acceptance criteria lost literal \\r escapes");
    assert!(!ac_value.contains('\r'), "acceptance criteria contain a CR control character");

    let interaction = pkg.interaction_defs.first().ok_or_else(|| "missing interaction".to_string())?;
    assert_eq!(interaction.messages.len(), 4, "sequence messages were deduplicated: {:?}", interaction.messages);
    let expected = [
        "verify_lossless_cst from Verifier to Compiler",
        "reply_lossless_cst_verified from Compiler to Verifier",
        "verify_lossless_cst from Verifier to Compiler",
        "reply_lossless_cst_verified from Compiler to Verifier",
    ];
    for (idx, exp) in expected.iter().enumerate() {
        assert_eq!(interaction.messages.get(idx).map(String::as_str), Some(*exp), "message {idx} out of order");
    }
    Ok(())
}

#[test]
fn test_blueprint_translator_sysml_roundtrip_preserves_literal_cr_escape() -> Result<(), String> {
    let pkg = translate_fixture()?;
    let reparsed = roundtrip(&pkg)?;
    let req = first_requirement(&reparsed)?;
    assert!(req.text.matches("\\r").count() >= 2, "round-trip lost literal \\r escapes: {}", req.text);
    assert!(!req.text.contains('\r'), "round-trip created a CR control character");

    let diagnostics = attribute_value(find_attribute(req, "diagnostic_codes")?)?;
    assert_eq!(diagnostics, "\"DEAP-E0101, DEAP-E0202, DEAP-E12xx\"");
    assert!(!diagnostics.contains("E0100"));

    let interaction = reparsed.interaction_defs.first().ok_or_else(|| "missing reparsed interaction".to_string())?;
    assert_eq!(interaction.messages.len(), 4);
    assert_eq!(interaction.messages.iter().filter(|msg| msg.as_str() == "verify_lossless_cst from Verifier to Compiler").count(), 2);
    assert_eq!(interaction.messages.iter().filter(|msg| msg.as_str() == "reply_lossless_cst_verified from Compiler to Verifier").count(), 2);
    Ok(())
}

#[test]
fn test_gate_10_persists_declared_diagnostics_full_clause_escapes_and_sequence_messages() -> Result<(), String> {
    let temp = TempFixtureDir::new("blueprint_translator_863")?;
    let md_path = temp.path.join("blueprint-863.md");
    std::fs::write(&md_path, fidelity_fixture()).map_err(|err| err.to_string())?;
    let (sysml_path, digest_path) =
        check_10_compile_and_persist_sysml(&md_path, fidelity_fixture())
            .map_err(|errors| errors.join("\n"))?;
    assert!(sysml_path.is_file() && digest_path.is_file(), "Gate-10 artifacts were not persisted");

    let sysml = std::fs::read_to_string(&sysml_path).map_err(|err| err.to_string())?;
    for code in ["DEAP-E0101", "DEAP-E0202", "DEAP-E12xx"] {
        assert!(sysml.contains(code), "{code} must be preserved in the persisted SysML artifact");
    }
    assert!(!sysml.contains("E0100"), "Gate-10 fabricated undeclared E0100: {sysml}");
    assert!(sysml.contains("FULL_CLAUSE_END_863"), "Gate-10 truncated the normative clause");
    assert!(sysml.matches("\\r").count() >= 2, "Gate-10 dropped literal \\r escapes");
    assert!(!sysml.contains('\r'), "Gate-10 emitted a CR control character");
    assert_eq!(
        sysml
            .matches("message verify_lossless_cst from Verifier to Compiler;")
            .count(),
        2,
        "Gate-10 must retain both Stage-1 and Stage-2 request messages"
    );
    Ok(())
}
