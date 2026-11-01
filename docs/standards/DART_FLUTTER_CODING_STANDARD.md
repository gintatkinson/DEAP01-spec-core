# DEAP Dart & Flutter Coding Standard

| Field | Value |
| :--- | :--- |
| Standard ID | `DEAP-DART-FLUTTER-CODING-STANDARD` |
| Version | `2.0` |
| Status | `Active / Normative` |
| Supersedes | `1.0` |
| Applies to | All Dart and Flutter client, operator console, and HMI code under `app_flutter/` in DEAP downstream application repositories |
| Target assurance | Ring-3 DO-178C DAL-D/E / ISO 26262 QM assurance containment, DO-178C DAL B/C HMI display/control determinism, ARINC 661 CDS/UA & Ring-2/Ring-1 FFI boundary isolation |
| Companion standards | [RUST_CODING_STANDARD.md](./RUST_CODING_STANDARD.md), [README.md](./README.md) |
| Owner | DEAP Systems Engineering |
| Review requirement | Fresh reviewer subagent and explicit user approval before adoption |

## 1. Purpose, Regulatory Scope & Target Confinement

This standard defines mandatory Dart and Flutter architectural, domain-modeling, documentation, concurrency, resource-lifecycle, persistence, viewport-ergonomics, and verification rules for DEAP downstream operator consoles and Human-Machine Interface (HMI) client targets.

### 1.1 Regulatory Scope and Precedence

- **Safety-Critical HMI Readiness**: Rules in this document target DO-178C DAL B/C display and operator-control readiness and ARINC 661 Cockpit Display System / User Application (CDS/UA) partition boundary isolation, ensuring deterministic rendering, bounded frame latency, explicit degraded-mode indication, and zero UI-induced corruption of safety kernels.
- **Governance Precedence**: This standard is subordinate to the project constitution (`.pipeline/constitution.md`), `.pipeline/ACTIVE_RULES_BUNDLE.md`, and active platform profiles. Where two rules overlap, the stricter rule governs. Conflicts must be resolved via an approved remediation plan, never silently bypassed.
- **Evidence Over Assertion**: Compliance with this coding standard does not by itself constitute DO-178C or ARINC 661 certification. Any certification or assurance readiness claim requires complete requirement-to-code-to-test traceability, structural coverage, live persistence verification, and frame-time profiling evidence.
- **Cross-Language Boundary**: Tier-0 DO-330 offline compiler/verification crates, Ring-1 `#![no_std]` Ferrocene Rust safety kernels, Track B discrete-time Rust simulation kernels, and the native side of Ring-2 `#[repr(C)]` / `flutter_rust_bridge` FFI adapters are governed by [RUST_CODING_STANDARD.md](./RUST_CODING_STANDARD.md). Shared cross-language assurance invariants and the 5-stage compiler/formal verification hierarchy are indexed in [README.md](./README.md).

### 1.2 Strict Target Directory Confinement (`app_flutter/`)

- All Dart source files (`lib/**`), automated tests (`test/**`, `integration_test/**`), asset bundles, platform runner wrappers, `pubspec.yaml`, `pubspec.lock`, and `analysis_options.yaml` MUST reside strictly within `app_flutter/`.
- Writing Dart source files, generated serialization adapters, configuration files, or build outputs at the repository root or outside `app_flutter/` is strictly prohibited.
- When integrating with a downstream Rust control or simulation engine via `dart:ffi` / `flutter_rust_bridge`, generated and consumer FFI bindings reside exclusively inside `app_flutter/lib/src/ffi/`, pass `dart analyze --fatal-infos` with zero diagnostics, and consume read-only snapshot buffers and bounded command ring buffers without mutating native engine heap memory directly.

### 1.3 Ring-3 Assurance Containment & Ring-2/Ring-1 FFI Boundary Rules

Per `docs/designs/blueprint-09-clippy-dylint-miri-kani-formal-verification-gate.md`, `docs/designs/blueprint-11-ferrocene-do178c-do330-3ring-certification-architecture.md`, and `docs/architecture/blueprints/DEAP_STRIX_HALO_COMPILER_NATIVE_VERIFICATION_ARCHITECTURE.md`, downstream operator consoles enforce strict 3-Ring Assurance Containment:

