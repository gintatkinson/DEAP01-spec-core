//! Integration verification test suite for `verify_baseline::blueprint_gate` Checks 1 through 12,
//! `REQ-0604`..`REQ-0606`, and Check 37 blueprint grounding.
//!
//! Safety & Traceability: Realises: [REQ-BLUEPRINT-GATE-TESTS-1-5], [DO-330-TQL-4-BLUEPRINT-TESTS]
//! Contract: Validates positive acceptance fixtures and fail-closed negative mutations across all blueprint, sync, and static-auditor gates.
//! Bounds & Faults: Executes deterministically over bounded in-memory and RAII temporary directory fixtures with zero panics or unsafe code.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use verify_baseline::blueprint_gate::{
    check_10_compile_and_persist_sysml, check_11_source_references, check_12_publish_github_ssot,
    check_1_zero_ephemeral_path, check_2_metadata_table, check_3_sections,
    check_4_derivation_table, check_5_components, check_6_mermaid_syntax, check_7_latex_katex,
    check_8_bdd_and_test_target, check_9_rust_code_blocks, run_blueprint_cli, verify_blueprint,
    verify_closed_loop_github_payload, BlueprintCliOptions, BlueprintMetadata, BlueprintReport,
};

const MAX_MODULE_WALK_DEPTH: usize = 16;

/// Resolves a target `.rs` file or its decomposed `<stem>/` module directory into sorted `.rs` paths.
/// Realises: [REQ-BLUEPRINT-MODULE-RESOLVER]
fn resolve_module_rs_files(path: &Path) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.to_path_buf()];
    }
    let dir_path = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.with_extension("")
    };
    if !dir_path.is_dir() {
        return Vec::new();
    }
    let mut collected = BTreeSet::new();
    let mut stack = vec![(dir_path, 0usize)];
    while let Some((current_dir, depth)) = stack.pop() {
        if depth > MAX_MODULE_WALK_DEPTH {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&current_dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let entry_path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                let next_depth = depth.saturating_add(1);
                if next_depth <= MAX_MODULE_WALK_DEPTH {
                    stack.push((entry_path, next_depth));
                }
            } else if file_type.is_file()
                && entry_path.extension().and_then(|ext| ext.to_str()) == Some("rs")
            {
                collected.insert(entry_path);
            }
        }
    }
    collected.into_iter().collect()
}

/// Returns a canonical valid 7-section blueprint markdown payload satisfying Checks 2 through 9.
/// Realises: [REQ-BLUEPRINT-TEST-FIXTURE]
fn valid_blueprint_fixture() -> &'static str {
    "\
| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-01-TELEMETRY-GATE` |
| **Requirement ID** | `REQ-BP-001` |
| **Title** | Deterministic Telemetry Gate Blueprint |
| **Status** | Approved |
| **Target Crates** | `verify-baseline`, `compile-sysml` |
| **Tracked Issues** | `#501` |
| **Generation Mode** | `subagent` |
| **Specification Source** | `docs/features/feat-01.md` |

## 1. Executive Summary & Derivation-Annotation Contract

| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade (Directly Evidenced / Analytically Derived / Declared Assumption) | Caveat / Downstream Impact |
| :--- | :--- | :--- | :--- | :--- |
| Frame rate bounded by $\\|x\\| \\le 100$ | `schema/system.sysml:14` | DO-178C Section 6.3.1 | **Directly Evidenced** | Requires deterministic buffer sizing |
| Worst-case jitter stays under 2 ms | `docs/features/feat-01.md:42` | MIL-STD-882E Task 202 | `Analytically Derived` | Validated by discrete step solver |
| Channel failover budget holds | `#501` | ISO 26262 Part 6 | Declared Assumption | Must be confirmed on target hardware |

## 2. Structural Component Metamodel

### Component: TelemetryRouter

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `maxRetries` | `Integer` | `3` | `count` | Maximum packet retransmission attempts |
| `isRedundant` | `Boolean` | `true` | `dimensionless` | Enables dual-channel failover routing |
| `samplePeriod` | `Real` | `0.004` | `s` | Nominal discrete sampling period |
| `channelTag` | `String` | `\"PRI\"` | `dimensionless` | Primary channel identifier label |
| `framePolicy` | `RoutingPolicy` | `StrictOrder` | `dimensionless` | Deterministic frame dispatch policy |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `rawTelemetryIn` | `in` | `TelemetryFrame` | `ARINC429` | Inbound sensor frame stream |
| `validatedOut` | **out** | `ValidatedFrame` | `DeterministicBus` | Verified outbound frame stream |
| `diagBus` | `inout` | `DiagnosticPacket` | `MaintenanceBus` | Bidirectional health diagnostics |

### Component: WatchdogMonitor

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `timeoutTicks` | Integer | 250 | ticks | Watchdog expiration threshold |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `heartbeatIn` | in | HeartbeatSignal | InternalClock | Periodic liveness pulse |

## 3. UML & Behavioral Topology

```mermaid
classDiagram
    class TelemetryRouter {
        +Integer maxRetries
        +Boolean isRedundant
        +routeFrame(TelemetryFrame frame) ValidatedFrame
    }
    class WatchdogMonitor {
        +Integer timeoutTicks
        +checkPulse(HeartbeatSignal sig) Boolean
    }
    TelemetryRouter --> WatchdogMonitor : monitors
```

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Active : startRouting
    Active --> FaultLocked : watchdogExpired
```

## 4. Formal Mathematical & Safety Constraints

The worst-case latency bound for **REQ-BP-001** under hazard **H-1** and safety constraint **SC-01** satisfies:

$$
\\begin{aligned}
T_{bound} &= N \\cdot \\Delta t + \\tau_{sync} \\\\
\\Delta t &\\le 0.004
\\end{aligned}
$$

## 5. Acceptance Criteria & Verification Matrix

<!-- test-target: crates/verify-baseline/tests/blueprint_gate_tests.rs -->
- **Test-Target**: `crates/verify-baseline/tests/blueprint_gate_tests.rs`

### AC-01: Nominal Frame Routing
- **Given**: A valid inbound telemetry frame on `rawTelemetryIn`
- **When**: `TelemetryRouter` processes the frame within the nominal period
- **Then**: A verified `ValidatedFrame` is emitted on `validatedOut` within 1 tick

### AC-02: Redundant Channel Failover
- **Given**: `isRedundant` is enabled and the primary channel reports link loss
- **When**: The router evaluates channel health
- **Then**: Frame dispatch switches deterministically to the secondary channel

### AC-03: Negative Fail-Closed Rejection on Corrupted Frame
- **Given**: An invalid telemetry frame with a corrupted header payload
- **When**: `TelemetryRouter` executes frame validation
- **Then**: The router must reject the frame fail-closed and return a non-zero error code

### AC-04: Watchdog Expiration Lockout
- **Given**: `WatchdogMonitor` has elapsed `timeoutTicks` without a `heartbeatIn` pulse
- **When**: The discrete step timer advances by 1 tick
- **Then**: The monitor transitions to `FaultLocked` and halts outbound emission

### AC-05: Diagnostic Bus Status Reporting
- **Given**: A completed routing cycle with recorded retry counters
- **When**: `diagBus` is polled by the maintenance scheduler
- **Then**: A `DiagnosticPacket` reflecting the exact retry count is returned

## 6. Proposed Deterministic Implementation Contract

```rust
pub struct RouterConfig {
    pub max_retries: u32,
    pub is_redundant: bool,
}

pub fn validate_retry_count(retries: u32, limit: u32) -> Result<u32, &'static str> {
    if retries <= limit {
        Ok(retries)
    } else {
        Err(\"retry limit exceeded\")
    }
}
```

## 7. Source References

- `crates/verify-baseline/src/blueprint_gate.rs:1`
- [Blueprint Gate Tests](crates/verify-baseline/tests/blueprint_gate_tests.rs#L1-L50)
- `Cargo.toml`
- External reference: https://www.rust-lang.org and tracker issue `#501`
"
}

/// Verifies that `check_1_zero_ephemeral_path` accepts valid relative and absolute paths under `<workspace_root>/docs/designs/blueprint-*.md`.
/// Realises: [REQ-BLUEPRINT-CHECK-1-PASS]
#[test]
fn test_check_1_accepts_valid_blueprint_paths() {
    let ws = Path::new("/workspace/deap-project");
    let rel = Path::new("docs/designs/blueprint-01-telemetry.md");
    let res_rel = check_1_zero_ephemeral_path(ws, rel);
    assert_eq!(
        res_rel,
        Ok(PathBuf::from(
            "/workspace/deap-project/docs/designs/blueprint-01-telemetry.md"
        ))
    );

    let abs = Path::new("/workspace/deap-project/docs/designs/blueprint-02-control.md");
    let res_abs = check_1_zero_ephemeral_path(ws, abs);
    assert_eq!(
        res_abs,
        Ok(PathBuf::from(
            "/workspace/deap-project/docs/designs/blueprint-02-control.md"
        ))
    );
}

/// Verifies that `check_1_zero_ephemeral_path` fails closed on ephemeral directories and non-canonical locations.
/// Realises: [REQ-BLUEPRINT-CHECK-1-FAIL-CLOSED]
#[test]
fn test_check_1_rejects_ephemeral_and_invalid_paths() {
    let ws = Path::new("/workspace/deap-project");

    let tmp_path = Path::new("/tmp/blueprint-01-ephemeral.md");
    assert!(check_1_zero_ephemeral_path(ws, tmp_path).is_err());

    let tmp_nested = Path::new("/tmp/docs/designs/blueprint-01-ephemeral.md");
    assert!(check_1_zero_ephemeral_path(Path::new("/tmp"), tmp_nested).is_err());

    let private_tmp = Path::new("/private/tmp/blueprint-01-ephemeral.md");
    assert!(check_1_zero_ephemeral_path(ws, private_tmp).is_err());

    let var_folders = Path::new("/var/folders/ab/cd/T/blueprint-01-ephemeral.md");
    assert!(check_1_zero_ephemeral_path(ws, var_folders).is_err());

    let private_var_folders = Path::new("/private/var/folders/ab/cd/T/blueprint-01-ephemeral.md");
    assert!(check_1_zero_ephemeral_path(ws, private_var_folders).is_err());

    let env_temp =
        std::env::temp_dir().join(format!("blueprint-01-temp-{}.md", std::process::id()));
    assert!(check_1_zero_ephemeral_path(ws, &env_temp).is_err());

    let wrong_dir = Path::new("docs/epics/blueprint-01-wrong-dir.md");
    assert!(check_1_zero_ephemeral_path(ws, wrong_dir).is_err());

    let wrong_prefix = Path::new("docs/designs/design-01-wrong-prefix.md");
    assert!(check_1_zero_ephemeral_path(ws, wrong_prefix).is_err());

    let wrong_ext = Path::new("docs/designs/blueprint-01-wrong-ext.txt");
    assert!(check_1_zero_ephemeral_path(ws, wrong_ext).is_err());

    let empty_slug = Path::new("docs/designs/blueprint-.md");
    assert!(check_1_zero_ephemeral_path(ws, empty_slug).is_err());

    let traversal = Path::new("docs/designs/../../tmp/blueprint-01-traversal.md");
    assert!(check_1_zero_ephemeral_path(ws, traversal).is_err());

    let outside_ws = Path::new("/other_workspace/docs/designs/blueprint-01-outside.md");
    assert!(check_1_zero_ephemeral_path(ws, outside_ws).is_err());
}

/// Verifies `check_2_metadata_table` extracts all 8 mandatory attributes and rejects invalid keys, `REQ-` IDs, or generation modes.
/// Realises: [REQ-BLUEPRINT-CHECK-2-TESTS]
#[test]
fn test_check_2_metadata_table_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let res = check_2_metadata_table(valid);
    assert!(res.is_ok(), "Expected Check 2 pass, got: {res:?}");
    let meta = res.unwrap_or_default();
    assert_eq!(
        meta,
        BlueprintMetadata {
            blueprint_id: "BP-01-TELEMETRY-GATE".to_string(),
            requirement_id: "REQ-BP-001".to_string(),
            title: "Deterministic Telemetry Gate Blueprint".to_string(),
            status: "Approved".to_string(),
            target_crates: "`verify-baseline`, `compile-sysml`".to_string(),
            tracked_issues: "#501".to_string(),
            generation_mode: "subagent".to_string(),
            specification_source: "docs/features/feat-01.md".to_string(),
        }
    );

    // Mutation 1: Wrong Generation Mode ("manual" instead of "subagent")
    let bad_mode = valid.replace(
        "| **Generation Mode** | `subagent` |",
        "| **Generation Mode** | `manual` |",
    );
    assert!(check_2_metadata_table(&bad_mode).is_err());

    // Mutation 2: Requirement ID missing "REQ-"
    let bad_req = valid.replace(
        "| **Requirement ID** | `REQ-BP-001` |",
        "| **Requirement ID** | `BP-001` |",
    );
    assert!(check_2_metadata_table(&bad_req).is_err());

    // Mutation 3: Missing Specification Source row
    let missing_source = valid.replace(
        "| **Specification Source** | `docs/features/feat-01.md` |\n",
        "",
    );
    assert!(check_2_metadata_table(&missing_source).is_err());

    // Mutation 4: Empty Title value
    let empty_title = valid.replace(
        "| **Title** | Deterministic Telemetry Gate Blueprint |",
        "| **Title** |   |",
    );
    assert!(check_2_metadata_table(&empty_title).is_err());

    // Mutation 5: Table pushed past line 12
    let pushed_down = format!("\n\n\n\n\n\n{valid}");
    assert!(check_2_metadata_table(&pushed_down).is_err());
}

