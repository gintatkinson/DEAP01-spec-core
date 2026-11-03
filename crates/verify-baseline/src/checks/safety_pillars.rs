//! STPA 8-Pillar and failure dimension validators for Check 17.
//! Realises: [REQ-CHECKS-SAFETY-STPA]

use std::path::Path;

use crate::checks::safety_fmeca::{audit_pillar_7_fmeca, FmecaData};

const INTERFACE_KEYWORDS: &[&str] = &[
    "interface", "port", "bus", "signal", "protocol", "packet", "message", "frame", "channel",
    "link", "network", "payload", "telemetry", "command", "sensor", "actuator", "input", "output",
    "io", "i/o", "rx", "tx", "uart", "spi", "i2c", "can", "ethernet", "api", "handshake",
    "checksum", "crc", "corrupt", "corruption", "dropout", "loss", "timeout", "noise", "spoof",
    "injection", "γ", "\\gamma", "gamma",
];

const STATE_KEYWORDS: &[&str] = &[
    "state", "mode", "transition", "deadlock", "latch", "phase", "statechart", "fsm", "sync",
    "synchronization", "desync", "drift", "stuck", "uninitialized", "freeze", "lockup", "trip",
    "abort", "corruption", "disagreement", "φ", "\\phi", "phi",
];

const ACTION_KEYWORDS: &[&str] = &[
    "action", "command", "execution", "operation", "control", "timing", "deadline", "compute",
    "calculation", "process", "logic", "omission", "commission", "latency", "jitter", "delay",
    "rate", "clamping", "limiter", "saturation", "step", "overshoot", "schedule", "task",
    "authority", "miss", "ω", "\\omega", "omega",
];

const RESOURCE_KEYWORDS: &[&str] = &[
    "resource", "memory", "cpu", "buffer", "power", "energy", "battery", "thermal", "heat",
    "overheat", "overload", "bandwidth", "storage", "capacity", "stack", "heap", "overflow",
    "underflow", "brownout", "voltage", "current", "load", "fault", "short", "sag", "circuit",
    "crowbar", "degradation", "flash", "ram", "supply", "undervoltage", "overvoltage", "seizure",
    "winding", "windings", "wiper", "hardware", "bearing", "dielectric", "squib", "fuse", "fusing",
    "ψ", "\\psi", "psi",
];

/// Checks if `lower` contains any keyword from `keywords` at word boundaries.
/// Realises: [REQ-CHECKS-SAFETY-STPA-MATCHES-DIMENSION-LOWER]
pub(crate) fn matches_dimension_lower(lower: &str, keywords: &[&str]) -> bool {
    for &kw in keywords {
        let kw_len = kw.len();
        let mut start = 0;
        while let Some(pos) = lower[start..].find(kw) {
            let actual_pos = start + pos;
            let end_pos = actual_pos + kw_len;
            let prev_ok = actual_pos == 0
                || lower
                    .get(..actual_pos)
                    .and_then(|s| s.chars().next_back())
                    .is_none_or(|prev_char| !prev_char.is_alphanumeric() && prev_char != '_');
            let next_ok = end_pos == lower.len()
                || lower
                    .get(end_pos..)
                    .and_then(|s| s.chars().next())
                    .is_none_or(|next_char| !next_char.is_alphanumeric() && next_char != '_');
            if prev_ok && next_ok {
                return true;
            }
            start = actual_pos + 1;
            while start < lower.len() && !lower.is_char_boundary(start) {
                start += 1;
            }
        }
    }
    false
}

/// Checks coverage of the 4 Universal Failure Dimensions across FMECA rows and document content.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-FAILURE-DIMENSION-COVERAGE]
pub fn check_failure_dimension_coverage(content: &str, fmeca: &FmecaData) -> Vec<String> {
    let mut found_interface = false;
    let mut found_state = false;
    let mut found_action = false;
    let mut found_resource = false;

    for row in &fmeca.rows {
        let combined = format!(
            "{} {} {}",
            row.failure_mode,
            row.component,
            row.basis.as_deref().unwrap_or("")
        )
        .to_lowercase();
        if !found_interface && matches_dimension_lower(&combined, INTERFACE_KEYWORDS) {
            found_interface = true;
        }
        if !found_state && matches_dimension_lower(&combined, STATE_KEYWORDS) {
            found_state = true;
        }
        if !found_action && matches_dimension_lower(&combined, ACTION_KEYWORDS) {
            found_action = true;
        }
        if !found_resource && matches_dimension_lower(&combined, RESOURCE_KEYWORDS) {
            found_resource = true;
        }
    }

    let content_lower = content.to_lowercase();
    if !found_interface && matches_dimension_lower(&content_lower, INTERFACE_KEYWORDS) {
        found_interface = true;
    }
    if !found_state && matches_dimension_lower(&content_lower, STATE_KEYWORDS) {
        found_state = true;
    }
    if !found_action && matches_dimension_lower(&content_lower, ACTION_KEYWORDS) {
        found_action = true;
    }
    if !found_resource && matches_dimension_lower(&content_lower, RESOURCE_KEYWORDS) {
        found_resource = true;
    }

    let mut missing = Vec::new();
    if !found_interface {
        missing.push("Interface (Γ)".to_string());
    }
    if !found_state {
        missing.push("State (Φ)".to_string());
    }
    if !found_action {
        missing.push("Action (Ω)".to_string());
    }
    if !found_resource {
        missing.push("Resource (Ψ)".to_string());
    }
    missing
}

