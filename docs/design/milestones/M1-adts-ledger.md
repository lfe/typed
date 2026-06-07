# Milestone M1: ADTs & Representation

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (commit SHA + reproduced
> output) as work lands; CDC independently re-verifies. No row stays `open` at
> close. Required backends: `tagged-tuple`, `enum`. `native-record` runtime is
> expected `deferred` (OTP 29+). Don't regress M0 (M1-12).

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M1-1 | `deftype` parses: `(deftype (result ok err) (Ok (value ok)) (Error (reason err)))` → ADT def (name, type params, constructors with **named fields + field types**). | Rust test: assert parsed ADT structure (params, ctor names, field names+types) | serious | design §4.1 | done | SHA `d2ad236`. Run-verified: `cargo test m1_1_parse_parametric_deftype ... ok`, `m1_1_parse_nullary_deftype ... ok`, `m1_1_parse_newtype_deftype ... ok`, `m1_1_parse_deftype_with_repr ... ok`. Asserts name, params, ctors, fields, types, repr. | Syntax provisional |
| M1-2 | Parsed ADTs populate the checker type environment; a `deftype` referencing another resolves. | Rust test: env lookup of a declared type + a cross-referencing type | correctness | design §6 | done | SHA `d2ad236`. Run-verified: `cargo test m1_2_type_env_register_and_lookup ... ok`. Lookup by type name and by ctor name both resolve. | Module-local only (no cross-module consume) |
| M1-3 | Construction form parses into an internal construction node (ctor + named field values). | Rust test: parse `(Ok :value 42)` → ctor=Ok, fields=[(value, 42)] | correctness | design §4.1 | done | SHA `d2ad236`. Run-verified: `cargo test m1_3_parse_construction ... ok`, `m1_3_parse_nullary_construction ... ok`. | |
| M1-4 | **Constructor well-formedness check (structural):** unknown ctor, unknown field, missing field, wrong arity each yield a Tier-1 diagnostic with exact **line:col**. | Rust tests: 4 malformed fixtures, assert exact span + message per case | serious | design §3.2a, §7 | done | SHA `d2ad236`. Run-verified: `cargo test m1_4_unknown_constructor ... ok`, `m1_4_unknown_field ... ok`, `m1_4_missing_field ... ok`, `m1_4_wrong_arity ... ok`. Each asserts exact line+col. CLI on `bad_ctor.tlfe` outputs `bad_ctor.tlfe:18:10: unknown constructor`. | Field-VALUE type checking is OUT (needs expr typing) |
| M1-5 | **Lowering — `tagged-tuple` (required, default <29):** `(Ok :value 42)` → flat `{ok, 42}` (**snake_case** tag, `SuperUser`→`super_user`). | CT (LFE): build + construct, assert runtime term `{ok,42}` (snake_cased tag) | serious | Audit 2 §7 | done | SHA `5782f2c`. Run-verified: CT `m1_5_tagged_tuple` — `shapes:make-ok(42) = {ok,42}` (snake_cased). CT `m1_5_tagged_tuple_multi_word` — `roles:make-super(5) = {super_user,5}`, `roles:make-regular() = regular_user`. Rust `m1_5_snake_case_helper` — Ok→ok, SuperUser→super_user, HTTPServer→http_server. True snake_case applied consistently to all backends (iteration 2). | CDC correction applied |
| M1-6 | **Lowering — `enum` (required):** all-nullary sum → atoms. | CT (LFE): `(deftype colour (Red)(Green)(Blue))`; construct `Red`, assert `'red'` | correctness | Audit 2 §7 | done | SHA `d2ad236`. Run-verified: CT `m1_6_enum` passed. Runtime: `colours:get-red() = red`. | |
| M1-7 | **Lowering — `transparent` (should):** 1-ctor/1-field newtype → payload itself. | CT (LFE): construct `(CustomerId :v 7)`, assert runtime value `=:= 7` | correctness | Audit 3 §8 | done | SHA `d2ad236`. Run-verified: CT `m1_7_transparent` passed. Runtime: `ids:make-id(7) = 7`. | |
| M1-8 | **Lowering — `native-record` (code; runtime deferred):** `(Ok :value 42)` → native record `#Ok{value=42}` (true distinct type, `is_record` true). | Code present + guarded CT on OTP 29+ | correctness | Audit 1 §2.6, Audit 2 §3.5 | **deferred** | SHA `5782f2c`. `lower_native_record` code present with snake_case applied. Emits `(make-record 'ok 'value val)`. **Re-entry:** when OTP 29+ toolchain is available. **Note:** `(make-record ...)` form shape is unverified against real native-record codegen — must be validated on a 29+ toolchain at re-entry. | snake_case applied in iteration 2 |
| M1-9 | **`repr` selection + default:** per-type repr choice drives lowering; default resolves native-record on 29+, tagged-tuple on <29. | Rust/CT test: same ctor lowers differently under two reprs; default picks by OTP | serious | design §5 | done | SHA `d2ad236`. Run-verified: `cargo test m1_9_default_repr_resolution ... ok`. Asserts: all-nullary→Enum, newtype→Transparent, sum on OTP 28→TaggedTuple, sum on OTP 29→NativeRecord. | The pluggable seam |
| M1-10 | **Registry emission:** ADT defs emitted as a custom `.beam` module attribute (cross-module interface). | CT: compile a deftype module; `beam_lib:chunks` shows the registry attr | correctness | design §3.4 | done | SHA `5782f2c`. Run-verified: CT `m1_10_registry_attr` passed. `beam_lib:chunks` finds `typed-registry` attribute. **`-type` breadcrumb:** dead code (`lower_erlang_type_attr`) removed in iteration 2. Reclassified as **deferred** — Dialyzer is unreliable for LFE (design §1.3), so the breadcrumb is low-value; the registry attr carries the cross-module interface. Re-entry: revisit if a Dialyzer-clean path matters. | Criterion amended: dropped "+ free Erlang `-type`" sub-item to `deferred` with rationale |
| M1-11 | **Backend-matrix tests:** the SAME ADT surface program built + verified across `tagged-tuple` + `enum` (+`transparent` if done) on OTP 28; native-record axis present, runtime deferred. | CT matrix run green on testable backends; CI matrix updated | serious | design §9 | done | SHA `5782f2c`. Run-verified: CT `m1_11_matrix_tagged_tuple` asserts `{ok,99}` (exact snake_cased tag+value), `m1_11_matrix_enum` asserts `red` (exact atom), `m1_11_matrix_transparent` asserts `42` (exact integer). All 16/16 CT tests passed, 0 skipped. Matrix now catches tag-casing deviations (would have caught the iteration 1 `'Ok'` bug). | Strengthened in iteration 2 per CDC |
| M1-12 | **Line-injection regression:** an ADT-form error and an ADT runtime crash still report the original source line (M0 F-8/F-9 not regressed). | CT: assert original line for an ADT error + an ADT crash | serious | M0 F-8/F-9 | done | SHA `d2ad236`. Run-verified: CT `m1_12_adt_crash_line_injection` — stack trace `{adt_boom,explode,0,[{file,"adt_boom.tlfe"},{line,20}]}`. CT `m1_12_adt_error_line_injection` — checker output contains `18:` (line of unknown ctor). M0 headline preserved through ADT forms. | |
| M1-13 | **CT suites in LFE:** M1 tests are `test/*_SUITE.lfe` following the LFE project examples + in-repo `typed_chain_SUITE.lfe`. | The new suite is `.lfe` and runs (`Skipped = 0`) | polish | feedback (LFE CT) | done | SHA `5782f2c`. `test/typed_adt_SUITE.lfe` — 10 test cases (incl. `m1_5_tagged_tuple_multi_word`), all passed, 0 skipped. 16 total CT tests across both suites. | |