/// Verifies `check_3_sections` validates the exact 7-section topology and rejects missing, duplicated, or out-of-order headers.
/// Realises: [REQ-BLUEPRINT-CHECK-3-TESTS]
#[test]
fn test_check_3_sections_order_and_completeness() {
    let valid = valid_blueprint_fixture();
    let res = check_3_sections(valid);
    assert!(res.is_ok(), "Expected Check 3 pass, got: {res:?}");
    let ranges = res.unwrap_or_default();
    assert_eq!(ranges.len(), 7);

    let mut prev_end = 0usize;
    for (idx, (start, end, header)) in ranges.iter().enumerate() {
        assert!(*start < *end, "Section {idx} must have start < end");
        if idx > 0 {
            assert_eq!(*start, prev_end, "Section {idx} must start at previous end");
        }
        assert!(header.starts_with("## "));
        prev_end = *end;
    }

    // Mutation 1: Missing Section 4
    let missing_sec4 = valid.replace(
        "## 4. Formal Mathematical & Safety Constraints",
        "## 4. Invalid Header Name",
    );
    assert!(check_3_sections(&missing_sec4).is_err());

    // Mutation 2: Out-of-order sections (Section 3 before Section 2)
    let out_of_order = valid
        .replace("## 2. Structural Component Metamodel", "__TEMP_SEC_2__")
        .replace(
            "## 3. UML & Behavioral Topology",
            "## 2. Structural Component Metamodel",
        )
        .replace("__TEMP_SEC_2__", "## 3. UML & Behavioral Topology");
    assert!(check_3_sections(&out_of_order).is_err());

    // Mutation 3: Duplicate Section 5 header
    let duplicate_sec5 = format!("{valid}\n## 5. Acceptance Criteria & Verification Matrix\n");
    assert!(check_3_sections(&duplicate_sec5).is_err());
}

/// Verifies `check_4_derivation_table` parses 5-column derivation rows (including escaped pipes) and rejects invalid grades or empty cells.
/// Realises: [REQ-BLUEPRINT-CHECK-4-TESTS]
#[test]
fn test_check_4_derivation_table_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(valid).unwrap_or_default();
    let (s1_start, s1_end, _) = ranges.first().copied().unwrap_or((0, 0, ""));
    let sec1_lines = lines.get(s1_start..s1_end).unwrap_or(&[]);

    let res = check_4_derivation_table(sec1_lines);
    assert!(res.is_ok(), "Expected Check 4 pass, got: {res:?}");
    let rows = res.unwrap_or_default();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows.first().map(|r| r.grade.as_str()),
        Some("Directly Evidenced")
    );
    assert_eq!(
        rows.get(1).map(|r| r.grade.as_str()),
        Some("Analytically Derived")
    );
    assert_eq!(
        rows.get(2).map(|r| r.grade.as_str()),
        Some("Declared Assumption")
    );
    assert!(rows.first().is_some_and(|r| r.claim.contains("\\|x\\|")));

    // Mutation 1: Invalid Derivation Grade
    let bad_grade = [
        "## 1. Executive Summary & Derivation-Annotation Contract",
        "| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade | Caveat / Downstream Impact |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| Claim 1 | `schema/a.sysml:1` | DO-178C | **Guessed Grade** | Impact 1 |",
    ];
    assert!(check_4_derivation_table(&bad_grade).is_err());

    // Mutation 2: Empty cell / placeholder cell ("TBD" / "-")
    let empty_cell = [
        "## 1. Executive Summary & Derivation-Annotation Contract",
        "| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade | Caveat / Downstream Impact |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| Claim 1 | - | DO-178C | Directly Evidenced | Impact 1 |",
    ];
    assert!(check_4_derivation_table(&empty_cell).is_err());

    // Mutation 3: Zero data rows
    let zero_rows = [
        "## 1. Executive Summary & Derivation-Annotation Contract",
        "| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade | Caveat / Downstream Impact |",
        "| :--- | :--- | :--- | :--- | :--- |",
    ];
    assert!(check_4_derivation_table(&zero_rows).is_err());
}

/// Verifies `check_5_components` parses Section 2 components, properties, and ports and rejects invalid identifiers, types, or directions.
/// Realises: [REQ-BLUEPRINT-CHECK-5-TESTS]
#[test]
fn test_check_5_components_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(valid).unwrap_or_default();
    let (s2_start, s2_end, _) = ranges.get(1).copied().unwrap_or((0, 0, ""));
    let sec2_lines = lines.get(s2_start..s2_end).unwrap_or(&[]);

    let res = check_5_components(sec2_lines);
    assert!(res.is_ok(), "Expected Check 5 pass, got: {res:?}");
    let comps = res.unwrap_or_default();
    assert_eq!(comps.len(), 2);
    assert_eq!(
        comps.first().map(|c| c.name.as_str()),
        Some("TelemetryRouter")
    );
    assert_eq!(comps.first().map(|c| c.properties.len()), Some(5));
    assert_eq!(comps.first().map(|c| c.ports.len()), Some(3));
    assert_eq!(
        comps.get(1).map(|c| c.name.as_str()),
        Some("WatchdogMonitor")
    );

    // Verify public types BlueprintReport and BlueprintCliOptions are constructible
    let report = BlueprintReport {
        path: PathBuf::from("docs/designs/blueprint-01-telemetry.md"),
        components: comps,
        ..BlueprintReport::default()
    };
    assert_eq!(report.components.len(), 2);
    let cli_opts = BlueprintCliOptions {
        path: report.path.clone(),
        dry_run: true,
        ..BlueprintCliOptions::default()
    };
    assert!(cli_opts.dry_run);

    // Mutation 1: Non-PascalCase component identifier (contains underscore)
    let bad_comp_name = [
        "## 2. Structural Component Metamodel",
        "### Component: Telemetry_Router",
        "| Attribute | Type | Default | Unit | Description |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| `maxRetries` | `Integer` | `3` | `count` | Max retries |",
        "| Port | Direction | Type | Protocol | Description |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| `cmdIn` | `in` | `CommandFrame` | `Bus` | Command input |",
    ];
    assert!(check_5_components(&bad_comp_name).is_err());

    // Mutation 2: Invalid property type ("i32" instead of SysML type / PascalCase)
    let bad_prop_type = [
        "## 2. Structural Component Metamodel",
        "### Component: TelemetryRouter",
        "| Attribute | Type | Default | Unit | Description |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| `maxRetries` | `i32` | `3` | `count` | Max retries |",
        "| Port | Direction | Type | Protocol | Description |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| `cmdIn` | `in` | `CommandFrame` | `Bus` | Command input |",
    ];
    assert!(check_5_components(&bad_prop_type).is_err());

    // Mutation 3: Invalid port direction ("bidirectional" instead of in/out/inout)
    let bad_port_dir = [
        "## 2. Structural Component Metamodel",
        "### Component: TelemetryRouter",
        "| Attribute | Type | Default | Unit | Description |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| `maxRetries` | `Integer` | `3` | `count` | Max retries |",
        "| Port | Direction | Type | Protocol | Description |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| `cmdIn` | `bidirectional` | `CommandFrame` | `Bus` | Command input |",
    ];
    assert!(check_5_components(&bad_port_dir).is_err());

    // Mutation 4: Missing Port table
    let missing_port_table = [
        "## 2. Structural Component Metamodel",
        "### Component: TelemetryRouter",
        "| Attribute | Type | Default | Unit | Description |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| `maxRetries` | `Integer` | `3` | `count` | Max retries |",
    ];
    assert!(check_5_components(&missing_port_table).is_err());
}

/// Verifies `check_6_mermaid_syntax` accepts valid Section 3 Mermaid diagrams and rejects invalid headers, unquoted `<>`, `{}` members, or unclosed fences.
/// Realises: [REQ-BLUEPRINT-CHECK-6-TESTS]
#[test]
fn test_check_6_mermaid_syntax_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(valid).unwrap_or_default();
    let (s3_start, s3_end, _) = ranges.get(2).copied().unwrap_or((0, 0, ""));
    let sec3_lines = lines.get(s3_start..s3_end).unwrap_or(&[]);

    let res = check_6_mermaid_syntax(sec3_lines);
    assert_eq!(res, Ok(2));

    // Also verify flowchart TD and sequenceDiagram headers are accepted
    let valid_flow_and_seq = [
        "## 3. UML & Behavioral Topology",
        "```mermaid",
        "%% leading comment is allowed",
        "flowchart TD",
        "    A[\"Start Node\"] --> B[\"End Node\"]",
        "```",
        "```mermaid",
        "sequenceDiagram",
        "    Alice->>Bob: Synchronize",
        "```",
    ];
    assert_eq!(check_6_mermaid_syntax(&valid_flow_and_seq), Ok(2));

    // Mutation 1: Zero mermaid blocks in Section 3
    let no_mermaid = [
        "## 3. UML & Behavioral Topology",
        "No diagrams provided here.",
    ];
    assert!(check_6_mermaid_syntax(&no_mermaid).is_err());

    // Mutation 2: Invalid diagram header ("pie" instead of allowed 4 diagram types)
    let bad_header = [
        "## 3. UML & Behavioral Topology",
        "```mermaid",
        "pie title Invalid Diagram Type",
        "    \"Alpha\" : 40",
        "```",
    ];
    assert!(check_6_mermaid_syntax(&bad_header).is_err());

    // Mutation 3: Unquoted '<' / '>' in diagram line
    let unquoted_angle = [
        "## 3. UML & Behavioral Topology",
        "```mermaid",
        "flowchart TD",
        "    A[Value < 10] --> B[Done]",
        "```",
    ];
    assert!(check_6_mermaid_syntax(&unquoted_angle).is_err());

    // Mutation 4: Curly braces '{}' in classDiagram member line
    let member_braces = [
        "## 3. UML & Behavioral Topology",
        "```mermaid",
        "classDiagram",
        "    class TelemetryRouter {",
        "        +routeFrame() { return true }",
        "    }",
        "```",
    ];
    assert!(check_6_mermaid_syntax(&member_braces).is_err());

    // Mutation 5: Secondary colon ':' in classDiagram member
    let secondary_colon = [
        "## 3. UML & Behavioral Topology",
        "```mermaid",
        "classDiagram",
        "    class TelemetryRouter {",
        "        +routeFrame(frame: TelemetryFrame) : ValidatedFrame",
        "    }",
        "```",
    ];
    assert!(check_6_mermaid_syntax(&secondary_colon).is_err());

    // Mutation 6: Leaking / unclosed mermaid fence
    let leaking_fence = [
        "## 3. UML & Behavioral Topology",
        "```mermaid",
        "stateDiagram-v2",
        "    [*] --> Idle",
    ];
    assert!(check_6_mermaid_syntax(&leaking_fence).is_err());
}

/// Verifies `check_7_latex_katex` accepts isolated `$$` `\begin{aligned}...\end{aligned}` blocks and rejects `\begin{align}`, inline `$$`, or `$REQ-*$` IDs.
/// Realises: [REQ-BLUEPRINT-CHECK-7-TESTS]
#[test]
fn test_check_7_latex_katex_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(valid).unwrap_or_default();
    let (s4_start, s4_end, _) = ranges.get(3).copied().unwrap_or((0, 0, ""));
    let sec4_lines = lines.get(s4_start..s4_end).unwrap_or(&[]);

    let res = check_7_latex_katex(sec4_lines, valid);
    assert_eq!(res, Ok(1));

    // Mutation 1: Section 4 has no $$ \begin{aligned}...\end{aligned} $$ block
    let no_aligned = [
        "## 4. Formal Mathematical & Safety Constraints",
        "Plain prose without display math.",
    ];
    assert!(check_7_latex_katex(&no_aligned, &no_aligned.join("\n")).is_err());

    // Mutation 2: Top-level \begin{align} instead of \begin{aligned}
    let bad_align = [
        "## 4. Formal Mathematical & Safety Constraints",
        "$$",
        "\\begin{align}",
        "x &= 1",
        "\\end{align}",
        "$$",
    ];
    assert!(check_7_latex_katex(&bad_align, &bad_align.join("\n")).is_err());

    // Mutation 3: Non-isolated $$ delimiter on same line as formula
    let non_isolated_dd = [
        "## 4. Formal Mathematical & Safety Constraints",
        "$$ \\begin{aligned} x &= 1 \\end{aligned} $$",
    ];
    assert!(check_7_latex_katex(&non_isolated_dd, &non_isolated_dd.join("\n")).is_err());

    // Mutation 4: Inline $REQ-0601$ math-delimited identifier in document prose
    let inline_req_id = format!("{valid}\nIn prose we forbid $REQ-0601$ and $H-1$ and $SC-01$.\n");
    assert!(check_7_latex_katex(sec4_lines, &inline_req_id).is_err());

    // Mutation 5: Inline $UCA-1$ math-delimited identifier inside Section 4
    let inline_uca = [
        "## 4. Formal Mathematical & Safety Constraints",
        "Constraint for $UCA-1$:",
        "$$",
        "\\begin{aligned}",
        "x &= 1",
        "\\end{aligned}",
        "$$",
    ];
    assert!(check_7_latex_katex(&inline_uca, &inline_uca.join("\n")).is_err());
}

