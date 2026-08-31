---
title: "Implementation Profile — Flutter"
project: "Digital Engineering Agent Platform (DEAP)"
tier: implementation
platform: flutter
created: "2026-06-29"
last_updated: "2026-06-29"
---

# Implementation Profile: Flutter

> This document governs feature implementation on Flutter only.
> Read alongside `.pipeline/constitution.md` (functional layer).

## Platform & Stack
- Framework: Flutter SDK 3.44.0 (stable channel)
- Language: Dart 3.12.0
- Forbidden dependencies: None explicitly prohibited. Evaluate each dependency for maintenance status and license compatibility.
- Required dependencies: `sqflite_common_ffi` (desktop SQLite), `firebase_core` + `cloud_firestore` (when targeting Firebase), `path_provider`, `http` (REST transport).

## Coding Standards
- Type safety: Use strict null safety. All variables must have explicit types; avoid `var` in public API signatures. Use `final` for immutable declarations.
- Naming conventions: Files — `snake_case.dart`. Directories — `snake_case`. Classes — `PascalCase`. Constants — `camelCase` with `const` keyword. Private members prefixed with `_`.
- Architecture pattern: MVVM (Model-View-ViewModel). Views are stateless widgets that consume ViewModels via dependency injection. ViewModels hold business logic and state. Models are data classes with serialization support.
- State management: Use `ChangeNotifier` + `ListenableBuilder` or `ValueListenableBuilder`. Avoid global state libraries unless justified by cross-cutting concerns.
- Dependency injection: Use constructor injection. Repository and DataSource instances are resolved at bootstrap via `RepositoryResolver`.
- Documentation: Full docstrings (DartDoc `///`) are mandatory for all public classes, interfaces, methods, functions, and properties, enforced via `public_member_api_docs` as a mandatory blocking linter rule.

## Testing Mandates
- Unit tests: Required for all ViewModels, domain models, and data source adapters. Command: `flutter test`. Framework: `flutter_test`.
- Integration tests: Required for all user flows. Command: `flutter test -d macos integration_test/`. Framework: `integration_test` package.
- Widget tests: Required for custom widgets with non-trivial rendering logic.
- Benchmark thresholds: <10% regression against baseline for all performance metrics.
- TDD enforcement: Test-first approach required for all data layer and domain logic. UI tests may follow implementation.

## Build & Deployment
- Build command: `cd app_flutter && flutter build <platform>` where `<platform>` is `macos`, `linux`, `windows`, `web`, `android`, or `ios`.
- Lint command: `cd app_flutter && flutter analyze` (must pass with zero errors and zero warnings).
- CI/CD: Configured per repository settings. All pushes to integration branches trigger lint + unit test + integration test pipeline.
- Deployment target: Desktop (macOS, Linux, Windows) via direct bundle. Mobile (Android, iOS) via respective app stores when configured.

## Domain Engineering Standards
All Flutter feature implementations MUST adhere strictly to the 15 mandatory domain engineering standards:
1. **Result<T> Over Exceptions** — All fallible domain operations and repository methods MUST return explicit `Result<T>` signatures (`Success<T>` or `Failure<T>`) rather than throwing untyped runtime exceptions.
2. **Sealed Class Hierarchies** — Domain states, algebraic data types, events, and error hierarchies MUST use sealed class hierarchies (`sealed class`) to enforce exhaustive pattern matching and prevent invalid state variants.
3. **Named Constructors with Validation** — Complex domain entities MUST declare private or named constructors that perform assertion and validation logic on inputs to guarantee invalid objects cannot be instantiated.
4. **Typed Errors per Domain** — Every domain module MUST define explicit, strongly-typed error classes extending a sealed domain error base (`DomainError`) rather than returning raw error strings or untyped exceptions.
5. **@immutable Annotation Mandatory** — Every domain class, entity, value object, event, and state container MUST be annotated with `@immutable` to enforce compile-time immutability.
6. **Interface Segregation** — Domain interfaces and repository contracts MUST be narrow, lean, and highly cohesive so that clients are not forced to depend on methods they do not use.
7. **Zero dynamic** — The use of `dynamic` or untyped `Object?` in domain signatures, interfaces, properties, or variables is strictly prohibited. All data flows must be strongly typed.
8. **BDD Test Naming** — Unit and integration test names for domain logic MUST use explicit BDD behavior-driven naming patterns (`given_when_then` or `should [behavior] when [condition]`).
9. **UML Traceability Tags Mandatory (`/// Realises: [...]`)** — Every public domain class, interface, mixin, extension, or typedef header MUST include a DartDoc traceability tag (`/// Realises: [SpecName/ClassName]`) referencing its underlying specification or UML classifier.
10. **Public Member Docstrings Mandatory (`///`)** — Full DartDoc comments (`///`) are mandatory for every public class, interface, method, function, constructor, getter, and property in the domain layer.
11. **const Constructors** — Immutable domain classes and value objects with `final` fields MUST declare `const` constructors to support compile-time constant canonicalization.
12. **Value Equality (`==`/`hashCode`)** — All domain value objects and entities MUST override `operator ==` and `hashCode` (or extend `Equatable`) to guarantee value-based equality rather than reference identity.
13. **Typedefs for Callbacks** — Callback functions, listener signatures, and event handlers MUST be declared as explicit `typedef` aliases rather than raw inline function types.
14. **Private Constructors with Public Factories** — Domain entities requiring construction validation MUST restrict direct instantiation via private constructors (`._()`) and expose public factory constructors or builder methods.
15. **Separation of Serialization** — Domain models MUST remain completely decoupled from JSON, database, or network serialization logic (such as `fromJson`/`toJson`). Serialization logic MUST reside strictly in separate DTOs or data layer adapters.

