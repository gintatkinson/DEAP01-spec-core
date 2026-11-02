#![deny(clippy::too_many_lines, clippy::too_many_arguments, clippy::undocumented_unsafe_blocks)]
//! CST complexity envelope and cryptographic supply-chain verification receipt. [DOC-TIER: A]
//!
//! ## 1. Safety Intent & Regulatory Scope
//! Conforms to DO-178C Level A, DO-330 TQL-4, ISO 26262 ASIL D, and ECSS-E-ST-40C qualification.
//! Implements air-gapped supply-chain and CST complexity envelope sealing (TD-03 / US-51).
//!
//! ## 2. Invariants & Mathematical Properties
//! - C_CST_01..C_CST_06 envelope validation enforces strict upper bounds on complexity.
//! - Cryptographic seal binds manifest, lockfile, package records, and complexity envelope.
//!
//! ## 3. Preconditions
//! - Callers provide valid snapshot records, verified crate names, and complexity envelope.
//!
//! ## 4. Postconditions
//! - Returns sealed SupplyChainVerificationReceipt authorizing Idle -> ParsingLosslessCst.
//!
//! ## 5. Algorithmic Complexity Bounds
//! - O(P log P) deterministic SHA-256 seal computation over P package records.
//!
//! ## 6. Failure Modes & Fault Tolerance
//! - Typed OfflineLockGovernanceError fail-closed diagnostics without unhandled panic paths.
//!
//! ## 7. Realization & Traceability
//! Realises: [LosslessRustCstAnalyzer/verify_analyzer_envelope_and_seal_receipt], [REQ-0608]
//! SysML v2: part def LosslessRustCstAnalyzer

use std::collections::{BTreeMap, BTreeSet};

pub const MAX_CST_FN_EXEC_SLOC: u32 = 50;
pub const MAX_CST_FN_PARAM_COUNT: u32 = 5;
pub const REQUIRED_MAX_PREDICATE_ARITY: u32 = 3;
pub const REQUIRED_TIER_BC_DOC_LINES: u32 = 2;
pub const BP08_DETERMINISTIC_UUID: &str = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
pub const MAX_AST_DEPTH: usize = 64;

/// Provenance of a locked package dependency in the air-gapped build graph. Realises: [REQ-0608]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum PackageSourceKind {
    WorkspaceMember,
    RegistryPinned,
    VendoredLocal,
}

/// Metadata record for a single package pinned in Cargo.lock. Realises: [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LockedPackageRecord {
    pub name: String,
    pub version: String,
    pub checksum: Option<String>,
    pub source_kind: PackageSourceKind,
    pub dependencies: BTreeSet<String>,
}

/// Air-gapped supply chain snapshot of workspace manifests and locked dependencies. Realises: [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LockManifestSnapshot {
    pub cargo_toml_sha256: [u8; 32],
    pub cargo_lock_sha256: [u8; 32],
    pub packages: BTreeMap<String, LockedPackageRecord>,
    pub workspace_members: BTreeSet<String>,
    pub offline_locked_enforced: bool,
    pub vendor_bundle_supported: bool,
}

/// Typed fail-closed error taxonomy for air-gapped lock and envelope governance. Realises: [REQ-0608]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum OfflineLockGovernanceError {
    LockPolicyDisabled,
    ManifestSizeCeilingExceeded,
    MalformedTomlOrLockSyntax,
    UnpinnedGitOrWildcardSpecifier,
    ForbiddenSemanticDependencyDetected,
    MissingRequiredCstDependency,
    InvalidSha256ChecksumHex,
    PackageGraphDepthOrCycleExceeded,
    ManifestToLockDriftDetected,
    CstConstraintEnvelopeViolation,
    OfflineCargoCheckFailed,
}

impl OfflineLockGovernanceError {
    /// Returns canonical diagnostic code. Realises: [REQ-0608]
    pub const fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::CstConstraintEnvelopeViolation => "DEAP-E0803",
            _ => "DEAP-E0804",
        }
    }
}

impl std::fmt::Display for OfflineLockGovernanceError {
    /// Formats error for display. Realises: [REQ-0608]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] offline lock governance failure", self.diagnostic_code())
    }
}

