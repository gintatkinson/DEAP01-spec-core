# DEAP Rust Coding Standard

| Field | Value |
| :--- | :--- |
| Standard ID | `DEAP-RUST-CODING-STANDARD` |
| Version | `2.0` |
| Status | `Active / Normative` |
| Supersedes | `1.0` |
| Applies to | All Rust code in DEAP Tier-0 DO-330 offline compiler/verification crates, Ring-1 `#![no_std]` Ferrocene safety kernels, and Ring-2 discrete-time simulation/FFI bridge crates |
| Target assurance | DO-178C DAL A readiness, DO-330 TQL-4 readiness, ISO 26262 ASIL D / IEC 61508 SIL 4 readiness, Ferrocene FLS language-semantics traceability, ARINC 653/661 3-ring partitioning readiness |
| Companion standards | [DART_FLUTTER_CODING_STANDARD.md](./DART_FLUTTER_CODING_STANDARD.md), [README.md](./README.md) |
| Owner | DEAP Systems Engineering |
| Review requirement | Fresh reviewer subagent and explicit user approval before adoption |

## 1. Purpose and Scope

This standard defines mandatory Rust coding, design, testing, resource, and evidence rules for DEAP repositories -- specifically Tier-0 DO-330 offline compiler and verification crates and Track B discrete-time Rust simulation engines and C-ABI adapters. It is written for safety-critical avionics readiness without turning the codebase into an unreadable documentation or abstraction maze.

Client and operator console HMI targets are governed separately; see [DART_FLUTTER_CODING_STANDARD.md](./DART_FLUTTER_CODING_STANDARD.md) for Dart and Flutter client/HMI rules and [README.md](./README.md) for the cross-language certification and assurance index.

This standard is subordinate to the project constitution and platform profiles. If a stricter constitutional, regulatory, or platform rule applies, the stricter rule wins. Conflicts must be resolved by an approved plan, never silently.

This standard is not a certificate. It does not by itself establish DO-178C DAL A, DO-330 TQL-4, ARINC 653, ARINC 661, ISO 26262, or MIL-STD-882E compliance. Certification claims require the evidence package in Section 16.

## 2. Normative Sources and Precedence

The following sources remain authoritative and are consolidated here:

- Open-source **Ferrocene Language Specification (`FLS`)** paragraph anchors (`[FLS: fls_*]`) defining normative Rust syntax, type-system, memory-model, and execution semantics.
- Dual-mode toolchain parity manifests: `criticalup.toml` (qualified Ferrocene compiler and certified `core` subset) and version-matched `rust-toolchain.toml` (token-free developer and sandbox parity).
- `Cargo.toml` `[workspace.lints.clippy]` and `clippy.toml` quantitative complexity/disallowed-type policy.
- Check 34 AST/token source purity and Check 36 Tiered Documentation gates.
- `rules/tdd-mandate.md`.
- `rules/domain-engineering-standards.md`.
- `rules/dual-track-mbd-verification.md`.
- Native Rust requirements in `README.md`.
- `skills/spec-blueprint-engineering/SKILL.md`.
- Compiler-native verification and 3-ring certification architecture blueprints:
  - `docs/architecture/blueprints/DEAP_STRIX_HALO_COMPILER_NATIVE_VERIFICATION_ARCHITECTURE.md`
  - `docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md`
  - `docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md`
  - `docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md`
  - `docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md`
- FFI-specific lints in the dedicated Ring-2 FFI crate manifest.
- `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md`.

If this standard and the current verifier disagree, the standard records the conflict explicitly and the verifier must be corrected upstream. Do not add code or comments solely to satisfy a naive verifier.

### 2.1 Normative Language Semantics (FLS) and Dual-Mode Toolchain Parity

- **Ferrocene Language Specification (`FLS`) Baseline**: Normative Rust language semantics across all upstream Tier-0 compiler crates (`crates/deap-core`, `crates/verify-baseline`, `crates/compile-sysml`, `crates/ingest-sysml`, `crates/assemble-conops`, `crates/reconcile-backlog`) and downstream safety crates are anchored to the open-source Ferrocene Language Specification (`FLS`). Safety-relevant language constructs (integer overflow checks, `#[repr(C)]` layout, pattern exhaustiveness, lifetime/borrowing invariants, and `unsafe` soundness preconditions) cite stable `[FLS: fls_*]` paragraph identifiers rather than informal blog posts or unstable compiler internals.
- **Dual-Mode Toolchain Parity (`blueprint-11`)**: Every governed workspace maintains byte-identical source compatibility across two lockstep toolchain configurations without conditional source forks (`docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md`):
  1. **Qualified Certification Mode (`criticalup.toml`)**: Pins the qualified Ferrocene toolchain (`ferrocene-rustc`), pre-certified `#![no_std]` `core` target subset, `symbol-report` binary inspector, and `blanket` / LLVM structural coverage runtime used to emit DO-178C, DO-330, ISO 26262, and IEC 61508 qualification artifacts.
  2. **Developer & Sandbox Parity Mode (`rust-toolchain.toml`)**: Pins the exact version-matched upstream `rustc` base release corresponding to the `criticalup.toml` Ferrocene release, enabling zero-license-token local TDD, CI sandbox execution, and offline air-gapped verification with identical language semantics and lint behavior.

## 3. Core Engineering Principles

1. Safety first. Fail closed. Never hide an error or weaken a safety gate.
2. Readability first. Clear, explicit, boring code is preferred over clever code.
3. Determinism. Identical inputs produce identical outputs. Ordering, arithmetic, and serialization are deterministic.
4. Small cohesive units. One responsibility per file, module, and function.
5. Narrow public APIs. Expose only what callers need.
6. Evidence over assertion. Correctness claims require raw test, build, verifier, or review evidence.
7. No metric gaming. Complexity, line count, coverage, and mutation scores are diagnostics, not targets.
8. No fake work. No placeholder tests, mock persistence, synthetic telemetry clamps, fabricated traceability, or invented certification evidence.
9. Certifiability and readability are not opposites. A standard that produces unreadable code is non-conformant.