/// Verifies that all 4 STPA UCA guide words / categories are covered in content.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-UCA-CATEGORIES]
pub fn check_uca_categories(content: &str) -> Vec<String> {
    let mut missing = Vec::new();
    let lower = content.to_lowercase();

    let has_not_providing = lower.contains("not providing")
        || lower.contains("not provided")
        || lower.contains("omission")
        || lower.split_whitespace().collect::<Vec<_>>().windows(2).any(|w| {
            w.first() == Some(&"not")
                && w.get(1).is_some_and(|w1| {
                    w1.starts_with("provid") || *w1 == "providing" || *w1 == "provided"
                })
        });
    if !has_not_providing {
        missing.push("1. Not providing causes hazard".to_string());
    }

    let has_providing_explicit = lower.contains("commission")
        || lower.contains("providing causes")
        || lower.contains("providing incorrectly");
    let has_providing = has_providing_explicit || has_standalone_providing(&lower);
    if !has_providing {
        missing.push("2. Providing causes hazard".to_string());
    }

    let has_timing = lower.contains("too early")
        || lower.contains("too late")
        || lower.contains("out of order")
        || lower.contains("timing")
        || lower.contains("early/late");
    if !has_timing {
        missing.push("3. Providing too early, too late, or out of order".to_string());
    }

    let has_duration = lower.contains("stopped too soon")
        || lower.contains("applied too long")
        || lower.contains("duration")
        || lower.contains("stopped early");
    if !has_duration {
        missing.push("4. Stopped too soon or applied too long".to_string());
    }

    missing
}

/// Checks whether `"providing"` appears as a standalone word not followed by `"too"`.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-UCA-CATEGORIES]
fn has_standalone_providing(lower: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = lower.get(start..).and_then(|s| s.find("providing")) {
        let actual_pos = start + pos;
        let end_pos = actual_pos + "providing".len();
        let prev_ok = actual_pos == 0
            || lower
                .get(..actual_pos)
                .and_then(|s| s.chars().next_back())
                .is_none_or(|prev_char| !prev_char.is_alphanumeric() && prev_char != '_');
        let next_ok = end_pos == lower.len()
            || lower
                .get(end_pos..)
                .and_then(|s| s.chars().next())
                .is_none_or(|next_char| !next_char.is_alphanumeric() && next_char != '_');
        if prev_ok && next_ok {
            let after = lower.get(end_pos..).unwrap_or("").trim_start();
            let is_providing_too = after.strip_prefix("too").is_some_and(|rem| {
                rem.chars().next().is_none_or(|c| !c.is_alphanumeric() && c != '_')
            });
            if !is_providing_too {
                return true;
            }
        }
        start = actual_pos + 1;
        while start < lower.len() && !lower.is_char_boundary(start) {
            start += 1;
        }
    }
    false
}

/// Checks whether `lower` contains an STPA identifier with `prefix` (e.g. `"l-"`, `"h-"`, `"uca-"`).
/// Realises: [REQ-CHECKS-SAFETY-STPA-HAS-STPA-ID]
pub(crate) fn has_stpa_id(lower: &str, prefix: &str) -> bool {
    for raw_token in lower.split_whitespace() {
        let token = raw_token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '$');
        let clean = token.strip_prefix('$').unwrap_or(token);
        if let Some(rest) = clean.strip_prefix(prefix) {
            let digits = rest.trim_end_matches(|c: char| !c.is_ascii_digit());
            if !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()) {
                return true;
            }
        }
    }
    let mut start = 0;
    while let Some(pos) = lower.get(start..).and_then(|s| s.find(prefix)) {
        let actual_pos = start + pos;
        let prev_ok = actual_pos == 0
            || lower
                .get(..actual_pos)
                .and_then(|s| s.chars().next_back())
                .is_none_or(|prev_char| !prev_char.is_alphanumeric() && prev_char != '_');
        let after_prefix = lower.get(actual_pos + prefix.len()..).unwrap_or("");
        let digits_count = after_prefix.chars().take_while(|c| c.is_ascii_digit()).count();
        if prev_ok && digits_count > 0 {
            let after_digits = after_prefix.get(digits_count..).unwrap_or("");
            if after_digits.chars().next().is_none_or(|c| !c.is_alphanumeric() && c != '_') {
                return true;
            }
        }
        start = actual_pos + prefix.len();
        while start < lower.len() && !lower.is_char_boundary(start) {
            start += 1;
        }
    }
    false
}

