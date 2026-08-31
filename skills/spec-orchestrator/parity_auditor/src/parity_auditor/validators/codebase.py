"""
Validator that enforces codebase compliance rules across React, Flutter,
Python and specification files.

Uses AST-based checks for import validation (C10) and event-echo guard
enforcement (I4).  Detects hardcoded design-token colours, banned import
libraries, missing write-lock controls, and missing playhead-rate clamps.
"""

import os
import re
import json
from typing import List
from .base import IValidator
from ..core.findings import Finding
from ..core.workspace import WorkspaceRepository
from ..utils.comment_utils import strip_c_style_comments
from ..utils.color_utils import extract_hex_colors_from_json
from ..utils.ast_utils import verify_python_ast
from ..utils.codebase import ast_check_banned_imports

class CodebaseValidator(IValidator):
    """Enforce codebase compliance: AST import checks, event-echo guard, design-token colours, forbidden libs."""

    def validate(self, repo: WorkspaceRepository, **kwargs) -> List[str]:
        """
        Run all codebase compliance checks.

        Walks React, Flutter and Python source directories, verifying:
          - No hardcoded design-token colours.
          - No banned imported libraries (AST-based for Python, regex fallback
            for JS/Dart).
          - Event-Echo Guard: selection setters that emit callbacks must call
            the configured guard method.
          - Write-lock controls in network gateway files.
          - Playhead-rate clamps in 4D viewport files.
          - FFI memory-safety checks (NativeFinalizer + refcounting).

        Also checks specification files for DOM and pixel-dimension leakage.

        Args:
            repo: WorkspaceRepository providing rules and target directories.
            **kwargs: Unused additional keyword arguments.

        Returns:
            List of error strings, empty when all checks pass.
        """
        errors = []
        workspace_dir = repo.workspace_dir
        rules = repo.get_codebase_rules()
        
        val_rules = rules.validation_rules
        playhead_rate_limits = val_rules.playhead_rate_limits
        target_dirs = rules.target_directories
        
        flutter_rules = rules.flutter_rules
        python_rules = rules.python_rules
        spec_rules = rules.spec_rules
        
        def has_files_with_extensions(extensions: List[str]) -> bool:
            exclusions = {".git", ".agents", "skills", ".pipeline", ".tessl-plugin", "docs"}
            for root, dirs, files in os.walk(workspace_dir):
                dirs[:] = [d for d in dirs if d not in exclusions]
                for file in files:
                    if any(file.endswith(ext) for ext in extensions):
                        return True
            return False

        flutter_dir_name = target_dirs.flutter
        flutter_dir = os.path.join(workspace_dir, flutter_dir_name) if flutter_dir_name else None
        if flutter_dir_name and not os.path.exists(flutter_dir):
            if has_files_with_extensions(flutter_rules.file_extensions):
                errors.append(Finding(
                    "platform-directory-must-exist-when-configured",
                    f"Compliance Bypass Loophole: Configured Flutter directory '{flutter_dir_name}' does not exist on disk.",
                    location=flutter_dir_name,
                ))
        
        # Design tokens / hardcoded color logic
        design_tokens_path_rel = spec_rules.design_tokens_path
        if not design_tokens_path_rel:
            errors.append(Finding(
                "design-tokens-path-must-be-configured",
                "Missing 'spec_rules.design_tokens_path' in codebase_rules.json",
            ))
            return errors
            
        design_tokens_path = os.path.join(workspace_dir, design_tokens_path_rel)
        if not os.path.exists(design_tokens_path):
            errors.append(Finding(
                "design-tokens-file-must-exist",
                f"Design tokens file does not exist at path: {design_tokens_path}",
                location=design_tokens_path_rel,
            ))
            return errors
            
        try:
            with open(design_tokens_path, "r", encoding="utf-8") as f:
                tokens_data = json.load(f)
            forbidden_colors_hex = extract_hex_colors_from_json(tokens_data)
        except Exception as e:
            errors.append(Finding(
                "design-tokens-file-must-be-loadable",
                f"Failed to load design tokens for compliance check: {e}",
                location=design_tokens_path_rel,
            ))
            return errors
            
        if not forbidden_colors_hex:
            errors.append(Finding(
                "design-tokens-must-declare-colours",
                f"No forbidden design token colors could be extracted from design tokens file at: {design_tokens_path}",
                location=design_tokens_path_rel,
            ))
            return errors
            
        hardcoded_colors_flutter = {}
        for hex_val in forbidden_colors_hex:
            clean_hex = hex_val.replace("#", "").lower()
            if len(clean_hex) == 3:
                clean_hex = "".join([c*2 for c in clean_hex])
            if len(clean_hex) == 8:
                flutter_hex = clean_hex[6:8] + clean_hex[0:6]
            else:
                flutter_hex = "ff" + clean_hex
            hardcoded_colors_flutter[flutter_hex] = f"Forbidden design token color (0x{flutter_hex.upper()})"
                                
        # 2. Flutter Desktop/Web Codebase Compliance
        if flutter_dir and os.path.exists(flutter_dir):
            flutter_exts = tuple(flutter_rules.file_extensions)
            flutter_exclusions = set(flutter_rules.exclusions)
            flutter_selection_setters = flutter_rules.selection_setters
            flutter_selection_triggers = flutter_rules.selection_triggers
            flutter_loop_guard_keywords = flutter_rules.loop_guard_keywords
            flutter_forbidden_words = flutter_rules.forbidden_words
            flutter_write_lock_keywords = flutter_rules.write_lock_keywords
            flutter_playhead_clamp_regex = flutter_rules.playhead_clamp_regex
            flutter_ffi_keywords = flutter_rules.ffi_keywords
            flutter_ffi_finalizer_keywords = flutter_rules.ffi_finalizer_keywords
            flutter_ffi_refcount_keywords = flutter_rules.ffi_refcount_keywords
            flutter_ui_dirs = flutter_rules.ui_directories
            flutter_net_dirs = flutter_rules.network_directories
            flutter_viewport_file_patterns = flutter_rules.viewport_file_patterns
            flutter_network_file_patterns = flutter_rules.network_file_patterns
            
            for root, dirs, files in os.walk(flutter_dir):
                dirs[:] = [d for d in dirs if d not in flutter_exclusions]
                for file in files:
                    if file.endswith(flutter_exts):
                        filepath = os.path.join(root, file)
                        rel_path = os.path.relpath(filepath, workspace_dir)
                        try:
                            with open(filepath, "r", encoding="utf-8") as f:
                                content = f.read()
                        except UnicodeDecodeError:
                            errors.append(Finding(
                                "source-file-must-be-valid-utf8",
                                f"Compliance Bypass Risk: '{rel_path}' is not valid UTF-8. Binary files are not permitted.",
                                location=rel_path,
                            ))
                            continue
                        except OSError as e:
                            errors.append(Finding(
                                "source-file-must-be-readable",
                                f"System Error: Failed to open '{rel_path}' for compliance read: {e}",
                                location=rel_path,
                            ))
                            continue
                            
                        if "design_tokens" in file:
                            continue
                            
                        for color_val, desc in hardcoded_colors_flutter.items():
                            pattern = re.compile(rf"\b0x{re.escape(color_val)}\b", re.IGNORECASE)
                            if pattern.search(content):
                                errors.append(Finding(
                                    "hardcoded-design-token-colour-forbidden",
                                    f"Flutter File '{rel_path}' contains hardcoded alarm color '0x{color_val.upper()}' ({desc}). Reference ThemeData or design-tokens config instead.",
                                    location=rel_path,
                                ))
                                
                        clean_content = strip_c_style_comments(content)
                        clean_content_lower = clean_content.lower()
                        
                        if any(setter in clean_content for setter in flutter_selection_setters):
                            if any(trigger in clean_content for trigger in flutter_selection_triggers):
                                if not any(g in clean_content_lower for g in flutter_loop_guard_keywords):
                                    errors.append(Finding(
                                        "selection-setter-requires-an-event-echo-guard",
                                        f"Flutter File '{rel_path}' contains selection setters and triggers updates, but lacks a loop guard variable (e.g. 'userInitiated' or 'programmatic') to satisfy the Event-Echo Guard.",
                                        location=rel_path,
                                    ))
                                    
                        is_flutter_ui = False
                        for d in flutter_ui_dirs:
                            if f"{d}/" in rel_path or rel_path.startswith(f"{d}/"):
                                is_flutter_ui = True
                                break
                        if is_flutter_ui:
                            banned_libs = self._check_banned_imports(content, flutter_forbidden_words, ".dart")
                            if banned_libs:
                                msg = flutter_rules.forbidden_words_message
                                errors.append(Finding(
                                    "ui-layer-must-not-import-banned-libraries",
                                    f"Flutter File '{rel_path}' is a {msg}",
                                    location=rel_path,
                                ))
                                
                        is_flutter_net = False
                        for d in flutter_net_dirs:
                            if f"{d}/" in rel_path or rel_path.startswith(f"{d}/"):
                                is_flutter_net = True
                                break
                        is_flutter_net_file = False
                        for pat in flutter_network_file_patterns:
                            if pat in file.lower():
                                is_flutter_net_file = True
                                break
                        if is_flutter_net and is_flutter_net_file:
                            if not any(lock_kw in clean_content_lower for lock_kw in flutter_write_lock_keywords):
                                errors.append(Finding(
                                    "network-gateway-requires-a-write-lock",
                                    f"Flutter Network Gateway File '{rel_path}' does not define a write-lock control to block egress mutations during timeline playback/scrubbing.",
                                    location=rel_path,
                                ))
                                
                        is_flutter_viewport = False
                        for pat in flutter_viewport_file_patterns:
                            if pat in rel_path.lower():
                                is_flutter_viewport = True
                                break
                        if is_flutter_viewport:
                            if not all(re.search(pat, clean_content) for pat in flutter_playhead_clamp_regex):
                                errors.append(Finding(
                                    "viewport-requires-playhead-rate-clamps",
                                    f"Flutter Viewport File '{rel_path}' does not implement the mandatory playhead rate clamps {playhead_rate_limits} for 4D spatial-temporal viewports.",
                                    location=rel_path,
                                ))
                                
                        if any(ffi_kw in clean_content for ffi_kw in flutter_ffi_keywords):
                            if not any(fn_kw in clean_content_lower for fn_kw in flutter_ffi_finalizer_keywords):
                                errors.append(Finding(
                                    "ffi-boundary-requires-a-native-finalizer",
                                    f"Flutter FFI File '{rel_path}' does not register a 'NativeFinalizer'. This violates memory safety rules.",
                                    location=rel_path,
                                ))
                            if not any(ref_kw in clean_content_lower for ref_kw in flutter_ffi_refcount_keywords):
                                errors.append(Finding(
                                    "ffi-boundary-requires-reference-counting",
                                    f"Flutter FFI File '{rel_path}' does not implement native allocation reference counting.",
                                    location=rel_path,
                                ))
                                
        # 3. Python AST-based Hardcoded Constants Audit
        python_scan_dirs = python_rules.scan_directories
        if not python_scan_dirs:
            python_scan_dirs = [workspace_dir]
        else:
            python_scan_dirs = [os.path.join(workspace_dir, d) for d in python_scan_dirs]
            
        python_exclusions = set(python_rules.exclusions)
        for scan_dir in python_scan_dirs:
            if os.path.exists(scan_dir):
                for root, dirs, files in os.walk(scan_dir):
                    dirs[:] = [d for d in dirs if d not in python_exclusions]
                    for file in files:
                        if file.endswith(".py"):
                            filepath = os.path.join(root, file)
                            ast_err = verify_python_ast(filepath, forbidden_colors_hex)
                            if ast_err:
                                errors.append(Finding(
                                    "python-source-must-not-hardcode-token-constants",
                                    ast_err,
                                    location=filepath,
                                ))
                                
        # 4. Check specification files for platform/visual leakage
        dom_patterns = spec_rules.dom_leak_patterns
        pixel_leak_patterns = spec_rules.pixel_leak_patterns
        spec_files = spec_rules.spec_files
        
        for spec_rel_path in spec_files:
            logical_components_path = os.path.join(workspace_dir, spec_rel_path)
            if os.path.exists(logical_components_path):
                try:
                    with open(logical_components_path, "r", encoding="utf-8") as f:
                        spec_content = f.read()
                    for pattern in dom_patterns:
                        if re.search(pattern, spec_content, re.IGNORECASE):
                            errors.append(Finding(
                                "specification-must-not-leak-dom-attributes",
                                f"Specification '{os.path.basename(logical_components_path)}' contains hardcoded DOM/accessibility leaks matching pattern '{pattern}'.",
                                location=spec_rel_path,
                            ))
                    for pattern in pixel_leak_patterns:
                        if re.search(pattern, spec_content, re.IGNORECASE):
                            errors.append(Finding(
                                "specification-must-not-hardcode-pixel-dimensions",
                                f"Specification '{os.path.basename(logical_components_path)}' contains hardcoded pixel dimensions (e.g., '150px'). Use dynamic configuration tokens instead.",
                                location=spec_rel_path,
                            ))
                except Exception as e:
                    errors.append(Finding(
                        "specification-file-must-be-readable",
                        f"Failed to read specification '{logical_components_path}': {e}",
                        location=spec_rel_path,
                    ))
                    
        return errors
        
    def _check_banned_imports(self, content: str, forbidden_words: List[str], file_ext: str) -> List[str]:
        """
        Check source code for banned import statements.

        Delegates to ``ast_check_banned_imports`` which uses Python AST
        parsing when possible and falls back to regex line-scanning for
        non-Python (JS/Dart) files.

        Args:
            content: Source code text.
            forbidden_words: List of package/module names to flag.
            file_ext: File extension hint (``.js``, ``.dart``, ``.py``).

        Returns:
            List of unique forbidden words found in imports; empty if none.
        """
        if not forbidden_words:
            return []
        return ast_check_banned_imports(content, forbidden_words)
