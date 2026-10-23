---
name: adversarial-code-auditor
version: "4.0"
description: "Pre-emptive adversarial audit against five correctness risk pillars."
compatibility: "Requires verify-baseline native Rust binary, gh CLI, and git."
metadata:
  title: "Adversarial Code Auditor"
  category: auditing
  risk: low
---

# Adversarial Code Auditor

## 1. Reference

### 1.1 Architecture
Subagent receives: `FILE_PATH`, `PILLAR`, `MODE`, `REPO`. Subagent reads this file, audits the target file, writes a 7-section dossier to `.pipeline/defects/<NAME>.md`, and invokes `./target/release/verify-baseline check-defect` to mechanically validate all 12 checks, persist the local artifact, publish/edit the GitHub SSOT issue, and verify closed-loop payload parity. Coordinator collects returned URLs and never touches body text.

### 1.2 When to Use
5+ open bugs in related files form a cluster, or user provides target paths. Single runtime bugs -- `debug-protocol`. Spec gaps -- `spec-implementation-auditor`.

### 1.3 Pillars
| Pillar | Focus |
| --- | --- |
| Memory Safety | UAF, double-free, dangling pointers, buffer overflow, signed/unsigned wrap, C++ exception across FFI, NativeFinalizer, mutex/pointer lifetime |
| Resource Lifecycle | Missing dispose(), cache eviction, sync I/O on UI thread, GC churn, socket leaks, HTTP timeouts, GPU overdraw |
| Concurrency | ChangeNotifier post-disposal, async races, state mutation in build(), re-entrant async, missing _disposed, TOCTOU |
| Test Integrity | FFI/DB-dependent tests, sleep loops, bare assert(), missing testWidgets, duplicated fakes, flaky assertions |
| Semantic Traceability | Test assertions mapped to defect invariants from issue body. Tests that pass without exercising the reported symptom or violated invariant. |