- **Ring-3 Untrusted Presentation & Input Shell (`app_flutter/` -- DAL-D/E / QM Containment)**: The Flutter runtime, Skia/Impeller GPU renderer, and Dart garbage-collected isolate heap operate strictly inside a Ring-3 DO-178C DAL-D/E / ISO 26262 QM assurance-containment envelope. Ring-3 functions solely as a deterministic display projection and untrusted operator-input shell; it never executes safety-critical control laws, authority arbitration, or unguarded actuator writes.
- **Ring-2 Synthesized FFI Guard Isolation (`flutter_rust_bridge` + `Miri`)**: Ring-3 is physically and logically isolated from Ring-1 (`#![no_std]` Ferrocene Rust `safety_kernel`) by Ring-2 -- a narrow `#[repr(C)]` and `crates/compile-sysml`-synthesized `flutter_rust_bridge` FFI guard layer. On the Rust side, Ring-2 traps panics via `std::panic::catch_unwind`, validates scalar bounds and enum discriminants before constructing Ring-1 types, and is verified for memory/aliasing soundness under `cargo +nightly miri test` (`-Zmiri-tree-borrows`). On the Dart side (`app_flutter/lib/src/ffi/**`), all synthesized and wrapper FFI bindings MUST pass `dart analyze --fatal-infos` without `// ignore:` suppressions.
- **Ring-1 `SafetyMonitor` Sequence-Numbered CRC/SHA-256 Readback & Challenge-Response Interlocks**:
  - Every safety-critical operator command initiated in Ring-3 MUST traverse a deterministic two-step **Propose -> Ring-1 Readback/Challenge -> Operator Confirm -> Ring-1 Authorize** interlock enforced by the Ring-1 `SafetyMonitor`.
  - Command envelopes and Ring-1 safety annunciations carry monotonic sequence numbers, freshness epochs, and CRC-32C or SHA-256 integrity digests computed and verified inside Ring-1.
  - Ring-3 confirmation views and annunciation banners MUST render the Ring-1-echoed readback payload and digest-verified state verbatim rather than optimistic local ViewModel state, transitioning immediately to an explicit `Failure<T>` / stale-interlock state on any sequence gap, timeout, or CRC/SHA-256 mismatch.

## 2. Architectural Isolation & MVVM 3-Layer Definition of Done

All downstream Flutter applications enforce strict **MVVM (Model-View-ViewModel)** unidirectional dependency flow:

`Presentation (Views / Widgets) -> State Management (ViewModels) -> Domain (Entities / Value Objects / Repository Contracts) <- Data & Adapters (Live SQLite / FFI / Register Maps / DTOs)`

### 2.1 Layer Isolation Rules

- **Pure Domain Core (`app_flutter/lib/src/domain/**`)**: Domain modules MUST NOT import `package:flutter/material.dart`, `package:flutter/widgets.dart`, `package:flutter/cupertino.dart`, `dart:ui`, `dart:ffi`, or SQLite/HTTP transport packages. Only pure Dart core libraries and `package:meta/meta.dart` (`@immutable`) are permitted.
- **Stateless Presentation (`app_flutter/lib/src/presentation/**`)**: Views are declarative projections of immutable ViewModel state (`StatelessWidget` by default; `StatefulWidget` is restricted to bounded local animation, focus, or scroll controllers). Views MUST NOT contain business rules, unit conversions, SQL queries, or FFI calls.
- **Dynamic Repository Injection at Bootstrap**: Views and ViewModels MUST NEVER instantiate concrete database handles, FFI libraries, file paths, or transport clients directly. All persistence, sensor-stream, and command operations interact exclusively with narrow abstract repository interfaces wired dynamically at the application composition root (`app_flutter/lib/main.dart`).

### 2.2 Mandatory 3-Layer Definition of Done (`N/A` Forbidden)

Every Epic, Feature, User Story, Use Case, and implementation micro-task MUST close all three architectural layers simultaneously. Marking any layer `N/A`, deferring a layer to a later task, or shipping disconnected domain types without presentation bindings is strictly forbidden:

| Layer | Architectural Scope | Mandatory Deliverables & Verification Evidence |
| :--- | :--- | :--- |
| **Layer 1: Domain State & Signal Model** | `lib/src/domain/**` and `lib/src/data/**` | `@immutable` entities and validated value objects, `sealed class` states and `DomainError` hierarchies, `Result<T>` signatures, narrow repository contracts, pure DTO mappers, and live persistence round-trip unit tests. |
| **Layer 2: Logic & Safety State Management** | `lib/src/application/**` or `lib/src/viewmodel/**` | Deterministic ViewModels and statecharts, input/command precondition guards, explicit stale/fault state transitions, `_disposed` lifecycle protection, and BDD state-transition tests covering nominal and fault paths. |
| **Layer 3: Display & Actuator Interface Binding** | `lib/src/presentation/**` and `test/**` | Responsive 1080p-6K widgets bound to live ViewModel streams, active gesture/actuator command bindings with disabled/fault affordances, pairwise non-overlapping HUD layouts, and BDD widget/integration tests. |

