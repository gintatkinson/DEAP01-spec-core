<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Test-Driven Development (TDD)

**ALWAYS enforce:** All implementation MUST follow the RED-GREEN-REFACTOR cycle.

## The cycle

1. **RED:** Write a failing test FIRST. Run it. Confirm it fails.
2. **GREEN:** Write the minimal code to make the test pass. Run it. Confirm it passes.
3. **REFACTOR:** Clean up the code while keeping tests green.

## Hard constraints

- Code written before its corresponding failing test must be deleted and re-implemented after the test.
- Never skip the "confirm it fails" step — a test that passes before implementation is not a valid driving test.
- **A compile error is not a RED phase.** A test that does not compile has not run, so it evidences nothing: a correctly-failing test and a broken one are indistinguishable. RED means an executed test failing on an assertion, with the failure message recorded.
- **Do not let a compile failure mask a behavioural one.** Where a task adds new symbols *and* corrects existing behaviour, land the tests that compile against the existing symbols first and observe them fail on their assertions. Those failures are the evidence the task exists. Bundling them behind an undefined-symbol error throws that evidence away and reports only that the new names are absent, which was never in doubt.
- Each micro-task (2-5 minutes of work) must have a driving test specified before execution begins.
- Use the test framework specified in the project's implementation profile (`.pipeline/profiles/<platform>.md`).

## Required assertion classes

A test suite can be green, large, and still assert nothing about the constraints the
specifications actually state. These are the assertion classes the parity auditor
requires a downstream test suite to demonstrate, enforced offline by
`parity_auditor/validators/test_completeness_validator.py`. They are stated here rather
than in a platform profile because the checker scans every supported test file type
(`_test.dart`, `.test.ts`, `.test.tsx`, `.spec.ts`, `.spec.tsx`) and the requirement is
about what is asserted, not about which framework asserts it.

- **Test Suite Must Exist**: a workspace with no discoverable test files fails outright.
  A suite that does not exist cannot be said to pass, and reporting the absence is what
  distinguishes an untested project from a compliant one.
- **Regex Pattern Assertions Required**: BDD acceptance criteria that constrain a string
  format MUST be verified by a pattern-matching assertion. Asserting only equality on one
  example value confirms the example, not the constraint.
- **Numerical Precision Assertions Required**: specifications that state decimal places or
  tolerances MUST be verified with a precision-aware assertion. Exact float equality
  either over-constrains or silently passes, depending on the platform's arithmetic.
- **Computed Style Assertions Required**: layout highlight and selection states MUST be
  verified against computed style rather than against the presence of a class name or
  flag. A widget can carry the state and render nothing.
- **Layout Size Assertions Required**: minimum widths and pane dimension constraints
  stated in a specification MUST be asserted. These are the constraints that regress
  invisibly, because a broken layout still renders.
- **Exception Path Assertions Required**: every specified failure or validation-error path
  MUST have an assertion that the failure actually occurs. A suite exercising only the
  happy path leaves the error contract unverified.

## Why

TDD prevents false confidence. A test written after the code is confirmation bias — it tests what was built, not what was specified. RED-GREEN-REFACTOR guarantees the test actually validates the requirement.
