//! Integration verification test suite for multi-component blueprints and `REQ-0604` `AC-01`, `AC-04`, and `AC-05` (Part E).

use std::path::PathBuf;
use verify_baseline::blueprint_gate::{verify_blueprint, BlueprintCliOptions};

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

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `rawTelemetryIn` | `in` | `TelemetryFrame` | `ARINC429` | Inbound sensor frame stream |
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
pub trait RetryContract {
    fn validate_retry_count(retries: u32, limit: u32) -> Result<u32, &'static str>;
}
```
## 7. Source References
- `crates/verify-baseline/src/blueprint_gate.rs:1`
- [Blueprint Gate Tests](crates/verify-baseline/tests/blueprint_gate_tests.rs#L1-L50)
- `Cargo.toml`
- External reference: https://www.rust-lang.org and tracker issue `#501`
"
}

fn multi_comp_sec2_fixture() -> &'static str {
    r#"## 2. Structural Component Metamodel

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
"#
}

/// Verifies multi-component blueprints with PascalCase classifier and primitive types, H3 subheadings, and `--dry-run` SysML persistence.
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

    let base_fixture = format!(
        "{}\n{}",
        valid_blueprint_fixture_part_a(),
        valid_blueprint_fixture_part_b()
    );
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
    let multi_bp_md = format!("{prefix}{}{suffix}", multi_comp_sec2_fixture());

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

    let report = verify_blueprint(&temp_ws, &opts).unwrap_or_default();
    assert_eq!(report.components.len(), 2);
    assert!(report.sysml_path.exists() && report.digest_path.exists());
    let sysml_text = std::fs::read_to_string(&report.sysml_path).unwrap_or_default();
    assert!(sysml_text.contains("part def BlueprintGateEngine"));
    assert!(sysml_text.contains("part def ConstraintSpec"));
    assert!(!sysml_text.contains("part def Structural_Class_Diagram"));
    assert!(
        sysml_text.contains("attribute active_constraint_spec : String = \"StrictSpec\"")
            || sysml_text
                .contains("attribute active_constraint_spec : ConstraintSpec = \"StrictSpec\"")
    );
    assert!(
        sysml_text.contains("attribute boolean_policy_mode : String = \"FailClosed\"")
            || sysml_text
                .contains("attribute boolean_policy_mode : BooleanPolicy = \"FailClosed\"")
    );

    std::fs::remove_dir_all(&temp_ws).unwrap_or(());
}

/// Verifies `REQ-0604` `AC-01` and `AC-05`: parameterized `ActionDef` Mermaid/Markdown operation signatures and reverse bullet parsing.
#[test]
fn test_req_0604_ac01_and_ac05_operation_signatures_and_reverse_parsing() {
    use compile_sysml::sync::forward::{
        format_markdown_operation_bullet, format_mermaid_operation_signature,
        forward_sync_sysml_to_specs, parse_markdown_operation_bullet, ForwardSyncOptions,
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
    assert_eq!(
        parse_markdown_operation_bullet(&md_bullet),
        Some(action.clone())
    );
    assert_eq!(
        parse_markdown_operation_bullet("- `ComputeTrajectory()`"),
        None
    );
    assert_eq!(parse_markdown_operation_bullet("- `+()`"), None);
    assert_eq!(parse_markdown_operation_bullet("plain text line"), None);

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

    let mut root_model = PackageDef {
        name: "SystemRoot".to_string(),
        ..Default::default()
    };
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
    assert!(feat_md.contains(&mermaid_sig) && feat_md.contains(&md_bullet));
}
