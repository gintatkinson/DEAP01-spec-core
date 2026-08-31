"""Structured validator findings (issue #301).

Validators historically returned ``List[str]`` of English sentences. That is fine for a
human reading one workspace, but it defeats the purpose of aggregating symptoms across
several downstream projects: deciding that two messages describe *the same kind* of
problem requires parsing prose, which breaks the moment a message is reworded.

``Finding`` **subclasses ``str``**. That is deliberate and load-bearing. A ``Finding``
*is* its message, so every existing consumer keeps working unchanged — ``" ".join(errors)``,
``"text" in error``, equality, f-string interpolation, sorting. Roughly 172 tests assert on
message content in those forms; without this the migration would require rewriting all of
them, and a rewrite of that size is where regressions come from.

What it adds is ``rule_id``, a stable identifier the aggregator groups on, and
``signature()``, which is deliberately **independent of the downstream project** so the
same defect in two differently-named repositories collapses to one row.
"""

from typing import Any, Dict, Optional


class Finding(str):
    """A validator message that also knows which rule produced it.

    Behaves as the message string everywhere a string is expected.

    >>> f = Finding("mermaid-no-semicolon", "docs/a.md:3: semicolon in Note", "docs/a.md:3")
    >>> str(f) == "docs/a.md:3: semicolon in Note"
    True
    >>> "semicolon" in f
    True
    >>> f.rule_id
    'mermaid-no-semicolon'
    """

    rule_id: str
    location: str
    detail: Dict[str, Any]

    def __new__(
        cls,
        rule_id: str,
        message: str,
        location: str = "",
        detail: Optional[Dict[str, Any]] = None,
    ) -> "Finding":
        if not rule_id:
            # A finding without a rule id is invisible to aggregation, which is the
            # whole point of the type. Fail loudly at construction rather than
            # producing a row that silently never groups.
            raise ValueError(f"Finding requires a rule_id (message: {message!r})")
        obj = super().__new__(cls, message)
        obj.rule_id = rule_id
        obj.location = location
        obj.detail = dict(detail or {})
        return obj

    def signature(self) -> str:
        """Downstream-independent identity, used to group across workspaces.

        Only the rule id participates. File paths, ordinals and project names are
        deliberately excluded: ``feat-04`` colliding in project A and ``feat-11``
        colliding in project B are the *same* pipeline defect and must aggregate to one
        row. Including the location would scatter them into separate rows and hide the
        recurrence, which is the signal worth having.
        """
        return self.rule_id

    def __repr__(self) -> str:  # pragma: no cover - diagnostic only
        return f"Finding(rule_id={self.rule_id!r}, message={str(self)!r})"

    # str is immutable and these must survive slicing/copying only as the plain message,
    # which is correct: a sliced message is no longer a whole finding.


def rule_ids(findings) -> set:
    """Rule ids present in a mixed list of Findings and plain strings.

    Un-migrated validators still return plain strings; those contribute nothing and are
    skipped rather than guessed at.
    """
    return {f.rule_id for f in findings if isinstance(f, Finding)}


def unmigrated_count(findings) -> int:
    """How many entries carry no rule id, so coverage is visible rather than assumed."""
    return sum(1 for f in findings if not isinstance(f, Finding))
