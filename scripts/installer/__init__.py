#!/usr/bin/env python3
"""
DEAP Modular Installer Package.
Provides transactional, schema-driven, non-destructive downstream pipeline installation and verification.
/// Realises: [InstallerPackageExports]
"""

__version__ = "1.0.0"

from scripts.installer.metadata import (
    GitRemoteInfo,
    InstallOptions,
    PreservedMetadata,
    ProjectMetadataInfo,
    RepositoryRole,
    TrackerProvider,
)
from scripts.installer.rollback import (
    CLIValidationError,
    InstallerError,
    MetadataResolutionError,
    PromotionError,
    RollbackError,
    SafetyFixtureError,
    StagingError,
)


def __getattr__(name: str):
    if name == "main":
        from scripts.installer.cli import main
        return main
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


__all__ = [
    "__version__",
    "main",
    "InstallOptions",
    "RepositoryRole",
    "TrackerProvider",
    "GitRemoteInfo",
    "PreservedMetadata",
    "ProjectMetadataInfo",
    "InstallerError",
    "CLIValidationError",
    "MetadataResolutionError",
    "StagingError",
    "PromotionError",
    "RollbackError",
    "SafetyFixtureError",
]