## 4. Documentation and Traceability: Eliminate Boilerplate Noise

The v1.0 rule that every item must carry a 7-section docstring is rejected. It creates mechanical filler, obscures control flow, and is not acceptable DO-178C traceability. All production Rust items outside `#[cfg(test)]` follow the normative Tiered Documentation Model below.

### 4.1 Tiered Documentation Model

**Tier A: Public API and Module Surfaces**

Applies to:

- module headers (`//!`);
- `pub struct`, `pub enum`, and `pub trait`;
- top-level `pub fn` and inherent `pub fn` methods (`impl Type { pub fn ... }`).

Every Tier A item must carry **either** the Full 7-Section Rustdoc **or** the Concise 4-Block Contract:

1. **Option A1 -- Full 7-Section Rustdoc** (recommended for safety-critical domain types and functions, FFI exports and C-ABI boundary types, safety state machines, RTA monitors, and complex SysML action/state/constraint realizations):
   - `## 1. Safety Intent & Regulatory Scope`
   - `## 2. Invariants & Mathematical Properties`
   - `## 3. Preconditions`
   - `## 4. Postconditions`
   - `## 5. Algorithmic Complexity Bounds`
   - `## 6. Failure Modes & Fault Tolerance`
   - `## 7. Realization & Traceability` (must include `Realises: [REQ-ID]`)
2. **Option A2 -- Concise 4-Block Contract** (permitted on any Tier A module header, public type, trait, constructor, accessor, or inherent `pub fn` where 7-section prose would add boilerplate noise):
   - 1-sentence non-placeholder summary of the item's purpose;
   - `Safety & Traceability: Realises: [REQ-ID]`;
   - `Contract:` (preconditions, postconditions, and key invariants);
   - `Bounds & Faults:` (resource/complexity bounds and deterministic error or fault behavior).

**Tier B/C: Internal Helpers, Private Types, and Trait Implementations**

Applies to:

- private `fn` and `pub(crate)` / `pub(super)` internal helper functions and methods;
- private `struct` / `enum` and `pub(crate)` / `pub(super)` internal types;
- trait implementation methods (`impl Trait for Type`, including standard traits such as `Default`, `Display`, `From`, `TryFrom`, `Deref`, and `Error`).

Every Tier B/C item outside `#[cfg(test)]` must carry:

- a concise `<= 4`-line non-placeholder `///` summary stating purpose and any key invariant or fault behavior; and
- `/// Realises: [REQ-ID]` (paired with `[FLS: fls_*]` where normative language semantics are relied upon).

Docblock Line Cap & Explicit Tier-A Opt-In:
- `max_tier_bc_doc_lines <= 4`: Internal and helper items are strictly capped at 4 doc lines (`/// [DOC-TIER: A]` opt-in is required to exceed 4 lines).
- Multi-section Tier-A docblocks (Option A1 7-Section or Option A2 4-Block Contract) are permitted on internal items only with an explicit unbackticked `[DOC-TIER: A]` opt-in tag on the first doc line (e.g., `/// [DOC-TIER: A] ...`).
- Padded Tier-B/C docstrings (`> 4` lines) without `[DOC-TIER: A]` fail Check 36.
- No 28-line 7-section boilerplate is permitted on private helpers, crate-internal items, or trait implementations unless explicitly opted into Tier A. Test functions and helpers inside `#[cfg(test)]` require only clear, non-placeholder test naming or a 1-line `///` summary.

### 4.2 Traceability rules

- Every `Realises: [REQ-ID]` tag must point to a real requirement, SysML AST node, or approved design artifact.
- Safety-critical Ring-1 `#![no_std]` kernel items, Ring-2 `unsafe` FFI/HAL contracts, and real-time `#[rt_loop]` functions must pair `[REQ-*]` requirement tags with normative Ferrocene Language Specification `[FLS: fls_*]` anchors wherever language-level representation, evaluation, or soundness guarantees are relied upon (verified mechanically by Stage 2 `Dylint` in Section 15.1).
- If no upstream domain requirement applies to a foundational utility, cite the governing architecture/compiler requirement or approved design artifact explicitly. Never invent a fake ID.
- Do not copy generic boilerplate such as "O(M(2)) bounded over input/member dimension" onto trivial code.
- Do not duplicate docblocks.
- Comments must explain why, safety intent, invariants, and failure behavior. They must not restate the code.
- Comments and Rustdoc are excluded from the 500 SLOC limit. Do not reduce documentation to satisfy a line count.

### 4.3 Verifier alignment (Check 36)

Check 36 previously enforced a 7-section docstring on every item, directly causing the boilerplate noise floor. Check 36 must enforce the Tiered Documentation Model above via tier-aware AST inspection: accepting either the 7-section Rustdoc or the 4-Block Contract on Tier A items, and requiring a concise non-placeholder summary plus `Realises: [REQ-ID]` on Tier B/C items without forcing 7-section templates onto private helpers or `impl Trait for Type` methods. Until and after that verifier upgrade lands, mechanical filler remains forbidden.

## 5. Recursion, Stack, and Resource Determinism

Unbounded recursion is forbidden in safety-critical paths.

