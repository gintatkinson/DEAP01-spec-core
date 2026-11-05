//! Integration verification test suite for `verify_baseline::blueprint_gate` Checks 10, 11, and E2E orchestrator (Part D).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use verify_baseline::blueprint_gate::{
    check_10_compile_and_persist_sysml, check_11_source_references, check_3_sections,
    run_blueprint_cli, verify_blueprint, BlueprintCliOptions,
};

const MAX_MODULE_WALK_DEPTH: usize = 16;

/// Resolves a target `.rs` file or its decomposed `<stem>/` module directory into sorted `.rs` paths.
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

| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade | Caveat / Downstream Impact |
| :--- | :--- | :--- | :--- | :--- |
| Frame rate bounded by $\\|x\\| \\le 100$ | `schema/system.sysml:14` | DO-178C Section 6.3.1 | **Directly Evidenced** | Requires deterministic buffer sizing |

## 2. Structural Component Metamodel

### Component: TelemetryRouter

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `maxRetries` | `Integer` | `3` | `count` | Maximum packet retransmission attempts |
| `isRedundant` | `Boolean` | `true` | `dimensionless` | Enables dual-channel failover routing |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `rawTelemetryIn` | `in` | `TelemetryFrame` | `ARINC429` | Inbound sensor frame stream |
| `validatedOut` | **out** | `ValidatedFrame` | `DeterministicBus` | Verified outbound frame stream |

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

fn valid_blueprint_fixture() -> String {
    format!(
        "{}\n{}",
        valid_blueprint_fixture_part_a(),
        valid_blueprint_fixture_part_b()
    )
}

/// Verifies `check_11_source_references` resolves workspace file references and rejects phantom files.
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
        let src_text = std::fs::read_to_string(file_path).unwrap_or_default();
        assert!(!src_text.trim().is_empty());
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

    let no_local_refs = [
        "## 7. Source References",
        "- External standard: https://www.rust-lang.org",
        "- Issue tracker: `#501`",
    ];
    assert!(check_11_source_references(workspace_root, &no_local_refs).is_err());

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

/// Verifies Check 10 compiles blueprint Markdown into bijective SysML v2 and persists `<stem>.sysml` and `<stem>.digest.json`.
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
    std::fs::write(&bp_path, &fixture).unwrap_or(());

    assert!(check_10_compile_and_persist_sysml(&bp_path, &fixture).is_ok());

    let opts = BlueprintCliOptions {
        path: PathBuf::from("docs/designs/blueprint-99-sample.md"),
        repo: None,
        issue: None,
        title: None,
        publish: false,
        dry_run: true,
    };

    let report = verify_blueprint(&temp_ws, &opts).unwrap_or_default();
    let expected_sysml = designs_dir.join("blueprint-99-sample.sysml");
    let expected_digest = designs_dir.join("blueprint-99-sample.digest.json");
    assert_eq!(report.sysml_path, expected_sysml);
    assert_eq!(report.digest_path, expected_digest);

    let sysml_text = std::fs::read_to_string(&expected_sysml).unwrap_or_default();
    let digest_text = std::fs::read_to_string(&expected_digest).unwrap_or_default();
    assert!(!sysml_text.trim().is_empty() && !digest_text.trim().is_empty());
    assert!(sysml_text.contains("package ") && sysml_text.contains("requirement def "));
    assert!(sysml_text.contains("part def ") && sysml_text.contains("constraint def "));

    if let Ok(pkg) = compile_sysml::parse_sysml(&sysml_text) {
        assert_eq!(compile_sysml::to_sysml(&pkg), sysml_text);
    }

    let incomplete_bp = designs_dir.join("blueprint-98-incomplete.md");
    assert!(check_10_compile_and_persist_sysml(
        &incomplete_bp,
        "# EmptyDoc\n\nNo requirements or parts.\n"
    )
    .is_err());

    std::fs::remove_dir_all(&temp_ws).unwrap_or(());
}

/// Verifies end-to-end execution of `verify_blueprint` and `run_blueprint_cli` on passing and failing blueprint files.
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
    assert_eq!(run_blueprint_cli(&temp_ws, &pass_opts), 0);

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
    assert!(verify_blueprint(&temp_ws, &fail_opts).is_err());
    assert_eq!(run_blueprint_cli(&temp_ws, &fail_opts), 1);
    assert!(!designs_dir.join("blueprint-02-invalid.sysml").exists());

    std::fs::remove_dir_all(&temp_ws).unwrap_or(());
}
