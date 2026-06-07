# Milestone M2: Pattern Matching, Exhaustiveness & the Diagnostic Engine

> Per LEDGER_DISCIPLINE.md. CC fills Status/Evidence (commit SHA + reproduced output,
> CI-green) as work lands; CDC re-verifies. No row stays `open` at close. Headline:
> **M2-3** (exhaustiveness rejection) + **M2-6** (the diagnostic engine). Assert
> **exact** diagnostics (the M1 lesson). Split to M2.5 if it runs to iter 4–5.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| M2-1 | `case/typed` parses: scrutinee + clauses with **constructor patterns** (binding fields), **nullary**, **wildcard `_`**, **variable** patterns, into an internal match node. | Rust test: assert parsed clauses/patterns/bindings from a fixture | serious | design §4.3 | open | | Top-level patterns only |
| M2-2 | **Scrutinee type resolution:** the scrutinee's ADT type is determined from a contract-typed `defun/typed` arg in scope (and/or explicit annotation); contract `:args` types are threaded into the body env. Unknown ⇒ clear diagnostic. | Rust test: scrutinee bound to a contract arg resolves to its ADT; unknown case yields the diagnostic | serious | design §6 | open | | Seeds the contract→body env (M3 grows it) |
| M2-3 | **EXHAUSTIVENESS (thesis):** clauses must cover every constructor of the scrutinee's sum (or a `_`/var catch-all); **non-exhaustive ⇒ REJECT**, naming EVERY missing constructor. | Rust + CT: a non-exhaustive fixture is rejected; the message lists all missing ctors (exact, snapshot) | serious | Audit 1 §6, Audit 3 §6 | open | | Top-level sum only |
| M2-4 | **Pattern well-formedness:** a pattern using a constructor not in the scrutinee's type, or wrong field/arity, yields a Tier-1 diagnostic with exact line:col. | Rust tests: 3 malformed-pattern fixtures, exact span + message each | serious | design §7 | open | | Deconstruction analog of M1-4 |
| M2-5 | **Field access via patterns:** field vars bound in a constructor pattern are usable in the clause body and carry the right values at runtime. | CT (LFE): match `(Shipped t)`, use `t` in body, assert runtime value | correctness | design §4.1 | open | | M1 deferred this to M2 |
| M2-6 | **Diagnostic engine (real + reusable):** span + caret underline, "not matched: …" list, actionable `Hint:`, alias-aware type names, **multi-error collection**. M0/M1 ad-hoc diagnostics refactored onto it. | Rust tests + snapshots; grep shows old call sites use the engine | serious | design §8, Audit 3 §7 | open | | Gleam is the bar |
| M2-7 | **Machine-readable diagnostics:** `--format json` (or equiv) emits structured diagnostics (code, span, severity, message, missing-ctors, hint) — same info as human form. | Rust/CLI test: JSON output parses + carries the fields for a non-exhaustive error | should | design §8 (LLM half) | open | | May defer to M2.5 with rationale |
| M2-8 | **Match lowering (repr-aware):** `case/typed` lowers to plain LFE `case` over the carrier — tagged-tuple `{ok,V}`, enum atom, transparent payload, native-record patterns (runtime deferred). Bindings preserved. | Rust + CT: lowered form correct; runtime match works on testable backends | serious | design §5 | open | | native-record runtime `deferred` (29+) |
| M2-9 | **Backend-matrix for matching:** the SAME `case/typed` program matches + runs correctly across tagged-tuple + enum + transparent; **EXACT** assertions. | CT matrix green (0 skipped); native-record axis deferred | serious | design §9 | open | | Exact reps, not just types (M1 lesson) |
| M2-10 | **Redundant/unreachable clause** warning (duplicate ctor; clause after a catch-all). | Rust/CT: redundant fixture yields a warning naming the dead clause | should | Audit 3 §6 | open | | Defer to M2.5 if budget tight |
| M2-11 | **Golden-snapshot diagnostic tests:** the EXACT rendered output (human + JSON) for the non-exhaustive case (and ≥1 pattern error) is snapshot-tested. | snapshot test files present + green; review the snapshot content | serious | M1 CDC lesson | open | | Assert EXACT output, not "an error" |
| M2-12 | **Line/col precision + regression:** exhaustiveness/pattern errors carry exact line:col (Tier-1); M0/M1 runtime line injection still holds through `case/typed`. | CT/Rust: assert exact span for an M2 error + an injected line for an ADT crash | serious | M0 F-8/F-9, M1-12 | open | | |
| M2-13 | **Process:** CT suites in LFE (`*_SUITE.lfe`); `make check` clean (clippy -D, rustfmt, xref); CI matrix green (0 skipped). | CI green run; `make check` exit 0 | polish | feedback (LFE CT) | open | | |

## What Worked

_(Filled in at close.)_

## CDC Verification

_(Filled in by CDC against the closing SHA.)_

## Closure

_(Filled in at close. Total rows: 13.)_
