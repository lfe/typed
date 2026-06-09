# Design Note 05 — LFE Macro Expander: Review, Oracle, and Rust-Port Plan

> **Purpose:** review LFE's macro expander (`lfe_macro.erl`) and assess (a) a standalone
> **oracle** that expands LFE forms (so we can diff against it) and (b) a faithful **Rust
> port** of the expander. **Motivation:** `typed` has now shipped two incomplete
> backquote reimplementations in Rust (lists-only, then typed-forms-only — see
> [[typed-project-goals]] M9). The recurring lesson ([[typed-forms-not-macros]], cf. M4.6)
> is that reimplementing a battle-tested LFE facility piecemeal invites corner-misses. A
> faithful, oracle-validated port would end that. Grounded in a full read of LFE 2.2.1
> `lfe_macro.erl` (1432 lines) + `lfe_env`, `lfe_eval`, `lfe_macro_include`,
> `lfe_macro_record`/`_struct`, `lfe_internal`, `lfe_io`.

## Verdict (the load-bearing fact)

**LFE macro expansion EVALUATES arbitrary user code at expand time.** A `defmacro` is
stored as a `match-lambda` and, when called, is run through the **full LFE evaluator**
(`exp_userdef_macro/4` → `lfe_eval:apply/3`, `lfe_macro.erl:826`). `eval-when-compile`
and `let-function`/`letrec-function` also evaluate at expand time. So a **100% pure-Rust
port is gated on reimplementing the entire LFE interpreter** (`lfe_eval.erl`, ~2000 lines)
*plus* a BEAM-semantics term/BIF model — the dominant cost and risk.

**But the expander cleanly splits into three tiers**, and the *majority* of it (the pure
tree-transform surface) is a tractable mechanical port. The right move is: **build the
Erlang oracle now, port the pure tier, and put a clearly-documented "delegate to the
oracle/BEAM" boundary around the eval-time tier** rather than reimplementing the BEAM.

## What the expander does (categories)

Public entry points: `expand_expr_all/2` (full recursive expansion of one expression —
what `lfe_eval` calls), `expand_form/4` and `expand_fileforms/3,4` (the compiler's
whole-file path). Two knobs: `deep` (full vs. shallow) and `keep` (retain vs. drop
macro/fn defs). Forms are LFE sexps as Erlang terms; FileForms are `{Form,Line}` pairs.

The dispatcher (`exp_form/3` cascade + `exp_macro/3` + the `exp_predef/3` table) handles:

- **Core-form structural recursion** — quote/function, all data constructors
  (cons/list/tuple/binary/map/tref/…), binding forms (let, let-function, letrec-function,
  let-macro), control (progn/if/case/cond/maybe/receive/catch/try/funcall/call),
  comprehensions (lc/bc), definitions (define-*). Pure tree walks.
- **Backquote/quasiquote** — `exp_backquote/2` (`:1349-1408`), R6RS-compatible: nested
  backquote, comma, comma-at, splicing into lists/tuples/maps, with cons/append
  optimizers. **Pure tree transform — the cleanest piece to port.**
- **Predefined macros** — `c*r` accessors, comparison aliases, `list*/let*/flet*/do/fun/?`,
  CL-style `defun/defmacro/flet/fletrec/macrolet/deftype/defspec` lowering, `MODULE/LINE`,
  colon-call sugar. Mostly static rewrites.
- **defrecord/defstruct** — generate whole families of accessor macros (pure name-mangling;
  `lfe_macro_record.erl`, `lfe_macro_struct.erl`).
- **User macros** — **evaluated** via `lfe_eval` (the hard part).
- **eval-when-compile / set** — **evaluated** at expand time.
- **include-file / include-lib** — filesystem reads + recursive expansion; `.lfe` is easy,
  but `.hrl` pulls in Erlang's `epp` preprocessor + `erl_parse` (`lfe_macro_include.erl`).
- **QLC, match-specs, imported macros** — `qlc` round-trips through vanilla Erlang AST;
  imported macros `code:load_file` a compiled BEAM module and call its
  `'LFE-EXPAND-EXPORTED-MACRO'/3` (runs compiled BEAM — impossible in pure Rust).

**Hygiene:** LFE macros are **unhygienic** (CL-style) — *no* renaming algorithm to
reproduce (good news). Only manual gensyms (`new_symb`/`new_fun_name`, counters in
`#mac{}`). But gensym name formats (`|-0-|`, `do$^0`) and counter threading **must be
byte-identical** for oracle diffing.

**State:** `#mac{}` (plain record — trivial to port) + `Env` (`lfe_env.erl`, 266 lines,
pure data — easy to port).

## The three tiers (port feasibility)

| Tier | Contents | Risk | Port path |
|------|----------|------|-----------|
| **1 — pure tree transforms** | core-form recursion, backquote, static `exp_predef` table, defrecord/defstruct, env, gensyms | **Low** | Mechanical Rust port; reuse oxur's sexp reader ([[oxur-sexp-reuse]]). ~1432 LFE lines → a few thousand Rust. Fiddly spots: exact gensym counters + backquote cons/append choices. |
| **2 — eval-time** | user macros, eval-when-compile, let/letrec-function | **High (dominant)** | Requires a full LFE interpreter in Rust (`lfe_eval` ~2000 lines) + BEAM term/BIF model. Multi-month. |
| **3 — foreign toolchain** | `.hrl` include (epp/erl_parse), QLC, imported macros (BEAM load), match-specs | **High (narrow)** | `.lfe` include easy; `.hrl`/QLC/imported-macros need Erlang's preprocessor/parser/BEAM — no faithful pure-Rust analog. |

