# Milestone M1: ADTs & Representation

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (commit SHA + reproduced
> output) as work lands; CDC independently re-verifies. No row stays `open` at
> close. Required backends: `tagged-tuple`, `enum`. `native-record` runtime is
> expected `deferred` (OTP 29+). Don't regress M0 (M1-12).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M1-1 | `deftype` parses: `(deftype (result ok err) (Ok (value ok)) (Error (reason err)))` → ADT def (name, type params, constructors with **named fields + field types**). | Rust test: assert parsed ADT structure (params, ctor names, field names+types) | serious | design §4.1 | open | | Syntax provisional |
| M1-2 | Parsed ADTs populate the checker type environment; a `deftype` referencing another resolves. | Rust test: env lookup of a declared type + a cross-referencing type | correctness | design §6 | open | | Module-local only (no cross-module consume) |
| M1-3 | Construction form parses into an internal construction node (ctor + named field values). | Rust test: parse `(Ok :value 42)` → ctor=Ok, fields=[(value, 42)] | correctness | design §4.1 | open | | |
| M1-4 | **Constructor well-formedness check (structural):** unknown ctor, unknown field, missing field, wrong arity each yield a Tier-1 diagnostic with exact **line:col**. | Rust tests: 4 malformed fixtures, assert exact span + message per case | serious | design §3.2a, §7 | open | | Field-VALUE type checking is OUT (needs expr typing) |
| M1-5 | **Lowering — `tagged-tuple` (required, default <29):** `(Ok :value 42)` → flat `{'Ok', 42}` (snake_case tag). | CT (LFE): build + construct, assert runtime term `{'Ok',42}` (or `#('Ok' 42)`) | serious | Audit 2 §7 | open | | Flat (Gleam), not nested (Alpaca) |
| M1-6 | **Lowering — `enum` (required):** all-nullary sum → atoms. | CT (LFE): `(deftype colour (Red)(Green)(Blue))`; construct `Red`, assert `'red'` | correctness | Audit 2 §7 | open | | |
| M1-7 | **Lowering — `transparent` (should):** 1-ctor/1-field newtype → payload itself. | CT (LFE): construct `(CustomerId :v 7)`, assert runtime value `=:= 7` | correctness | Audit 3 §8 | open | | May defer with rationale if M1 runs long |
| M1-8 | **Lowering — `native-record` (code; runtime deferred):** `(Ok :value 42)` → native record `#Ok{value=42}` (true distinct type, `is_record` true). | Code present + guarded CT on OTP 29+ | correctness | Audit 1 §2.6, Audit 2 §3.5 | open | | Runtime row `deferred` on OTP 28; re-entry: 29+ toolchain |
| M1-9 | **`repr` selection + default:** per-type repr choice drives lowering; default resolves native-record on 29+, tagged-tuple on <29. | Rust/CT test: same ctor lowers differently under two reprs; default picks by OTP | serious | design §5 | open | | The pluggable seam |
| M1-10 | **Registry emission:** ADT defs emitted as a custom `.beam` module attribute (cross-module interface) + free Erlang `-type`. | CT: compile a deftype module; `beam_lib:chunks` shows the registry attr + `-type` | correctness | design §3.4 | open | | Emission only; consumption is M4+ |
| M1-11 | **Backend-matrix tests:** the SAME ADT surface program built + verified across `tagged-tuple` + `enum` (+`transparent` if done) on OTP 28; native-record axis present, runtime deferred. | CT matrix run green on testable backends; CI matrix updated | serious | design §9 | open | | The cross-backend equivalence guarantee |
| M1-12 | **Line-injection regression:** an ADT-form error and an ADT runtime crash still report the original source line (M0 F-8/F-9 not regressed). | CT: assert original line for an ADT error + an ADT crash | serious | M0 F-8/F-9 | open | | Guard the headline through new forms |
| M1-13 | **CT suites in LFE:** M1 tests are `test/*_SUITE.lfe` following the LFE project examples + in-repo `typed_chain_SUITE.lfe`. | The new suite is `.lfe` and runs (`Skipped = 0`) | polish | feedback (LFE CT) | open | | NOT Erlang `.erl` |

## What Worked

_(Filled in at close.)_

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in at close. Total rows: 13.)_
