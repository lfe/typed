# M6 Close-Out — Claude Code prompt (CC iteration 2)

> Paste into Claude Code from the `typed` project root. R-1..R-6, R-8, R-9 are CDC-verified
> and done. **R-7 is reopened** — the STATIC half of construction checking was never
> implemented (generated record functions' signatures aren't registered, so `make-` calls
> synth to `Dynamic` and aren't statically checked). Read the ledger's "## CDC Verification"
> section first.

```
You are CC closing out Milestone M6 ("Typed Records"). ITERATION 2 (of 5). CDC verified
R-1..R-6, R-8, R-9 are done. R-7 is OVERCLAIMED: its STATIC half is not implemented. Read
docs/design/milestones/M6-records-ledger.md "## CDC Verification" before starting.

# The root cause (not just a missing test — a missing wiring)
In main.rs, FunSigs are collected ONLY from defun/typed forms (all_fun_sigs). The GENERATED
record functions (make-<rec>, <rec>-<field>, set-<rec>-<field>) are emitted as output forms
but their signatures are NEVER registered. So in typecheck.rs synth_call, a call like
`(make-order "not-an-int" 'pending 1000)`:
  lookup_fun MISS -> builtin MISS -> lookup_record_accessor MISS -> falls to Type::Dynamic.
Result: construction args are NOT statically checked; make- synthesizes to Dynamic (not the
record type); an unknown-field accessor falls to Dynamic with no diagnostic. The wrong-field-
type error is caught only at RUNTIME (r2), and the unknown-field-accessor case (named in R-7)
is neither implemented nor tested.

# The fix
1. REGISTER generated record-function signatures into all_fun_sigs / body_env (alongside the
   defun/typed sigs in main.rs), so the normal static call-checking path applies:
   - make-<rec>: args = field types in declared order; returns = the RECORD type.
   - <rec>-<field> accessor: arg = the record type; returns = the field type. (You may keep
     lookup_record_accessor, but a real FunSig also lets the checker verify the arg is that
     record.)
   - set-<rec>-<field>: args = (record, field-type); returns = the record type.
2. UNKNOWN-FIELD ACCESSOR: a call shaped `<knownrecord>-<bogusfield>` on a known record type
   must produce a teaching diagnostic (not Dynamic). Pick the cleanest implementation
   (explicit check in synth_call, or driven by the registered sigs) and give an exact message.
3. TESTS (all EXACT, per standing rules):
   - STATIC wrong-field-type at construction: add a fixture with `(make-order "not-an-int"
     'pending 1000)` in a typed body; run the checker BINARY; assert NON-ZERO exit + the
     EXACT teaching diagnostic. (This is the Goal-2 headline — must be a static rejection,
     not a runtime catch.)
   - Rust: assert `(make-order 1 'pending 2)` synthesizes to the RECORD type (not Dynamic).
   - Unknown-field accessor: exact diagnostic (Rust snapshot and/or static CT).
   - KEEP the existing runtime tests (r2/r4) — runtime enforcement stays; you're ADDING the
     static half.

# STANDING RULES (NON-NEGOTIABLE)
- Exact assert_eq!/snapshots, never .contains()/is_list. TEST THE ACTUAL SUBJECT: the STATIC
  checker rejecting a wrong-typed make- (non-zero exit + exact diagnostic), and make-'s
  synthesized TYPE. Unwired ≠ done. Status honesty. No blind `sed`; `git checkout` to recover;
  `make check` after edits. CT in LFE.
- Generate per-type logic only; type-agnostic helpers stay hand-written.

# Ledger discipline
- Iteration 2 of 5. Don't expand scope. Per-row walk at close; leave the CDC section intact.
- Re-anchor R-7 to the new SHA; full M0–M5 + M6 regression green, 0 skipped; make check clean.

# Definition of a clean close
- make-/accessor/set- signatures registered; construction args statically checked; make-
  synthesizes to the record type.
- STATIC wrong-field-type-at-construction rejected (non-zero exit + EXACT diagnostic);
  unknown-field accessor gives an EXACT diagnostic; make- return type asserted exactly.
- Runtime tests retained; make check clean; CI green, 0 skipped.

Do NOT expand scope: no multi-field update sugar, no native-record runtime, no cross-module
use (M7), no new language features. Just wire the static construction-checking that R-7
requires + its exact tests.
```