## The oracle (build this first — it's trivial)

An **Erlang escript** that wraps the real `lfe_macro`, so it's faithful by construction.
Chain: `lfe_io:read_file/1` → `lfe_env:new/0` → `lfe_macro:expand_form_init/2` →
`lfe_macro:expand_form/4` (fold per form, threading `Env`+`#mac{}`) → `lfe_io:print1/1`.
~25 lines. Decisions for diff stability: pick `keep` true/false and match it in Rust;
**pin the printer** (`lfe_io:print1` formatting must match what Rust emits, or compare
structurally by re-reading both outputs); gensym counter behavior must align.

This gives a faithfulness ruler **before** writing any Rust, and a regression corpus.

## Recommended phased plan

0. **Phase 0 — Oracle + golden corpus.** Build the escript; assemble `.lfe` inputs with
   golden expanded outputs (start with macro-free / no-EWC / no-include programs).
1. **Phase 1 — Port Tier 1.** `exp_form` cascade, backquote, static `exp_predef`,
   defrecord/defstruct, env, gensyms. Gate everything through the oracle. This already
   covers a large fraction of real LFE structurally — **and immediately retires `typed`'s
   ad-hoc backquote/tuple reader code with a faithful, oracle-tested implementation.**
2. **Phase 2 — Decide the eval strategy (the fork in the road):**
   - **(a) Delegate (hybrid, recommended first):** when a user macro / EWC / `.hrl` /
     QLC / imported macro is hit, shell out to the Erlang oracle for that subtree. Ships
     fast; 100% faithful by delegation; not pure-Rust.
   - **(b) Embed BEAM:** call an Erlang node/NIF to run `lfe_eval` for macro bodies.
     Faithful; runtime dependency.
   - **(c) Reimplement `lfe_eval` in Rust:** the only fully self-contained path; treat as
     its own multi-month project (env + eval_expr + pattern/guard engine + binary/map/
     record/struct + curated BIF set), validated against an lfe-eval oracle, before wiring
     into the expander.
3. **Phase 3 — Includes & friends.** `.lfe` includes (easy); leave `.hrl`/QLC/imported
   macros behind the documented "delegates to Erlang" boundary unless full fidelity is a
   hard requirement.

## How this serves `typed`

- **Immediate:** Phase 1 replaces the fragile in-Rust backquote/tuple handling (two
  corner-misses already) with a faithful, oracle-validated transform — ending that class
  of bug and de-risking the M12 dirs port (dirs is quasiquote-heavy).
- **Position fidelity:** a Rust port keeps expansion in our control, preserving the M0
  line/position injection that Model Y depends on (vs. delegating to `lfe_macro`, which
  may not preserve our annotations) — this is *why* we expand in Rust at all.
- **Strategic:** combined with [[04-lfe-column-positions]], a faithful Rust expander +
  column-aware LFE is the foundation for best-in-class typed-LFE diagnostics.

## Honest assessment

The pure-transform port (Tier 1) is genuinely worth doing soon and is low-risk. The
**100% port is gated on Tier 2** — reimplementing the LFE interpreter and a slice of BEAM
semantics — which is a large, separable project; for that tier the pragmatic faithful path
is **delegation to the Erlang oracle**, not reimplementing the BEAM in Rust. Recommend:
build the oracle + corpus, port Tier 1 to retire the ad-hoc reader code, and explicitly
decide (a/b/c) for Tier 2 as its own milestone rather than letting it block Tier 1.

## M9.x interim series (DECIDED — Duncan, 2026-06-08): implement Tier 1 now

**Why now, not later:** the typed driver runs **no `lfe_macro`**, so *no* macro expansion
happens on passed-through forms. Backquote was the first symptom (M9 D-4); `cond`, `let*`,
`do`, `defrecord`, etc. in real LFE break identically. A faithful Tier-1 expander is the
prerequisite for the pipeline to compile real LFE — so it lands as an interim **M9.x**
series before M10 (naming). M10 and beyond wait.

- **M9.1 — Phase 0: oracle + corpus + harness.** Erlang escript wrapping the real
  `lfe_macro` (`read_file → expand_form → print1`); a golden corpus covering Tier-1
  categories; a diff harness; pinned conventions (`keep`, printer, gensym). No Rust port.
- **M9.2 — Phase 1a: backquote + core-form recursion.** Port `exp_backquote` + the
  `exp_form` structural recursion faithfully; **run expansion over ALL emitted forms**
  (replaces the ad-hoc `qq_expand`; fixes the plain-`defun` gap noted at M9 close);
  oracle-validated. The immediate win — retires the two fragile backquote attempts.
- **M9.3 — Phase 1b: static predef + records.** Port the static `exp_predef` table (`c*r`,
  comparison aliases, `list*`/`let*`/`flet*`/`do`/`fun`/`?`, CL `def*` lowering,
  `MODULE`/`LINE`, colon-call sugar) + `defrecord`/`defstruct` generation + gensym
  byte-fidelity; oracle-validated. After this the pipeline compiles real LFE using Tier-1
  macros.

Tier 2 (eval-time macros) and Tier 3 (includes/QLC/imported) remain explicitly out — to be
decided (delegate vs. embed vs. reimplement) as their own future milestone, not blocking
this series.
