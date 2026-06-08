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
| M4-1 | **Head guards — base types:** every `defun/typed` lowers so each arg gets a native guard (`integer→is_integer`, `float→is_float`, `binary→is_binary`, `atom→is_atom`, `boolean`, `string`/`list→is_list`, `map→is_map`). | Rust: lowered form has the guards; CT: correct call works | serious | runtime-enforcement | done | SHA `e849bd0`. `guards.rs:guard_for_type` maps all base types. `lower_typed_fun` emits `match-lambda` with `(when ...)` guards. CT `m4_1_correct_call_passes` — `guarded:double(21) = 42`. | |
| M4-2 | **Head guards — ADT carriers:** tagged-tuple → `is_tuple` + tag (+arity); enum → `is_atom` + membership; transparent → underlying-type guard; native-record → `is_record` (29+, runtime deferred). | Rust + CT per backend | serious | runtime-enforcement | done | SHA `ae27641`. `guard_for_tagged_tuple_adt` generates per-constructor guard: nullary → `X =:= 'tag'`; with fields → `(andalso (is_tuple X) (=:= (element 1 X) 'tag) (=:= (tuple_size X) N))`. CT `m4_2_wrong_tag_rejected` — `{bogus,1}` raises `{type_error, [{expected,'order-status'}, ...]}` while `'pending'` passes. Enum membership unchanged. native-record deferred (29+). | Tag+arity fixed in iteration 2 |
| M4-3 | **HEADLINE — wrong arg crashes with a structured error:** calling a typed fn with a wrong-typed arg RAISES a structured type-error (let-it-crash), not a silent pass or a bare `function_clause`. | CT: wrong-typed call raises the structured term; exact assertion | serious | runtime-enforcement | done | SHA `c0053cc`. Run-verified: CT `m4_3_wrong_arg_crashes` — `guarded:double("not-an-integer")` raises `{type_error, [{expected,integer}, {function,double}, ...]}`, NOT `function_clause`. The guard fallback clause raises the structured error. | |
| M4-4 | **Structured type-error term + render:** `#(type_error #{expected, got, function, arg, path})` (or agreed shape); a human-render helper prints it Gleam-style. | Rust/CT: exact snapshot of the term + the rendered string | serious | design §8, runtime-enforcement | done (caveat) | SHA `c0053cc`. Run-verified: CT `m4_4_structured_error_fields` — error term carries `expected=integer`, `got="oops"`, `function=double`, `arg=1`. All fields present. **Caveat:** human-render helper not yet implemented (the term itself is teaching-grade; render helper deferred to M4.5). | Teaching-grade term; render helper → M4.5 |
| M4-5 | **Deep validators:** `(validate <type> term) -> #(ok term) | #(error type_error)`, generated per type, **recursive** over ADT fields/nested types. | Rust + CT: nested ADT validated; bad field → `#(error …)` exact | serious | runtime-enforcement | **deferred** | Deferred to M4.5. Guards (the non-negotiable core) are done; validators are a separate sub-system. | Splittable → M4.5 |
| M4-6 | **`decode` membrane entry:** `(decode <type> untyped) -> #(ok T) | #(error type_error)` — graceful (NO crash) for untrusted input. | CT: valid → `#(ok …)`; invalid → `#(error type_error)` (exact) | serious | runtime-enforcement | **deferred** | Deferred to M4.5. | The `dynamic → T` boundary; splittable → M4.5 |
| M4-7 | **HEADLINE — web-input demo:** a fixture decodes an untyped term into an ADT; valid → `#(ok …)`, invalid → `#(error type_error)` with a teaching message (the 400 case). | CT: both paths; exact error content | serious | runtime-enforcement (motivating example) | **deferred** | Deferred to M4.5. | Dogfoods the membrane; splittable → M4.5 |
| M4-8 | **Guards CRASH vs validators RETURN:** the two behaviors are distinct and both correct — a head-guard violation crashes; a `decode`/`validate` failure returns `#(error …)`. | CT: same bad value crashes via a head call but returns error via `decode`; documented | serious | runtime-enforcement | **deferred** | Deferred to M4.5 — requires validators/decode (M4-5/M4-6) first. Guards-crash half is done (M4-3). | The deliberate split |
| M4-9 | **Cross-backend matrix:** guards + validators correct across tagged-tuple + enum + transparent; **EXACT** assertions; native-record runtime deferred. | CT matrix green (0 skipped) | serious | design §9 | done (guards only) | SHA `c0053cc`. Guards work across backends: tagged-tuple (describe demo), base types (double). Validator matrix deferred to M4.5. native-record runtime deferred (29+). | Guards matrix done; validator matrix → M4.5 |
| M4-10 | **Static+runtime interplay + full regression:** always-on guards don't break static checking; M0–M3.5 suites ALL pass *with guards on*; the README example still works; line injection holds. | full CT + Rust green (0 skipped); README demo green | serious | M0–M3.5 | done | SHA `c0053cc`. All 30 CT tests pass (0 skipped): 6 chain + 10 adt + 6 matching + 5 typecheck + 3 runtime. All 63 Rust tests pass. README describe demo still works WITH guards on. Line injection preserved (m2_12, m1_12). | |
| M4-11 | **Perf note — redundant guards documented/deferred:** typed→typed calls re-check args (the always-on cost); elision is a documented future optimization, not done here. | doc note present; no claim of elision | polish | runtime-enforcement | done | Always-on guards are the chosen posture. Redundant-guard elision documented as a future optimization in M4-runtime-enforcement.md §Out of scope. No claim of elision. | `no-op` with rationale |
| M4-12 | **Standing discipline applied:** exact `assert_eq!`/snapshots (no `.contains()` on messages); every backend tested; honest statuses; CT in LFE. | grep new tests for `.contains(`; review | serious | [[typed-test-discipline]] | done | SHA `c0053cc`. CT tests use exact pattern matching on error term fields (not `.contains()`). Status honesty: validators/decode/web-demo honestly `deferred` to M4.5 rather than claimed done. | |
| M4-13 | **Process:** `make check` clean (clippy -D, rustfmt, xref); CI green (0 skipped). | CI green; `make check` exit 0 | polish | feedback | done | SHA `c0053cc`. `make check` clean. 63/63 Rust, 30/30 CT (0 skipped). `typed_runtime_SUITE.lfe` — 3 tests, LFE patterns. | |

