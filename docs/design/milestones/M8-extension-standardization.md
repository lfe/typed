# Milestone M8 — Extension Standardization (`.tlfe` → `.lfet`)

> **Goal:** give typed LFE its own honest, readable extension — **`.lfet`** — and make
> the codebase consistent about it. **Builds on:** M0–M7 (all closed).
> **Type:** cleanup / consistency milestone (no new language surface).
> **Iteration budget:** 5.

## Decision & rationale (Duncan, 2026-06-08)

**Keep a distinct extension; do NOT use `.lfe`.** Typed LFE is genuinely a *non-LFE
source format*: the typed forms (`defun/typed`, `deftype`, `defrecord/typed`,
`case/typed`, `import-types`) are NOT LFE macros — they are surface syntax the Rust
checker *lowers* to plain LFE (Model Y). The stock LFE compiler **cannot parse them**.
Sharing `.lfe` would mislead humans and tooling (editors/LSP/`rebar3_lfe` would treat
the file as LFE and then choke). `.lfe` only becomes honest the day the LFE compiler
itself dispatches to `typed-check` — which requires community adoption + Robert's
official blessing of this typing approach as standard. Until then, a distinct
extension tells the truth. (Model Y stays — chosen for column-accurate diagnostics;
the "make typed forms real LFE macros so `.lfe` works" path is a separate, future
re-architecture, explicitly not in scope.)

**`.lfet`, not `.tlfe`.** Humans parse `dot + name + suffix`; `.lfet` reads as
"lfe + typed-suffix" (a modifier on a known base), which scans naturally. `.tlfe`
reads as an opaque new token. Verified safe: `*.lfe` globs match strings *ending* in
`.lfe`; `foo.lfet` ends in `.lfet`, so the stock LFE compiler's `*.lfe` glob skips
`.lfet` files exactly as it skips `.tlfe` today — the separation/protection is
preserved.

## Why a distinct extension makes this milestone SMALL

Because `.lfet` is distinct, the hard parts of the abandoned `.lfe` plan vanish:

- **No routing/exclusion problem.** The stock `lfe compile` pre-hook globs `*.lfe`
  and never sees `.lfet` files — no double-compile, no need to exclude them.
- **Detection becomes extension-based.** Glob `*.lfet`; every match is typed by
  definition. This dissolves the latent "content-detection (`defun/typed` only) misses
  a **types-only** module" gap — a types-only `.lfet` is found by the glob like any
  other.

So M8 is essentially: adopt `.lfet`, rename consistently, and fix the globs that are
currently inconsistent.

## In scope

- **Adopt `.lfet`** as the typed-file extension across the project.
- **Fix the inconsistent globs (the real bugs today):**
  - the rebar3 provider currently globs `*.lfe` + content-filters `defun/typed` →
    change to glob `*.lfet` (and drop/relax the content filter; all `.lfet` are typed);
  - the cross-module scanner (`cross_module.rs::find_tlfe_files`) globs `.tlfe` →
    `*.lfet` (this is also the file the M7 X-2 flat-scan limitation lives in — see
    "Optional" below).
- **Rename + update everywhere:** all `*.tlfe` fixtures → `*.lfet`; every CT/Rust test
  path; **every exact diagnostic snapshot that embeds a `.tlfe` filename** (filenames
  appear in messages — update exactly); diagnostic text ("no `.lfet` file declares
  module `…`"); `docs/usage.md`, README, CLAUDE.md, design docs.
- **Verify separation:** a project containing a `.lfet` typed module and a plain `.lfe`
  module builds correctly — `.lfet` routed to `typed-check`, `.lfe` to stock
  `lfe compile`, no conflict (extension-based, should be automatic — add a test that
  proves it).
- **Full M0–M7 regression** under `.lfet`; standing discipline.

## Optional (fold in only if cheap; else defer with a row)

- **Recursive project scan.** M7's `scan_project` is flat (input file's directory
  only). Since the scanner is being touched for the glob rename anyway, optionally make
  it walk the project tree (or accept a project root) so sub-dir sources resolve. If
  not cheap, leave a deferred row — it's an independent improvement, not part of the
  extension decision.
- **Provider project-wide UX** (M7 X-7, deferred): also lives near this code; fold in
  only if trivial, else it stays its own deferred item.

## Out of scope

- Making typed forms real LFE macros / any move toward `.lfe` (the future
  re-architecture; needs Robert's blessing — not now).
- Shipping `typed check` as an auto-wired published rebar3 lifecycle plugin (release
  concern).

## Definition of done

Typed files use `.lfet`; the provider globs `*.lfet`, the cross-module scanner globs
`*.lfet`, diagnostics say `.lfet`; all fixtures/tests/snapshots/docs use `.lfet`; a
mixed `.lfet` + `.lfe` project builds with correct separation (tested); full M0–M7
regression green; `make check` clean.

## Standing discipline (in force)

[[typed-test-discipline]] (exact assertions; **test the actual subject** — the mixed
`.lfet`/`.lfe` separation actually working; unwired ≠ done; status honesty) ·
[[cc-editing-safety]] (**no blind `sed`** — this is a rename milestone, the exact
hazard; rename deliberately, `git mv`, `git checkout` to recover, numstat-check that
renames dropped no content) · [[lfe-ct-tests-in-lfe]] (CT in LFE) ·
[[typed-forms-not-macros]] (why the extension must stay distinct).
