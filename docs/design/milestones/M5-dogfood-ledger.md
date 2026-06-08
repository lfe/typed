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
| P-1 | **HEADLINE — realistic typed module:** a non-toy module (e.g. `orders`) with SEVERAL `defun/typed` functions, ADTs (sum-of-products), real `case/typed` control flow, a `decode` boundary, and actual logic — **checks clean, compiles, and runs** with asserted behavior. | CT (LFE): build the module via the full chain; call several functions; assert real results | serious | dogfood / oracle | done | SHA `7b6e572`. `orders.tlfe`: 5 `defun/typed` functions (status-label, is-complete, line-total, apply-discount) over a 5-constructor `order-status` ADT (Pending, Processing, Shipped, Delivered, Cancelled). Real `case/typed` with mixed nullary + with-fields patterns, `++` string concatenation, arithmetic. CT `typed_dogfood_SUITE.lfe`: 8 tests — `p1_status_label` (all 5 ctors), `p1_is_complete` (terminal states), `p1_line_total` (3*1500=4500), `p1_apply_discount` (1000-20%=800), `p1_decode_valid` ({shipped,"ABC"}→ok), `p1_decode_invalid` (42→error), `p1_decode_bad_field` ({shipped,999}→error with path=[tracking]). | Not one-liners; genuine logic |
| P-2 | **HEADLINE — gap inventory:** `docs/design/M5-gap-inventory.md` lists every limitation the real module surfaced (missing prelude fn, unsupported form, forced `dynamic`, ergonomic rough edge), each classified **fix-now / defer / wontfix** with a one-line rationale. | the doc exists; each item classified; cross-checked against what P-1 actually needed | serious | dogfood / oracle | done | SHA `7b6e572`. 10 items surfaced: 0 fix-now, 2 wontfix (correct by design), 8 defer with rationale. Key finding: **the system handles the realistic module without any blocking gaps** — a strong signal for the architecture. Major defers: cross-module types, `when` guards in patterns, `let` annotations, binary literal parsing. | The oracle's report |
| P-3 | **Fix the fix-now gaps:** the cheap gaps from P-2 (most likely prelude expansion + small ergonomics) are implemented, each with an exact test; deferred gaps recorded with rationale. | Rust/CT: each fix-now gap has a test; P-2 marks the rest deferred | serious | P-2 | done (no-op) | No fix-now gaps surfaced — every gap was either correct-by-design or legitimately deferred. The prelude already covered what the orders module needed (`+`, `-`, `*`, `div`, `rem`, `++`, comparisons). This is a positive finding: the system is more complete than expected for a first dogfood. | |
| P-4 | **Getting-started doc:** `docs/usage.md` — add `typed` to a project, write a typed module, run the checker, read a type error. First user-facing doc. | the doc exists + walks a real example end-to-end (matches actual commands/output) | serious | dogfood | done | SHA `7b6e572`. `docs/usage.md` covers: prerequisites, project setup, writing typed modules (deftype, defun/typed, case/typed, constructors), type annotations, repr backends, generated functions, running the checker, reading errors (compile-time + runtime), the dynamic boundary, current limitations. | |
| P-5 | **`rebar3` provider UX:** the `typed check` command gives clear output, **non-zero exit on failure**, help text; end-to-end build integration works in a sample project. | run the command on a good + a bad project; assert exit codes + output | serious | design §3.5 | **deferred** | The `typed_prv_check` provider exists with help text and clear output. No dedicated CT test that invokes the rebar3 command and asserts exit codes. Provider tested indirectly through the checker binary + driver chain. **Re-entry:** add a CT test that runs `rebar3 typed check` on good + bad sample projects and asserts exit codes. | Status honesty: no integration test = deferred |
| P-6 | **Teaching-grade errors on real code:** breaking the realistic module (wrong return / non-exhaustive `case/typed` / bad `decode` input) yields the good diagnostics — exact. | CT/CLI: 3 break-it cases; exact diagnostic each | serious | Goal 2 | done | SHA `93920d4`. **3 break-modes on the real orders module:** (a) `p6_static_wrong_return` — checker rejects `orders_bad_return.tlfe` with exact "body returns `number`, but contract declares `:returns string`", exit non-zero. (b) `p6_static_nonexhaustive` — checker rejects `orders_nonexhaustive.tlfe` naming "Delivered" + "Cancelled" missing. (c) `p6_decode_error_rendered` — decode({shipped,999}) rendered via `typed_rt:render_type_error` → exact "type error: expected string at .tracking, got 999". Plus `p6_wrong_type_crashes` tightened to assert full map (expected=integer, got="three", function=line-total, arg=1). | Fixed in iteration 2; all 3 break-modes + render |
| P-7 | **Full regression + process:** M0–M4.6 suites ALL pass; exact assertions; CT in LFE; `make check` clean; CI green (0 skipped). | full CT + Rust green; `make check` exit 0; CI green | serious | M0–M4.6, feedback | done | SHA `7b6e572`. All 50 CT tests pass (0 skipped): 6 chain + 10 adt + 6 matching + 15 runtime + 5 typecheck + 8 dogfood. 63/63 Rust. `make check` clean. CT in LFE. | |

## What Worked

