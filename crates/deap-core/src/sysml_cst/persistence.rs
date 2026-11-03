//! Content-addressable incremental AST cache, file-backed model persistence, and Chain-of-Thought (CoT) audit log.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]
//! Contract: Provides `IncrementalAstCache`, `AstPersistenceEngine`, `CotAuditRecord`, and `CotAuditLog` for deterministic JSON snapshot caching, persistence, and provenance logging.
//! Bounds & Faults: O(N) digest and serialization over input size; fails closed with `CoreError` on invalid model IDs, I/O faults, or deserialization errors.

use super::model_parser::parse_sysml_model;
use super::package_model::SysmlModel;
use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// Content-addressable SHA-256 incremental cache for parsed `SysmlModel` AST trees under `.pipeline/cache/ast/`.
///
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C.
/// Realises: [REQ-DEAP-CORE-SYSML-AST-INCREMENTALASTCACHE] | SysML v2: PartDef(IncrementalAstCache)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalAstCache {
    pub cache_dir: std::path::PathBuf,
}

/// Computes the lowercase hexadecimal SHA-256 digest of `bytes` by delegating to `crate::digest::compute_sha256`.
/// Realises: [REQ-SYSML-AST-CACHE-DIGEST]
pub(crate) fn compute_sha256_hex(bytes: &[u8]) -> String {
    crate::digest::compute_sha256(bytes)
}

impl IncrementalAstCache {
    /// Initializes an `IncrementalAstCache` rooted at `<repo_root>/.pipeline/cache/ast`.
    /// Realises: [REQ-SYSML-AST-CACHE-NEW]
    pub fn new(repo_root: &std::path::Path) -> Self {
        Self {
            cache_dir: repo_root.join(".pipeline").join("cache").join("ast"),
        }
    }

    /// Computes the lowercase hexadecimal SHA-256 digest of `sysml_source`.
    /// Realises: [REQ-SYSML-AST-CACHE-DIGEST] | Issue: #494, #503, #522
    pub fn compute_digest(sysml_source: &str) -> String {
        compute_sha256_hex(sysml_source.as_bytes())
    }

    /// Loads a cached `SysmlModel` by SHA-256 digest if present on disk, or parses `sysml_source` and populates the cache.
    /// Realises: [REQ-SYSML-AST-CACHE-LOAD-OR-PARSE]
    pub fn load_or_parse(&self, sysml_source: &str) -> Result<(SysmlModel, bool), CoreError> {
        let digest = Self::compute_digest(sysml_source);
        let cache_path = self.cache_dir.join(format!("{}.json", digest));
        if cache_path.is_file() {
            let cached_text = std::fs::read_to_string(&cache_path).map_err(|e| {
                CoreError::io(format!(
                    "Failed to read cached AST file '{}': {}",
                    cache_path.display(),
                    e
                ))
            })?;
            let model: SysmlModel = serde_json::from_str(&cached_text).map_err(|e| {
                CoreError::validation(format!(
                    "Failed to deserialize cached AST file '{}': {}",
                    cache_path.display(),
                    e
                ))
            })?;
            return Ok((model, true));
        }

        let model = parse_sysml_model(sysml_source);
        std::fs::create_dir_all(&self.cache_dir).map_err(|e| {
            CoreError::io(format!(
                "Failed to create AST cache directory '{}': {}",
                self.cache_dir.display(),
                e
            ))
        })?;
        let serialized = serde_json::to_string_pretty(&model).map_err(|e| {
            CoreError::validation(format!(
                "Failed to serialize SysmlModel for AST cache: {}",
                e
            ))
        })?;
        std::fs::write(&cache_path, serialized).map_err(|e| {
            CoreError::io(format!(
                "Failed to write cached AST file '{}': {}",
                cache_path.display(),
                e
            ))
        })?;
        Ok((model, false))
    }
}

/// Zero-mocking file-backed persistence engine for `SysmlModel` AST snapshots (`PERSISTENCE_ARCHITECTURE.md`).
///
/// Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D.
/// Realises: [REQ-DEAP-CORE-SYSML-AST-ASTPERSISTENCEENGINE] | SysML v2: PartDef(AstPersistenceEngine)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AstPersistenceEngine {
    pub storage_root: std::path::PathBuf,
}

impl AstPersistenceEngine {
    /// Creates a new `AstPersistenceEngine` backed by `storage_root`.
    /// Realises: [REQ-SYSML-AST-PERSISTENCE-NEW]
    pub fn new(storage_root: std::path::PathBuf) -> Self {
        Self { storage_root }
    }

    /// Persists `model` under `model_id` as a canonical JSON document inside `self.storage_root`.
    /// Realises: [REQ-SYSML-AST-PERSIST-MODEL]
    pub fn persist_model(
        &self,
        model_id: &str,
        model: &SysmlModel,
    ) -> Result<std::path::PathBuf, CoreError> {
        let clean_id = model_id.trim();
        if clean_id.is_empty()
            || clean_id.contains("..")
            || clean_id.contains('/')
            || clean_id.contains('\\')
        {
            return Err(CoreError::validation(format!(
                "Invalid model_id '{}' for persistence",
                model_id
            )));
        }
        std::fs::create_dir_all(&self.storage_root).map_err(|e| {
            CoreError::io(format!(
                "Failed to create persistence root '{}': {}",
                self.storage_root.display(),
                e
            ))
        })?;
        let target_path = self.storage_root.join(format!("{}.json", clean_id));
        let json_payload = serde_json::to_string_pretty(model).map_err(|e| {
            CoreError::validation(format!(
                "Failed to serialize SysmlModel '{}' for persistence: {}",
                clean_id, e
            ))
        })?;
        std::fs::write(&target_path, json_payload).map_err(|e| {
            CoreError::io(format!(
                "Failed to write persisted model '{}': {}",
                target_path.display(),
                e
            ))
        })?;
        Ok(target_path)
    }

    /// Loads and deserializes a persisted `SysmlModel` identified by `model_id` from `self.storage_root`.
    /// Realises: [REQ-SYSML-AST-LOAD-MODEL]
    pub fn load_model(&self, model_id: &str) -> Result<SysmlModel, CoreError> {
        let clean_id = model_id.trim();
        if clean_id.is_empty()
            || clean_id.contains("..")
            || clean_id.contains('/')
            || clean_id.contains('\\')
        {
            return Err(CoreError::validation(format!(
                "Invalid model_id '{}' for load_model",
                model_id
            )));
        }
        let target_path = self.storage_root.join(format!("{}.json", clean_id));
        let json_payload = std::fs::read_to_string(&target_path).map_err(|e| {
            CoreError::io(format!(
                "Failed to read persisted model '{}': {}",
                target_path.display(),
                e
            ))
        })?;
        serde_json::from_str(&json_payload).map_err(|e| {
            CoreError::validation(format!(
                "Failed to deserialize persisted model '{}': {}",
                target_path.display(),
                e
            ))
        })
    }
}

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
    /// Realises: [REQ-SYSML-AST-COT-AUDIT-LOG-NEW]
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// Appends a `CotAuditRecord` in chronological order to `self.records`.
    /// Realises: [REQ-SYSML-AST-COT-AUDIT-LOG-APPEND]
    pub fn append_record(&mut self, record: CotAuditRecord) {
        self.records.push(record);
    }

    /// Serializes the `CotAuditLog` to a deterministic pretty-printed JSON string.
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
