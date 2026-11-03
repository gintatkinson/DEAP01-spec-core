## 1. Context and References

<!-- test-target: crates/compile-sysml/tests/formal_verification_gatekeeper.rs -->

- **File**: `crates/compile-sysml/src/formal_harnesses.rs:43-144`
- **Pillar**: Test Integrity
- **Symptom**: `synthesize_kani_proof_module` emits a tautological CBMC harness (`kani::assume(sample >= min && sample <= max); assert!(sample >= min && sample <= max);`) that never invokes any target function or evaluates SysML/STPA postconditions, `synthesize_ring2_frb_ffi_bridge` emits an unguarded dummy `unsafe { core::ptr::read(&input) }` identity function instead of `ICD_01`/`ICD_02` fail-closed boundary guards, and `lints/deap-dylint-mir/src/lib.rs` implements naive substring `.contains()` matching instead of `rustc_middle::mir::Body` call-graph verification.
- **Test-Target**: `crates/compile-sysml/tests/formal_verification_gatekeeper.rs`

## 2. Root Cause Analysis (5 Whys)

1. **Why do Stage 2, Stage 3, and Stage 4 formal verification gates pass without proving real-time zero-allocation, FFI boundary safety, or CBMC postcondition validity?** Because `formal_harnesses.rs`, `lints/deap-dylint-mir/src/lib.rs`, and `formal_stage3_4_miri_kani.rs` were implemented as superficial syntactic stubs rather than the compiler-backed verifiers specified in `BP-09` (`REQ-0609`).
2. **Why does `synthesize_kani_proof_module` fail to falsify arithmetic overflows, panics, or STPA unsafe control actions (`UCA-*`)?** Because lines `89-93` generate a standalone function that binds `let sample: i64 = kani::any();`, assumes `sample >= min_b && sample <= max_b`, and immediately asserts the identical predicate `assert!(sample >= min_b && sample <= max_b)` without calling any target function or emitting `#[kani::unwind(N)]`.
3. **Why does `synthesize_ring2_frb_ffi_bridge` fail to enforce Stage 3 `flutter_rust_bridge` and Miri Tree Borrows safety contracts (`SC-03`)?** Because lines `123-137` ignore SysML `port def` / `item flow` range bounds, IEEE-754 finiteness checks, and FSM state guards (`FrbIcdPortGuardSpec`), emitting a trivial `pub fn #fn_ident(input: i64) -> i64 { unsafe { core::ptr::read(&input) } }` that splices `// SAFETY:` via string replacement solely to appease the token scanner in `formal_stage3_4_miri_kani.rs:145-173`.
4. **Why does `lints/deap-dylint-mir/src/lib.rs` fail to detect transitive heap allocations or dangling `#[realises("REQ-*")]` requirement bindings in Stage 2 (`SC-02`)?** Because `evaluate_mir_lint_rules` (`lints/deap-dylint-mir/src/lib.rs:139-181`) never links `rustc_lint` or traverses `rustc_middle::mir::Body` basic-block `TerminatorKind::Call` / `TerminatorKind::Drop` edges, relying instead on flat line substring checks (`line.contains("Vec::new")`, `line.contains("#[realises(\"\")]")`).
5. **Why did unit tests in `formal_harnesses.rs:151-192` and `formal_stage3_4_miri_kani.rs:210-231` report green status despite these stubs?** Because the tests only asserted substring presence (`out_a.contains("#[kani::proof]")`, `code.contains("// SAFETY: [SC-1102]")`, and `assert!(1 == 1)`) instead of verifying `BP-09` behavioral contracts (`SC-01` through `SC-05`, `AC-01` through `AC-07`, and the 15-module `< 380`-SLOC architecture in Section 6.1).

## 3. Correctness Analysis

Cross-checking the implementation against `docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md` (`BP-09` / `REQ-0609`) exposes four interlocking correctness and test-integrity violations across the formal verification pipeline:

