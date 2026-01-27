---
description: Bump version, publish to crates.io, and create GitHub release
---

# Release Command

Release a new version of envision. Takes an optional version argument (major, minor, patch, or explicit version like "0.4.0").

**Usage:** `/release [version]`

**Examples:**
- `/release patch` - Bump 0.3.0 → 0.3.1
- `/release minor` - Bump 0.3.0 → 0.4.0
- `/release major` - Bump 0.3.0 → 1.0.0
- `/release 0.4.0` - Set explicit version

## Procedure

### Step 1: Determine New Version

1. Read current version from `Cargo.toml`
2. If argument provided:
   - `patch`: increment patch version (0.3.0 → 0.3.1)
   - `minor`: increment minor version (0.3.0 → 0.4.0)
   - `major`: increment major version (0.3.0 → 1.0.0)
   - explicit version (e.g., "0.4.0"): use as-is
3. If no argument: ask user what version type

### Step 2: Pre-flight Checks

1. Run `cargo test --quiet` - all tests must pass
2. Run `cargo clippy -- -D warnings` - no clippy warnings
3. Run `cargo fmt --check` - code must be formatted
4. Run `fastbreak check` - specification must validate
5. Check `git status` - working tree should be clean (warn if not)
6. Check we're on the main branch
7. Check for unpublished commits since last tag

### Step 3: Update Version

1. Edit `Cargo.toml` to update version
2. Run `cargo check` to update `Cargo.lock`
3. Commit both files with signed commit:
   ```
   Bump version to X.Y.Z

   Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
   ```

### Step 4: Push and Tag

1. Push commits to origin/main
2. Create signed tag: `git tag -s vX.Y.Z -m "Release vX.Y.Z"`
3. Push tag: `git push origin vX.Y.Z`

### Step 5: Publish to crates.io

1. Run `cargo publish`
2. Wait for confirmation

### Step 6: Create GitHub Release

Generate release notes from commits since last tag:

```bash
gh release create vX.Y.Z --title "vX.Y.Z" --notes "RELEASE_NOTES"
```

Release notes should include:
- **Features** - New functionality added
- **API Changes** - Breaking or notable API changes
- **Bug Fixes** - Issues resolved
- **Documentation** - Doc improvements
- **Internal** - Test count, spec verification status
- Link to full changelog

### Step 7: Verify

1. Confirm crates.io shows new version
2. Confirm GitHub release is published
3. Report success with links

## Rollback

If something goes wrong:
- Before push: `git reset --hard HEAD~1`
- After tag but before publish: `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
- After crates.io publish: Cannot unpublish, must yank: `cargo yank --version X.Y.Z`