## Platform Audit Gates

These are the platform-specific constraints the parity auditor enforces against a
Flutter workspace. They live here rather than in `rules/` because each one names the
Flutter source tree, a Flutter widget, or Flutter UI directories, and would be
meaningless on another platform — see `rules/platform-independence.md`
§ *Where platform-specific details belong*. Enforced offline by
`parity_auditor/validators/profile_scoping_validator.py`,
`parity_auditor/validators/schema_mapping_validator.py`, and
`parity_auditor/validators/profile_compliance_validator.py`.

- **Profile Scoping Requires Platform Sources**: profile-compliance auditing runs over
  the Flutter source tree named by `target_directories.flutter`. A workspace with no
  matching source files is reported rather than passed. A silent pass would be
  indistinguishable from full compliance, and across a corpus of downstream projects an
  empty codebase is itself the finding worth aggregating.
- **Splitter Widgets Require Pointer Gesture Listeners**: any Dart source implementing a
  splitter — by filename or by containing `Splitter` — MUST attach pointer gesture event
  handling via `Listener` or `GestureDetector`. A splitter without one renders as a
  divider that cannot be dragged, so the layout requirement is met visually and not
  functionally.
- **Schema Mapping Requires Platform Sources**: schema-to-codebase mapping likewise runs
  over the Flutter source tree, and reports an empty codebase rather than reporting that
  every schema field is unmapped. Stated separately from the profile-scoping precondition
  above because the two gates fail independently and a grouped multi-workspace report
  must say which one fired.
- **Schema Fields Must Be Realised In The Codebase**: every container, list, grouping,
  typedef, identity, RPC, action, notification and leaf defined in the workspace schema
  modules MUST appear in the Flutter sources as a declaration of the corresponding kind.
  A schema node with no realisation is specified and unimplemented, which the schema is
  the source of truth for.
- **Schema Fields Must Be Bound To A UI Component**: a schema field realised in the
  codebase but absent from every file under `flutter_rules.ui_directories` is reachable
  in the data layer and invisible to the operator. Where the workspace declares UI
  directories at all, realisation without a UI binding is reported.
- **Public Member Docstrings Mandatory**: every public class, interface, method, function,
  and public property MUST include full docstrings (DartDoc ///, JSDoc /** */, Python """).
  Enforced as a mandatory blocking linter rule via `public_member_api_docs`.
- **UML Traceability Tags Mandatory**: Every public Dart class MUST include a DartDoc comment citing its underlying specification or UML class (e.g., /// Realises: [Feat-002/VirtualCameraNormalization]).
- **Domain Immutable Annotations Mandatory**: Every public domain model class MUST include an @immutable annotation.
- **Domain Result Signatures Mandatory**: Every fallible domain operation MUST return a Result<T> type signature.

## Security & Ops
- API key management: API keys and secrets MUST NOT be committed to the repository. Use platform-specific secure storage (e.g., macOS Keychain, Android Keystore) or environment variables resolved at build time via `--dart-define`.
- Auth provider: Authentication is injected at the DataSource layer. The abstract `DataSource` interface does not prescribe an auth mechanism. Concrete implementations may use token-based, certificate-based, or unauthenticated access as appropriate.
- Data protection: All database files stored in the platform's application support directory. No sensitive data stored without encryption. Transport security (TLS) enforced for all remote data source connections.
- Logging: Structured logging via `dart:developer` in debug builds only. No personally identifiable information (PII) may be logged.

## LUI Resolution Guidelines
Micro-task implementers MUST follow these guidelines to resolve unbound specification bindings (`Unbound (Deferred to Implementation Profile)`) into concrete Flutter UI components during implementation profile execution:
1. *Layout Manifest Resolution*: Inspect `.pipeline/logical-ui/logical-layout.json` (or `app_flutter/assets/logical-layout.json`) to determine the target container ID and component structure.
2. *Unbound Binding Resolution*:
   - If a specification sets `Target Interface Component` or `Target Container / Endpoint` to `Unbound (Deferred to Implementation Profile)`, the micro-task implementer is responsible for mapping the feature data fields to concrete UI widgets.
   - Key-value attributes and schema properties MUST map to `PropertyGrid` or `ConfigurationForm` inside containers such as `details_and_relations_tab`, `properties_view`, or `elements_view`.
   - Tabular, multi-item, or collection data fields MUST map to `TableView` or `DensityTable` inside containers such as `elements_view` or `components_table`.
   - Numeric inputs, ranges, or bounded parameters MUST map to `NumericSpinBox` or specialized numeric input controls.
3. *Data Source Binding Resolution*: Authoritative schema paths (e.g. `/nwi:network-inventory/...` or `schema:...`) specified in the feature MUST be bound directly to the corresponding Flutter ViewModel properties and widget controllers.