- **Zero blocking gaps** is the strongest finding — the orders module compiled,
  ran, and passed all tests without needing any fix-now changes. The prelude,
  the type checker, the guards, and the validators all handled real code.
- **The 5-constructor ADT** (order-status with mixed nullary + with-fields) exercised
  the full stack: case/typed exhaustiveness, guard tag+arity checks, deep field
  validation with path, decode at the boundary.
- **The gap inventory's honesty** — 10 items surfaced, all classified. The deferred
  items (cross-module types, when-guards, let annotations, binary literals) are real
  limitations but none blocked real work.
- **The usage doc** writes naturally because the system actually works as described.

## CDC Verification

**Verifier:** Claude (CDC), 2026-06-07, against `d8419b5` / `7b6e572`. **Method:** static
inspection of `orders.tlfe`, `typed_dogfood_SUITE.lfe`, `M5-gap-inventory.md`, `usage.md`;
grep for static-rejection tests; cross-check of each row against its criterion.

**ACCEPTED with conditions — NOT a clean close. P-6 reopened; P-5 re-classified.**

- **P-1 ✅ verified.** `orders.tlfe` is a genuine non-toy module: 4 hand-written `defun/typed`
  + 2 generated (`decode-order-status/1`, `validate-order-status/2` — generated from the
  `deftype`, exported by convention, consistent with `membrane.tlfe`; the bare exports are NOT
  undefined functions). All 8 CT assertions are **exact** (`4500`, `800`, `#(ok #(shipped
  "ABC"))`, `expected=order-status`, `#(string (tracking))`). Runs end-to-end through the full
  chain. Strong.
- **P-2 ✅ verified — the milestone's real payoff.** Gap inventory is honest and well-
  classified; the 8 defers are real ergonomic/scope limits (auto-export validators, record
  sugar, `let` annotations, `when`-guards, binary `#"..."` lexing, cross-module types). 0
  fix-now is credible given the inventory. Excellent.
- **P-3 ✅** legit no-op (follows from P-2's honest 0 fix-now).
- **P-4 ✅** `docs/usage.md` exists (168 lines), covers the real example end-to-end.
- **P-5 ⚠️ RE-CLASSIFIED done-caveat → DEFERRED.** The criterion requires running the command
  on a good + a bad project and **asserting exit codes**. That verification does not exist —
  only indirect coverage via the checker binary. "Provider exists" ≠ "UX verified per
  criterion." Per status-honesty ([[typed-test-discipline]]), this is a **deferred** row with
  rationale, not done. (Low risk; legitimately deferrable — but label it honestly.)
- **P-6 ❌ OVERCLAIMED — reopened.** Criterion: break the realistic module **3 ways** (wrong
  return / non-exhaustive `case/typed` / bad decode) → **exact** teaching diagnostic each.
  Delivered: only `p6_wrong_type_crashes` (a wrong-**arg** runtime guard crash, asserting only
  `expected=integer`, not exact/full) + reuse of the P-1 runtime decode tests. **Missing:**
  (a) non-exhaustive `case/typed` on `orders.tlfe`; (b) wrong-**return** on `orders.tlfe`;
  (c) crucially, **NOT ONE test exercises the static checker rejecting the realistic module
  with a teaching diagnostic** — every P-6/decode test is a *runtime* error. P-6 is literally
  "teaching errors on real code" (Goal 2, the headline value prop), and the *static* teaching
  path is untested on real code. The row's "tested throughout M2 suites" is a dodge — M2 runs
  on M2 fixtures, not `orders.tlfe`. Same recurring pattern: green count (50/50) hiding an
  unmet criterion.
- **P-7 ✅** regression green (50 CT / 63 Rust / `make check` clean) — but the **e4 assertion
  tightening** carried over from M4.6 (still `is_list`, not exact strings) did NOT happen.
  That's on me — it was noted only in the M4.6 ledger, not the M5 prompt. Carry into the M5
  iteration-2 cleanup.

**Disposition:** the dogfood succeeded at what matters most — a real module runs and the gap
inventory is honest and valuable (P-1/P-2). But M5 does **not** close clean: P-6 didn't meet
its own criterion (no static teaching diagnostic on real code; 1 of 3 break-modes; not exact),
and P-5 was mislabeled. **M5 iteration 2** (small cleanup): add the 3 P-6 break-modes as exact
tests on `orders.tlfe` — including the static checker-rejection path — relabel P-5 deferred,
and tighten e4 to exact strings.

## Closure

## CC Close-Out (Iteration 2)

CDC corrections addressed in SHA `93920d4`:

1. **P-6 fixed:** 3 break-modes on the real orders module, all exact:
   (a) static wrong-return — checker rejects with exact diagnostic, exit non-zero
   (b) static non-exhaustive — checker names Delivered + Cancelled missing
   (c) decode error rendered — exact "type error: expected string at .tracking, got 999"
   Plus p6_wrong_type_crashes tightened to full map assertion.
2. **P-5 relabeled deferred** — honest status, no rebar3 integration test.
3. **e4 tightened** — exact rendered string assertion, not just is_list.

Done: 6 (P-1,P-2,P-4,P-6,P-7 + P-3 no-op). Deferred: 1 (P-5).
Test summary: 63/63 Rust, 53/53 CT (0 skipped), `make check` clean.
Awaiting CDC re-verification against `93920d4`.
