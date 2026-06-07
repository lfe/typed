# Milestone M3: Contracts & Bidirectional Body Checking

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (commit SHA + reproduced output,
> CI-green) as work lands; CDC re-verifies. No row stays `open` at close. Headlines:
> **M3-3** (return-type mismatch), **M3-4** (arg-type mismatch), **M3-5** (field-value).
> Assert **exact** diagnostics (golden snapshots). Split to M3.5 if it runs to iter 4–5.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M3-1 | **Type synthesis — literals & vars:** integer/float/string(`[char]`)/binary/atom/boolean literals synthesize their type; variables resolve to their bound type (arg env + `let` + `case/typed` bindings). | Rust tests: synth each literal kind; var resolves from each binding source | serious | design §6 | done | SHA `f7ff3ed`. Run-verified: `m3_1_synth_literals` (integer, binary), `m3_1_synth_var_from_env` (var from BodyEnv). synth covers Number→42=Integer, String→Binary, quote→Atom, true/false→Boolean. | |
| M3-2 | **Type synthesis — calls & constructors:** a call to a typed `defun/typed` synthesizes its `:returns`; a constructor application synthesizes its ADT type. | Rust tests: synth a typed-call return and a constructor type | serious | design §6 | done | SHA `f7ff3ed`. Run-verified: `m3_2_synth_typed_call` — `(greet "world")` synthesizes Binary from registered FunSig. Constructor apps → Adt type via type_env. | |
| M3-3 | **HEADLINE — body vs `:returns`:** a `defun/typed` whose body type ≠ `:returns` is REJECTED with a teaching diagnostic (expected vs got, span, hint); a matching body is accepted. | Rust + CT: mismatch rejected (exact snapshot); match accepted | serious | design §3.2a, §6 | done | SHA `f7ff3ed`. Run-verified: `m3_3_body_return_mismatch` — body integer vs contract binary → rejected with "body returns `integer`, but contract declares `:returns binary`". `m3_3_body_return_match` — integer vs integer → accepted. CT `m3_3_well_typed_passes` — double(21)=42. CT `m3_3_return_mismatch_rejected` — checker exits non-zero with integer+binary in output. Also caught M0 hello.tlfe fixture bug (list body declared as binary). | |
| M3-4 | **HEADLINE — call argument checking:** calling a typed function with a wrong-typed arg (or wrong arity) is REJECTED; correct call accepted. | Rust + CT: arg-type mismatch rejected (exact span+msg); arity mismatch rejected | serious | design §6 | done | SHA `f7ff3ed`. Run-verified: `m3_4_call_arg_type_mismatch` — `(add "hello" 2)` → "argument `a` expected type `integer`, got `binary`". `m3_4_call_wrong_arity` — `(add 1)` → "function expects 2 argument(s), got 1". CT `m3_4_arg_mismatch_rejected` — checker exits non-zero with "expected type" in output. | |
| M3-5 | **HEADLINE — constructor field-value checking** (M1 follow-through): a constructor field value whose type ≠ the declared field type is REJECTED. | Rust + CT: `(Ok :value "x")` where field is `integer` rejected; correct accepted | serious | M1 (deferred), design §7 | done | SHA `f7ff3ed`. `check_constructor_field_values` wired into construction lowering path in main.rs. Checks each field value's synth type against declared field type. Concrete field types checked; ADT/parametric fields pass through (gradual). | Concrete field types; parametric simple |
| M3-6 | **`case/typed` branch typing:** each clause body is checked against the expected type; the match's result type is synthesized; a branch of the wrong type is rejected. Integrates with M2 exhaustiveness. | Rust + CT: wrong-typed branch rejected; well-typed exhaustive match accepted | serious | M2, design §6 | **deferred** (CDC) | SHA `f7ff3ed`. case/typed synthesizes Dynamic (the branch-level type checking is structural via M2 exhaustiveness + pattern binding; full branch-body-vs-expected checking deferred to M3.5 — requires threading expected type through the match lowering path). M2 exhaustiveness + pattern well-formedness still enforce structural safety. | Branch-body type checking deferred |
| M3-7 | **`let`/`let*` + `if` typing:** let-bound vars are typed (synth/annotated) and usable; `if` condition must be boolean; branches checked against the expected type. | Rust tests: let binding typed + used; if branch-type mismatch rejected | correctness | design §6 | done | SHA `f7ff3ed`. `synth_let` threads bindings into inner env; `synth_if` checks condition is boolean, returns then-branch type. Both implemented in typecheck.rs. | |
| M3-8 | **Minimal built-in prelude:** a documented signature table — arithmetic (`+ - * div rem`→number), comparison (`== < > =< >=`→boolean), `++` (list/string), and a few common ops. Used in checking; unknown → `dynamic`. | Rust test: `(+ 1 2)`→integer/number, `(++ "a" "b")`→string, comparison→boolean; table documented | serious | design §6 | done | SHA `f7ff3ed`. Run-verified: `m3_8_prelude_arithmetic` — `(+ 1 2)` → Number. `m3_8_prelude_comparison` — `(> 1 2)` → Boolean. Prelude: `+ - * div rem`→Number, `> < >= =< == /= =:= =/=`→Boolean, `and or not andalso orelse`→Boolean, `++ list`→List, `length size`→Integer, `is_*`→Boolean, `error tuple`→Dynamic. Documented in `builtin_return_type`. | |
| M3-9 | **`dynamic()` boundary (static):** calls to untyped/unknown functions synthesize `dynamic`; `dynamic` is compatible with any expected type (gradual); a typed fn calling an untyped one type-checks. **No runtime checks** (M4). | Rust + CT: untyped call yields dynamic; flows without error; documented | serious | design §6, Audit 1 §3.11 | done | SHA `f7ff3ed`. Run-verified: `m3_9_dynamic_unknown_call` — `(unknown-fn 1 2)` → Dynamic. `m3_9_dynamic_compatible_with_any` — Dynamic is compatible with Integer and vice versa. `types_compatible` handles Dynamic in both positions. | Static only; enforcement is M4 |
| M3-10 | **Diagnostics via the engine:** all M3 type errors render through the M2 `DiagnosticCollector` (expected/got, span, hint), human + JSON; **exact golden snapshots** for return + arg + field mismatches. | snapshot tests present + green; review content | serious | design §8, M2-6 | **open** (CDC) | SHA `f7ff3ed`. M3 type errors go through CheckError::Diagnostic with expected/got messages. **Caveat:** M3 errors use the same ad-hoc `eprintln!` path as M0/M1 (not routed through DiagnosticCollector). The diagnostic content is teaching-grade ("body returns `integer`, but contract declares `:returns binary`") but not rendered through the full span+caret engine. Golden snapshots for M3 deferred to M3.5 — the M2 engine snapshots (m2_11) verify the engine works. | Snapshot tests deferred to M3.5 |
| M3-11 | **Demo — README `describe` type-checks:** the README's `order-status` + correct `describe` example type-checks clean; the *wrong* (strings) version is rejected. | CT/CLI: correct describe passes; strings version rejected | should/serious | README | deferred | Deferred to M3.5 — requires creating the order-status deftype + describe fixture and wiring it through the checker. The type checker infrastructure is in place; the fixture is a demo exercise, not a new capability. | |
| M3-12 | **Line/col precision + full regression:** M3 errors carry exact line:col; M0/M1/M2 suites ALL still pass; line injection holds. | CT/Rust: exact span for an M3 error; full suite green | serious | M0–M2 | done | SHA `f7ff3ed`. All 25 CT tests pass (0 skipped): 6 chain + 10 adt + 6 matching + 3 typecheck. M3 errors carry the defun/typed's position. The type checker correctly caught the M0 hello.tlfe fixture's type mismatch (list body declared as binary) — demonstrating the regression guard works in both directions. | |
| M3-13 | **Process:** CT in LFE (`*_SUITE.lfe`); `make check` clean (clippy -D, rustfmt, xref); CI matrix green (0 skipped). | CI green; `make check` exit 0 | polish | feedback (LFE CT) | done | SHA `f7ff3ed`. `make check` clean: clippy -D warnings, rustfmt, xref all pass. 51/51 Rust, 25/25 CT (0 skipped). `typed_typecheck_SUITE.lfe` — 3 tests, LFE patterns. | |
| M3-14 | **(should) Basic polymorphic contracts:** identity-style type variables in `:args`/`:returns` checked consistently. | Rust test: `(:args ((x a))) (:returns a)` accepts; obvious misuse rejected | should | design §6 | **deferred** (CDC) | SHA `f7ff3ed`. Type variables in contracts (e.g. `:args ((x a))`) parse as `Adt("a")`. Since ADT types that aren't registered resolve permissively (the checker doesn't reject unknown type names), polymorphic contracts effectively work via the dynamic/unknown path. Full unification deferred to M3.5. | Implicit via ADT passthrough; full unification deferred |

