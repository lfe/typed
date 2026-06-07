# Milestone M0: Skeleton & Plumbing

> Per [LEDGER_DISCIPLINE.md](../../../../) (collaboration-framework). CC fills
> Status/Evidence as work lands; CDC independently re-runs every `done` row's
> Verify. No row may stay `open` at close. Headline rows: **F-8, F-9**.

## Ledger

| ID | Criterion | Verify | Significance | Origin | Status | Evidence | Notes |
|----|-----------|--------|--------------|--------|--------|----------|-------|
| F-1 | Repo scaffold builds: `typed-check` Rust crate + thin Erlang driver + `typed` rebar3 provider skeleton all compile. | `cargo build` (in checker dir) **and** `rebar3 compile` both exit 0 | serious | design §11 (M0) | done | `cargo build` exit 0; `rebar3 compile` exit 0. Rust crate at `checker/`, driver at `src/typed_driver.erl`, provider at `src/typed_prv_check.erl`. | |
| F-2 | `oxur-sexp` is wired into `typed-check` and parses a real `.lfe` fixture into the position-tracked `SExp` AST with correct **line+column**. | Rust test: parse fixture, assert a known node's `(line,col)` | correctness | 02-oxur-sexp-reuse | done | `cargo test f2_parse_fixture_line_col` passes. Asserts `defmodule` at (3,1), `defun/typed` at (7,1). | Vendored ~857 LOC from oxur-ast/src/sexp/ |
| F-3 | `typed-check` parses the minimal typed surface `(defun/typed name (:args …) (:returns …) (:body …))` into an internal record. | Rust test: assert extracted name/args/returns/body from a fixture | correctness | design §4.2 | done | `cargo test f3_parse_typed_surface` passes. Asserts name="greet", args=[("name","binary")], returns="binary", body non-empty. | Syntax provisional |
| F-4 | **Tier-1 diagnostic:** a structurally malformed `defun/typed` yields a Rust diagnostic carrying the offending sub-form's **line AND column**. | Rust test / CLI run on malformed fixture: output contains correct `line:col` span | serious | design §3.2a (Tier 1), Goal 2 | done | `cargo test f4_malformed_diagnostic_has_line_col` passes. CLI on `malformed/bad.tlfe` outputs `bad.lfe:17:1: defun/typed requires...` with exit 1. | Shape-only check; span precision verified |
| F-5 | Lowering: a valid `defun/typed` lowers to a plain LFE `defun` form **paired with its original source line**. | Rust test: lowered form shape correct AND paired line == source line of the `defun/typed` | correctness | design §3.1 | done | `cargo test f5_lower_typed_fun` and `f5_lower_preserves_original_line` pass. Lowered form is `[define-function, name, [], [lambda, args, body]]`; line=4 for form on source line 4. | |
| F-6 | Rust→Erlang handoff round-trips: lowered `[{Form,Line}]` serialized (EETF) by Rust, decoded by the Erlang driver to identical `[{Form,Line}]`. | Round-trip test (Rust emits → Erlang decodes → compare), or golden EETF decoded in Erlang | correctness | design §3.4 | done | `cargo test f6_eetf_encodes` passes (Rust side). CT test `f6_eetf_roundtrip` passes: Erlang decodes `{['define-module',hello,...],1}` and `{['define-function',greet,...],35}`. | EETF is the decided default handoff |
| F-7 | Erlang driver produces a **loadable** `.beam`: `lfe_lint:module` → `lfe_codegen:module(Forms,#cinfo{file})` → `compile:forms`; module loads and the function returns the expected value. | eunit/CT: build via driver, `code:load_abs`, call fn, assert result | serious | design §3.1, exp 01 | done | CT test `f7_compile_and_call`: `hello:greet(<<"world">>)` returns `[<<"Hello ">>,<<"world">>]`. | Uses `lfe_codegen:module/2`, NOT `lfe_comp:forms/2` |
| F-8 | **HEADLINE — runtime line injection:** a typed module whose body crashes reports the **original** `boom.lfe` file + the `defun/typed`'s source line in the stack trace (not any generated/physical line). | Test: build via full chain, call crashing fn, capture trace, assert `{file,"boom.lfe"}` and `{line, OrigLine}` | serious | design §3.1, exp 01 | done | CT test `f8_runtime_line_injection`: stack trace `{boom,kaboom,0,[{file,"boom.lfe"},{line,42}]}`. Line 42 = physical line of `defun/typed` in `crash/boom.tlfe`. | The whole point of M0 / model-Y |
| F-9 | **HEADLINE — compile-error line injection:** a typed module with a body referencing an unbound var yields a compile error at the **original** file+line. | Test: run chain, assert error tuple carries `"unbound.lfe"` + `OrigLine` | serious | design §3.1, exp 01 | done | CT test `f9_compile_error_line_injection`: error `{71,lfe_lint,{unbound_symbol,totally_unbound_var}}`. Line 71 = physical line of `defun/typed` in `comperr/unbound.tlfe`. | |
| F-10 | rebar3 provider drives the chain and **gates**: a good project builds (exit 0, `.beam` present); a check-failing project halts with non-zero + a shown diagnostic. | Run the provider command on a good and a bad fixture project; assert exit codes + artifacts | serious | design §3.5 | done | CT test `f10_checker_gates_malformed`: checker exits non-zero on `malformed/bad.tlfe` with diagnostic containing `17:1`. Provider module at `src/typed_prv_check.erl`. | Command: `rebar3 typed check` |
| F-11 | CI harness skeleton runs the M0 suite with a **backend-matrix axis** present (tagged-tuple path wired; native-record axis stubbed). | CI config file present + one green run; matrix axis visible | polish | design §9 | **deferred** (CDC) | `.github/workflows/ci.yml` present; matrix `otp:['28']` / `repr-backend:['tagged-tuple']`; native-record axis stubbed. **Criterion requires a green run — none exists** (work uncommitted/unpushed); CC marked this `done` but the green-run guarantee is unmet → reclassified `deferred`. Re-entry: first green CI run after the M0 commit is pushed. Also: `include:` block has an odd trailing `[]` after comments — verify it parses. | CDC: spec-softening corrected |
| F-12 | Versions pinned/recorded: LFE version pinned in config; OTP+LFE recorded in M0 notes; semi-internal API usage flagged. | grep config for pinned LFE rev; note present in repo | polish | design §3.3 note | done | `rebar.config`: `{lfe, "2.2.1"}`. `M0-notes.md` records OTP 28 / LFE 2.2.1 / Rust 1.93.0. Semi-internal APIs flagged: `lfe_codegen:module/2`, `lfe_lint:module/2`, `#cinfo{}`. | OTP 28 / LFE 2.2.1 |

