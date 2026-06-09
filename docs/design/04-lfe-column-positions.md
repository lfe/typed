# Design Note 04 — Line+Column Positions in LFE: Analysis & Upstream Plan

> **Purpose:** assess what it would take for **LFE itself** to report line **+ column**
> positions on errors and warnings (today it is line-only), and — if feasible — propose
> a branch + development plan for a PR to LFE's `develop`. Grounded in a full read of
> LFE 2.2.1 source (`lfe_scan`, `lfe_parse`, `lfe_io`, `lfe_lib`, `lfe_lint`,
> `lfe_codegen`/`lfe_translate`, `lfe_comp`, `lfe_error`) cross-referenced against OTP
> `erl_anno`. **Relevance to `typed`:** this is the upstream half of our Model-Y bet —
> if LFE gains column positions, our checker can lean on them instead of re-reading
> source for spans ([[lfe-positions-and-metadata]]).

## Verdict

**Feasible, and well worth proposing.** The single most encouraging finding: **LFE's
scanner already computes columns** — it threads a `Col` accumulator through every scan
function — but **discards it at token-construction time** (every token is built with line
only). There is even dormant, commented-out scaffolding (`location/2`, `incr_column/2`,
the `scan_error/6` column args) showing the author anticipated this. Downstream, Erlang's
`erl_anno` has supported `{Line,Column}` locations since OTP 18, and LFE's codegen output
feeds straight into `compile:forms`, so the **Erlang side already accepts columns with no
change.** The bottleneck is entirely the LFE-internal "position = bare integer line"
convention.

There are **two tiers of ambition**, and the proposal hinges on choosing between them.

## Current state (how positions flow today)