## 3. The 15 Mandatory Domain Engineering Standards

All code under `app_flutter/` MUST satisfy the following 15 normative domain engineering rules:

1. **`Result<T>` Over Exceptions**: All fallible domain operations, value-object factories, state transitions, and repository methods MUST return an explicit `Result<T>` sealed union (`Success<T>` or `Failure<T>`) carrying a typed `DomainError` rather than throwing runtime exceptions. `try`/`catch` blocks are confined strictly to external data/FFI adapter boundaries to translate third-party throws into `Failure<T>`.
2. **`sealed class` Hierarchies with Exhaustive Pattern Matching**: All algebraic domain states, ViewModel UI states, operator events, `Result<T>` types, and error families MUST be declared as `sealed class` hierarchies. Consumers MUST switch exhaustively without wildcard `default:` branches that silently swallow newly added variants.
3. **Named Constructors with Validation**: Domain entities and physical quantities MUST validate all structural and numerical invariants (non-empty trimmed identifiers, finite floating-point checks rejecting `NaN` and `infinity`, physical range bounds, monotonic timestamps) at construction time so invalid states cannot exist in memory.
4. **Typed `DomainError` Hierarchies**: Every domain module MUST declare closed, strongly-typed error classes extending a `sealed class DomainError` base with deterministic machine-readable error codes and structured context fields. Returning raw `String` error messages or untyped `Exception` objects is forbidden.
5. **`@immutable` Mandatory**: Every domain entity, value object, command envelope, event, `DomainError`, and ViewModel state snapshot MUST carry `@immutable`, declare all fields `final`, and wrap internal collections in unmodifiable views (`List.unmodifiable`, `Map.unmodifiable`, `Set.unmodifiable`).
6. **Narrow Interface Segregation**: Abstract repository and service interfaces MUST be narrow, role-focused, and cohesive (separating read/stream queries from actuator mutation contracts) so no consumer depends on unused operations.
7. **Zero `dynamic` and Zero Untyped `Object?`**: `dynamic` and untyped `Object?` types in domain/ViewModel signatures, properties, collections, or unchecked `as` casts are strictly prohibited. Untrusted external maps inside DTO decoders MUST be validated field-by-field via typed Dart 3 pattern matching (`case {'id': final String id, 'value': final num raw}`).
8. **BDD `given_when_then` Test Naming**: All unit, widget, and integration tests MUST use behavior-driven naming (`given_<context>_when_<trigger>_then_<outcome>` or `'should <expected behavior> when <condition>'`) asserting observable domain invariants and UI contracts rather than private implementation mechanics.
9. **`/// Realises: [Spec/Class]` Traceability Tags**: Every production library, public class, `sealed class`, interface, mixin, extension, `typedef`, and callable member MUST include a `/// Realises: [REQ-ID]` (or `/// Realises: [SpecName/ClassName]`) DartDoc tag pointing to a real SysML v2 AST node, requirement identifier, or approved blueprint classifier.
10. **100% Public DartDoc (`///`) Coverage**: Every public library, type, constructor, factory, method, getter, enum value, mixin, extension, and `typedef` under `app_flutter/lib/**` MUST carry non-placeholder `///` documentation satisfying Section 4. Block comments (`/* ... */`) and line comments (`//`) do not satisfy DartDoc enforcement.
11. **`const` Constructors**: Immutable value objects, `sealed class` state/error variants, and stateless widgets whose fields are compile-time constants or `final` references MUST provide `const` constructors to enable canonicalization and eliminate per-frame heap churn.
12. **Value Equality (`operator ==` & `hashCode`)**: All domain entities, value objects, ViewModel state snapshots, and `DomainError` subtypes MUST implement deterministic structural equality by overriding `operator ==` and `hashCode` (`Object.hash` / `Object.hashAll`), preventing spurious widget rebuilds on unchanged snapshots.
13. **Explicit `typedef` Callbacks**: Every callback parameter, predicate, listener signature, and actuator command handler MUST be declared via a named `typedef` alias (for example, `typedef ReadingSelectedCallback = void Function(EntityId id);`) rather than inline `void Function(...)` function types.
14. **Private Constructors `._()` with Validating Public Factories**: Domain types enforcing construction invariants MUST prevent unvalidated instantiation via a private constructor (`const SensorReading._({...})`) and expose public `static Result<SensorReading> create({...})` or named factory constructors.
15. **Strict DTO Serialization Separation**: Domain models MUST contain zero JSON, SQLite, binary, or FFI marshaling methods (`fromJson`, `toJson`, `fromMap`, `toMap`). All serialization and schema mapping resides strictly inside dedicated data-layer DTOs (`app_flutter/lib/src/data/dto/**`) that validate untrusted inputs before invoking domain factories.