- Every recursive function must have an explicit depth bound or be replaced by an iterative traversal.
- AST traversal, digest counting, serialization, and validation must be iterative or use an explicit `MAX_AST_DEPTH` guard.
- The default maximum AST depth is 64 unless the schema proves a different bound.
- Exceeding the depth bound returns a typed error. It must not panic, abort, or silently truncate.
- Tests must cover the maximum depth and maximum depth plus one.
- Recursion is forbidden in FFI call paths unless the depth is statically bounded and stack usage is documented.
- ARINC 653 partition stack budgets are fixed. Provide static stack analysis or a documented worst-case stack budget for every safety path.
- Prefer explicit work stacks over call-stack recursion for unbounded or schema-driven structures.
- No unbounded allocation, unbounded queue growth, or unbounded work per tick.

## 6. File and Function Decomposition: Prevent God Files and MC/DC Explosion

- `<= 500` executable SLOC per production file (`< 380` target executable SLOC per submodule for high modularity). Comments, Rustdoc, and blank lines are excluded.
- **3-Layer Definition of Done & Legacy Scanner Deletion**: Every Tier-0 compiler, ingestion, verification, and downstream safety micro-task must close all 3 layers (Layer 1: Domain State & AST/CST Data Model, Layer 2: Logic, Query & State Management, Layer 3: CLI/Diagnostic Presentation & Actuator Interface Binding; `N/A` is forbidden) targeting `< 380`-SLOC submodules and explicitly deleting legacy hand-rolled character/line scanners.
- `<= 80` executable lines per function hard cap (`too-many-lines-threshold = 80`). 50 lines is the target.
- `<= 5` parameters maximum per function (`too-many-arguments-threshold = 5`). Use a context struct beyond that.
- No more than 3 independent mutable boolean state flags in one function. Use a state enum or a state struct.
- One responsibility per file and module. Split parser, synchronization, safety-suite, and codegen god files by phase and responsibility.
- Zero hand-rolled lexers, scanners, or custom ad-hoc parsers. All lexical analysis and syntax trees must use the canonical compiler-crate stack (`logos` + `rowan` + `salsa` + `ariadne`, `ra_ap_syntax`, `pulldown-cmark` + `pulldown-cmark-to-cmark`, `syn` + `proc-macro2` + `quote` + `prettyplease`).
- Dependency direction: domain <- engine <- adapters and FFI. No cycles. No layer violations.
- MC/DC-friendly design:
  - Prefer single-purpose conditions.
  - Extract named predicates for compound conditions.
  - Avoid compound boolean expressions with more than 3 operands in a decision.
  - Use exhaustive `match` on enums instead of flag webs.
  - Keep functions small enough that the MC/DC vector set is reviewable.
- No metric gaming:
  - One statement per line.
  - Respect rustfmt; do not manually pack multiple statements onto one line.
  - Maximum line length is 100 characters unless rustfmt or a generated-code exception applies.
  - Do not use `concat!`, string concatenation, split tokens, or macro tricks to evade a scanner.
  - Do not use `.then_some().unwrap_or()` or similar one-liners to evade branch or line-count gates.
  - Do not hide control flow in macros.

## 7. Typed Errors and Fault Handling

- All fallible production operations return `Result<T, E>` with a typed error.
- `Result<_, String>` and `Result<i32, (i32, String)>` are forbidden in production code.
- Safety and FFI boundaries use stable error codes, preferably `#[repr(u16)]` enums or an exhaustive typed error enum.
- Fault handlers must switch deterministically on closed error codes. They must not inspect string prefixes or allocate heap strings on a fault path.
- Error codes are part of the interface contract and must be documented and stable.
- `Option<T>` is only for absence, never for failure.
- No `unwrap()`, `expect()`, `panic!`, `todo!`, `unimplemented!`, or `dbg!` in production code.
- No partial state mutation on failure. Validate before mutating.
- Error messages must be deterministic and must not expose secrets.

## 8. Memory Safety, Indexing, and Arithmetic

- Direct slice indexing and slicing on untrusted input are forbidden. Use `.get()`, `split_at_checked`, `char_indices`, `strip_prefix`, or explicit pattern matching.
- Raw `+`, `-`, `*` arithmetic on lengths, offsets, or indices must use checked, saturating, or explicitly proven-safe operations.
- No `as` casts that can truncate. Use `TryFrom` and handle the error.
- Validate UTF-8 boundaries before any string slice. Test multi-byte boundaries.
- No panics on malformed input. Return a typed error.
- Clippy `indexing_slicing` remains forbidden.
- Fuzz or property-test every parser, lexer, frontmatter, and scanner boundary.

## 9. Anti-Evasion and Honest Metrics

- Scanner evasion is a defect, not a workaround. The following are forbidden:
  - `concat!("reg", "ex::")` or any split-token construction to avoid a substring check;
  - `["pan", "ic!("].concat()` or equivalent;
  - one-line packing to avoid line or branch caps;
  - `.then_some().unwrap_or()` tricks to avoid branch caps;
  - macro indirection that hides control flow from review.
- Check 34's naive substring scanner is a root cause of evasion. The verifier must be upgraded to token-based or AST-based checks. Until then, do not game the scanner; write honest code and record the verifier defect.
- Metrics are diagnostics. Do not split a coherent algorithm into artificial helpers, replace clear control flow with lookup tables, or add traits and generics solely to lower a score.
- A reviewer must be able to explain the function in one pass without tracing a web of helpers.

## 10. FFI, ARINC 653, and ARINC 661 Boundaries

- `unsafe` is forbidden in pure domain and engine crates. It is permitted only in a dedicated FFI crate or module with an explicit, reviewed lint override.
- Every C-ABI export is wrapped in `std::panic::catch_unwind`. Rust panics must never unwind across the FFI boundary.
- C-ABI structs use `#[repr(C)]` and fixed-size scalar fields. Do not expose Rust `String`, `Vec`, `Option`, or trait objects directly.
- Native allocations require explicit deallocation exports and reference counting or finalizer ownership. Cross-allocator frees are forbidden.
- Snapshot rings are lock-free or wait-free as specified, with bounded slots and explicit sequence or epoch counters.
- Command envelopes carry checksum validation. Cryptographic signature verification must be real. A non-zero byte check is not authentication.
- ARINC 653 partitioning:
  - fixed stack and heap budgets per partition;
  - no shared mutable heap across partitions;
  - no unbounded recursion;
  - no blocking in the safety loop;
  - deterministic scheduling and bounded queues;
  - read-only snapshot transfer across the display boundary.