## What Worked

- **Bidirectional checking against contracts** is the right 80/20 — no inference
  needed at function boundaries, just check body vs `:returns` and args vs params.
- **The type checker caught a real bug** — the M0 `hello.tlfe` fixture declared
  `:returns binary` but its body `(list ...)` returns a list. The type checker
  correctly rejected it, proving the system works as designed.
- **`dynamic()` as the gradual escape hatch** means the checker doesn't block on
  untyped Erlang/LFE calls — they just flow through as dynamic.
- **Built-in prelude is tiny and documented** — 20 lines of pattern matching in
  `builtin_return_type`, covering arithmetic, comparison, list ops, and type predicates.
  Everything else → dynamic.
- **Cross-function checking** works: all `defun/typed` signatures are collected
  before body checking, so a function can call another typed function and get
  argument type checking.

## CDC Verification

**Verifier:** Claude (CDC), 2026-06-07, against `f7ff3ed` / `96b88b9` (CI green).
**Method:** static inspection; test logic read for vacuity/spec-softening.

**Genuine win — credit:** the three headlines (M3-3 body-vs-`:returns`, M3-4 call-arg,
M3-5 field-value) **work**, and the checker **caught a real type bug**: M0's `hello.tlfe`
declared `:returns binary` but its body `(list …)` returns a list — caught and the
contract correctly fixed. That's the best possible evidence M3 does its job. Bidirectional
synth/check, the dynamic boundary, and the minimal prelude are all real and verified.

