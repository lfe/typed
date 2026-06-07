# Experiment 01 — Can we control the *reported* source line through LFE → BEAM?

> **For:** the `typed` project (statically typed LFE). Decides one architecture
> fork: can a code generator stamp **original-source line numbers** onto generated
> LFE forms such that downstream **compile errors** and **runtime stack traces**
> report *those* lines (Elixir-style), or must we instead keep a side-car source
> map and intercept/translate downstream errors ourselves?
> **Deliverable:** a definitive, evidence-backed answer + a minimal repro.
> **Run with:** Claude Code, in a real Erlang/OTP + LFE environment.

---

## The Claude Code prompt (copy/paste)

```
You are running a rigorous, empirical compiler experiment. Do real work — run
commands, paste real outputs, cite source by file:line. Do NOT speculate where you
can test; report negative results honestly; distinguish "verified by running" from
"the source suggests."

# Goal
Determine DEFINITIVELY whether LFE lets us control the source LINE that is reported
for a form all the way through to the BEAM — specifically whether the *reported*
line can be made to DIFFER from the *physical* line in the file. We are building a
statically-typed LFE layer with a code generator; we want generated/lowered LFE to
carry line annotations pointing at the *original* (typed) source, so that LFE-
compiler errors, erlc errors, and runtime stack traces all map back to the user's
original code (the way Elixir stamps Elixir source lines into the Erlang it emits).
Also confirm the granularity ceiling (line only, or line+column?).

# Environment setup (record exact versions)
- Print: `erl -eval 'io:format("~s~n",[erlang:system_info(otp_release)]), halt().' -noshell`
  and the full `erl +V`.
- Ensure LFE is available. If not installed, clone https://github.com/lfe/lfe and
  build it (`make` / rebar3), or use a local checkout if one is provided. Record the
  LFE version/commit.
- Work in a fresh scratch directory; keep every test file; list them at the end.

# PART A — Source inspection (answer with file:line + short excerpts)
A1. In `lfe_codegen.erl`, find EXACTLY how the line/annotation on each emitted
    Erlang abstract form (`{function,Anno,...}`, `{attribute,Anno,...}`,
    clause/expr annos) is determined. Does the line come from: (i) the `{Form,Line}`
    fileform pair, (ii) an annotation stored inside the form, or (iii) `erl_anno`
    carried on tokens? Show the code path.
A2. Does LFE use `erl_anno`? Is LINE the only component end-to-end (i.e., no column
    survives)? Confirm or refute.
A3. Enumerate every surface/API that could let us SET or influence a form's line:
    e.g. a `(line ...)` form, a `-file`/`(file ...)` attribute that emits an Erlang
    `-file` directive, options to `lfe_comp`, or a forms-based compile entry
    (`lfe_comp:forms/2` or similar) that accepts `{Form,Line}` pairs. List what
    actually exists (with file:line), and what does NOT.

# PART B — Baseline ground truth (establish what's normal)
B1. Write a tiny LFE module whose function crashes at runtime (e.g. `(error 'boom)`
    or a failing match) at a KNOWN physical line. Compile (via `lfec` and/or
    `lfe_comp:file/2`), run, capture the stack trace. Record: is a line reported?
    WHICH line (function clause? the crashing sub-expression?)? Establish the
    granularity.
B2. Induce a COMPILE error (e.g. unbound variable) at a known physical line; record
    the reported line + granularity.
B3. Extract the generated Erlang abstract code from the `.beam`
    (`beam_lib:chunks("M.beam",[abstract_code])`, and try `[debug_info]`); paste the
    line annotations on the functions/clauses. Confirm they match physical lines.

# PART C — The decoupling test (the crux: reported line != physical line)
Try each mechanism that PART A surfaced as plausible; for each, make the intended
line deliberately impossible to confuse with a physical position (e.g. 9000+):
C1. Forms-based compile: if an API accepts `{Form,Line}` (or annotated forms),
    build forms whose Line = 9000+, compile to `.beam`, then (i) inspect
    abstract_code line annos and (ii) induce a runtime crash and check whether the
    stack trace reports ~9000. Document the exact call.
C2. `erl_anno` path: construct forms with `erl_anno:new(9000)` (or set_line) and run
    them through codegen; check propagation to abstract_code and to a stack trace.
C3. `-file` directive path: see if LFE can emit an Erlang `-file("orig.lfe", N)`
    attribute (or honors a `(file ...)`/`-file` form). If so, test whether erlc and
    stack traces shift reported lines accordingly.
C4. Macro path (control/sanity): write a macro whose expansion yields a form on a
    different physical line than its call site; report which line the expanded form
    is reported at (expected: the enclosing call-site form line). This documents
    expansion's line behavior.

A mechanism only COUNTS as success if the *reported* line (in a real stack trace
AND/OR a real compile diagnostic) follows our chosen annotation, NOT the physical
file position. Verify end-to-end, not just in abstract_code.

# PART D — Verdict & report
1. DEFINITIVE answer: can we set the reported line independent of physical position,
   at form granularity, through to (a) runtime stack traces and (b) compile errors?
   Via which exact mechanism/API? (It's fine if (a) works but (b) doesn't, or vice
   versa — report precisely.)
2. Confirm the granularity ceiling: line-only, no column, downstream? 
3. Gotchas: per-function vs per-clause vs per-expression; requires `-file`; only
   works via a particular API; differs across OTP versions; etc.
4. A MINIMAL reproducible example for the working mechanism (or, if none works, the
   closest attempt and exactly what's missing).
5. Recommendation: is "inject original-source lines into generated LFE" VIABLE, or
   must we fall back to a side-car source map + our own downstream-error translation?

# Output format
A short report organized by A/B/C/D, with real command outputs and stack traces
pasted in, source claims cited file:line, exact OTP+LFE versions at the top, and a
one-paragraph bottom-line verdict. List all scratch files created.
```

---

## Why these specific tests (rationale, for our discussion of the results)

- **PART A** tells us the *mechanism* and whether a settable line even exists in the
  pipeline; it also nails the **column ceiling** from source (we expect line-only).
- **PART B** is the control: what LFE reports *normally*, and at what granularity
  (we expect per-form/clause line, no column).
- **PART C** is the actual question — the Elixir trick. If any mechanism makes the
  reported line follow our annotation rather than the physical line, the "inject
  original lines" path is **viable** and downstream errors map back for nearly free.
  If none does, we fall back to a **side-car map + error interception** (still doable
  since model Y owns the chain, just more work).
- **PART C4** (macro path) doubles as confirmation of the expansion-loses-positions
  finding from our earlier dig — useful cross-check.

## How the result picks our branch *today*

- **If C succeeds (reported line follows annotation):** adopt model Y (own the
  chain) + line-injection; source-mapping downstream is cheap; schedule it as the
  early "Robert prototype" milestone.
- **If C fails for runtime but works for compile (or vice versa):** mixed strategy —
  injection where it works, side-car map where it doesn't.
- **If C fails entirely:** model Y + a side-car source map that the chain driver
  applies by translating `lfec`/`erlc`/trace output. Still viable; more code.
- **Either way:** column-precise diagnostics remain exclusive to our own type-error
  layer (confirmed by A2/B), so that win is unaffected.
