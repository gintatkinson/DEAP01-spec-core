# Solution Walkthrough: README Installation Guide Fix

This document details the fix applied to the clone commands in the project's README installation instructions and explains the technical rationale for this adjustment.

---

## 1. Code Realization Table

| Feature / Area | Source File | Lines Affected | Rationale |
| :--- | :--- | :--- | :--- |
| Installation Guide (Stable) | `README.md` | 146 | Replace placeholder `<owner>/<template-repo>` with `gintatkinson/DEAP-spec-core` |
| Installation Guide (Refactored) | `README.md` | 158 | Replace placeholder `<owner>/<template-repo>` with `gintatkinson/DEAP-spec-core` |

---

## 2. Walkthrough of Changes

The installation instructions in the root `README.md` guide users to copy the project's configurations and skills into their active repository workspaces by cloning the pipeline repository. 

### 2.1. Stable Version Installation (`master` branch)
Previously, the clone command at line 146 was specified as:
```bash
git clone https://github.com/<owner>/<template-repo>.git ./.tmp-pipeline
```
This was updated to use the actual repository path:
```bash
git clone https://github.com/gintatkinson/DEAP-spec-core.git ./.tmp-pipeline
```

### 2.2. Refactored Version Installation (`refactor` branch)
Previously, the clone command at line 158 was specified as:
```bash
git clone -b refactor https://github.com/<owner>/<template-repo>.git ./.tmp-pipeline
```
This was updated to use the actual repository path:
```bash
git clone -b refactor https://github.com/gintatkinson/DEAP-spec-core.git ./.tmp-pipeline
```

---

## 3. Technical Rationale

The use of angle-bracket placeholders like `<owner>/<template-repo>` in bash/sh code blocks poses several critical operational issues:

1. **Shell Redirection Errors**:
   The `<` and `>` characters are interpreted by most Unix-like shells (such as bash, zsh, and sh) as input and output redirection operators. If a developer copies and runs the command exactly as written, the shell interprets it as redirecting input from a file named `owner` and redirecting output to a file named `template-repo`. This results in syntax errors or command failures (e.g., `No such file or directory` or creating unintended empty files).

2. **Integration & Automation Failures**:
   Autonomous coding agents or automated setup scripts parsing code blocks from documentation and executing them directly will fail or produce incorrect side effects when executing placeholders. Replacing these placeholders with the concrete repository URL ensures the setup commands can be executed safely, idempotently, and without manual intervention.

---

## 4. Verification

1. Checked syntax and layout using local git tools.
2. Verified that `README.md` modifications were correct via diff review.
