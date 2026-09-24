# Adversarial Code Auditor Skill (Local Copy)

Source: skills/adversarial-code-auditor/SKILL.md

Refer to upstream copy at /Users/perkunas/jail/DEAP01-spec-core/skills/adversarial-code-auditor/SKILL.md.
Methodology: Pre-emptive adversarial audit against four/five correctness risk pillars (Memory Safety, Resource Lifecycle, Concurrency, Test Integrity, Semantic Traceability).
Protocol: Read, Audit, Write, Verify, File.
Output: 7-section verified defect dossier adhering strictly to Section 2 of SKILL.md.
Verification: Step D checks 1-12, including Check 7 offline Mermaid syntax validation.
Filing: python3 scripts/file_defect.py or gh/glab CLI.
