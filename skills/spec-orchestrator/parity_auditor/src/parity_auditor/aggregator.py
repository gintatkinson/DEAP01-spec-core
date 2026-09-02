"""Aggregate symptoms across multiple downstream workspaces (issue #301).

The working model for this pipeline is: run it against several downstream projects, read
the symptoms in their output, and isolate the shared upstream cause. Downstream content is
a diagnostic instrument and is discarded.

``verify_downstream_baseline.py`` answers *"is this one workspace healthy"*. This answers
*"what does this corpus of workspaces tell me about the pipeline"*, which is a different
question and the one that distinguishes a pipeline defect from a local accident.

**Breadth is the signal.** A symptom present in 5 of 5 workspaces is almost certainly an
upstream defect. One present in 1 of 5 is probably local to that project. Ranking by breadth
rather than by raw count keeps a single workspace with 200 instances of one rule from
drowning out a rule that quietly appears everywhere.

The strongest evidence in the audit that motivated this issue came from recurrence: seven
separate issues turned out to be one defect class, but that only became visible after the
seventh, and only because one reader happened to hold all seven in mind at once.
"""

import os
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Sequence

from .core.findings import Finding, unmigrated_count
from .core.workspace import WorkspaceRepository
from .validators.behavioral import BehavioralValidator
from .validators.cardinality_validator import SchemaCardinalityValidator
from .validators.codebase import CodebaseValidator
from .validators.dependency_validator import DependencyValidator
from .validators.docs import DocsValidator
from .validators.logical_ui_validator import LogicalUiValidator
from .validators.mermaid_syntax_validator import MermaidSyntaxValidator
from .validators.profile_scoping_validator import ProfileScopingValidator
from .validators.schema_mapping_validator import SchemaMappingValidator
from .validators.spec_filename_validator import SpecFilenameValidator
from .validators.spec_title_uniqueness_validator import SpecTitleUniquenessValidator
from .validators.source_reference_validator import SourceReferenceValidator
from .validators.link_validator import LinkValidator
from .validators.spec_validator import SpecValidator
from .validators.uml import UmlValidator
from .validators.test_completeness_validator import TestCompletenessValidator
from .validators.docstring_validator import DocstringValidator
from .validators.profile_compliance_validator import ProfileComplianceValidator
from .validators.dispatch_preamble_validator import DispatchPreambleValidator
from .validators.plan_validator import PlanValidator
from .validators.concept_provenance_validator import ConceptProvenanceValidator
from .validators.safety_trace_validator import SafetyTraceValidator
from .validators.doc_metadata_validator import DocMetadataValidator
from .validators.icd_completeness_validator import ICDCompletenessValidator
from .validators.operational_allocation_validator import OperationalAllocationValidator
from .validators.standards_measurement_validator import StandardsAndMeasurementValidator
from .validators.conops_completeness_validator import ConopsCompletenessValidator, MissionIntentCompletenessValidator
from .validators.research_inventory_validator import ResearchInventoryValidator
from .validators.coverage_digest_validator import CoverageDigestValidator
from .validators.obligation_witness_validator import ObligationWitnessValidator

# Validators migrated to structured findings. Un-migrated validators are deliberately
# excluded rather than included and silently ungroupable — see `coverage_note`.
# Migration progress is gated by tests/test_validator_findings_migration_issue304.py,
# which asserts that every module listed as migrated is also listed here: wrapping an
# emission in Finding without wiring the validator in produces rule ids that reach no
# report, which looks like progress and is not.
AGGREGATING_VALIDATORS = (
    MermaidSyntaxValidator,
    SpecFilenameValidator,
    SpecTitleUniquenessValidator,
    SourceReferenceValidator,
    LinkValidator,
    SchemaCardinalityValidator,
    SpecValidator,
    DependencyValidator,
    ProfileScopingValidator,
    SchemaMappingValidator,
    TestCompletenessValidator,
    BehavioralValidator,
    DocsValidator,
    LogicalUiValidator,
    CodebaseValidator,
    UmlValidator,
    DocstringValidator,
    ProfileComplianceValidator,
    DispatchPreambleValidator,
    PlanValidator,
    ConceptProvenanceValidator,
    SafetyTraceValidator,
    DocMetadataValidator,
    ICDCompletenessValidator,
    OperationalAllocationValidator,
    StandardsAndMeasurementValidator,
    ConopsCompletenessValidator,
    MissionIntentCompletenessValidator,
    ResearchInventoryValidator,
    CoverageDigestValidator,
    ObligationWitnessValidator,
)

# SyncValidator is migrated to structured findings but deliberately absent: it shells
# out to the issue tracker, and `pipeline-tooling.md` § Validation Gates forbids network
# egress inside a blocking gate. See AGGREGATION_EXEMPT in
# tests/test_validator_findings_migration_issue304.py.


