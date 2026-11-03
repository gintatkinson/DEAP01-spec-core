//! Integration verification test suite for `verify_baseline::blueprint_gate` Checks 1 through 4 (Part A).
//!
//! Safety & Traceability: Realises: [REQ-BLUEPRINT-GATE-TESTS-1-5], [DO-330-TQL-4-BLUEPRINT-TESTS]

use std::path::{Path, PathBuf};
use verify_baseline::blueprint_gate::{
    check_1_zero_ephemeral_path, check_2_metadata_table, check_3_sections,
    check_4_derivation_table, BlueprintMetadata,
};

fn valid_blueprint_fixture_part_a() -> &'static str {
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
"
}

fn valid_blueprint_fixture_part_b() -> &'static str {
    "\
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
"
}

fn valid_blueprint_fixture_part_c() -> &'static str {
    "\
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

fn valid_blueprint_fixture() -> String {
    format!(
        "{}\n{}\n{}",
        valid_blueprint_fixture_part_a(),
        valid_blueprint_fixture_part_b(),
        valid_blueprint_fixture_part_c()
    )
}

/// Verifies that `check_1_zero_ephemeral_path` accepts valid relative and absolute paths under `<workspace_root>/docs/designs/blueprint-*.md`.
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
#[test]
fn test_check_2_metadata_table_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let res = check_2_metadata_table(&valid);
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

    let bad_mode = valid.replace(
        "| **Generation Mode** | `subagent` |",
        "| **Generation Mode** | `manual` |",
    );
    assert!(check_2_metadata_table(&bad_mode).is_err());

    let bad_req = valid.replace(
        "| **Requirement ID** | `REQ-BP-001` |",
        "| **Requirement ID** | `BP-001` |",
    );
    assert!(check_2_metadata_table(&bad_req).is_err());

    let missing_source = valid.replace(
        "| **Specification Source** | `docs/features/feat-01.md` |\n",
        "",
    );
    assert!(check_2_metadata_table(&missing_source).is_err());

    let empty_title = valid.replace(
        "| **Title** | Deterministic Telemetry Gate Blueprint |",
        "| **Title** |   |",
    );
    assert!(check_2_metadata_table(&empty_title).is_err());

    let pushed_down = format!("\n\n\n\n\n\n{valid}");
    assert!(check_2_metadata_table(&pushed_down).is_err());
}

/// Verifies `check_3_sections` validates the exact 7-section topology and rejects missing, duplicated, or out-of-order headers.
#[test]
fn test_check_3_sections_order_and_completeness() {
    let valid = valid_blueprint_fixture();
    let res = check_3_sections(&valid);
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

    let missing_sec4 = valid.replace(
        "## 4. Formal Mathematical & Safety Constraints",
        "## 4. Invalid Header Name",
    );
    assert!(check_3_sections(&missing_sec4).is_err());

    let out_of_order = valid
        .replace("## 2. Structural Component Metamodel", "__TEMP_SEC_2__")
        .replace(
            "## 3. UML & Behavioral Topology",
            "## 2. Structural Component Metamodel",
        )
        .replace("__TEMP_SEC_2__", "## 3. UML & Behavioral Topology");
    assert!(check_3_sections(&out_of_order).is_err());

    let duplicate_sec5 = format!("{valid}\n## 5. Acceptance Criteria & Verification Matrix\n");
    assert!(check_3_sections(&duplicate_sec5).is_err());
}

/// Verifies `check_4_derivation_table` parses 5-column derivation rows (including escaped pipes) and rejects invalid grades or empty cells.
#[test]
fn test_check_4_derivation_table_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(&valid).unwrap_or_default();
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

    let bad_grade = [
        "## 1. Executive Summary & Derivation-Annotation Contract",
        "| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade | Caveat / Downstream Impact |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| Claim 1 | `schema/a.sysml:1` | DO-178C | **Guessed Grade** | Impact 1 |",
    ];
    assert!(check_4_derivation_table(&bad_grade).is_err());

    let empty_cell = [
        "## 1. Executive Summary & Derivation-Annotation Contract",
        "| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade | Caveat / Downstream Impact |",
        "| :--- | :--- | :--- | :--- | :--- |",
        "| Claim 1 | - | DO-178C | Directly Evidenced | Impact 1 |",
    ];
    assert!(check_4_derivation_table(&empty_cell).is_err());

    let zero_rows = [
        "## 1. Executive Summary & Derivation-Annotation Contract",
        "| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade | Caveat / Downstream Impact |",
        "| :--- | :--- | :--- | :--- | :--- |",
    ];
    assert!(check_4_derivation_table(&zero_rows).is_err());
}

/// Verifies `BP-02` Check 34 CST purity, fail-closed I/O CST, and `cargo_manifest_ast` contracts (`[REQ-0602]`).
#[test]
fn bp02_check34_cst_purity_and_manifest_gate_contract() {
    assert!(verify_baseline::checks::cst_purity_gate::audit_rust_cst_purity("pub fn bad(s: &[u8], i: usize) -> u8 { s[i] }", "p.rs", true).is_err());
    assert!(verify_baseline::checks::io_fail_closed_gate::audit_io_fail_closed_cst("pub fn f(v: Vec<Result<u8, ()>>) { for _ in v.into_iter().flatten() {} }", "p.rs").is_err());
    assert!(deap_core::cargo_manifest_ast::ForbiddenAstPattern::RegexCrateImport.code() == "C34-REGEX-IMPORT");
}
