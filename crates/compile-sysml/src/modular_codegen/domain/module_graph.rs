//! Domain representations of partitioned compilation units, dependency DAGs, and fixed-point tree digests.
//! ## 1. Safety Intent & Regulatory Scope
//! Defines the pure domain graph model (`module_graph.rs`) governing decomposed SysML v2 software partitions under DO-178C and ISO 26262 modularity rules.
//! ## 2. Invariants & Mathematical Properties
//! Callers populate deterministic AST facets, architectural `LayerRole` assignments, and ordered `Vec` collections without nondeterministic map containers.
//! ## 3. Preconditions
//! Exposes `UnitKind`, outlined facet plans, `CompilationUnit`, `DependencyEdge`, `ModuleGraph`, and SHA-256 `GeneratedTreeManifest` fixed-point verification.
//! ## 4. Postconditions
//! Structural data carriers are infallible value types, while regeneration divergence surfaces explicitly as `ModularityErrorCode::FixedPointDriftE0705`.
//! ## 5. Algorithmic Complexity Bounds
//! All collections preserve insertion or lexicographic `Vec` ordering, and manifest verification runs in linear `O(N)` time over emitted file digests.
//! ## 6. Failure Modes & Fault Tolerance
//! Verified by unit and mutation tests exercising facet modeling, graph construction, and stage-1 versus stage-2 manifest drift detection across all failure branches.
//! ## 7. Realization & Traceability
//! Realises: [REQ-0607]
//! Anchors the domain-tier compilation unit DAG and cryptographic fixed-point manifest contract for `compile-sysml` modular code generation.

use deap_core::sysml_ast::{CalcDef, PartDef, PortDef};

use super::bounds::{ModularityDiagnostic, ModularityErrorCode};
use super::layer_role::LayerRole;

/// Closed taxonomy of structural compilation unit roles emitted by the SysML v2 modular partitioner.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub enum UnitKind` classifies every generated Rust module into a single auditable responsibility facet to prevent monolithic multi-concern files.
/// ## 2. Invariants & Mathematical Properties
/// `pub enum UnitKind` is a fieldless eleven-variant enumeration supporting total ordering (`Ord`), value copying (`Copy`), and deterministic hashing (`Hash`).
/// ## 3. Preconditions
/// Distinguishes package `BarrelFacade` modules, six `Part*Facet` slices (`Model`, `Invariants`, `Ports`, `Actions`, `States`, `Calcs`), and four `Standalone*` units.
/// ## 4. Postconditions
/// Exhaustive compile-time matching across all eleven `pub enum UnitKind` discriminants prevents unhandled module archetypes during code emission.
/// ## 5. Algorithmic Complexity Bounds
/// `pub enum UnitKind` occupies one stack discriminant byte with `O(1)` equality and ordering comparisons and zero dynamic memory allocation.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by asserting discriminant distinctness, total `Ord` ranking, and exhaustive emitter dispatch coverage over all eleven `UnitKind` variants.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Governs single-responsibility facet decomposition for `CompilationUnit` nodes inside the modular codegen domain graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnitKind {
    BarrelFacade,
    PartModelFacet,
    PartInvariantsFacet,
    PartPortsFacet,
    PartActionsFacet,
    PartStatesFacet,
    PartCalcsFacet,
    StandalonePort,
    StandaloneAction,
    StandaloneState,
    StandaloneCalc,
}

/// Outlined pure boolean invariant helper function extracted to bound per-function cyclomatic complexity.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct HoistedPredicate` isolates an individual SysML constraint check into an independently testable Rust predicate with audited `v(G)`.
/// ## 2. Invariants & Mathematical Properties
/// `fn_name` holds a valid snake_case identifier, `params` pairs parameter names with Rust type tokens, and `cyclomatic_complexity` is positive.
/// ## 3. Preconditions
/// Carries the complete signature (`fn_name`, `params`), lowered boolean `rust_expr`, and statically measured `cyclomatic_complexity` for emission.
/// ## 4. Postconditions
/// `pub struct HoistedPredicate` performs zero fallible parsing, leaving complexity ceiling enforcement against `ModularityBounds` to verification gates.
/// ## 5. Algorithmic Complexity Bounds
/// Memory usage scales linearly `O(|params| + |rust_expr|)` with deterministic sequential parameter ordering stored in an owned `Vec`.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by testing predicate hoisting on compound invariant clauses and asserting exact `fn_name`, typed `params`, and `cyclomatic_complexity` values.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Represents extracted constraint helper predicates attached to `PartInvariantsFacet` compilation units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoistedPredicate {
    pub fn_name: String,
    pub params: Vec<(String, String)>,
    pub rust_expr: String,
    pub cyclomatic_complexity: usize,
}

