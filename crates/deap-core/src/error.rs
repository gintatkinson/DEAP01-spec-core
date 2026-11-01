//! Typed domain error hierarchy and fallible result alias for `deap-core`.
//!
//! Safety & Traceability: Realises: [REQ-DEAP-CORE-ERROR], [DO-330-TQL-4-CORE-ERROR]
//! Contract: Categorizes all fallible `deap-core` operations into closed `Io`, `Syntax`, `Validation`, and `Workspace` error variants carrying deterministic diagnostic messages.
//! Bounds & Faults: O(1) variant construction and borrowed string projection with zero panics, zero unchecked indexing, and deterministic `Display`, `Deref<Target = str>`, and `String` conversion.

#![forbid(unsafe_code)]

/// Canonical fallible return type for `deap-core` operations backed by [`CoreError`].
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-ERROR]
/// Contract: Pairs a typed success value `T` with a closed [`CoreError`] failure discriminant.
/// Bounds & Faults: Zero runtime overhead beyond standard `Result<T, CoreError>` representation.
pub type CoreResult<T> = Result<T, CoreError>;

/// Closed domain error enumeration for `deap-core` AST parsing, validation, I/O, and workspace gates.
///
/// Safety & Traceability: Realises: [REQ-DEAP-CORE-ERROR]
/// Contract: Every variant preserves its canonical formatted diagnostic `message: String` for deterministic fault classification and string projection.
/// Bounds & Faults: O(1) discriminant inspection; total, panic-free accessors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// Filesystem read, write, directory creation, or directory traversal failure.
    Io {
        /// Canonical formatted diagnostic message.
        message: String,
    },
    /// Source or schema lexical and delimiter balance syntax failure.
    Syntax {
        /// Canonical formatted diagnostic message.
        message: String,
    },
    /// Semantic, serialization, budget, or parameter validation failure.
    Validation {
        /// Canonical formatted diagnostic message.
        message: String,
    },
    /// Workspace baseline configuration or structural invariant failure.
    Workspace {
        /// Canonical formatted diagnostic message.
        message: String,
    },
}

impl CoreError {
    /// Constructs a [`CoreError::Io`] variant from a formatted filesystem diagnostic message.
    ///
    /// Safety & Traceability: Realises: [REQ-DEAP-CORE-ERROR]
    /// Contract: Stores `message.into()` verbatim inside `CoreError::Io`.
    /// Bounds & Faults: O(1) move from `String` or O(M) copy from `&str`; infallible.
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            message: message.into(),
        }
    }

    /// Constructs a [`CoreError::Syntax`] variant from a formatted lexical or delimiter diagnostic message.
    ///
    /// Safety & Traceability: Realises: [REQ-DEAP-CORE-ERROR]
    /// Contract: Stores `message.into()` verbatim inside `CoreError::Syntax`.
    /// Bounds & Faults: O(1) move from `String` or O(M) copy from `&str`; infallible.
    pub fn syntax(message: impl Into<String>) -> Self {
        Self::Syntax {
            message: message.into(),
        }
    }

    /// Constructs a [`CoreError::Validation`] variant from a formatted semantic or serialization diagnostic message.
    ///
    /// Safety & Traceability: Realises: [REQ-DEAP-CORE-ERROR]
    /// Contract: Stores `message.into()` verbatim inside `CoreError::Validation`.
    /// Bounds & Faults: O(1) move from `String` or O(M) copy from `&str`; infallible.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Constructs a [`CoreError::Workspace`] variant from a formatted workspace baseline diagnostic message.
    ///
    /// Safety & Traceability: Realises: [REQ-DEAP-CORE-ERROR]
    /// Contract: Stores `message.into()` verbatim inside `CoreError::Workspace`.
    /// Bounds & Faults: O(1) move from `String` or O(M) copy from `&str`; infallible.
    pub fn workspace(message: impl Into<String>) -> Self {
        Self::Workspace {
            message: message.into(),
        }
    }

    /// Borrows the canonical formatted diagnostic message carried by this error.
    ///
    /// Safety & Traceability: Realises: [REQ-DEAP-CORE-ERROR]
    /// Contract: Returns a borrowed `&str` identical to the inner variant `message` field.
    /// Bounds & Faults: O(1) time and space; total across all `CoreError` variants.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Io { message }
            | Self::Syntax { message }
            | Self::Validation { message }
            | Self::Workspace { message } => message.as_str(),
        }
    }

    /// Consumes this error and returns the owned canonical formatted diagnostic `String`.
    ///
    /// Safety & Traceability: Realises: [REQ-DEAP-CORE-ERROR]
    /// Contract: Moves the inner `message: String` out of `self` without reallocation.
    /// Bounds & Faults: O(1) move; infallible.
    pub fn into_string(self) -> String {
        match self {
            Self::Io { message }
            | Self::Syntax { message }
            | Self::Validation { message }
            | Self::Workspace { message } => message,
        }
    }
}

