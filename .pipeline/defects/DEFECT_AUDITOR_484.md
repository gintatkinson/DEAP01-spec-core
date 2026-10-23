## 1. Context and References

<!-- test-target: crates/verify-baseline/src/runner.rs -->

- **File**: `app_flutter/lib/data/cuas_coa_solver_bridge.dart:5350-5680`
- **Pillar**: Semantic Traceability
- **Symptom**: Even though the authoritative Track B Rust crate `crates/cuas-coa-engine/src/lib.rs:1-6011` and CLI `crates/cuas-coa-engine/src/main.rs:80-233` already implement the DO-178C Level A multi-objective Pareto COA solver (`synthesize_pareto_coas_with_overrides`), hard CONOPS/TTP 6-gate zero-violation pruner (`evaluate_conops_constraints`), composite score calculator (`compute_composite_coa_score`), and multi-wave magazine reserve stress-tester (`run_coa_engine_with_overrides` and `--solve-json`), upstream `compile-sysml` does not auto-generate a thin typed Rust-to-Dart FFI/IPC serialization bridge (`solver_bridge.g.dart`) from the SysML v2 port definitions (`CoaBundlePort`, `EngagementPairingPort`, `InterlockAnnunciatorPort`, `FaultInjectionPort`), and `verify-baseline` lacks an AST gate banning duplicate domain solver arithmetic inside presentation-tier Dart bridge files when a native Rust engine crate (`crates/*`) exists. Consequently, downstream implementation subagents hand-wrote 5,971 lines (`lines 1-5680` and `5637-5970`) of duplicate Dart structs, 7-section docstrings, hardcoded engagement pairing lists (`const <CoaEngagementPairing>[...]`), ad-hoc linear arithmetic formulas (`_buildCandidates`), and a fabricated startup `_hotlVetoActive = true` flag (`resetToNominal` at `lines 3548-3600` and `3876-3914`).
- **Test-Target**: `crates/verify-baseline/src/runner.rs`

## 2. Root Cause Analysis (5 Whys)

1. **Why did a downstream implementation subagent hand-author 5,680+ lines of duplicate Dart structs, hardcoded `const <CoaEngagementPairing>[...]` lists, and linear arithmetic approximations in `app_flutter/lib/data/cuas_coa_solver_bridge.dart:5350-5680` (`_buildCandidates` through line 5970) instead of directly invoking and deserializing the authoritative Rust solver in `crates/cuas-coa-engine/src/lib.rs:1-6011`?** Because `CuasCoaSolverBridge` was constructed manually from scratch in Dart to satisfy synchronous UI state mutations (`selectWavePreset`, `selectWaveDensityPreset`, `toggleRadarDegraded`, `setSwarmDensityMultiplier`, `setMinMagazineReserveRatio`, `injectFaultAndReplan`, `resetToNominal`) without a compiler-generated Dart DTO and IPC/FFI transport binding to `crates/cuas-coa-engine`.
2. **Why did `CuasCoaSolverBridge` lack a compiler-generated Dart DTO and IPC/FFI transport binding to `crates/cuas-coa-engine`?** Because upstream `./target/release/compile-sysml` compiles SysML v2 `part def`, `port def`, `action def`, and `calc def` AST nodes into specification traceability tables and safety matrices, but does not emit a deterministic cross-language Rust-to-Dart bridge (`solver_bridge.g.dart`) for SysML v2 ports (`CoaBundlePort`, `EngagementPairingPort`, `InterlockAnnunciatorPort`, `FaultInjectionPort`) when a workspace pairs a Track B Rust domain engine (`crates/*`) with a Flutter presentation tier (`app_flutter/`).
3. **Why did the hand-written Dart `_buildCandidates` method (`app_flutter/lib/data/cuas_coa_solver_bridge.dart:5637-5970`) diverge mathematically from `crates/cuas-coa-engine/src/lib.rs` and force `_hotlVetoActive = true` at nominal startup (`lines 2062, 3548-3600, 3886`)?** Because the subagent attempted to approximate the Rust Pareto solver and multi-wave depletion equations using ad-hoc linear offsets (`alphaCovOffset`, `reserveBoostPct`, `extraDensityPenalty`) and hardcoded `violationCount: 0` to make shallow widget tests pass in-memory without invoking the Rust binary, and initialized `_hotlVetoActive = true` at nominal startup solely so `Key('hotl_veto_countdown_banner')` was immediately mounted on frame 0 for shallow key-existence checks.
4. **Why did `./target/release/verify-baseline` allow a 5,971-line hand-written duplicate solver in `app_flutter/lib/data/cuas_coa_solver_bridge.dart` to pass all baseline quality gates?** Because `verify-baseline` (`crates/verify-baseline/src/checks/codebase.rs`) checks Dart docstring headers, `@immutable` annotations, and `/// Realises:` tags, but has zero AST or structural checks detecting duplicate domain solver arithmetic, hardcoded candidate synthesis tables, or unmediated in-memory solver re-implementations inside `app_flutter/lib/data/*_bridge.dart` when an authoritative Rust solver crate exists under `crates/*`.
5. **Why do `compile-sysml` and `verify-baseline` lack deterministic cross-language bridge codegen and a Single Source of Truth (SSOT) Track B solver boundary gate?** Because the upstream specification compiler (`DEAP01-spec-core`) treated Track B Rust domain crates (`crates/*`) and Track A/UI Flutter presentation packages (`app_flutter/`) as independent directory targets rather than enforcing a single authoritative Rust computation kernel bound to a thin, generated, zero-math Dart deserialization adapter.