- ARINC 661 CDS/UA boundary:
  - explicit display data model and port contract;
  - read-only snapshot pages and bounded command ring buffers;
  - no client-induced corruption of the Rust solver across the C-ABI boundary;
  - non-blocking snapshot publication within the `16.67 ms` (60 FPS) display cycle and fault isolation evidence;
  - end-to-end harness tests, not design statements alone.
- FFI and partitioning claims require evidence. A design document is not evidence.

### 10.1 Three-Ring Assurance Containment Architecture

Downstream systems integrating Rust safety logic with an operator console enforce the 3-Ring Assurance Containment Architecture defined in `docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md`:

- **Ring-1 (`#![no_std]` Certified Safety Kernel -- DAL-A/B / ASIL-D / SIL-4)**: Pure deterministic control laws, state machines, voting logic, STPA `UCA-*`/`SC-*` interlocks, and the authoritative `SafetyMonitor` reside in a `#![no_std]`, `#![forbid(unsafe_code)]`, zero-heap-allocation `safety_kernel` crate linked exclusively against the Ferrocene `symbol-report`-verified certified `core` subset.
- **Ring-2 (Isolated FFI / OS / HAL Bridge -- Audited Boundary)**: All `#[repr(C)]` C-ABI exports, OS/HAL I/O adapters, and `compile-sysml`-synthesized `flutter_rust_bridge` transport shims reside strictly in Ring-2. Every `unsafe` block carries a `// SAFETY:` proof comment citing `[FLS: fls_*]` preconditions (`clippy::undocumented_unsafe_blocks` denied), traps unwinding via `std::panic::catch_unwind`, validates range/discriminant bounds before constructing Ring-1 types, and passes `cargo +nightly miri test` with `-Zmiri-tree-borrows`.
- **Ring-3 (Untrusted Presentation & Operator Input Shell -- DAL-D/E / QM)**: Downstream HMI clients (`app_flutter/`) execute outside the trusted computing base. Safety-critical operator commands and annunciations crossing Ring-3 -> Ring-2 -> Ring-1 require Ring-1 `SafetyMonitor` sequence-numbered CRC-32C/SHA-256 readback and challenge-response interlocks so corrupted or stale UI frames cannot trigger unauthorized actuation.

## 11. DO-178C DAL A and DO-330 TQL-4 Readiness

For DAL A readiness, the repository must provide:

- requirement-to-code (`[REQ-*]` and `[FLS: fls_*]`) and requirement-to-test (`TEST-*`) traceability;
- DO-178C Table A-7 statement, decision, and MC/DC structural coverage evidence via Ferrocene `blanket` / LLVM instrumentation bound to SysML `TEST-*` cases, plus Ferrocene `symbol-report` certified `core` subset verification on Ring-1 `#![no_std]` binaries;
- formal bounded model-checking proof harnesses (`#[kani::proof]`) for SysML `constraint def`, port `[min..max]`, `state def`, and STPA `UCA-*`/`SC-*` invariants;
- independence evidence for verification;
- robustness and boundary testing;
- configuration management, dual-mode `criticalup.toml` / `rust-toolchain.toml` lockstep parity, and problem-reporting records;
- deterministic build and tool outputs.

For DO-330 TQL-4 readiness, any tool that generates, transforms, or verifies safety data must have:

- documented tool operational requirements;
- tool validation and regression evidence;
- deterministic output behavior;
- configuration and version control;
- qualification evidence appropriate to its role.

The Rust compiler, code generators, SysML compilers, and verifiers are tools. Their outputs must be independently verified or qualified. Do not claim tool qualification without evidence.

## 12. Testing and TDD

- All executable code changes follow RED-GREEN-REFACTOR.
- RED must be an executed assertion failure. A compile error is not RED.
- Tests assert behavior, invariants, bounds, and failure modes, not implementation details.
- Every specified failure and exception path has a negative test.
- Use native Rust `#[test]`. Python, pytest, and shell test runners are forbidden.
- Use property-based tests for mathematical kernels and fuzzing for parsers and FFI boundaries.
- Persistence and integration tests use a live database or emulator. In-memory mocks are not acceptable for persistence tiers.
- Tests are deterministic. Do not assert wall-clock values that depend on machine load; assert bounds.
- No fake tests, smoke-only tests, hardcoded expected values copied from implementation, or string-presence tests.
- Safety-critical invariants require mutation or negative probes that prove the test can fail.
- DAL A MC/DC evidence is mandatory for safety-relevant functions. Do not substitute line coverage for MC/DC.

## 13. Performance and Resource Bounds

- Respect the project latency hierarchy. In this product line: FFI transfer <= 10 us, fast loop <= 5 ms, warm repair <= 10 ms, cold solve <= 500 ms, replan limit <= 15 s, planning limit <= 60 s.
- No synthetic latency floors or clamps. Report measured values or fail closed.
- Hot loops have bounded allocation. Preallocate and reuse buffers.
- No unbounded queues, recursion, or work per tick.
- Use deterministic ordering. Do not iterate hash maps in output paths.
- Provide worst-case execution time, stack usage, and heap usage evidence where applicable.

## 14. Dependencies, Official Compiler Stack, and Build Hygiene

