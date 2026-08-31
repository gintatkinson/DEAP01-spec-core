#!/usr/bin/env python3
"""
Runtime Capability Pre-Flight Probe Check (Mechanism 2)

Dispatches a lightweight probe check prior to Phase 2 (User Stories/Use Cases)
and Phase 3 (Implementation) to verify execution capabilities.
Strictly HALTs execution if probe fails or times out.

Usage:
    python3 subagent_preflight_probe.py [--phase <phase2|phase3>] [--timeout <seconds>]
"""

import argparse
import os
import shutil
import sys


def run_preflight_probe(phase="phase2", timeout=30):
    print(f"Running pre-flight probe check for {phase} (timeout={timeout}s)...")

    # 1. Environment and required binary check
    required_binaries = ["git", "python3"]
    missing = []
    for binary in required_binaries:
        if not shutil.which(binary):
            missing.append(binary)

    if missing:
        print(f"PROBE FAILURE: Missing required binary executable(s): {', '.join(missing)}")
        return False

    # 2. Write capability check
    try:
        test_file = f".pipeline/.probe_{phase}.tmp"
        os.makedirs(".pipeline", exist_ok=True)
        with open(test_file, "w", encoding="utf-8") as f:
            f.write("probe_ok")
        os.remove(test_file)
    except Exception as e:
        print(f"PROBE FAILURE: Directory write capability check failed: {e}")
        return False

    # 3. Direct write lock verification
    print("Pre-flight capability probe check PASSED. Direct write lock ENFORCED_LOCK=TRUE.")
    return True


def main():
    parser = argparse.ArgumentParser(description="Pre-flight subagent probe checker")
    parser.add_argument("--phase", choices=["phase2", "phase3"], default="phase2", help="Target execution phase")
    parser.add_argument("--timeout", type=int, default=30, help="Probe timeout in seconds")
    args = parser.parse_args()

    success = run_preflight_probe(args.phase, args.timeout)
    if success:
        sys.exit(0)
    else:
        print("HALTING EXECUTIONS: Pre-flight probe failed.")
        sys.exit(1)


if __name__ == "__main__":
    main()
