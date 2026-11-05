use super::struct_check_a_tests::join_pair;
use super::structural::*;

/// Builds a compliant tiered Rustdoc fixture for Check 34 rule (f) positive verification.
fn clean_tiered_rustdoc_fixture() -> String {
    let mut s = String::from(
        "//! Sample parity module documented with a Tier A 4-Block Contract.\n\
         //! Safety & Traceability: Realises: [REQ-PARITY-MOD]\n\
         //! Contract: Enforces deterministic baseline parity checks across audited artifacts.\n\
         //! Bounds & Faults: O(N) bounded traversal; fails closed with diagnostic vectors.\n\n\
         /// [DOC-TIER: A] Public gate carrier documented with a Tier A 4-Block Contract.\n\
         /// Safety & Traceability: Realises: [REQ-PARITY-CARRIER]\n\
         /// Contract: Wraps immutable verification threshold `limit`.\n\
         /// Bounds & Faults: O(1) stack footprint; infallible value type.\n\
         pub struct SampleGate { pub limit: usize }\n\
         impl SampleGate {\n\
             /// [DOC-TIER: A] Evaluates the configured gate limit via a Tier A 4-Block Contract.\n\
             /// Safety & Traceability: Realises: [REQ-PARITY-INHERENT-PUB]\n\
             /// Contract: Pure read-only accessor returning `self.limit`.\n\
             /// Bounds & Faults: O(1) time and space; infallible.\n\
             pub fn configured_limit(&self) -> usize { self.limit }\n\
             /// Computes internal clamped offset as a Tier B/C private method.\n\
             /// Realises: [REQ-PARITY-PRIV-METHOD]\n\
             fn clamped_offset(&self) -> usize { self.limit.saturating_add(1) }\n\
         }\n\
         impl Default for SampleGate {\n\
             /// Constructs a zero-limit `SampleGate` via Tier B/C trait implementation.\n\
             /// Realises: [REQ-PARITY-DEFAULT-IMPL]\n\
             fn default() -> Self { Self { limit: 0 } }\n\
         }\n",
    );
    s.push_str(
        "/// [DOC-TIER: A] High-integrity parity check documented with full 7-section Tier A Rustdoc.\n\
         /// ## 1. Safety Intent & Regulatory Scope\n/// Conforms to DO-178C Level A.\n\
         /// ## 2. Invariants & Mathematical Properties\n/// Deterministic.\n\
         /// ## 3. Preconditions\n/// repo_root is valid directory.\n\
         /// ## 4. Postconditions\n/// Returns Ok(()) on verification.\n\
         /// ## 5. Algorithmic Complexity Bounds\n/// O(1).\n\
         /// ## 6. Failure Modes & Fault Tolerance\n/// Fails closed.\n\
         /// ## 7. Realization & Traceability\n/// Realises: [REQ-PARITY-CHECK]\n\
         pub fn check_audit() -> Result<(), Vec<String>> { Ok(()) }\n\
         /// Crate-internal helper documented under Tier B/C.\n\
         /// Realises: [REQ-PARITY-CRATE-FN]\n\
         pub(crate) fn crate_step(v: usize) -> usize { v }\n\
         /// Private helper documented under Tier B/C without 7-section boilerplate.\n\
         /// Realises: [REQ-PARITY-PRIV-FN]\n\
         fn local_step(v: usize) -> usize { v.saturating_add(1) }\n",
    );
    s
}

#[test]
fn test_check_34_purity_f_rejects_missing_seven_section_rustdoc(
) -> Result<(), Box<dyn std::error::Error>> {
    let clean_tiered = clean_tiered_rustdoc_fixture();
    let res_clean = audit_source_purity(&clean_tiered, "sample/src/checks/parity_gates.rs", true);
    assert!(
        res_clean.is_ok(),
        "Clean Tier A (7-section & 4-Block) and Tier B/C items must pass Check 34: {:?}",
        res_clean.err()
    );

    let partial_tier_a = r#"/// [DOC-TIER: A] Partial doc missing section 7 and Realises tag.
/// ## 1. Safety Intent & Regulatory Scope
/// Conforms to DO-178C Level A.
/// ## 2. Invariants & Mathematical Properties
/// Deterministic.
/// ## 3. Preconditions
/// repo_root is valid directory.
/// ## 4. Postconditions
/// Returns Ok(()) on verification.
/// ## 5. Algorithmic Complexity Bounds
/// O(1).
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed.
pub fn check_audit() -> Result<(), Vec<String>> {
    Ok(())
}
"#;
    let res_partial_a =
        audit_source_purity(partial_tier_a, "sample/src/checks/parity_gates.rs", true);
    assert!(
        res_partial_a.is_err(),
        "Tier A pub fn missing section 7 must fail"
    );
    let errs_partial_a = res_partial_a.err().ok_or("expected Err")?;
    assert!(
        errs_partial_a.iter().any(|e| e.contains(
            "Tier A fn 'check_audit' missing valid 7-section Rustdoc or 4-Block Contract"
        )),
        "{:?}",
        errs_partial_a
    );
    Ok(())
}

