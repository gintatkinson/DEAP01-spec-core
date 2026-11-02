//! Content-addressable incremental cache for parsed `SysmlModel` AST trees.
//!
//! Safety & Traceability: Realises: [REQ-SYSML-AST-MODULE], [DO-330-TQL-4-SYSML-AST]
//! Contract: Provides `IncrementalAstCache` for SHA-256 content-addressable caching of parsed `SysmlModel` AST trees under `.pipeline/cache/ast/`.
//! Bounds & Faults: O(N) digest and serialization over input size; fails closed with `CoreError` on I/O or deserialization faults.

use super::package_model::SysmlModel;
use super::parser::parse_sysml_model;
use crate::error::CoreError;

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
    ///
    /// Conforms to DO-178C Level A and `DEAP_SYSML_V2_INGESTION_ENGINE_BLUEPRINT.md`.
    /// Realises: [REQ-SYSML-AST-CACHE-NEW]
    pub fn new(repo_root: &std::path::Path) -> Self {
        Self {
            cache_dir: repo_root.join(".pipeline").join("cache").join("ast"),
        }
    }

    /// Computes the lowercase hexadecimal SHA-256 digest of `sysml_source`.
    ///
    /// Conforms to DO-178C Level A, DO-330 TQL-4, and FIPS 180-4.
    /// Realises: [REQ-SYSML-AST-CACHE-DIGEST] | Issue: #494, #503, #522
    pub fn compute_digest(sysml_source: &str) -> String {
        compute_sha256_hex(sysml_source.as_bytes())
    }

    /// Loads a cached `SysmlModel` by SHA-256 digest if present on disk, or parses `sysml_source` and populates the cache.
    ///
    /// Conforms to DO-178C Level A and `DEAP_SYSML_V2_INGESTION_ENGINE_BLUEPRINT.md`.
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