/// Verifies `check_8_bdd_and_test_target` enforces `>= 5` Given/When/Then `AC-XX` scenarios, `>= 1` negative scenario, and grounded `test-target` declarations.
/// Realises: [REQ-BLUEPRINT-CHECK-8-TESTS]
#[test]
fn test_check_8_bdd_and_test_target_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(valid).unwrap_or_default();
    let (s5_start, s5_end, _) = ranges.get(4).copied().unwrap_or((0, 0, ""));
    let sec5_lines = lines.get(s5_start..s5_end).unwrap_or(&[]);

    let res = check_8_bdd_and_test_target(sec5_lines);
    assert_eq!(
        res,
        Ok((
            5,
            "crates/verify-baseline/tests/blueprint_gate_tests.rs".to_string()
        ))
    );

    // Mutation 1: Fewer than 5 AC scenarios (only 4)
    let only_four_acs = [
        "## 5. Acceptance Criteria & Verification Matrix",
        "<!-- test-target: crates/deap-core/tests/core_test.rs -->",
        "- **Test-Target**: `crates/deap-core/tests/core_test.rs`",
        "### AC-01: One",
        "- **Given**: A",
        "- **When**: B",
        "- **Then**: C",
        "### AC-02: Two",
        "- **Given**: A",
        "- **When**: B",
        "- **Then**: C",
        "### AC-03: Three (Negative)",
        "- **Given**: Invalid input",
        "- **When**: Parsed",
        "- **Then**: Reject fail-closed",
        "### AC-04: Four",
        "- **Given**: A",
        "- **When**: B",
        "- **Then**: C",
    ];
    assert!(check_8_bdd_and_test_target(&only_four_acs).is_err());

    // Mutation 2: AC missing "Then" clause
    let missing_then_str = valid.replace(
        "- **Then**: A `DiagnosticPacket` reflecting the exact retry count is returned",
        "- **Result**: Packet returned",
    );
    let m2_lines: Vec<&str> = missing_then_str.lines().collect();
    let m2_sec5 = m2_lines.get(s5_start..s5_end).unwrap_or(&[]);
    assert!(check_8_bdd_and_test_target(m2_sec5).is_err());

    // Mutation 3: No negative / fail-closed scenario across the 5 ACs
    let no_negative = [
        "## 5. Acceptance Criteria & Verification Matrix",
        "<!-- test-target: crates/deap-core/tests/core_test.rs -->",
        "- **Test-Target**: `crates/deap-core/tests/core_test.rs`",
        "### AC-01: Nominal A",
        "- **Given**: State A\n- **When**: Event A\n- **Then**: Output A",
        "### AC-02: Nominal B",
        "- **Given**: State B\n- **When**: Event B\n- **Then**: Output B",
        "### AC-03: Nominal C",
        "- **Given**: State C\n- **When**: Event C\n- **Then**: Output C",
        "### AC-04: Nominal D",
        "- **Given**: State D\n- **When**: Event D\n- **Then**: Output D",
        "### AC-05: Nominal E",
        "- **Given**: State E\n- **When**: Event E\n- **Then**: Output E",
    ];
    let no_neg_joined = no_negative.join("\n");
    let no_neg_lines: Vec<&str> = no_neg_joined.lines().collect();
    assert!(check_8_bdd_and_test_target(&no_neg_lines).is_err());

    // Mutation 4: Mismatched HTML comment vs bullet Test-Target path
    let mismatched_target = valid.replace(
        "- **Test-Target**: `crates/verify-baseline/tests/blueprint_gate_tests.rs`",
        "- **Test-Target**: `crates/deap-core/tests/other_tests.rs`",
    );
    let m4_lines: Vec<&str> = mismatched_target.lines().collect();
    let m4_sec5 = m4_lines.get(s5_start..s5_end).unwrap_or(&[]);
    assert!(check_8_bdd_and_test_target(m4_sec5).is_err());

    // Mutation 5: Test-Target outside the 6 allowed upstream crates
    let invalid_crate_target = valid
        .replace(
            "<!-- test-target: crates/verify-baseline/tests/blueprint_gate_tests.rs -->",
            "<!-- test-target: crates/unknown-crate/tests/foo.rs -->",
        )
        .replace(
            "- **Test-Target**: `crates/verify-baseline/tests/blueprint_gate_tests.rs`",
            "- **Test-Target**: `crates/unknown-crate/tests/foo.rs`",
        );
    let m5_lines: Vec<&str> = invalid_crate_target.lines().collect();
    let m5_sec5 = m5_lines.get(s5_start..s5_end).unwrap_or(&[]);
    assert!(check_8_bdd_and_test_target(m5_sec5).is_err());
}

/// Verifies `check_9_rust_code_blocks` accepts clean Section 6 Rust blocks and rejects missing blocks, regex usage, panicking/indexing calls, or domain tokens.
/// Realises: [REQ-BLUEPRINT-CHECK-9-TESTS]
#[test]
fn test_check_9_rust_code_blocks_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(valid).unwrap_or_default();
    let (s6_start, s6_end, _) = ranges.get(5).copied().unwrap_or((0, 0, ""));
    let sec6_lines = lines.get(s6_start..s6_end).unwrap_or(&[]);

    let res = check_9_rust_code_blocks(sec6_lines);
    assert_eq!(res, Ok(1));

    // Mutation 1: No ```rust block in Section 6
    let no_rust = [
        "## 6. Proposed Deterministic Implementation Contract",
        "No code block here.",
    ];
    assert!(check_9_rust_code_blocks(&no_rust).is_err());

    // Mutation 2: Check 34 Zero Regex violation
    let block_with_regex =
        "## 6. Proposed Deterministic Implementation Contract\n```rust\npub fn bad() {\n    let _r = Regex::new(\"abc\");\n}\n```";
    let m2_lines: Vec<&str> = block_with_regex.lines().collect();
    assert!(check_9_rust_code_blocks(&m2_lines).is_err());

    // Mutation 3: Clippy-forbid .unwrap() violation
    let block_with_unwrap =
        "## 6. Proposed Deterministic Implementation Contract\n```rust\npub fn bad(opt: Option<u32>) -> u32 {\n    opt.unwrap()\n}\n```";
    let m3_lines: Vec<&str> = block_with_unwrap.lines().collect();
    assert!(check_9_rust_code_blocks(&m3_lines).is_err());

    // Mutation 4: Direct range slice indexing s[0..4]
    let block_with_slice =
        "## 6. Proposed Deterministic Implementation Contract\n```rust\npub fn bad(s: &str) -> &str {\n    &s[0..4]\n}\n```";
    let m4_lines: Vec<&str> = block_with_slice.lines().collect();
    assert!(check_9_rust_code_blocks(&m4_lines).is_err());

    // Mutation 5: Check 19 forbidden domain token
    let block_with_domain =
        "## 6. Proposed Deterministic Implementation Contract\n```rust\npub const BAD: &str = \"FLIGHT_ENVELOPE_MAP\";\n```";
    let m5_lines: Vec<&str> = block_with_domain.lines().collect();
    assert!(check_9_rust_code_blocks(&m5_lines).is_err());
}

/// Verifies `check_11_source_references` resolves workspace file references (whether `blueprint_gate.rs` or decomposed `blueprint_gate/`) and rejects phantom files.
/// Realises: [REQ-BLUEPRINT-CHECK-11-TESTS]
#[test]
fn test_check_11_source_references_pass_and_mutations() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or(manifest_dir.as_path());

    let resolved_bp_gate_files = resolve_module_rs_files(
        &workspace_root.join("crates/verify-baseline/src/blueprint_gate.rs"),
    );
    assert!(
        !resolved_bp_gate_files.is_empty(),
        "Expected crates/verify-baseline/src/blueprint_gate.rs (or blueprint_gate/ directory) to resolve to at least one .rs file"
    );
    let mut bp_gate_rel_paths = Vec::new();
    for file_path in &resolved_bp_gate_files {
        let src_res = std::fs::read_to_string(file_path);
        assert!(
            src_res.is_ok(),
            "Expected resolved blueprint_gate source {} to be readable: {src_res:?}",
            file_path.display()
        );
        if let Ok(src_text) = src_res {
            assert!(
                !src_text.trim().is_empty(),
                "Expected resolved blueprint_gate source {} to be non-empty",
                file_path.display()
            );
        }
        let rel = file_path
            .strip_prefix(workspace_root)
            .unwrap_or(file_path)
            .display()
            .to_string();
        bp_gate_rel_paths.push(rel);
    }

    let bp_gate_bullets: Vec<String> = bp_gate_rel_paths
        .iter()
        .map(|rel| format!("- `{rel}:1`"))
        .collect();
    let adapted_valid = valid_blueprint_fixture().replace(
        "- `crates/verify-baseline/src/blueprint_gate.rs:1`",
        &bp_gate_bullets.join("\n"),
    );
    let lines: Vec<&str> = adapted_valid.lines().collect();
    let ranges = check_3_sections(&adapted_valid).unwrap_or_default();
    let (s7_start, s7_end, _) = ranges.get(6).copied().unwrap_or((0, 0, ""));
    let sec7_lines = lines.get(s7_start..s7_end).unwrap_or(&[]);

    let res = check_11_source_references(workspace_root, sec7_lines);
    assert!(res.is_ok(), "Expected Check 11 pass, got: {res:?}");
    let refs = res.unwrap_or_default();
    let mut expected_refs = bp_gate_rel_paths.clone();
    expected_refs.push("crates/verify-baseline/tests/blueprint_gate_tests.rs".to_string());
    expected_refs.push("Cargo.toml".to_string());
    assert_eq!(refs, expected_refs);

    // Mutation 1: Zero local repository references (only external URLs and issue tags)
    let no_local_refs = [
        "## 7. Source References",
        "- External standard: https://www.rust-lang.org",
        "- Issue tracker: `#501`",
    ];
    assert!(check_11_source_references(workspace_root, &no_local_refs).is_err());

    // Mutation 2: Cited file path does not exist on disk
    let first_bp_gate_anchor = format!(
        "- `{}:10`",
        bp_gate_rel_paths
            .first()
            .map(String::as_str)
            .unwrap_or("Cargo.toml")
    );
    let phantom_file = [
        "## 7. Source References",
        first_bp_gate_anchor.as_str(),
        "- `crates/verify-baseline/src/non_existent_phantom_module.rs:42`",
    ];
    assert!(check_11_source_references(workspace_root, &phantom_file).is_err());
}

