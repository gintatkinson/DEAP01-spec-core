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
| [RUST_CODING_STANDARD.md](./RUST_CODING_STANDARD.md) | `DEAP-RUST-CODING-STANDARD` v2.0 | `Active / Normative` | Tier-0 DO-330 TQL-4 offline SysML v2 compiler and baseline verification crates (`crates/*`), Track B DO-178C DAL A discrete-time Rust simulation kernels, and `#[repr(C)]` ARINC 653 / ARINC 661 C-ABI FFI boundaries. |
| [DART_FLUTTER_CODING_STANDARD.md](./DART_FLUTTER_CODING_STANDARD.md) | `DEAP-DART-FLUTTER-CODING-STANDARD` v2.0 | `Active / Normative` | Downstream operator console and HMI client targets confined strictly under `app_flutter/` (DO-178C DAL B/C HMI readiness, ARINC 661 User Application display/actuator boundary, 3-Layer MVVM, and 60 FPS `16.67 ms` frame determinism). |

## 2. Shared Cross-Language Assurance Invariants

Both language standards realize five shared DEAP assurance pillars through idiomatic, AST-verifiable rules:

| Shared Assurance Invariant | Rust Realization ([RUST_CODING_STANDARD.md](./RUST_CODING_STANDARD.md)) | Dart & Flutter Realization ([DART_FLUTTER_CODING_STANDARD.md](./DART_FLUTTER_CODING_STANDARD.md)) |
| :--- | :--- | :--- |
| **1. Requirement Traceability (`Realises: [REQ-ID]`)** | Sections 4.2 & 11: Every production item cites a real SysML v2 AST node, requirement ID, or approved design blueprint via `Realises: [REQ-ID]`; zero fabricated IDs or `PENDING_PARAMETER` cells. | Section 3 (#9) & Section 4: Every production library, class, `sealed class`, `typedef`, and callable member carries `/// Realises: [REQ-ID]` or `/// Realises: [Spec/Class]` traced to authoritative AST/spec artifacts. |
| **2. Tiered Documentation Model (Check 36)** | Section 4.1: **Tier A** (`//!`, `pub struct`/`enum`/`trait`/`fn`) requires either the 7-Section Rustdoc or the 4-Block Contract. **Tier B/C** (private/`pub(crate)` items and `impl Trait for Type`) requires a 1-3 line `///` summary + `Realises: [REQ-ID]`. | Section 4.1-4.2: **Tier A** (`library;`, public types, `sealed class`, `typedef`, public methods) requires either the 7-Section DartDoc or the 4-Block Contract. **Tier B/C** (private `_` helpers, `_State`, `@override` members) requires a 1-3 line `///` summary + `Realises: [REQ-ID]`. |
| **3. Total Functions & Zero Unchecked Panics / Null-Bangs** | Sections 5, 7 & 8: Typed `Result<T, E>` (no `Result<_, String>`), `MAX_AST_DEPTH` recursion bounds, zero `unwrap()`/`expect()`/`panic!`, `.get()` bounds-safe slicing, `TryFrom` conversions, and `catch_unwind` at C-ABI exports. | Section 3 (#1-#4, #7) & Section 5: Sealed `Result<T>` (`Success`/`Failure`) with typed `DomainError`, zero `!` null-assertion operators in `lib/**`, zero `dynamic`, `.elementAtOrNull` access, and `isNaN`/`isInfinite` layout guards. |
| **4. Bounded Complexity & Structural Modularity** | Section 6: `< 500` executable SLOC/file, `<= 80` SLOC/function cap (`50` target), `<= 5` parameters, `<= 3` mutable boolean flags, strict `domain <- engine <- adapters/FFI` layering, and MC/DC-reviewable predicates. | Sections 2, 6 & 9: `< 500` executable SLOC/file, `<= 60` SLOC/method, `v(G) <= 10`, max nesting `<= 4`, `<= 5` parameters, mandatory 3-Layer MVVM Definition of Done (`N/A` forbidden), pure `build()`, and `<= 16.67 ms` (60 FPS) frame budget. |
| **5. Zero-Mocking Persistence & Deterministic Verification Gates** | Sections 9, 12 & 15: Native Rust `#[test]` RED-GREEN-REFACTOR TDD, property/fuzz testing, live DB/emulator persistence (zero in-memory mocks), zero scanner-evasion tricks, `cargo clippy -- -D warnings`, `cargo test --workspace`, and `verify-baseline`. | Sections 7, 8 & 10: Section 1.9 live SQLite/emulator persistence, Check 38 anti-hardcoding, Check 39 zero-mocking (`mockito`/`mocktail` banned) & reverse-order `dispose()`, Check 40 1080p-6K pairwise HUD non-overlap, `flutter analyze`, and `flutter test`. |

## 3. Governance Precedence & Change Control

1. Both coding standards operate under `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md`. Stricter constitutional or safety-case requirements always take precedence.
2. Verification gates are enforced via token- and AST-aware checks in `./target/release/verify-baseline`; scanner evasion, silent lint suppression (`#[allow(...)]` or `// ignore:`), and unapproved waivers are defects.
3. Any modification to files in `docs/standards/` requires an approved plan, fresh context-isolated reviewer verification, zero regressions across `verify-baseline`, and explicit user approval.
