<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Verification-Before-Completion

<a id="rule-verification-required"></a>
## 20. Rule: Verification Required (`rules/verification-required.md`)

**Enforce on code/model modifications only (never on startup or read-only turns):** Never assert state or completion of code/model modifications without direct, empirical verification.

### Hard Constraints
- **Pre-Commit Quality Gates (Code/Model Edits Only)**: Only when tracked source or model files were modified in the current turn, run `cargo test --workspace` and `./target/release/verify-baseline . --no-domain` once before committing (never on session initialization or read-only turns).
- **Remote Sync Verification**: Push all commits to the remote tracking branch and verify `git diff origin/<branch>` is empty.
- **Forensic Verification Evidence**: Document raw terminal outputs, test results, commit hashes, and issue transitions in handoff reports and issue comments.
