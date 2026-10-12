## 1. Context and References

<!-- test-target: crates/reconcile-backlog/src/frontmatter.rs -->

- **File**: `crates/reconcile-backlog/src/frontmatter.rs:18-120`
- **Pillar**: Tooling & Traceability Integrity
- **Symptom**: `reconcile-backlog` fails to parse native CommonMark 2-column metadata tables (`| Attribute | Specification Detail |`) at lines 1-10 of specification documents, discovering 0 specifications and preventing projection of specification bodies, titles, and checklist items to GitHub issues.
- **Test-Target**: `crates/reconcile-backlog/src/frontmatter.rs`

## 2. Root Cause Analysis (5 Whys)

1. **Why did `reconcile-backlog --dry-run` and `--sync-bodies` discover 0 specifications across the repository?** Because the parser in `frontmatter.rs` rejected all 94 specification files in `docs/` as lacking valid metadata.
2. **Why were all 94 specification files rejected as lacking valid metadata?** Because `frontmatter.rs` was implemented to expect legacy YAML frontmatter delimited by `---` fences at lines 1-10.
3. **Why did specification files lack `---` YAML frontmatter?** Because the repository governance (`.pipeline/constitution.md:93-96`, `rules/specification-metadata-integrity.md`) mandates that all specification files use native CommonMark 2-column metadata tables at lines 1-10 (`| Attribute | Specification Detail |`) instead of YAML frontmatter.
4. **Why did `frontmatter.rs` not parse CommonMark metadata tables?** Because while `crates/compile-sysml/src/sync/reverse.rs` had been updated with table-parsing logic, `crates/reconcile-backlog/src/frontmatter.rs` was never updated to mirror this parser, creating a structural parser bifurcation between compilation and backlog reconciliation tooling.
5. **Why was issue ID extraction also affected?** Because `update_frontmatter_issue_id` relied on regex substitution expecting `issue_id:` YAML keys, injecting invalid syntax and corrupting CommonMark table formatting.

## 3. Correctness Analysis

In `crates/reconcile-backlog/src/frontmatter.rs`, the parsing logic strictly checked for YAML opening and closing delimiters:

```rust
if !content.starts_with("---") {
    return None;
}
```

Because all generated specification files in `docs/epics/`, `docs/features/`, `docs/user-stories/`, and `docs/use-cases/` begin with:

```markdown
| Attribute | Specification Detail |
| :--- | :--- |
| **Issue ID** | #101 |
| **Title** | Feature 01: System Vision and Invariants |
```

`reconcile-backlog` failed to extract metadata, skipped every file, and reported:
`Discovered 0 specification files`.

This broke the critical terminal delivery boundary: no specification bodies or checklist items could be synchronized to GitHub issues.

## 4. Invariant Violations

1. **Section 1.8 Unique Backlog Identifiers (.pipeline/constitution.md:93-96)**: All local specification files must include permanent unique identifiers in CommonMark metadata tables at lines 1-10 rather than YAML frontmatter.
2. **Backlog Reconciliation Mandate (AGENTS.md)**: Execution of `./target/release/reconcile-backlog` must synchronize local specifications, checklists, and diagrams with the issue tracker.
3. **REQ-0015 Zero-Regex Scanning**: Issue ID updates and token scanning must use pure lexical/scalar token scanning without regex engines.

## 5. Minimal Upstream Fix

1. Port the native CommonMark 2-column table parser (`| Attribute | Specification Detail |`) from `crates/compile-sysml/src/sync/reverse.rs` into `crates/reconcile-backlog/src/frontmatter.rs`.
2. Extract metadata keys (`Issue ID`, `Title`, `Type`, `Primary Actor`, `Scope`, `Level`, `Generation Mode`, `Specification Source`) using zero-regex scalar string scanning.
3. Update `update_frontmatter_issue_id` to update or inject `| **Issue ID** | #<id> |` directly in the lines 1-10 table without legacy YAML fences.

## 6. Verification and Evidence

- `cargo build --release --workspace` succeeds with 0 warnings/errors.
- `./target/release/reconcile-backlog --dry-run` discovers all 94 specification files in `docs/`.
- `./target/release/reconcile-backlog --sync-bodies` projects 100% of specification bodies to GitHub issues cleanly.
