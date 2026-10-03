#!/usr/bin/env bash
# ==============================================================================
# DEAP Safety-Critical Engineering Pipeline Bootstrap Installer
# Delegates execution to modular Python package: scripts/installer/
# ==============================================================================
set -euo pipefail

cleanup() {
  local exit_code=$?
  if [ $exit_code -ne 0 ]; then
    echo "Installer wrapper exiting with code ${exit_code}." >&2
  fi
  exit $exit_code
}

trap cleanup EXIT
trap 'echo "Installation interrupted by user." >&2; exit 130' INT
trap 'echo "Installation terminated by signal." >&2; exit 143' TERM

# 1. Validate host prerequisites
if ! command -v git >/dev/null 2>&1; then
  echo "ERROR: 'git' command not found. Git is required to install the pipeline." >&2
  exit 1
fi

PYTHON_EXEC=""
for cand in "${PYTHON_BIN:-}" python3 python3.14 python3.13 python3.12 python3.11 python3.10 /opt/homebrew/bin/python3 /usr/local/bin/python3; do
  if [ -n "$cand" ] && command -v "$cand" >/dev/null 2>&1; then
    ver_str="$("$cand" -V 2>&1 | cut -d' ' -f2)"
    maj="$(echo "$ver_str" | cut -d'.' -f1)"
    min="$(echo "$ver_str" | cut -d'.' -f2)"
    if [ "$maj" -gt 3 ] || { [ "$maj" -eq 3 ] && [ "$min" -ge 10 ]; }; then
      PYTHON_EXEC="$cand"
      break
    fi
  fi
done

if [ -z "$PYTHON_EXEC" ]; then
  found_ver="$(python3 -V 2>&1 || true)"
  echo "ERROR: Python 3.10 or higher is required. Found Python: ${found_ver}" >&2
  exit 1
fi

# 2. Resolve repository root and configure PYTHONPATH
INSTALLER_ROOT="$(cd -P "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
export PYTHONPATH="${INSTALLER_ROOT}:${PYTHONPATH:-}"

# 3. Delegate execution directly to the modular Python installer CLI
exec "$PYTHON_EXEC" -m scripts.installer.cli "$@"
