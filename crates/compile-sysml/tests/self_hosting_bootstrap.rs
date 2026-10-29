//! Self-hosting 4-level modular multi-file codegen bootstrap suite (`BP-07` / `REQ-0607`).
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, and ISO 26262 ASIL D compiler verification mandates.
//! ## 2. Invariants & Mathematical Properties
//! Verifies fail-closed DAG/layer gates, 4-level modularity bounds, and Stage-1/Stage-2 SHA-256 identity.
//! ## 3. Preconditions
//! Invoked via Cargo test runner with writable OS temporary storage and workspace root metadata.
//! ## 4. Postconditions
//! All temporary fixtures are reclaimed via RAII `Drop` with zero residual filesystem mutations.
//! ## 5. Algorithmic Complexity Bounds
//! Time Complexity: $O(N + E)$ across bounded test schemas; Space Complexity: $O(N)$ temporary AST trees.
//! ## 6. Failure Modes & Fault Tolerance
//! Propagates typed `Result` errors or assertion failures without `.unwrap()` or `.expect()`.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607] -- Self-hosting modular codegen contract verification suite.
use compile_sysml::{cli::Cli, driver, flutter_codegen::{
    compute_schema_hash, emit_dart_solver_bridge, emit_flutter_contracts, escape_dart_single_quoted, extract_bounds_from_doc,
    format_display_name, generate_domain_seed_dart, generate_schema_models_dart, generate_solver_bridge_dart, generate_ui_contract_test_dart,
    parse_topology_json, split_ident_into_words, to_camel_case, to_pascal_case, FlutterCodegenError, FlutterContractOptions, FlutterContractReport, TopologyNode,
}, modular_codegen::{
    domain::{GeneratedTreeManifest, LayerRole, ModularityBounds, ModularityErrorCode},
    emitters::rust_emitter::compute_directory_tree_manifest, lowering::estimate_rust_fn_cyclomatic_complexity,
    generated::{domain::modularity_bounds_spec::model::ModularityBoundsSpec, lowering::modular_codegen_lifecycle::ModularCodegenLifecycleState},
}};
use std::{any::{type_name as tn, type_name_of_val as tv}, error::Error, fs, path::{Path, PathBuf}, sync::atomic::{AtomicU64, Ordering}};
type TestResult = Result<(), Box<dyn Error>>;
static COUNTER: AtomicU64 = AtomicU64::new(1);
/// RAII temporary directory guard ensuring deterministic fixture cleanup on scope exit.
/// ## 1. Safety Intent & Regulatory Scope
/// Satisfies DO-330 TQL-4 isolated test fixture containment and Pillar 2 resource hygiene (`E0602`).
/// ## 2. Invariants & Mathematical Properties
/// Holds a process-unique directory path under `std::env::temp_dir()` for the lifetime of `Self`.
/// ## 3. Preconditions
/// Constructed exclusively via `ScopedTempDir::new` with an ASCII test identifier prefix.
/// ## 4. Postconditions
/// Directory exists throughout test scope and is recursively unlinked when dropped.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(1)$ handle storage; Space Complexity: $O(1)$ single `PathBuf` ownership.
/// ## 6. Failure Modes & Fault Tolerance
/// Unwinds cleanly on early `?` returns or assertion failures without leaking temporary trees.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- RAII temporary directory fixture isolation.
struct ScopedTempDir { path: PathBuf }
impl ScopedTempDir {
    /// Allocates and creates a unique temporary directory under the host OS temp root.
    /// ## 1. Safety Intent & Regulatory Scope
    /// Prevents concurrent test race conditions across parallel DO-330 TQL-4 test threads.
    /// ## 2. Invariants & Mathematical Properties
    /// Monotonic `AtomicU64` sequence combined with PID guarantees collision-free directory paths.
    /// ## 3. Preconditions
    /// Host operating system provides a writable `std::env::temp_dir()` root directory.
    /// ## 4. Postconditions
    /// Returns an initialized `ScopedTempDir` whose `path` directory exists on disk.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: $O(1)$ atomic increment and `mkdir`; Space Complexity: $O(1)$ path string.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Propagates `std::io::Error` via `Result` if directory creation fails.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607] -- Collision-free temporary test workspace allocation.
    fn new(prefix: &str) -> Result<Self, Box<dyn Error>> {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!("deap_sh_{prefix}_{}_{id}", std::process::id()));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}
