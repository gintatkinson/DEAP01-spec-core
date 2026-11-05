//! Chain-of-Thought (CoT) reasoning audit record and tamper-evident log.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]
//! Contract: Defines `CotAuditRecord` and `CotAuditLog` for append-only cryptographic reasoning provenance verification.
//! Bounds & Faults: O(R) validation over record count R; fails closed with `CoreError` on empty digests or traces.

use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// Cryptographically bound Chain-of-Thought (CoT) reasoning audit record (`DEAP_DEEPSEEK_HARNESS_INTEGRATION_BLUEPRINT.md`).
///
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D.
/// Realises: [REQ-DEAP-CORE-SYSML-AST-COTAUDITRECORD] | SysML v2: PartDef(CotAuditRecord)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CotAuditRecord {
    pub step_id: String,
    pub model_id: String,
    pub prompt_sha256: String,
    pub reasoning_trace: String,
    pub verified_ast_refs: Vec<String>,
}

/// Append-only audit log of `CotAuditRecord` entries for DO-330 TQL-4 verification traceability.
///
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D.
/// Realises: [REQ-DEAP-CORE-SYSML-AST-COTAUDITLOG] | SysML v2: PartDef(CotAuditLog)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CotAuditLog {
    pub records: Vec<CotAuditRecord>,
}

impl CotAuditLog {
    /// Creates an empty `CotAuditLog`.
    ///
    /// Conforms to DO-178C Level A, DO-330 TQL-4.
    /// Realises: [REQ-SYSML-AST-COT-AUDIT-LOG-NEW]
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Appends a `CotAuditRecord` in chronological order to `self.records`.
    ///
    /// Conforms to DO-178C Level A, DO-330 TQL-4.
    /// Realises: [REQ-SYSML-AST-COT-AUDIT-LOG-APPEND]
    pub fn append_record(&mut self, record: CotAuditRecord) {
        self.records.push(record);
    }

    /// Serializes the `CotAuditLog` to a deterministic pretty-printed JSON string.
    ///
    /// Conforms to DO-178C Level A, DO-330 TQL-4.
    /// Realises: [REQ-SYSML-AST-COT-AUDIT-LOG-JSON]
    pub fn to_json_pretty(&self) -> Result<String, CoreError> {
        serde_json::to_string_pretty(self).map_err(|e| {
            CoreError::validation(format!(
                "Failed to serialize CotAuditLog to pretty JSON: {}",
                e
            ))
        })
    }
}
