<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Verification-Before-Completion

<a id="rule-verification-required"></a>
## 20. Rule: Verification Required (`rules/verification-required.md`)

**Enforce on code/model modifications only (never on startup or read-only turns):** Never assert state or completion of code/model modifications without direct, empirical verification.

### Hard Constraints
- **Pre-Commit Quality Gates (3-Tier Path-Scoped Incremental Build & Test Gate)**: Never run `cargo build`, `cargo test`, or `verify-baseline` on session initialization or read-only turns. When tracked files are modified, enforce the 3-tier path-scoped gate prior to commit:
  - **Tier 1 (Harness / Governance / Docs Markdown Only -- `AGENTS.md`, `rules/*.md`, `skills/**`, `.pipeline/*`, `README.md`)**: Run **zero** `cargo build` or `cargo test` commands; run `./target/release/verify-baseline . --no-domain` only when validating governed Markdown changes prior to commit.
  - **Tier 2 (Single-Crate Internal or `#[cfg(test)]` Change -- `crates/<crate>/**` without cross-crate `pub` signature breakage)**: Run targeted `cargo test -p <crate> <test_filter>` during TDD RED/GREEN and `cargo test -p <crate>` before commit (rebuild `cargo build --release -p <bin>` only when a CLI binary's runtime code changed), plus `./target/release/verify-baseline . --no-domain`.
  - **Tier 3 (Cross-Crate Public API Signature Change in `deap-core`/`compile-sysml` or `Cargo.toml`/`Cargo.lock` Change)**: Run `cargo test -p <modified-crate>` + `cargo check --workspace --all-targets` during iteration, and `cargo test --workspace` + `./target/release/verify-baseline . --no-domain` before commit.
- **Multi-Provider Remote Sync Verification (GitHub & GitLab)**: Commit all verified changes and push all commits to the configured remote tracking branch (`origin/<branch>` on GitHub or GitLab), verifying `git diff origin/<branch>` is empty.
- **Forensic Verification Evidence**: Document raw terminal outputs, test results, commit hashes, and issue transitions in handoff reports and issue comments.