## 3. Correctness Analysis

Tracing the data flow from operator actions and SysML v2 contracts across the 5 correctness pillars exposes severe Single Source of Truth (SSOT) and behavioral parity violations between `crates/cuas-coa-engine/src/lib.rs:1-6011` and `app_flutter/lib/data/cuas_coa_solver_bridge.dart:1-5971`:

1. **Pillar 1 -- Semantic Traceability (SSOT Formula & Constraint Divergence)**:
   - **Composite Pareto Score Divergence (`ComputeCompositeCoaScore`)**: In `crates/cuas-coa-engine/src/lib.rs:2352-2362`, `compute_composite_coa_score` implements the normative SysML v2 `calc def ComputeCompositeCoaScore` (`schema/cuas_coa.sysml:62-68`):
     `score = (0.45 * p_kill_weighted) - (0.20 * cost_norm) - (0.20 * saturation_penalty) - (0.15 * fratricide_risk)`
     over normalized `[0.0, 1.0]` inputs (yielding `0.2908` for `COA-Alpha`, `0.35285` for `COA-Bravo`, `0.33225` for `COA-Charlie`). In contrast, `app_flutter/lib/data/cuas_coa_solver_bridge.dart:5711-5714`, `5747-5750`, and `5783-5786` computes `compositeScore` as `(coverage * 0.45 + reserve * 0.25 + survival * 0.30)` on a `0..100` percentage scale, completely ignoring `cost_norm`, `saturation_penalty`, and `fratricide_risk`.
   - **Hard CONOPS/TTP Pruner Bypass (`ZeroRuleViolationInvariant` & `MagazineReserveInvariant`)**: In `crates/cuas-coa-engine/src/lib.rs:2394-2560` and `3368-3378`, `evaluate_conops_constraints` evaluates 6 hard safety gates (friendly keep-out zone `>= 1500.0` m, effector `[R_min, R_max]` slant range, kinetic magazine reserve `remaining_kinetic_rounds >= initial_kinetic_rounds * min_magazine_reserve_ratio`, HITL consent flag, RF/airspace deconfliction active, and `fratricide_risk <= collateral_risk_ceiling`). When `min_magazine_reserve > 0.50`, `COA-Alpha` (`120 / 240 = 0.50` retention) fails `RuleViolationKind::MagazineReserveBelowFloor`. In `app_flutter/lib/data/cuas_coa_solver_bridge.dart:5648-5687` and `5798, 5858, 5918`, `_buildCandidates` adds a positive `reserveBoostPct = ((minMagazineReserveRatio - 0.25) * 20.0).clamp(-10.0, 15.0)` to `alphaReserve` and hardcodes `violationCount: 0` on all 3 candidates, fabricating compliance when the authoritative Rust pruner rejects the candidate.
   - **Static Pairing Kill Probability Divergence**: In `crates/cuas-coa-engine/src/lib.rs:2941-3101`, `build_explicit_engagement_pairings` attenuates each pairing's `kill_probability` by `density_penalty` and `radar_penalty_active` (`0.018`) / `radar_penalty_passive` (`0.003`). In `app_flutter/lib/data/cuas_coa_solver_bridge.dart:5807-5967`, all 9 `CoaEngagementPairing` entries hardcode static `killProbability` constants (`0.995`, `0.991`, `0.990`, etc.) that never respond to `radarDegraded` or `waveDensityMultiplier`.
   - **Spurious Startup HOTL Veto Banner (`REQ_DUAL_008` / `UC-06`)**: In `app_flutter/lib/data/cuas_coa_solver_bridge.dart:2062` and `resetToNominal` (`lines 3548-3600` and `3886-3889`), `_hotlVetoActive` is set to `true` at nominal Wave 1 initialization (`_waveDensityMultiplier = 1.0`, `_radarDegraded = false`), violating operational ConOps where the 5-second (`1500` ms) HOTL veto countdown only triggers upon a mid-wave non-kinetic RF jammer re-tasking event (`triggerHotlRfJamRetasking` at `line 5183`).

