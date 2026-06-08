# Milestone M4: Runtime Enforcement

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (commit SHA + reproduced output,
> CI-green); CDC re-verifies. No row stays `open` at close. Posture: **always-on guards**.
> STANDING RULES ([[typed-test-discipline]], [[cc-editing-safety]]): exact `assert_eq!`/
> snapshots (never `.contains()`); test every backend; unwired ≠ done; test the actual
> subject; no blind `sed`. Headlines: **M4-3** (wrong arg → structured crash), **M4-7**
> (web-input decode). Split validators/decode to M4.5 if budget tightens.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M4-1 | **Head guards — base types:** every `defun/typed` lowers so each arg gets a native guard (`integer→is_integer`, `float→is_float`, `binary→is_binary`, `atom→is_atom`, `boolean`, `string`/`list→is_list`, `map→is_map`). | Rust: lowered form has the guards; CT: correct call works | serious | runtime-enforcement | open | | |
| M4-2 | **Head guards — ADT carriers:** tagged-tuple → `is_tuple` + tag (+arity); enum → `is_atom` + membership; transparent → underlying-type guard; native-record → `is_record` (29+, runtime deferred). | Rust + CT per backend | serious | runtime-enforcement | open | | native-record runtime `deferred` (29+) |
| M4-3 | **HEADLINE — wrong arg crashes with a structured error:** calling a typed fn with a wrong-typed arg RAISES a structured type-error (let-it-crash), not a silent pass or a bare `function_clause`. | CT: wrong-typed call raises the structured term; exact assertion | serious | runtime-enforcement | open | | |
| M4-4 | **Structured type-error term + render:** `#(type_error #{expected, got, function, arg, path})` (or agreed shape); a human-render helper prints it Gleam-style. | Rust/CT: exact snapshot of the term + the rendered string | serious | design §8, runtime-enforcement | open | | Teaching-grade; reuse type vocabulary |
| M4-5 | **Deep validators:** `(validate <type> term) -> #(ok term) | #(error type_error)`, generated per type, **recursive** over ADT fields/nested types. | Rust + CT: nested ADT validated; bad field → `#(error …)` exact | serious | runtime-enforcement | open | | Splittable → M4.5 |
| M4-6 | **`decode` membrane entry:** `(decode <type> untyped) -> #(ok T) | #(error type_error)` — graceful (NO crash) for untrusted input. | CT: valid → `#(ok …)`; invalid → `#(error type_error)` (exact) | serious | runtime-enforcement | open | | The `dynamic → T` boundary; splittable → M4.5 |
| M4-7 | **HEADLINE — web-input demo:** a fixture decodes an untyped term into an ADT; valid → `#(ok …)`, invalid → `#(error type_error)` with a teaching message (the 400 case). | CT: both paths; exact error content | serious | runtime-enforcement (motivating example) | open | | Dogfoods the membrane; splittable → M4.5 |
| M4-8 | **Guards CRASH vs validators RETURN:** the two behaviors are distinct and both correct — a head-guard violation crashes; a `decode`/`validate` failure returns `#(error …)`. | CT: same bad value crashes via a head call but returns error via `decode`; documented | serious | runtime-enforcement | open | | The deliberate split |
| M4-9 | **Cross-backend matrix:** guards + validators correct across tagged-tuple + enum + transparent; **EXACT** assertions; native-record runtime deferred. | CT matrix green (0 skipped) | serious | design §9 | open | | |
| M4-10 | **Static+runtime interplay + full regression:** always-on guards don't break static checking; M0–M3.5 suites ALL pass *with guards on*; the README example still works; line injection holds. | full CT + Rust green (0 skipped); README demo green | serious | M0–M3.5 | open | | |
| M4-11 | **Perf note — redundant guards documented/deferred:** typed→typed calls re-check args (the always-on cost); elision is a documented future optimization, not done here. | doc note present; no claim of elision | polish | runtime-enforcement | open | | `no-op`/deferred with rationale |
| M4-12 | **Standing discipline applied:** exact `assert_eq!`/snapshots (no `.contains()` on messages); every backend tested; honest statuses; CT in LFE. | grep new tests for `.contains(`; review | serious | [[typed-test-discipline]] | open | | |
| M4-13 | **Process:** `make check` clean (clippy -D, rustfmt, xref); CI green (0 skipped). | CI green; `make check` exit 0 | polish | feedback | open | | |

## What Worked

_(Filled in at close.)_

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in at close. Total rows: 13.)_
