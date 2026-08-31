#!/usr/bin/env python3
# Copyright Gint Atkinson, gint.atkinson@gmail.com
"""Aggregate pipeline symptoms across multiple downstream workspaces (issue #301).

Usage:
    ./scripts/aggregate_downstream_symptoms.py <workspace> [<workspace> ...]

Downstream content is a diagnostic instrument, not a deliverable. This reports which
symptoms recur across the corpus, because breadth is what distinguishes a pipeline
defect from a local accident.

Exit code is 0 when no symptom crosses the systemic threshold, 1 when one does. A
non-zero exit means "the pipeline has a defect", not "a downstream project is broken".
"""

import os
import sys

sys.path.insert(
    0,
    os.path.abspath(
        os.path.join(
            os.path.dirname(__file__),
            "..", "skills", "spec-orchestrator", "parity_auditor", "src",
        )
    ),
)

from parity_auditor.aggregator import collect, format_report  # noqa: E402


def main(argv):
    if len(argv) < 2:
        print(__doc__.strip())
        return 2
    report = collect(argv[1:])
    print(format_report(report))
    if not report.corpus:
        print("\n[!] No readable workspaces in the corpus.", file=sys.stderr)
        return 2
    if len(report.corpus) < 2:
        print(
            "\n[-] Note: a single workspace cannot establish recurrence. Pass two or "
            "more to distinguish pipeline defects from local accidents.",
            file=sys.stderr,
        )
    return 1 if report.systemic else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