1. **Stage 4 Tautological Kani Proof Synthesis (`crates/compile-sysml/src/formal_harnesses.rs:43-110` vs. `BP-09` §4 `SC-04` & §6 `KaniProofHarnessSpec`)**:
   - Normative contract `SC-04` (`docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md:256-264, 628-685`) mandates that each synthesized `#[kani::proof]` harness accept a `KaniProofHarnessSpec` containing `target_fn_path`, `unwind_bound` ($N \in [1, 64]$, `C-VER-06`), `symbolic_bounds`, `postcondition_exprs`, and `stpa_safety_constraint_ids`, emitting `#[kani::unwind(N)]` and invoking `target_fn_path` on symbolic inputs to prove $\text{Pre}_{\text{sysml}}(x) \implies \text{Post}_{\text{sysml}}(x, f(x)) \land \bigwedge \neg \text{UCA}_j(x, f(x))$.
   - In `crates/compile-sysml/src/formal_harnesses.rs:43-54`, `KaniProofSpec` omits `target_fn_path`, `unwind_bound`, and `postcondition_exprs`. Lines `85-94` emit:
     `let sample: i64 = kani::any(); kani::assume(sample >= #min_b && sample <= #max_b); assert!(sample >= #min_b && sample <= #max_b);`
   - Because $P(x) \implies P(x)$ is vacuously true for all $x \in \mathbb{Z}_{64}$ regardless of target code behavior, CBMC returns `UNSAT` (0 counterexamples) even when the actual domain function panics, overflows, or violates STPA safety constraints.
   - Compounding this, `verify_kani_harness_cst` in `crates/verify-baseline/src/checks/formal_stage3_4_miri_kani.rs:180-204` only checks that `#[kani::proof]` appears in the CST, without checking for `#[kani::unwind(...)]`, target function calls, or non-tautological assertions.

2. **Stage 3 Unguarded Dummy Ring-2 FFI Bridge (`crates/compile-sysml/src/formal_harnesses.rs:116-144` vs. `BP-09` §4 `SC-03` & §6 `FrbIcdPortGuardSpec`)**:
   - Normative contract `SC-03` (`docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md:246-254, 580-626`) requires Ring-2 FFI facades synthesized from `ICD_01` / `ICD_02` `port def` and `item flow` contracts to enforce total guard predicate $G_p(u, s) \iff \text{is\_finite}(u) \land (p_{\min} \le u \le p_{\max}) \land (s \in S_p)$, rejecting `NaN`/`Inf`, out-of-range payloads, and illegal FSM states with `DEAP-E0903` before invoking pure `#![forbid(unsafe_code)]` domain logic.
   - `synthesize_ring2_frb_ffi_bridge` (`crates/compile-sysml/src/formal_harnesses.rs:116-144`) accepts only `(bridge_name: &str, sc_id: &str)` and emits `pub fn #fn_ident(input: i64) -> i64 { unsafe { core::ptr::read(&input) } }`. It performs an unnecessary `core::ptr::read` on a stack value solely to fabricate an `unsafe` block, injects `// SAFETY:` via string `.replacen("unsafe {", ...)` after `prettyplease::unparse` (violating pure AST/CST codegen), and omits all `FrbIcdPortGuardSpec` interval, finiteness, and FSM state checks.

3. **Stage 2 Non-MIR Substring Stub & Missing `cargo-dylint` Discovery (`lints/deap-dylint-mir/src/lib.rs:139-181` & `crates/verify-baseline/src/checks/formal_stage3_4_miri_kani.rs:85-105` vs. `BP-09` §4 `SC-02` & `SC-05`)**:
   - `BP-09` `SC-02` requires `./lints/deap-dylint-mir/` to traverse `rustc_middle::mir::Body` basic-block `TerminatorKind::Call` and `TerminatorKind::Drop` graphs from `#[rt_loop]` entrypoints and validate `#[realises("REQ-*")]` against `.digest.json`.
   - `lints/deap-dylint-mir/src/lib.rs:139-181` contains zero `rustc_middle::mir` or `rustc_lint::LateLintPass` code. Instead:
     - Line `142` sets `let in_rt_loop = normalized.contains("#[rt_loop]");` across the entire file, causing false positives on non-`#[rt_loop]` functions in the same file and false negatives on any indirect helper allocation (`vec![]`, `format!`, `String::from`, `RawVec::grow`, drop glue).
     - Line `159` checks `line.contains("pub enum ") && line.contains("Error") && !normalized.contains("#[must_use]")`, which is bypassed if `#[must_use]` appears anywhere else in the file.
     - Line `169` only checks `line.contains("#[realises(\"\")]")`, ignoring dangling IDs like `#[realises("REQ-9999")]` and emitting `DEAP-E0903` (`line 98`) instead of `DEAP-E0902`.
   - In `crates/verify-baseline/src/checks/formal_stage3_4_miri_kani.rs:85-105`, `check_formal_toolchains_in_dirs` probes `cargo`, `cargo-miri`/`miri`, and `cargo-kani`, completely omitting `cargo-dylint` required by `BP-09` `SC-05` (`ToolchainDriverKind::CargoDylint`).

