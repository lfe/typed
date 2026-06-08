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
| S-1 | **Provider globs `.lfet`:** `typed_prv_check` finds typed files by `*.lfet` (not `*.lfe` + content-filter); all `.lfet` are treated as typed. Fixes the current latent bug (provider globbed `*.lfe` while sources were `.tlfe`). | read provider; run on a `.lfet` project — typed files found + checked | serious | latent bug | | | All `.lfet` are typed by definition |
| S-2 | **Cross-module scanner globs `.lfet`:** `cross_module.rs` scans `*.lfet` (was `find_tlfe_files`/`.tlfe`); cross-module resolution works on `.lfet` modules; diagnostic text says `.lfet`. | grep: no `.tlfe` in checker/src (except historical comments); cross-module CT green on `.lfet` | serious | inconsistency fix | | | Also the M7 X-2 flat-scan file (see S-6 optional) |
| S-3 | **Mixed-project separation:** a project with a `.lfet` typed module AND a plain `.lfe` module builds correctly — `.lfet` → typed-check, `.lfe` → stock `lfe compile`, no conflict/double-compile. | CT/integration: build the mixed project; both produce correct BEAM; assert | serious | the honesty test | | | Should be automatic (extension-based) — PROVE it |
| S-4 | **Fixtures + tests renamed:** all `*.tlfe` → `*.lfet` (via `git mv`); all CT/Rust test paths updated; every exact diagnostic SNAPSHOT embedding a `.tlfe` filename updated to `.lfet`. | full CT + Rust green; grep no stale `.tlfe` paths; numstat shows pure renames (no content loss) | serious | rename | | | Watch exact snapshots w/ filenames |
| S-5 | **Docs:** `docs/usage.md`, README, CLAUDE.md, design docs use `.lfet` (one-line "formerly `.tlfe`" aside where helpful). | docs grep clean; usage commands match real behavior | normal | docs | | | |
| S-6 | **Regression + process:** full M0–M7 suites pass under `.lfet`; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0 | serious | M0–M7 | | | |

> **Optional (fold in only if cheap, else deferred row):** recursive/project-root scan
> (M7 X-2 flat-scan limit) and provider project-wide UX (M7 X-7) both live near this
> code. Include only if trivial; otherwise leave each as a named deferred row — they
> are independent of the extension decision.

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in by CC at close: per-row walk, totals, test summary, SHA.)_
