# DEAP Cross-Language Certification & Coding Standards Index

| Field | Value |
| :--- | :--- |
| Document ID | `DEAP-CODING-STANDARDS-INDEX` |
| Version | `2.0` |
| Status | `Active / Normative` |
| Owner | DEAP Systems Engineering |

## 1. Purpose & Architectural Partitioning

DEAP enforces uniform safety-critical engineering assurance across offline Model-Based Systems Engineering (MBSE) compilers, real-time simulation/control engines, and downstream operator consoles while maintaining strict language and runtime separation. Each target tier is governed by a dedicated normative coding standard tailored to its execution model, memory/concurrency semantics, and regulatory boundary:

| Standard | ID & Version | Status | Governed Targets & Regulatory Scope |
| :--- | :--- | :--- | :--- |
| [RUST_CODING_STANDARD.md](./RUST_CODING_STANDARD.md) | `DEAP-RUST-CODING-STANDARD` v2.0 | `Active / Normative` | Tier-0 DO-330 TQL-4 offline SysML v2 compiler and baseline verification crates (`crates/deap-core`, `crates/verify-baseline`, `crates/compile-sysml`, `crates/ingest-sysml`, `crates/assemble-conops`, `crates/reconcile-backlog`), Ring-1 `#![no_std]` Ferrocene Rust `safety_kernel` crates (DO-178C DAL A / ISO 26262 ASIL D / IEC 61508 SIL 4, anchored to Ferrocene `FLS` `[FLS: fls_*]` with dual-mode `criticalup.toml` + `rust-toolchain.toml` parity), and Ring-2 `Miri`-verified `#[repr(C)]` / `flutter_rust_bridge` FFI boundaries. |
| [DART_FLUTTER_CODING_STANDARD.md](./DART_FLUTTER_CODING_STANDARD.md) | `DEAP-DART-FLUTTER-CODING-STANDARD` v2.0 | `Active / Normative` | Ring-3 downstream operator console and untrusted presentation/input shell confined strictly under `app_flutter/` (DAL-D/E / QM assurance containment isolated from Ring-1 via Ring-2 `compile-sysml`-synthesized `flutter_rust_bridge` guards, Ring-1 CRC/SHA-256 sequence-numbered `SafetyMonitor` readback/challenge-response interlocks, `dart analyze --fatal-infos`, 3-Layer MVVM, and 60 FPS `16.67 ms` frame determinism). |

### 1.1 Normative Compiler-Native Verification & 3-Ring Certification Blueprints

Both coding standards operationalize the compiler-native refactoring, formal verification, and 3-ring certification architecture specified across the following master and design blueprints:

| Blueprint Artifact | Normative Role Across Standards |
| :--- | :--- |
| `docs/architecture/blueprints/DEAP_STRIX_HALO_COMPILER_NATIVE_VERIFICATION_ARCHITECTURE.md` | Master workstation and CI architecture uniting lossless CST refactoring, incremental SysML compilation, 5-stage compiler/formal verification, and 3-ring Ferrocene certification. |
| `docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md` | Lossless `ra_ap_syntax` (`rowan`) Rust CST codemods (`--list-rust-items`, `--extract-rust-module`, `--compact-tier-bc`), `syn`/`quote`/`prettyplease` AST codegen, and unified `--message-format=json` `MachineApplicable` diagnostics. |
| `docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md` | Layered static, dynamic-UB, and CBMC formal verification gatekeeper (Clippy/`clippy.toml`, nightly `Dylint` `rustc_middle::mir`, `Miri` `-Zmiri-tree-borrows`, and `compile-sysml`-synthesized `#[kani::proof]` harnesses). |
| `docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md` | `logos` lexing + `rowan` lossless CST + `salsa` demand-driven incremental query architecture for fault-tolerant SysML v2 compilation (`crates/deap-core/src/sysml_ast/mod.rs`). |
| `docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md` | Open-source Ferrocene Language Specification (`FLS` `[FLS: fls_*]`) traceability, dual-mode `criticalup.toml` / `rust-toolchain.toml` parity, 3-Ring Assurance Containment (Ring-1 `#![no_std]` `safety_kernel`, Ring-2 FFI bridge, Ring-3 `app_flutter/`), `symbol-report` certified `core` subset verification, and `blanket` / LLVM Table A-7 MC/DC coverage. |

