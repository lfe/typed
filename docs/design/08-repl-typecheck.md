# Design Note 08 — Type-Checking in the REPL

> **Status:** decision-seeking draft, in the style of [03-capability-unlock.md](03-capability-unlock.md) —
> mechanism recommendations are the assistant's; genuine UX/surface choices are Duncan's.
> **Provenance:** the typed-REPL exploration session (2026-06-09/10), commissioned from the
> "Typed LFE" book-chapter design session, run evidence-first per its bootstrap
> (`workbench/typed-repl-exploration-bootstrap.md`). Debated to convergence with Duncan
> before drafting. **Companion thread:** the xrepl Rust-core architecture review
> (`xrepl/xrepl/workbench/2026.06.10-rust-core-architecture-review.md`) — this note and
> that one were written against each other; §5 records where this note pushed back.
> **The named hazard:** both sessions flagged the same enthusiasm pull (a Rust REPL is
> the most thrilling option). Discipline applied: the boring options were costed first,
> and the recommendation below cites evidence that could have changed it.

---

## 1. The question

`typed`'s checking is a batch build step; Lispers live in the REPL; the LFE Machine
Manual's house style is REPL-transcript-heavy. Today the stock LFE REPL cannot evaluate
typed forms at all (typed surface forms are checker syntax, not LFE macros —
[[typed-forms-not-macros]]). Does typed LFE get an interactive story, and how?

## 2. Evidence gathered (all verified this session)

### 2.1 LFE shell internals (LFE 2.2.1 source)

- REPL `defun` → `lfe_eval:add_dynamic_func` into the shell environment
  (`lfe_shell.erl:492-501`). Interactive definitions are **interpreted, never
  compiled** — there is no compile step to gate. Checking interactive forms means
  checking forms the build pipeline never sees.
- The shell macro-expands every input form through the real expander —
  `lfe_macro:expand_fileforms` (`lfe_shell.erl:453`, deep=false, keep=true). **The
  Tier-2 boundary does not exist BEAM-side**: a BEAM-resident integration gets
  user-macro expansion natively. This asymmetry shapes the recommendation.
- Shell commands (`c/1`, `h/1`, …) are env-bound lambdas installed at state creation
  (`add_shell_functions`, `lfe_shell.erl:322`). No plugin API, but a wrapper can extend
  the base env cleanly; `rebar3_lfe`'s repl provider (ours) is a thin launcher with
  full latitude.

### 2.2 The cold-start benchmark (the measurement that reordered the option space)

`typed-check` release build, dogfood module (`test/fixtures/dogfood/orders.lfet`,
includes the sibling project scan), hyperfine `-N`, 550 runs, 2026-06-09:

```
mean 5.3 ms ± 1.2 ms   (min 4.4 ms; user+sys ≈ 3 ms — ~half is fork/exec overhead)
```

**10× under the 50 ms threshold** the bootstrap set. Consequences:

- "Re-run the batch checker per definition" is interactive-grade.
- A persistent checker daemon is **demoted from prerequisite to optimization**.
- Stateless per-check invocation — the simplest possible architecture — is viable.

*Bound not yet measured:* replay cost is O(session log). The benchmark is a 1-module
project; a synthetic ~500-form session should be measured before the memoization
question is allowed to matter (it gates nothing now; see §7).

### 2.3 xrepl (the asset the bootstrap didn't name)

