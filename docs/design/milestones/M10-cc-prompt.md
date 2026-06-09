# M10 — Claude Code implementation prompt (Naming Convention: `deftype/typed`)

> Paste into Claude Code from the `typed` project root. Rename `deftype` → `deftype/typed`
> and formalize the `<lfe-form>/typed` convention. Builds on closed M0–M9. Pure
> naming/consistency change — no new capability.

```
You are implementing Milestone M10 ("Naming Convention") of the `typed` project. You are CC
(implementer) under LEDGER DISCIPLINE. M0–M9 are CLOSED. Rename the typed ADT form
`deftype` -> `deftype/typed`, and document the `<lfe-form>/typed` convention. Decisions
(Duncan, 2026-06-08): `import-types` is KEPT (documented exception — no LFE form to shadow);
the other forms (defun/typed, defrecord/typed, case/typed) already conform.

# Read first (then STOP and confirm scope)
1. docs/design/milestones/M10-naming-convention.md        (rationale + the convention table)
2. docs/design/milestones/M10-naming-convention-ledger.md (criteria N-1..N-6)
3. checker/src/adt.rs (extract_deftype), checker/src/main.rs (where deftype is recognized),
   checker/src/cross_module.rs (is_deftype / scan), checker/src/typed_surface.rs
4. all test/fixtures/**/*.lfet using `deftype`; the README "taste" section; docs/usage.md

# WHY (so the negative test matters)
LFE already has `deftype` (type specs). Our typed ADT form must stop shadowing it. After this:
`deftype/typed` = our ADT form; bare `deftype` = plain LFE (NOT extracted as a typed ADT).

# STANDING RULES (NON-NEGOTIABLE)
- NO BLIND `sed`. `deftype` is a SUBSTRING hazard (it appears inside `deftype/typed` and may
  appear as LFE's own deftype in comments/examples). Rename deliberately; use `git mv` for
  file renames; `git diff --numstat` to confirm; `git checkout` to recover. Exact
  assert_eq!/snapshots — the form name appears in diagnostics, update exactly. TEST THE ACTUAL
  SUBJECT: both that `deftype/typed` IS recognized and that bare `deftype` is NOT (passthrough).
  Unwired ≠ done. Status honesty. CT in LFE.

# What to do (each row gets a test)
1. N-1 RECOGNIZE `deftype/typed`: parser/extractor treats `(deftype/typed …)` as the typed ADT
   form, same structure + `(repr …)` as before. Rust: `(deftype/typed (result ok err) (Ok
   (value ok)) (Error (reason err)))` -> AdtDef with correct params/ctors/fields. Cover a
   header form `(result ok err)`, a nullary ctor, and the `(repr …)` option.
2. N-2 BARE `deftype` NOT TYPED: a `(deftype …)` form is no longer extracted as a typed ADT
   (no registry entry, no guards). Rust/CT assert it is NOT picked up. (This is the
   stop-the-shadow guarantee — test it explicitly.)
3. N-3 SCANNER: cross_module.rs recognizes `deftype/typed` (rename is_deftype etc.). CT: a
   cross-module type declared with `deftype/typed` resolves in another module (re-run the M7
   cross-module path on the new form).
4. N-4 RENAME: update every test/fixtures/**/*.lfet using `deftype` -> `deftype/typed`; every
   CT/Rust test; every exact diagnostic SNAPSHOT embedding the form name. Full suite green;
   grep shows no stale typed-`deftype` (bare `deftype` only where intentionally testing
   passthrough). 
5. N-5 DOCS + CONVENTION: README "taste" section + docs/usage.md examples -> `deftype/typed`;
   write the convention (the `<lfe-form>/typed` table, with `import-types` as the documented
   exception) into docs/usage.md.
6. N-6 REGRESSION: full M0–M9 green; make check clean; CI green, 0 skipped.

# Ledger discipline
- Work N-1..N-6. Budget 5 iterations. Per-row walk at close; leave CDC section for CDC. Anchor
  done rows to the SHA; CI green.

# Definition of done
`deftype/typed` recognized (N-1); bare `deftype` NOT treated as typed (N-2); scanner updated
(N-3); all fixtures/tests/snapshots/docs/README renamed (N-4); convention documented (N-5);
full regression green (N-6). Per-row walk at close.

Do NOT expand scope: do NOT rename import-types; do NOT touch defun/typed/defrecord/typed/
case/typed (already conformant); no new forms or capability. Just deftype -> deftype/typed +
document the convention.
```