impl Drop for ScopedTempDir {
    /// Recursively removes the scoped temporary directory upon normal return or stack unwind.
    /// ## 1. Safety Intent & Regulatory Scope
    /// Enforces DO-330 TQL-4 zero-workspace-pollution and Pillar 2 deterministic teardown.
    /// ## 2. Invariants & Mathematical Properties
    /// Executes at most once per `ScopedTempDir` instance at lexical scope termination.
    /// ## 3. Preconditions
    /// `self.path` references an isolated subdirectory under `std::env::temp_dir()`.
    /// ## 4. Postconditions
    /// Temporary directory tree is removed from the host filesystem without throwing during unwind.
    /// ## 5. Algorithmic Complexity Bounds
    /// Time Complexity: $O(F)$ in temporary fixture file count; Space Complexity: $O(1)$.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Suppresses benign OS teardown errors via `let _ =` to remain unwind-safe inside `Drop`.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607] -- Deterministic RAII temporary directory reclamation.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
/// Iteratively collects all files matching `suffix` beneath `root` without recursion.
/// ## 1. Safety Intent & Regulatory Scope
/// Supports DO-330 TQL-4 artifact tree auditing using bounded iterative traversal.
/// ## 2. Invariants & Mathematical Properties
/// Visits every reachable directory entry in `root` via an explicit heap workstack (zero recursion).
/// ## 3. Preconditions
/// `root` must reference an existing readable directory on disk.
/// ## 4. Postconditions
/// Returns every file `PathBuf` under `root` whose UTF-8 lossy name ends with `suffix`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(F + D)$ over files and directories; Space Complexity: $O(F + D)$ workstack.
/// ## 6. Failure Modes & Fault Tolerance
/// Returns `Err` immediately if `root` or any nested subdirectory cannot be read.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- Non-recursive emitted module discovery helper.
fn collect_files_with_suffix(root: &Path, suffix: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let (mut stack, mut matched) = (vec![root.to_path_buf()], Vec::new());
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.to_string_lossy().ends_with(suffix) {
                matched.push(path);
            }
        }
    }
    Ok(matched)
}
/// Verifies directed dependency cycles fail closed with `E0702` and zero emitted files.
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces DO-178C Level A topological DAG acyclicity prior to any filesystem persistence.
/// ## 2. Invariants & Mathematical Properties
/// Mutual `NodeA <-> NodeB` part references violate strict topological ordering (`E0702`).
/// ## 3. Preconditions
/// Temporary schema contains a two-node cycle within `CyclePkg` and target `out_dir` is absent.
/// ## 4. Postconditions
/// `driver::execute` returns `Err((1, msg))` containing `E0702` and `!out_dir.exists()`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(V + E)$ Kahn cycle detection; Space Complexity: $O(V + E)$ adjacency map.
/// ## 6. Failure Modes & Fault Tolerance
/// Aborts in-memory before creating output directories, preventing partial artifact corruption.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- Negative mutation probe for `E0702` cycle fail-closed gate.
#[test]
fn test_e0702_directed_cycle_fails_closed_with_zero_files_written() -> TestResult {
    let tmp = ScopedTempDir::new("e0702")?;
    let (schema, out_dir) = (tmp.path.join("cycle.sysml"), tmp.path.join("out"));
    fs::write(&schema, "package CyclePkg { part def NodeA { @Layer(\"Domain\"); part b_ref : NodeB; } part def NodeB { @Layer(\"Domain\"); part a_ref : NodeA; } }")?;
    let cli = Cli::try_parse_from(["compile-sysml", "--emit-modular-rust", out_dir.to_str().ok_or("u8")?, "--schema", schema.to_str().ok_or("u8")?])?;
    let (code, msg) = driver::execute(cli).err().ok_or("expected E0702 cycle failure")?;
    assert_eq!(code, 1);
    assert!(msg.contains("E0702") && !out_dir.exists(), "unexpected E0702 result: {msg}");
    Ok(())
}
/// Verifies upward `@Layer` imports (`Domain -> Presentation`) fail closed with `E0703`.
/// ## 1. Safety Intent & Regulatory Scope
/// Enforces DO-178C Level A 3-layer architectural partitioning across generated module boundaries.
/// ## 2. Invariants & Mathematical Properties
/// Every dependency edge `(u -> v)` must satisfy `tier(u) >= tier(v)`; `1 >= 3` is false (`E0703`).
/// ## 3. Preconditions
/// Temporary schema binds `@Layer("Domain")` `CoreEntity` to `@Layer("Presentation")` `PresentationView`.
/// ## 4. Postconditions
/// `driver::execute` returns `Err((1, msg))` containing `E0703` and `!out_dir.exists()`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(E)$ edge tier verification; Space Complexity: $O(V)$ layer lookup table.
/// ## 6. Failure Modes & Fault Tolerance
/// Fails transactionally in memory with zero directories or partial `.rs` files persisted.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- Negative mutation probe for `E0703` upward layer dependency gate.
#[test]
fn test_e0703_upward_layer_import_domain_to_presentation_fails_closed() -> TestResult {
    assert!(LayerRole::Domain != LayerRole::Application && LayerRole::Domain != LayerRole::Presentation);
    let tmp = ScopedTempDir::new("e0703")?;
    let (schema, out_dir) = (tmp.path.join("layer.sysml"), tmp.path.join("out"));
    fs::write(&schema, "package LayerViolationPkg { part def CoreEntity { @Layer(\"Domain\"); part ui_ref : PresentationView; } part def PresentationView { @Layer(\"Presentation\"); attribute label : String = \"view\"; } }")?;
    let cli = Cli::try_parse_from(["compile-sysml", "--emit-modular-rust", out_dir.to_str().ok_or("u8")?, "--schema", schema.to_str().ok_or("u8")?])?;
    let (code, msg) = driver::execute(cli).err().ok_or("expected E0703 upward layer failure")?;
    assert_eq!(code, 1);
    assert!(msg.contains("E0703") && !out_dir.exists(), "unexpected E0703 result: {msg}");
    Ok(())
}
/// Verifies `GeneratedTreeManifest::verify_fixed_point` rejects tampered SHA-256 digests with `E0705`.
/// ## 1. Safety Intent & Regulatory Scope
/// Guards DO-330 TQL-4 metacircular bootstrap determinism against silent tree hash divergence.
/// ## 2. Invariants & Mathematical Properties
/// `verify_fixed_point` returns `Ok(())` iff root `tree_sha256` and per-file SHA-256 entries match.
/// ## 3. Preconditions
/// Two identical compilations produce `dir_a` and `dir_b`, after which `manifest_b.tree_sha256` is mutated.
/// ## 4. Postconditions
/// `manifest_a.verify_fixed_point(&manifest_b)` yields `FixedPointDriftE0705` (`"E0705"`).
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(B)$ total emitted bytes hashed; Space Complexity: $O(F)$ manifest entries.
/// ## 6. Failure Modes & Fault Tolerance
/// Deterministically surfaces cryptographic drift rather than accepting stale Stage-1 artifacts.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- Negative mutation probe for `E0705` fixed-point drift gate.
#[test]
fn test_e0705_fixed_point_drift_detection_fails_closed() -> TestResult {
    let tmp = ScopedTempDir::new("e0705")?;
    let (schema, dir_a, dir_b) = (tmp.path.join("v.sysml"), tmp.path.join("a"), tmp.path.join("b"));
    fs::write(&schema, "package DriftPkg { part def UnitNode { @Layer(\"Domain\"); attribute v : Integer = 1; } }")?;
    for target in [&dir_a, &dir_b] {
        let cli = Cli::try_parse_from(["compile-sysml", "--emit-modular-rust", target.to_str().ok_or("u8")?, "--schema", schema.to_str().ok_or("u8")?])?;
        assert_eq!(driver::execute(cli).map_err(|(_, e)| e)?, 0);
    }
    let manifest_a: GeneratedTreeManifest = compute_directory_tree_manifest(&dir_a)?;
    let mut manifest_b = compute_directory_tree_manifest(&dir_b)?;
    manifest_b.tree_sha256 = "0000".to_string();
    let diag = manifest_a.verify_fixed_point(&manifest_b).err().ok_or("expected E0705 drift")?;
    assert!(diag.code == ModularityErrorCode::FixedPointDriftE0705 && diag.code.code() == "E0705");
    Ok(())
}
/// Verifies `--emit-modular-rust --dry-run` validates and lowers in memory while writing zero files.
/// ## 1. Safety Intent & Regulatory Scope
/// Preserves `ARCH-04` read-only preflight verification semantics for DO-330 TQL-4 CI pipelines.
/// ## 2. Invariants & Mathematical Properties
/// `--dry-run` executes full AST validation and 4-level lowering while performing zero disk writes.
/// ## 3. Preconditions
/// Valid `.sysml` schema exists on disk and target `out_dir` does not yet exist.
/// ## 4. Postconditions
/// `driver::execute` returns `Ok(0)` and `!out_dir.exists()` holds after execution.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(N)$ in-memory synthesis; Space Complexity: $O(N)$ transient compilation units.
/// ## 6. Failure Modes & Fault Tolerance
/// Any validation or bounds error still fails closed with non-zero exit code without touching disk.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- Dry-run zero-side-effect invariant for `--emit-modular-rust`.
#[test]
fn test_emit_modular_rust_dry_run_writes_zero_files() -> TestResult {
    let tmp = ScopedTempDir::new("dry_run")?;
    let (schema, out_dir) = (tmp.path.join("dry.sysml"), tmp.path.join("out"));
    fs::write(&schema, "package DryPkg { part def DryNode { @Layer(\"Domain\"); attribute ok : Boolean = true; } }")?;
    let cli = Cli::try_parse_from(["compile-sysml", "--emit-modular-rust", out_dir.to_str().ok_or("u8")?, "--schema", schema.to_str().ok_or("u8")?, "--dry-run"])?;
    assert_eq!(driver::execute(cli).map_err(|(_, e)| e)?, 0);
    assert!(!out_dir.exists(), "--dry-run must write zero files");
    Ok(())
}
/// Verifies 4-level deterministic lowering compiles arbitrary-scale schemas within all modularity bounds.
/// ## 1. Safety Intent & Regulatory Scope
/// Proves DO-178C Level A zero-god-file and bounded cyclomatic complexity invariants across all 4 levels.
/// ## 2. Invariants & Mathematical Properties
/// Every emitted file satisfies `LOC <= 400`, `items <= 12`, `v(G) <= 8`, and Check 36 7-section docstrings.
/// ## 3. Preconditions
/// Nested 3-layer schema contains a multi-facet `PartDef`, 40-step action, 30-transition FSM, and 25-clause guard.
/// ## 4. Postconditions
/// Compilation succeeds (`Ok(0)`), emits `>= 10` files and nested `mod.rs` barrels, and obeys `ModularityBounds`.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(P + S + T + C)$ linear in AST nodes; Space Complexity: $O(F)$ bounded output modules.
/// ## 6. Failure Modes & Fault Tolerance
/// Fails test assertions immediately if any generated unit breaches line, item, complexity, or docstring gates.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- Four-level lowering scale and zero-dead-end verification.
#[test]
fn test_four_level_lowering_arbitrary_scale_schema_obeys_all_bounds() -> TestResult {
    let tmp = ScopedTempDir::new("scale")?;
    let (schema, out_dir) = (tmp.path.join("scale.sysml"), tmp.path.join("out"));
    let attrs: String = (1..=25).map(|k| format!("attribute x_{k:02} : Integer = 1; ")).collect();
    let params = (1..=25).map(|k| format!("in x_{k:02} : Integer")).collect::<Vec<_>>().join(", ");
    let prefix: String = (1..=12).map(|k| format!("x_{:02} >= 0 and x_{:02} >= 0 or ", 2 * k - 1, 2 * k)).collect();
    let steps: String = (1..=40).map(|k| format!("step s_{k:02}; ")).collect();
    let states: String = (1..=14).map(|k| format!("state S{k:02}; ")).collect();
    let act_tx: String = (1..=12).map(|k| format!("transition T_a_{k:02} first Active accept ev_a_{k:02} then S{k:02}; ")).collect();
    let ret_tx: String = (1..=10).map(|k| format!("transition T_r_{k:02} first S{k:02} accept ev_r_{k:02} then Active; ")).collect();
    let sysml = format!(
        "package ScaleRoot {{ package core_domain {{ package sub_domain {{ part def SubLeafSpec {{ @Layer(\"Domain\"); attribute leaf_id : Integer = 1; }} }} part def BaseMetricSpec {{ @Layer(\"Domain\"); attribute limit_val : Integer = 10; part leaf_ref : SubLeafSpec; }} calc def ScaleMetricCalc(in raw_val : Integer) : Integer {{ raw_val + 1; }} }} \
         package app_lowering {{ port def ScaleInPort {{ attribute protocol_family : String = \"ScaleIn\"; flow in req_stream : String; }} port def ScaleOutPort {{ attribute protocol_family : String = \"ScaleOut\"; flow out resp_stream : String; }} \
         part def MultiFacetController {{ @Layer(\"Application\"); attribute enabled : Boolean = true; attribute calc_ref : String = \"ScaleMetricCalc\"; attribute action_ref : String = \"FortyStepPipeline\"; attribute state_ref : String = \"ThirtyTransitionFsm\"; {attrs} part base_ref : BaseMetricSpec; port in ingress : ScaleInPort; port out egress : ScaleOutPort; constraint def TwentyFiveClauseGuard({params}) {{ {prefix}x_25 >= 0; }} }} \
         action def FortyStepPipeline {{ in input_count : Integer; out output_count : Integer; {steps} }} \
         state def ThirtyTransitionFsm {{ state Idle; state Active; {states} transition T_init first Initial then Idle; transition T_start first Idle accept ev_start then Active; {act_tx}{ret_tx} \
         transition T_c11 first S11 accept ev_c11 then S13; transition T_c12 first S12 accept ev_c12 then S14; transition T_r13 first S13 accept ev_r13 then Idle; transition T_r14 first S14 accept ev_r14 then Idle; transition T_h01 first S01 accept ev_h01 then S02; transition T_h02 first S02 accept ev_h02 then S03; }} }} \
         package ui_emitters {{ port def ScaleViewPort {{ attribute protocol_family : String = \"ScaleView\"; flow out frame_stream : String; }} part def ScalePresenter {{ @Layer(\"Presentation\"); attribute ready : Boolean = true; part ctrl_ref : MultiFacetController; port out view_out : ScaleViewPort; }} }} }}"
    );
    fs::write(&schema, sysml)?;
    let cli = Cli::try_parse_from(["compile-sysml", "--emit-modular-rust", out_dir.to_str().ok_or("u8")?, "--schema", schema.to_str().ok_or("u8")?])?;
    assert_eq!(driver::execute(cli).map_err(|(_, e)| e)?, 0);
    for rel in ["core_domain/mod.rs", "core_domain/sub_domain/mod.rs", "app_lowering/mod.rs", "ui_emitters/mod.rs"] {
        assert!(out_dir.join(rel).is_file(), "missing nested subpackage barrel: {rel}");
    }
    let bounds = ModularityBounds::canonical();
    assert!((2..=bounds.max_cyclomatic_complexity).contains(&estimate_rust_fn_cyclomatic_complexity("fn p(x: i64) -> i64 { if x > 0 { x } else { 0 } }")));
    let manifest = compute_directory_tree_manifest(&out_dir)?;
    assert!(manifest.files.len() >= 10, "expected >= 10 files, got {}", manifest.files.len());
    for file in &manifest.files {
        assert!(file.line_count <= bounds.max_file_lines && file.item_count <= bounds.max_items_per_file, "size bound: {}", file.relative_path);
        assert!(file.max_cyclomatic_complexity <= bounds.max_cyclomatic_complexity, "v(G) bound: {}", file.relative_path);
        let content = fs::read_to_string(out_dir.join(&file.relative_path))?;
        assert!(content.contains("## 1. Safety Intent & Regulatory Scope") && content.contains("## 7. Realization & Traceability"));
    }
    Ok(())
}
/// Verifies `--emit-flutter-contracts` emits decomposed per-package `.g.dart` units plus top-level barrels.
/// ## 1. Safety Intent & Regulatory Scope
/// Ensures downstream Flutter codegen eliminates single-file Dart monoliths while preserving barrel imports.
/// ## 2. Invariants & Mathematical Properties
/// All 4 legacy barrel `.g.dart` entrypoints coexist with modular `.g.dart` files across 5 subdirectories.
/// ## 3. Preconditions
/// Multi-package `ModularApp` `SysML` schema declares `domain::AlphaNode` and `presentation::BetaPanel`.
/// ## 4. Postconditions
/// `driver::execute` returns `Ok(0)` and populates all 4 barrels and all 5 decomposed `.g.dart` trees.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(P + A)$ linear in packages and classifiers; Space Complexity: $O(F)$ Dart units.
/// ## 6. Failure Modes & Fault Tolerance
/// Fails assertions if any top-level barrel file or modular `.g.dart` directory is missing or empty.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- Modular multi-file Flutter contract emission and barrel compatibility.
#[test]
fn test_modular_flutter_emission_decomposes_packages_and_barrels() -> TestResult {
    let tmp = ScopedTempDir::new("flutter_mod")?;
    let (schema, app_dir) = (tmp.path.join("app.sysml"), tmp.path.join("app_flutter"));
    fs::write(&schema, "package ModularApp { package domain { part def AlphaNode { attribute count : Integer = 1; } } package presentation { port def AlphaSignalPort { flow out payload : String; } part def BetaPanel { attribute title : String = \"ok\"; port out sig : AlphaSignalPort; } } }")?;
    let cli = Cli::try_parse_from(["compile-sysml", "--emit-flutter-contracts", app_dir.to_str().ok_or("u8")?, "--schema", schema.to_str().ok_or("u8")?])?;
    assert_eq!(driver::execute(cli).map_err(|(_, e)| e)?, 0);
    for b in ["lib/src/models/generated_schema_models.g.dart", "lib/src/data/solver_bridge.g.dart", "lib/src/data/seeds/generated_domain_seed.g.dart", "test/schema_ui_contract_test.g.dart"] {
        assert!(app_dir.join(b).is_file(), "missing top-level Dart barrel: {b}");
    }
    for d in ["lib/src/models/generated", "lib/src/data/generated/ffi", "lib/src/data/generated/dto", "lib/src/data/seeds/generated", "test/generated/support"] {
        assert!(!collect_files_with_suffix(&app_dir.join(d), ".g.dart")?.is_empty(), "empty modular Dart dir: {d}");
    }
    Ok(())
}
/// Verifies `flutter_codegen.rs` is `< 100` lines, exports all 18 symbols, and `modular_codegen/` obeys line bounds.
/// ## 1. Safety Intent & Regulatory Scope
/// Prevents regression back to the 2,489-line `flutter_codegen.rs` monolith while locking 100% public API parity.
/// ## 2. Invariants & Mathematical Properties
/// `lines(flutter_codegen.rs) < 100`, all 18 public symbols resolve, and `lines(m) <= 400` for every module `m`.
/// ## 3. Preconditions
/// Workspace root contains `crates/compile-sysml/src/flutter_codegen.rs` and `src/modular_codegen/`.
/// ## 4. Postconditions
/// Confirms facade line count, type/function resolution, naming helper outputs, and per-file `<= 400` LOC bounds.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(M \cdot L)$ scanning `modular_codegen` source lines; Space Complexity: $O(M)$ paths.
/// ## 6. Failure Modes & Fault Tolerance
/// Fails compilation if any of the 18 symbols is removed and fails runtime assertions if any file exceeds bounds.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- Monolith elimination and 18-symbol `flutter_codegen` facade contract.
#[test]
fn test_flutter_codegen_facade_under_100_lines_and_all_18_symbols_preserved() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(|p| p.parent()).ok_or("missing root")?;
    let facade = fs::read_to_string(root.join("crates/compile-sysml/src/flutter_codegen.rs"))?;
    assert!(facade.lines().count() < 100, "flutter_codegen.rs facade must be < 100 lines");
    assert!(to_pascal_case("alpha_node") == "AlphaNode" && to_camel_case("AlphaNode") == "alphaNode");
    assert!(escape_dart_single_quoted("a'b").contains("\\'"));
    assert!(!split_ident_into_words("AlphaNode").is_empty() && !format_display_name("alpha_node").is_empty());
    let syms = [
        tn::<FlutterCodegenError>(), tn::<FlutterContractOptions>(), tn::<FlutterContractReport>(), tn::<TopologyNode>(),
        tv(&compute_schema_hash), tv(&extract_bounds_from_doc), tv(&parse_topology_json), tv(&generate_domain_seed_dart),
        tv(&generate_schema_models_dart), tv(&generate_solver_bridge_dart), tv(&emit_dart_solver_bridge), tv(&generate_ui_contract_test_dart), tv(&emit_flutter_contracts),
    ];
    assert!(syms.iter().all(|name| !name.is_empty()));
    let rs_files = collect_files_with_suffix(&root.join("crates/compile-sysml/src/modular_codegen"), ".rs")?;
    assert!(!rs_files.is_empty(), "modular_codegen must contain decomposed .rs modules");
    let max_lines = ModularityBounds::canonical().max_file_lines;
    for path in &rs_files {
        assert!(fs::read_to_string(path)?.lines().count() <= max_lines, "{} exceeded {max_lines}", path.display());
    }
    Ok(())
}
/// Verifies metacircular self-hosting fixed-point identity (`SHA-256(Stage 1) == SHA-256(Stage 2)`).
/// ## 1. Safety Intent & Regulatory Scope
/// Proves DO-330 TQL-4 Stage-1 -> Stage-2 deterministic bootstrap reproducibility on `modular_codegen.sysml`.
/// ## 2. Invariants & Mathematical Properties
/// Recompiling `.pipeline/upstream/schema/modular_codegen.sysml` yields bit-identical tree SHA-256 digests.
/// ## 3. Preconditions
/// Stage-1 tree resides at `crates/compile-sysml/src/modular_codegen/generated` and schema SSOT exists.
/// ## 4. Postconditions
/// `stage1.verify_fixed_point(&stage2)` is `Ok(())`, root hashes match, and Stage-1 Rust types execute cleanly.
/// ## 5. Algorithmic Complexity Bounds
/// Time Complexity: $O(N + B)$ compilation plus SHA-256 tree hashing; Space Complexity: $O(F)$ manifest nodes.
/// ## 6. Failure Modes & Fault Tolerance
/// Fails closed if Stage-1 is empty, out of sync with `modular_codegen.sysml`, or non-deterministic.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607] -- Metacircular Stage-1 == Stage-2 SHA-256 fixed-point verification.
#[test]
fn test_metacircular_self_hosting_stage1_equals_stage2_sha256() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(|p| p.parent()).ok_or("missing root")?;
    let (schema, stage1_dir) = (root.join(".pipeline/upstream/schema/modular_codegen.sysml"), root.join("crates/compile-sysml/src/modular_codegen/generated"));
    let tmp = ScopedTempDir::new("stage2")?;
    let stage2_dir = tmp.path.join("generated_stage2");
    let cli = Cli::try_parse_from(["compile-sysml", "--emit-modular-rust", stage2_dir.to_str().ok_or("u8")?, "--schema", schema.to_str().ok_or("u8")?])?;
    assert_eq!(driver::execute(cli).map_err(|(_, e)| e)?, 0);
    let (stage1, stage2) = (compute_directory_tree_manifest(&stage1_dir)?, compute_directory_tree_manifest(&stage2_dir)?);
    assert!(!stage1.files.is_empty() && stage1.verify_fixed_point(&stage2).is_ok());
    assert_eq!(stage1.tree_sha256, stage2.tree_sha256);
    let idle = ModularCodegenLifecycleState::Idle;
    assert_eq!(compile_sysml::modular_codegen::self_hosted_modularity_bounds(), ModularityBounds::canonical());
    assert!(ModularityBoundsSpec::default().max_file_lines == 400 && idle == ModularCodegenLifecycleState::Idle);
    Ok(())
}