## 2. Shared Cross-Language Assurance Invariants

Both language standards realize five shared DEAP assurance pillars through idiomatic, AST-verifiable rules:

| Shared Assurance Invariant | Rust Realization ([RUST_CODING_STANDARD.md](./RUST_CODING_STANDARD.md)) | Dart & Flutter Realization ([DART_FLUTTER_CODING_STANDARD.md](./DART_FLUTTER_CODING_STANDARD.md)) |
| :--- | :--- | :--- |
| **1. Requirement & Language-Semantics Traceability (`Realises: [REQ-ID]` / `[FLS: fls_*]`)** | Sections 2.1, 4.2 & 11: Every production item cites a real SysML v2 AST node, requirement ID, or approved design blueprint via `Realises: [REQ-ID]`, paired with Ferrocene `[FLS: fls_*]` anchors on Ring-1/Ring-2/`#[rt_loop]` safety paths; zero fabricated IDs or `PENDING_PARAMETER` cells. | Section 3 (#9) & Section 4: Every production library, class, `sealed class`, `typedef`, and callable member carries `/// Realises: [REQ-ID]` or `/// Realises: [Spec/Class]` traced to authoritative AST/spec artifacts. |
| **2. Tiered Documentation Model (Check 36)** | Section 4.1 & Section 14.1: **Tier A** (`//!`, `pub struct`/`enum`/`trait`/`fn`) requires either the 7-Section Rustdoc or the 4-Block Contract. **Tier B/C** (private/`pub(crate)` items and `impl Trait for Type`) requires a 1-3 line `///` summary + `Realises: [REQ-ID]` (compacted losslessly via `ra_ap_syntax` `--compact-tier-bc`). | Section 4.1-4.2: **Tier A** (`library;`, public types, `sealed class`, `typedef`, public methods) requires either the 7-Section DartDoc or the 4-Block Contract. **Tier B/C** (private `_` helpers, `_State`, `@override` members) requires a 1-3 line `///` summary + `Realises: [REQ-ID]`. |
| **3. Total Functions, 3-Ring FFI Containment & Zero Unchecked Panics / Null-Bangs** | Sections 5, 7, 8 & 10.1: Typed `Result<T, E>` (no `Result<_, String>`), `MAX_AST_DEPTH` recursion bounds, zero `unwrap()`/`expect()`/`panic!`, `.get()` bounds-safe slicing, `TryFrom` conversions, `#![forbid(unsafe_code)]` on Ring-1/domain crates, and `catch_unwind` + `Miri`-verified Ring-2 FFI guards. | Section 1.3, Section 3 (#1-#4, #7) & Section 5: Ring-3 DAL-D/E / QM containment behind `compile-sysml`-synthesized `flutter_rust_bridge` Ring-2 guards, Ring-1 CRC/SHA-256 sequence-numbered `SafetyMonitor` readback/challenge-response interlocks, sealed `Result<T>`, zero `!` in `lib/**`, zero `dynamic`, and `isNaN`/`isInfinite` layout guards. |
| **4. Bounded Complexity & Structural Modularity** | Section 6 & Section 14.1: `< 500` executable SLOC/file, `<= 80` SLOC/function cap (`too-many-lines-threshold = 80`), `<= 5` parameters (`too-many-arguments-threshold = 5`), `<= 3` mutable boolean flags, lossless `ra_ap_syntax` `--extract-rust-module` decomposition, strict `domain <- engine <- adapters/FFI` layering, and MC/DC-reviewable predicates. | Sections 2, 6 & 9: `< 500` executable SLOC/file, `<= 60` SLOC/method, `v(G) <= 10`, max nesting `<= 4`, `<= 5` parameters, mandatory 3-Layer MVVM Definition of Done (`N/A` forbidden), pure `build()`, and `<= 16.67 ms` (60 FPS) frame budget. |
| **5. Zero-Mocking Persistence & 5-Stage Formal / Deterministic Verification Gates** | Sections 9, 12 & 15.1: Native Rust `#[test]` RED-GREEN-REFACTOR TDD, live DB/emulator persistence, unified `--message-format=json` `MachineApplicable` diagnostics, the 5-stage Clippy / `Dylint` MIR / `Miri` / `Kani` / Ferrocene `symbol-report` + `blanket` Table A-7 pipeline, and `verify-baseline`. | Sections 7, 8 & 10: Section 1.9 live SQLite/emulator persistence, Check 38 anti-hardcoding, Check 39 zero-mocking (`mockito`/`mocktail` banned) & reverse-order `dispose()`, Check 40 1080p-6K pairwise HUD non-overlap, `dart analyze --fatal-infos`, `flutter analyze --fatal-infos --fatal-warnings`, and `flutter test`. |

### 2.1 Five-Stage Compiler & Formal Verification Hierarchy

| Stage | Gatekeeper Tooling | Enforced Contract (`blueprint-09` & `blueprint-11`) |
| :--- | :--- | :--- |
| **Stage 1** | Stable / Ferrocene `[workspace.lints.clippy]` + `clippy.toml` | Denies `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `arithmetic_side_effects`, `cast_possible_truncation`, `cast_sign_loss`, and `undocumented_unsafe_blocks`; enforces `too-many-lines-threshold = 80`, `too-many-arguments-threshold = 5`, and `disallowed-types` (paired with `dart analyze --fatal-infos` on Ring-3/Ring-2 Dart FFI bindings). |
| **Stage 2** | Isolated nightly `Dylint` (`rustc_middle::mir`) | Verifies zero heap allocations in `#[rt_loop]` real-time hot paths and audits `[REQ-*]` / `[FLS: fls_*]` traceability annotations out-of-tree. |
| **Stage 3** | `#![forbid(unsafe_code)]` + `cargo +nightly miri test` | Prohibits `unsafe` across pure domain and Ring-1 crates; executes `Miri` (`-Zmiri-tree-borrows`) across isolated Ring-2 FFI/HAL bridge modules. |
| **Stage 4** | `compile-sysml`-synthesized `#[kani::proof]` (CBMC) | Synthesizes bounded model-checking proof harnesses from SysML `constraint def`, port `[min..max]` ranges, `state def` transitions, and STPA `UCA-*`/`SC-*` safety constraints across upstream parsers and downstream safety crates. |
| **Stage 5** | Ferrocene `symbol-report` + `blanket` / LLVM Table A-7 | Verifies Ring-1 `#![no_std]` safety kernels link solely against the certified `core` subset and produces DO-178C Table A-7 statement, decision, and MC/DC coverage bound to SysML `TEST-*`. |

## 3. Governance Precedence & Change Control

1. Both coding standards operate under `.pipeline/constitution.md`, `.pipeline/ACTIVE_RULES_BUNDLE.md`, and dual-mode `criticalup.toml` / `rust-toolchain.toml` toolchain parity (`docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md`). Stricter constitutional or safety-case requirements always take precedence.
2. Verification gates are enforced via lossless CST-, token-, and AST-aware checks in `./target/release/verify-baseline` (supporting `--message-format=json` `MachineApplicable` diagnostics per `docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md`) and the 5-stage compiler/formal verification gatekeeper; scanner evasion, silent lint suppression (`#[allow(...)]` or `// ignore:`), and unapproved waivers are defects.
3. Any modification to files in `docs/standards/` requires an approved plan, fresh context-isolated reviewer verification, zero regressions across `verify-baseline`, and explicit user approval.
