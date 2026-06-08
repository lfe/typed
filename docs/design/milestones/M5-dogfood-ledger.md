# Milestone M5: Polish & Dogfood on Real LFE

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (SHA + reproduced output, CI-green);
> CDC re-verifies. No row stays `open` at close. STANDING RULES ([[typed-test-discipline]],
> [[cc-editing-safety]], [[lfe-ct-tests-in-lfe]]): exact assertions; test the actual
> subject; unwired ≠ done; no blind `sed`; CT in LFE. Exploratory milestone — discovered
> gaps become **deferred rows with rationale**, never silent drops. Headline: **P-1** (a
> real module runs) + **P-2** (the gap inventory).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| P-1 | **HEADLINE — realistic typed module:** a non-toy module (e.g. `orders`) with SEVERAL `defun/typed` functions, ADTs (sum-of-products), real `case/typed` control flow, a `decode` boundary, and actual logic — **checks clean, compiles, and runs** with asserted behavior. | CT (LFE): build the module via the full chain; call several functions; assert real results | serious | dogfood / oracle | open | | Not one-liners; genuine logic |
| P-2 | **HEADLINE — gap inventory:** `docs/design/M5-gap-inventory.md` lists every limitation the real module surfaced (missing prelude fn, unsupported form, forced `dynamic`, ergonomic rough edge), each classified **fix-now / defer / wontfix** with a one-line rationale. | the doc exists; each item classified; cross-checked against what P-1 actually needed | serious | dogfood / oracle | open | | The oracle's report; completeness > length |
| P-3 | **Fix the fix-now gaps:** the cheap gaps from P-2 (most likely prelude expansion + small ergonomics) are implemented, each with an exact test; deferred gaps recorded with rationale. | Rust/CT: each fix-now gap has a test; P-2 marks the rest deferred | serious | P-2 | open | | Defer the rest, don't grind |
| P-4 | **Getting-started doc:** `docs/usage.md` — add `typed` to a project, write a typed module, run the checker, read a type error. First user-facing doc. | the doc exists + walks a real example end-to-end (matches actual commands/output) | serious | dogfood | open | | Keep in sync with real behavior |
| P-5 | **`rebar3` provider UX:** the `typed check` command gives clear output, **non-zero exit on failure**, help text; end-to-end build integration works in a sample project. | run the command on a good + a bad project; assert exit codes + output | serious | design §3.5 | open | | |
| P-6 | **Teaching-grade errors on real code:** breaking the realistic module (wrong return / non-exhaustive `case/typed` / bad `decode` input) yields the good diagnostics — exact. | CT/CLI: 3 break-it cases; exact diagnostic each | serious | Goal 2 | open | | Real code, not fixtures |
| P-7 | **Full regression + process:** M0–M4.6 suites ALL pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0; CI green | serious | M0–M4.6, feedback | open | | |

## What Worked

_(Filled in at close.)_

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in at close. Total rows: 7.)_