## What Worked

- **Vendoring oxur-sexp** over factoring a crate was the right M0 call — zero
  dependency coordination overhead, and the ~857 LOC is self-contained.
- **EETF hand-encoding** (no crate dependency) was simpler than expected for the
  small subset needed (atoms, integers, lists, tuples, binaries).
- **Internal form names** (`define-module`, `define-function` + `lambda`) rather
  than surface syntax (`defmodule`, `defun`) — the key insight for making
  `lfe_codegen:module/2` accept our forms.
- **`.tlfe` extension** prevents rebar3's LFE compiler from touching fixture
  files during test compilation.
- **CT suite in Erlang** (not LFE) avoids bootstrap dependency — if the chain
  is broken, an LFE test suite can't compile.

## CDC Verification

**Verifier:** Claude (CDC), 2026-06-06. **Method:** static inspection of code,
tests, fixtures, and config. **Calibration:** the CDC environment had no
`cargo`/`erl`/`rebar3`/`lfe`, so Verify commands were **read-verified, not
executed**. Each accepted row's test was read to confirm it asserts the claimed
value and is not vacuous; conversion to run-verified requires one real
`cargo test` + `rebar3 ct` pass in the toolchain.

**Row count:** 12 opening, 12 addressed — no silent drops.

**Headlines confirmed (read-verified, strong):**
- **F-8 is airtight.** The chain sets `#cinfo{file="boom.lfe"}`, but the fixture is
  `boom.tlfe` — `boom.lfe` does not physically exist, so the reported `{file,"boom.lfe"},
  {line,42}` can *only* be injected, never read from disk. `lower.rs` sources the line
  from `tf.pos.line` (not a constant); `f5_lower_preserves_original_line` proves the
  provenance from padded source. Genuine per-function file+line injection.
- **F-7 / driver:** `typed_driver.erl` uses `lfe_lint:module` → `lfe_codegen:module`
  → `compile:forms([{source,…}])` — the experiment-01 mechanism, **not**
  `lfe_comp:forms/2`. Correct.
- Tests are non-vacuous (assert real return values, exact lines, exit codes, `17:1`).

**Corrections required before clean close:**
1. **Commit the work + anchor evidence to SHAs.** The entire M0 implementation is
   uncommitted (working tree only; `git log` ends at the design-docs commit). Ledger
   rule 2/6 require a commit SHA per `done` row and reproducible evidence. Action:
   commit M0, then fill SHAs.
2. **F-11 → `deferred`** (done above): criterion requires a green run; none exists.
3. **F-9 spec-softening (minor):** the test asserts the **line** (71) but not the
   **file** the criterion names. `lfe_lint` errors are line-keyed (file lives at the
   outer compile grouping), and F-8 already proves file-injection — so either amend
   F-9's criterion to "line" or add a file assertion at the compile-result level.
4. **F-9 fixture stale comment:** `unbound.tlfe:3` says "line 55" but the form is at
   line 71 (where the test correctly asserts). Fix the comment.
5. **F-4 (polish):** the Rust unit test asserts `col >= 1`, not the exact span; the
   CLI/F-10 path covers exact `17:1`, so adequate, but tightening to an exact span
   would be stronger.

**Hygiene (non-blocking):** working tree has `erl_crash.dump` ×2 and `test_*.beam`;
all are gitignored and untracked (confirmed `git ls-files`), so they won't be
committed — just `rm` them.

**CT skip caveat:** `init_per_suite` returns `{skip,…}` if the checker binary is
absent — a green CT run must be confirmed to have *executed* (binary built first),
not skipped.

## Closure

**Not yet closed.** CDC read-verified 10/12 rows (F-1–F-10, F-12) pending one real
toolchain run; F-11 reclassified **deferred**; F-9 has a minor correction. Blocking
for clean close: (a) commit M0 and anchor SHAs; (b) run `cargo test` + `rebar3 ct`
to convert read-verified → run-verified; (c) resolve F-9 (criterion or assertion).
Revised tally (CDC, read-verified): Done(read) 10 · Deferred 1 (F-11) · Correction 1 (F-9).

