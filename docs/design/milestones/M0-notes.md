# M0 — Skeleton & Plumbing: Implementation Notes

## Environment

| Component | Version |
|-----------|---------|
| OTP | 28 (BEAM emulator 16.1.1) |
| LFE | 2.2.1 (pinned in rebar.config) |
| Rust | 1.93.0 |
| Platform | macOS Darwin 24.6.0, aarch64 |

## Semi-internal API usage

The following LFE/OTP APIs are not part of the documented public interface and
may change across versions:

| API | Module | Risk |
|-----|--------|------|
| `lfe_codegen:module/2` | `lfe_codegen.erl` | Semi-internal; used per experiment 01. The `#cinfo{}` record is defined in `lfe_comp.hrl`. |
| `lfe_lint:module/2` | `lfe_lint.erl` | Semi-internal; accepts `[{Form,Line}]` + `#cinfo{}`. |
| `#cinfo{}` record | `lfe_comp.hrl` | Internal record; fields: `file`, `opts`, `ipath`, `mod`. |
| `compile:forms/2` | OTP stdlib | Documented, stable. |

**Mitigation:** LFE version pinned to `2.2.1` in `rebar.config`. These APIs
should be proposed for stabilization to the LFE maintainer as a collaboration
touchpoint.

## OTP 28 compatibility note

`lfe -eval` is broken on OTP 28 due to a `user_drv` `{badkey, input}` crash.
See `docs/design/03-lfe-otp28-user-drv-bug.md` for the full analysis and
proposed fix. This does not affect M0 — we use `erl -noshell` with LFE on the
code path, and our chain bypasses `lfe_init` entirely.

## File extensions

Typed LFE source files use `.tlfe` to distinguish them from plain `.lfe` files
and prevent rebar3's LFE compiler from attempting to compile them directly.

## Architecture decisions confirmed by M0

1. **Model-Y works.** The `typed-check` → EETF → `typed_driver` →
   `lfe_codegen:module` → `compile:forms` chain produces working `.beam` files
   with correct original-source line injection, verified end-to-end.

2. **EETF handoff works.** Rust emits Erlang External Term Format, Erlang
   decodes with `binary_to_term/1`. No third-party Erlang library needed.

3. **oxur-sexp is viable.** The vendored S-expression reader (from `oxur`)
   parses typed LFE with correct line+column positions. Two adaptations were
   made: added `parse_all_str`/`parse_all_file` for multi-form files, and added
   quote (`'`) and `.` to the symbol character set.