/// Contiguous chunk of sequential action statements outlined into a bounded helper stage function.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct OutlinedActionStage` caps action body length and branching density by partitioning long behavioral sequences into indexed stages.
/// ## 2. Invariants & Mathematical Properties
/// `stage_index` records the zero-based execution ordinal, `fn_name` identifies the generated helper, and `steps` holds ordered statement lines.
/// ## 3. Preconditions
/// Preserves exact intra-stage statement sequencing so consecutive `OutlinedActionStage` invocations reproduce the source SysML `ActionDef` semantics.
/// ## 4. Postconditions
/// `pub struct OutlinedActionStage` is a passive owned record; step-count violations above `max_action_steps_per_stage` are flagged by bounds validators.
/// ## 5. Algorithmic Complexity Bounds
/// Stage storage is `O(|steps|)` in heap memory with deterministic `PartialEq` and `Eq` comparisons across stage ordinals and statement strings.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by slicing multi-step actions across stage boundaries and confirming monotonic `stage_index` progression and lossless `steps` partitioning.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Encapsulates one bounded pipeline stage within an `OutlinedActionPlan` behavioral decomposition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinedActionStage {
    pub stage_index: usize,
    pub fn_name: String,
    pub steps: Vec<String>,
}

/// Decomposed execution plan lowering a SysML `ActionDef` into typed parameters and bounded stage functions.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct OutlinedActionPlan` prevents monolithic action functions by organizing behavioral execution as a thin orchestrator over `OutlinedActionStage` calls.
/// ## 2. Invariants & Mathematical Properties
/// `action_name` names the source SysML action, `in_params` and `out_params` store `(name, rust_type)` pairs, and `stages` are ordered by `stage_index`.
/// ## 3. Preconditions
/// Supplies emitters with deterministic input bindings, output return contracts, and partitioned `stages` respecting `ModularityBounds` step budgets.
/// ## 4. Postconditions
/// Empty or multi-stage `stages` vectors are represented uniformly without special-case traps or unchecked slice indexing.
/// ## 5. Algorithmic Complexity Bounds
/// Allocates `O(|in_params| + |out_params| + |stages|)` owned vectors whose iteration order is strictly fixed by AST declaration sequence.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by lowering parameterized actions exceeding single-stage limits and checking orchestrator-to-stage wiring and parameter tuple fidelity.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Drives modular Rust emission for `PartActionsFacet` and `StandaloneAction` compilation units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinedActionPlan {
    pub action_name: String,
    pub in_params: Vec<(String, String)>,
    pub out_params: Vec<(String, String)>,
    pub stages: Vec<OutlinedActionStage>,
}

/// Per-source-state transition dispatch helper extracted from a SysML state machine definition.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct OutlinedStateHandler` bounds state-machine dispatch `v(G)` by isolating outgoing `(trigger, target_state)` arms per `source_state` chunk.
/// ## 2. Invariants & Mathematical Properties
/// Each entry in `transitions` pairs a trigger token with a destination state identifier, and `cyclomatic_complexity` reflects the handler branch count.
/// ## 3. Preconditions
/// Binds `source_state` and generated `fn_name` to an ordered `transitions` table so top-level state dispatch delegates cleanly without deep nesting.
/// ## 4. Postconditions
/// Unmatched triggers fall through safely in generated handlers; `OutlinedStateHandler` itself holds pure owned strings and an unsigned complexity scalar.
/// ## 5. Algorithmic Complexity Bounds
/// Occupies `O(|transitions|)` memory and preserves deterministic first-declared transition priority across identical `source_state` evaluations.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by partitioning high-fanout state nodes and asserting transition tuple ordering and `cyclomatic_complexity <= max_cyclomatic_complexity`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Models one bounded transition-handler function inside an `OutlinedStateMachinePlan`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinedStateHandler {
    pub source_state: String,
    pub fn_name: String,
    pub transitions: Vec<(String, String)>,
    pub cyclomatic_complexity: usize,
}