#[test]
fn test_check_34_purity_f_rejects_downgraded_and_undocumented_items(
) -> Result<(), Box<dyn std::error::Error>> {
    let downgraded_pub = r#"/// [DOC-TIER: A] Concise two-line docstring is invalid on explicit Tier A pub fn.
/// Realises: [REQ-DOWNGRADED-PUB]
pub fn downgraded_api() {}
"#;
    assert!(
        audit_source_purity(downgraded_pub, "sample/src/checks/parity_gates.rs", true).is_err(),
        "Unrestricted pub fn downgraded to Tier B/C summary must fail closed"
    );

    let downgraded_mod = r#"//! Two-line module docstring lacking Tier A 7-section or 4-Block Contract.
//! Realises: [REQ-DOWNGRADED-MOD]

/// Private helper with valid Tier B/C docstring.
/// Realises: [REQ-VALID-HELPER]
fn valid_helper() {}
"#;
    assert!(
        audit_source_purity(downgraded_mod, "sample/src/checks/parity_gates.rs", true).is_err(),
        "Tier A //! module header downgraded to 2-line Tier B/C format must fail closed"
    );

    for bad_src in [
        "fn undocumented_helper() {}\n",
        "/// Private helper missing traceability tag.\nfn untraced_helper() {}\n",
        "pub(crate) fn undocumented_crate_helper() {}\n",
    ] {
        assert!(
            audit_source_purity(bad_src, "sample/src/checks/parity_gates.rs", true).is_err(),
            "Undocumented or untraced item must fail closed: {bad_src}"
        );
    }

    for marker in ["TODO", "TBD", "FIXME", "XXX"] {
        let placeholder_priv = format!(
            "/// Helper summary with {marker} placeholder.\n/// Realises: [REQ-PLACEHOLDER]\nfn helper_with_marker() {{}}\n"
        );
        assert!(
            audit_source_purity(&placeholder_priv, "sample/src/checks/parity_gates.rs", true)
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn test_check_34_hardening_b_rejects_static_lifetime_string_slice_const(
) -> Result<(), Box<dyn std::error::Error>> {
    let clean = "pub const ALL: &'static [Self] = &[Self::VariantA, Self::VariantB];";
    let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
    assert!(
        res_clean.is_ok(),
        "Clean typed enum array must pass: {:?}",
        res_clean.err()
    );

    let ty = join_pair("&'static [", "&'static str]");
    let mutant = format!("pub const BAD: {} = &[\"a\"];", ty);
    let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
    assert!(
        res_mutant.is_err(),
        "Mutant with 'static string-slice const must fail"
    );
    let errs = res_mutant.err().ok_or("expected Err")?;
    assert!(
        errs.iter()
            .any(|e| e.contains("static string-slice array detected")),
        "{:?}",
        errs
    );

    let arr_ty = join_pair("[&'static ", "str; 2]");
    let mutant_arr = format!("pub const BAD: {} = [\"a\", \"b\"];", arr_ty);
    let res_arr = audit_source_purity(&mutant_arr, "sample/src/lib.rs", false);
    assert!(
        res_arr.is_err(),
        "Mutant with 'static string array must fail"
    );
    Ok(())
}

#[test]
fn test_check_34_hardening_b_rejects_static_str_array_into_iter(
) -> Result<(), Box<dyn std::error::Error>> {
    let clean =
        "fn kinds() -> std::array::IntoIter<Kind, 3> { [Kind::A, Kind::B, Kind::C].into_iter() }";
    let res_clean = audit_source_purity(clean, "sample/src/lib.rs", false);
    assert!(
        res_clean.is_ok(),
        "Clean typed IntoIter must pass: {:?}",
        res_clean.err()
    );

    let iter_ty = join_pair("IntoIter<", "&'static str, 3>");
    let mutant = format!(
        "fn names() -> std::array::{} {{ [\"a\", \"b\", \"c\"].into_iter() }}",
        iter_ty
    );
    let res_mutant = audit_source_purity(&mutant, "sample/src/lib.rs", false);
    assert!(
        res_mutant.is_err(),
        "Mutant with 'static str array iterator must fail"
    );
    let errs = res_mutant.err().ok_or("expected Err")?;
    assert!(
        errs.iter()
            .any(|e| e.contains("static string-slice array detected")),
        "{:?}",
        errs
    );
    Ok(())
}

#[test]
fn test_check_34_hardening_e_rejects_direct_range_slice_indexing(
) -> Result<(), Box<dyn std::error::Error>> {
    let clean = "let sub = s.get(1..3);";
    let res_clean = audit_source_purity(clean, "sample/src/checks/parity_gates.rs", true);
    assert!(
        res_clean.is_ok(),
        "Clean .get(..) slicing must pass: {:?}",
        res_clean.err()
    );

    let mutant = "let sub = &s[1..3];";
    let res_mutant = audit_source_purity(mutant, "sample/src/checks/parity_gates.rs", true);
    assert!(
        res_mutant.is_err(),
        "Mutant with direct panicking range indexing must fail"
    );
    let errs = res_mutant.err().ok_or("expected Err")?;
    assert!(
        errs.iter()
            .any(|e| e.contains("direct panicking range-slice indexing detected")),
        "{:?}",
        errs
    );

    let res_unaudited = audit_source_purity(mutant, "sample/src/lib.rs", false);
    assert!(
        res_unaudited.is_ok(),
        "Rule (e) applies only when check_rustdoc is true: {:?}",
        res_unaudited.err()
    );
    Ok(())
}

#[test]
fn test_check_34_hardening_f_rejects_duplicate_boilerplate_rustdoc(
) -> Result<(), Box<dyn std::error::Error>> {
    let body = "/// [DOC-TIER: A]\n\
                /// ## 1. Safety Intent & Regulatory Scope\n/// Conforms to DO-178C Level A.\n\
                /// ## 2. Invariants & Mathematical Properties\n/// Deterministic.\n\
                /// ## 3. Preconditions\n/// None.\n\
                /// ## 4. Postconditions\n/// Returns Ok(()).\n\
                /// ## 5. Algorithmic Complexity Bounds\n/// O(1).\n\
                /// ## 6. Failure Modes & Fault Tolerance\n/// Fails closed.\n\
                /// ## 7. Realization & Traceability\n";
    let mutant = format!(
        "/// First gate summary.\n{body}/// Realises: [REQ-A].\npub fn gate_a() {{}}\n\n/// Second gate summary.\n{body}/// Realises: [REQ-B].\npub fn gate_b() {{}}\n",
        body = body
    );
    let res_mutant = audit_source_purity(&mutant, "sample/src/checks/parity_gates.rs", true);
    assert!(
        res_mutant.is_err(),
        "Mutant with duplicated 7-section Rustdoc bodies must fail"
    );
    let errs = res_mutant.err().ok_or("expected Err")?;
    assert!(
        errs.iter()
            .any(|e| e.contains("duplicate/boilerplate 7-section")),
        "{:?}",
        errs
    );

    let unique_body = body.replace("Deterministic.", "Deterministic and idempotent.");
    let clean = format!(
        "/// First gate summary.\n{body}/// Realises: [REQ-A].\npub fn gate_a() {{}}\n\n/// Second gate summary.\n{unique}/// Realises: [REQ-B].\npub fn gate_b() {{}}\n",
        body = body,
        unique = unique_body
    );
    let res_clean = audit_source_purity(&clean, "sample/src/checks/parity_gates.rs", true);
    assert!(
        res_clean.is_ok(),
        "Function-specific Rustdoc bodies must pass: {:?}",
        res_clean.err()
    );
    Ok(())
}
