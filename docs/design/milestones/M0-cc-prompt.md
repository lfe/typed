# M0 — Claude Code implementation prompt

> Paste the block below into Claude Code, run from the `typed` project root.
> It implements M0 against the ledger, under ledger discipline.

```
You are implementing Milestone M0 ("Skeleton & Plumbing") of the `typed` project —
a statically typed LFE with ADTs. You are CC (implementer) under LEDGER DISCIPLINE.

# Read first (in this order), then STOP and confirm your understanding before coding
1. docs/design/milestones/M0-skeleton.md          (what M0 proves; in/out of scope)
2. docs/design/milestones/M0-skeleton-ledger.md   (the 12 acceptance criteria F-1..F-12)
3. docs/design/01-design-v0.md  §3                (model-Y architecture)
4. docs/design/02-oxur-sexp-reuse.md              (the reader we reuse)
5. docs/design/experiments/01-lfe-line-anno-probe.md  (the EXACT line-injection mechanism — reuse it)

# Ledger discipline (non-negotiable)
- The ledger IS the spec. Work against it, not around it. Do not silently drop or
  reshape a row — if a criterion is wrong/impossible, raise an amendment request.
- Fill Status + Evidence in the ledger AS each row lands (commit SHA + the actual
  output of that row's Verify command). Do not batch evidence to the end.
- In your closing report, walk the ledger ROW BY ROW (F-1..F-12), each with final
  status and evidence. Do NOT write a prose summary or "deviations: none."
- Name uncertainty: "done with caveat X" beats a confident "done" that's softpedalled.
- Iteration budget: 5. If you hit iteration 5 without convergence, STOP and report;
  do not iterate a sixth time.

# Scope reminder (M0 is PLUMBING — resist building more)
- NO real type checking, NO inference, NO ADTs, NO case/typed, NO repr backends,
  NO cross-module. The only typed form is `defun/typed` with :args/:returns/:body,
  and the "check" is shape-only. The HEADLINE is end-to-end line injection (F-8/F-9).

# What to build (the vertical slice)
1. `typed-check` (Rust crate): 
   - depend on the oxur `sexp` reader (prefer a factored `oxur-sexp` crate; vendoring
     the ~857-LOC sexp/ module is acceptable for M0). Extend its lexer ONLY as far as
     the M0 fixtures need.
   - parse a `.lfe` file (column-aware), recognize `(defun/typed name (:args ((Name Type)...)) (:returns Type) (:body Expr...))`,
   - do a shape-only check; on malformed input emit a diagnostic with line+COLUMN,
   - on success, LOWER to a plain LFE `(defun name (Args) Body)` form and pair it with
     the ORIGINAL source line of the `defun/typed`,
   - emit `[{plain-lfe-form, orig-line}, ...]` to stdout/file as EETF (Rust side:
     produce Erlang External Term Format; pick a crate or hand-encode the small subset).
2. Thin Erlang driver (in src/, Erlang or LFE): decode the EETF `[{Form,Line}]`, then
   replicate experiment 01 EXACTLY:
     lfe_lint:module(Forms, ...) ,
     {ok,_Mod,AST,_Ws} = lfe_codegen:module(Forms, #cinfo{file=OrigFile, opts=[debug_info], ipath=["."]}),
     {ok,_,Bin,_} = compile:forms(AST, [{source,OrigFile}, binary, debug_info, return]),
     write OrigBase ++ ".beam".
   (Include lfe_comp.hrl for #cinfo. Use lfe_codegen:module/2 — NOT lfe_comp:forms/2.)
3. rebar3 provider (src/, namespace lfe or typed): a `-behaviour(provider)` module that
   invokes the Rust binary (as a port / os:cmd) then the driver; non-zero/halt on check
   failure, proceed on success. Register via rebar_state:add_provider in the plugin init.
4. Fixtures: 
   - good/hello.lfe         — one defun/typed that returns a value (F-7)
   - crash/boom.lfe         — defun/typed whose body crashes (error/1 or bad match) (F-8)
   - comperr/unbound.lfe    — defun/typed body with an unbound var (F-9)
   - malformed/bad.lfe      — structurally malformed defun/typed (F-4)
   Put the defun/typed forms on KNOWN, distinctive line numbers so injection is provable.
5. Tests (Rust unit + Erlang eunit/CT) per the Verify columns. F-8/F-9 must assert the
   ORIGINAL line/file appears in the trace/compile-error — that is the milestone.
6. CI skeleton (.github/workflows or equivalent) running the suite, with a backend-matrix
   axis present (tagged-tuple wired; native-record axis stubbed/deferred — note OTP 29+).
7. Pin LFE version in rebar.config; record OTP+LFE versions in a short M0 notes file.

# Environment
OTP 28 / LFE 2.2.1 (record exact `erl +V` and lfe version). LFE checkout is available
locally; reference experiment 01 for the working API calls and includes.

# Definition of done
F-1..F-12 each reach a final ledger status with reproducible evidence; F-8 and F-9 are
`done` with a captured stack trace / compile-error showing the ORIGINAL source file+line.
Keep all fixtures and tests. End with the per-row ledger walk.
```