/// Partitioned lowering blueprint for a SysML `StateDef` automaton and its outlined state handlers.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct OutlinedStateMachinePlan` structures generated state machines into explicit state enumerations, deterministic initialization, and bounded handlers.
/// ## 2. Invariants & Mathematical Properties
/// `state_def_name` names the automaton, `states` lists unique state identifiers, `initial_state` belongs to `states`, and `handlers` cover outgoing transitions.
/// ## 3. Preconditions
/// Provides complete metadata to emit the state enum, default initial state constructor, and delegated `OutlinedStateHandler` transition functions.
/// ## 4. Postconditions
/// Degenerate single-state automata with zero `handlers` remain valid total structures requiring zero exceptional control paths.
/// ## 5. Algorithmic Complexity Bounds
/// Retains `O(|states| + |handlers|)` owned storage in deterministic declaration order with zero hash-randomized traversal.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by lowering multi-state SysML state definitions and asserting `initial_state` retention, state enum coverage, and handler delegation parity.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Supplies state-machine emission plans for `PartStatesFacet` and `StandaloneState` compilation units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlinedStateMachinePlan {
    pub state_def_name: String,
    pub states: Vec<String>,
    pub initial_state: String,
    pub handlers: Vec<OutlinedStateHandler>,
}

/// Self-contained specification of a single generated Rust source file within the partitioned module tree.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct CompilationUnit` binds one relative output path to one `LayerRole` and one `UnitKind` so architectural and size gates audit every file uniformly.
/// ## 2. Invariants & Mathematical Properties
/// `unit_id` and `relative_path` uniquely identify the unit, `package_path` records SysML ancestry, and payload vectors match the assigned `UnitKind`.
/// ## 3. Preconditions
/// Aggregates submodule declarations (`child_modules`, `reexport_children`), AST definitions (`part_def`, `port_defs`, `calc_defs`), and outlined behavioral plans.
/// ## 4. Postconditions
/// Optional and vector payloads default cleanly to `None` or empty `Vec` values when a facet kind carries no items of a given AST category.
/// ## 5. Algorithmic Complexity Bounds
/// Owns bounded AST and plan slices proportional to the single partitioned facet, enabling independent linear-time source emission per unit.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by constructing barrel, part-facet, and standalone units and confirming path mapping, `LayerRole` assignment, and AST payload isolation.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Serves as the vertex payload of `ModuleGraph` across partitioning, DAG validation, and Rust file emission.
#[derive(Debug, Clone, PartialEq)]
pub struct CompilationUnit {
    pub unit_id: String,
    pub relative_path: String,
    pub package_path: Vec<String>,
    pub layer: LayerRole,
    pub kind: UnitKind,
    pub child_modules: Vec<String>,
    pub reexport_children: bool,
    pub part_def: Option<PartDef>,
    pub port_defs: Vec<PortDef>,
    pub calc_defs: Vec<CalcDef>,
    pub hoisted_predicates: Vec<HoistedPredicate>,
    pub action_plans: Vec<OutlinedActionPlan>,
    pub state_plans: Vec<OutlinedStateMachinePlan>,
}

