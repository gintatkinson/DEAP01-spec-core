<!-- Copyright Gint Atkinson, gint.atkinson@gmail.com -->

# Rule: Domain Engineering Standards

**ALWAYS enforce:** All domain models, interfaces, value objects, domain errors, and state entities in target codebases MUST adhere strictly to the 15 mandatory domain engineering standards detailed below.

## Scope and Architecture

These standards govern Tier 1 domain models and clean architecture boundaries across all supported target platforms. They ensure immutability, exhaustive error handling, static type safety, and spec-to-code traceability across downstream application codebases.

Enforced offline by `parity_auditor/validators/profile_compliance_validator.py` and platform profiles (`.pipeline/profiles/<platform>.md`).

## The 15 Non-Negotiable Domain Engineering Standards

1. **Result<T> Over Exceptions**: All fallible domain operations and repository methods MUST return explicit `Result<T>` signatures (`Success<T>` or `Failure<T>`) rather than throwing untyped runtime exceptions.
2. **Sealed Class Hierarchies**: Domain states, algebraic data types, events, and error hierarchies MUST use sealed class hierarchies (`sealed class`) to enforce exhaustive pattern matching and prevent invalid state variants.
3. **Named Constructors with Validation**: Complex domain entities MUST declare private or named constructors that perform assertion and validation logic on inputs to guarantee invalid objects cannot be instantiated.
4. **Typed Errors per Domain**: Every domain module MUST define explicit, strongly-typed error classes extending a sealed domain error base (`DomainError`) rather than returning raw error strings or untyped exceptions.
5. **@immutable Annotation Mandatory**: Every domain class, entity, value object, event, and state container MUST be annotated with `@immutable` to enforce compile-time immutability.
6. **Interface Segregation**: Domain interfaces and repository contracts MUST be narrow, lean, and highly cohesive so that clients are not forced to depend on methods they do not use.
7. **Zero dynamic**: The use of `dynamic` or untyped `Object?` in domain signatures, interfaces, properties, or variables is strictly prohibited. All data flows must be strongly typed.
8. **BDD Test Naming**: Unit and integration test names for domain logic MUST use explicit BDD behavior-driven naming patterns (`given_when_then` or `should [behavior] when [condition]`).
9. **UML Traceability Tags Mandatory (`/// Realises: [...]`)**: Every public domain class, interface, mixin, extension, or typedef header MUST include a DartDoc traceability tag (`/// Realises: [SpecName/ClassName]`) referencing its underlying specification or UML classifier.
10. **Public Member Docstrings Mandatory (`///`)**: Full DartDoc comments (`///`) are mandatory for every public class, interface, method, function, constructor, getter, and property in the domain layer.
11. **const Constructors**: Immutable domain classes and value objects with `final` fields MUST declare `const` constructors to support compile-time constant canonicalization.
12. **Value Equality (`==`/`hashCode`)**: All domain value objects and entities MUST override `operator ==` and `hashCode` (or extend `Equatable`) to guarantee value-based equality rather than reference identity.
13. **Typedefs for Callbacks**: Callback functions, listener signatures, and event handlers MUST be declared as explicit `typedef` aliases rather than raw inline function types.
14. **Private Constructors with Public Factories**: Domain entities requiring construction validation MUST restrict direct instantiation via private constructors (`._()`) and expose public factory constructors or builder methods.
15. **Separation of Serialization**: Domain models MUST remain completely decoupled from JSON, database, or network serialization logic (such as `fromJson`/`toJson`). Serialization logic MUST reside strictly in separate DTOs or data layer adapters.

## Why

Unenforced domain boundaries lead to runtime null-pointer crashes, unhandled exception leaks, fragile dynamic casting, and drift between functional specs and source implementations. Codifying and auditing these 15 rules guarantees high robustness and automated verification across the codebase.