/// Verifies Check 10 compiles blueprint Markdown into bijective SysML v2 and persists `<stem>.sysml` and `<stem>.digest.json` in `docs/designs/`.
/// Realises: [REQ-BLUEPRINT-CHECK-10-TESTS]
#[test]
fn test_check_10_compiles_and_persists_sysml_and_digest_in_docs_designs() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_ws = std::env::temp_dir().join(format!(
        "deap_bp_check10_test_{}_{}",
        std::process::id(),
        nanos
    ));
    let designs_dir = temp_ws.join("docs").join("designs");
    let src_dir = temp_ws.join("crates").join("verify-baseline").join("src");
    let tests_dir = temp_ws.join("crates").join("verify-baseline").join("tests");

    std::fs::create_dir_all(&designs_dir).unwrap_or(());
    std::fs::create_dir_all(&src_dir).unwrap_or(());
    std::fs::create_dir_all(&tests_dir).unwrap_or(());

    std::fs::write(src_dir.join("blueprint_gate.rs"), "// stub\n").unwrap_or(());
    std::fs::write(tests_dir.join("blueprint_gate_tests.rs"), "// test stub\n").unwrap_or(());
    std::fs::write(temp_ws.join("Cargo.toml"), "[workspace]\n").unwrap_or(());

    let bp_path = designs_dir.join("blueprint-99-sample.md");
    let fixture = valid_blueprint_fixture();
    std::fs::write(&bp_path, fixture).unwrap_or(());

    let direct_res = check_10_compile_and_persist_sysml(&bp_path, fixture);
    assert!(
        direct_res.is_ok(),
        "Expected check_10_compile_and_persist_sysml to succeed, got: {direct_res:?}"
    );

    let opts = BlueprintCliOptions {
        path: PathBuf::from("docs/designs/blueprint-99-sample.md"),
        repo: None,
        issue: None,
        title: None,
        publish: false,
        dry_run: true,
    };

    let report_res = verify_blueprint(&temp_ws, &opts);
    assert!(
        report_res.is_ok(),
        "Expected verify_blueprint to succeed, got: {report_res:?}"
    );
    let report = report_res.unwrap_or_default();

    let expected_sysml = designs_dir.join("blueprint-99-sample.sysml");
    let expected_digest = designs_dir.join("blueprint-99-sample.digest.json");
    assert_eq!(report.sysml_path, expected_sysml);
    assert_eq!(report.digest_path, expected_digest);
    assert!(expected_sysml.is_file(), ".sysml file must exist on disk");
    assert!(
        expected_digest.is_file(),
        ".digest.json file must exist on disk"
    );

    let sysml_res = std::fs::read_to_string(&expected_sysml);
    let digest_res = std::fs::read_to_string(&expected_digest);
    assert!(sysml_res.is_ok(), ".sysml must be readable");
    assert!(digest_res.is_ok(), ".digest.json must be readable");
    let Ok(sysml_text) = sysml_res else {
        return;
    };
    let Ok(digest_text) = digest_res else {
        return;
    };
    assert!(!sysml_text.trim().is_empty(), ".sysml must be non-empty");
    assert!(
        !digest_text.trim().is_empty(),
        ".digest.json must be non-empty"
    );
    assert!(sysml_text.contains("package "));
    assert!(sysml_text.contains("requirement def "));
    assert!(sysml_text.contains("part def "));
    assert!(sysml_text.contains("constraint def "));

    let parsed = compile_sysml::parse_sysml(&sysml_text);
    assert!(parsed.is_ok(), "Persisted .sysml must parse cleanly");
    if let Ok(pkg) = parsed {
        let roundtrip = compile_sysml::to_sysml(&pkg);
        assert_eq!(
            roundtrip, sysml_text,
            "Persisted .sysml must be byte-for-byte roundtrip identical"
        );
    }

    // Negative mutation: Markdown lacking component/constraint constructs fails Check 10
    let incomplete_bp = designs_dir.join("blueprint-98-incomplete.md");
    let incomplete_res = check_10_compile_and_persist_sysml(
        &incomplete_bp,
        "# EmptyDoc\n\nNo requirements or parts.\n",
    );
    assert!(
        incomplete_res.is_err(),
        "Check 10 must reject markdown lacking required SysML constructs"
    );

    std::fs::remove_dir_all(&temp_ws).unwrap_or(());
}

/// Verifies Check 12 (`verify_closed_loop_github_payload` and `check_12_publish_github_ssot`) passes valid payloads and rejects title, requirement, or body mismatches.
/// Realises: [REQ-BLUEPRINT-CHECK-12-TESTS]
#[test]
fn test_check_12_closed_loop_payload_verification_passes_and_rejects_mismatch() {
    let valid_json = r#"{
        "number": 501,
        "title": "Deterministic Telemetry Gate Blueprint",
        "body": "| **Blueprint ID** | `BP-01-TELEMETRY-GATE` |\n| **Requirement ID** | `REQ-BP-001` |\nFull body content.",
        "url": "https://github.com/gintatkinson/DEAP01-spec-core/issues/501"
    }"#;

    let pass_res = verify_closed_loop_github_payload(
        valid_json,
        "Deterministic Telemetry Gate Blueprint",
        "REQ-BP-001",
    );
    assert!(
        pass_res.is_ok(),
        "Expected closed-loop verification pass, got: {pass_res:?}"
    );
    let summary = pass_res.unwrap_or_default();
    assert!(
        summary.contains("[CLOSED-LOOP VERIFIED]"),
        "Summary must contain [CLOSED-LOOP VERIFIED], got: {summary}"
    );

    // Negative 1: Mismatched title
    let bad_title =
        verify_closed_loop_github_payload(valid_json, "Wrong Blueprint Title", "REQ-BP-001");
    assert!(bad_title.is_err());

    // Negative 2: Missing requirement ID in body
    let missing_req = verify_closed_loop_github_payload(
        valid_json,
        "Deterministic Telemetry Gate Blueprint",
        "REQ-BP-999-MISSING",
    );
    assert!(missing_req.is_err());

    // Negative 3: Empty body
    let empty_body_json = r#"{
        "number": 501,
        "title": "Deterministic Telemetry Gate Blueprint",
        "body": "   ",
        "url": "https://github.com/gintatkinson/DEAP01-spec-core/issues/501"
    }"#;
    let empty_body_res = verify_closed_loop_github_payload(
        empty_body_json,
        "Deterministic Telemetry Gate Blueprint",
        "REQ-BP-001",
    );
    assert!(empty_body_res.is_err());

    // Dry-run mode on check_12_publish_github_ssot skips network and succeeds
    let opts = BlueprintCliOptions {
        path: PathBuf::from("docs/designs/blueprint-01-telemetry.md"),
        repo: Some("gintatkinson/DEAP01-spec-core".to_string()),
        issue: Some(501),
        title: None,
        publish: true,
        dry_run: true,
    };
    let meta = check_2_metadata_table(valid_blueprint_fixture()).unwrap_or_default();
    let dry_res = check_12_publish_github_ssot(&opts, &meta, valid_blueprint_fixture());
    assert!(dry_res.is_ok());
}

/// Verifies end-to-end execution of `verify_blueprint` and `run_blueprint_cli` on passing and failing blueprint files.
/// Realises: [REQ-BLUEPRINT-FULL-ORCHESTRATOR-TESTS]
#[test]
fn test_full_verify_blueprint_end_to_end_pass_and_fail_closed() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_ws =
        std::env::temp_dir().join(format!("deap_bp_e2e_test_{}_{}", std::process::id(), nanos));
    let designs_dir = temp_ws.join("docs").join("designs");
    let src_dir = temp_ws.join("crates").join("verify-baseline").join("src");
    let tests_dir = temp_ws.join("crates").join("verify-baseline").join("tests");

    std::fs::create_dir_all(&designs_dir).unwrap_or(());
    std::fs::create_dir_all(&src_dir).unwrap_or(());
    std::fs::create_dir_all(&tests_dir).unwrap_or(());

    std::fs::write(src_dir.join("blueprint_gate.rs"), "// stub\n").unwrap_or(());
    std::fs::write(tests_dir.join("blueprint_gate_tests.rs"), "// test stub\n").unwrap_or(());
    std::fs::write(temp_ws.join("Cargo.toml"), "[workspace]\n").unwrap_or(());

    let valid_bp_path = designs_dir.join("blueprint-01-valid.md");
    std::fs::write(&valid_bp_path, valid_blueprint_fixture()).unwrap_or(());

    let pass_opts = BlueprintCliOptions {
        path: PathBuf::from("docs/designs/blueprint-01-valid.md"),
        repo: None,
        issue: None,
        title: None,
        publish: false,
        dry_run: true,
    };
    let exit_pass = run_blueprint_cli(&temp_ws, &pass_opts);
    assert_eq!(exit_pass, 0, "Expected exit code 0 on valid blueprint");

    // Violated blueprint (invalid Generation Mode + broken Mermaid header)
    let bad_bp_path = designs_dir.join("blueprint-02-invalid.md");
    let bad_content = valid_blueprint_fixture()
        .replace(
            "| **Generation Mode** | `subagent` |",
            "| **Generation Mode** | `manual` |",
        )
        .replace("classDiagram", "pie title Broken");
    std::fs::write(&bad_bp_path, bad_content).unwrap_or(());

    let fail_opts = BlueprintCliOptions {
        path: PathBuf::from("docs/designs/blueprint-02-invalid.md"),
        repo: None,
        issue: None,
        title: None,
        publish: false,
        dry_run: true,
    };
    let verify_fail = verify_blueprint(&temp_ws, &fail_opts);
    assert!(
        verify_fail.is_err(),
        "Expected verify_blueprint to fail closed"
    );
    let exit_fail = run_blueprint_cli(&temp_ws, &fail_opts);
    assert_eq!(exit_fail, 1, "Expected exit code 1 on violated blueprint");
    assert!(
        !designs_dir.join("blueprint-02-invalid.sysml").exists(),
        "Check 10 must not persist .sysml when Checks 1-9 or 11 fail"
    );

    std::fs::remove_dir_all(&temp_ws).unwrap_or(());
}

/// Verifies multi-component blueprints with PascalCase classifier and primitive types, H3 subheadings, and `--dry-run` SysML persistence.
/// Realises: [REQ-BLUEPRINT-CHECK-5-COMPONENTS], [REQ-BLUEPRINT-CHECK-10-SYSML-PERSISTENCE]
#[test]
fn test_multi_component_pascal_case_types_and_dry_run_sysml_persistence() {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let temp_ws = std::env::temp_dir().join(format!(
        "deap_bp_multicomp_test_{}_{}",
        std::process::id(),
        nanos
    ));
    let designs_dir = temp_ws.join("docs").join("designs");
    let src_dir = temp_ws.join("crates").join("verify-baseline").join("src");
    let tests_dir = temp_ws.join("crates").join("verify-baseline").join("tests");

    std::fs::create_dir_all(&designs_dir).unwrap_or(());
    std::fs::create_dir_all(&src_dir).unwrap_or(());
    std::fs::create_dir_all(&tests_dir).unwrap_or(());

    std::fs::write(src_dir.join("blueprint_gate.rs"), "// stub\n").unwrap_or(());
    std::fs::write(tests_dir.join("blueprint_gate_tests.rs"), "// test stub\n").unwrap_or(());
    std::fs::write(temp_ws.join("Cargo.toml"), "[workspace]\n").unwrap_or(());

    let multi_comp_sec2 = r#"## 2. Structural Component Metamodel

### Component: BlueprintGateEngine

#### Component Attributes

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `max_allowed_errors` | `Integer` | `0` | count | Maximum permitted verification errors |
| `strict_gate_enabled` | `Boolean` | `true` | flag | Enables strict fail-closed verification |
| `timeout_budget_sec` | `Real` | `5.0` | s | Bounded execution timeout budget |
| `default_mode` | `String` | `subagent` | token | Default generation mode token |
| `active_constraint_spec` | `ConstraintSpec` | `StrictSpec` | classifier | Active constraint specification classifier |
| `boolean_policy_mode` | `BooleanPolicy` | `FailClosed` | classifier | Policy classifier containing bool substring |

#### Component Ports

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `markdown_in` | in | `String` | FileBuffer | Raw blueprint markdown input |
| `error_count_out` | out | `Integer` | CounterPort | Emitted error count |
| `gate_pass_out` | out | `Boolean` | StatusPort | Gate pass boolean indicator |
| `elapsed_sec_out` | out | `Real` | MetricPort | Elapsed verification duration |
| `spec_channel` | inout | `ConstraintSpec` | SpecBus | Bidirectional constraint spec channel |

### Component: ConstraintSpec

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `rule_count` | `Integer` | `12` | count | Number of enforced gates |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `spec_out` | out | `String` | SpecStream | Serialized specification stream |

## 3. UML & Behavioral Topology

### Structural Class Diagram
"#;

    let base_fixture = valid_blueprint_fixture();
    let Some(sec2_idx) = base_fixture.find("## 2. Structural Component Metamodel\n") else {
        return;
    };
    let Some(sec3_idx) = base_fixture.find("## 3. UML & Behavioral Topology\n") else {
        return;
    };
    let prefix = base_fixture.get(..sec2_idx).unwrap_or("");
    let suffix = base_fixture
        .get((sec3_idx + "## 3. UML & Behavioral Topology\n".len())..)
        .unwrap_or("");
    let multi_bp_md = format!("{prefix}{multi_comp_sec2}{suffix}");

    let bp_path = designs_dir.join("blueprint-03-multicomp.md");
    std::fs::write(&bp_path, &multi_bp_md).unwrap_or(());

    let opts = BlueprintCliOptions {
        path: PathBuf::from("docs/designs/blueprint-03-multicomp.md"),
        repo: None,
        issue: None,
        title: None,
        publish: false,
        dry_run: true,
    };

    let report_res = verify_blueprint(&temp_ws, &opts);
    assert!(
        report_res.is_ok(),
        "Expected multi-component blueprint to pass all 12 checks, got: {:?}",
        report_res
    );

    if let Ok(report) = report_res {
        assert_eq!(report.components.len(), 2);
        assert!(
            report.sysml_path.exists(),
            "Expected .sysml file to be persisted under --dry-run"
        );
        assert!(
            report.digest_path.exists(),
            "Expected .digest.json file to be persisted under --dry-run"
        );
        let sysml_res = std::fs::read_to_string(&report.sysml_path);
        assert!(sysml_res.is_ok(), "Persisted .sysml must be readable");
        let Ok(sysml_text) = sysml_res else {
            return;
        };
        assert!(sysml_text.contains("part def BlueprintGateEngine"));
        assert!(sysml_text.contains("part def ConstraintSpec"));
        assert!(
            !sysml_text.contains("part def Structural_Class_Diagram"),
            "Section 3 subheading must not leak as a phantom PartDef"
        );
        assert!(
            sysml_text.contains("attribute active_constraint_spec : String = \"StrictSpec\"")
                || sysml_text.contains("attribute active_constraint_spec : ConstraintSpec = \"StrictSpec\""),
            "PascalCase type containing 'int' substring must preserve its default value: got:\n{sysml_text}"
        );
        assert!(
            sysml_text.contains("attribute boolean_policy_mode : String = \"FailClosed\"")
                || sysml_text.contains("attribute boolean_policy_mode : BooleanPolicy = \"FailClosed\""),
            "PascalCase type containing 'bool' substring must preserve its default value: got:\n{sysml_text}"
        );
    }

    std::fs::remove_dir_all(&temp_ws).unwrap_or(());
}