## 4. Tiered DartDoc Documentation Model

Applying a 28-line 7-section docstring template to private `_` helpers, `_State` internals, or `@override` widget `build` / `operator ==` methods creates unreadable boilerplate noise and obscures safety-critical logic. All production Dart files under `app_flutter/lib/**` follow the Tiered DartDoc Model aligned with Check 36.

### 4.1 Tier A: Public API & Library Surfaces

Applies to:

- `library;` directives at the top of public files;
- public classes (`class`, `abstract interface class`, `sealed class` bases and public variants, `enum`, `mixin`, `extension`);
- public `typedef` declarations;
- top-level public functions and public constructors, factories, getters, and methods on public types (excluding `@override` members).

Every Tier A item MUST carry **either** **Option A1 (Full 7-Section Safety DartDoc)** **or** **Option A2 (Concise 4-Block Contract)**:

1. **Option A1 -- Full 7-Section Safety DartDoc** (recommended for safety-critical domain aggregates, statechart ViewModels, actuator command validators, FFI snapshot readers, and live persistence repositories):
   - `/// ## 1. Safety Intent & Regulatory Scope`
   - `/// ## 2. Invariants & Mathematical Properties`
   - `/// ## 3. Preconditions`
   - `/// ## 4. Postconditions`
   - `/// ## 5. Algorithmic Complexity Bounds`
   - `/// ## 6. Failure Modes & Fault Tolerance`
   - `/// ## 7. Realization & Traceability` (must contain `Realises: [REQ-ID]`)
2. **Option A2 -- Concise 4-Block Contract** (permitted on any Tier A library header, value object, state/error variant, stateless widget, `typedef`, factory, getter, or public method where 7-section prose would introduce mechanical filler):
   - 1-sentence non-placeholder summary of the item's responsibility;
   - `/// Safety & Traceability: Realises: [REQ-ID]`;
   - `/// Contract:` (preconditions, postconditions, and structural invariants);
   - `/// Bounds & Faults:` (complexity or frame-time bounds and deterministic `Result` / fault behavior).

Zero placeholder tokens (`TODO`, `TBD`, `FIXME`, `XXX`), zero fabricated requirement IDs, and zero copy-pasted generic complexity slogans are permitted in any DartDoc block.

### 4.2 Tier B/C: Private `_` Declarations, Internal `_State` Helpers & `@override` Members

Applies to:

- private (`_`-prefixed) classes, `_State` subclasses, private constructors, private methods, and top-level `_` helper functions;
- `@override` framework and `Object` trait implementations (`build`, `initState`, `didUpdateWidget`, `dispose`, `operator ==`, `hashCode`, `toString`).

Every Tier B/C item requires ONLY:

- a concise 1-3 line non-placeholder `///` summary stating purpose and any key lifecycle or safety invariant; and
- `/// Realises: [REQ-ID]`.

Test declarations under `app_flutter/test/**` and `app_flutter/integration_test/**` require clear BDD `given_when_then` identifiers or a 1-line `///` summary.

### 4.3 Illustrative Tier A & Tier B/C Pattern

```dart
/// Validated immutable physical sensor sample bounded to calibrated limits.
///
/// Safety & Traceability: Realises: [REQ-CORE-001]
/// Contract: `magnitude` is finite and lies within `[0.0, 1000.0]`; `id` is non-empty.
/// Bounds & Faults: `O(1)` time and allocation; returns `Failure(ReadingOutOfRangeError)` on invalid input.
@immutable
final class SensorReading {
  const SensorReading._({required this.id, required this.magnitude});

  /// Canonical sensor entity identifier.
  ///
  /// Safety & Traceability: Realises: [REQ-CORE-001]
  /// Contract: Trimmed non-empty identifier validated at construction.
  /// Bounds & Faults: `O(1)` immutable field read; total accessor.
  final EntityId id;

  /// Calibrated scalar magnitude in standard engineering units.
  ///
  /// Safety & Traceability: Realises: [REQ-CORE-001]
  /// Contract: Finite `double` within `[0.0, 1000.0]`.
  /// Bounds & Faults: `O(1)` immutable field read; total accessor.
  final double magnitude;

  /// Validates raw inputs and constructs a bounded [SensorReading].
  ///
  /// Safety & Traceability: Realises: [REQ-CORE-001]
  /// Contract: Rejects `NaN`, infinite values, and magnitudes outside `[0.0, 1000.0]`.
  /// Bounds & Faults: `O(1)` validation; returns `Failure` without throwing exceptions.
  static Result<SensorReading> create({
    required EntityId id,
    required double rawMagnitude,
  }) {
    if (!_isWithinCalibratedRange(rawMagnitude)) {
      return const Failure<SensorReading>(ReadingOutOfRangeError());
    }
    return Success<SensorReading>(
      SensorReading._(id: id, magnitude: rawMagnitude),
    );
  }

  /// Checks finite numerical bounds for calibrated sensor magnitudes.
  /// Realises: [REQ-CORE-001]
  static bool _isWithinCalibratedRange(double value) =>
      !value.isNaN && !value.isInfinite && value >= 0.0 && value <= 1000.0;

  /// Evaluates structural value equality across identifier and magnitude fields.
  /// Realises: [REQ-CORE-001]
  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      (other is SensorReading &&
          other.id == id &&
          other.magnitude == magnitude);

  /// Computes deterministic structural hash code for snapshot deduplication.
  /// Realises: [REQ-CORE-001]
  @override
  int get hashCode => Object.hash(id, magnitude);
}
```