/// Directed architectural dependency relation between two `CompilationUnit` vertices in a `ModuleGraph`.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct DependencyEdge` makes inter-module coupling explicit and auditable against `E0702` cycle freedom and `E0703` downward-only layering rules.
/// ## 2. Invariants & Mathematical Properties
/// `from_unit` and `to_unit` reference `CompilationUnit::unit_id` keys, paired with their resolved `from_layer`, `to_layer`, and traceable `reason`.
/// ## 3. Preconditions
/// Captures the exact directed edge `from_unit -> to_unit` so layering checks test `from_layer.permits_dependency_on(to_layer)` directly on each edge.
/// ## 4. Postconditions
/// Illegal upward edges or back-edges do not corrupt memory; graph validators convert offending `DependencyEdge` records into `ModularityDiagnostic` errors.
/// ## 5. Algorithmic Complexity Bounds
/// Holds three owned `String` identifiers and two 1-byte `LayerRole` values with deterministic lexicographic and structural equality semantics.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by injecting valid downward edges, forbidden upward layer edges, and cycle edges into `ModuleGraph` validation test suites.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Encodes directed inter-unit coupling edges inside `ModuleGraph` for topological sorting and layer enforcement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyEdge {
    pub from_unit: String,
    pub to_unit: String,
    pub from_layer: LayerRole,
    pub to_layer: LayerRole,
    pub reason: String,
}

/// Validated directed acyclic graph of partitioned `CompilationUnit` nodes and their topological emission order.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct ModuleGraph` represents the whole-package modular decomposition artifact proved acyclic and stratified prior to filesystem code emission.
/// ## 2. Invariants & Mathematical Properties
/// `root_package_name` identifies the root crate or module, `units` have unique `unit_id` keys, `edges` connect declared units, and `topological_order` spans `units`.
/// ## 3. Preconditions
/// Preserves deterministic `units`, `edges`, and `topological_order` sequences so repeated compilations of identical SysML AST inputs emit bit-identical trees.
/// ## 4. Postconditions
/// Cycle (`E0702`) and layering (`E0703`) violations prevent invalid graph acceptance before emission rather than failing mid-write.
/// ## 5. Algorithmic Complexity Bounds
/// Graph storage is `O(|V| + |E|)` over compilation units `V` and dependency edges `E`, using ordered `Vec` containers exclusively.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by testing multi-package DAG assembly, deterministic topological ordering tie-breaks, and rejection of cyclic or upward-coupled graphs.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Aggregates the complete domain module graph consumed by modular codegen emitters and architecture gates.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleGraph {
    pub root_package_name: String,
    pub units: Vec<CompilationUnit>,
    pub edges: Vec<DependencyEdge>,
    pub topological_order: Vec<String>,
}

/// Cryptographic digest and quantitative modularity metrics measured on one emitted source file.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct GeneratedFileDigest` records both SHA-256 content integrity and structural complexity metrics for each generated file in audit manifests.
/// ## 2. Invariants & Mathematical Properties
/// `relative_path` is normalized workspace-relative UTF-8, `sha256` is lowercase hex, and `line_count`, `item_count`, and `max_cyclomatic_complexity` reflect emitted text.
/// ## 3. Preconditions
/// Enables simultaneous `E0704` quantitative ceiling checks against `ModularityBounds` and `E0705` per-file content parity comparison across regeneration passes.
/// ## 4. Postconditions
/// Pure immutable metric record with zero fallible operations; drift or ceiling breaches are surfaced by manifest and bounds verification passes.
/// ## 5. Algorithmic Complexity Bounds
/// Occupies two owned `String` fields and three `usize` counters (`O(|relative_path| + 64)` bytes) with `O(1)` scalar metric access.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by hashing emitted Rust files and asserting exact SHA-256 hex strings, physical line counts, AST item totals, and peak `v(G)`.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Forms the per-file leaf record aggregated by `GeneratedTreeManifest` for fixed-point and bounds certification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFileDigest {
    pub relative_path: String,
    pub sha256: String,
    pub line_count: usize,
    pub item_count: usize,
    pub max_cyclomatic_complexity: usize,
}

