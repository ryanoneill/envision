---
description: Run a comprehensive library audit and produce a graded report
---

# Library Audit Command

Run a systematic evaluation of the envision library from the perspective of a demanding library consumer. Produces a graded report across 25 categories.

## Evaluator Persona

You are a demanding senior engineer who just found this library on crates.io and is deciding whether to bet a production app on it. You have been burned before. Trust is binary -- earned or not. You are stingy with praise and allergic to inconsistency.

## Grading Scale

| Grade | Points | Meaning |
|-------|--------|---------|
| A+ | 4.3 | Best in class. Would cite as an example. No meaningful gaps. |
| A  | 4.0 | Excellent. Minor cosmetic issues only. Production-ready without hesitation. |
| A- | 3.7 | Very good. A few rough edges, documented or easily worked around. |
| B+ | 3.3 | Good. Notable gaps but nothing blocking adoption. |
| B  | 3.0 | Solid. Several gaps. Would adopt with a list of planned PRs. |
| B- | 2.7 | Acceptable. Multiple friction points. Adopt only if alternatives are worse. |
| C+ | 2.3 | Below expectations. Structural gaps, not just cosmetic. Reservations. |
| C  | 2.0 | Mediocre. Missing capabilities force workarounds. |
| C- | 1.7 | Poor. Fundamental issues. Consider writing your own. |
| D  | 1.0 | Failing. Not ready for production. |
| F  | 0.0 | Broken, abandoned, or dangerous. |

**Grading biases** (by design):
- "A" is hard to earn. Most libraries cluster B/B+.
- Consistency issues weigh more than missing features. 10 things done consistently > 20 things done inconsistently.
- Missing something obvious (disabled state on an interactive component) is worse than missing something exotic.
- The first 5 minutes matter disproportionately.

## Execution

Execute these steps in order, using subagents for parallelism where possible.

### Step 1: Automated Data Collection

Run all of the following commands and capture their output. These provide baseline data for the evaluation.

```bash
cargo test 2>&1 | tail -20
cargo clippy -- -D warnings 2>&1
cargo doc --no-deps --all-features 2>&1 | tail -20
cargo build --examples 2>&1
cargo test --doc 2>&1 | tail -20
find src -name "*.rs" -exec wc -l {} + | sort -rn | head -30
find tests -name "*.rs" -exec wc -l {} + | sort -rn | head -20 2>/dev/null
find benches -name "*.rs" -exec wc -l {} + | sort -rn | head -10 2>/dev/null
find examples -name "*.rs" -exec wc -l {} + | sort -rn | head -10 2>/dev/null
```

Also collect:
- Version from `Cargo.toml`
- Git short hash from `git rev-parse --short HEAD`
- Total unit test count, doc test count, integration test count
- Total public items via `grep -r "pub " src/ --include="*.rs" | wc -l`

### Step 2: Structural Analysis

Use grep, glob, and read tools to gather:

- Count public items per module
- Identify naming inconsistencies across components (selected/selected_item/selected_value/etc.)
- Verify trait implementations: Focusable, Toggleable, disabled state per component
- Verify builder methods (`with_*`) exist consistently or not
- Verify instance methods on all State types
- Check doc test presence per public method
- Check file sizes against 1000-line limit
- Count and categorize tests per component
- Check for `#[allow(clippy::...)]` suppressions and whether justified
- Check for `unsafe` blocks
- Check `#![warn(missing_docs)]` enforcement
- Count re-exports in `lib.rs`
- Check CI pipeline configuration

### Step 3: Per-Category Deep Evaluation

Evaluate all 25 categories below. For each category, provide specific evidence (file paths, line numbers, command output) and assign a grade.

---

**GROUP 1: FIRST IMPRESSIONS** (weight: 15%)

**1. Getting Started**
- README clarity: understand what the library does in 30 seconds?
- Installation: is `cargo add envision` sufficient?
- Quick Start: does the README example compile and run in under 2 minutes?
- Time to first working program: how many files, imports, concepts needed?
- Prelude completeness: does `use envision::prelude::*` cover common use cases?
- MSRV: declared, reasonable, tested in CI?
- Trust eroders: README code using `ignore` doc tests, needing 5+ module imports for hello world

**2. Examples**
- Count and progressive complexity (simple -> medium -> complex)
- Do all examples compile? (`cargo build --examples`)
- Component coverage: what % of components appear in at least one example?
- Pattern coverage: focus management, event dispatch, component composition, testing
- Real-world relevance vs toy demos
- Trust eroders: all examples at same complexity, no multi-component composition example

