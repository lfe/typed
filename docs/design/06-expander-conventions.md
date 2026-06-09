# Expander Conventions (binds the Rust port)

> Pinned by M9.1. These choices are the contract the Rust expander (M9.2/M9.3)
> must match. Changing them requires re-generating all goldens and re-validating.

## Expansion knobs

- **`deep = true`** — fully expand all macros recursively (the compiler mode).
- **`keep = true`** — retain `define-function`, `define-macro`, `define-record`,
  etc. in the output (so the expanded forms can be inspected and diffed). The
  typed pipeline needs these definitions to survive for lowering/codegen.

## Printer / diff strategy

- **Canonical printer: `lfe_io:prettyprint1/1`** — stable, readable output used
  by the oracle. Goldens are committed as `prettyprint1` text.
- **Diff strategy: text comparison** — the Rust port's output will be compared
  as printed text against the committed goldens. This means the Rust port must
  produce output that, when printed by `lfe_io:prettyprint1/1`, matches the
  golden byte-for-byte. (In practice: the Rust port emits LFE terms as EETF,
  which are re-read by an Erlang harness and printed via `prettyprint1` for
  comparison.)
- **Alternative (structural compare):** if text matching proves too fragile
  (whitespace sensitivity), switch to term comparison: re-read both the golden
  and the Rust output as Erlang terms and `==` them. Document if/when this
  switch happens.

## Gensym fidelity

LFE's macro expander generates fresh names via `#mac.vc` (variable counter)
and `#mac.fc` (function counter). The name formats:

- **Variable gensyms:** `|-N-|` where N is the `vc` counter (e.g. `|-0-|`,
  `|-1-|`, `|-2-|`). Used by `new_symb/2` in `lfe_macro.erl`.
- **Function gensyms:** `do$^N` where N is the `fc` counter. Used by
  `new_fun_name/2` (for `do` loop expansion).
- **Counter threading:** `vc` and `fc` start at 0 in the initial `#mac{}` and
  increment per use, threaded through the expansion. The Rust port must
  reproduce the exact counter values in the same order, so generated names
  match byte-for-byte.

## Core-form names (surface → internal)

The expander lowering produces these internal forms:

| Surface | Internal |
|---------|----------|
| `defmodule` | `(progn (define-module ...) (define-macro MODULE ...))` |
| `defun` | `(define-function name () (lambda/match-lambda ...))` |
| `defmacro` | `(define-macro name () (match-lambda ...))` |
| `defrecord` | `(progn (define-record ...) (define-macro make-R ...) ...)` |
| `let*` | nested `let` |
| `cond` | nested `if` |
| `do` | letrec-function with gensym'd loop name |
| `` `(a ,b) `` | `(list 'a b)` (or `cons`/`++` for splice) |
| `` `#(a ,b) `` | `(tuple 'a b)` |
| `(mod:fun args)` | `(call 'mod 'fun args)` |
| `(MODULE)` | module name atom via define-macro |

## What the Rust port does NOT expand (Tier 2/3)

- User-defined macros (`defmacro` bodies evaluated by `lfe_eval`)
- `eval-when-compile` / `set` forms
- `.hrl` includes (Erlang preprocessor)
- QLC, imported macros, match-specs

These are behind the "delegate to the oracle/BEAM" boundary.
