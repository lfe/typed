# M1 — Claude Code implementation prompt

> Paste into Claude Code from the `typed` project root. Implements M1 (ADTs +
> representation) against the ledger, under ledger discipline. Builds on the
> closed M0 chain.

```
You are implementing Milestone M1 ("ADTs & Representation") of the `typed` project.
You are CC (implementer) under LEDGER DISCIPLINE. M0 is CLOSED — the model-Y chain
(read source → check → lower with original lines → lfe_codegen:module → compile:forms
→ BEAM) and line injection already work; build ON it, do not rebuild it.

# Read first (then STOP and confirm scope before coding)
1. docs/design/milestones/M1-adts.md            (scope; in/out; provisional syntax)
2. docs/design/milestones/M1-adts-ledger.md     (acceptance criteria M1-1..M1-13)
3. docs/design/01-design-v0.md §3.2a, §4.1, §5, §6   (tiers, ADT surface, repr, scope)
4. docs/design/audits/02-erlang-data-type-taxonomy.md §7   (carrier representations)
5. test/typed_chain_SUITE.lfe + lfe/test/example_SUITE.lfe  (LFE CT style to follow)

# Ledger discipline (in force)
- The ledger IS the spec (M1-1..M1-13). Work against it; don't silently drop/reshape
  rows. If a criterion is wrong/impossible, raise an amendment WITH justification.
- Fill Status + Evidence (commit SHA + reproduced command output) AS rows land.
- Iteration budget: 5. If M1 runs to iteration 4–5, STOP and propose splitting
  (e.g. carve extra backends into M1.5) rather than grinding.
- Closing report: per-row walk M1-1..M1-13, each with final status + evidence.
  Name uncertainty honestly. Leave the CDC Verification section for CDC.

# Scope (do exactly this — no more)
IN: deftype (named-field, parametric ctors); construction; structural constructor
well-formedness checking with line+col diagnostics; pluggable repr backends; registry
emission as .beam attribute; backend-matrix tests; line-injection regression guard.
OUT (do NOT build): pattern matching / deconstruction / exhaustiveness (M2);
field-VALUE type checking (needs expr typing — M1 checks structure only); function
contract checking (M3); dynamic()/untyped interop and registry CONSUMPTION (M4+);
derived Eq/Ord.

# What to build
1. typed-check (Rust): 
   - parse `(deftype (Name params...) [(repr <backend>)] (Ctor (field type)...) ...)`
     into an ADT def; populate the type environment; support parametric type vars.
   - parse the construction form (provisional `(Ctor :field expr ...)`).
   - CHECK structural well-formedness: unknown ctor, unknown field, missing field,
     wrong arity — each an exact line+col Tier-1 diagnostic (reuse M0's diagnostic path).
   - LOWER constructions per the chosen `repr`:
       * tagged-tuple (default <29): `(Ok :value 42)` -> flat `{'Ok',42}` (snake_case tag)
       * enum: all-nullary sum -> atoms
       * transparent: 1-ctor/1-field -> the payload itself
       * native-record (29+): `#Ctor{field=..}` true distinct type (code now; runtime test deferred)
   - resolve the default repr by OTP version (native-record on 29+, else tagged-tuple).
   - emit the ADT registry as a custom module attribute (cross-module interface) +
     a free Erlang `-type` breadcrumb; pass forms to the existing driver unchanged.
2. Erlang driver / provider: reuse M0's typed_driver + provider; extend only as needed
   to carry the new forms/attribute. Keep lfe_codegen:module + compile:forms.
3. Fixtures (.tlfe): a good result/option type; an all-nullary enum; a newtype; and
   4 malformed-construction fixtures (unknown ctor / unknown field / missing field /
   wrong arity) on KNOWN distinctive lines.
4. Tests — CT suites in **LFE** (`test/*_SUITE.lfe`), following test/typed_chain_SUITE.lfe
   and lfe/test/example_SUITE.lfe. Plus Rust unit tests for parsing/checking. The
   matrix tests build the SAME surface program under each testable backend and assert
   the runtime representation; native-record runtime rows are `deferred` (OTP 29+).
   Include a line-injection regression test for an ADT error + an ADT crash.
5. CI: extend the matrix axis to exercise tagged-tuple + enum (+ transparent if done);
   keep native-record axis stubbed/deferred (OTP 29+).

# Environment
OTP 28 / LFE 2.2.1. native-record runtime cannot be tested here — mark those rows
`deferred` with re-entry "when a 29+ toolchain is available". Record exact versions.

# Definition of done
M1-1..M1-13 each final with SHA + reproduced output (or justified deferred/no-op).
Required backends (tagged-tuple, enum) run-verified on OTP 28 with the matrix green;
transparent done-or-justified; native-record code present + runtime deferred. M0's
line injection still holds (M1-12). End with the per-row walk; CT shows Skipped=0 for
the rows you claim ran.
```