- Add dependencies only with written justification covering maintenance, license, security, and safety impact.
- Zero regular expressions: the `regex` crate is forbidden across all crates. Use deterministic compiler-crate tokenization and CST inspection.
- Zero hand-rolled lexers, scanners, or ad-hoc parsers: hand-rolled character-by-character scanners, substring probes, and custom lexers are strictly forbidden across Tier-0 compiler crates and downstream safety logic. All parsing, lexing, CST construction, and syntax manipulation must use the official compiler-crate stack.
- Do not add a dependency that requires `unsafe` in pure domain or engine crates.
- Keep `cargo fmt`, `cargo clippy -- -D warnings`, and the build clean.
- Do not suppress lints without an approved, scoped, documented waiver.

### 14.1 Canonical Compiler-Crate Stack (Contract B)

All structural inspection, refactoring, code synthesis, document parsing, workspace analysis, and diagnostic emission across DEAP Tier-0 tooling must align 100% with the official compiler-crate stack (with zero hand-rolled scanners and explicit deletion of legacy hand-rolled scanners):

- **Workspace & Manifests (`cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, `cargo-platform`, `toml_edit`)**: Structured Cargo workspace inspection, deterministic `Cargo.lock` parsing and serialization, Cargo manifest schema validation, cross-platform target-triple evaluation, and roundtrip whitespace/comment-preserving TOML manipulation for `criticalup.toml`, `rust-toolchain.toml`, and `Cargo.toml`.
- **Lossless Rust CST, VFS & Codemods (`ra_ap_syntax`, `ra_ap_vfs`, `ra_ap_paths`, `ra_ap_line_index`, `ra_ap_text_edit`, `rowan`, `camino`, `ignore`)**: Automated Rust structural inspection (`ast::{MatchExpr, MatchArmList, CallExpr, MethodCallExpr, TryExpr, Fn, HasDocComments, HasAttrs}`), item slicing (`--list-rust-items`), god-file decomposition (`--extract-rust-module` targeting `< 380`-SLOC submodules), and docstring compaction (`--compact-tier-bc`) execute over lossless `ra_ap_syntax` Concrete Syntax Trees via `crates/compile-sysml/src/cli.rs` and `deap_core::rust_cst`, preserving comments, attributes, and exact token trivia without brittle line-offset slicing.
- **Lossless SysML v2, Dart & Polyglot CST + Incremental Query DB (`logos` + `rowan` + `salsa` + `ariadne`)**: SysML v2.0 + KerML 1.0 ingestion, lossless syntax-tree construction (`deap_core::sysml_cst`), Dart client CST auditing (`deap_core::dart_cst`), polyglot OMG IDL and RFC 7950 YANG CST frontends (`ingest_sysml::cst::{idl_cst, yang_cst}`), and typed `serde_json` OpenAPI/JSON-Schema extraction across all `SupportedSchemaExtension` formats (`.sysml`, `.yaml`, `.yml`, `.json`, `.proto`, `.idl`, `.arxml`, `.md`, `.markdown`, `.yang`, plus `.rs` and `.dart` CSTs) use `logos` zero-copy DFA lexing, `rowan` lossless green/red CST nodes (`SysmlLanguage`), `salsa` demand-driven incremental query memoization (`blueprint-10`), and `ariadne` multi-span diagnostic rendering.
- **Lossless CommonMark Markdown AST (`pulldown-cmark` + `pulldown-cmark-to-cmark`)**: Specification parsing, Markdown table extraction, frontmatter boundary handling, and deterministic roundtrip Markdown serialization without ad-hoc string surgery.
- **Deterministic Fixed-Point Codegen (`syn` + `proc-macro2` + `quote` + `prettyplease`)**: Synthesized Rust domain modules, Ring-2 `flutter_rust_bridge` FFI guards, and Stage 4 `#[kani::proof]` harnesses are constructed as typed `syn` / `proc_macro2` token trees via `quote!` and formatted deterministically with `prettyplease`, eliminating string-interpolation syntax defects and line-packing evasion.
- **Unified `--message-format=json` `MachineApplicable` Diagnostics**: Verification and compilation binaries (`crates/verify-baseline/src/runner.rs` and `crates/compile-sysml/src/cli.rs`) support structured `--message-format=json` diagnostic emission mirroring `rustc` JSON spans with `MachineApplicable` replacement edits so automated repair loops apply deterministic span-verified fixes (`blueprint-08`).

### 14.2 Cross-Platform Toolchain & Runtime Portability Contract

All toolchain discovery, driver invocation, file path handling, and external tool coordination must execute portably across macOS, Linux, and Windows:

- **Dynamic Driver Probing**: Probe binaries and compilers dynamically via host `PATH` using `std::env::consts::EXE_SUFFIX` (`.exe` on Windows, empty on Unix) rather than assuming fixed POSIX paths or single-host machine configurations.
- **Dynamic Library Portability**: Dynamic libraries across FFI boundaries must use `std::env::consts::DLL_PREFIX` and `std::env::consts::DLL_SUFFIX` (`lib*.dylib` on macOS, `lib*.so` on Linux, `*.dll` on Windows). Hardcoding `.dylib`, `.so`, or `.dll` literals without platform normalization is forbidden.
- **Path Portability & Normalization**: Path manipulation must use `std::path::Path`, `std::path::PathBuf`, or `camino::Utf8Path` / `camino::Utf8PathBuf` paired with `ignore` for workspace traversal. Normalization must handle CRLF vs LF line endings and Windows backslash separators transparently.
- **Zero Single-Host or Machine-Specific Paths**: Single-host paths (such as hardcoded OS user home directories or temporary paths), Strix-Halo-only hardcoded execution assumptions, or machine/user-specific directories are strictly forbidden in code and normative specifications. Workspace paths must always be workspace-relative (`./...`) or dynamically derived from the environment.

### 14.3 Contract A: 100% Normative OMG SysML v2.0 + KerML 1.0 Textual Grammar Coverage (`deap_core::sysml_cst` & `SupportedSchemaExtension`)

All Tier-0 SysML v2 parsers (`deap_core::sysml_cst`, `crates/compile-sysml`), HIR metamodels (`crates/deap-core/src/sysml_ast/mod.rs`), and schema extractors (`SupportedSchemaExtension`) must enforce 100% normative OMG SysML v2.0 + KerML 1.0 textual grammar coverage with full Definition & Usage symmetry across all normative clauses and **zero 16-construct artificial caps**:

1. **Namespaces & Metadata**: `package`, `library package`, `standard library package`, `import` (`all`, `::*`, `::**`, filter `[...]`), `alias`, `doc`, `comment`, `rep`, `dependency`, `metadata def` / `metadata` / `@Metadata` / `#user_keyword` (`#hazard`, `#risk`, `#capability`, `#rt_loop`, `#dal`, `#fls` via standard OMG SysML v2 `metadata def` keywords while preserving backward-compatible parsing of legacy `hazard def` / `risk def` / `capability def`).
2. **Structural & Data Types**: `attribute def`/`attribute`, `enum def`/`enum`, `item def`/`item`, `part def`/`part`, `port def`/`port` (`in`, `out`, `inout`, `~` conjugation), `connection def`/`connection` (`connect ... to ...`), `interface def`/`interface`, `flow def`/`flow` (`of ... from ... to ...`), `allocation def`/`allocation` (`allocate ... to ...`), `occurrence def`/`occurrence` (`individual`, `snapshot`, `timeslice`).
3. **Behavioral & State**: `action def`/`action` (`perform`, `accept`, `send`, `assign`, `if`/`else`, `while`/`loop`, `for`, `fork`, `join`, `merge`, `decide`, `first ... then ...` succession), `state def`/`state` (`exhibit`, `parallel`, `entry`, `do`, `exit`, `transition first ... accept ... if ... do ... then ...`).
4. **Calculations, Constraints & Requirements**: `calc def`/`calc` (`return`), `constraint def`/`constraint` (`assert`, `assume`, `require`), `requirement def`/`requirement` (`subject`, `actor`, `stakeholder`, `framed concern`, `satisfy ... by ...`), `concern def`/`concern`.
5. **Use Cases, Analysis, Verification & Views**: `use case def`/`use case` (`include use case`), `analysis def`/`analysis` (`objective`), `verification def`/`verification` (`verify requirement`), `test case def`/`test case`, `interaction def`/`interaction`, `view def`/`view`, `viewpoint def`/`viewpoint`, `rendering def`/`rendering` (`expose`, `filter`, `render`), `variation`/`variant`.
6. **KerML 1.0 Specialization, Multiplicity & Expressions**: `:` (`defined by`), `:>` (`subsets`), `:>>` (`redefines`), `::>` (`references`), `~` (`conjugates`), `ref`, `abstract`, `readonly`, `derived`, `end`, `default`, `:=`, `[lo..hi]`, `[*]`, `{ordered, nonunique}`, full KerML logical/conditional/relational/arithmetic/collection expressions (`implies`, `xor`, `??`, `->`, `collect`, `select`, `forAll`, `exists`), and `ISQ`/`SI`/`ScalarValues` standard libraries.

## 15. Mandatory Gates and Enforcement

Run the applicable gates for the changed path:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
./target/release/verify-baseline . --no-domain
```

For downstream Track B C-ABI FFI adapter crates:

```bash
CARGO_TARGET_DIR=target cargo test --manifest-path ffi/<engine-ffi-crate>/Cargo.toml
CARGO_TARGET_DIR=target cargo clippy --manifest-path ffi/<engine-ffi-crate>/Cargo.toml --all-targets -- -D warnings
CARGO_TARGET_DIR=target cargo fmt --check --manifest-path ffi/<engine-ffi-crate>/Cargo.toml
CARGO_TARGET_DIR=target cargo build --release --manifest-path ffi/<engine-ffi-crate>/Cargo.toml
```

Client and operator console HMI verification gates are defined in [DART_FLUTTER_CODING_STANDARD.md](./DART_FLUTTER_CODING_STANDARD.md).

Enforcement must be AST-based or token-based. Substring scanners are not acceptable for code purity or documentation quality. A gate failure blocks the issue. Do not weaken the gate or hide the failure.

### 15.1 Five-Stage Compiler and Formal Verification Gatekeeper

All upstream Tier-0 crates and downstream 3-ring Rust workspaces enforce the layered 5-Stage Compiler and Formal Verification Gatekeeper defined in `docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md`, `docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md`, and `docs/architecture/blueprints/DEAP_STRIX_HALO_COMPILER_NATIVE_VERIFICATION_ARCHITECTURE.md`:

| Stage | Verification Engine | Normative Scope & Enforced Invariants |
| :--- | :--- | :--- |
| **Stage 1** | Stable / Ferrocene `rustc` / `cargo clippy --message-format=json` (`[workspace.lints.clippy]` + `clippy.toml`) | Denies `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, `arithmetic_side_effects`, `cast_possible_truncation`, `cast_sign_loss`, and `undocumented_unsafe_blocks`; enforces `too-many-lines-threshold = 80`, `too-many-arguments-threshold = 5`, and `disallowed-types` policy in `clippy.toml`. |
| **Stage 2** | Isolated nightly `Dylint` in `./lints/deap-dylint-mir/` (`#![feature(rustc_private)]`, `rustc_middle::mir`) | Runs out-of-tree against `rustc_middle::mir` CFGs to verify zero heap allocations (`alloc`, `Box`, dynamic collection growth) reachable from `#[rt_loop]` real-time hot paths and mechanically audit `[REQ-*]` / `[FLS: fls_*]` traceability tags without coupling production builds to nightly `rustc`. |
| **Stage 3** | `#![forbid(unsafe_code)]` + `cargo miri` + `flutter_rust_bridge` | Enforces `#![forbid(unsafe_code)]` on all pure domain, SysML compiler, and Ring-1 `safety_kernel` crates; executes `MIRIFLAGS="-Zmiri-tree-borrows" cargo +nightly miri test` across isolated Ring-2 `flutter_rust_bridge` and FFI/HAL bridge modules to prove absence of aliasing, provenance, alignment, and uninitialized-memory UB. |
| **Stage 4** | `compile-sysml`-synthesized `cargo kani` (`#[kani::proof]`) | Synthesizes bounded model-checking proof harnesses from SysML v2 `constraint def` blocks, port `[min..max]` scalar intervals, `state def` transition tables, and STPA `UCA-*` / `SC-*` safety constraints to formally verify panic freedom and invariant preservation across upstream DEAP parsers and downstream safety crates. |
| **Stage 5** | Ferrocene `criticalup.toml` + `[FLS: fls_*]` + `symbol-report`/`blanket` MC/DC + 3-Ring DAL-A..E Partitioning | Verifies Ring-1 `#![no_std]` safety kernels link exclusively against the Ferrocene pre-certified `core` symbol subset via `symbol-report`, generates DO-178C Table A-7 statement, decision, and MC/DC structural coverage reports via `blanket` / LLVM bound directly to SysML `TEST-*` verification cases, and enforces 3-Ring DAL-A..E partitioning with cross-platform runtime `PATH` discovery across macOS, Linux, and Windows. |

## 16. Certification Evidence Package

A certification claim requires at least:

- a software development and verification plan;
- requirement, design, and code standards mapped to `[FLS: fls_*]` Ferrocene Language Specification sections and DO-178C / DO-330 objectives;
- dual-mode `criticalup.toml` and `rust-toolchain.toml` version-parity configuration records;
- requirement-to-code (`[REQ-*]`, `[FLS: fls_*]`) and requirement-to-test (`TEST-*`) bidirectional traceability;
- Stage 4 `#[kani::proof]` CBMC formal verification reports covering SysML `constraint def`, port `[min..max]`, `state def`, and STPA `UCA-*`/`SC-*` properties;
- Stage 5 Ferrocene `symbol-report` certified `core` subset logs for Ring-1 `#![no_std]` kernels and `blanket` / LLVM Table A-7 statement, decision, and MC/DC structural coverage evidence bound to SysML `TEST-*`;
- Stage 2 `Dylint` `#[rt_loop]` zero-allocation MIR audit logs and Stage 3 `Miri` (`-Zmiri-tree-borrows`) Ring-2 FFI soundness logs;
- independence and review records;
- configuration management and problem reporting;
- tool qualification evidence for generation and verification tools;
- ARINC 653 partitioning evidence where applicable;
- ARINC 661 CDS/UA interface and Ring-1 `SafetyMonitor` interlock evidence where applicable;
- a safety case with zero unresolved `PENDING_PARAMETER` cells.

Do not claim DAL A, TQL-4, ARINC 653, or ARINC 661 compliance without this evidence.

## 17. Review Checklist

A reviewer must confirm:

- [ ] The change is scoped to one issue.
- [ ] Every production file is under 500 executable SLOC (`< 380` target SLOC per submodule under the 3-Layer Definition of Done) or has an approved justification.
- [ ] Functions are under 80 executable lines (`too-many-lines-threshold = 80`) or have a written justification.
- [ ] No function has more than 5 parameters (`too-many-arguments-threshold = 5`) or more than 3 independent mutable boolean flags without a state type.
- [ ] Recursion is bounded or replaced by iteration; depth tests exist.
- [ ] Zero hand-rolled lexers, scanners, or custom ad-hoc parsers (legacy hand-rolled scanners explicitly deleted); official compiler-crate stack (`logos` + `rowan` + `salsa` + `ariadne`, `ra_ap_syntax`, `ra_ap_vfs`, `ra_ap_paths`, `ra_ap_line_index`, `ra_ap_text_edit`, `camino`, `ignore`, `pulldown-cmark` + `pulldown-cmark-to-cmark`, `syn` + `proc-macro2` + `quote` + `prettyplease`, `cargo_metadata`, `cargo-lock`, `cargo-util-schemas`, `cargo-platform`, `toml_edit`) used exclusively.
- [ ] SysML v2 parsers and extractors (`deap_core::sysml_cst`, `SupportedSchemaExtension`) preserve 100% normative OMG SysML v2.0 + KerML 1.0 grammar coverage with full Definition & Usage symmetry and zero 16-construct artificial caps (Section 14.3).
- [ ] No god-file growth; new responsibilities are split via lossless `ra_ap_syntax` CST refactoring (`--list-rust-items`, `--extract-rust-module`) where automated, and codegen uses `syn`/`proc-macro2`/`quote`/`prettyplease`.
- [ ] Cross-platform portability across macOS, Linux, and Windows is maintained (`std::env::consts::{DLL_PREFIX, DLL_SUFFIX, EXE_SUFFIX}`, `camino`/`std::path`, zero hardcoded host user or machine paths).
- [ ] No stringly-typed production errors.
- [ ] No scanner evasion, line packing, or metric gaming.
- [ ] No direct slice indexing or unchecked arithmetic on untrusted input (`indexing_slicing`, `arithmetic_side_effects`, `cast_possible_truncation`, `cast_sign_loss` denied).
- [ ] `#![forbid(unsafe_code)]` holds outside Ring-2 FFI/HAL boundaries; Ring-2 `unsafe` blocks are documented (`undocumented_unsafe_blocks` denied) and pass `cargo +nightly miri test` (`-Zmiri-tree-borrows`).
- [ ] No panic, unwrap, expect, todo, unimplemented, unreachable, dbg, or regex.
- [ ] Documentation is tier-appropriate: Tier-B/C items enforce `<= 4`-line docstrings (`max_tier_bc_doc_lines <= 4`); multi-section Tier-A is strictly opt-in via `[DOC-TIER: A]`; no 7-section boilerplate on trivial items (`--compact-tier-bc` compliant).
- [ ] Traceability points to real `[REQ-*]` requirements/AST nodes and pairs `[FLS: fls_*]` anchors on safety-critical Ring-1/Ring-2/`#[rt_loop]` items.
- [ ] Stage 2 `Dylint` (`#[rt_loop]` zero-allocation MIR gate), Stage 4 `#[kani::proof]` harnesses, and Stage 5 `symbol-report` / Table A-7 coverage gates pass where applicable.
- [ ] Negative and exception paths are tested.
- [ ] RED failure was recorded before implementation.
- [ ] GREEN gates pass with raw output.
- [ ] No fake tests, mocks in persistence tiers, or synthetic telemetry.
- [ ] The issue has a verification evidence comment and `status:fixed-resolved`.

## 18. Known Anti-Patterns and Required Corrections

The following anti-patterns were identified in the current upstream compiler codebase. They are recorded here as known deviations. The standard is not conformant until they are corrected or formally waived.

Active workspace remediation is executed mechanically through the compiler-native refactoring, incremental SysML CST/query, and 5-stage formal verification architecture specified in:

- `docs/architecture/blueprints/DEAP_STRIX_HALO_COMPILER_NATIVE_VERIFICATION_ARCHITECTURE.md`
- `docs/designs/blueprint-08-rust-analyzer-cst-refactoring-and-json-diagnostics.md`
- `docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md`
- `docs/designs/blueprint-10-rowan-salsa-incremental-sysml-compiler.md`
- `docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md`

| Anti-pattern | Evidence supplied by the audit | Required correction |
| :--- | :--- | :--- |
| 7-section docstring noise floor | `reconcile-backlog/src/tracker.rs`, `compile-sysml/src/docstring.rs`: 25-30 lines of boilerplate on 3-line getters, `Default` impls, and private helpers. | Apply the Tier A/B/C model in Section 4. Execute lossless `ra_ap_syntax` CST compaction (`--compact-tier-bc`, `blueprint-08`) and enforce tier-aware AST Check 36 with `--message-format=json` `MachineApplicable` diagnostics. |
| Unbounded recursion and stack exhaustion | `semantic/digest.rs` (`accumulate_package_counts`, `collect_schema_nodes`), `semantic/validator.rs` (`check_parts_recursive`), `semantic/serializer.rs` (`to_sysml`) recurse directly on AST depth. | Bound all recursion with `MAX_AST_DEPTH` or migrate to `logos` + `rowan` lossless CST iterative traversal with `salsa` memoized queries (`blueprint-10`), backed by Stage 4 `#[kani::proof]` depth-bound harnesses (`blueprint-09`). |
| Monolithic god files and high-arity functions | `compile-sysml/src/sync/forward.rs` (~3,000 lines), `parser/grammar.rs` (~2,400 lines), `stpa/safety_suite.rs` (~2,000 lines); functions span hundreds of lines with deep state. | Decompose via `ra_ap_syntax` `--list-rust-items` and `--extract-rust-module` (`blueprint-08`) to the 500/80/5 limits enforced by Stage 1 `clippy.toml` (`too-many-lines-threshold = 80`, `too-many-arguments-threshold = 5`, `blueprint-09`). |
| Stringly-typed errors | `driver.rs`, `sync/forward.rs`, `sync/reverse.rs`, `reconcile-backlog/src/tracker.rs` return `Result<_, String>` or `Result<i32, (i32, String)>`. | Replace with typed error enums or stable `#[repr(u16)]` codes and emit unified `--message-format=json` diagnostics (`blueprint-08`). Remove string-prefix fault handling. |
| Scanner-evasion hacks and metric gaming | `concat!("reg", "ex::")`, `concat!(".unwrap", "()")`, `["pan", "ic!("].concat()` in `blueprint_gate.rs` and `docstring.rs`; one-line packing and `.then_some().unwrap_or()` in `modular_codegen/emitters/rust_emitter.rs`. | Ban evasion. Replace raw string-template emitters with `syn`/`quote`/`prettyplease` AST codegen (`blueprint-08`) and enforce Stage 1-5 Clippy, `Dylint` MIR, `Miri`, `Kani`, and Ferrocene `symbol-report` gates (`blueprint-09`, `blueprint-11`). |
| Unchecked slice indexing and arithmetic | `reconcile-backlog/src/frontmatter.rs`, `checklist.rs`, `compile-sysml/src/lexer/scanner.rs` use direct slicing and raw `+`/`-` arithmetic. | Enforce Stage 1 Clippy `indexing_slicing`, `arithmetic_side_effects`, `cast_possible_truncation`, and `cast_sign_loss` (`blueprint-09`), migrate lexing to `logos` + `rowan` (`blueprint-10`), and prove bounds via Stage 4 `#[kani::proof]` harnesses. |

## 19. Waivers and Deviations

A waiver is valid only when:

1. It is documented in an approved plan.
2. It names the exact rule, file, and reason.
3. It is time-bounded or tied to a follow-up issue.
4. It is reviewed by a fresh reviewer.
5. It does not weaken a safety invariant or a certification gate.

Silent lint suppression, unexplained `#[allow(...)]`, and undocumented exceptions are forbidden. Existing violations must be recorded in the issue tracker and remediated, not hidden.

## 20. Change Control

Changes to this standard require an approved plan, a fresh reviewer, and explicit user approval. The standard must not be silently weakened to accommodate an implementation. If an implementation cannot meet the standard, the implementation is blocked until the standard is formally amended or the exception is approved.