/// Deterministic whole-tree cryptographic manifest summarizing all files emitted by a modular codegen run.
/// ## 1. Safety Intent & Regulatory Scope
/// `pub struct GeneratedTreeManifest` anchors stage-1 versus stage-2 regeneration reproducibility under DO-178C deterministic build verification requirements.
/// ## 2. Invariants & Mathematical Properties
/// `files` stores `GeneratedFileDigest` entries sorted in canonical path order, and `tree_sha256` digests the ordered `(relative_path, sha256)` sequence.
/// ## 3. Preconditions
/// Exposes `GeneratedTreeManifest::verify_fixed_point` to prove bit-exact idempotence across consecutive modular compilation stages.
/// ## 4. Postconditions
/// Any cardinality mismatch, per-file path or hash divergence, or aggregate `tree_sha256` mismatch yields `ModularityErrorCode::FixedPointDriftE0705`.
/// ## 5. Algorithmic Complexity Bounds
/// Manifest storage scales as `O(|files|)` over emitted artifacts with zero nondeterministic ordering or ambient filesystem dependencies.
/// ## 6. Failure Modes & Fault Tolerance
/// Verified by comparing identical stage-1 and stage-2 manifests (`Ok(())`) alongside mutated file-count, file-hash, and tree-hash negative probes.
/// ## 7. Realization & Traceability
/// Realises: [REQ-0607]
/// Certifies cryptographic fixed-point convergence across the emitted modular source tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedTreeManifest {
    pub files: Vec<GeneratedFileDigest>,
    pub tree_sha256: String,
}

impl GeneratedTreeManifest {
    /// Verifies bit-exact fixed-point parity between two `GeneratedTreeManifest` snapshots (`self` vs `other`).
    /// ## 1. Safety Intent & Regulatory Scope
    /// `pub fn verify_fixed_point(&self, other: &Self) -> Result<(), ModularityDiagnostic>` enforces regeneration idempotence by rejecting any stage-to-stage drift (`E0705`).
    /// ## 2. Invariants & Mathematical Properties
    /// Both `self` (stage 1) and `other` (stage 2) borrow initialized `GeneratedTreeManifest` values ordered canonically by `relative_path`.
    /// ## 3. Preconditions
    /// Returns `Ok(())` iff `files.len()`, every paired `(relative_path, sha256)` entry, and aggregate `tree_sha256` match identically between `self` and `other`.
    /// ## 4. Postconditions
    /// Returns `Err(ModularityDiagnostic)` tagged with `ModularityErrorCode::FixedPointDriftE0705` pinpointing `"GeneratedTreeManifest"`, the drifting path, or `"tree_sha256"`.
    /// ## 5. Algorithmic Complexity Bounds
    /// Executes a single `O(N)` lockstep zip scan over `files` with cyclomatic complexity `v(G) = 6` (`<= 8`) and allocates only on error path formatting.
    /// ## 6. Failure Modes & Fault Tolerance
    /// Verified by four unit cases: identical manifests, mismatched `files.len()`, mutated per-file `relative_path` or `sha256`, and mutated `tree_sha256`.
    /// ## 7. Realization & Traceability
    /// Realises: [REQ-0607]
    /// Implements the normative `E0705` deterministic fixed-point verification gate on `GeneratedTreeManifest`.
    pub fn verify_fixed_point(&self, other: &Self) -> Result<(), ModularityDiagnostic> {
        if self.files.len() != other.files.len() {
            return Err(ModularityDiagnostic::new(
                ModularityErrorCode::FixedPointDriftE0705,
                "GeneratedTreeManifest",
                format!(
                    "file count drift: stage1={} vs stage2={}",
                    self.files.len(),
                    other.files.len()
                ),
            ));
        }
        for (lhs, rhs) in self.files.iter().zip(other.files.iter()) {
            if lhs.relative_path != rhs.relative_path || lhs.sha256 != rhs.sha256 {
                return Err(ModularityDiagnostic::new(
                    ModularityErrorCode::FixedPointDriftE0705,
                    &lhs.relative_path,
                    format!(
                        "per-file SHA-256 drift on '{}' vs '{}'",
                        lhs.relative_path, rhs.relative_path
                    ),
                ));
            }
        }
        if self.tree_sha256 != other.tree_sha256 {
            return Err(ModularityDiagnostic::new(
                ModularityErrorCode::FixedPointDriftE0705,
                "tree_sha256",
                format!("tree SHA-256 drift: {} != {}", self.tree_sha256, other.tree_sha256),
            ));
        }
        Ok(())
    }
}
