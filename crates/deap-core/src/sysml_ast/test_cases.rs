//! Test case definition structures and parser for SysML v2.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `TestCaseDef` and `parse_test_case_defs` for extracting SysML v2 `test case def` blocks and requirement verification links.
//! Bounds & Faults: Executes in linear O(N) time over source text with bounds-checked `.get` slicing and zero panics.

use super::identifiers::{
    extract_doc_comment_before, extract_ident, find_matching_brace, is_word_boundary_after,
    is_word_boundary_before, mask_string_literals, skip_ws,
};
use super::parser::strip_sysml_comments;
use serde::{Deserialize, Serialize};

/// Represents a verification test case definition in SysML v2.
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C.
/// Realises: [REQ-DEAP-CORE-SYSML-AST-TESTCASEDEF] | SysML v2: PartDef(TestCaseDef)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct TestCaseDef {
    pub name: String,
    pub subject_part: Option<String>,
    #[serde(default)]
    pub verified_requirements: Vec<String>,
    pub objective: Option<String>,
    #[serde(default)]
    pub test_steps: Vec<String>,
    pub doc: Option<String>,
}

/// Parsed elements extracted from a test case body block.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-TEST-CASE-DEFS]
#[derive(Default)]
struct TestCaseBodyContent {
    subject_part: Option<String>,
    verified_requirements: Vec<String>,
    objective: Option<String>,
    test_steps: Vec<String>,
}

/// Parses the inner body statements of a test case block.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-TEST-CASE-DEFS]
fn parse_test_case_body(body: &str) -> TestCaseBodyContent {
    let mut content = TestCaseBodyContent::default();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("subject") && is_word_boundary_after(trimmed, 7) {
            let val = trimmed.get(7..).unwrap_or("").trim().trim_end_matches(';').trim();
            if !val.is_empty() {
                content.subject_part = Some(val.to_string());
            }
        } else if trimmed.starts_with("verify") && is_word_boundary_after(trimmed, 6) {
            let mut rest = trimmed.get(6..).unwrap_or("").trim();
            if rest.starts_with("assert") && is_word_boundary_after(rest, 6) {
                rest = rest.get(6..).unwrap_or("").trim();
            }
            if rest.starts_with("requirement") && is_word_boundary_after(rest, 11) {
                rest = rest.get(11..).unwrap_or("").trim();
            } else if rest.starts_with("constraint") && is_word_boundary_after(rest, 10) {
                rest = rest.get(10..).unwrap_or("").trim();
            }
            for target in rest.trim_end_matches(';').trim().split(',') {
                let t = target.trim();
                if !t.is_empty() {
                    content.verified_requirements.push(t.to_string());
                }
            }
        } else if trimmed.starts_with("objective") && is_word_boundary_after(trimmed, 9) {
            let val = trimmed.get(9..).unwrap_or("").trim().trim_end_matches(';').trim().trim_matches('"');
            if !val.is_empty() {
                content.objective = Some(val.to_string());
            }
        } else if trimmed.starts_with("step") && is_word_boundary_after(trimmed, 4) {
            let val = trimmed.get(4..).unwrap_or("").trim().trim_end_matches(';').trim();
            if !val.is_empty() {
                content.test_steps.push(val.to_string());
            }
        }
    }
    content
}

/// Parse test case definitions from SysML v2 source text.
/// Conforms to DO-178C Level A, ISO 26262 ASIL D, and ECSS-E-ST-40C.
/// Realises: [REQ-SYSML-AST-SSOT-PARITY/PARSE-TEST-CASE-DEFS]
pub fn parse_test_case_defs(content: &str) -> Vec<TestCaseDef> {
    let clean = strip_sysml_comments(content);
    let masked = mask_string_literals(&clean);
    let mut test_cases = Vec::new();
    let mut idx = 0;
    while let Some(pos) = masked.get(idx..).and_then(|s| s.find("test")) {
        let test_pos = idx + pos;
        if !is_word_boundary_before(&masked, test_pos)
            || !is_word_boundary_after(&masked, test_pos + 4)
        {
            idx = test_pos + 4;
            continue;
        }
        let doc = extract_doc_comment_before(&clean, test_pos);
        let mut curr = test_pos + 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if !masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("case") && is_word_boundary_after(s, 4))
        {
            idx = test_pos + 4;
            continue;
        }
        curr += 4;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));
        if masked
            .get(curr..)
            .is_some_and(|s| s.starts_with("def") && is_word_boundary_after(s, 3))
        {
            curr += 3;
            curr += skip_ws(masked.get(curr..).unwrap_or(""));
        }
        let (name, name_len) = extract_ident(masked.get(curr..).unwrap_or(""));
        if name.is_empty() {
            idx = test_pos + 4;
            continue;
        }
        curr += name_len;
        curr += skip_ws(masked.get(curr..).unwrap_or(""));

        let rem_body = masked.get(curr..).unwrap_or("");
        let (body_content, next_idx) = if rem_body.starts_with('{') {
            if let Some(close_pos) = find_matching_brace(&clean, curr) {
                let body = clean.get(curr + 1..close_pos).unwrap_or("");
                (parse_test_case_body(body), close_pos + 1)
            } else {
                idx = test_pos + 4;
                continue;
            }
        } else if rem_body.starts_with(';') {
            (TestCaseBodyContent::default(), curr + 1)
        } else {
            idx = test_pos + 4;
            continue;
        };

        idx = next_idx;
        test_cases.push(TestCaseDef {
            name: name.to_string(),
            subject_part: body_content.subject_part,
            verified_requirements: body_content.verified_requirements,
            objective: body_content.objective,
            test_steps: body_content.test_steps,
            doc,
        });
    }
    test_cases
}