## 5. Total Error Handling, Null Safety & Collection Determinism

- **Zero `!` Bang Null-Assertion Operator**: The postfix null-assertion operator (`value!`) is strictly forbidden anywhere in production `app_flutter/lib/**`. Unwrap nullable bindings exclusively via local type promotion (`if (value != null)`), Dart 3 null-check patterns (`if (value case final resolved?)`), null-coalescing defaults (`??`), or explicit `Failure` transitions.
- **Avoid `late` Runtime Traps**: `late` fields without inline initializers are forbidden in domain models and discouraged in ViewModels; only `State.initState` controller bindings whose lifecycle is strictly proven within `State` may use `late final`, and explicit nullable or constructor-injected fields are preferred.
- **Total Bounds-Safe Collection Access**: Direct bracket indexing (`items[index]`), `.first`, `.last`, `.single`, and `.firstWhere(...)` without fallback throw unchecked `RangeError` or `StateError` exceptions and are forbidden on untrusted or dynamic collections. Use `items.elementAtOrNull(index)`, `items.firstOrNull`, `items.lastOrNull`, `items.singleOrNull`, or list pattern matching (`switch (items) { case [final head, ...]: ... }`).
- **Finite Layout & Numeric Safety**: Passing `double.nan` or `double.infinity` into Flutter layout constraints (`SizedBox`, `Positioned`, `Transform`, `CustomPainter`) triggers unrecoverable rendering failures. All computed geometry ratios and external telemetry scalars MUST guard against division-by-zero, `isNaN`, and `isInfinite` prior to reaching presentation state.
- **Deterministic Collection Ordering**: Every collection driving persisted records, serialized exports, data tables, or HUD overlays MUST apply an explicit, total deterministic sort order (`ORDER BY` at the SQLite layer or explicit comparator sorting before wrapping in `List.unmodifiable`). Never depend on nondeterministic async completion order or hash-set iteration accidents.

## 6. Isolate Concurrency, Async Safety & 60 FPS (`16.67 ms`) Frame Determinism

### 6.1 60 FPS (`16.67 ms`) Frame Budget

- Operator console interfaces MUST sustain deterministic 60 FPS rendering (`<= 16.67 ms` total end-to-end frame time across build, layout, paint, and raster phases, with main UI isolate build/layout budget `<= 8.0 ms`) under peak operational update rates.
- High-frequency sensor or FFI snapshot streams MUST be coalesced or frame-paced (`SchedulerBinding` / bounded latest-snapshot sampling) rather than triggering unbounded per-packet full-screen widget rebuilds.
- Scope rebuild trees tightly using `ValueListenableBuilder`, `ListenableBuilder`, `RepaintBoundary`, and `const` leaf widgets.

### 6.2 Main UI Isolate Purity & Worker Isolate Offloading

- **Zero Synchronous I/O on Main Isolate**: Synchronous file and directory operations (`File.readAsBytesSync`, `readAsStringSync`, `writeAsBytesSync`, `existsSync`, `Directory.listSync`), blocking FFI calls, and large synchronous JSON/SQLite batch decodes on the root UI isolate are strictly forbidden.
- **Background Isolate Offloading**: Any parsing, batch projection, cryptographic verification, or spatial layout computation exceeding `2.0 ms` worst-case execution time MUST run on a background worker (`Isolate.run` or a dedicated supervised long-lived `Isolate`).
- **Pure Methodless DTOs Across `SendPort` Boundaries**: Messages crossing `SendPort` / `ReceivePort` isolate boundaries MUST consist solely of immutable primitives, `TransferableTypedData` byte buffers, or pure methodless DTO records. Passing closures, active `StreamController`s, open database handles, mutable repositories, or `BuildContext` / `ChangeNotifier` instances across isolates is forbidden.

