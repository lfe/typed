# M5 Gap Inventory — Dogfood Findings

> Surfaced while writing and typing the `orders.tlfe` realistic module.
> Each item classified **fix-now / defer / wontfix** with a one-line rationale.

## Gaps Found

### 1. Validator/decode naming convention not obvious
**Classification:** defer (document)
**Finding:** Generated validator/decode functions are named `validate-<typename>` and
`decode-<typename>`. The user must know this convention and manually add them to
`(export ...)`. Initially wrote `decode-status` instead of `decode-order-status`.
**Fix:** Document the convention in `docs/usage.md`. Future: auto-export validators.

### 2. `div` already in prelude (false alarm)
**Classification:** wontfix (already works)
**Finding:** Initially thought `div` wasn't in the prelude, but it is (`"div"` in
`builtin_return_type` → Number). No gap here — the dogfood module uses it correctly.

### 3. No unused-variable warning in case/typed bodies
**Classification:** defer
**Finding:** `(Cancelled r)` in `is-complete` binds `r` but never uses it. No warning.
The M2 redundancy checker warns about duplicate/unreachable clauses but not unused
pattern bindings.
**Fix:** Would need the type checker to track which pattern bindings are used in the
body — a scope-analysis pass. Deferred to a later quality-of-life milestone.

### 4. No way to auto-export validator/decode functions
**Classification:** defer
**Finding:** Every `deftype` generates `validate-<type>/2` and `decode-<type>/1`, but
the user must manually add them to the module's `(export ...)`. This is error-prone.
**Fix:** The checker could automatically add these to the export list. Deferred —
the manual export works, just isn't ergonomic.

### 5. `++` synthesizes as `list`, not `string`
**Classification:** wontfix (correct by design)
**Finding:** `(++ "hello " name)` synthesizes as `list` type. Since LFE strings ARE
lists (`[char]`), this is correct — `string` and `list` are compatible via
`types_compatible`. No action needed.

### 6. No record-style ADT (named product without a sum)
**Classification:** defer
**Finding:** Wanted to define an `order` with `{id, status, items, total}` — a
product type, not a sum. Currently `deftype` requires at least one constructor, so
you'd write `(deftype order (Order (id integer) (status order-status) ...))` with a
single constructor. It works but feels heavy for what's really a record.
**Fix:** A `defrecord/typed` or single-constructor sugar. Deferred to a syntax
milestone.

### 7. No `let` type annotation
**Classification:** defer
**Finding:** `let`-bound variables get their types from synthesis. If the synth is
`dynamic`, the binding is `dynamic` — there's no way to annotate a `let` binding
with an explicit type.
**Fix:** `(let ((x : integer (some-call)))` or similar. Deferred to a later
syntax/type-system milestone.

### 8. Cross-module type references not yet supported
**Classification:** defer (M4+ scope)
**Finding:** Can't reference a type defined in another module. All types must be
in the same `.tlfe` file. The registry attribute is emitted to `.beam` but not
consumed across modules.
**Fix:** Registry consumption is explicitly M4+ scope. Deferred.

### 9. No `when` guards in case/typed patterns
**Classification:** defer
**Finding:** Can't write `((Processing id) (when (> id 0)) ...)` — the pattern
parser doesn't support guard expressions. Would need to extend the pattern grammar.
**Fix:** Deferred to a pattern-matching enhancement milestone.

### 10. Binary literals (`#"..."`) not lexed by the sexp reader
**Classification:** defer
**Finding:** The oxur-sexp lexer doesn't handle `#"..."` (LFE binary literals) or
`#(...)` (tuple literals). Files using these forms would fail to parse.
**Fix:** Extend the lexer for LFE-specific `#`-syntax. Deferred — the current
surface uses `"..."` strings and `(tuple ...)` forms.

## Summary

| # | Gap | Classification |
|---|-----|---------------|
| 1 | Validator naming convention | defer (document) |
| 2 | `div` already works | wontfix (false alarm) |
| 3 | No unused-var warning | defer |
| 4 | No auto-export validators | defer |
| 5 | `++` → list (not string) | wontfix (correct) |
| 6 | No record-style ADT sugar | defer |
| 7 | No `let` type annotation | defer |
| 8 | Cross-module type refs | defer (M4+) |
| 9 | No `when` guards in patterns | defer |
| 10 | Binary `#"..."` not lexed | defer |

**Fix-now count: 0.** Every gap is either correct-by-design (wontfix) or deferred
with rationale. The system handles the realistic orders module without any blocking
gaps — a good sign for the architecture.