## What Worked

- **match-lambda with guard + fallback** is the right lowering shape — it compiles
  cleanly through `lfe_codegen`, the guards work natively, and the fallback clause
  provides a clean structured type-error instead of a bare `function_clause`.
- **Mixed nullary+non-nullary guard** (`orelse is_tuple is_atom`) was the key insight
  for tagged-tuple ADTs where some constructors are atoms (nullary) and others are
  tuples (with fields). Without this, the describe demo would false-reject `pending`.
- **Full regression held on first try** — adding always-on guards to every typed
  function didn't break any existing fixture or test. The M0-M3.5 chain is robust.
- **Structured type-error term** carries enough info (expected, got, function, arg)
  for both logging and programmatic handling — the foundation for validators/decode.

## M4/M4.5 Split Proposal

**Guards + structured error are done** (the non-negotiable core, per the milestone's
own split boundary). **Validators/decode/web-demo are deferred to M4.5:**

| Done (M4) | Deferred (M4.5) |
|---|---|
| M4-1/M4-2 head guards (base + ADT) | M4-5 deep validators |
| M4-3 wrong arg → structured crash | M4-6 decode membrane |
| M4-4 structured error term (caveat: render helper) | M4-7 web-input demo |
| M4-9 guard matrix (guards only) | M4-8 guards-vs-validators duality |
| M4-10 full regression WITH guards | M4-4 caveat: human render helper |
| M4-11/M4-12/M4-13 process | |

This follows the milestone's own guidance: "Guards + the structured error are the
non-negotiable core; validators + decode are the splittable half."

## CDC Verification

**Verifier:** Claude (CDC), 2026-06-07, against `c0053cc` / `fc083df`. **Method:**
static inspection of `guards.rs`, the lowering, and the runtime CT suite.

**Split accepted:** the M4/M4.5 split (guards core in M4; deep validators + `decode` +
web-demo + duality in M4.5) is the **pre-authorized** boundary from the milestone doc —
legitimate, not goalpost-moving, and it didn't blow the cap. M4.5 carries M4-5/6/7/8 +
the M4-4 render helper. ✅

**Genuinely done (verified):**
- **M4-3 (headline):** `m4_3_wrong_arg_crashes` catches `#(error #(type_error info) _)`,
  asserts `expected=integer` + `function=double` exactly, and **fails on `function_clause`**
  — proving the structured error is raised, not a bare clause error. Real, exact. ✅
- **M4-1 base guards** (is_integer/is_binary/…); **M4-10 full regression WITH guards on**
  (30/30 CT, 63/63 Rust, README demo still works — the always-on-changes-every-head risk
  did not bite); honest statuses; exact assertions (no `.contains()`).

**Finding requiring correction:**
- **M4-2 → `open`: the tagged-tuple head guard checks SHAPE, not the TAG.** For
  constructors-with-fields, `guard_for_adt` emits bare `is_tuple` (and `is_tuple orelse
  is_atom` for mixed) — so **any** tuple passes as a valid value of that ADT (e.g.
  `{not_a_ctor, 1}` slips through). The criterion (M4-2) said **"is_tuple + tag (+arity)"**,
  and a tag/arity check is **guard-legal and cheap** (`element(1,X) =:= 'shipped' orelse …`
  + `tuple_size(X) =:= N`), and it's what makes an always-on guard *meaningful* for ADTs
  under the max-safety posture you chose. (Enum/all-nullary already check membership —
  those are tight; the gap is specifically constructors-with-fields.) **Fix:** generate a
  tag (+arity) check at the head; add a test that a *wrong-tagged* tuple is REJECTED
  (structured crash), exact. Deep field-type validation stays the validator's job (M4.5) —
  tag+arity at the head is O(1) and distinct from that.

**Note-only:** the structured error is a **proplist** `[{expected,…},…]`, while the doc
wrote a **map** `#{…}`. Functionally fine; consider standardizing on a map (nicer
`maps:get`/pattern access) when the render helper lands in M4.5. Native-record runtime
stays deferred (OTP 29+).

**Disposition:** the guards core + headline are real and the split is legitimate — but
**M4-2 is under-delivered** (shape-only ADT guard vs the specified tag check). One small
close-out: add tag(+arity) checks for tagged-tuple-with-fields + a wrong-tag rejection test.

## Closure

**CC close-out (iteration 2), SHA `ae27641`:**
M4-2 fixed: tagged-tuple head guards now check tag+arity per constructor, not just
`is_tuple`. `guard_for_tagged_tuple_adt` generates per-constructor `orelse` of
`(andalso is_tuple element(1)=tag tuple_size=N)` for ctors with fields + `X =:= 'tag'`
for nullary ctors. CT `m4_2_wrong_tag_rejected` proves `{bogus,1}` raises structured
type-error while `'pending'` passes.

Done: 9 (M4-1,2,3,4,9,10,11,12,13). Done with caveat: 1 (M4-4, render helper → M4.5).
Deferred to M4.5: 4 (M4-5,6,7,8).
Test summary: 63/63 Rust, 31/31 CT (0 skipped), `make check` clean.
Awaiting CDC re-verification against `ae27641`.
