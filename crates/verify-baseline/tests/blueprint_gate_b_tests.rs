//! Integration verification test suite for `verify_baseline::blueprint_gate` Checks 5 through 7 (Part B).

use std::path::PathBuf;
use verify_baseline::blueprint_gate::{
    check_3_sections, check_5_components, check_6_mermaid_syntax, check_7_latex_katex,
    BlueprintCliOptions, BlueprintReport,
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

## 5. Acceptance Criteria & Verification Matrix

<!-- test-target: crates/verify-baseline/tests/blueprint_gate_tests.rs -->
- **Test-Target**: `crates/verify-baseline/tests/blueprint_gate_tests.rs`

### AC-01: Nominal Frame Routing
- **Given**: A valid inbound telemetry frame on `rawTelemetryIn`
- **When**: `TelemetryRouter` processes the frame within the nominal period
- **Then**: A verified `ValidatedFrame` is emitted on `validatedOut` within 1 tick

## 6. Proposed Deterministic Implementation Contract

```rust
pub trait RetryContract {
    fn validate_retry_count(retries: u32, limit: u32) -> Result<u32, &'static str>;
}
```

## 7. Source References

- `Cargo.toml`
"
}

fn valid_blueprint_fixture() -> String {
    format!(
        "{}\n{}",
        valid_blueprint_fixture_part_a(),
        valid_blueprint_fixture_part_b()
    )
}

/// Verifies `check_5_components` parses Section 2 components, properties, and ports and rejects invalid identifiers, types, or directions.
#[test]
fn test_check_5_components_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(&valid).unwrap_or_default();
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
#[test]
fn test_check_6_mermaid_syntax_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(&valid).unwrap_or_default();
    let (s3_start, s3_end, _) = ranges.get(2).copied().unwrap_or((0, 0, ""));
    let sec3_lines = lines.get(s3_start..s3_end).unwrap_or(&[]);

    let res = check_6_mermaid_syntax(sec3_lines);
    assert_eq!(res, Ok(2));

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

    let no_mermaid = [
        "## 3. UML & Behavioral Topology",
        "No diagrams provided here.",
    ];
    assert!(check_6_mermaid_syntax(&no_mermaid).is_err());

    let bad_header = [
        "## 3. UML & Behavioral Topology",
        "```mermaid",
        "pie title Invalid Diagram Type",
        "    \"Alpha\" : 40",
        "```",
    ];
    assert!(check_6_mermaid_syntax(&bad_header).is_err());

    let unquoted_angle = [
        "## 3. UML & Behavioral Topology",
        "```mermaid",
        "flowchart TD",
        "    A[Value < 10] --> B[Done]",
        "```",
    ];
    assert!(check_6_mermaid_syntax(&unquoted_angle).is_err());

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

    let leaking_fence = [
        "## 3. UML & Behavioral Topology",
        "```mermaid",
        "stateDiagram-v2",
        "    [*] --> Idle",
    ];
    assert!(check_6_mermaid_syntax(&leaking_fence).is_err());
}

/// Verifies `check_7_latex_katex` accepts isolated `$$` `\begin{aligned}...\end{aligned}` blocks and rejects `\begin{align}`, inline `$$`, or `$REQ-*$` IDs.
#[test]
fn test_check_7_latex_katex_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(&valid).unwrap_or_default();
    let (s4_start, s4_end, _) = ranges.get(3).copied().unwrap_or((0, 0, ""));
    let sec4_lines = lines.get(s4_start..s4_end).unwrap_or(&[]);

    let res = check_7_latex_katex(sec4_lines, &valid);
    assert_eq!(res, Ok(1));

    let no_aligned = [
        "## 4. Formal Mathematical & Safety Constraints",
        "Plain prose without display math.",
    ];
    assert!(check_7_latex_katex(&no_aligned, &no_aligned.join("\n")).is_err());

    let bad_align = [
        "## 4. Formal Mathematical & Safety Constraints",
        "$$",
        "\\begin{align}",
        "x &= 1",
        "\\end{align}",
        "$$",
    ];
    assert!(check_7_latex_katex(&bad_align, &bad_align.join("\n")).is_err());

    let non_isolated_dd = [
        "## 4. Formal Mathematical & Safety Constraints",
        "$$ \\begin{aligned} x &= 1 \\end{aligned} $$",
    ];
    assert!(check_7_latex_katex(&non_isolated_dd, &non_isolated_dd.join("\n")).is_err());

    let inline_req_id = format!("{valid}\nIn prose we forbid $REQ-0601$ and $H-1$ and $SC-01$.\n");
    assert!(check_7_latex_katex(sec4_lines, &inline_req_id).is_err());

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