4. **Missing Modular Submodule Layout (`BP-09` §6.1) & Unremoved Legacy Scanners (`BP-09` §6.2)**:
   - None of the 15 modular files specified in `BP-09` §6.1 (`crates/verify-baseline/src/checks/formal_stages/*`, `crates/compile-sysml/src/formal_codegen/*`, `lints/deap-dylint-mir/src/{rt_alloc,must_use_err,req_trace}.rs`) exist, and the legacy hand-rolled scanners slated for deletion in `BP-09` §6.2 remain in place.

## 4. UML Diagrams

```mermaid
classDiagram
    class KaniProofHarnessSpec {
        +String harness_ident
        +String target_fn_path
        +u32 unwind_bound
        +Vec~KaniSymbolicBound~ symbolic_bounds
        +Vec~String~ postcondition_exprs
        +Vec~String~ stpa_safety_constraint_ids
    }
    class FrbIcdPortGuardSpec {
        +String port_id
        +String signal_name
        +f64 min_value
        +f64 max_value
        +BTreeSet~String~ allowed_fsm_states
    }
    class FormalCodegenSynthesizer {
        +synthesize_kani_proof_module(specs) Result~String~
        +synthesize_ring2_frb_ffi_bridge(spec) Result~String~
        +guard_inbound_frb_f64(spec, raw_value, active_fsm_state) Result~f64~
    }
    class IsolatedDylintMirAllocationGate {
        +evaluate_stage2_dylint_mir(input) Result~Void~
        +check_fn_mir_body(tcx, body, def_id) Vec~DylintMirDiagnostic~
    }

    FormalCodegenSynthesizer --> KaniProofHarnessSpec : "emits #[kani::unwind(N)] + target_fn call + postconditions"
    FormalCodegenSynthesizer --> FrbIcdPortGuardSpec : "enforces is_finite + [min..max] + FSM state guard (DEAP-E0903)"
    IsolatedDylintMirAllocationGate --> FormalCodegenSynthesizer : "verifies Stage 2 MIR call graph and REQ-* digest (DEAP-E0901/E0902)"
```

## 5. Affected Callers / Downstream Impact

- `crates/compile-sysml/src/formal_harnesses.rs` (`synthesize_kani_proof_module`, `synthesize_ring2_frb_ffi_bridge`) -- Any downstream or upstream caller invoking Stage 4 Kani harness synthesis receives a vacuous `assume(P); assert!(P);` tautology that never exercises target functions, and any caller invoking Ring-2 FRB FFI synthesis receives an unguarded identity function lacking `ICD_01`/`ICD_02` domain range, finiteness, and FSM state protection.
- `lints/deap-dylint-mir/src/lib.rs` (`evaluate_mir_lint_rules`) -- Stage 2 verification misses transitive heap allocations in `#[rt_loop]` functions and fails to validate `#[realises("REQ-*")]` attributes against compiled `.sysml` digests.
- `crates/verify-baseline/src/checks/formal_stage3_4_miri_kani.rs` (`check_formal_toolchains_in_dirs`, `verify_kani_harness_cst`) -- Qualification CI (`--require-formal-toolchains`) fails to probe `cargo-dylint` and accepts tautological Kani harnesses that lack `#[kani::unwind(N)]` and target invocations.

## 6. Proposed Correction

<!-- MUST NOT use regex (Check 34: deterministic linear slice/char scanning, no regex crate). MUST be domain-agnostic for upstream crates (Check 19). -->

