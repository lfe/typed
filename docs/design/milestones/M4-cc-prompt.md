# M4 — Claude Code implementation prompt (Runtime Enforcement)

> Paste into Claude Code from the `typed` project root. Implements M4 (runtime
> enforcement) against the ledger, under ledger discipline. Builds on closed M0–M3.5.

```
You are implementing Milestone M4 ("Runtime Enforcement") of the `typed` project. You
are CC (implementer) under LEDGER DISCIPLINE. M0–M3.5 are CLOSED (chain, ADTs, repr
backends, matching/exhaustiveness, bidirectional contract checking, diagnostic engine).
This milestone makes types real at RUNTIME. POSTURE (decided): ALWAYS-ON guards
everywhere — every typed function head enforces its arg types at runtime.

# Read first (then STOP and confirm scope)
1. docs/design/milestones/M4-runtime-enforcement.md           (model, posture, two mechanisms, scope)
2. docs/design/milestones/M4-runtime-enforcement-ledger.md    (criteria M4-1..M4-13)
3. checker/src/{lower.rs,match_lower.rs,typecheck.rs,adt.rs,type_env.rs,diagnostic.rs,main.rs}
4. test/typed_*_SUITE.lfe (LFE CT style)

# STANDING RULES (NON-NEGOTIABLE — memory: typed-test-discipline, cc-editing-safety)
- Diagnostic/error tests assert EXACT output (assert_eq!/snapshot), never `.contains()`.
- Test EVERY backend (never assume "backend-identical").
- Unwired / cfg-test-only code is `deferred`, NOT `done`.
- A test/fixture must exercise the criterion's ACTUAL subject, not a stand-in.
- NEVER edit source with blind `sed`/in-place regex; if a file breaks, `git checkout` and
  redo carefully; run `make check` after bulk edits.

# Ledger discipline
- Work against M4-1..M4-13; amendments need written justification. Fill Status + Evidence
  (SHA + reproduced output; CI green) as rows land. Per-row walk at close. Leave CDC
  section for CDC. Budget 5; if it tightens, PROPOSE SPLIT: M4 = guards + structured error
  + matrix; M4.5 = deep validators + decode + web-demo + elision.

# Two mechanisms, two behaviors (key design)
- HEAD GUARDS (shallow, on every typed fn head): on violation → CRASH (raise a structured
  type-error). Internal contract enforcement, let-it-crash.
- VALIDATORS / decode (deep, recursive, at the untyped membrane): on violation → RETURN
  `#(error type_error)`. Untrusted external input handled gracefully (e.g. a 400).

# What to build
1. HEAD GUARDS (M4-1/M4-2): extend lowering so each `defun/typed` arg gets a native guard:
   integer→is_integer, float→is_float, binary→is_binary, atom→is_atom, boolean→is_boolean,
   string/list→is_list, map→is_map; ADT carriers: tagged-tuple→is_tuple + element(1) tag
   (+arity), enum→is_atom + membership, transparent→underlying-type guard, native-record→
   is_record (29+, runtime row deferred). Add a fallback clause that raises the structured
   type-error (M4-3). (Reuse to_snake_case for tags.)
2. STRUCTURED TYPE-ERROR (M4-4): a term like `#(type_error #{expected => T, got => G,
   function => F, arg => N, path => P})` + a human-render helper (Gleam-style). Exact
   snapshot the term AND the rendered string.
3. DEEP VALIDATORS (M4-5): generate `(validate <type> term) -> #(ok term)|#(error te)`,
   recursive over ADT fields + nested types, for tagged-tuple/enum/transparent.
4. decode (M4-6): `(decode <type> untyped) -> #(ok T)|#(error te)` = validate at the
   boundary; graceful, NO crash.
5. WEB-INPUT DEMO (M4-7): a fixture that decodes an untyped term (e.g. a map/tuple
   resembling parsed input) into an ADT — valid → `#(ok …)`, invalid → `#(error te)` with
   a teaching message. This is the motivating example; make it real.
6. GUARDS-vs-VALIDATORS (M4-8): show the same bad value CRASHES via a head call but RETURNS
   `#(error …)` via decode. Document + test both.
7. MATRIX (M4-9): guards + validators across tagged-tuple/enum/transparent, exact; native-
   record runtime `deferred` (29+).
8. REGRESSION (M4-10): all M0–M3.5 suites pass WITH guards on; README example still works;
   line injection holds. (Watch: adding head guards changes every typed fn's lowering —
   make sure existing fixtures still compile + run.)
9. M4-11 perf note (document elision is deferred); M4-12 discipline; M4-13 make check + CI.

# Run & evidence
- cd checker && cargo build && cargo test; rebar3 ct; make check. Show Skipped=0.
- Commit (small, logical commits); anchor every done row's Evidence to the SHA; CI green.

# Definition of done
M4-1..M4-13 final with SHA + CI-green evidence (or justified deferred/no-op). A wrong-typed
arg raises a structured, snapshot-tested type error across testable backends (M4-3); decode
turns untyped input into `#(ok T)`/graceful `#(error te)` (M4-6); the web-input demo works
both ways (M4-7); full M0–M3.5 regression green WITH guards on (M4-10). Per-row walk at close.

Do NOT expand scope: no guard elision, no native-record runtime, no message/process
enforcement, no framework helpers, no disable-guards knob.
```
