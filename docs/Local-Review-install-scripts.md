# Code Review: One-Command Install System

**Scope**: 2 commits (`511b863`, `7c2b05a`) — 7 files, +535 lines
**Reviewed**: 2026-03-26
**Base**: `origin/main`

---

## TL;DR

Clean implementation. All critical/high issues from the first review pass were fixed in the second commit. No security vulnerabilities, no blocking issues. Two minor improvements remain.

**Verdict: PASS — ready to ship.**

---

## Findings

### MEDIUM-1: `grep` without `-F` for artifact lookup in checksums

**File**: `install.sh:108`
**Category**: Correctness

```sh
EXPECTED=$(grep "${ARTIFACT}" "${WORK_DIR}/checksums.txt" | awk '{print $1}')
```

`ARTIFACT` is `syncfu-darwin-arm64` — the hyphens are literal in regex and current names have no dots, so this works today. But if artifact names ever include dots (e.g., `syncfu-v0.2.0-darwin-arm64`), dots would match any character. `grep -F` (fixed string) is the defensive choice.

**Suggestion**: `grep -F "${ARTIFACT}"` — zero-cost fix.

---

### MEDIUM-2: "From source" CLI install missing clone instructions

**File**: `README.md:85-87`

```markdown
### From source
cargo install --path cli
```

This requires being in a cloned repo, but the clone command only appears under "Desktop app" below it. A user wanting CLI-from-source would miss it.

**Suggestion**: Add `git clone ... && cd syncfu` before `cargo install`, or note "requires cloned repo".

---

### LOW-1: PowerShell 5.1 redirect header access

**File**: `install.ps1:22`

```powershell
$RedirectUrl = $Response.Headers.Location
```

On PS 5.1, `HttpWebResponse.Headers` uses indexer syntax (`["Location"]`), not property access. The fallback to GitHub API on line 23-25 handles this gracefully (if Location is null, it hits the API instead), so it degrades correctly — just an unnecessary API call on older Windows.

---

### LOW-2: `softprops/action-gh-release@v2` unpinned

**File**: `release-cli.yml:77`

Mutable tag. Consider pinning to a commit SHA for supply-chain hardening once the workflow is proven stable.

---

### LOW-3: `strip = true` applies to entire workspace

**File**: `Cargo.toml:7`

Strips all symbols including panic locations from both CLI and Tauri app. For CLI distribution this is ideal. For local Tauri development, `cargo build` (dev profile) is unaffected. Only matters if someone debugs a release Tauri build.

---

## Verification Walkthrough

### Flow: `curl | sh` on macOS arm64
1. Detects `Darwin`/`arm64` → artifact `syncfu-darwin-arm64`
2. curl pre-check passes
3. Resolves version via GitHub redirect (no API rate limit hit)
4. Semver validated
5. Downloads binary, checks HTTP 200 (not file size)
6. Downloads `checksums.txt`, verifies via `shasum -a 256`
7. If no shasum: warns honestly, does NOT spoof match
8. `chmod +x`, `xattr -d` quarantine removal
9. Installs to `/usr/local/bin` or `~/.syncfu/bin`
10. Prints PATH instructions for zsh

### Flow: `irm | iex` on Windows
1. `param()` correctly at line 1 — `$Version` binds from CLI args
2. Resolves version via redirect (PS 7+) or API fallback (PS 5.1)
3. Semver validated via regex
4. Downloads to temp dir with `-UseBasicParsing`
5. Checksum: both sides `.ToLower()` before compare
6. Moves to `$HOME\.syncfu\bin`, adds to user PATH via exact split match
7. `finally{}` always cleans temp dir

### Flow: GitHub Actions release
1. Tag `v*` triggers 5 parallel matrix builds
2. ARM64 Linux: gets cross-linker + cmake + GITHUB_ENV scoped linker var
3. Other targets: native compilation, no cross env leakage
4. Artifacts uploaded individually, then merged in release job
5. `sha256sum` generates checksums, `softprops/action-gh-release` publishes

---

## Re-verification Audit

| Finding | Calibration | Notes |
|---------|-------------|-------|
| MEDIUM-1 (grep -F) | Confirmed MEDIUM | No current risk but trivial to fix |
| MEDIUM-2 (README clone) | Confirmed MEDIUM | UX confusion, not functional |
| LOW-1 (PS 5.1) | Confirmed LOW | Graceful fallback exists |
| LOW-2 (unpinned action) | Confirmed LOW | Acceptable for new project |
| LOW-3 (strip workspace) | Confirmed LOW | Only affects release Tauri builds |

**Missed-item check**: All entry points covered (curl, irm, tag push). Error paths traced (download failure, checksum failure, missing tools). No security issues found.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 2 |
| LOW | 3 |

**Merge readiness: PASS**
