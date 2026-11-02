//! Zero-mocking file-backed persistence engine for `SysmlModel` AST snapshots.
//!
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C.
//! Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]

use super::package_model::SysmlModel;
use crate::error::CoreError;
use serde::{Deserialize, Serialize};

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
    ///
    /// Conforms to DO-178C Level A and `.pipeline/constitution.md` Section 1.9.
    /// Realises: [REQ-SYSML-AST-PERSISTENCE-NEW]
    pub fn new(storage_root: std::path::PathBuf) -> Self {
        Self { storage_root }
    }

    /// Persists `model` under `model_id` as a canonical JSON document inside `self.storage_root`.
    ///
    /// Conforms to DO-178C Level A and `PERSISTENCE_ARCHITECTURE.md`.
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
    ///
    /// Conforms to DO-178C Level A and `PERSISTENCE_ARCHITECTURE.md`.
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