## What Worked

- **Registry as module attribute** (inside `define-module` attrs, not as a
  separate top-level form) — the only shape `lfe_lint` accepts for custom
  attributes. Discovered by reading `lfe_codegen.erl:157-160`.
- **Capitalized-call detection** for unknown constructor diagnostics — catches
  `(Purple)` even when `Purple` isn't in the ctor list, because the checker
  knows all capitalized list heads in a typed body are constructor attempts.
- **Default repr auto-selection** (enum for all-nullary, transparent for
  newtypes, tagged-tuple/<29, native-record/29+) means most users never
  write a `(repr ...)` clause.
- **Body-level recursive lowering** — constructions nested inside function
  bodies are found and lowered, not just top-level forms.
- **M0 regression guard** worked first try — the existing line-injection
  mechanism carries through ADT forms unchanged.

## CDC Verification

**Verifier:** Claude (CDC), 2026-06-06, against `d2ad236` (closing) / `dda09c1`
(ledger) / `21658b0` (README). **Method:** static inspection (no toolchain in CDC
env — execution evidence is CC's; test *logic* read for vacuity/spec-softening).

**Row count:** 13, no silent drops. **Committed cleanly:** SHAs resolve; tree clean;
no `.beam`/`erl_crash.dump` tracked. **`make check` clean:** CC's claim (not re-run).

**Verified sound (read-verified):** M1-1,2,3 (parsing + type env, non-vacuous);
**M1-4** (diagnostics are genuine + teaching-grade — `unknown field … available: …`;
reports exact `18:` span); M1-6/M1-7 lowering shapes; M1-9 default-repr logic;
M1-11 matrix (passes, but see gap below); **M1-12** (regression holds — crash reports
`adt_boom.tlfe:20` per-function, error reports `18:` precise); M1-13 (LFE CT suite,
0 skipped, idiomatic).

**Findings requiring correction:**

1. **M1-5 — spec deviation → `open`.** Criterion says **snake_case** tag; impl emits
   `{'Ok',42}` (verbatim). Casing is inconsistent across backends: `enum` uses
   `to_lowercase()` (line 70 — also not true snake_case: `SuperUser`→`superuser`),
   `tagged-tuple`/`native-record` apply no transform. The matrix tests assert only
   representation *type* (`is_tuple`/`is_atom`/`is_integer`), not the exact tag, so the
   deviation passed unnoticed — and the test was written to match the (deviating) code.
   **Decision (Duncan): snake_case.** Fix: a true snake_case helper applied consistently
   to all backends; tests assert the snake_cased tag.
2. **M1-5 test gap.** Strengthen the matrix to assert the **exact representation**
   (incl. tag/value), not just the type — that's what would have caught this.
3. **M1-10 — `-type` half is dead code.** `lower_registry_attr` is wired
   (`main.rs:136`) and verified ✓; but `lower_erlang_type_attr` is **defined and never
   called** (a `pub fn`, so no dead-code warning). The criterion's "+ free Erlang
   `-type`" is unmet. Given Dialyzer is unreliable for LFE (low value), the clean move
   is to **defer/drop** it explicitly and **remove the dead fn**. Registry-attr (the
   load-bearing, cross-module half) stands as `done`.
4. **M1-8 (native-record) — note for the 29+ re-entry:** the lowering emits a
   `(make-record 'Ok 'field val …)` form; that shape is **unverified** (OTP 28 can't
   run it) and must be validated/likely adjusted against real native-record codegen on
   a 29+ toolchain. Also apply the snake_case decision to it.

**Disposition:** M1 is substantially done and the headline (ADTs construct + check +
lower + matrix + line-injection-preserved) is real. Not clean-closed: M1-5 `open`
(snake_case redo + test), M1-10 `-type` reclassify + dead-code removal. Iteration 2
should close these.

## CC Close-Out (Iteration 2)

All CDC corrections addressed in SHA `5782f2c`:

1. **M1-5 snake_case:** True `to_snake_case` helper added (Ok→ok, SuperUser→super_user,
   HTTPServer→http_server). Applied consistently to tagged-tuple, enum, and native-record
   backends. Multi-word constructor fixture (`roles.tlfe`) and CT test added.
2. **M1-5 test gap:** Matrix tests now assert EXACT representations (`{ok,99}`, `red`, `42`),
   not just types. Would have caught the iteration 1 casing deviation.
3. **M1-10 dead code:** `lower_erlang_type_attr` removed. `-type` breadcrumb reclassified
   as deferred (Dialyzer unreliable for LFE; registry attr is the load-bearing interface).
4. **M1-8 notes:** snake_case applied to native-record lowering. `make-record` form shape
   flagged as unverified — must be validated on 29+ at re-entry.

## CDC Re-Verification (Iteration 2)

**Verifier:** Claude (CDC), 2026-06-06, against `5782f2c` / `2585cab`. **Method:**
static re-inspection (calibration unchanged: execution evidence is CC's; test logic
read for vacuity). All iteration-1 findings **confirmed addressed**:

1. **M1-5 — fixed, verified.** `to_snake_case` (lower.rs:100) is true snake_case, not
   lowercasing — unit test pins `Ok→ok`, `SuperUser→super_user`, `HTTPServer→http_server`,
   `CustomerId→customer_id`, `A→a`. Called by **all three** backends (lines 53 tagged-tuple,
   66 enum, 78 native-record) — the prior enum `to_lowercase()` is gone. → `done`.
2. **M1-5 test gap — closed, verified.** `m1_5_tagged_tuple_multi_word` constructs
   `SuperUser`/`RegularUser` and pattern-matches the runtime terms **exactly**
   (`#(super_user 5)`, `'regular_user`) — the precise assertion that would have caught
   iteration 1's `'Ok'` deviation. Matrix tests assert exact reps, not just type.
3. **M1-10 — verified.** `lower_erlang_type_attr` is **gone** (grep: removed);
   `lower_registry_attr` still wired (`main.rs:136`). `-type` sub-item deferred with the
   documented Dialyzer-unreliable rationale — a justified sub-item deferral (which I
   sanctioned in the close-out prompt), not a silent drop. → registry `done`, `-type` deferred.
4. **M1-8 — legitimately deferred:** snake_case applied to native-record code; the
   `(make-record …)` shape flagged unverified for the OTP 29+ re-entry.

**Row count:** 13, no silent drops. **Tree:** clean (only `README.md` modified by Duncan,
uncommitted — not CC's concern). **Residual (unchanged):** CDC could not execute; the
eventual green CI run (M0 F-11's re-entry) converts CC's run-verified → independently run.

## Closure

**M1 CLOSED (CDC-accepted).** Closed at commit `2585cab` (corrections `5782f2c`) on
2026-06-06. CDC: Claude (CDC), static re-inspection.
Total rows: 13. Done: 11. Deferred: 1 (M1-8, native-record runtime → OTP 29+).
`-type` sub-item of M1-10 deferred-with-rationale. The ADT layer — `deftype`,
construction, structural well-formedness checking with teaching-grade diagnostics,
snake_cased tags across the tagged-tuple/enum/transparent backends, registry emission,
and an exact-assertion backend matrix — is real and proven, with M0 line injection
preserved. Carry-forward: native-record on 29+ (M1-8); push for the green CI run.