impl std::fmt::Display for CoreError {
    /// Writes the canonical diagnostic message slice to the target formatter stream.
    /// Realises: [REQ-DEAP-CORE-ERROR]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::error::Error for CoreError {}

impl std::ops::Deref for CoreError {
    type Target = str;

    /// Dereferences to the underlying diagnostic message string slice.
    /// Realises: [REQ-DEAP-CORE-ERROR]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for CoreError {
    /// Borrows the canonical diagnostic message as a `&str`.
    /// Realises: [REQ-DEAP-CORE-ERROR]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for CoreError {
    /// Compares the canonical diagnostic message against a `str` slice.
    /// Realises: [REQ-DEAP-CORE-ERROR]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for CoreError {
    /// Compares the canonical diagnostic message against a borrowed `&str` reference.
    /// Realises: [REQ-DEAP-CORE-ERROR]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for CoreError {
    /// Compares the canonical diagnostic message against an owned `String`.
    /// Realises: [REQ-DEAP-CORE-ERROR]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl From<CoreError> for String {
    /// Converts a [`CoreError`] into its owned canonical diagnostic `String`.
    /// Realises: [REQ-DEAP-CORE-ERROR]
    fn from(err: CoreError) -> Self {
        err.into_string()
    }
}

impl From<crate::source_ast::CstMetricsError> for CoreError {
    /// Bridges a typed [`crate::source_ast::CstMetricsError`] into its canonical [`CoreError`] category.
    /// Realises: [REQ-0608], [REQ-DEAP-CORE-ERROR]
    fn from(err: crate::source_ast::CstMetricsError) -> Self {
        let message = err.to_string();
        match err {
            crate::source_ast::CstMetricsError::LosslessRoundtripOrSpanMismatch { .. }
            | crate::source_ast::CstMetricsError::MalformedUtf8OrRustSyntax { .. } => {
                Self::syntax(message)
            }
            crate::source_ast::CstMetricsError::UnverifiedParsePrecondition { .. }
            | crate::source_ast::CstMetricsError::ThresholdEnvelopeOrMetricsViolation { .. }
            | crate::source_ast::CstMetricsError::AstDepthOrArithmeticExhaustion { .. } => {
                Self::validation(message)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreError, CoreResult};

    /// Verifies variant construction, `Display`, `Deref<Target = str>`, `AsRef<str>`, `PartialEq`, and `String::from`.
    #[test]
    fn test_core_error_variants_display_deref_and_into_string() {
        let io_err = CoreError::io("Failed to read source file 'a.rs'");
        let syn_err = CoreError::syntax("a.rs:3:5: syntax error: unclosed delimiter '{'");
        let val_err = CoreError::validation("Unsupported source language 'cobol'");
        let ws_err = CoreError::workspace("Check 10 failed: .gitignore missing");

        assert!(matches!(io_err, CoreError::Io { .. }));
        assert!(matches!(syn_err, CoreError::Syntax { .. }));
        assert!(matches!(val_err, CoreError::Validation { .. }));
        assert!(matches!(ws_err, CoreError::Workspace { .. }));

        assert_eq!(io_err.as_str(), "Failed to read source file 'a.rs'");
        assert_eq!(
            format!("{syn_err}"),
            "a.rs:3:5: syntax error: unclosed delimiter '{'"
        );
        assert!(syn_err.contains("syntax error: unclosed delimiter"));
        assert!(val_err.starts_with("Unsupported source language"));
        assert_eq!(ws_err.as_ref(), "Check 10 failed: .gitignore missing");

        assert_eq!(ws_err, "Check 10 failed: .gitignore missing");
        assert_eq!(*ws_err, *"Check 10 failed: .gitignore missing");
        assert_eq!(ws_err, String::from("Check 10 failed: .gitignore missing"));

        let converted = String::from(io_err);
        assert_eq!(converted, "Failed to read source file 'a.rs'");

        let fallible: CoreResult<usize> = Err(CoreError::validation("budget exceeded"));
        let propagated: Result<usize, String> = fallible.map_err(String::from);
        assert_eq!(propagated, Err(String::from("budget exceeded")));
    }

    /// Verifies that `deap-core` parsers, validators, and workspace checks emit typed `CoreError` variants.
    #[test]
    fn test_deap_core_entrypoints_emit_typed_core_error_variants() {
        let lang_err = crate::SourceLanguage::from_flag("fortran");
        assert!(matches!(lang_err, Err(CoreError::Validation { .. })));

        let syn_err = crate::validate_source_syntax("pub fn broken( {", "broken.rs");
        assert!(matches!(syn_err, Err(CoreError::Syntax { .. })));

        let empty_model = crate::SysmlModel::default();
        let slice_err = empty_model.slice_package_subsystem("", 64);
        assert!(matches!(slice_err, Err(CoreError::Validation { .. })));

        let missing_root = std::env::temp_dir().join("deap_core_nonexistent_ws_step_4a");
        let ws_err = crate::check_gitignore_exists(&missing_root);
        assert!(matches!(ws_err, Err(CoreError::Workspace { .. })));
    }
}