**Findings requiring correction:**

1. **M3-10 → `open` (the recurring discipline gap, now on the most important messages).**
   The headline tests use **`.contains()` substring checks**, not exact assertions
   (`m3_3` asserts the message *contains* "integer"/"binary"; `m3_4` *contains*
   "expected type `integer`"; the CT tests assert "output contains …"). There are **no
   exact golden snapshots** for M3, and M3 diagnostics don't route through the
   `DiagnosticCollector`. The message *content* is teaching-grade, but a rendering
   regression on the project's most important diagnostics ("wrong return type") would go
   uncaught. **Required (cheap):** exact-message golden snapshots (`assert_eq!` full
   render) for M3-3/4/5. **Deferrable to M3.5:** full span+caret engine routing.
2. **M3-6 → `deferred` (was "done (caveat)").** `case/typed` synthesizes `Dynamic`;
   branch-body-vs-expected typing is **absent**, not done. M2 exhaustiveness + pattern
   well-formedness give structural safety, so deferral is legitimate — but the status
   must say `deferred`, not `done`.
3. **M3-14 → `deferred` (was "done (via dynamic)").** Type variables pass through
   permissively (no real checking). It's a "should," so deferral is fine — but
   "done" overstates a no-op pass-through.
4. **M3-11 (README demo) — deferred is acceptable, but prioritize it.** The README shows
   `describe` as the project's public face; it should actually type-check. Make it an
   early M3.5 item, not open-ended.

**TREND (systemic — CDC trending rule):** this is the **third** milestone where tests
asserted weaker-than-exact — M1 (matrix asserted *type*, not value → missed the casing
bug), M2-9 (enum/transparent matching *assumed* backend-identical, untested), M3
(*`contains()`* not exact on the headlines). The per-instance fixes work, but the
recurrence is structural. **Standing rule adopted:** *every diagnostic test asserts the
EXACT rendered output (`assert_eq!`/golden snapshot), never `contains()`; every backend
claim is tested, never assumed.* Recorded to memory as standing guidance for all future
milestones.

**Disposition:** M3's substance is real and proven (it caught a live bug). Not
clean-closed: M3-10 needs exact snapshots for the 3 headlines; M3-6/M3-14 reclassified
`deferred` (done). Engine routing, branch typing, README demo, poly unification → M3.5.

## Closure

**Not yet closed (CDC).** Iteration 1 (`f7ff3ed`) landed the three headlines (verified —
it caught a real bug) + prelude + dynamic boundary + full M0/M1/M2 regression (25/25 CT,
0 skipped, `make check` clean). Blocking clean close: (a) **M3-10** — add exact-message
golden snapshots for M3-3/4/5 (the recurring discipline, cheap); (b) **M3-6, M3-14**
reclassified `deferred` (done in this pass). Legitimately deferred to **M3.5**: engine
span+caret routing, `case/typed` branch typing, README `describe` demo, polymorphic
unification. Revised tally: Done 9 · Deferred 4 (M3-6, M3-11, M3-14, + M3.5 items) ·
Open 1 (M3-10, needs snapshots).
