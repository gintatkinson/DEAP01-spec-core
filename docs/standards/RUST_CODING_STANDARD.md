# DEAP Rust Coding Standard

| Field | Value |
| :--- | :--- |
| Standard ID | `DEAP-RUST-CODING-STANDARD` |
| Version | `2.0` |
| Status | `Draft for review` |
| Supersedes | `1.0` |
| Applies to | All Rust code in DEAP upstream compiler and downstream application repositories |
| Target assurance | DO-178C DAL A readiness, DO-330 TQL-4 readiness, ARINC 653 partitioning readiness, ARINC 661 CDS boundary readiness |
| Owner | DEAP Systems Engineering |
| Review requirement | Fresh reviewer subagent and explicit user approval before adoption |

## 1. Purpose and Scope

This standard defines mandatory Rust coding, design, testing, resource, and evidence rules for DEAP repositories. It is written for safety-critical avionics readiness without turning the codebase into an unreadable documentation or abstraction maze.

This standard is subordinate to the project constitution and platform profiles. If a stricter constitutional, regulatory, or platform rule applies, the stricter rule wins. Conflicts must be resolved by an approved plan, never silently.

This standard is not a certificate. It does not by itself establish DO-178C DAL A, DO-330 TQL-4, ARINC 653, ARINC 661, ISO 26262, or MIL-STD-882E compliance. Certification claims require the evidence package in Section 16.

## 2. Normative Sources and Precedence

The following sources remain authoritative and are consolidated here:

- `Cargo.toml` workspace lints.
- Check 34 source purity.
- `rules/tdd-mandate.md`.
- `rules/domain-engineering-standards.md`.
- `rules/dual-track-mbd-verification.md`.
- Native Rust requirements in `README.md`.
- `skills/spec-blueprint-engineering/SKILL.md`.
- FFI-specific lints in the dedicated FFI crate manifest.
- `.pipeline/constitution.md` and `.pipeline/ACTIVE_RULES_BUNDLE.md`.

If this standard and the current verifier disagree, the standard records the conflict explicitly and the verifier must be corrected upstream. Do not add code or comments solely to satisfy a naive verifier.

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

The v1.0 rule that every item must carry a 7-section docstring is rejected. It creates mechanical filler, obscures control flow, and is not acceptable DO-178C traceability.

### 4.1 Documentation tiers

**Tier A: Full 7-section Rustdoc**

Required for:

- public safety-relevant domain types and functions;
- FFI exports and C-ABI boundary types;
- safety state machines, RTA monitors, and safety constraints;
- functions that implement a requirement or a SysML action, state, or constraint;
- functions with non-trivial failure modes or resource bounds.

The 7 sections are exactly:

1. `## 1. Safety Intent & Regulatory Scope`
2. `## 2. Invariants & Mathematical Properties`
3. `## 3. Preconditions`
4. `## 4. Postconditions`
5. `## 5. Algorithmic Complexity Bounds`
6. `## 6. Failure Modes & Fault Tolerance`
7. `## 7. Realization & Traceability`

**Tier B: Concise contract documentation**

Required for public non-safety helpers and simple public constructors. It must state purpose, preconditions, postconditions, failure behavior, and traceability if one exists. It does not need the 7-section template.

**Tier C: Minimal or no documentation**

Applies to private helpers, trivial getters and setters, derived `Default`/`Clone`/`Debug` implementations, and test helpers. A one-line doc is sufficient. No fabricated 7-section boilerplate is permitted.

### 4.2 Traceability rules

- Every traceability tag must point to a real requirement, SysML AST node, or approved design artifact.
- If no traceability exists, state that explicitly. Do not invent an ID.
- Do not copy generic boilerplate such as "O(M(2)) bounded over input/member dimension" onto trivial code.
- Do not duplicate docblocks.
- Comments must explain why, safety intent, invariants, and failure behavior. They must not restate the code.
- Comments and Rustdoc are excluded from the 500 SLOC limit. Do not reduce documentation to satisfy a line count.

### 4.3 Verifier conflict

Check 36 currently enforces a 7-section docstring on every item. That rule is the direct cause of the boilerplate noise floor. The verifier must be upgraded to a tier-aware AST check based on visibility, safety annotation, requirement linkage, and complexity. Until that upgrade lands, do not add mechanical filler. Record the conflict in the issue and use the tiered rule above.

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

- 500 executable SLOC per production file. Comments, Rustdoc, and blank lines are excluded.
- 80 executable lines per function hard cap. 50 lines is the target.
- 5 parameters maximum. Use a context struct beyond that.
- No more than 3 independent mutable boolean state flags in one function. Use a state enum or a state struct.
- One responsibility per file and module. Split parser, synchronization, safety-suite, and codegen god files by phase and responsibility.
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
  - no UI-induced corruption of the solver;
  - 60 FPS frame budget and fault isolation evidence;
  - end-to-end harness tests, not design statements alone.
- FFI and partitioning claims require evidence. A design document is not evidence.

## 11. DO-178C DAL A and DO-330 TQL-4 Readiness

For DAL A readiness, the repository must provide:

- requirement-to-code and requirement-to-test traceability;
- MC/DC structural coverage evidence for safety-relevant code;
- independence evidence for verification;
- robustness and boundary testing;
- configuration management and problem-reporting records;
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

## 14. Dependencies and Build Hygiene

- Add dependencies only with written justification covering maintenance, license, security, and safety impact.
- The `regex` crate is forbidden. Use deterministic slice, character, or token scanning.
- Do not add a dependency that requires `unsafe` in the pure engine crate.
- Keep `cargo fmt`, `cargo clippy -- -D warnings`, and the build clean.
- Do not suppress lints without an approved, scoped, documented waiver.