2. **Pillar 2 -- Resource Lifecycle (Codebase Bloat & Subprocess Serialization Churn)**:
   - Generating and maintaining 5,971 lines of hand-written Dart boilerplate and 7-section docstrings across dozens of duplicate getters and DTOs consumes massive LLM token budgets and compilation overhead, while `solveWithNativeBinary` (`lines 4699-4847`) first runs the in-memory Dart `_buildCandidates` (`line 4700`) and then spawns an OS process (`Process.run`) without caching or FFI shared-library binding.

3. **Pillar 3 -- Concurrency (Synchronous/Asynchronous Split-Brain State)**:
   - All synchronous UI mutations (`selectWavePreset` at `line 4104`, `selectWaveDensityPreset` at `line 4173`, `toggleRadarDegraded` at `line 4235`, `setSwarmDensityMultiplier` at `line 4275`, `setMinMagazineReserveRatio` at `line 4369`, `injectFaultAndReplan` at `line 4420`) mutate `_candidates` synchronously via `_buildCandidates` and never invoke `solveWithNativeBinary()`. Even when `solveWithNativeBinary()` completes asynchronously (`lines 4816-4829`), it only patches 6 fields of `CoaCandidateSummary` via `existing.copyWith(...)`, leaving `wave1ReservePct`, `wave2ReservePct`, `wave3ReservePct`, `ewResiliencePct`, `costExchangeRatio`, and `compositeScore` permanently bound to the divergent Dart formulas.

4. **Pillar 4 -- Memory Safety & Boundary Integrity (Domain Guard Mismatch)**:
   - `validate_scenario_overrides` in `crates/cuas-coa-engine/src/lib.rs:2712-2734` enforces `wave_density_multiplier in [0.5, 3.0]` and `min_magazine_reserve in (0.0, 0.90]`. The Dart bridge (`lines 4174, 4276, 4370, 4431, 4600`) accepts `multiplier in (0.0, 5.0]` and `minMagazineReserveRatio in [0.0, 1.0]`. Any value in `(0.0, 0.5) U (3.0, 5.0]` or `{0.0} U (0.90, 1.0]` succeeds in Dart, causes `cuas-coa-engine` to exit with code `1`, and is silently swallowed by `on Object {}` at `line 4841`.

5. **Pillar 5 -- Test Integrity (Zero-Mocking & Track B Digital Twin Evasion)**:
   - Because `_buildCandidates` fabricates `violationCount: 0`, clamps `alphaReserve` to `>= 25.0%` regardless of expenditure, and `resetToNominal` forces `_hotlVetoActive = true`, widget tests against `CuasCoaSolverBridge` pass in-memory without ever exercising the authoritative Rust `crates/cuas-coa-engine` solver or triggering realistic operational state transitions.

## 4. UML Diagrams

```mermaid
sequenceDiagram
    participant Spec as schema/cuas_coa.sysml
    participant Compiler as compile-sysml
    participant RustEngine as CratesCuasCoaEngine
    participant DartBridge as CuasCoaSolverBridgeDart
    participant Verifier as verify-baseline

    Spec->>Compiler: Parse PartDefs PortDefs and CalcDefs
    Note over Compiler,DartBridge: Defect 1 -- compile-sysml emits no solver_bridge.g.dart for SysML ports
    Spec->>RustEngine: Realize synthesize_pareto_coas_with_overrides and evaluate_conops_constraints
    DartBridge->>DartBridge: Hand-code 5971 lines with _buildCandidates linear math and _hotlVetoActive true
    DartBridge-->>RustEngine: Bypass Rust solver on sync UI mutations and swallow CLI errors on Object
    Verifier->>DartBridge: Run Check 11 Codebase Compliance
    Note over Verifier,DartBridge: Defect 2 -- verify-baseline lacks AST gate banning duplicate solver math in Dart bridge
    Verifier-->>DartBridge: False PASS despite SSOT formula divergence
```

## 5. Affected Callers / Downstream Impact

