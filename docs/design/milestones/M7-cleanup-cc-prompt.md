# M7 Close-Out — Claude Code prompt (CC iteration 2)

> Paste into Claude Code from the `typed` project root. X-1, X-2, X-4, X-5, X-6, X-8, X-9 are
> CDC-verified; X-7 honestly deferred. **X-3 is reopened** — only its happy path is tested;
> the criterion's "wrong value rejected at the boundary" + "accessors/matching across the
> boundary" clauses are untested. Read the ledger's "## CDC Verification" first.

```
You are CC closing out Milestone M7 ("Cross-Module Type References"). ITERATION 2 (of 5). CDC
verified X-1/X-2/X-4/X-5/X-6/X-8/X-9 done and X-7 deferred. X-3 is OVERCLAIMED: only the happy
path is tested. Read docs/design/milestones/M7-cross-module-ledger.md "## CDC Verification".

# What's missing (X-3's untested clauses)
X-3's criterion = qualified `mod:type` resolves AND behaves like a local type
(matching/guards/validators/ACCESSORS across the boundary) AND **a wrong value is REJECTED at
the boundary**. Only `x3_qualified_cross_module` (the happy path) exists. The enforcement is
wired (qualified types are registered with full repr; guard_for_type resolves them), so it
LIKELY works — but it is unproven, and guards have been subtly shape-only before (M4-2:
is_tuple without the tag+arity check). Prove it.

# The fix (tests only — no new feature unless a test exposes a real hole)
1. X-3a BOUNDARY REJECTION (the key gap): a function in module B typed over a cross-module
   record/ADT (e.g. orders_web:get-order-total over orders:order) must REJECT a wrong value.
   CT: call it with (a) a NON-tuple (e.g. an integer) and (b) a WRONG-TAGGED tuple (e.g.
   `#(not_order 1 2 3)`) — assert each raises the structured type-error (the M4 guard fired),
   NOT a silent pass and NOT a generic crash. Exact on the error shape. (Per M4-2: confirm the
   guard checks the TAG, not just is_tuple — the wrong-tag case is what catches a shape-only
   guard.)
2. X-3b CROSS-MODULE ACCESSOR / MATCH: exercise the type as a real local type across the
   boundary — call a generated accessor on a cross-module record (e.g. `orders:order-total`
   from orders_web), OR a `case/typed` over a cross-module ADT (`orders:order-status`). CT:
   checks clean, compiles, runs, exact result. (Replace or augment the raw `(element 4 o)` in
   the dogfood with the real accessor so "behaves like a local type" is actually exercised.)
3. If either test exposes a real hole (guard not generated / wrong-tag accepted / accessor
   unresolved across modules), FIX it minimally, then the test goes green. If they pass as-is,
   great — the gap was test coverage.

# STANDING RULES (NON-NEGOTIABLE)
- Exact assert on the error shape / result, never .contains()/is_list (CT integration may use
  string:find on a checker diagnostic, but a RUNTIME rejection must match the structured
  type-error exactly). TEST THE ACTUAL SUBJECT: the boundary actually rejecting a wrong value,
  and a cross-module accessor actually resolving. Unwired ≠ done. Status honesty. No blind
  `sed`; `git checkout` to recover; `make check` after edits. CT in LFE.

# Ledger discipline
- Iteration 2 of 5. Don't expand scope (no provider UX — X-7 stays deferred; no recursive
  scan — that's M8). Per-row walk at close; leave the CDC section intact. Re-anchor X-3 to the
  new SHA; full M0–M6 + M7 regression green, 0 skipped; make check clean.

# Definition of a clean close
- X-3 boundary rejection tested: non-tuple AND wrong-tag both rejected with the exact
  structured type-error (M4-2 tag check confirmed across the boundary).
- A cross-module accessor or case/typed exercised (checks/compiles/runs, exact).
- make check clean; CI green, 0 skipped.

Do NOT expand scope: X-7 (provider UX) stays deferred; recursive/project-tree scan is M8;
no new syntax. Just prove (and if needed, fix) cross-module ENFORCEMENT + accessor/match.
```