### 6.3 Post-Disposal & Async Continuation Guards

- **ViewModel `_disposed` Guard**: Every `ChangeNotifier` or reactive ViewModel MUST track `bool _disposed = false;` and check `if (_disposed) return;` immediately after every `await` boundary and before every `notifyListeners()` invocation.
- **Widget `mounted` Guard**: Every `State<T>` async method MUST check `if (!mounted) return;` after every `await` before reading `context` or invoking `setState(...)`.
- **Zero Side Effects Inside `build()`**: `Widget.build(BuildContext context)` MUST be a pure, side-effect-free function of `BuildContext` and immutable state. Triggering repository reads, subscribing to streams, mutating ViewModel fields, instantiating controllers (`ScrollController`, `TextEditingController`, `FocusNode`), or scheduling unguarded async work inside `build()` is strictly forbidden.

## 7. Resource Lifecycle & Deterministic Disposal

Aligned with Check 39 lifecycle auditing, every reactive stream, timer, controller, native pointer, and database handle MUST have a single deterministic owner and follow strict reverse-acquisition teardown inside `dispose()`:

1. **Cancel Timers & Subscriptions First**: Cancel active `Timer` instances and `StreamSubscription` handles first so in-flight events cannot invoke callbacks during teardown.
2. **Detach Listeners & Stop Animations**: Remove external `Listenable` callbacks (`removeListener`) and dispose owned `AnimationController` instances before tearing down backing state.
3. **Close Owned Streams**: Close every owned `StreamController` (`if (!controller.isClosed) { unawaited(controller.close()); }`).
4. **Dispose UI Controllers**: Deterministically dispose owned `FocusNode`, `ScrollController`, `TextEditingController`, `TabController`, and `TransformationController` instances in the owning `State.dispose()`.
5. **Latch `_disposed` Before `super.dispose()`**: Set `_disposed = true;` prior to calling `super.dispose()` on `ChangeNotifier` subclasses so late microtasks fail closed without throwing `FlutterError`.
6. **Close Persistence & Native Handles**: Flush and close live SQLite connections, prepared statements, and FFI native buffers in repository `close()` / teardown hooks protected by `try`/`finally`.

Widget and ViewModel test suites MUST pump teardown cycles (`await tester.pumpWidget(const SizedBox.shrink())`) and verify zero leaked timers, zero post-disposal notifications, and clean SQLite file release.

## 8. Presentation Anti-Hardcoding, Zero-Mocking Live Persistence & Viewport Ergonomics

### 8.1 Section 1.9 Zero-Mocking Live Persistence Mandate

- All client-side application targets MUST execute runtime and persistence operations against a live persistent SQLite database (`sqflite` / `sqflite_common_ffi` backed by a real SQLite engine), protocol/hardware emulator, or memory-mapped local register file.
- In-memory fake lists (`FakeRepository`, `InMemoryRepository`, static sample arrays, `Future.delayed` simulated saves) are strictly prohibited in production code and persistence verification suites.

### 8.2 Check 38 -- Presentation Anti-Hardcoding AST Gate

- Widgets and ViewModels MUST render exclusively from injected live repository streams, explicit ViewModel state snapshots, and centralized theme/design tokens.
- Hardcoding synthetic operational readings, static status strings pretending to reflect live health, magic pixel coordinates, or fallback sample entities inside `build()` methods is a Check 38 defect.
- Loading, empty, stale-signal, degraded, and fault states MUST be represented as explicit `sealed class` state variants and rendered truthfully to the operator.

### 8.3 Check 39 -- Zero-Mocking Test AST Gate

- Test suites under `app_flutter/test/**` and `app_flutter/integration_test/**` MUST NOT import or invoke mocking frameworks (`mockito`, `mocktail`, `@GenerateMocks`, `Mock`, `Fake`, `when(...).thenReturn(...)`, `verify(...)`) for domain models, ViewModels, or persistence repositories.
- Persistence and vertical-slice tests MUST instantiate real SQLite schemas inside isolated auto-cleaned temporary directories (`Directory.systemTemp.createTempSync` cleaned in `addTearDown`) or deterministic loopback register files, proving real SQL constraints, transaction rollback on fault, and deterministic handle disposal.

### 8.4 Check 40 -- Interactive Gesture, 1080p-6K Viewport & Pairwise 2D HUD Non-Overlap Gate

