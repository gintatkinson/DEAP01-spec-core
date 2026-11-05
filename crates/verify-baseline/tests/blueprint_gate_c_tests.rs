//! Integration verification test suite for `verify_baseline::blueprint_gate` Checks 8, 9, and 12 (Part C).

use std::path::PathBuf;
use verify_baseline::blueprint_gate::{
    check_12_publish_github_ssot, check_2_metadata_table, check_3_sections,
    check_8_bdd_and_test_target, check_9_rust_code_blocks, verify_closed_loop_github_payload,
    BlueprintCliOptions,
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

| Claim / Requirement | SSOT Input Anchor (file:line / #ID) | Methodology Citation | Derivation Grade | Caveat / Downstream Impact |
| :--- | :--- | :--- | :--- | :--- |
| Frame rate bounded | `schema/system.sysml:14` | DO-178C | Directly Evidenced | Bounded buffer |

## 2. Structural Component Metamodel

### Component: TelemetryRouter

| Attribute | Type | Default | Unit | Description |
| :--- | :--- | :--- | :--- | :--- |
| `maxRetries` | `Integer` | `3` | `count` | Maximum packet retransmission attempts |

| Port | Direction | Type | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| `rawTelemetryIn` | `in` | `TelemetryFrame` | `ARINC429` | Inbound sensor frame stream |

## 3. UML & Behavioral Topology

```mermaid
stateDiagram-v2
    [*] --> Idle
```

## 4. Formal Mathematical & Safety Constraints

$$
\\begin{aligned}
T_{bound} &= N \\cdot \\Delta t
\\end{aligned}
$$
"
}

fn valid_blueprint_fixture_part_b() -> &'static str {
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

/// Verifies `check_8_bdd_and_test_target` enforces `>= 5` Given/When/Then `AC-XX` scenarios and rejects missing `Then` clauses.
#[test]
fn test_check_8_bdd_and_test_target_pass_and_ac_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(&valid).unwrap_or_default();
    let (s5_start, s5_end, _) = ranges.get(4).copied().unwrap_or((0, 0, ""));
    let sec5_lines = lines.get(s5_start..s5_end).unwrap_or(&[]);

    let res = check_8_bdd_and_test_target(sec5_lines);
    assert_eq!(
        res,
        Ok((
            5,
            vec!["crates/verify-baseline/tests/blueprint_gate_tests.rs".to_string()]
        ))
    );

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

    let missing_then_str = valid.replace(
        "- **Then**: A `DiagnosticPacket` reflecting the exact retry count is returned",
        "- **Result**: Packet returned",
    );
    let m2_lines: Vec<&str> = missing_then_str.lines().collect();
    let m2_sec5 = m2_lines.get(s5_start..s5_end).unwrap_or(&[]);
    assert!(check_8_bdd_and_test_target(m2_sec5).is_err());
}

