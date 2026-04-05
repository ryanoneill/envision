# Parallelized Library Audit (Beta)

Run a systematic evaluation of the envision library using parallel subagents for speed.

## Execution Strategy

Execute ALL THREE steps simultaneously using parallel agents, then compile the report.

### Step 1: Launch all agents in parallel

Launch these 3 agents simultaneously in a single message:

**Agent 1: "Run audit tool and cargo checks"**
```
Build and run the envision-audit tool, then return the full output:
  cd tools/audit && cargo build --release 2>&1 && cd ../..
  ./tools/audit/target/release/envision-audit all 2>&1
Report the complete output verbatim.
```

**Agent 2: "Evaluate Groups 1-3" (subagent_type: Explore)**
```
You are a demanding senior engineer auditing the envision TUI framework at /home/ryano/workspace/ryanoneill/envision for production readiness.

Evaluate these 11 categories with specific evidence (file paths, line numbers). For each, assign a grade (A+ through F) and a one-line summary.

GROUP 1: FIRST IMPRESSIONS (15%)
1. Getting Started — README clarity, prelude completeness, MSRV, time to first program
2. Examples — Count, progressive complexity, component coverage, real-world relevance
3. Documentation — Module docs, type docs, method docs, doc test coverage, cross-references

GROUP 2: API DESIGN (25%)
4. Consistency & Symmetry — Naming patterns, builder methods, accessor symmetry, trait coverage
5. Modularity & Composability — Feature flags, dependency weight, component independence
6. Usability & Ergonomics — Import ergonomics, boilerplate ratio, edge case handling
7. Complexity Hiding — ratatui abstraction, runtime hiding, testing setup simplicity
8. Type Safety & Errors — Error types, panic freedom, boundary validation

GROUP 3: ENGINEERING QUALITY (20%)
9. Algorithms & Data Structures — Collection choices, allocation patterns, rendering efficiency
10. Smart Library Usage — ratatui/tokio integration, clippy discipline, dependency minimality
11. Performance & Benchmarking — Benchmark coverage, relevance, CI integration

Grading scale: A+ (4.3) best in class, A (4.0) excellent, A- (3.7) very good, B+ (3.3) good, B (3.0) solid, B- (2.7) acceptable, C+ (2.3) below expectations. "A" is hard to earn.

For each category provide 3-5 specific evidence items with file:line references, then grade and one-line summary.
```

**Agent 3: "Evaluate Groups 4-6" (subagent_type: Explore)**
```
You are a demanding senior engineer auditing the envision TUI framework at /home/ryano/workspace/ryanoneill/envision for production readiness.

Evaluate these 14 categories with specific evidence (file paths, line numbers). For each, assign a grade (A+ through F) and a one-line summary.

GROUP 4: TESTING (20%)
12. Unit Testing — Coverage breadth/depth, snapshot testing, test helpers, message coverage
13. Integration & E2E — Multi-component tests, property testing, stress tests, AppHarness
14. Doc Test Coverage — Count, quality, realistic usage demonstration

GROUP 5: ARCHITECTURE (10%)
15. Solving Pain Points — Headless testing, state management, component reuse, theming
16. Code Organization — File sizes (<1000), module hierarchy, CI pipeline, public API surface
17. Extensibility — Custom components/themes/subscriptions, semver discipline, CHANGELOG

GROUP 6: MISSING PIECES (10%)
18. Feature Flags — Granularity, opt-out capability
19. Error Infrastructure — Custom error hierarchy, distinguishable failures
20. Logging & Debugging — Tracing integration, debug representations
21. Guides & Migration — CHANGELOG, MIGRATION.md, CONTRIBUTING.md quality
22. Advanced Features — Clipboard, undo/redo, search, lifecycle hooks
23. Serialization — serde coverage, field skipping, persistence utility
24. Security — Terminal escape handling, input validation, zero unsafe
25. Ecosystem Integration — ratatui compatibility, backend flexibility, cross-platform

Grading scale: A+ (4.3) best in class, A (4.0) excellent, A- (3.7) very good, B+ (3.3) good, B (3.0) solid, B- (2.7) acceptable, C+ (2.3) below expectations. "A" is hard to earn.

For each category provide 3-5 specific evidence items with file:line references, then grade and one-line summary.
```

### Step 2: Compile the report

Once all 3 agents complete, compile the final report using their findings. Use Agent 1's data for test counts, version, commit hash, and file statistics. Use Agents 2 and 3 for per-category grades and evidence.

Produce the report in this format:

```
==========================================================================
ENVISION LIBRARY AUDIT REPORT
Date: {date}
Version: {version from Cargo.toml}
Commit: {git short hash}
Test Results: {unit} unit / {doc} doc / {integration} integration
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
...

TOP 5 IMPROVEMENTS (highest ROI):
1. {action} -- {category} from {grade} to {target}
...
```

### Grade Calculation

1. Convert each grade to points: A+=4.3, A=4.0, A-=3.7, B+=3.3, B=3.0, B-=2.7, C+=2.3, C=2.0, C-=1.7, D=1.0, F=0.0
2. Group GPA = average of category grades in group
3. Weights: Group 1 (15%), Group 2 (25%), Group 3 (20%), Group 4 (20%), Group 5 (10%), Group 6 (10%)
4. Overall = sum of (group GPA × weight)
5. Map back to nearest letter grade

### Important Notes
- Launch ALL 3 agents in a SINGLE message for true parallelism
- Do not inflate grades — "A" is hard to earn
- Every grade must have specific evidence
- Trust-eroding findings ranked by impact on prospective users