- **Live Actuator & Gesture Binding**: Every interactive widget (`FilledButton`, `OutlinedButton`, `IconButton`, `Switch`, `Slider`, `InkWell`, `GestureDetector`, keyboard shortcut intent) MUST bind either to a live ViewModel command callback or an explicit disabled state (`onPressed: null` when precondition guards disallow actuation). Dead no-op handlers (`onPressed: () {}`) are strictly forbidden. Interactive pointer/touch targets MUST meet minimum `44.0 x 44.0` logical pixel ergonomics.
- **1080p-6K Multi-Resolution Scaling**: All screens and panels MUST render with zero `RenderFlex` overflow errors across standard operator console viewports from Full HD (`1920 x 1080`) and QHD (`2560 x 1440`) through 4K UHD (`3840 x 2160`) and 6K (`6016 x 3384`), using constraint-driven `LayoutBuilder`, `Expanded`/`Flexible`, and bounded scrollable regions.
- **Pairwise 2D HUD Non-Overlap**: Concurrent floating HUD overlays, status headers, alert banners, inspector drawers, and primary canvas controls MUST satisfy strict pairwise bounding-box non-overlap (`panelA.overlaps(panelB) == false`) across all supported viewport resolutions and text-scale settings, verified via automated `RenderBox` bounding-rectangle assertions in widget tests.

## 9. Structural Complexity Bounds & Known Anti-Patterns

### 9.1 Quantitative Structural & Complexity Thresholds

| Metric | Normative Bound | Enforcement & Decomposition Rule |
| :--- | :--- | :--- |
| Production file size | `< 500` executable SLOC | Excludes blank lines and `///` DartDoc comments. Split files by single domain, ViewModel, or sub-widget responsibility. |
| Function / method / `build()` length | `<= 60` executable SLOC | Extract sub-trees into dedicated `const` `StatelessWidget` classes rather than private `_buildSection()` helper methods. |
| Cyclomatic complexity | `v(G) <= 10` | Replace nested flag branches with exhaustive `switch` expressions on `sealed class` states and named predicate getters. |
| Control-flow / branch nesting depth | `<= 4` | Use early guard returns and flat `sealed class` state sub-widgets. |
| Behavioral parameter arity | `<= 5` parameters | Group cohesive domain arguments into `@immutable` command or query value objects. |
| Analyzer lint suppressions | `0` unapproved `// ignore:` | `// ignore:` and `// ignore_for_file:` are forbidden without an inline safety justification citing an approved Section 11 waiver ID. |

### 9.2 Known Anti-Patterns & Required Normative Patterns

