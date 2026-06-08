# M8 — Claude Code implementation prompt (Extension Standardization `.tlfe` → `.lfet`)

> Paste into Claude Code from the `typed` project root. Adopt `.lfet` as the typed-file
> extension. Builds on closed M0–M7. This is a consistency rename + glob fixes — NOT a
> routing problem (a distinct extension already separates typed files from stock LFE).

```
You are implementing Milestone M8 ("Extension Standardization, .tlfe -> .lfet") of the
`typed` project. You are CC (implementer) under LEDGER DISCIPLINE. M0–M7 are CLOSED. This is
a cleanup/consistency milestone — NO new language surface.

# Decision & why it's SMALL
Extension = `.lfet` (distinct from `.lfe`; Duncan decided 2026-06-08). Typed LFE is a non-LFE
source format (typed forms aren't LFE macros — the Rust checker lowers them; stock LFE can't
parse them), so it must NOT share `.lfe`. Because `.lfet` is distinct:
- NO routing problem: the stock `lfe compile` pre-hook globs `*.lfe` and never matches
  `*.lfet` (a `*.lfe` glob matches strings ENDING in `.lfe`; `foo.lfet` ends in `.lfet`).
- Detection is extension-based: glob `*.lfet`; every match is typed by definition. No content
  detection needed (this also fixes the types-only-module gap).

# Read first (then STOP and confirm scope)
1. docs/design/milestones/M8-extension-standardization.md        (decision + rationale)
2. docs/design/milestones/M8-extension-standardization-ledger.md (criteria S-1..S-6)
3. src/typed_prv_check.erl (provider: currently globs *.lfe + content-filters — fix to *.lfet)
4. checker/src/cross_module.rs (find_tlfe_files / .tlfe globs — fix to *.lfet)
5. all test/fixtures/**/*.tlfe + the CT/Rust tests + Rust snapshots that embed .tlfe filenames

# STANDING RULES (NON-NEGOTIABLE)
- NO BLIND `sed`. This is a rename milestone — the exact hazard CC has been bitten by.
  Use `git mv` for file renames; `git diff --numstat` to confirm renames dropped no content;
  `git checkout` to recover. Exact assert_eq!/snapshots — filenames appear IN diagnostic
  messages, update them exactly. Test the actual subject. Unwired ≠ done. Status honesty.
  CT in LFE.

# What to do (each row gets evidence)
1. S-1 PROVIDER: change typed_prv_check to glob `*.lfet` (drop/relax the `defun/typed`
   content filter — all .lfet are typed). Verify it finds + checks typed files in a .lfet
   project.
2. S-2 SCANNER: cross_module.rs scans `*.lfet` (rename find_tlfe_files); update diagnostic
   text to ".lfet" ("no `.lfet` file declares module `...`"); remove remaining `.tlfe`
   assumptions in checker/src. Cross-module CT green on .lfet.
3. S-3 SEPARATION (the honesty test): a project with a `.lfet` typed module AND a plain
   `.lfe` module builds correctly — .lfet routed to typed-check, .lfe to stock lfe compile,
   no conflict. CT/integration: assert both produce correct BEAM. (Should be automatic since
   it's extension-based — but PROVE it.)
4. S-4 RENAME: `git mv` all test/fixtures/**/*.tlfe -> *.lfet; update every CT/Rust test path;
   update every exact Rust SNAPSHOT and CT assertion that embeds a .tlfe filename -> .lfet.
   Full suite green; grep shows no stale .tlfe paths; numstat shows pure renames.
5. S-5 DOCS: docs/usage.md, README.md, CLAUDE.md, design docs -> .lfet (one-line "formerly
   .tlfe" aside where helpful). Commands match real behavior.
6. S-6 REGRESSION: full M0–M7 green under .lfet; make check clean; CI green, 0 skipped.

# Optional (fold in ONLY if trivial; else leave a named deferred row)
- Recursive/project-root scan (M7 X-2 flat-scan limit) — scanner is already being touched.
- Provider project-wide UX (M7 X-7). 
Do NOT grind on these; a deferred row with rationale is the correct outcome if non-trivial.

# Ledger discipline
- Work S-1..S-6. Budget 5 iterations. Per-row walk at close; leave CDC section for CDC. Anchor
  done rows to the SHA; CI green.

# Definition of done
Typed files are .lfet; provider + scanner glob .lfet; diagnostics say .lfet; all
fixtures/tests/snapshots/docs use .lfet; mixed .lfet/.lfe project builds with correct
separation (tested); full regression green; make check clean.

Do NOT expand scope: no move toward .lfe / typed-forms-as-macros (future re-architecture);
no published rebar3 lifecycle plugin. Just the .lfet adoption + glob fixes + consistency.
```
