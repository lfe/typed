# Milestone M10 — Naming Convention (`deftype/typed` + the `/typed` rule)

> **Goal:** make every typed macro follow one convention — **`<lfe-form>/typed`** — so
> the typed forms never shadow their LFE namesakes, and the surface reads consistently.
> Concretely: rename `deftype` → `deftype/typed`, and formalize the convention as policy.
> **Builds on:** M0–M9 (M9 reader landing first). **Iteration budget:** 5.

## Why

LFE already has `deftype` (for type specs). The typed checker currently claims the bare
name `deftype` for its ADT form — shadowing LFE's. The other four typed forms already
follow the convention (`defun/typed`, `defrecord/typed`, `case/typed` — each shadows a
real LFE form: `defun`, `defrecord`, `case`). Only `deftype` is out of line. Fixing it:

- **stops the shadow** — bare `deftype` is freed for LFE's own type-spec form, so a
  `.lfet` file could use both `deftype/typed` (our ADT) and `deftype` (LFE type spec);
- **makes the surface uniform** — every typed form reads as "the typed variant of an
  LFE form": `<lfe-form>/typed`.

Target surface (per Duncan):
```lisp
(deftype/typed (result ok err)
  (Ok    (value  ok))
  (Error (reason err)))

(deftype/typed (option a)
  (Some (value a))
  (None))

(deftype/typed order-status
  (Pending)
  (Shipped   (tracking string))
  (Cancelled (reason   string)))
```

## The convention (formalized policy)

A typed macro is named **`<lfe-form>/typed`** when it is the typed variant of an
existing LFE form:

| typed form | shadows LFE | status |
|------------|-------------|--------|
| `defun/typed` | `defun` | ✓ already |
| `defrecord/typed` | `defrecord` | ✓ already |
| `case/typed` | `case` | ✓ already |
| `deftype/typed` | `deftype` | **this milestone** |
| `import-types` | *(none)* | **documented exception** — no LFE form to shadow; self-documenting; stays as-is (Duncan, 2026-06-08) |

Future typed forms that shadow an LFE form MUST use `<lfe-form>/typed`. Typed-only
constructs with no LFE namesake (like `import-types`) are named for clarity.

## In scope

- **Parser:** recognize `deftype/typed` as the typed ADT form; **bare `deftype` is no
  longer the typed form** (it passes through as ordinary LFE — no longer extracted as an
  ADT). `(repr …)` option and all existing `deftype` structure carry over unchanged.
- **Cross-module scanner:** `is_deftype` → recognize `deftype/typed` (the scanner
  extracts ADTs from this form for the project registry).
- **Rename across the tree:** every `.lfet` fixture using `deftype` → `deftype/typed`;
  every CT/Rust test and **exact diagnostic snapshot** that embeds `deftype`; the README
  "taste" section and `docs/usage.md` examples; design-doc examples.
- **Document the convention** (the table above) in `docs/usage.md` (and/or a short design
  note), so the policy is explicit for future forms.
- **Negative check:** a bare `(deftype …)` in a `.lfet` file is NOT treated as a typed
  ADT (it's plain LFE passthrough) — assert the checker no longer picks it up.
- **Full M0–M9 regression**; standing discipline.

## Out of scope

- Renaming `import-types` (decided: kept — documented exception).
- Any new typed forms or capability — this is purely a naming/consistency change.
- Touching `defun/typed`/`defrecord/typed`/`case/typed` (already conformant).

## Definition of done

`deftype/typed` is the typed ADT form; bare `deftype` is no longer treated as typed
(passthrough); the cross-module scanner recognizes `deftype/typed`; all
fixtures/tests/snapshots/docs/README use `deftype/typed`; the convention is documented;
full M0–M9 regression green; `make check` clean.

## Standing discipline (in force)

[[typed-test-discipline]] (exact assertions; **test the actual subject** — `deftype/typed`
is recognized AND bare `deftype` is no longer picked up as typed, both asserted; unwired ≠
done; status honesty) · [[cc-editing-safety]] (**no blind `sed`** — `deftype` is a
substring hazard and LFE's own `deftype` may appear; rename deliberately, `git mv`,
`git checkout` to recover, numstat-check) · [[lfe-ct-tests-in-lfe]] (CT in LFE) ·
[[typed-forms-not-macros]] (these are checker surface forms lowered to LFE).
