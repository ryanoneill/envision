# Envision Library Roadmap

Based on the comprehensive library audit performed on 2026-03-01 against version 0.5.0 (commit 8b55b07).

**Overall Audit Grade: B (3.13 weighted GPA)**

## Iteration 1: API Quality (Target: B+ overall)

These are the top 5 highest-ROI improvements identified by the audit.

### 1. Standardize Selected Accessors

**Category**: Consistency & Symmetry (4) -- from B- toward B+

**Problem**: Five different naming patterns for the same concept across components:
- `selected()` returns item: RadioGroup, Tabs
- `selected_item()` returns item: SelectableList, Menu, LoadingList
- `selected_value()` returns `&str`: Dropdown, Select
- `selected_row()` returns item: Table
- `selected_node()` returns item: Tree

`selected_index()` is consistent across all components (returns `Option<usize>`).

**Fix**: Standardize on `selected_item()` as the universal item accessor. Keep component-specific variants as aliases for domain clarity (e.g., `selected_row()` on Table, `selected_node()` on Tree) but ensure `selected_item()` exists on every component with a selection concept.

### 2. Add Disabled State to All Focusable Components

**Category**: Consistency & Symmetry (4) -- from B- toward A-

**Problem**: 8 of 17 Focusable components lack `is_disabled()`/`set_disabled()`:
- Dialog
- InputField
- Menu
- MultiProgress
- SelectableList
- StatusLog
- TextArea
- Tree

Meanwhile 9 components have it: Accordion, Breadcrumb, Button, Checkbox, Dropdown, RadioGroup, Select, Table, Tabs.

**Fix**: Add `is_disabled()`, `set_disabled(bool)`, and `with_disabled(bool)` to all 8 missing components. Add disabled state guards to `handle_event()` and disabled rendering to `view()`. Add tests for disabled behavior.

### 3. Add Instance Methods to Remaining Focusable Components

**Category**: Consistency & Symmetry (4) -- from B- toward A-

**Problem**: 5 Focusable components lack `handle_event()`/`dispatch_event()`/`update()` instance methods on their State types:
- LoadingList
- Menu
- MultiProgress
- RadioGroup
- StatusLog

These force users to use the `Component::handle_event()` turbofish pattern instead of the ergonomic `state.component.dispatch_event(&event)` pattern available on the other 12 components.

**Fix**: Add instance methods to all 5 State types, following the existing pattern from components like SelectableList and Tabs.

### 4. Add Feature Flags

**Category**: Feature Flags (18) -- from D toward B

**Problem**: Cargo.toml has no `[features]` section. Users compile all 26 components, tokio, serde, and 217 transitive dependencies with no way to opt out.

**Fix**: Add feature flags:
- `default = ["full"]`
- `full` -- all components + all optional features
- Individual component features or component group features
- `serde` -- serialization support (currently hard dependency)
- Consider tokio feature gating if feasible

### 5. Update Changelog for 0.4.0 and 0.5.0

**Category**: Guides & Migration (21) -- from C- toward B

**Problem**: CHANGELOG.md only covers 0.1.0 through 0.3.0. Current version is 0.5.0, meaning two releases are undocumented.

**Fix**: Reconstruct changelog entries from git history for 0.4.0 and 0.5.0. Add CONTRIBUTING.md.

---

## Iteration 2: Testing Depth

### 6. Expand Integration Tests
- Category: Integration & E2E Testing (13) -- from C+ toward B+
- Add multi-component workflow tests
- Add AppHarness-based integration tests
- Target: 20+ integration tests covering realistic user scenarios

### 7. Add Property-Based Testing
- Category: Integration & E2E Testing (13)
- Add proptest for event handler invariants
- Focus on: SelectableList, Table, Tree, InputField, TextArea
- Verify: navigation never panics, indices stay in bounds, state invariants hold

### 8. Add Stress Tests
- Category: Integration & E2E Testing (13)
- Table with 10,000+ rows
- Tree with deeply nested hierarchies (100+ depth)
- SelectableList with 100,000 items

### 9. Convert Ignored Doc Tests to Runnable
- Category: Doc Test Coverage (14) -- from B toward A-
- Reduce 40 ignored doc tests, especially in runtime and command modules
- Add explanatory comments where ignore is truly necessary

---

## Iteration 3: Performance

### 10. Add Component View Benchmarks
- Category: Performance & Benchmarking (11) -- from B toward A-
- Benchmark view() for all components at multiple sizes
- Benchmark with 1000+ item lists/tables/trees
- Add CI benchmark regression detection

---

## Iteration 4: Missing Infrastructure

### 11. Add Custom Error Types
- Category: Error Infrastructure (19) -- from C- toward B+
- Create EnvisionError enum with meaningful variants
- Replace BoxedError where possible
- Add error handling documentation

### 12. Add Tracing Integration
- Category: Logging & Debugging (20) -- from C toward B+
- Optional tracing behind a feature flag
- Instrument event->message->update flow
- Add debug mode for runtime loop visibility

### 13. Add CONTRIBUTING.md and Security Documentation
- Categories: Guides (21), Security (24)
- CONTRIBUTING.md with development setup, testing, PR process
- SECURITY.md with security model documentation

---

## Iteration 5: Advanced Features

### 14. Search/Filter for List Components
- Category: Advanced Features (22)
- Add search/filter to SelectableList, Table, Tree (Dropdown already has it)

### 15. Clipboard Integration
- Category: Advanced Features (22)
- Copy/paste for InputField and TextArea

### 16. Undo/Redo for Text Components
- Category: Advanced Features (22)
- Undo/redo stack for InputField and TextArea

### 17. Component State Serialization
- Category: Serialization (23) -- from C- toward B
- Add Serialize/Deserialize derives to component State types
- Enable session persistence

---

## Iteration 6: Ecosystem

### 18. Backend Abstraction
- Category: Ecosystem Integration (25)
- Abstract backend to support termion, termwiz alternatives
- Consider async runtime abstraction (tokio vs async-std)

---

## Audit Category Reference

| # | Category | Current Grade | Target After Iter 1 |
|---|----------|--------------|---------------------|
| 1 | Getting Started | A- | A- |
| 2 | Examples | B | B |
| 3 | Documentation | A- | A- |
| 4 | Consistency & Symmetry | B- | A- |
| 5 | Modularity & Composability | C+ | B- |
| 6 | Usability & Ergonomics | A- | A |
| 7 | Complexity Hiding | B+ | B+ |
| 8 | Type Safety & Errors | B | B |
| 9 | Algorithms & Data Structures | A- | A- |
| 10 | Smart Library Usage | A- | A- |
| 11 | Performance & Benchmarking | B | B |
| 12 | Unit Testing | A | A |
| 13 | Integration & E2E Testing | C+ | C+ |
| 14 | Doc Test Coverage | B | B |
| 15 | Solving Pain Points | A- | A- |
| 16 | Code Organization | A- | A- |
| 17 | Extensibility | B+ | A- |
| 18 | Feature Flags | D | B |
| 19 | Error Infrastructure | C- | C- |
| 20 | Logging & Debugging | C | C |
| 21 | Guides & Migration | C- | B |
| 22 | Advanced Features | C | C |
| 23 | Serialization | C- | C- |
| 24 | Security | C | C |
| 25 | Ecosystem Integration | B- | B- |