### 1.4 Severity
| Severity | Tracker Label | Rule |
| --- | --- | --- |
| Critical | `bug` | Crash, corruption, or resource leak from code paths reachable by current callers. |
| Important | `bug` | Wrong behavior, degradation under load, correctness risk in edge cases. |
| Suggestion | `enhancement` | Forward-looking risk, missing guard for future code, test gap, dead code. NOT a current bug (see issue #287). |
| Nitpick | `enhancement` | Style, naming. No correctness impact. |

Constraints: "No validation" is false if any guard exists. Read code before claiming absence. Stubs that cannot throw have no exception risk.

### 1.5 UML
Critical and Important findings require a fenced Mermaid diagram (```` ```mermaid ````) in Section 4 (`rules/platform-independence.md` -- no ASCII art, no semicolons in `Note` or message text, no curly braces in `classDiagram` member lines, quoted angle brackets):
* UAF, double-free, dangling pointer, exception across FFI, missing dispose, ChangeNotifier post-disposal, TOCTOU, async race: `sequenceDiagram`
* Buffer overflow, signed wrap, FFI-dependent test, missing mock: `classDiagram`
* Cache eviction, LRU violation: `stateDiagram-v2`
* Semantic Traceability, unverified invariant-to-assertion mapping: `sequenceDiagram` or `classDiagram`

### 1.6 Governance & Invariants
- **Commit Message Non-Closure Invariant**: Strictly forbid auto-closing keywords (`fix`, `fixes`, `fixed`, `close`, `closes`, `closed`, `resolve`, `resolves`, `resolved` preceding `#<id>`). Use neutral citations `(#<id>)` or `(refs #<id>)` exclusively (`.pipeline/constitution.md:266`).
- **Zero Regex Invariant (Check 34)**: Proposed corrections in Section 6 (` ```rust ` blocks) MUST NOT use the `regex` crate (`regex::`, `Regex::new`, `RegexSet`, `use regex`). Propose deterministic linear slice, char, or token scanning exclusively.
- **Domain Cleanliness Invariant (Check 19)**: In upstream compiler crates (`crates/*`), proposed corrections MUST remain completely domain-agnostic with zero hardcoded customer domain tokens. All entity IDs, labels, attributes, and ports must be dynamically extracted from schema and topology contracts at runtime.

## 2. Output Format

Every finding MUST match this skeleton character-for-character in section headers and field labels, starting on line 1 with `## 1. Context and References` (never prepend a `# ` H1 header):

## 1. Context and References

<!-- test-target: [crates/package/tests/reproducer_tests.rs] -->

- **File**: `[path]:[line-line]`
- **Pillar**: [Memory Safety | Resource Lifecycle | Concurrency | Test Integrity | Semantic Traceability]
- **Symptom**: [description]
- **Test-Target**: `[crates/package/tests/reproducer_tests.rs]`

## 2. Root Cause Analysis (5 Whys)

1. **Why [symptom]?** Because [reason].
2. **Why [reason]?** Because [deeper].
3. **Why [deeper]?** Because [deeper still].
4. **Why [deeper still]?** Because [design cause].
5. **Why [design cause]?** Because [root cause].

## 3. Correctness Analysis

[Trace data flow from trigger to failure. Name the invariant violated. Cite file:line references.]

## 4. UML Diagrams

[MANDATORY ```mermaid block for Critical/Important. For Suggestion/Nitpick: "N/A -- [severity] severity."]

## 5. Affected Callers / Downstream Impact

[caller] -- [how affected]

## 6. Proposed Correction

<!-- MUST NOT use regex (Check 34: deterministic linear slice/char scanning, no regex crate). MUST be domain-agnostic for upstream crates (Check 19). -->

```[language]
[code]
```

## 7. Relationship to Existing Issues

Discovered in audit -- new finding.

## Audit Source

Adversarial [Pillar] Audit
SEVERITY: [Critical | Important | Suggestion | Nitpick]
FILE_LOCATION: [path]:[line-line]

## 3. Protocol

### Step A -- Read
1. Read `skills/adversarial-code-auditor/SKILL.md`, `[FILE_PATH]`, and `.pipeline/constitution.md` (plus `.pipeline/profiles/flutter.md` for Dart files).

### Step B -- Audit
1. Scan every line of `[FILE_PATH]` through `[PILLAR]`. Verify exact `file:line` range, caller reachability, existing guards, Severity (Section 1.4), and UML type (Section 1.5).

### Step C -- Write
1. Write `.pipeline/defects/<NAME>.md` starting directly on line 1 with `## 1. Context and References` (never add a leading `# ` H1 inside the body).
2. Populate all 7 sections and `## Audit Source`, ensuring `<!-- test-target: <path> -->` and `- **Test-Target**: \`<path>\`` cite the identical executable Rust test target path under `crates/` (in upstream workspaces, MUST belong to one of the 6 upstream compiler crates).
3. In Section 6 (`Proposed Correction`), strictly mandate Check 34 (Zero Regex Invariant: deterministic linear slice/char scanning, no regex crate) and Check 19 (Zero domain tokens in upstream crates; domain-agnostic).

### Step D & E -- Verify, Persist & Publish (Closed-Loop Native Rust Gate)
1. ALWAYS execute the native Rust verification and publishing gate autonomously inside the standard sandbox (running `gh` inside the standard sandbox executes cleanly; never set sandbox bypass options, and never invoke `security add-trusted-cert` or macOS Keychain commands):
   ```bash
   ./target/release/verify-baseline check-defect .pipeline/defects/<NAME>.md --repo "[REPO]" --title "[AUDIT] [file.ext]: <description>" --publish
   ```
   (In `bug-based` mode when updating an existing tracker issue, pass `--issue <ID>` to atomically overwrite the issue title and body via `gh issue edit`.)
2. `verify-baseline check-defect` publishes to the GitHub SSOT tracker by default whenever `--dry-run` is not passed, automatically resolves `<owner>/<repo>` from `git remote get-url origin` when `--repo` is omitted, and exits non-zero (`Err(1)`) on any `gh` CLI creation/edit/view error.
3. The Rust binary mechanically enforces all 12 schema/Mermaid checks, rejects ephemeral `/tmp/` paths, persists `.pipeline/defects/<NAME>.md`, publishes/updates the GitHub SSOT issue, and runs the Closed-Loop Payload Verification Gate (`gh issue view --json title,body,url`) in a single atomic invocation.
4. The agent MUST halt immediately and report a hard failure if `verify-baseline check-defect` exits non-zero or does not emit a verified GitHub issue URL/ID (`[CLOSED-LOOP VERIFIED]`).

## 5. Coordinator Dispatch

Execute adversarial-code-auditor skill.
Read skills/adversarial-code-auditor/SKILL.md in full. Follow the Protocol (Section 3) exactly -- Read, Audit, Write, Verify, File.
FILE_PATH: [FILE_PATH] PILLAR: [PILLAR] MODE: [MODE] REPO: [REPO]
Return issue URLs with severities. PROCEED