| Anti-Pattern | Failure Mode / Audit Defect | Required Normative Pattern |
| :--- | :--- | :--- |
| 28-line 7-section DartDoc pasted onto `_State.build`, `operator ==`, `hashCode`, or private `_` helpers | Obscures widget/domain logic with mechanical filler (Check 36 noise floor). | Apply Tier A (7-section or 4-Block Contract) ONLY to public API surfaces; use 1-3 line `///` summary + `/// Realises: [REQ-ID]` on Tier B/C items (Section 4). |
| Postfix `!` bang null-assertions and raw `list[i]` / `.first` indexing in `lib/**` | Unhandled runtime `TypeError`, `RangeError`, or `StateError` grey-screen crash on malformed/empty telemetry. | Ban `!` in `lib/**`; use null-safe pattern matching, `.elementAtOrNull(i)`, `.firstOrNull`, and typed `Result<T>` returns (Section 5). |
| `dynamic` / `Map<String, dynamic>` in domain types or `fromJson`/`toJson` on domain entities | Couples pure domain invariants to wire schemas and permits silent runtime cast failures. | Enforce zero `dynamic`; isolate marshaling inside `lib/src/data/dto/**` using pattern-matched field validation before calling domain factories (Section 3 #7, #15). |
| `mockito` / `mocktail` mocks or `InMemoryRepository` list stubs | Masks SQLite schema mismatches, transaction bugs, and resource leaks (Section 1.9 & Check 39 violation). | Execute tests against live temp-directory SQLite instances or deterministic register files with real repositories and `addTearDown` cleanup (Section 8.1, 8.3). |
| Hardcoded telemetry literals in `build()`, dead `onPressed: () {}` buttons, or colliding `Positioned` HUD panels | Misleads operators with fake data and occludes safety controls across 1080p-6K monitors (Checks 38 & 40). | Bind exclusively to live ViewModel `sealed class` states, wire real command handlers, and assert pairwise `RenderBox` non-overlap across 1080p-6K viewports (Section 8.2, 8.4). |
| Unguarded `notifyListeners()` / `setState()` after `await` or uncancelled `StreamSubscription`s | Post-disposal `FlutterError` exceptions, memory leaks, and zombie callbacks mutating stale views (Check 39). | Enforce `_disposed` and `mounted` guards after every `await` and deterministic reverse-order teardown in `dispose()` (Sections 6.3, 7). |
| Synchronous `File.*Sync()` / heavy parsing on UI isolate or instantiating controllers inside `build()` | Drops frame rendering below 60 FPS (`> 16.67 ms` jank spikes) and leaks per-frame allocations. | Offload work `> 2.0 ms` via `Isolate.run` using pure DTOs; keep `build()` pure and allocate controllers exclusively in `initState` (Section 6.1, 6.2). |
| Monolithic `> 500` SLOC god widgets and silent `// ignore:` analyzer directives | Defeats static verification, inflates cyclomatic complexity (`v(G) > 10`), and blocks reviewability. | Decompose to `< 500` SLOC/file, `<= 60` SLOC/method, `v(G) <= 10`, nesting `<= 4`, with zero unwaived lint suppressions (Section 9.1). |
| Hand-rolled unverified `dart:ffi` pointer mutation or optimistic Ring-3 safety actuation without Ring-1 `SafetyMonitor` readback | Violates 3-ring DAL-D/E / QM containment and risks corrupting Ring-1 `safety_kernel` state (`blueprint-09`, `blueprint-11`). | Route exclusively through `crates/compile-sysml`-synthesized `flutter_rust_bridge` Ring-2 guards (`Miri` + `dart analyze --fatal-infos`) and enforce Ring-1 CRC/SHA-256 sequence-numbered `SafetyMonitor` readback/challenge-response interlocks (Section 1.3). |

## 10. Mandatory Verification Gates & Review Checklist

All downstream Dart and Flutter changes (including `app_flutter/lib/src/ffi/` Ring-2 consumer bindings) MUST pass the following verification gates with zero infos, warnings, or errors prior to commit:

```bash
cd app_flutter && dart analyze --fatal-infos
cd app_flutter && flutter analyze --fatal-infos --fatal-warnings
cd app_flutter && flutter test
./target/release/verify-baseline .
```

During downstream full-application verification gates, run the target platform release/debug build (`flutter build macos`, `flutter build linux`, or target profile equivalent) to verify native runner linkage and asset bundling.

### Reviewer Verification Checklist

- [ ] All Dart/Flutter files reside strictly inside `app_flutter/` with zero root source writes.
- [ ] Ring-3 (`app_flutter/`) preserves DAL-D/E / QM assurance containment, communicates with Ring-1 (`#![no_std]` Ferrocene `safety_kernel`) exclusively via `crates/compile-sysml`-synthesized `flutter_rust_bridge` Ring-2 FFI guards passing `dart analyze --fatal-infos`, and enforces Ring-1 CRC/SHA-256 sequence-numbered `SafetyMonitor` readback/challenge-response interlocks for safety-critical commands and annunciations.
- [ ] All 3 MVVM layers (Domain State, ViewModel Logic, Display/Actuator Binding) are closed with zero `N/A` placeholders.
- [ ] All 15 Mandatory Domain Engineering Standards (Section 3) are satisfied (`Result<T>`, `sealed class`, `@immutable`, zero `dynamic`, `const`, value equality, `typedef`, `._()` factories, pure DTO separation).
- [ ] DartDoc adheres to the Section 4 Tiered Model (Tier A 7-section or 4-Block Contract; Tier B/C 1-3 line summary + `Realises: [REQ-ID]`; zero placeholder tokens).
- [ ] Zero `!` null-assertion operators, zero unchecked collection indexing, and zero unguarded `NaN`/`infinity` layout inputs exist in `app_flutter/lib/**`.
- [ ] Main UI isolate executes zero synchronous I/O; `_disposed` and `mounted` guards protect every async boundary; `build()` is pure; `dispose()` follows reverse-acquisition order.
- [ ] Tests follow RED-GREEN-REFACTOR with `given_when_then` BDD naming against live SQLite/emulator persistence (zero `mockito`/`mocktail` mocks) and verify 1080p-6K pairwise HUD non-overlap.
- [ ] Every file is `< 500` executable SLOC, every method is `<= 60` SLOC, `v(G) <= 10`, max nesting `<= 4`, and zero unapproved `// ignore:` directives exist.

## 11. Waivers, Deviations & Change Control

A deviation from any rule in this standard is valid only when:

1. Documented in an explicitly approved remediation or implementation plan;
2. Scoped to the exact rule identifier, file path, and technical safety justification;
3. Linked to a tracked remediation issue or formal architectural boundary constraint;
4. Audited by a fresh context-isolated reviewer subagent; and
5. Preserving all safety invariants and `verify-baseline` gate checks without weakening verification thresholds.

Amendments to this standard require an approved plan, fresh reviewer verification, and explicit user authorization.