Replace the tautological Kani harness generator and unguarded FFI stub in `crates/compile-sysml` with the contract-driven `KaniProofHarnessSpec` and `FrbIcdPortGuardSpec` synthesizers defined in `BP-09` §6 (`crates/compile-sysml/src/formal_codegen/`), require `#[kani::unwind(N)]` + target function invocation + postcondition assertions, enforce fail-closed `guard_inbound_frb_f64` checks (`DEAP-E0903`), and implement transitive call-graph and `.digest.json` requirement verification in `lints/deap-dylint-mir` and `crates/verify-baseline`:

```rust
use quote::quote;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KaniSymbolicBound {
    pub param_ident: String,
    pub rust_type: String,
    pub min_bound: i64,
    pub max_bound: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KaniProofHarnessSpec {
    pub harness_ident: String,
    pub target_fn_path: String,
    pub req_id: String,
    pub sc_id: String,
    pub unwind_bound: u32,
    pub symbolic_bounds: Vec<KaniSymbolicBound>,
    pub postcondition_exprs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrbIcdPortGuardSpec {
    pub bridge_ident: String,
    pub target_domain_fn: String,
    pub port_id: String,
    pub signal_name: String,
    pub sc_id: String,
    pub min_value: f64,
    pub max_value: f64,
    pub allowed_fsm_states: BTreeSet<String>,
}

pub fn synthesize_verified_kani_harness(spec: &KaniProofHarnessSpec) -> Result<String, String> {
    if spec.unwind_bound == 0 || spec.unwind_bound > 64 {
        return Err(format!("invalid unwind_bound {} (must be in 1..=64)", spec.unwind_bound));
    }
    if spec.postcondition_exprs.is_empty() {
        return Err(format!("missing postconditions for harness `{}`", spec.harness_ident));
    }
    let harness_fn = syn::parse_str::<syn::Ident>(&spec.harness_ident)
        .map_err(|e| format!("invalid harness_ident: {e}"))?;
    let target_fn = syn::parse_str::<syn::Path>(&spec.target_fn_path)
        .map_err(|e| format!("invalid target_fn_path: {e}"))?;
    let unwind = spec.unwind_bound;

    let mut let_stmts = Vec::with_capacity(spec.symbolic_bounds.len());
    let mut arg_idents = Vec::with_capacity(spec.symbolic_bounds.len());
    for bound in &spec.symbolic_bounds {
        if bound.min_bound > bound.max_bound {
            return Err(format!("inverted bounds for `{}`", bound.param_ident));
        }
        let param = syn::parse_str::<syn::Ident>(&bound.param_ident)
            .map_err(|e| format!("invalid param_ident: {e}"))?;
        let ty = syn::parse_str::<syn::Type>(&bound.rust_type)
            .map_err(|e| format!("invalid rust_type: {e}"))?;
        let min_v = bound.min_bound;
        let max_v = bound.max_bound;
        let_stmts.push(quote! {
            let #param: #ty = kani::any();
            kani::assume(#param >= #min_v && #param <= #max_v);
        });
        arg_idents.push(param);
    }

    let mut post_asserts = Vec::with_capacity(spec.postcondition_exprs.len());
    for expr_src in &spec.postcondition_exprs {
        let post_expr = syn::parse_str::<syn::Expr>(expr_src)
            .map_err(|e| format!("invalid postcondition expr `{expr_src}`: {e}"))?;
        post_asserts.push(quote! {
            assert!(#post_expr);
        });
    }

    let tokens = quote! {
        #![forbid(unsafe_code)]
        #[cfg(kani)]
        #[kani::proof]
        #[kani::unwind(#unwind)]
        pub fn #harness_fn() {
            #(#let_stmts)*
            let result = #target_fn(#(#arg_idents),*);
            #(#post_asserts)*
        }
    };
    let syntax_tree = syn::parse2::<syn::File>(tokens)
        .map_err(|e| format!("syn parse2 failure: {e}"))?;
    Ok(prettyplease::unparse(&syntax_tree))
}
```

## 7. Relationship to Existing Issues

Discovered in audit -- new finding (refs #742).

## Audit Source

Adversarial Test Integrity Audit
SEVERITY: Critical
FILE_LOCATION: crates/compile-sysml/src/formal_harnesses.rs:43-144