impl std::error::Error for OfflineLockGovernanceError {}

/// Verified structural complexity envelope bounding LosslessRustCstAnalyzer. Realises: [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct CstComplexityEnvelope {
    pub max_fn_exec_sloc: u32,
    pub max_fn_param_count: u32,
    pub max_mut_bool_flags: u32,
    pub max_predicate_arity: u32,
    pub stage_fixed_point_mismatch_files: u32,
    pub runtime_llm_token_count: u32,
    pub tier_bc_max_doc_lines: u32,
    pub lossless_roundtrip_required: bool,
    pub deterministic_uuid: String,
}

impl Default for CstComplexityEnvelope {
    /// Default complexity envelope. Realises: [REQ-0608]
    fn default() -> Self {
        Self {
            max_fn_exec_sloc: MAX_CST_FN_EXEC_SLOC,
            max_fn_param_count: MAX_CST_FN_PARAM_COUNT,
            max_mut_bool_flags: 0,
            max_predicate_arity: REQUIRED_MAX_PREDICATE_ARITY,
            stage_fixed_point_mismatch_files: 0,
            runtime_llm_token_count: 0,
            tier_bc_max_doc_lines: REQUIRED_TIER_BC_DOC_LINES,
            lossless_roundtrip_required: true,
            deterministic_uuid: BP08_DETERMINISTIC_UUID.to_string(),
        }
    }
}

/// Cryptographically sealed supply-chain receipt authorizing transition from Idle. Realises: [REQ-0608]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct SupplyChainVerificationReceipt {
    pub cargo_toml_sha256: [u8; 32],
    pub cargo_lock_sha256: [u8; 32],
    pub verified_crates: BTreeSet<String>,
    pub workspace_members: BTreeSet<String>,
    pub envelope: CstComplexityEnvelope,
    pub seal_sha256: [u8; 32],
    pub supply_chain_receipt_sealed: bool,
}

impl SupplyChainVerificationReceipt {
    /// Verifies snapshot parity against active receipt. Realises: [REQ-0608]
    pub fn verify_snapshot_parity(
        &self,
        snapshot: &LockManifestSnapshot,
    ) -> Result<(), OfflineLockGovernanceError> {
        if !self.supply_chain_receipt_sealed
            || self.cargo_toml_sha256 != snapshot.cargo_toml_sha256
            || self.cargo_lock_sha256 != snapshot.cargo_lock_sha256
            || self.workspace_members != snapshot.workspace_members
            || self.verified_crates.len() != snapshot.packages.len()
        {
            return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
        }
        validate_cst_complexity_envelope(&self.envelope)?;
        let expected = compute_supply_chain_receipt_seal(snapshot, &self.verified_crates, &self.envelope);
        if self.seal_sha256 != expected {
            return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
        }
        Ok(())
    }
}

/// Validates structural complexity envelope limits. Realises: [REQ-0608]
fn validate_cst_complexity_envelope(
    envelope: &CstComplexityEnvelope,
) -> Result<(), OfflineLockGovernanceError> {
    if envelope.max_fn_exec_sloc == 0
        || envelope.max_fn_exec_sloc > MAX_CST_FN_EXEC_SLOC
        || envelope.max_fn_param_count > MAX_CST_FN_PARAM_COUNT
        || envelope.max_mut_bool_flags != 0
        || envelope.stage_fixed_point_mismatch_files != 0
        || envelope.runtime_llm_token_count != 0
        || envelope.tier_bc_max_doc_lines != REQUIRED_TIER_BC_DOC_LINES
        || envelope.max_predicate_arity != REQUIRED_MAX_PREDICATE_ARITY
        || !envelope.lossless_roundtrip_required
        || envelope.deterministic_uuid != BP08_DETERMINISTIC_UUID
    {
        return Err(OfflineLockGovernanceError::CstConstraintEnvelopeViolation);
    }
    Ok(())
}