/// Verifies `REQ-0604` `AC-01` and `AC-05`: parameterized `ActionDef` Mermaid/Markdown operation signatures, reverse bullet parsing, and parent Epic helpers.
/// Realises: [REQ-0604]
#[test]
fn test_req_0604_ac01_and_ac05_operation_signatures_and_parent_epic_helpers() {
    use compile_sysml::sync::forward::{
        collect_package_story_numbers, format_markdown_operation_bullet,
        format_mermaid_operation_signature, forward_sync_sysml_to_specs,
        parse_markdown_operation_bullet, resolve_parent_epic_link, ForwardSyncOptions, StoryMeta,
    };
    use deap_core::sysml_ast::{ActionDef, AttributeDef, PackageDef, PartDef};

    let action = ActionDef {
        name: "ComputeTrajectory".to_string(),
        in_params: vec![
            AttributeDef {
                name: "target_alt".to_string(),
                type_name: "Real".to_string(),
                ..Default::default()
            },
            AttributeDef {
                name: "step_limit".to_string(),
                type_name: "Integer".to_string(),
                ..Default::default()
            },
        ],
        out_params: vec![AttributeDef {
            name: "nav_solution".to_string(),
            type_name: "NavVector".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let mermaid_sig = format_mermaid_operation_signature(&action);
    assert_eq!(
        mermaid_sig,
        "+ComputeTrajectory(in_target_alt_Real, in_step_limit_Integer, out_nav_solution_NavVector) NavVector"
    );

    let md_bullet = format_markdown_operation_bullet(&action);
    assert_eq!(
        md_bullet,
        "- `+ComputeTrajectory(in target_alt: Real, in step_limit: Integer, out nav_solution: NavVector) : NavVector`"
    );

    let parsed = parse_markdown_operation_bullet(&md_bullet);
    assert_eq!(parsed, Some(action.clone()));

    // Negative / fail-closed probes for malformed bullets (AC-05)
    assert_eq!(
        parse_markdown_operation_bullet("- `ComputeTrajectory()`"),
        None
    );
    assert_eq!(parse_markdown_operation_bullet("- `+()`"), None);
    assert_eq!(parse_markdown_operation_bullet("plain text line"), None);

    let pkg_root = PackageDef {
        name: "SystemRoot".to_string(),
        ..Default::default()
    };
    let pkg_conops = PackageDef {
        name: "ConOps".to_string(),
        ..Default::default()
    };
    let mut pkg_sub = PackageDef {
        name: "GuidanceSubsystem".to_string(),
        ..Default::default()
    };
    let mut guidance_part = PartDef {
        name: "GuidanceComputer".to_string(),
        ..Default::default()
    };
    guidance_part.actions.push(action);
    pkg_sub.part_defs.push(guidance_part);

    let ordered = vec![&pkg_root, &pkg_conops, &pkg_sub];
    assert_eq!(
        resolve_parent_epic_link(&ordered, "SystemRoot"),
        "[EPIC-01-SystemRoot](../epics/EPIC-01-SystemRoot.md)"
    );
    assert_eq!(
        resolve_parent_epic_link(&ordered, "ConOps"),
        "[EPIC-02-ConOps](../epics/EPIC-02-ConOps.md)"
    );
    assert_eq!(
        resolve_parent_epic_link(&ordered, "GuidanceSubsystem"),
        "[EPIC-03-GuidanceSubsystem](../epics/EPIC-03-GuidanceSubsystem.md)"
    );

    let stories = vec![
        StoryMeta {
            pkg_name: "GuidanceSubsystem".to_string(),
            s_num: 1,
            ..Default::default()
        },
        StoryMeta {
            pkg_name: "OtherSubsystem".to_string(),
            s_num: 2,
            ..Default::default()
        },
        StoryMeta {
            pkg_name: "GuidanceSubsystem".to_string(),
            s_num: 3,
            ..Default::default()
        },
    ];
    assert_eq!(
        collect_package_story_numbers(&stories, "GuidanceSubsystem"),
        vec![1, 3]
    );

    let mut root_model = pkg_root.clone();
    root_model.packages.push(pkg_sub);
    let opts = ForwardSyncOptions {
        schema_path: PathBuf::from("schema/model.sysml"),
        docs_dir: PathBuf::from("docs"),
        out_dir: None,
        dry_run: true,
        force: true,
    };
    let generated = forward_sync_sysml_to_specs(&root_model, &opts).unwrap_or_default();
    let feat_md = generated
        .get("features/FEAT-01-GuidanceComputer.md")
        .map(String::as_str)
        .unwrap_or("");
    assert!(feat_md.contains(&mermaid_sig));
    assert!(feat_md.contains(&md_bullet));
}

/// Verifies `REQ-0604` `AC-02`: Level 1C Logical ICD Matrix (`ICD_01`) and Master Signal Dictionary (`ICD_02`) compilation and `E0200` rejection.
/// Realises: [REQ-0604]
#[test]
fn test_req_0604_ac02_logical_icd_matrix_and_signal_dictionary() {
    use compile_sysml::sync::forward::{
        compile_logical_icd_artifacts, emit_logical_icd_artifacts,
        try_compile_logical_icd_artifacts,
    };
    use deap_core::sysml_ast::{ConnectionDef, PackageDef, PartDef, PortDef};

    let mut pkg = PackageDef {
        name: "AvionicsSystem".to_string(),
        ..Default::default()
    };
    pkg.part_defs.push(PartDef {
        name: "FlightDirector".to_string(),
        ports: vec![PortDef {
            name: "cmd_out".to_string(),
            type_name: "GuidanceCmd".to_string(),
            direction: "out".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });
    pkg.part_defs.push(PartDef {
        name: "ServoActuator".to_string(),
        ports: vec![PortDef {
            name: "cmd_in".to_string(),
            type_name: "GuidanceCmd".to_string(),
            direction: "in".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    });
    pkg.connection_defs.push(ConnectionDef {
        name: "GuidanceLink".to_string(),
        source_part: Some("FlightDirector".to_string()),
        source_port: "cmd_out".to_string(),
        target_part: Some("ServoActuator".to_string()),
        target_port: "cmd_in".to_string(),
        item_flow_ref: Some("GuidanceCmd".to_string()),
        is_flow: true,
        ..Default::default()
    });

    let (icd01, icd02) = compile_logical_icd_artifacts(&pkg);
    assert!(icd01.contains("# ICD_01: System Interface Matrix"));
    assert!(icd01.contains("| **FlightDirector** | --- |"));
    assert!(icd01.contains("GuidanceCmd"));
    assert!(icd02.contains("# ICD_02: Master Signal Dictionary"));
    assert!(icd02.contains("GuidanceCmd"));

    let tmp_dir =
        std::env::temp_dir().join(format!("deap_req_0604_icd_test_{}", std::process::id()));
    let emitted = emit_logical_icd_artifacts(&pkg, &tmp_dir);
    assert!(
        emitted.is_ok(),
        "Expected ICD emission to succeed: {emitted:?}"
    );
    if let Ok((p1, p2)) = emitted {
        assert!(p1.is_file());
        assert!(p2.is_file());
    }
    std::fs::remove_dir_all(&tmp_dir).unwrap_or(());

    // Negative probe 1: undeclared endpoint fails closed with Diagnostic: E0200
    let mut bad_pkg = pkg.clone();
    bad_pkg.connection_defs.push(ConnectionDef {
        name: "BrokenLink".to_string(),
        source_part: Some("FlightDirector".to_string()),
        source_port: "cmd_out".to_string(),
        target_part: Some("GhostSubsystem".to_string()),
        target_port: "missing_port".to_string(),
        ..Default::default()
    });
    let err_res = try_compile_logical_icd_artifacts(&bad_pkg);
    assert!(
        err_res
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("Diagnostic: E0200")),
        "Expected Diagnostic: E0200 on undeclared endpoint, got: {err_res:?}"
    );

    // Negative probe 2: empty package with zero interfaces fails closed with Diagnostic: E0200
    let empty_pkg = PackageDef {
        name: "EmptyPkg".to_string(),
        ..Default::default()
    };
    let empty_res = try_compile_logical_icd_artifacts(&empty_pkg);
    assert!(
        empty_res
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("Diagnostic: E0200")),
        "Expected Diagnostic: E0200 on empty model, got: {empty_res:?}"
    );
}

/// Verifies `REQ-0604` `AC-03`: Level 4 MIL-STD-881E WBS & Enterprise Export compilation, 7-file persistence, and `E0300` rejection on empty models.
/// Realises: [REQ-0604]
#[test]
fn test_req_0604_ac03_wbs_level4_and_enterprise_exports() {
    use compile_sysml::sync::forward::{
        compile_wbs_artifacts, emit_wbs_artifacts, try_compile_wbs_artifacts,
    };
    use deap_core::sysml_ast::{PackageDef, PartDef, RequirementDef, TestCaseDef};

    let mut root = PackageDef {
        name: "MissionSystem".to_string(),
        ..Default::default()
    };
    let mut sub = PackageDef {
        name: "NavigationPkg".to_string(),
        ..Default::default()
    };
    sub.part_defs.push(PartDef {
        name: "InertialSolver".to_string(),
        ..Default::default()
    });
    sub.requirement_defs.push(RequirementDef {
        name: "ReqNavAccuracy".to_string(),
        req_id: "REQ-0101".to_string(),
        ..Default::default()
    });
    sub.test_case_defs.push(TestCaseDef {
        name: "TC_NavAccuracy".to_string(),
        ..Default::default()
    });
    root.packages.push(sub);

    let bundle = compile_wbs_artifacts(&root);
    assert!(bundle.wbs_01_md.contains("01.00.00"));
    assert!(bundle.wbs_01_md.contains("01.01.00"));
    assert!(bundle.wbs_01_md.contains("01.01.01"));
    assert!(bundle.wbs_02_md.contains("FEAT-01"));
    assert!(bundle.wbs_02_md.contains("REQ-0101"));
    assert!(bundle.wbs_03_md.contains("Enterprise Export Schema"));
    assert!(bundle.jira_csv.contains("WBS Code"));
    assert!(bundle.monday_csv.contains("WBS Code"));
    assert!(bundle.msproject_xml.contains("<Project"));
    assert!(bundle.ast_json.contains("\"wbs_code\": \"01.01.01\""));

    let tmp_dir =
        std::env::temp_dir().join(format!("deap_req_0604_wbs_test_{}", std::process::id()));
    let written = emit_wbs_artifacts(&root, &tmp_dir);
    assert!(
        written.is_ok(),
        "Expected WBS emission to succeed: {written:?}"
    );
    if let Ok(paths) = written {
        assert_eq!(paths.len(), 7);
        for p in &paths {
            assert!(
                p.is_file(),
                "Expected emitted WBS artifact to exist: {}",
                p.display()
            );
        }
    }
    std::fs::remove_dir_all(&tmp_dir).unwrap_or(());

    // Negative probe: empty model with zero PartDefs fails closed with Diagnostic: E0300
    let empty_pkg = PackageDef {
        name: "EmptySystem".to_string(),
        ..Default::default()
    };
    let err_res = try_compile_wbs_artifacts(&empty_pkg);
    assert!(
        err_res
            .as_ref()
            .err()
            .is_some_and(|e| e.contains("Diagnostic: E0300")),
        "Expected Diagnostic: E0300 on zero PartDef model, got: {err_res:?}"
    );
}

/// Verifies `REQ-0604` `AC-04`: inline HTML comment re-open transitions, inclusive Epic counting, and `--emit-icd` / `--emit-wbs` pre-dispatch.
/// Realises: [REQ-0604]
#[test]
fn test_req_0604_ac04_html_comment_reopen_and_inclusive_epic_count() {
    use compile_sysml::sync::forward::{
        advance_html_comment_state, count_total_epics_inclusive, execute_emit_cli_args,
    };

    let mut in_comment = false;
    assert!(advance_html_comment_state(
        "<!-- start comment",
        &mut in_comment
    ));
    assert!(in_comment);

    // Same-line close followed by unclosed re-open must keep in_comment == true
    assert!(advance_html_comment_state(
        "end first --> middle <!-- reopen second",
        &mut in_comment
    ));
    assert!(
        in_comment,
        "Inline '--> ... <!--' without second close must remain in_html_comment = true"
    );

    // Same-line close followed by closed re-open must set in_comment == false
    assert!(advance_html_comment_state(
        "end second --> middle <!-- inline closed -->",
        &mut in_comment
    ));
    assert!(!in_comment);

    let child_specs = vec![
        (
            "epics/EPIC-02-ConOps.md".to_string(),
            "# Epic 2".to_string(),
        ),
        ("epics/EPIC-03-Nav.md".to_string(), "# Epic 3".to_string()),
        (
            "epics/EPIC-02-ConOps.md".to_string(),
            "# Epic 2 dup".to_string(),
        ),
        (
            "features/FEAT-01-Core.md".to_string(),
            "# Feat 1".to_string(),
        ),
    ];
    assert_eq!(
        count_total_epics_inclusive("epics/EPIC-01-Root.md", true, &child_specs),
        3
    );
    assert_eq!(
        count_total_epics_inclusive("features/FEAT-01-Core.md", false, &child_specs),
        2
    );

    // Verify CLI pre-dispatch returns Ok(None) when neither --emit-icd nor --emit-wbs is passed
    let normal_args = vec![
        "compile-sysml".to_string(),
        "schema/model.sysml".to_string(),
    ];
    assert_eq!(execute_emit_cli_args(&normal_args), Ok(None));
}

/// Verifies `REQ-0605` `AC-01`..`AC-05`: unified lexer state, multi-line Dart `@pragma` idempotency, strict `owner_name` schema isolation, and deduplicated scanners.
/// Realises: [REQ-0605]
#[test]
fn test_req_0605_ac01_ac05_unified_lexer_multiline_annotations_and_schema_isolation() {
    use compile_sysml::docstring::{
        collect_all_parts, inject_dart_file_docstrings, inject_rust_file_docstrings,
        resolve_schema_context_strict,
    };
    use deap_core::source_ast::{
        extract_rust_impl_header_prefix, parse_rust_impl_header, scan_line_with_unified_lex_state,
        strip_dart_annotations_tracking_state, RustDartLexState, ScopedItemScannerState,
        SourceLanguage,
    };
    use deap_core::sysml_ast::{ConstraintDef, PackageDef, PartDef, RequirementDef};
    use verify_baseline::checks::docstring_gate::{
        audit_dart_source_docstrings, audit_rust_source_docstrings,
    };

    // AC-01 & AC-02: Multi-line Dart @pragma annotation 2-pass idempotency and Check 36 audit
    let dart_multiline_annot_src = "\
@pragma(
  'vm:prefer-inline',
)
int computeSignalStep(int currentStep) {
  return currentStep + 1;
}
";
    let pass1_res = inject_dart_file_docstrings(
        dart_multiline_annot_src,
        "app_flutter/lib/signal_step.dart",
        None,
        false,
    );
    assert!(
        pass1_res.is_ok(),
        "Expected Pass 1 of inject_dart_file_docstrings to succeed: {pass1_res:?}"
    );
    let (pass1_out, pass1_count) = pass1_res.unwrap_or_default();
    assert_eq!(
        pass1_count, 1,
        "Expected Pass 1 to enrich 1 Dart function with multi-line @pragma, got {pass1_count} in:\n{pass1_out}"
    );
    let doc_pos = pass1_out.find("/// ## 1. Safety Intent");
    let pragma_pos = pass1_out.find("@pragma(");
    assert!(
        doc_pos.is_some() && pragma_pos.is_some() && doc_pos < pragma_pos,
        "Synthesized docstring must be placed BEFORE @pragma(, got:\n{pass1_out}"
    );

    let pass2_res =
        inject_dart_file_docstrings(&pass1_out, "app_flutter/lib/signal_step.dart", None, false);
    assert!(
        pass2_res.is_ok(),
        "Expected Pass 2 of inject_dart_file_docstrings to succeed: {pass2_res:?}"
    );
    let (pass2_out, pass2_count) = pass2_res.unwrap_or_default();
    assert_eq!(
        pass2_count, 0,
        "Expected Pass 2 on multi-line @pragma Dart function to be idempotent (count == 0), got {pass2_count} in:\n{pass2_out}"
    );
    assert_eq!(pass2_out, pass1_out);
    assert_eq!(
        audit_dart_source_docstrings(&pass1_out, "app_flutter/lib/signal_step.dart"),
        Ok(1)
    );

    // AC-03 & AC-04: Unmapped impl owner must not fall through to package-level requirement_defs / constraint_defs
    let schema_pkg = PackageDef {
        name: "CoreControlPkg".to_string(),
        part_defs: vec![PartDef {
            name: "MappedDispatcher".to_string(),
            is_def: true,
            satisfied_requirements: vec!["REQ-MAPPED-01".to_string()],
            ..Default::default()
        }],
        requirement_defs: vec![RequirementDef {
            name: "process_frame".to_string(),
            req_id: "REQ-PKG-LEAK-999".to_string(),
            ..Default::default()
        }],
        constraint_defs: vec![ConstraintDef {
            name: "invariant_process_frame_1".to_string(),
            expression: "frame_seq > 0".to_string(),
            is_assertion: true,
            ..Default::default()
        }],
        ..Default::default()
    };

    let unmapped_owner_rust = "\
pub struct UnmappedHelper;

impl UnmappedHelper {
    pub fn process_frame(&self, frame_seq: u32) -> u32 {
        frame_seq
    }
}
";
    let mut parts_out = Vec::new();
    collect_all_parts(&schema_pkg, &mut parts_out);
    assert_eq!(parts_out.len(), 1);
    assert_eq!(
        resolve_schema_context_strict(
            Some(&schema_pkg),
            Some("UnmappedHelper"),
            "process_frame"
        ),
        (None, None),
        "resolve_schema_context_strict must return (None, None) when owner_name is absent from all_parts"
    );
    assert_eq!(
        resolve_schema_context_strict(
            Some(&schema_pkg),
            Some("MappedDispatcher"),
            "process_frame"
        ),
        (
            Some("REQ-PKG-LEAK-999".to_string()),
            Some("invariant_process_frame_1 (frame_seq > 0)".to_string()),
        ),
        "resolve_schema_context_strict must resolve member requirement and constraint when owner_name is a valid PartDef"
    );
    let rust_res = inject_rust_file_docstrings(
        unmapped_owner_rust,
        "crates/core_module/src/unmapped_helper.rs",
        Some(&schema_pkg),
        false,
    );
    assert!(
        rust_res.is_ok(),
        "Expected Rust docstring injection to succeed: {rust_res:?}"
    );
    let (rust_enriched, rust_count) = rust_res.unwrap_or_default();
    assert_eq!(rust_count, 2);
    assert!(
        !rust_enriched.contains("REQ-PKG-LEAK-999"),
        "Unmapped impl owner UnmappedHelper must NOT inherit package-level REQ-PKG-LEAK-999:\n{rust_enriched}"
    );
    assert!(
        !rust_enriched.contains("frame_seq > 0"),
        "Unmapped impl owner UnmappedHelper must NOT inherit package-level constraint 'frame_seq > 0':\n{rust_enriched}"
    );
    assert_eq!(
        audit_rust_source_docstrings(&rust_enriched.replace("Realises: [", "[DOC-TIER: A] Realises: ["), "crates/core_module/src/unmapped_helper.rs"),
        Ok(2)
    );

    // AC-05: Unified lexer state and absence of duplicate scanners in docstring and docstring_gate modules
    let mut dart_lex = RustDartLexState {
        language: SourceLanguage::Dart,
        ..Default::default()
    };
    let mut scoped = ScopedItemScannerState::default();
    assert_eq!(scoped.dart_annotation_paren_depth, 0);
    assert_eq!(
        strip_dart_annotations_tracking_state("@pragma(", &mut scoped.dart_annotation_paren_depth),
        ""
    );
    assert_eq!(scoped.dart_annotation_paren_depth, 1);
    assert_eq!(
        strip_dart_annotations_tracking_state(
            "'vm:prefer-inline',",
            &mut scoped.dart_annotation_paren_depth
        ),
        ""
    );
    assert_eq!(scoped.dart_annotation_paren_depth, 1);
    assert_eq!(
        strip_dart_annotations_tracking_state(
            ") void runStep() {}",
            &mut scoped.dart_annotation_paren_depth
        )
        .trim(),
        "void runStep() {}"
    );
    assert_eq!(scoped.dart_annotation_paren_depth, 0);
    assert_eq!(
        extract_rust_impl_header_prefix("impl<F: Fn(u32) -> bool> CoreHandler<F> {"),
        Some("impl<F: Fn(u32) -> bool> CoreHandler<F>".to_string())
    );
    assert_eq!(
        parse_rust_impl_header("impl<F: Fn(u32) -> bool> CoreHandler<F> {"),
        Some("CoreHandler".to_string())
    );
    let scanned_dart =
        scan_line_with_unified_lex_state("void/*inline*/executeStep() {}", &mut dart_lex);
    assert_eq!(scanned_dart.trim(), "void  executeStep() {}");

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("."));
    let docstring_files = resolve_module_rs_files(
        &workspace_root
            .join("crates")
            .join("compile-sysml")
            .join("src")
            .join("docstring.rs"),
    );
    assert!(
        !docstring_files.is_empty(),
        "Expected crates/compile-sysml/src/docstring.rs (or docstring/ directory) to resolve to at least one .rs file"
    );
    for file_path in &docstring_files {
        let read_res = std::fs::read_to_string(file_path);
        assert!(
            read_res.is_ok(),
            "Expected {} to be readable: {read_res:?}",
            file_path.display()
        );
        if let Ok(docstring_rs_src) = read_res {
            assert!(
                !docstring_rs_src.contains("fn extract_rust_impl_header_prefix("),
                "{} must delegate extract_rust_impl_header_prefix to deap_core::source_ast",
                file_path.display()
            );
        }
    }

    let docstring_gate_files = resolve_module_rs_files(
        &workspace_root
            .join("crates")
            .join("verify-baseline")
            .join("src")
            .join("checks")
            .join("docstring_gate.rs"),
    );
    assert!(
        !docstring_gate_files.is_empty(),
        "Expected crates/verify-baseline/src/checks/docstring_gate.rs (or docstring_gate/ directory) to resolve to at least one .rs file"
    );
    for file_path in &docstring_gate_files {
        let read_res = std::fs::read_to_string(file_path);
        assert!(
            read_res.is_ok(),
            "Expected {} to be readable: {read_res:?}",
            file_path.display()
        );
        if let Ok(docstring_gate_src) = read_res {
            assert!(
                !docstring_gate_src.contains("enum DartMultilineString")
                    && !docstring_gate_src.contains("fn scan_dart_line_preserving_lexical_state("),
                "{} must delegate Dart lexical scanning to deap_core::source_ast",
                file_path.display()
            );
        }
    }
}

/// RAII temporary workspace directory fixture that deterministically removes its directory tree on drop.
/// Realises: [REQ-0606/TEMP-WORKSPACE-RAII]
struct TempWorkspace {
    root: PathBuf,
}

impl TempWorkspace {
    /// Allocates a fresh RAII temporary workspace directory underneath `std::env::temp_dir()`.
    /// Realises: [REQ-0606/TEMP-WORKSPACE-NEW]
    fn new(prefix: &str) -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let root =
            std::env::temp_dir().join(format!("deap_{}_{}_{}", prefix, std::process::id(), nanos));
        let create_res = std::fs::create_dir_all(&root);
        assert!(
            create_res.is_ok(),
            "Failed to create TempWorkspace root {}: {create_res:?}",
            root.display()
        );
        Self { root }
    }

    /// Writes `contents` to `rel_path` inside `self.root`, creating parent directories as needed.
    /// Realises: [REQ-0606/TEMP-WORKSPACE-WRITE]
    fn write_file(&self, rel_path: &str, contents: &str) -> PathBuf {
        let full = self.root.join(rel_path);
        if let Some(parent) = full.parent() {
            let dir_res = std::fs::create_dir_all(parent);
            assert!(
                dir_res.is_ok(),
                "Failed to create parent directory {}: {dir_res:?}",
                parent.display()
            );
        }
        let write_res = std::fs::write(&full, contents);
        assert!(
            write_res.is_ok(),
            "Failed to write fixture file {}: {write_res:?}",
            full.display()
        );
        full
    }
}

impl Drop for TempWorkspace {
    /// Reclaims the temporary workspace directory tree from `std::env::temp_dir()` upon scope exit.
    /// Realises: [REQ-0606/TEMP-WORKSPACE-DROP]
    fn drop(&mut self) {
        if self.root.exists() {
            std::fs::remove_dir_all(&self.root).unwrap_or(());
        }
    }
}

/// Reads all synthesized `DEFECT-*.md` files from `defects_dir` into a deterministically ordered map.
/// Realises: [REQ-0606/DOSSIER-SNAPSHOT-HELPER]
fn collect_defect_dossiers(defects_dir: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut dossiers = std::collections::BTreeMap::new();
    let read_res = std::fs::read_dir(defects_dir);
    assert!(
        read_res.is_ok(),
        "Expected defects directory {} to be readable: {read_res:?}",
        defects_dir.display()
    );
    if let Ok(entries) = read_res {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if file_name.starts_with("DEFECT-") && file_name.ends_with(".md") {
                let bytes_res = std::fs::read(&path);
                assert!(
                    bytes_res.is_ok(),
                    "Expected dossier {} to be readable: {bytes_res:?}",
                    path.display()
                );
                if let Ok(bytes) = bytes_res {
                    dossiers.insert(file_name, bytes);
                }
            }
        }
    }
    dossiers
}

/// Verifies `REQ-0606` `AC-01` through `AC-04`: `verify-baseline audit-code` 5-pillar static defect
/// detection (`E0601`..`E0605`), closed-loop 12-check defect dossier synthesis (`check-defect`),
/// 100% byte-identical two-run idempotency, `--pillar concurrency` filtering, and fail-closed CLI
/// exit codes (`1` on missing `--path` or unbalanced braces, `2` on invalid `--pillar`).
///
/// Realises: [REQ-0606/AC-01], [REQ-0606/AC-02], [REQ-0606/AC-03], [REQ-0606/AC-04]
#[test]
fn req_0606_ac01_to_ac04_cli_audit_code_five_pillars_dossiers_idempotency_and_fail_closed() {
    use std::process::Command;
    use verify_baseline::audit_code::{
        ClosedLoopDefectPublisherGate, DeterministicDossierSynthesizer,
        DeterministicStaticDefectAuditor, FivePillarStaticAstAnalyzer, MermaidUmlDiagramGenerator,
    };

    let auditor_meta = DeterministicStaticDefectAuditor::default();
    assert_eq!(auditor_meta.auditor_subagent_count, 0);
    assert_eq!(auditor_meta.dossier_schema_check_count, 12);
    assert_eq!(auditor_meta.pillar_coverage_count, 5);
    assert_eq!(auditor_meta.closed_loop_mismatch_bytes, 0);
    let analyzer_meta = FivePillarStaticAstAnalyzer::default();
    assert_eq!(analyzer_meta.active_pillar_count, 5);
    assert!(analyzer_meta.zero_regex_mode);
    let synth_meta = DeterministicDossierSynthesizer::default();
    assert_eq!(synth_meta.mandatory_section_count, 7);
    assert_eq!(synth_meta.whys_chain_depth, 5);
    let uml_meta = MermaidUmlDiagramGenerator::default();
    assert!(uml_meta.forbid_semicolons && uml_meta.check_13b_compliant);
    let gate_meta = ClosedLoopDefectPublisherGate::default();
    assert_eq!(gate_meta.schema_gate_count, 12);
    assert!(gate_meta.require_byte_parity);

    let ws = TempWorkspace::new("req_0606_five_pillars");
    assert!(std::fs::create_dir_all(ws.root.join(".git").join("objects")).is_ok());
    assert!(std::fs::create_dir_all(ws.root.join(".git").join("refs")).is_ok());
    ws.write_file(".git/HEAD", "ref: refs/heads/main\n");
    ws.write_file(
        ".git/config",
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n[remote \"origin\"]\n\turl = https://github.com/sample-org/sample-repo.git\n",
    );
    let defects_dir = ws.root.join(".pipeline").join("defects");
    let create_defects_res = std::fs::create_dir_all(&defects_dir);
    assert!(
        create_defects_res.is_ok(),
        "Failed to create .pipeline/defects directory: {create_defects_res:?}"
    );

    ws.write_file(
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/sample_crate\"]\n",
    );
    ws.write_file(
        "crates/sample_crate/Cargo.toml",
        "[package]\nname = \"sample_crate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );

    // Dirty Rust source triggering:
    // - Pillar 1 (E0601): unchecked slice `&buf[0..2]`, direct index `sub[0]`, and narrowing `wide as u8` cast
    // - Pillar 4 (E0604): 0-assertion `#[test]` function
    // - Pillar 5 (E0605): `pub fn` lacking `Realises: [REQ-...]`
    let dirty_rust = r#"/// Computes a sample byte using unchecked slicing and narrowing cast.
pub fn slice_and_cast_sample(buf: &[u8], wide: u64) -> u8 {
    let sub = &buf[0..2];
    let narrow = wide as u8;
    sub[0].wrapping_add(narrow)
}

/// Public function intentionally missing any requirement traceability tag.
pub fn untraced_counter_step(current: u32) -> u32 {
    current.saturating_add(1)
}

#[test]
fn hollow_zero_assertion_probe() {
    let computed = 2 + 2;
    if computed == 99 {}
}
"#;
    ws.write_file("crates/sample_crate/src/lib.rs", dirty_rust);

    // Dirty Dart source triggering:
    // - Pillar 2 (E0602): ChangeNotifier holding undisposed StreamSubscription
    // - Pillar 3 (E0603): async method calling notifyListeners() after await without !mounted / _disposed guard
    let dirty_dart = r#"import 'dart:async';
import 'package:flutter/foundation.dart';

class SampleStreamNotifier extends ChangeNotifier {
  StreamSubscription<int>? _subscription;
  int latestValue = 0;

  void attachStream(Stream<int> source) {
    _subscription = source.listen((event) {
      latestValue = event;
      notifyListeners();
    });
  }

  Future<void> refreshAfterAwait(Future<int> pendingValue) async {
    final resolved = await pendingValue;
    latestValue = resolved;
    notifyListeners();
  }
}
"#;
    ws.write_file("app_flutter/lib/sample_notifier.dart", dirty_dart);

    let bin = env!("CARGO_BIN_EXE_verify-baseline");
    let ws_root_str = ws.root.to_str().unwrap_or("");
    assert!(
        !ws_root_str.is_empty(),
        "Workspace path must be valid UTF-8"
    );

    // Run 1: full 5-pillar audit in --dry-run mode
    let run1_res = Command::new(bin)
        .args(["audit-code", "--path", ws_root_str, "--dry-run"])
        .output();
    assert!(
        run1_res.is_ok(),
        "Failed to invoke verify-baseline audit-code (run 1): {run1_res:?}"
    );
    let Ok(run1_out) = run1_res else {
        return;
    };
    assert_eq!(
        run1_out.status.code(),
        Some(0),
        "Expected audit-code run 1 exit code Some(0), got {:?}; stderr:\n{}",
        run1_out.status.code(),
        String::from_utf8_lossy(&run1_out.stderr)
    );
    let run1_dossiers = collect_defect_dossiers(&defects_dir);
    assert!(
        !run1_dossiers.is_empty(),
        "Expected .pipeline/defects/DEFECT-*.md dossiers to be synthesized on run 1"
    );

    // Run 2: repeat identical invocation to verify 100% byte-identical idempotency (AC-03)
    let run2_res = Command::new(bin)
        .args(["audit-code", "--path", ws_root_str, "--dry-run"])
        .output();
    assert!(
        run2_res.is_ok(),
        "Failed to invoke verify-baseline audit-code (run 2): {run2_res:?}"
    );
    let Ok(run2_out) = run2_res else {
        return;
    };
    assert_eq!(
        run2_out.status.code(),
        Some(0),
        "Expected audit-code run 2 exit code Some(0), got {:?}; stderr:\n{}",
        run2_out.status.code(),
        String::from_utf8_lossy(&run2_out.stderr)
    );
    let run2_dossiers = collect_defect_dossiers(&defects_dir);
    assert_eq!(
        run1_dossiers, run2_dossiers,
        "Run 1 and Run 2 synthesized DEFECT-*.md sets and byte contents must be 100% identical (AC-03)"
    );

    // Verify all 5 pillars (Pillar 1..Pillar 5 and E0601..E0605) are covered across synthesized dossiers (AC-01)
    let mut combined_corpus = String::new();
    for bytes in run1_dossiers.values() {
        combined_corpus.push_str(&String::from_utf8_lossy(bytes));
        combined_corpus.push('\n');
    }
    for pillar_label in ["Pillar 1", "Pillar 2", "Pillar 3", "Pillar 4", "Pillar 5"] {
        assert!(
            combined_corpus.contains(pillar_label),
            "Synthesized defect dossiers must cover {pillar_label}; corpus:\n{combined_corpus}"
        );
    }
    for diag_code in ["E0601", "E0602", "E0603", "E0604", "E0605"] {
        assert!(
            combined_corpus.contains(diag_code),
            "Synthesized defect dossiers must report diagnostic {diag_code}; corpus:\n{combined_corpus}"
        );
    }

    // Verify every synthesized dossier passes `verify-baseline check-defect <dossier> --dry-run` (AC-02)
    for file_name in run1_dossiers.keys() {
        let rel_dossier = format!(".pipeline/defects/{file_name}");
        let check_res = Command::new(bin)
            .current_dir(&ws.root)
            .args(["check-defect", &rel_dossier, "--dry-run"])
            .output();
        assert!(
            check_res.is_ok(),
            "Failed to spawn check-defect for {rel_dossier}: {check_res:?}"
        );
        if let Ok(check_out) = check_res {
            assert_eq!(
                check_out.status.code(),
                Some(0),
                "Synthesized dossier {rel_dossier} failed check-defect --dry-run with {:?}; stdout:\n{}\nstderr:\n{}",
                check_out.status.code(),
                String::from_utf8_lossy(&check_out.stdout),
                String::from_utf8_lossy(&check_out.stderr)
            );
        }
    }

    // Verify `--pillar concurrency` filters strictly to Pillar 3 (AC-01)
    for file_name in run1_dossiers.keys() {
        let rm_res = std::fs::remove_file(defects_dir.join(file_name));
        assert!(
            rm_res.is_ok(),
            "Failed to clear dossier {file_name}: {rm_res:?}"
        );
    }
    let filter_res = Command::new(bin)
        .args([
            "audit-code",
            "--path",
            ws_root_str,
            "--pillar",
            "concurrency",
            "--dry-run",
        ])
        .output();
    assert!(
        filter_res.is_ok(),
        "Failed to invoke audit-code --pillar concurrency: {filter_res:?}"
    );
    if let Ok(filter_out) = filter_res {
        assert_eq!(
            filter_out.status.code(),
            Some(0),
            "Expected audit-code --pillar concurrency exit code Some(0), got {:?}; stderr:\n{}",
            filter_out.status.code(),
            String::from_utf8_lossy(&filter_out.stderr)
        );
    }
    let filtered_dossiers = collect_defect_dossiers(&defects_dir);
    assert!(
        !filtered_dossiers.is_empty(),
        "Expected --pillar concurrency to synthesize at least one Pillar 3 dossier"
    );
    for (file_name, bytes) in &filtered_dossiers {
        let body = String::from_utf8_lossy(bytes);
        assert!(
            body.contains("Pillar 3") && body.contains("E0603"),
            "Filtered dossier {file_name} must target Pillar 3 / E0603, got:\n{body}"
        );
        for other_code in ["E0601", "E0602", "E0604", "E0605"] {
            assert!(
                !body.contains(other_code),
                "Filtered concurrency dossier {file_name} must not include {other_code}"
            );
        }
    }

    // Verify relative subdirectory & single-file `--path` resolve workspace root cleanly (#691)
    for rel_subtarget in ["crates/sample_crate", "crates/sample_crate/src/lib.rs"] {
        let existing_dossiers = collect_defect_dossiers(&defects_dir);
        for file_name in existing_dossiers.keys() {
            let rm_res = std::fs::remove_file(defects_dir.join(file_name));
            assert!(
                rm_res.is_ok(),
                "Failed to clear dossier {file_name} before relative --path {rel_subtarget}: {rm_res:?}"
            );
        }
        let rel_run_res = Command::new(bin)
            .current_dir(&ws.root)
            .args(["audit-code", "--path", rel_subtarget, "--dry-run"])
            .output();
        assert!(
            rel_run_res.is_ok(),
            "Failed to invoke audit-code --path {rel_subtarget}: {rel_run_res:?}"
        );
        if let Ok(rel_out) = rel_run_res {
            assert_eq!(
                rel_out.status.code(),
                Some(0),
                "Expected audit-code --path {rel_subtarget} exit code Some(0), got {:?}; stderr:\n{}",
                rel_out.status.code(),
                String::from_utf8_lossy(&rel_out.stderr)
            );
        }
        assert!(
            !ws.root.join("crates/.pipeline").exists(),
            "Relative --path {rel_subtarget} must not create nested crates/.pipeline"
        );
        assert!(
            !ws.root.join("crates/sample_crate/.pipeline").exists(),
            "Relative --path {rel_subtarget} must not create nested crates/sample_crate/.pipeline"
        );
        let rel_dossiers = collect_defect_dossiers(&defects_dir);
        assert!(
            !rel_dossiers.is_empty(),
            "Expected relative --path {rel_subtarget} to synthesize dossiers in workspace .pipeline/defects"
        );
        for (file_name, bytes) in &rel_dossiers {
            let body = String::from_utf8_lossy(bytes);
            assert!(
                body.contains("FILE_LOCATION: crates/sample_crate/src/lib.rs:"),
                "Dossier {file_name} for --path {rel_subtarget} must preserve full workspace-relative FILE_LOCATION, got:\n{body}"
            );
            let rel_dossier = format!(".pipeline/defects/{file_name}");
            let check_res = Command::new(bin)
                .current_dir(&ws.root)
                .args(["check-defect", &rel_dossier, "--dry-run"])
                .output();
            assert!(
                check_res.is_ok(),
                "Failed to spawn check-defect for {rel_dossier} (--path {rel_subtarget}): {check_res:?}"
            );
            if let Ok(check_out) = check_res {
                assert_eq!(
                    check_out.status.code(),
                    Some(0),
                    "Synthesized dossier {rel_dossier} (--path {rel_subtarget}) failed check-defect --dry-run with {:?}; stdout:\n{}\nstderr:\n{}",
                    check_out.status.code(),
                    String::from_utf8_lossy(&check_out.stdout),
                    String::from_utf8_lossy(&check_out.stderr)
                );
            }
        }
    }

    // Verify direct library `run_audit_code` resolves relative `args.path` against `workspace_root` hermetically (#691)
    let pre_lib_dossiers = collect_defect_dossiers(&defects_dir);
    for file_name in pre_lib_dossiers.keys() {
        let rm_res = std::fs::remove_file(defects_dir.join(file_name));
        assert!(
            rm_res.is_ok(),
            "Failed to clear dossier {file_name} before direct run_audit_code probe: {rm_res:?}"
        );
    }
    let direct_lib_res = verify_baseline::audit_code::run_audit_code(
        &ws.root,
        &verify_baseline::audit_code::AuditCodeArgs {
            path: std::path::PathBuf::from("crates/sample_crate"),
            pillar: None,
            issue: None,
            repo: None,
            publish: false,
            dry_run: true,
        },
    );
    assert!(
        direct_lib_res.is_ok(),
        "Expected direct run_audit_code with relative path against &ws.root to succeed, got: {direct_lib_res:?}"
    );
    let lib_dossiers = collect_defect_dossiers(&defects_dir);
    assert!(
        !lib_dossiers.is_empty(),
        "Expected direct run_audit_code with relative path to persist dossiers under ws.root/.pipeline/defects"
    );

    // Negative CLI probe 1 (AC-04): non-existent --path exits Some(1)
    let missing_path = ws.root.join("non_existent_audit_target");
    let missing_str = missing_path.to_str().unwrap_or("");
    let neg_missing_res = Command::new(bin)
        .args(["audit-code", "--path", missing_str, "--dry-run"])
        .output();
    assert!(
        neg_missing_res.is_ok(),
        "Failed to spawn negative missing-path probe"
    );
    if let Ok(neg_missing_out) = neg_missing_res {
        assert_eq!(
            neg_missing_out.status.code(),
            Some(1),
            "Expected non-existent --path to exit Some(1), got {:?}",
            neg_missing_out.status.code()
        );
    }

    // Negative CLI probe 2 (AC-04): unbalanced-brace .rs file exits Some(1)
    let broken_ws = TempWorkspace::new("req_0606_unbalanced_brace");
    let broken_defects = broken_ws.root.join(".pipeline").join("defects");
    assert!(std::fs::create_dir_all(&broken_defects).is_ok());
    let broken_rs = "pub fn malformed_block() {\n    if true {\n";
    broken_ws.write_file("crates/broken_crate/src/lib.rs", broken_rs);
    let broken_ws_str = broken_ws.root.to_str().unwrap_or("");
    let neg_brace_res = Command::new(bin)
        .args(["audit-code", "--path", broken_ws_str, "--dry-run"])
        .output();
    assert!(
        neg_brace_res.is_ok(),
        "Failed to spawn negative unbalanced-brace probe"
    );
    if let Ok(neg_brace_out) = neg_brace_res {
        assert_eq!(
            neg_brace_out.status.code(),
            Some(1),
            "Expected unbalanced-brace .rs file to exit Some(1), got {:?}",
            neg_brace_out.status.code()
        );
    }

    // Negative CLI probe 3 (AC-04): --pillar invalid-pillar exits Some(2)
    let neg_pillar_res = Command::new(bin)
        .args([
            "audit-code",
            "--path",
            ws_root_str,
            "--pillar",
            "invalid-pillar",
            "--dry-run",
        ])
        .output();
    assert!(
        neg_pillar_res.is_ok(),
        "Failed to spawn negative invalid-pillar probe"
    );
    if let Ok(neg_pillar_out) = neg_pillar_res {
        assert_eq!(
            neg_pillar_out.status.code(),
            Some(2),
            "Expected --pillar invalid-pillar to exit Some(2), got {:?}",
            neg_pillar_out.status.code()
        );
    }
}

/// Verifies `REQ-0606` `AC-05` and `AC-06`: `--dry-run` offline isolation under an empty `PATH`
/// environment (guaranteeing zero `gh` subprocess spawning) and end-to-end 12-check blueprint gate
/// verification on `docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md`.
///
/// Realises: [REQ-0606/AC-05], [REQ-0606/AC-06]
#[test]
fn req_0606_ac05_and_ac06_dry_run_offline_isolation_and_bp06_blueprint_gate() {
    use std::process::Command;

    let clean_ws = TempWorkspace::new("req_0606_clean_offline");
    assert!(std::fs::create_dir_all(clean_ws.root.join(".pipeline").join("defects")).is_ok());
    clean_ws.write_file(
        "Cargo.toml",
        "[workspace]\nmembers = [\"crates/clean_sample\"]\n",
    );
    clean_ws.write_file(
        "crates/clean_sample/Cargo.toml",
        "[package]\nname = \"clean_sample\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );

    let clean_rust = r#"/// Computes saturating addition of two bounded counters.
///
/// Realises: [REQ-0606/AC-05]
pub fn add_bounded(lhs: u32, rhs: u32) -> u32 {
    lhs.saturating_add(rhs)
}

/// Verifies `add_bounded` nominal saturating sum.
#[test]
fn verify_add_bounded() {
    assert_eq!(add_bounded(2, 3), 5);
}
"#;
    clean_ws.write_file("crates/clean_sample/src/lib.rs", clean_rust);

    let bin = env!("CARGO_BIN_EXE_verify-baseline");
    let clean_ws_str = clean_ws.root.to_str().unwrap_or("");
    assert!(!clean_ws_str.is_empty());

    // AC-05: Run with PATH="" to prove zero `gh` subprocess spawning occurs under --dry-run
    let offline_res = Command::new(bin)
        .args(["audit-code", "--path", clean_ws_str, "--dry-run"])
        .env("PATH", "")
        .output();
    assert!(
        offline_res.is_ok(),
        "Failed to invoke audit-code under empty PATH: {offline_res:?}"
    );
    let Ok(offline_out) = offline_res else {
        return;
    };
    assert_eq!(
        offline_out.status.code(),
        Some(0),
        "Expected audit-code --dry-run with PATH=\"\" to exit Some(0), got {:?}; stderr:\n{}",
        offline_out.status.code(),
        String::from_utf8_lossy(&offline_out.stderr)
    );

    // AC-06: Verify BP-06 passes all 12 blueprint gate checks from the repository root
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(Path::parent)
        .unwrap_or(manifest_dir.as_path());
    let bp06_res = Command::new(bin)
        .current_dir(repo_root)
        .args([
            "check-blueprint",
            "docs/designs/blueprint-06-deterministic-static-defect-dossier-auditor.md",
            "--dry-run",
        ])
        .output();
    assert!(
        bp06_res.is_ok(),
        "Failed to invoke check-blueprint on blueprint-06: {bp06_res:?}"
    );
    if let Ok(bp06_out) = bp06_res {
        assert_eq!(
            bp06_out.status.code(),
            Some(0),
            "Expected blueprint-06 check-blueprint --dry-run to exit Some(0), got {:?}; stdout:\n{}\nstderr:\n{}",
            bp06_out.status.code(),
            String::from_utf8_lossy(&bp06_out.stdout),
            String::from_utf8_lossy(&bp06_out.stderr)
        );
    }
}

/// Verifies Check 37 (`verify_baseline::checks::structural::check_blueprint_grounding`) fails closed
/// when a `docs/designs/blueprint-*.md` carries `| **Status** | \`Approved\` |` while referencing a
/// non-existent `crates/...` path, permits `Draft` blueprints to propose unimplemented paths, and
/// passes once the referenced `crates/...` module exists on disk.
///
/// Realises: [REQ-CHECK-37-APPROVED-BLUEPRINT-GROUNDING], [REQ-CHECK-37-APPROVED-GROUNDING]
#[test]
fn check_37_rejects_approved_design_blueprint_with_nonexistent_crates_path() {
    use verify_baseline::checks::structural::check_blueprint_grounding;

    let ws = TempWorkspace::new("check_37_approved_bp");
    ws.write_file(
        "crates/verify-baseline/src/lib.rs",
        "// existing crate root\n",
    );

    let approved_bp = r#"| Attribute | Specification Detail |
| :--- | :--- |
| **Blueprint ID** | `BP-99-UNIMPLEMENTED` |
| **Requirement ID** | `REQ-BP-099` |
| **Title** | Approved Blueprint Grounding Probe |
| **Status** | `Approved` |
| **Target Crates** | `verify-baseline` |
| **Tracked Issues** | `#691` |
| **Generation Mode** | `subagent` |
| **Specification Source** | `crates/verify-baseline/src/lib.rs` |

## 1. Executive Summary & Derivation-Annotation Contract

Anchored to `crates/verify-baseline/src/lib.rs`.

## 6. Proposed Deterministic Implementation Contract

Target module path: `crates/verify-baseline/src/missing_unimplemented_module.rs`.

## 7. Source References

- `crates/verify-baseline/src/lib.rs`
"#;
    ws.write_file("docs/designs/blueprint-99-unimplemented.md", approved_bp);

    // 1. Approved blueprint citing non-existent crates/ module MUST fail Check 37
    let approved_missing_res = check_blueprint_grounding(&ws.root);
    assert!(
        approved_missing_res.is_err(),
        "Check 37 must reject Approved design blueprint referencing non-existent crates/ path, got Ok(())"
    );
    if let Err(errs) = &approved_missing_res {
        let formatted_errs = format!("{errs:?}");
        assert!(
            formatted_errs.contains("missing_unimplemented_module.rs"),
            "Expected Check 37 error output to cite missing_unimplemented_module.rs, got: {formatted_errs}"
        );
    }

    // 2. Changing Status from Approved to Draft allows proposed unimplemented paths
    let draft_bp = approved_bp.replace("| **Status** | `Approved` |", "| **Status** | `Draft` |");
    ws.write_file("docs/designs/blueprint-99-unimplemented.md", &draft_bp);
    let draft_res = check_blueprint_grounding(&ws.root);
    assert!(
        draft_res.is_ok(),
        "Expected Draft design blueprint with proposed module path to return Ok(()), got: {draft_res:?}"
    );

    // 3. Keeping Status as Approved after creating the referenced module returns Ok(())
    ws.write_file("docs/designs/blueprint-99-unimplemented.md", approved_bp);
    ws.write_file(
        "crates/verify-baseline/src/missing_unimplemented_module.rs",
        "// implemented module\n",
    );
    let approved_present_res = check_blueprint_grounding(&ws.root);
    assert!(
        approved_present_res.is_ok(),
        "Expected Approved design blueprint with existing crates/ module to return Ok(()), got: {approved_present_res:?}"
    );
}

/// Verifies `resolve_module_rs_files` resolves both a single `foo.rs` file and a decomposed
/// `foo/{mod.rs,sub.rs}` module directory in deterministic sorted order.
#[test]
fn test_resolve_module_rs_files_single_file_and_decomposed_directory() {
    let ws = TempWorkspace::new("resolve_mod_rs");
    ws.write_file("src/foo.rs", "// single file\n");
    let single = resolve_module_rs_files(&ws.root.join("src/foo.rs"));
    assert_eq!(single, vec![ws.root.join("src/foo.rs")]);

    let rm_res = std::fs::remove_file(ws.root.join("src/foo.rs"));
    assert!(rm_res.is_ok(), "Failed to remove src/foo.rs: {rm_res:?}");

    ws.write_file("src/foo/sub.rs", "// submodule\n");
    ws.write_file("src/foo/mod.rs", "// mod root\n");
    ws.write_file("src/foo/notes.txt", "ignored\n");
    let decomposed = resolve_module_rs_files(&ws.root.join("src/foo.rs"));
    assert_eq!(
        decomposed,
        vec![
            ws.root.join("src/foo/mod.rs"),
            ws.root.join("src/foo/sub.rs"),
        ]
    );
}
