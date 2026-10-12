# PR 4 — `compact_str` 0.8 → 0.9 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move envision to `compact_str` 0.9 (ratatui-core's major), removing the 0.8/0.9 duplicate, and re-export the crate.

**Architecture:** `compact_str` stays in the public API (`SortKey::String`, `From<CompactString> for Cell`, `impl Into<CompactString>` parameters). A crate-root `pub use compact_str;` lets consumers name envision's exact version. A golden JSON test pins the wire format before the bump.

**Tech Stack:** Rust 1.88, compact_str 0.9 (with `serde` under the `serialization` feature).

**Spec:** `docs/superpowers/specs/2026-10-12-v018-programme-completion-design.md` § PR 4.

## Global Constraints

- `compact_str = "0.9"`. The `serialization` feature keeps `"compact_str/serde"`.
- `pub use compact_str;` at the crate root. Its rustdoc states the policy: *envision tracks the `compact_str` major used by ratatui-core; a change of that major is a breaking envision release.*
- No warnings. Commits are signed with the `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` trailer, and `cargo fmt --all` runs first.
- Branch `v018-compact-str` from `origin/main`.

## Review Focus

1. **A consumer passing `&String` or `String` still compiles.** This is leadline's usage. Task 2 adds a test calling `Cell::new(&my_string)` and `Cell::new(my_string)`.
2. **The JSON wire format of `EnhancedCell` is byte-identical before and after.** The Task 1 golden test covers it.
3. **A `match` on `SortKey::String(s)` using `s.as_str()` still compiles.** Task 2 test.

---

### Task 1: Golden JSON test (before the bump)

**Files:** Modify `src/backend/cell/tests.rs` (serialization-gated section).

- [ ] **Step 1: Write the characterization test.** Build a cell with a non-trivial symbol and style, serialize it, and pin the exact string:

```rust
#[cfg(feature = "serialization")]
#[test]
fn enhanced_cell_json_is_byte_stable() {
    use ratatui::style::{Color, Modifier, Style};
    let mut cell = EnhancedCell::with_symbol("中");
    cell.set_style(
        Style::new()
            .fg(Color::Red)
            .bg(Color::Rgb(1, 2, 3))
            .add_modifier(Modifier::BOLD)
            .underline_color(Color::Indexed(42)),
    );
    let json = serde_json::to_string(&cell).unwrap();
    assert_eq!(json, GOLDEN);
}
```

`GOLDEN` is a `const &str` holding the exact output of the current code (compact_str 0.8). Get it by running the test once with `GOLDEN = ""`, copying the `left` value from the failure, then hard-coding it. That's the characterization step; say so in a comment above the const.

- [ ] **Step 2: Run** `cargo nextest run --all-features -E 'test(enhanced_cell_json_is_byte_stable)'`. Expected: PASS on 0.8.
- [ ] **Step 3: Commit** with signature: "Pin EnhancedCell JSON wire format with a golden test".

---

### Task 2: Bump to 0.9 and re-export

**Files:**
- Modify: `Cargo.toml:59` (`compact_str = "0.9"`), `Cargo.lock`
- Modify: `src/lib.rs` (crate-root re-export, next to `pub use ratatui;`)
- Test: `tests/reexports.rs`

- [ ] **Step 1: Write the failing tests**, appended to `tests/reexports.rs`:

```rust
#[test]
fn compact_str_is_reexported_and_matches_cell_api() {
    let s: envision::compact_str::CompactString = "abc".into();
    let cell = envision::component::cell::Cell::from(s.clone());
    match cell.sort_key() {
        envision::component::cell::SortKey::String(v) => assert_eq!(v.as_str(), "abc"),
        other => panic!("unexpected sort key {other:?}"),
    }
}

#[test]
fn string_and_str_still_construct_cells() {
    let owned = String::from("x");
    let _a = envision::component::cell::Cell::new(&owned);
    let _b = envision::component::cell::Cell::new(owned);
    let _c = envision::component::cell::Cell::new("lit");
}
```

Adjust `cell.sort_key()` to the real accessor name: check `grep -n "pub fn sort_key\|fn sort_key" src/component/cell.rs`. If `Cell` exposes the key differently, use that accessor. Do not add new API.

- [ ] **Step 2: Run, expect RED.** `cargo nextest run --all-features --test reexports` fails with E0433, since `compact_str` is not found in `envision`.
- [ ] **Step 3: Implement.** Set `compact_str = "0.9"` in `Cargo.toml` and run `cargo update -p compact_str`. Then in `src/lib.rs`, next to `pub use ratatui;`:

```rust
/// The exact `compact_str` envision is built with.
///
/// `CompactString` appears in envision's public API (`SortKey::String`,
/// `From<CompactString> for Cell`, and `impl Into<CompactString>` parameters).
/// Use `envision::compact_str::CompactString` to name it. envision tracks the
/// `compact_str` major used by ratatui-core; a change of that major is a
/// breaking envision release.
pub use compact_str;
```

Fix any 0.9 API fallout at the source. The adversarial review expects none.

- [ ] **Step 4: Verify GREEN.**

```bash
cargo nextest run --all-features --profile ci        # incl. the golden test, unchanged
cargo nextest run --no-default-features --profile ci
cargo tree -d -e normal --all-features | grep compact_str   # Expected: no output
cargo clippy --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo audit --deny warnings
```

- [ ] **Step 5: Commit** with signature: "compact_str 0.9 (ratatui-core's major) + pub use compact_str".

---

### Task 3: Docs, PR, merge

**Files:** `CHANGELOG.md` (`[Unreleased]`), `MIGRATION.md` (§ v0.17.x → v0.18.0).

- [ ] **Step 1: CHANGELOG.**
  - Under Breaking Changes, add a `compact_str` 0.8 → 0.9 entry. Code that names `CompactString` must use 0.9 or `envision::compact_str`. Passing `&str`/`String` is unaffected.
  - Under Added, add `envision::compact_str`.
  - In Known Deferred Findings, close the "compact_str adoption is sporadic … commit-or-drop" entry by removing it. Note "decision: commit (ratatui-core uses 0.9; no-allocation cell symbols)".
- [ ] **Step 2: MIGRATION.**
  - § 1 table: add a row `compact_str 0.8 → 0.9`.
  - New subsection "`CompactString` type identity", with a before/after: a consumer's own `compact_str = "0.8"` `CompactString` passed to `Cell::from` or `SortKey::String` → use `envision::compact_str::CompactString`. Note that a plain `match` still compiles.
  - § 7 re-export list: add `envision::compact_str`.
- [ ] **Step 3:** `cargo test --doc --all-features`, then make a signed commit.
- [ ] **Step 4:** Merge `origin/main`, push, and open a PR (summary, verification, `🤖 Generated with [Claude Code](https://claude.com/claude-code)`). After an independent review and green CI, squash-merge.