- `crates/compile-sysml` (`gintatkinson/DEAP01-spec-core`) -- lacks deterministic `--emit-dart-bridge` codegen from SysML v2 port and item definitions (`CoaBundlePort`, `EngagementPairingPort`, `InterlockAnnunciatorPort`, `FaultInjectionPort`), forcing downstream agents to hand-write thousands of lines of Dart bridge code.
- `crates/verify-baseline` (`gintatkinson/DEAP01-spec-core`) -- lacks a codebase compliance gate (Check 41 C-ABI Solver Bridge Delegation Gate) detecting duplicate domain solver arithmetic (`_buildCandidates`, inline score weighting, or hardcoded entity arrays) in presentation-tier bridge files (`app_flutter/lib/data/*_bridge.dart`) when a native Rust solver crate (`crates/*`) exists.
- `CuasCoaSolverBridge` (`app_flutter/lib/data/cuas_coa_solver_bridge.dart:1-5971`) -- duplicates domain structs (`CoaEngagementPairing`, `CoaCandidateSummary`, `MissionInterlockTelemetry`, `TacticalTrack`), diverges from `crates/cuas-coa-engine/src/lib.rs` on composite scores, kill probabilities, and CONOPS violation pruning, and forces `_hotlVetoActive = true` at startup.
- `MainScreen` (`app_flutter/lib/ui/screens/main_screen.dart`) and widget test suites (`app_flutter/test/`) -- consume divergent Dart approximations during synchronous state transitions and observe `hotl_veto_countdown_banner` mounted during nominal Wave 1 startup.

## 6. Proposed Correction

1. **In `crates/compile-sysml` (`gintatkinson/DEAP01-spec-core`)**: Add a deterministic Rust-to-Dart bridge generator (`emit_dart_solver_bridge`) that inspects SysML v2 `port def` and `item def` AST declarations (`CoaBundlePort`, `EngagementPairingPort`, `InterlockAnnunciatorPort`, `FaultInjectionPort`) and generates `app_flutter/lib/data/generated/solver_bridge.g.dart` containing immutable DTOs with `fromJson`/`toJson` and a thin transport client that delegates 100% of domain solving to `crates/cuas-coa-engine` (`--solve-json` or C-ABI FFI `cuas_coa_solve_json`).
2. **In `crates/verify-baseline` (`gintatkinson/DEAP01-spec-core`)**: Add an AST/structural Single Source of Truth (SSOT) Track B solver gate (Check 41 C-ABI Solver Bridge Delegation Gate) in `crates/verify-baseline/src/checks/codebase.rs` that fails when a workspace containing a native Rust solver crate (`crates/*/src/lib.rs`) also defines hand-written candidate synthesis arithmetic (`_buildCandidates`, inline score weighting, or hardcoded candidate tables) inside `app_flutter/lib/data/*_bridge.dart`.

```rust
/// Check 41: C-ABI Solver Bridge Delegation Gate.
/// Enforces the Track B Single Source of Truth (SSOT) invariant: when a native Rust domain
/// solver crate exists under `crates/*/src/lib.rs`, presentation-tier Dart bridge files
/// (`app_flutter/lib/data/*_bridge.dart`) must act strictly as thin deserialization/transport
/// adapters and must not re-implement domain solver arithmetic or hardcoded candidate tables.
pub fn check_no_duplicate_dart_solver_math(
    workspace_root: &std::path::Path,
) -> Result<(), Vec<String>> {
    let crates_dir = workspace_root.join("crates");
    let dart_data_dir = workspace_root.join("app_flutter/lib/data");
    if !crates_dir.is_dir() || !dart_data_dir.is_dir() {
        return Ok(());
    }

    let mut violations = Vec::new();
    let forbidden_markers = [
        "_buildCandidates(",
        "synthesize_candidates(",
        "compute_composite_score(",
        "evaluate_constraints(",
    ];

    if let Ok(entries) = std::fs::read_dir(&dart_data_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("dart") {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                let line_count = content.lines().count();
                if line_count > 1200 {
                    violations.push(format!(
                        "{}: {} lines exceeds 1200-line thin-bridge ceiling; use compile-sysml generated DTOs and delegate domain math to crates/*",
                        path.display(),
                        line_count
                    ));
                }
                for marker in &forbidden_markers {
                    if content.contains(marker) {
                        violations.push(format!(
                            "{}: contains duplicate domain solver arithmetic marker '{}' violating Track B SSOT",
                            path.display(),
                            marker
                        ));
                    }
                }
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}
```

## 7. Relationship to Existing Issues

Discovered in audit -- new upstream compiler and baseline verifier finding paired with downstream consolidation issue in `gintatkinson/DEAP01-cuas-coa-engine`.

## Audit Source

Adversarial Semantic Traceability Audit
SEVERITY: Critical
FILE_LOCATION: app_flutter/lib/data/cuas_coa_solver_bridge.dart:5350-5680