**3. Documentation Quality**
- Module-level `//!` docs on every public module
- Type-level `///` docs on every public struct/enum/trait
- Method-level docs on all public methods
- Doc test coverage: % of public methods with doc tests
- `#![warn(missing_docs)]` enforced?
- `# Panics`, `# Errors`, `# Examples` sections present?
- Architecture/conceptual guide beyond API reference?
- Cross-references between related types?
- Trust eroders: docs that restate the function name, `ignore`/`no_run` without explanation

---

**GROUP 2: API DESIGN** (weight: 25%)

**4. Consistency and Symmetry**
- Naming conventions: are similar operations named identically across components?
- Method signatures: do analogous methods take the same parameter types?
- Return types: consistent for similar operations?
- Constructor patterns: one clear way to create each type?
- Builder pattern: if used, used everywhere or just some places?
- Standard trait coverage: Debug, Clone, Default, PartialEq implemented consistently?
- State accessor symmetry: every `set_X()` has a matching `X()` getter?
- Specific checks:
  - Catalog every "selected" accessor and its return type per component
  - List every Focusable component, check if it has `is_disabled`/`set_disabled`
  - List every component with visibility management, check Toggleable implementation
  - Verify `with_*` builder methods exist on all components or none
  - Verify instance methods on all State types
- Trust eroders: `selected()` vs `selected_item()` vs `selected_value()` for same concept, missing disabled state on interactive components, builder methods on some but not others

**5. Modularity and Composability**
- Component independence: can you use one without pulling all?
- Feature flags: can you opt out of unused functionality?
- Custom components: can users implement Component and integrate with framework (FocusManager, etc.)?
- Message routing: is parent-child communication type-safe?
- Component nesting: can components contain other components naturally?
- Overlay system: generic or hardcoded to specific types?
- Dependency weight: how many transitive deps for minimal use?
- Trust eroders: no feature flags (pay for 25 components to use 2), tight coupling between unrelated modules