/// Audits Pillars 1-8 and Commercial Toolchain Hooks on `STPA_MATRIX.md`.
/// Realises: [REQ-CHECKS-SAFETY-STPA-CHECK-SAFETY-INTEGRITY]
pub(crate) fn audit_stpa_pillars_1_to_8(
    repo_root: &Path,
    stpa_content: &str,
    combined_content: &str,
    errors: &mut Vec<String>,
) {
    let stpa_lower = stpa_content.to_lowercase();
    if !((stpa_lower.contains("loss") || stpa_lower.contains("losses")) && has_stpa_id(&stpa_lower, "l-")) {
        errors.push("Pillar 1 violation: Missing System Losses ($L-1..N$) identification in STPA_MATRIX.md.".to_string());
    }
    if !((stpa_lower.contains("hazard") || stpa_lower.contains("hazards")) && has_stpa_id(&stpa_lower, "h-")) {
        errors.push("Pillar 2 violation: Missing System Hazards ($H-1..N$) identification in STPA_MATRIX.md.".to_string());
    }
    if !(stpa_lower.contains("control structure") || (stpa_lower.contains("controller") && stpa_lower.contains("actuator"))) {
        errors.push("Pillar 3 violation: Missing Hierarchical Control Structure Topology in STPA_MATRIX.md.".to_string());
    }
    if !(stpa_lower.contains("unsafe control action") || stpa_lower.contains("unsafe control actions") || has_stpa_id(&stpa_lower, "uca-")) {
        errors.push("Pillar 4 violation: Missing Unsafe Control Actions ($UCA-1..N$) in STPA_MATRIX.md.".to_string());
    }
    let missing_uca_cats = check_uca_categories(stpa_content);
    if !missing_uca_cats.is_empty() {
        errors.push(format!("Pillar 4 violation: Missing UCA failure mode categories in STPA_MATRIX.md: {}.", missing_uca_cats.join(", ")));
    }
    let has_ls_kw = stpa_lower.contains("loss scenario")
        || stpa_lower.contains("loss scenarios")
        || stpa_lower.contains("causal scenario")
        || stpa_lower.contains("causal scenarios");
    if !(has_ls_kw && has_stpa_id(&stpa_lower, "ls-")) {
        errors.push("Pillar 5 violation: Missing Loss Scenarios ($LS-1..N$) & Causal Factors in STPA_MATRIX.md.".to_string());
    }
    if !((stpa_lower.contains("safety constraint") || stpa_lower.contains("safety constraints")) && has_stpa_id(&stpa_lower, "sc-")) {
        errors.push("Pillar 6 violation: Missing Formal Safety Constraints ($SC-1..N$) in STPA_MATRIX.md.".to_string());
    }

    audit_pillar_7_fmeca(repo_root, stpa_content, &stpa_lower, combined_content, errors);

    let has_pillar_8_kw = stpa_lower.contains("pillar 8")
        || stpa_lower.contains("regulatory objectives")
        || stpa_lower.contains("sora sail");
    let has_heading_8 = stpa_content.lines().any(|l| {
        let trimmed = l.trim();
        let after_hashes = trimmed.trim_start_matches('#');
        let hash_count = trimmed.len() - after_hashes.len();
        (2..=4).contains(&hash_count) && after_hashes.trim_start().starts_with("8.")
    });
    if !(has_pillar_8_kw || has_heading_8) {
        errors.push("Pillar 8 violation: Missing Pillar 8 in STPA_MATRIX.md (Domain Regulatory Objectives & Integrity Mapping / Regulatory Objectives & Risk Mitigations).".to_string());
    }
    let has_toolchain = stpa_lower.contains("matlab")
        || stpa_lower.contains("simulink")
        || stpa_lower.contains("stateflow")
        || stpa_lower.contains("embedded coder")
        || stpa_lower.contains("sldv");
    if !has_toolchain {
        errors.push("Commercial Toolchain violation: Missing MATLAB / Simulink / Stateflow / Embedded Coder integration hooks in STPA_MATRIX.md.".to_string());
    }
}
