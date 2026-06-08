# M4 Close-Out — Claude Code prompt (CC iteration 2)

> Paste into Claude Code from the `typed` project root. CDC accepted the M4/M4.5 split
> (pre-authorized) and verified the headline; one gap remains in the ADT head guard.
> Does NOT expand scope.

```
You are CC closing out Milestone M4 ("Runtime Enforcement"). ITERATION 2 (of 5). CDC
accepted the M4/M4.5 split (guards core in M4; validators/decode/web-demo → M4.5) and
verified the headline (wrong arg → structured type-error crash, not function_clause).
ONE gap remains. Read the ledger's "## CDC Verification" section first
(docs/design/milestones/M4-runtime-enforcement-ledger.md).

# STANDING RULES (NON-NEGOTIABLE — typed-test-discipline, cc-editing-safety)
- Diagnostic/error tests assert EXACT output (assert_eq!/pattern match), never `.contains()`.
- Test EVERY backend. Unwired ≠ done. Test the criterion's ACTUAL subject. No blind `sed`.

# Ledger discipline
- Iteration 2 of 5. Don't expand scope. Amendments need written justification. Every done
  row: SHA + reproduced output; CI green. Per-row walk at close. Leave CDC section intact.

# Required correction (M4-2)

The tagged-tuple head guard currently checks SHAPE only (`is_tuple`, or `is_tuple orelse
is_atom` for mixed) — so any tuple passes as a valid value of that ADT. The criterion
says **is_tuple + tag (+arity)**, and under the always-on max-safety posture the head
guard must actually reject wrong-tagged tuples. Tag/arity checks are guard-legal and cheap.

1. In `guards.rs` `guard_for_adt` (TaggedTuple, constructors WITH fields), generate a
   guard that accepts ONLY the ADT's real constructors, e.g. for `order-status` with
   `(Shipped (tracking ...))` / `(Cancelled (reason ...))` (snake_cased tags) plus nullary
   `(Pending)`:
     (orelse
       (andalso (is_tuple X) (=:= (element 1 X) 'shipped)  (=:= (tuple_size X) 2))
       (andalso (is_tuple X) (=:= (element 1 X) 'cancelled)(=:= (tuple_size X) 2))
       (=:= X 'pending))
   i.e. an `orelse` over each constructor: tuple ctors → `is_tuple AND element(1)=tag AND
   tuple_size=arity`; nullary ctors → `X =:= tag`. (Reuse to_snake_case for tags; reuse
   the existing enum-membership helper for the nullary atoms.)
2. Keep enum / all-nullary guards as they are (already tight membership checks).
3. native-record head guard stays deferred (OTP 29+) — note it.
4. DEEP field-type validation stays OUT (that's the M4.5 validator's job). M4-2 is just
   tag + arity at the head, which is O(1).

# Required test
- A CT (LFE) test: calling a typed fn whose arg is a tagged-tuple ADT with a WRONG-TAGGED
  tuple (e.g. `{bogus, 1}` where `order-status` is expected) RAISES the structured
  `{type_error, ...}` (NOT a silent pass, NOT function_clause). Exact assertion on the
  error fields. Also confirm a CORRECT constructor value still passes (no false-reject).

# Run & evidence
- cd checker && cargo build && cargo test; rebar3 ct; make check. Show Skipped=0.
- Commit; anchor M4-2 to the new SHA; confirm CI green; full M0–M3.5 regression green.

# Definition of a clean close
- M4-2: tagged-tuple head guard checks tag (+arity) per constructor; wrong-tagged tuple
  rejected (exact test); correct value still passes; enum/all-nullary unchanged.
- Per-row walk; CDC section intact.

(Optional, note-only: the structured error term is a proplist; standardizing on a map can
wait for the M4.5 render helper.)

Do NOT expand scope: no deep validators/decode/web-demo (M4.5), no native-record runtime,
no field-type validation at the head.
```