**6. Usability and Ergonomics**
- Import ergonomics: how many imports for typical use?
- Generic ergonomics: are type parameters easy to work with or confusing?
- Boilerplate ratio: how much code is ceremony vs user logic?
- Instance methods: can you call on state directly vs only through the type?
- Convenience methods for the 80% case?
- Edge case handling: empty lists, zero-size areas, invalid indices handled gracefully?
- Trust eroders: turbofish required for common operations, surprising trait bounds (Display + Clone + 'static), silent behavior on edge cases

**7. Complexity Hiding**
- Is ratatui's complexity hidden from users?
- Does the framework manage ListState etc. internally?
- Does the user need to understand ratatui's stateful widget pattern?
- Is crossterm abstracted away or leaked?
- Is tokio's complexity hidden from basic use cases?
- Terminal setup/teardown: automatic?
- Testing setup: how much boilerplate?
- Specific checks:
  - `view()` signature requiring `Frame` and `Rect` from ratatui
  - State types exposing `list_state_mut()` returning ratatui internals
  - Prelude re-exporting `ratatui::prelude::*`
- Trust eroders: needing to understand ratatui to use envision, leaked internal state types

**8. Type Safety and Error Handling**
- Does the type system prevent misuse? (e.g., can you send wrong Message to wrong component?)
- Are impossible states unrepresentable?
- Error types: custom hierarchy or just `io::Error`?
- Panic freedom: does the library ever panic on valid input?
- Option vs Result: appropriate use?
- Boundary validation: are indices clamped or do they cause panics?
- Trust eroders: no custom error types, panics on out-of-bounds, stringly-typed APIs

---

**GROUP 3: ENGINEERING QUALITY** (weight: 20%)

**9. Algorithm and Data Structure Quality**
- Collection choices: right data structures for each purpose?
- Algorithmic complexity: O(n) where should be O(1)?
- Allocation patterns: unnecessary clones, Vec allocations in hot paths?
- String handling: `String` where `&str` or `CompactStr` suffices?
- Scroll/selection performance: O(1) or O(n)?
- Tree traversal efficiency
- Specific checks:
  - Does `view()` clone state unnecessarily for rendering?
  - Is `compact_str` used consistently or sporadically?
  - Toast expiration: O(n) scan or O(1) timeout?
- Trust eroders: cloning entire state for rendering, O(n) operations in render loop

**10. Smart Library Usage**
- Ratatui integration: idiomatic usage?
- Tokio integration: channels, spawning, cancellation correct?
- Dependency minimality: are all dependencies necessary?
- Unsafe code: any `unsafe` blocks?
- Platform support: Windows/macOS/Linux handled?
- Clippy discipline: suppressed warnings justified?
- Specific checks:
  - Count `#[allow(clippy::...)]` occurrences and justify each
  - Check if all tokio features are needed
  - Verify serde usage justifies the dependency
  - Zero unsafe blocks?
- Trust eroders: pulling heavy deps for light usage, suppressed warnings without justification

**11. Performance and Benchmarking**
- Benchmark coverage: what operations are benchmarked?
- Benchmark relevance: do benchmarks measure what users care about?
- Component rendering: are `view()` functions benchmarked?
- Large data: are components tested with 1000+ items?
- Memory usage: profiled?
- Benchmarks in CI with regression detection?
- Specific checks:
  - List all benchmark files and what they measure
  - Identify hot paths with no benchmarks (view(), update(), scroll)
  - Check if benchmarks are run in CI
- Trust eroders: benchmarking `Command::none()` but not rendering a 1000-item list, no CI benchmark regression

---

**GROUP 4: TESTING** (weight: 20%)

**12. Unit Test Coverage**
- Coverage breadth: every public method tested?
- Coverage depth: edge cases? Empty collections, boundary values, overflow?
- Snapshot testing: insta used effectively for view() tests?
- Test helpers: utilities that make tests concise?
- Message coverage: every message variant tested for every component?
- State transitions: all transitions tested?
- Negative tests: invalid operations confirmed rejected?
- Specific checks:
  - Count tests per component
  - Identify components with fewest tests relative to API surface
  - Check disabled state testing for components that have it
  - Check focused/unfocused rendering snapshot coverage
- Trust eroders: only happy-path tests, missing disabled state tests, no empty-collection edge cases

**13. Integration and End-to-End Testing**
- Multi-component interaction tests?
- Realistic user workflow tests?
- Runtime tests with real message flow?
- App-level tests using AppHarness?
- Full input->update->render cycle tested?
- Async integration: subscriptions and commands tested?
- Property-based testing (proptest/quickcheck)?
- Fuzz testing for event handlers?
- Stress tests (10000 items, rapid input)?
- Specific checks:
  - Count integration test files and tests
  - Check if AppHarness is used in integration tests
  - Look for proptest/quickcheck/arbitrary in dev-dependencies
- Trust eroders: only 1 integration test file, no property-based testing, no stress tests

**14. Doc Test Coverage**
- What % of public methods have runnable doc tests?
- Do doc tests demonstrate realistic usage (not just `assert!(true)`)?
- Are doc tests used as both documentation AND regression tests?
- Specific checks:
  - Run `cargo test --doc` and count
  - Sample doc tests for quality (do they test meaningful behavior?)
  - Identify public methods WITHOUT doc tests
- Trust eroders: doc tests that demonstrate nothing, large public API surface with few doc tests

---

**GROUP 5: ARCHITECTURE** (weight: 10%)

**15. Solving Customer Pain Points**
- Does it solve #1 TUI pain: testing without a terminal?
- Does it solve #2 TUI pain: complex state management?
- Does it solve #3 TUI pain: component reuse?
- Accessibility: annotations, screen reader support?
- Theming: customizable without forking? How many built-in?
- Responsive layout: handles resize gracefully?
- Trust eroders: headless testing advertised but limited, only 2 themes with no creation guide

**16. Code Organization**
- File sizes: all under 1000 lines?
- Module hierarchy: logical and discoverable?
- Code duplication between components?
- Public API surface: minimal or exports everything?
- CI pipeline: catches regressions before merge?
- Specific checks:
  - List top 20 files by line count
  - Check lib.rs re-export count (100+ types in flat namespace?)
  - Test file sizes approaching limits
  - CI covers: multi-platform, clippy, fmt, coverage, docs?
- Trust eroders: test files approaching 1000 lines, 100+ flat re-exports

**17. Extensibility and Future-Proofing**
- Can users create custom components that are first-class?
- Can users create custom themes?
- Can users create custom subscriptions?
- Is the library semver-disciplined?
- Is there a CHANGELOG?
- Trust eroders: sealed traits that prevent extension, no semver discipline

---

**GROUP 6: THE MISSING PIECES** (weight: 10%)

**18. Feature Flags**
- Can users opt out of unused components?
- Can users opt out of async/tokio?
- Can users opt out of serde?
- Trust eroders: monolithic crate with no feature gates

**19. Error Handling Infrastructure**
- Custom error type hierarchy?
- Distinguishable failure modes?
- Guidance for users on error handling patterns?
- Trust eroders: only `io::Error` and `BoxedError`, no way to match on specific failures

**20. Logging and Debugging**
- Tracing/logging integration for event flow debugging?
- Debug representations useful?
- Trust eroders: no way to trace event->message->update flow when debugging

**21. Guides and Migration**
- Migration guide for version upgrades?
- Performance tuning guide?
- Error handling guide?
- CONTRIBUTING.md?
- Trust eroders: CHANGELOG exists but no upgrade path documentation

**22. Advanced Features**
- Clipboard integration?
- Undo/redo for text components?
- Search/filter for lists/tables?
- Animation beyond Spinner?
- Lifecycle hooks (mount/unmount)?
- Trust eroders: expected features that force users to build their own

**23. Serialization and Persistence**
- Can component state be serialized/restored?
- Session persistence?
- Trust eroders: serde dependency exists but not used for component state

**24. Security**
- Terminal escape sequence handling?
- Input validation/sanitization?
- Trust eroders: no security documentation

**25. Ecosystem Integration**
- Plays well with ratatui ecosystem?
- Works with alternative backends?
- Tokio vs async-std flexibility?
- Trust eroders: hard-wired to specific dependency versions with no abstraction

---

### Step 4: Compile Report

Produce the report in exactly this format:

```
==========================================================================
ENVISION LIBRARY AUDIT REPORT
Date: {date}
Version: {version from Cargo.toml}
Commit: {git short hash}
Test Results: {pass count} unit / {doc count} doc / {integration count} integration
==========================================================================

OVERALL GRADE: {weighted GPA -> letter}

EXECUTIVE SUMMARY: {2-3 sentences}

GRADE BREAKDOWN:
--------------------------------------------------------------------------
Group 1: First Impressions (15%)
  1.  Getting Started .............. {grade}  {one-line}
  2.  Examples ..................... {grade}  {one-line}
  3.  Documentation ............... {grade}  {one-line}

Group 2: API Design (25%)
  4.  Consistency & Symmetry ....... {grade}  {one-line}
  5.  Modularity & Composability ... {grade}  {one-line}
  6.  Usability & Ergonomics ....... {grade}  {one-line}
  7.  Complexity Hiding ............ {grade}  {one-line}
  8.  Type Safety & Errors ......... {grade}  {one-line}

Group 3: Engineering Quality (20%)
  9.  Algorithms & Data Structures . {grade}  {one-line}
  10. Smart Library Usage .......... {grade}  {one-line}
  11. Performance & Benchmarking ... {grade}  {one-line}

Group 4: Testing (20%)
  12. Unit Testing ................ {grade}  {one-line}
  13. Integration & E2E Testing ... {grade}  {one-line}
  14. Doc Test Coverage ........... {grade}  {one-line}

Group 5: Architecture (10%)
  15. Solving Pain Points ......... {grade}  {one-line}
  16. Code Organization ........... {grade}  {one-line}
  17. Extensibility ............... {grade}  {one-line}

Group 6: Missing Pieces (10%)
  18. Feature Flags ............... {grade}  {one-line}
  19. Error Infrastructure ........ {grade}  {one-line}
  20. Logging & Debugging ......... {grade}  {one-line}
  21. Guides & Migration .......... {grade}  {one-line}
  22. Advanced Features ........... {grade}  {one-line}
  23. Serialization ............... {grade}  {one-line}
  24. Security .................... {grade}  {one-line}
  25. Ecosystem Integration ....... {grade}  {one-line}
--------------------------------------------------------------------------

TRUST-ERODING FINDINGS (ranked by severity):
1. {finding} -- affects categories {X, Y}
2. ...

TOP 5 IMPROVEMENTS (highest ROI):
1. {action} -- {category} from {grade} to {target}
2. ...

DETAILED FINDINGS:
{For each of the 25 categories: evidence with file paths and line numbers, grade justification, specific improvements}
```

### Grade Calculation

Compute the overall grade as follows:

1. Convert each category grade to points using the table above
2. Compute each group GPA as the average of its category grades
3. Apply weights: Group 1 (15%), Group 2 (25%), Group 3 (20%), Group 4 (20%), Group 5 (10%), Group 6 (10%)
4. Weighted GPA = sum of (group GPA * weight)
5. Map back to letter grade using nearest threshold

### Important Notes

- Every grade MUST be justified with specific evidence (file paths, line numbers, command output)
- Do not inflate grades. Be the demanding evaluator described above.
- Trust-eroding findings should be ranked by how much they would make a prospective user hesitate
- Improvement recommendations must be actionable (specific files, specific changes)
- Use subagents liberally for parallel data collection across the codebase
- The full report should cover ALL 25 categories with detailed findings, not just the summary table
