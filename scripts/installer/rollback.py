#!/usr/bin/env python3
"""
Transactional workspace snapshot management, POSIX signal trapping, and automated atomic rollback.
/// Realises: [InstallerRollbackManager]
"""
import os
import shutil
import signal
import sys
import tempfile
from pathlib import Path
from typing import Dict, List, Optional, Set


class InstallerError(Exception):
    """Base exception for all installer runtime errors."""
    pass


class CLIValidationError(InstallerError):
    """Raised when command-line parameters fail validation constraints."""
    pass


class MetadataResolutionError(InstallerError):
    """Raised when repository metadata, git remote, or role cannot be resolved."""
    pass


class StagingError(InstallerError):
    """Raised when directory staging, asset assembly, or copy operations fail."""
    pass


class PromotionError(InstallerError):
    """Raised when promoting staged assets to the target directory fails."""
    pass


class RollbackError(InstallerError):
    """Raised when restoring the workspace from snapshot fails."""
    pass


class SafetyFixtureError(InstallerError):
    """Raised when required safety integrity fixtures are missing or corrupted."""
    pass


class RollbackManager:
    """
    Tracks workspace mutations, maintains pre-installation file snapshots,
    and executes rollback on error or signal interruption.
    """

    def __init__(self, target_dir: Path):
        self.target_dir = Path(target_dir).resolve()
        self.snapshot_dir: Optional[tempfile.TemporaryDirectory] = None
        self.snapshot_path: Optional[Path] = None
        self.modified_files: Dict[Path, Path] = {}  # target_file -> snapshot_file
        self.created_files: Set[Path] = set()
        self.created_dirs: Set[Path] = set()
        self._active: bool = False
        self._rolled_back: bool = False
        self._old_sigint = None
        self._old_sigterm = None

    def __enter__(self) -> "RollbackManager":
        self.snapshot_dir = tempfile.TemporaryDirectory(prefix="deap_snapshot_")
        self.snapshot_path = Path(self.snapshot_dir.name)
        self._active = True
        self._register_signal_handlers()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb) -> None:
        self._restore_signal_handlers()
        if exc_type is not None and not self._rolled_back:
            self.rollback()
        if self.snapshot_dir:
            try:
                self.snapshot_dir.cleanup()
            except Exception:
                pass
        self._active = False

    def record_pre_modification(self, target_file: Path) -> None:
        """
        Snapshots an existing target file prior to modification or replacement.
        """
        target_path = Path(target_file).resolve()
        if not target_path.exists():
            return
        if target_path in self.modified_files:
            return  # already snapshotted original version
        if not self.snapshot_path:
            return

        try:
            rel_path = target_path.relative_to(self.target_dir)
        except ValueError:
            rel_path = Path(target_path.name)

        snap_file = self.snapshot_path / rel_path
        snap_file.parent.mkdir(parents=True, exist_ok=True)
        try:
            shutil.copy2(target_path, snap_file)
            self.modified_files[target_path] = snap_file
        except Exception as e:
            raise RollbackError(f"Failed to snapshot file {target_path}: {e}") from e

    def record_creation(self, created_path: Path) -> None:
        """
        Records a newly created file or directory for deletion during rollback.
        """
        p = Path(created_path).resolve()
        if p.is_dir():
            self.created_dirs.add(p)
        else:
            self.created_files.add(p)

    def rollback(self) -> None:
        """
        Executes atomic rollback:
        1. Restores all modified files from their snapshot copies.
        2. Unlinks all newly created files.
        3. Removes newly created empty directories in reverse depth order.
        """
        if self._rolled_back:
            return
        self._rolled_back = True
        sys.stderr.write("Initiating automated rollback to restore pristine workspace state...\n")
        errors: List[str] = []

        # 1. Restore modified files
        for target_file, snap_file in self.modified_files.items():
            if snap_file.exists():
                try:
                    target_file.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copy2(snap_file, target_file)
                except Exception as e:
                    errors.append(f"Failed to restore {target_file}: {e}")

        # 2. Remove newly created files
        for f in self.created_files:
            if f.exists() and not f.is_dir():
                try:
                    f.unlink()
                except Exception as e:
                    errors.append(f"Failed to remove created file {f}: {e}")

        # 3. Remove newly created directories in reverse depth order
        sorted_dirs = sorted(self.created_dirs, key=lambda d: len(d.parts), reverse=True)
        for d in sorted_dirs:
            if d.exists() and d.is_dir():
                try:
                    if not any(d.iterdir()):
                        d.rmdir()
                except Exception:
                    pass

        if errors:
            msg = "\n".join(errors)
            sys.stderr.write(f"Rollback completed with warnings:\n{msg}\n")
        else:
            sys.stderr.write("Rollback completed successfully. Workspace restored.\n")

    def _register_signal_handlers(self) -> None:
        try:
            self._old_sigint = signal.getsignal(signal.SIGINT)
            self._old_sigterm = signal.getsignal(signal.SIGTERM)

            def _handle_sigint(signum, frame):
                sys.stderr.write("\nInstallation interrupted by user.\n")
                self.rollback()
                sys.exit(130)

            def _handle_sigterm(signum, frame):
                sys.stderr.write("\nInstallation terminated by signal.\n")
                self.rollback()
                sys.exit(143)

            signal.signal(signal.SIGINT, _handle_sigint)
            signal.signal(signal.SIGTERM, _handle_sigterm)
        except (ValueError, AttributeError):
            pass

    def _restore_signal_handlers(self) -> None:
        try:
            if self._old_sigint is not None:
                signal.signal(signal.SIGINT, self._old_sigint)
            if self._old_sigterm is not None:
                signal.signal(signal.SIGTERM, self._old_sigterm)
        except (ValueError, AttributeError):
            pass