/// Appends length-prefixed string slice to buffer. Realises: [REQ-0608]
fn push_len_prefixed_str(buf: &mut Vec<u8>, value: &str) {
    buf.extend_from_slice(&(value.len() as u64).to_be_bytes());
    buf.extend_from_slice(value.as_bytes());
}

/// Computes cryptographic receipt seal digest. Realises: [REQ-0608]
fn compute_supply_chain_receipt_seal(
    snapshot: &LockManifestSnapshot,
    verified_crates: &BTreeSet<String>,
    envelope: &CstComplexityEnvelope,
) -> [u8; 32] {
    let mut buf = Vec::with_capacity(512);
    buf.extend_from_slice(b"DEAP-BP08-SUPPLY-CHAIN-RECEIPT-V1");
    buf.extend_from_slice(&snapshot.cargo_toml_sha256);
    buf.extend_from_slice(&snapshot.cargo_lock_sha256);
    buf.push(u8::from(snapshot.offline_locked_enforced));
    buf.push(u8::from(snapshot.vendor_bundle_supported));
    for (pkg_key, rec) in &snapshot.packages {
        push_len_prefixed_str(&mut buf, pkg_key);
        let cs = rec.checksum.as_deref().unwrap_or_default();
        push_len_prefixed_str(&mut buf, cs);
    }
    for crate_key in verified_crates {
        push_len_prefixed_str(&mut buf, crate_key);
    }
    for member in &snapshot.workspace_members {
        push_len_prefixed_str(&mut buf, member);
    }
    buf.extend_from_slice(&envelope.max_fn_exec_sloc.to_be_bytes());
    buf.extend_from_slice(&envelope.max_fn_param_count.to_be_bytes());
    buf.extend_from_slice(&envelope.max_mut_bool_flags.to_be_bytes());
    buf.extend_from_slice(&envelope.max_predicate_arity.to_be_bytes());
    buf.extend_from_slice(&envelope.stage_fixed_point_mismatch_files.to_be_bytes());
    buf.extend_from_slice(&envelope.runtime_llm_token_count.to_be_bytes());
    buf.extend_from_slice(&envelope.tier_bc_max_doc_lines.to_be_bytes());
    buf.push(u8::from(envelope.lossless_roundtrip_required));
    push_len_prefixed_str(&mut buf, &envelope.deterministic_uuid);
    crate::digest::compute_sha256_bytes(&buf)
}

/// Verifies supply chain and envelope preconditions, sealing an immutable verification receipt.
/// Realises: [LosslessRustCstAnalyzer/verify_analyzer_envelope_and_seal_receipt], [REQ-0608]
pub fn verify_analyzer_envelope_and_seal_receipt(
    snapshot: &LockManifestSnapshot,
    verified_crates: &BTreeSet<String>,
    envelope: &CstComplexityEnvelope,
) -> Result<SupplyChainVerificationReceipt, OfflineLockGovernanceError> {
    if !snapshot.offline_locked_enforced || !snapshot.vendor_bundle_supported {
        return Err(OfflineLockGovernanceError::LockPolicyDisabled);
    }
    if snapshot.packages.is_empty() {
        return Err(OfflineLockGovernanceError::MalformedTomlOrLockSyntax);
    }
    if verified_crates.is_empty() {
        return Err(OfflineLockGovernanceError::ManifestToLockDriftDetected);
    }
    for req in ["ra_ap_syntax", "rowan", "syn", "quote", "prettyplease"] {
        if !verified_crates.iter().any(|c| c == req || c.starts_with(&format!("{req}@"))) {
            return Err(OfflineLockGovernanceError::MissingRequiredCstDependency);
        }
    }
    validate_cst_complexity_envelope(envelope)?;
    let seal_sha256 = compute_supply_chain_receipt_seal(snapshot, verified_crates, envelope);
    Ok(SupplyChainVerificationReceipt {
        cargo_toml_sha256: snapshot.cargo_toml_sha256,
        cargo_lock_sha256: snapshot.cargo_lock_sha256,
        verified_crates: verified_crates.clone(),
        workspace_members: snapshot.workspace_members.clone(),
        envelope: envelope.clone(),
        seal_sha256,
        supply_chain_receipt_sealed: true,
    })
}