Duncan's `xrepl` workspace: an nREPL-inspired LFE REPL — server-side sessions under
supervision, an evaluator isolated from the session loop, **a complete MessagePack wire
protocol** (`xrepl_protocol`: 83 ops / 12 modules / 776 tests, including `lint`-family
ops with `issues` payloads), and TypeScript/Emacs/VSCode clients. Status per Duncan
(2026-06-10): **alive — build on it.** He is resuming xrepl work (a Rust terminal
client; protocol/infra fixes for the audit's eight High findings), which clears this
note's runway as a side effect.

The 2026-06-10 xrepl audit matters here: the protocol layer carries every High-severity
finding (L-01 idle-keepalive desync, L-02 unbounded intake, L-03 atom minting, L-04
session/ETS leak, L-06 decorative security knobs). **S2 below builds on that layer;
those fixes are on its critical path** — and are already planned independently.

### 2.4 Prior art

- **Gleam:** still no REPL (issue #25, open since 2018). Maintainer position: needs
  "a suitable technical design given our current constraints," or the future custom VM.
  Users vocally pained ("for data exploration a REPL is basically mandatory"). Punting
  is survivable for an ML-culture audience; LFE's audience is Lisp culture — the REPL
  *is* the language experience.
- **Elixir v1.20 (2026-06-03):** now officially gradually typed (inference-only,
  verified-bugs). Its interactive story for the type system: **none** — types surface
  via compiler diagnostics; IEx evaluates unchecked. The BEAM's flagship typed effort
  punts REPL typing. Precedent that punting is defensible; also an open lane to be
  first on the BEAM with a typed REPL.
- **GHCi / utop:** redefinition handled by clean shadowing (GHCi keeps old closures
  referencing old types, generational naming). The model to copy — and the replay
  model below gets it for free.
- **evcxr (Rust):** recompiles a crate per input, hundreds of ms per eval, state via
  serialization hacks — the misery avoided by having an interpreter target (the BEAM)
  and a 5 ms checker.
- **reedline 0.48 / ratatui 0.30.1:** mature, actively maintained; full capability map
  in the xrepl companion doc. Relevant here only at the advisory tier (§5).

## 3. The architecture: authoritative server-side checking, stateless, replay-based

### 3.1 The core decisions

**D1 — The authoritative check lives behind the xrepl protocol, BEAM-side.** New
protocol ops (working names): `typed-eval` (check + lower + evaluate),
`typed-check` (check only), `typed-type-of` (query the session registry). Served by
the xrepl session; **every client inherits typed LFE the day the ops land** — Emacs,
VSCode, TypeScript, and the future Rust client are equal citizens. The checker's
existing `--json` diagnostics feed the protocol's `issues` payloads directly.

**D2 — Checking is stateless: session replay, no daemon.** The xrepl session keeps
the log of typed forms it has accepted (this is session state, and it lives where
session state already lives — on the BEAM, so `clone`/reattach work unchanged). Each
check = one ephemeral invocation of `typed-check` with (session log + candidate form).
At 5.3 ms mean, a realistic session replays in single-digit milliseconds.

**D3 — The OTP shape (Duncan's): a `typed-checker-manager` under the xrepl
supervision tree.** A gen_server that, per request, spawns a port child running the
checker binary, gathers stdout (JSON diagnostics or EETF lowered forms), and lets the
child die. Let-it-crash composes with statelessness: a checker crash is a failed check,
cleanly reported; the session log survives in the session process; nothing to recover.
**The daemon question dissolves**: if replay cost ever cliffs, the manager swaps the
ephemeral port for a long-lived one (memoized type env) behind the *same* protocol op —
an implementation detail no client can observe, reversible forever.

**D4 — Redefinition semantics (criterion #6, answered):** the log is the truth.
Later definitions overwrite earlier ones in replay order; dependents are re-checked
**on next use** (lazily — they get re-checked by construction, since every check
replays the current log). No generational shadowing machinery (GHCi needs it because
it holds compiled state; replay just rereads history). A redefinition that breaks a
dependent surfaces the first time the dependent is involved in a check again — the
honest, cheap answer.

**D5 — The evaluation handoff is already built.** The checker emits lowered vanilla
LFE as EETF — designed for the batch driver, equally consumable by the session
evaluator via `binary_to_term` → `lfe_eval`. Zero new serialization.

**D6 — Plain LFE passes through untouched.** Untyped forms at the prompt eval exactly
as today. The membrane is taught by the tool: the REPL *is* the untyped world; typed
forms enter it through the check. (One semantic wrinkle is Duncan's call — §6, Q1.)

### 3.2 Tier-2 macros at the prompt

v1: a typed form whose body uses a user-defined (session) macro is **rejected with a
clear message** — identical to the batch boundary, no new semantics.

Named upgrade path (post-v1): the BEAM side can pre-expand user macros natively
(`lfe_macro` + the live session env) before handing the form to the checker — the one
place typed LFE could *exceed* batch fidelity. Cost: `lfe_macro` discards positions,
so diagnostics for macro-generated code degrade to form-level. A deliberate later
decision, not smuggled into v1.

### 3.3 Bounded new work in the checker CLI

- A per-form / stdin entry point (today: file-path only, module-shaped input).
- `--session-log <file>` (or stdin framing) for the replay input.
- Exit-code/output conventions for "check only" vs "check + lower" (JSON mode exists).

## 4. The option matrix

| Option | Capability | Cost | Risk | Incrementality |
|---|---|---|---|---|
| **O1 — typed-load in existing shell** (`rebar3_lfe` repl and/or xrepl command: load a `.lfet`, render diagnostics, load the compiled module on pass) | module-granular; typed transcripts exist | **days** | minimal | first rung; useful forever |
| **O2 — typed shell conveniences** (`typed-check`, `typed-reload`, `typed-type-of` from registry) | discoverability | days | minimal | rides O1 |
| **O3 — typed protocol ops, replay model** (§3; the keystone) | `defun/typed` at the prompt, **for every protocol client** | ~1–2 weeks BEAM-side + bounded CLI work | low — gated on xrepl L-01..L-06 fixes (already planned) | substrate for everything below |
| **O4 — checker daemon** | latency floor | real (IPC, lifecycle, invalidation) | the classic cache-bug tail | **demoted by the benchmark**: an invisible later swap inside D3's manager |
| **O5 — Rust client embeds the checker** (advisory tier: check-as-you-type, registry completion, caret diagnostics pre-submit) | best-in-class UX | the xrepl Rust-core roadmap (its M1–M3) | owned by the xrepl thread | additive; same engine as O3, so advisory and authoritative never disagree |
| **O6 — punt** | none | zero | pedagogical price: no typed transcripts in the manual | Gleam/Elixir precedent says survivable; not preferred when O1 costs days |

## 5. Where this note diverged from the xrepl architecture review (recorded honestly)

The review's §5 placed the typed checker **in the Rust front-end** ("type-check in
Rust before the BEAM ever sees the code... doing typed-LFE-in-a-REPL well essentially
requires a Rust front-end"). This note rejects that placement for the *authoritative*
check, on three grounds debated and accepted 2026-06-10:

1. **Client fragmentation** — checking in one client gives typed LFE to one client;
   the protocol-as-ecosystem-hub argument (the review's own §6.6) demands the check
   behind the protocol.
2. **Session semantics** — the type registry is session state; sessions are
   server-side, clonable, reattachable. Client-held type state breaks `clone`/attach.
3. **Tier-2** — only the BEAM side can ever pre-expand user macros (§3.2).

The review's co-location insight survives at the **advisory tier** (O5): the Rust
client embeds the same checker crate for sub-frame latency features. LSP pattern:
server authoritative, client advisory, one engine.

## 6. Duncan's calls (genuine surface/UX decisions, not mechanism)

1. **Plain-shadows-typed semantics.** A plain `defun` at the prompt redefining a name
   that has a typed definition in the session log: demote the name to `dynamic` (with
   a warning)? reject? silently shadow? *Recommendation: demote + one-line warning —
   it's the membrane story told truthfully.*
2. **Command-surface naming.** `(typed-load "mod")` vs slash-style vs `load/typed` —
   the `<lfe-form>/typed` convention (M10) may or may not extend to shell commands.
3. **Where O1 lands first** — `rebar3_lfe` repl (reaches everyone today) vs xrepl
   (reaches the architecture we're committing to). *Recommendation: xrepl-first since
   it's now active again; a thin rebar3_lfe variant if cheap.*
4. **Protocol op names/shape** — coordinate with the xrepl protocol's existing
   `lint`/`issues` vocabulary when extending it.

## 7. Staged recommendation

- **S1 (days, on today's stack): O1 + O2.** Typed-aware load + conveniences in the
  xrepl shell (and/or `rebar3_lfe` repl). Ships the membrane story; gives the book
  chapter honest typed transcripts immediately.
- **S2 (the keystone): O3.** `typed-eval`/`typed-check`/`typed-type-of` protocol ops;
  `typed-checker-manager` under the xrepl supervision tree; stateless replay; D1–D6.
  Sequenced behind xrepl's L-01..L-06 protocol fixes (independently planned).
- **S3: typed forms at the prompt become the documented experience** — the chapter's
  transcripts upgrade from "load this file" to `(defun/typed …)` live.
- **S4 (the xrepl thread's M3, not this project's deliverable): the Rust client's
  advisory tier.** Same checker crate in-process; squiggles, completion, pre-submit
  carets.

Explicitly *not* recommended now: O4 as a built artifact (the benchmark killed its
urgency); any LFE interpreter in Rust (boundary discipline, design note 05 — re-affirmed).

**Sequencing note vs M11/M12:** S1 is orthogonal and can land anytime. S2's protocol
ops are *surface-stable* against M11 (multi-clause heads change what the checker
accepts, not the op contract), so S2 need not wait — but S3's *documented* experience
should follow M12's reality-grading so the chapter showcases a surface that has typed
real LFE.

## 8. Would-change-my-mind conditions

1. **Replay cliff:** if a ~500-form synthetic session check exceeds ~100 ms, the
   manager's memoized-port mode moves from "later" to "S2 scope." (Measure before S2
   closes.)
2. **xrepl stall:** if the L-01..L-06 fixes don't land within S2's horizon, S2 retargets
   the stock shell path (O1's wrapper grows the ops locally; protocol later).
3. **Per-form CLI mode proves non-trivial:** if module-shaped assumptions run deep in
   the checker (>~a week to add form-level entry), revisit whether S2 checks
   synthesized single-form modules as a stopgap.
4. **M12 surface churn:** if the dirs port forces major surface changes, S3 waits; S1/S2
   are contract-level and survive.
5. **Session-eval mismatch:** if evaluating lowered forms via `lfe_eval` diverges
   semantically from compiled-module behavior in ways that matter for typed code
   (e.g., guard behavior in interpreted match-lambdas), the lowered-eval path needs a
   compatibility audit before S3 ships transcripts as canon.

## 9. The paragraph for the book chapter (deliverable #4 of the mandate)

> Typed LFE's checking is, today, a batch step: `.lfet` files are checked and lowered
> at build time, and the stock REPL neither checks nor understands typed forms — at
> the prompt you are in the untyped world, and typed modules enter it through the
> membrane, fully checked, when you load them. That isn't a temporary embarrassment;
> it's the gradual-typing story told honestly, and this chapter's transcripts show it.
> The trajectory, though, is concrete: typed checking is being wired into xrepl —
> LFE's nREPL-class shell — as protocol operations backed by the same 5-millisecond
> checker the build uses, with sessions that accumulate your typed definitions and
> re-check against them as you go. When that lands, `(defun/typed …)` at the prompt
> will type-check before it evaluates, and the error you'd have gotten at build time
> arrives interactively instead — same diagnostics, same teaching quality, no waiting
> for a compile.

---

*Evidence base: LFE 2.2.1 `lfe_shell.erl` (file:line cites above); typed-check
benchmark (hyperfine, 2026-06-09, Duncan's machine); xrepl source + 2026-06-10 audit
and architecture review; Gleam issue gleam-lang/gleam#25; Elixir v1.20 release
announcement (2026-06-03); reedline 0.48 / ratatui 0.30.1 capability map (xrepl
workbench). Drafted after debate convergence, 2026-06-10.*