## 15. Mandatory Gates and Enforcement

Run the applicable gates for the changed path:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
./target/release/verify-baseline . --no-domain
```

For the C-ABI adapter:

```bash
CARGO_TARGET_DIR=target cargo test --manifest-path ffi/cuas-coa-engine-ffi/Cargo.toml
CARGO_TARGET_DIR=target cargo clippy --manifest-path ffi/cuas-coa-engine-ffi/Cargo.toml --all-targets -- -D warnings
CARGO_TARGET_DIR=target cargo fmt --check --manifest-path ffi/cuas-coa-engine-ffi/Cargo.toml
CARGO_TARGET_DIR=target cargo build --release --manifest-path ffi/cuas-coa-engine-ffi/Cargo.toml
```

For Flutter paths:

```bash
cd app_flutter && flutter analyze
cd app_flutter && flutter test
```

Enforcement must be AST-based or token-based. Substring scanners are not acceptable for code purity or documentation quality. A gate failure blocks the issue. Do not weaken the gate or hide the failure.

## 16. Certification Evidence Package

A certification claim requires at least:

- a software development and verification plan;
- requirement, design, and code standards with objective mapping;
- requirement-to-code and requirement-to-test traceability;
- MC/DC structural coverage evidence;
- independence and review records;
- configuration management and problem reporting;
- tool qualification evidence for generation and verification tools;
- ARINC 653 partitioning evidence where applicable;
- ARINC 661 CDS/UA interface evidence where applicable;
- a safety case with zero unresolved `PENDING_PARAMETER` cells.

Do not claim DAL A, TQL-4, ARINC 653, or ARINC 661 compliance without this evidence.

## 17. Review Checklist

A reviewer must confirm:

- [ ] The change is scoped to one issue.
- [ ] Every production file is under 500 executable SLOC or has an approved justification.
- [ ] Functions are under 80 executable lines or have a written justification.
- [ ] No function has more than 5 parameters or more than 3 independent mutable boolean flags without a state type.
- [ ] Recursion is bounded or replaced by iteration; depth tests exist.
- [ ] No god-file growth; new responsibilities are split.
- [ ] No stringly-typed production errors.
- [ ] No scanner evasion, line packing, or metric gaming.
- [ ] No direct slice indexing or unchecked arithmetic on untrusted input.
- [ ] No `unsafe` outside the FFI boundary.
- [ ] No panic, unwrap, expect, todo, unimplemented, dbg, or regex.
- [ ] Documentation is tier-appropriate; no 7-section boilerplate on trivial items.
- [ ] Traceability points to real requirements or AST nodes.
- [ ] Negative and exception paths are tested.
- [ ] RED failure was recorded before implementation.
- [ ] GREEN gates pass with raw output.
- [ ] No fake tests, mocks in persistence tiers, or synthetic telemetry.
- [ ] The issue has a verification evidence comment and `status:fixed-resolved`.

## 18. Known Anti-Patterns and Required Corrections

The following anti-patterns were identified in the current upstream compiler codebase. They are recorded here as known deviations. The standard is not conformant until they are corrected or formally waived.

| Anti-pattern | Evidence supplied by the audit | Required correction |
| :--- | :--- | :--- |
| 7-section docstring noise floor | `reconcile-backlog/src/tracker.rs`, `compile-sysml/src/docstring.rs`: 25-30 lines of boilerplate on 3-line getters, `Default` impls, and private helpers. | Apply the Tier A/B/C model in Section 4. Replace Check 36 with a tier-aware AST check. Remove mechanical filler. |
| Unbounded recursion and stack exhaustion | `semantic/digest.rs` (`accumulate_package_counts`, `collect_schema_nodes`), `semantic/validator.rs` (`check_parts_recursive`), `semantic/serializer.rs` (`to_sysml`) recurse directly on AST depth. | Bound all recursion with `MAX_AST_DEPTH` or use iterative traversal. Add depth-limit tests and stack-budget evidence. |
| Monolithic god files and high-arity functions | `compile-sysml/src/sync/forward.rs` (~3,000 lines), `parser/grammar.rs` (~2,400 lines), `stpa/safety_suite.rs` (~2,000 lines); functions span hundreds of lines with deep state. | Split by responsibility to the 500/80/5 limits. Replace flag webs with state enums. Design for MC/DC. |
| Stringly-typed errors | `driver.rs`, `sync/forward.rs`, `sync/reverse.rs`, `reconcile-backlog/src/tracker.rs` return `Result<_, String>` or `Result<i32, (i32, String)>`. | Replace with typed error enums or stable `#[repr(u16)]` codes. Remove string-prefix fault handling. |
| Scanner-evasion hacks and metric gaming | `concat!("reg", "ex::")`, `concat!(".unwrap", "()")`, `["pan", "ic!("].concat()` in `blueprint_gate.rs` and `docstring.rs`; one-line packing and `.then_some().unwrap_or()` in `modular_codegen/emitters/rust_emitter.rs`. | Ban evasion. Upgrade Check 34 to token or AST scanning. Enforce one statement per line and rustfmt. |
| Unchecked slice indexing and arithmetic | `reconcile-backlog/src/frontmatter.rs`, `checklist.rs`, `compile-sysml/src/lexer/scanner.rs` use direct slicing and raw `+`/`-` arithmetic. | Use `.get()`, `split_at_checked`, `char_indices`, and checked arithmetic. Add UTF-8 boundary and fuzz tests. |

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