/// Verifies `check_8_bdd_and_test_target` enforces `>= 1` negative scenario and grounded `test-target` declarations.
#[test]
fn test_check_8_bdd_and_test_target_negative_and_target_mutations() {
    let valid = valid_blueprint_fixture();
    let ranges = check_3_sections(&valid).unwrap_or_default();
    let (s5_start, s5_end, _) = ranges.get(4).copied().unwrap_or((0, 0, ""));

    let no_negative = [
        "## 5. Acceptance Criteria & Verification Matrix",
        "<!-- test-target: crates/deap-core/tests/core_test.rs -->",
        "- **Test-Target**: `crates/deap-core/tests/core_test.rs`",
        "### AC-01: Nominal A\n- **Given**: State A\n- **When**: Event A\n- **Then**: Output A",
        "### AC-02: Nominal B\n- **Given**: State B\n- **When**: Event B\n- **Then**: Output B",
        "### AC-03: Nominal C\n- **Given**: State C\n- **When**: Event C\n- **Then**: Output C",
        "### AC-04: Nominal D\n- **Given**: State D\n- **When**: Event D\n- **Then**: Output D",
        "### AC-05: Nominal E\n- **Given**: State E\n- **When**: Event E\n- **Then**: Output E",
    ];
    let no_neg_joined = no_negative.join("\n");
    let no_neg_lines: Vec<&str> = no_neg_joined.lines().collect();
    assert!(check_8_bdd_and_test_target(&no_neg_lines).is_err());

    let mismatched_target = valid.replace(
        "- **Test-Target**: `crates/verify-baseline/tests/blueprint_gate_tests.rs`",
        "- **Test-Target**: `crates/deap-core/tests/other_tests.rs`",
    );
    let m4_lines: Vec<&str> = mismatched_target.lines().collect();
    let m4_sec5 = m4_lines.get(s5_start..s5_end).unwrap_or(&[]);
    assert!(check_8_bdd_and_test_target(m4_sec5).is_err());

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

/// Verifies `check_8_bdd_and_test_target` reconciles multiple test-target declarations and rejects count/content mismatches.
#[test]
fn test_check_8_multi_target_reconciliation() {
    let valid = valid_blueprint_fixture();
    let second_pair = "<!-- test-target: crates/deap-core/tests/core_test.rs -->\n- **Test-Target**: `crates/deap-core/tests/core_test.rs`";
    let multi = valid.replace(
        "- **Test-Target**: `crates/verify-baseline/tests/blueprint_gate_tests.rs`",
        &format!(
            "- **Test-Target**: `crates/verify-baseline/tests/blueprint_gate_tests.rs`\n{second_pair}"
        ),
    );
    let ranges = check_3_sections(&multi).unwrap_or_default();
    let (s5_start, s5_end, _) = ranges.get(4).copied().unwrap_or((0, 0, ""));
    let lines: Vec<&str> = multi.lines().collect();
    let sec5 = lines.get(s5_start..s5_end).unwrap_or(&[]);
    assert_eq!(
        check_8_bdd_and_test_target(sec5),
        Ok((
            5,
            vec![
                "crates/verify-baseline/tests/blueprint_gate_tests.rs".to_string(),
                "crates/deap-core/tests/core_test.rs".to_string(),
            ]
        ))
    );

    let mismatched_count = multi.replace(
        "- **Test-Target**: `crates/deap-core/tests/core_test.rs`",
        "",
    );
    let count_ranges = check_3_sections(&mismatched_count).unwrap_or_default();
    let (c5_start, c5_end, _) = count_ranges.get(4).copied().unwrap_or((0, 0, ""));
    let count_lines: Vec<&str> = mismatched_count.lines().collect();
    assert!(
        check_8_bdd_and_test_target(count_lines.get(c5_start..c5_end).unwrap_or(&[])).is_err()
    );

    let mismatched_content = multi.replace(
        "<!-- test-target: crates/deap-core/tests/core_test.rs -->",
        "<!-- test-target: crates/compile-sysml/tests/other_test.rs -->",
    );
    let content_ranges = check_3_sections(&mismatched_content).unwrap_or_default();
    let (m5_start, m5_end, _) = content_ranges.get(4).copied().unwrap_or((0, 0, ""));
    let content_lines: Vec<&str> = mismatched_content.lines().collect();
    assert!(
        check_8_bdd_and_test_target(content_lines.get(m5_start..m5_end).unwrap_or(&[])).is_err()
    );
}

/// Verifies `check_9_rust_code_blocks` accepts declaration-only Rust blocks and rejects executable items.
#[test]
fn test_check_9_declaration_only_contract() {
    let valid = [
        "## 6. Proposed Deterministic Implementation Contract",
        "```rust",
        "use std::collections::BTreeMap;",
        "",
        "pub const MAX_RETRIES: u32 = 3;",
        "",
        "pub struct RouterConfig {",
        "    pub max_retries: u32,",
        "}",
        "",
        "pub enum RouterState {",
        "    Idle,",
        "    FaultLocked,",
        "}",
        "",
        "pub union NumericValue {",
        "    integer: u32,",
        "    real: f64,",
        "}",
        "",
        "pub type RouterResult = Result<u32, &'static str>;",
        "",
        "pub static DEFAULT_CONFIG: RouterConfig = RouterConfig { max_retries: MAX_RETRIES };",
        "",
        "pub trait RetryContract {",
        "    fn validate_retry_count(retries: u32, limit: u32) -> RouterResult;",
        "}",
        "",
        "mod nested {",
        "    pub trait NestedContract {",
        "        fn nested_check(value: u32) -> bool;",
        "    }",
        "}",
        "",
        "extern \"C\" {",
        "    pub fn external_check(value: u32) -> u32;",
        "}",
        "```",
    ];
    assert_eq!(check_9_rust_code_blocks(&valid), Ok(1));

    let fn_block = [
        "## 6. Proposed Deterministic Implementation Contract",
        "```rust",
        "pub fn bad() {}",
        "```",
    ];
    assert!(check_9_rust_code_blocks(&fn_block).is_err());

    let impl_block = [
        "## 6. Proposed Deterministic Implementation Contract",
        "```rust",
        "pub struct Foo;",
        "impl Foo {}",
        "```",
    ];
    assert!(check_9_rust_code_blocks(&impl_block).is_err());

    let trait_default_block = [
        "## 6. Proposed Deterministic Implementation Contract",
        "```rust",
        "pub trait BadTrait {",
        "    fn bad_default(&self) {}",
        "}",
        "```",
    ];
    assert!(check_9_rust_code_blocks(&trait_default_block).is_err());
}

/// Verifies `check_9_rust_code_blocks` accepts clean Section 6 Rust blocks and rejects missing blocks, regex usage, panicking/indexing calls, or domain tokens.
#[test]
fn test_check_9_rust_code_blocks_pass_and_mutations() {
    let valid = valid_blueprint_fixture();
    let lines: Vec<&str> = valid.lines().collect();
    let ranges = check_3_sections(&valid).unwrap_or_default();
    let (s6_start, s6_end, _) = ranges.get(5).copied().unwrap_or((0, 0, ""));
    let sec6_lines = lines.get(s6_start..s6_end).unwrap_or(&[]);

    let res = check_9_rust_code_blocks(sec6_lines);
    assert_eq!(res, Ok(1));

    let no_rust = [
        "## 6. Proposed Deterministic Implementation Contract",
        "No code block here.",
    ];
    assert!(check_9_rust_code_blocks(&no_rust).is_err());

    let block_with_regex =
        "## 6. Proposed Deterministic Implementation Contract\n```rust\npub fn bad() {\n    let _r = Regex::new(\"abc\");\n}\n```";
    let m2_lines: Vec<&str> = block_with_regex.lines().collect();
    assert!(check_9_rust_code_blocks(&m2_lines).is_err());

    let block_with_unwrap =
        "## 6. Proposed Deterministic Implementation Contract\n```rust\npub fn bad(opt: Option<u32>) -> u32 {\n    opt.unwrap()\n}\n```";
    let m3_lines: Vec<&str> = block_with_unwrap.lines().collect();
    assert!(check_9_rust_code_blocks(&m3_lines).is_err());

    let block_with_slice =
        "## 6. Proposed Deterministic Implementation Contract\n```rust\npub fn bad(s: &str) -> &str {\n    &s[0..4]\n}\n```";
    let m4_lines: Vec<&str> = block_with_slice.lines().collect();
    assert!(check_9_rust_code_blocks(&m4_lines).is_err());

    let block_with_domain =
        "## 6. Proposed Deterministic Implementation Contract\n```rust\npub const BAD: &str = \"FLIGHT_ENVELOPE_MAP\";\n```";
    let m5_lines: Vec<&str> = block_with_domain.lines().collect();
    assert!(check_9_rust_code_blocks(&m5_lines).is_err());
}

/// Verifies Check 12 (`verify_closed_loop_github_payload` and `check_12_publish_github_ssot`) passes valid payloads and rejects mismatches.
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

    assert!(
        verify_closed_loop_github_payload(valid_json, "Wrong Blueprint Title", "REQ-BP-001")
            .is_err()
    );
    assert!(verify_closed_loop_github_payload(
        valid_json,
        "Deterministic Telemetry Gate Blueprint",
        "REQ-BP-999-MISSING"
    )
    .is_err());

    let empty_body_json = r#"{
        "number": 501,
        "title": "Deterministic Telemetry Gate Blueprint",
        "body": "   ",
        "url": "https://github.com/gintatkinson/DEAP01-spec-core/issues/501"
    }"#;
    assert!(verify_closed_loop_github_payload(
        empty_body_json,
        "Deterministic Telemetry Gate Blueprint",
        "REQ-BP-001"
    )
    .is_err());

    let opts = BlueprintCliOptions {
        path: PathBuf::from("docs/designs/blueprint-01-telemetry.md"),
        repo: Some("gintatkinson/DEAP01-spec-core".to_string()),
        issue: Some(501),
        title: None,
        publish: true,
        dry_run: true,
    };
    let valid = valid_blueprint_fixture();
    let meta = check_2_metadata_table(&valid).unwrap_or_default();
    assert!(check_12_publish_github_ssot(&opts, &meta, &valid).is_ok());
}
