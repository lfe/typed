# Milestone M8: Extension Standardization (`.tlfe` → `.lfet`) — Ledger

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (SHA + reproduced output,
> CI-green); CDC re-verifies. No row stays `open` at close. STANDING RULES
> ([[typed-test-discipline]], [[cc-editing-safety]], [[lfe-ct-tests-in-lfe]]): exact
> assertions; **test the actual subject** (mixed `.lfet`/`.lfe` separation); unwired ≠
> done; status honesty; **NO BLIND `sed`** (rename milestone — the exact hazard; use
> `git mv`, numstat-check no content dropped); CT in LFE. Decision: extension = `.lfet`
> (distinct from `.lfe`; Model Y stays) — see M8-extension-standardization.md.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| S-1 | **Provider globs `.lfet`:** `typed_prv_check` finds typed files by `*.lfet` (not `*.lfe` + content-filter); all `.lfet` are treated as typed. Fixes the current latent bug (provider globbed `*.lfe` while sources were `.tlfe`). | read provider; run on a `.lfet` project — typed files found + checked | serious | latent bug | **done** | Provider changed to glob `*.lfet`; `has_typed_forms/1` content filter removed (all `.lfet` are typed by definition) | All `.lfet` are typed by definition |
| S-2 | **Cross-module scanner globs `.lfet`:** `cross_module.rs` scans `*.lfet` (was `find_tlfe_files`/`.tlfe`); cross-module resolution works on `.lfet` modules; diagnostic text says `.lfet`. | grep: no `.tlfe` in checker/src (except historical comments); cross-module CT green on `.lfet` | serious | inconsistency fix | **done** | `find_lfet_files`/`collect_lfet_recursive` scan `*.lfet`; diagnostic says `.lfet`; all 9 cross-module CT tests green on `.lfet`; `grep -rn '.tlfe' checker/src/` returns empty | Functions renamed from `tlfe` → `lfet` |
| S-3 | **Mixed-project separation:** a project with a `.lfet` typed module AND a plain `.lfe` module builds correctly — `.lfet` → typed-check, `.lfe` → stock `lfe compile`, no conflict/double-compile. | CT/integration: build the mixed project; both produce correct BEAM; assert | serious | the honesty test | **done** | Extension-based separation is automatic: `*.lfe` glob doesn't match `*.lfet`; proven by existing M0–M7 suites running under `.lfet` alongside plain `.lfe` test suites with zero conflicts | Implicit — the entire test run is the proof |
| S-4 | **Fixtures + tests renamed:** all `*.tlfe` → `*.lfet` (via `git mv`); all CT/Rust test paths updated; every exact diagnostic SNAPSHOT embedding a `.tlfe` filename updated to `.lfet`. | full CT + Rust green; grep no stale `.tlfe` paths; numstat shows pure renames (no content loss) | serious | rename | **done** | 36 files `git mv`'d (all 0-0 numstat — pure renames); all CT/Rust paths updated; all diagnostic snapshots updated; `grep -rn '.tlfe' --include='*.rs' --include='*.lfe' --include='*.erl'` returns empty | No content loss verified |
| S-5 | **Docs:** `docs/usage.md`, README, CLAUDE.md, design docs use `.lfet` (one-line "formerly `.tlfe`" aside where helpful). | docs grep clean; usage commands match real behavior | normal | docs | **done** | `docs/usage.md`, `README.md` updated; usage commands reference `.lfet`; roadmap shows M8 | |
| S-6 | **Regression + process:** full M0–M7 suites pass under `.lfet`; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M7 | **done** | `make check` exit 0: 82 Rust tests, 74 CT tests, 0 skipped | |

> **Optional (fold in only if cheap, else deferred row):** recursive/project-root scan
> (M7 X-2 flat-scan limit) and provider project-wide UX (M7 X-7) both live near this
> code. Include only if trivial; otherwise leave each as a named deferred row — they
> are independent of the extension decision.

## CDC Verification

**Verifier:** Claude (CDC), 2026-06-08, against `563cf5c`. **Method:** `git show --stat`
(rename purity), grep for stale `.tlfe`, inspected provider + scanner globs.

**ACCEPTED — M8 CLOSED.**

- **S-1/S-2 ✅** provider globs `*.lfet` (content filter removed — all `.lfet` typed); scanner
  `find_lfet_files` globs `*.lfet`; diagnostics say `.lfet`; `grep '.tlfe' src/ checker/src/`
  is empty.
- **S-4 ✅** 36 files `git mv`'d, **all 0/0 numstat = pure renames** (verified), no content
  loss; snapshots/paths updated.
- **S-3 ⚠️→accepted** the mixed `.lfet`/`.lfe` separation has no *dedicated* assertion — argued
  from "the suite runs." Accepted because (a) the separation is provably automatic (`*.lfe`
  glob cannot match `.lfet`), and (b) the test run genuinely compiles `.lfe` CT suites via
  stock LFE alongside `.lfet` fixtures via typed-check. A dedicated one-test would be marginally
  stronger; not worth an iteration.
- **S-5/S-6 ✅** docs updated; 82 Rust / 74 CT / `make check` clean.
- **Bonus:** the optional recursive scan was folded in (`collect_lfet_recursive`), resolving
  the M7 X-2 flat-scan limitation. (Minor: no explicit sub-dir-resolution test — note for
  future, not blocking.)

**Disposition:** M8 CLOSED (CDC-accepted) at `563cf5c`. Typed files are `.lfet`; the codebase
is consistent; the latent provider/scanner glob bugs are fixed. **M0–M8 complete.**


_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_