@dataclass
class SymptomGroup:
    """One rule, and the workspaces exhibiting it."""

    signature: str
    workspaces: Dict[str, List[str]] = field(default_factory=dict)

    @property
    def breadth(self) -> int:
        """Number of distinct workspaces exhibiting this symptom."""
        return len(self.workspaces)

    @property
    def total(self) -> int:
        return sum(len(v) for v in self.workspaces.values())

    def is_systemic(self, corpus_size: int) -> bool:
        """Present in more than half the corpus, and in at least two workspaces.

        Two is the floor because a single workspace proves nothing about the pipeline.
        """
        return self.breadth >= 2 and self.breadth * 2 > corpus_size


@dataclass
class AggregateReport:
    corpus: List[str]
    groups: List[SymptomGroup]
    unmigrated: int = 0

    @property
    def systemic(self) -> List[SymptomGroup]:
        return [g for g in self.groups if g.is_systemic(len(self.corpus))]

    def coverage_note(self) -> str:
        if self.unmigrated:
            return (
                f"{self.unmigrated} finding(s) carried no rule id and were excluded from "
                f"grouping. Only {len(AGGREGATING_VALIDATORS)} validators emit structured "
                f"findings so far; the rest still return plain strings and cannot be "
                f"aggregated. Migration progress is visible here rather than assumed."
            )
        return (
            f"All findings from the {len(AGGREGATING_VALIDATORS)} aggregating validators "
            f"carried rule ids. Validators not yet migrated contribute nothing to this "
            f"report."
        )


def collect(workspace_paths: Sequence[str]) -> AggregateReport:
    """Run the aggregating validators over each workspace and group by signature."""
    groups: Dict[str, SymptomGroup] = {}
    corpus: List[str] = []
    unmigrated = 0

    for path in workspace_paths:
        abs_path = os.path.abspath(path)
        if not os.path.isdir(abs_path):
            continue
        label = os.path.basename(abs_path.rstrip(os.sep)) or abs_path
        corpus.append(label)

        repo = WorkspaceRepository(workspace_dir=abs_path)
        dispatch_payload_text: Optional[str] = None
        dispatch_payload_path = os.environ.get("DISPATCH_PAYLOAD_PATH")
        if dispatch_payload_path and os.path.isfile(os.path.abspath(dispatch_payload_path)):
            try:
                with open(
                    os.path.abspath(dispatch_payload_path), "r", encoding="utf-8"
                ) as payload_file:
                    dispatch_payload_text = payload_file.read()
            except OSError:
                dispatch_payload_text = None
        findings: List[str] = []
        for validator_cls in AGGREGATING_VALIDATORS:
            try:
                validator_kwargs: Dict[str, str] = {}
                if (
                    validator_cls is DispatchPreambleValidator
                    and dispatch_payload_text is not None
                ):
                    validator_kwargs["prompt_text"] = dispatch_payload_text
                findings.extend(validator_cls().validate(repo, **validator_kwargs))
            except Exception as exc:  # a broken workspace must not abort the corpus
                findings.append(
                    Finding(
                        "aggregator-validator-error",
                        f"{label}: {validator_cls.__name__} raised {type(exc).__name__}: {exc}",
                        location=label,
                    )
                )

        unmigrated += unmigrated_count(findings)
        for finding in findings:
            if not isinstance(finding, Finding):
                continue
            group = groups.setdefault(finding.signature(), SymptomGroup(finding.signature()))
            group.workspaces.setdefault(label, []).append(str(finding))

    ordered = sorted(
        groups.values(),
        key=lambda g: (-g.breadth, -g.total, g.signature),
    )
    return AggregateReport(corpus=corpus, groups=ordered, unmigrated=unmigrated)


def format_report(report: AggregateReport) -> str:
    """Render a triage table, widest-recurrence first."""
    lines: List[str] = []
    n = len(report.corpus)
    lines.append("=== Multi-Downstream Symptom Aggregation ===")
    lines.append(f"Corpus: {n} workspace(s) -> {', '.join(report.corpus) or '(none)'}")
    lines.append("")

    if not report.groups:
        lines.append("No structured findings across the corpus.")
        lines.append("")
        lines.append(report.coverage_note())
        return "\n".join(lines)

    lines.append(f"{'BREADTH':>8}  {'TOTAL':>6}  {'SYSTEMIC':>8}  RULE")
    for g in report.groups:
        flag = "yes" if g.is_systemic(n) else "-"
        lines.append(f"{g.breadth:>6}/{n}  {g.total:>6}  {flag:>8}  {g.signature}")

    lines.append("")
    systemic = report.systemic
    if systemic:
        lines.append(
            f"{len(systemic)} symptom(s) appear in more than half the corpus and in at "
            f"least two workspaces. These are pipeline defects, not local accidents:"
        )
        for g in systemic:
            lines.append(f"  - {g.signature}  ({g.breadth}/{n} workspaces)")
    else:
        lines.append(
            "No symptom crosses the systemic threshold. Every finding is confined to "
            "fewer than half the corpus, so treat each as local until it recurs."
        )

    lines.append("")
    lines.append(report.coverage_note())
    return "\n".join(lines)
