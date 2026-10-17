---
title: "Implementation Profile -- Pipeline Tooling (UPSTREAM ONLY)"
project: "Digital Engineering Agent Platform (DEAP)"
tier: implementation
platform: pipeline-tooling
scope: upstream-only
created: "2026-07-31"
last_updated: "2026-10-17"
---

# Implementation Profile: Pipeline Tooling

> **UPSTREAM ONLY.** This profile governs the pipeline repository's own tooling --
> the native Rust workspace crates and Markdown governance documents that generate
> and validate specifications. It is **not** an application platform profile.
>
> It MUST NOT be copied into downstream projects. See *Upstream-Only Scoping* below.
> Read alongside `.pipeline/constitution.md` (functional layer).

## Platform & Stack

- **Target:** this repository's own tooling: `crates/` workspace (`crates/deap-core`,
  `crates/verify-baseline`, `crates/reconcile-backlog`, `crates/compile-sysml`,
  `crates/ingest-sysml`, `crates/assemble-conops`), plus Markdown governance under
  `rules/`, `skills/`, and `.pipeline/`.
- **Rust Toolchain:** Native Rust 2021 edition, managed via Cargo workspace.
- **Release Binaries:**
  - `./target/release/verify-baseline`: baseline verifier (all 35 checks).
  - `./target/release/reconcile-backlog`: backlog reconciler and tracker synchronizer.
  - `./target/release/compile-sysml`: SysML v2 AST compiler, syncer, and STPA suite.
  - `./target/release/ingest-sysml`: SysML v2 source AST ingest tool.
  - `./target/release/assemble-conops`: Concept of Operations document assembler.
- **Retired Tooling Decommission:** Legacy Python and shell tooling is permanently
  retired and forbidden under Check 32 (Zero Shell Scripts) and Check 33 (Zero Python
  Scripts).
- **Forbidden:** Network egress inside any blocking validation gate.
- **Zero Regex Invariant:** Regular expression engines are strictly forbidden across
  all crates under Check 34. All parsing uses deterministic string and token scanners.

## Coding Standards

- Workspace crates reside under `crates/` and follow canonical Rust API guidelines.
- Common metamodel and filesystem utilities live in `crates/deap-core`.
- High-integrity docstrings: Production modules and functions implement the 7-section
  safety docstring standard (Safety Intent, Invariants, Preconditions, Postconditions,
  Algorithmic Complexity Bounds, Failure Modes, Realization & Traceability).
- Static string slice arrays and hollow validation stubs are strictly forbidden.

## Testing Mandates

- **Workspace Test Suite Gate:** `cargo test --workspace` MUST pass with zero failures
  before any work package or issue is resolved.
- **TDD Mandate:** Every fix or feature follows the RED-GREEN-REFACTOR cycle, shipping
  a failing Rust unit/mutation test demonstrated RED before implementation.
- **Mutation Probes:** Negative mutation probes must verify that validation gates fail
  closed on non-compliant inputs.
- **Baseline Conformance:** `./target/release/verify-baseline . --no-domain` MUST report
  Success across all 35 baseline checks.

## Validation Gates

- Blocking gates MUST be **offline and dependency-free**.
- Third-party rendering APIs and network dependencies are strictly forbidden in blocking
  quality gates.
- Baseline verification and AST compilation run deterministically from local files.

## Build & Deployment

- Build release binaries:
  ```bash
  cargo build --release --workspace
  ```
- Run the full workspace test suite:
  ```bash
  cargo test --workspace
  ```
- Run the baseline verifier:
  ```bash
  ./target/release/verify-baseline . --no-domain
  ```
- Commit format per `.pipeline/constitution.md` § *Commit Format*:
  `fix:`, `feat:`, `test:`, `refactor:`, `chore:`.
- Neutral issue citations `(refs #<id>)` or `(#<id>)` are strictly mandatory;
  auto-closing keywords are forbidden by Check 35.

## Security & Ops

- No credentials, tokens, or absolute workstation paths in committed files (Check 12).
- Zero Unicode em dashes (`\u2014`) across all files (use ASCII `--` or `-`).

## Issue Lifecycle

Per `.pipeline/constitution.md` § *CMMI Level 3 & Scrum Issue Lifecycle Rules*:

- An agent may take an issue as far as **`Fixed / Resolved`** -- development complete,
  workspace tests green, baseline checks pass, and evidence posted on the issue.
- **`Closed` is unreachable by an agent.** It requires explicit Product Owner validation.
  Agents MUST NOT close issues.
- The issue state is tracked via the `status:fixed-resolved` label.

## Upstream-Only Scoping

This profile must never reach a downstream project. The leak vector is concrete:
README *Direct Copy Installation* runs `cp -RP ./.tmp-pipeline/.pipeline ./`, a
wholesale recursive copy, and the GitHub template route copies the entire tree.

Four layers enforce containment:

1. **Location.** This file lives in `.pipeline/upstream/`, never
   `.pipeline/profiles/`. The documented discovery command is
   `ls .pipeline/profiles/`, so a downstream agent enumerating platforms never
   sees it.
2. **Install.** The README Direct Copy block deletes `./.pipeline/upstream`
   immediately after copying `.pipeline`.
3. **Archive.** `.gitattributes` marks `.pipeline/upstream/ export-ignore`,
   covering `git archive` and release tarballs.
4. **Enforcement.** Check 16 in `crates/verify-baseline` asserts clean landing zones,
   and Check 29 asserts constitution and profile containment.

Anything else that is upstream-only belongs in `.pipeline/upstream/` for the
same reason.