1. **Scanner (`lfe_scan.erl`)** — computes `Col` per char (resets on newline) but builds
   every token as `{Type,Line}` or `{Type,Line,Value}`; the column is dropped. Token
   construction sites (`make_symbol_token/2`, string/binary/`#(`/`#\` builders) use line
   only. Inconsistency: punctuation tokens wrap line in `erl_anno:new/1` (`anno/1`),
   but numbers/symbols/strings use a **bare integer** — this must be unified first.
2. **Parser (`lfe_parse.erl`)** — `line(T) = element(2,T)`; records **one start line per
   top-level form**. The **sexp AST carries no position** — forms are raw lists/atoms/
   tuples with nowhere to hang an annotation.
3. **`lfe_io:parse_file`** — emits `{Sexpr, Line}` pairs. *This is the load-bearing
   convention:* one line per top-level form, stored outside the form.
4. **`lfe_lib:proc_forms`** — threads `L` opaquely to each form handler.
5. **`lfe_lint`** — errors/warnings are `{Line, Module, Term}`; the **same form-line `L`
   is threaded down the entire expression tree**, so a deep error reports the *form's*
   start line, not the offending node's.
6. **`lfe_translate`/`lfe_codegen`** — places that one integer `L` directly into every
   generated Erlang AST node's position field (which *is* an `erl_anno`).
7. **`lfe_comp`** — formats `~s:~w:` → `file:line:`. `forms/1` (no file) fabricates fake
   lines from the form index.
8. **Runtime (`lfe_error`, BEAM)** — exception locations are line-only because BEAM line
   tables don't store columns. **Columns are inherently a compile-time-only feature.**

## The two tiers

**Tier 1 — Form-start columns, end-to-end (Medium effort, high value/effort ratio).**
Carry an `erl_anno` location (`{Line,Col}`) from the scanner through `{Form,Anno}` pairs,
into the Erlang AST position fields, and out through a `file:L:C:` formatter. This
immediately gives column precision for: (a) **all scanner errors** (unterminated strings,
bad tokens — which already have exact columns, just discarded), and (b) **all errors the
Erlang compiler detects** in `compile:forms` (it reports columns natively once the anno
carries them). Lint's own errors stay form-granular (start of the offending form).

**Tier 2 — Per-subexpression columns (Large, architectural).** Point columns at the
*exact* offending node. Requires a **position-bearing sexp representation** — today sexps
are raw terms with no slot for an anno. Options: a side table keyed by node identity
(less invasive), or wrapping nodes (breaks every `is_list`/pattern-match in the macro
expander, lint, codegen — very invasive). This is the only way lint can say "the column
of *this* argument," but it is a substantial redesign.

## Recommended proposal (for LFE `develop`)

Adopt **`erl_anno` as the carried position type** (LFE already imports it), gated by a
`column` compiler option **mirroring OTP's own `erl_scan`/`compile` `column` option**.
With the option off, `erl_anno:new(Line)` of a bare line is byte-identical to today's
integer behavior — **zero observable change** — so the PR is compatibility-preserving by
construction.

**Phase 1 (the PR's core — Tier 1):**
1. `lfe_scan`: pass the already-computed `Col` into every token constructor; unify on
   `anno/1`; wire up the dormant `scan_error` column args. (Linchpin; ~25 sites, data
   already in hand.)
2. `lfe_parse`: `line/1` → location-aware accessor; record form-start location.
3. `lfe_io`: `{Sexpr, Anno}` pairs (document the shape change).
4. `lfe_lib`: no change (already opaque).
5. `lfe_lint`: type-widen `L` in `add_error/3`/`add_warning/3` (logic unchanged).
6. `lfe_codegen`/`lfe_translate`: pass the anno into AST position fields (no structural
   change — the field already accepts anno; **Erlang-side column errors then come free**).
7. `lfe_comp`: format `file:L[:C]:`; tolerate column-bearing `fix_erl_errors` output;
   construct annos where it currently fabricates integer lines (`forms/1` index lines,
   `FILE` macro).

**Phase 2 (optional, separable PR — Tier 2):** a position-bearing sexp representation
(side-table approach) so lint points at the offending subexpression. Propose only if the
LFE maintainers want it; Phase 1 already covers scanner + Erlang-compiler diagnostics,
which are the bulk of real errors.

## Compatibility / API-break surface (call out explicitly in the PR)

- `lfe_scan` token element 2 changes type (integer → anno). Affects raw-token consumers
  (rebar3_lfe, editor integrations).
- `lfe_io:parse_file/1,2` documented return `{Sexpr,Line}` → `{Sexpr,Anno}`. Wide blast
  radius — **including our own `typed` checker**, which consumes these.
- `lfe_parse:sexpr/form` `{ok,L,...}` `L` type widens.
- Error/warning tuple `{Line,Mod,Err}` → `{Location,Mod,Err}`; tools scraping `file:line:`
  must accept `file:line:col:`.
- **Unaffected:** all `format_error/1` clauses (they format the error *term*, not the
  position) — ~150 clauses need no change.

Mitigation: the `column` option (default matching OTP's current default) keeps the
line-only path available; opaque `erl_anno` of a bare line preserves existing behavior.

## Development plan (branch → PR)

1. **Branch** `feature/column-positions` off `develop`.
2. **Spike** the scanner: emit `{Line,Col}` annos behind the `column` opt; prove a
   scanner error reports a column. (De-risks the linchpin.)
3. Thread annos parser → `{Form,Anno}` → lint → codegen; update the `lfe_comp` formatter.
4. Add tests: scanner-error columns; an Erlang-compiler-detected error reporting a column;
   `column`-off byte-compat with current output (golden).
5. Audit the line-*construction* sites the research flagged for follow-up: `lfe_macro`
   (synthesizes lines for generated code), `lfe_macro_include` (include line bookkeeping),
   `lfe_comp` `forms/1` index lines, the `FILE` macro — each must construct annos.
6. Write the compatibility note; coordinate with maintainers on the `{Form,Anno}` shape
   change (the most consequential public contract).

## Honest uncertainties

- `lfe_macro.erl` and `lfe_macro_include.erl` were not exhaustively audited for
  line-*construction* sites; they must be swept (they synthesize positions for generated
  and included code). See [[05-lfe-expander-port]] for the expander's structure.
- Whether Tier 2 (per-node columns) is worth its cost is a maintainer call; Phase 1
  delivers the high-value subset (scanner + Erlang-compiler diagnostics) at Medium effort.
- Exact OTP-version behavior of the `column` default may need pinning per supported OTP.

## Bottom line for `typed`

Phase 1 alone would let our checker trust LFE's own column-bearing annos for the classes
of error the Erlang compiler surfaces — reducing (not eliminating) our need to re-read
source for spans. It also materially strengthens the case that typed-LFE diagnostics and
LFE diagnostics can converge. This is a credible, well-scoped upstream contribution that
also advances our roadmap's "further out" item (per-expression source mapping with LFE).
