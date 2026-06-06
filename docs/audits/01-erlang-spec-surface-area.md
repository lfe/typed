# Audit 1 — The Erlang Type-Spec Surface Area

> **Project:** `typed` — an experiment in a statically typed LFE with algebraic
> data types.
> **Audit question:** *What is the complete spec surface area for Erlang? Is
> anything uncovered?*
> **Status:** complete, pending review.

## How to read this document

This is an **inventory**, not an argument. Its job is to enumerate, exhaustively
and authoritatively, everything Erlang's own type-specification language can say,
so that later we can checklist what our LFE typed layer (`defun/typed` and
friends) must be able to express, emit, and explain.

Three lenses are applied to every construct:

1. **Surface** — the Erlang syntax and the AST node it parses to.
2. **LFE need** — what an s-expression surface has to be able to express to
   cover it.
3. **Diagnostic** — how a *great* error message would explain a violation of it
   (because a stated goal of this project is tooling whose output teaches both
   humans and LLMs how to fix mistakes).

### Provenance (ground truth, not memory)

Everything here is read from the OTP source mounted in this workspace, **pinned
to the version we are building against**, not from training priors. This matters
because the surface has grown materially in the last three releases, past the
point any static prior could be trusted.

| | |
|---|---|
| OTP version (mount) | **30.0-rc0** (release candidate; `OTP_VERSION`) |
| Grammar | `lib/stdlib/src/erl_parse.yrl` |
| Attribute dispatch | `lib/stdlib/src/erl_parse.yrl` `build_typed_attribute/2`, `build_type_spec/2` |
| Reference docs | `system/doc/reference_manual/{typespec,opaques,nominals,ref_man_native_records,data_types}.md` |

**Version-sensitive features** (flagged inline as you read):

- `dynamic()` gradual-typing type — OTP 26
- Redefining built-in type names — OTP 26
- `-nominal` nominal types — OTP 28
- Dialyzer checks opaques like nominals in-module; `opaque_union` option — OTP 28
- **Native records** (`-record #Name{...}` as a distinct type) — OTP 29,
  **experimental, may change incompatibly in OTP 30**

---

## 1. The surface has two layers

Conflating them is the most common way to get the design wrong, so we separate
them up front:

- **The declaration layer** — the module attributes that *introduce* or *attach*
  type information: `-type`, `-opaque`, `-nominal`, `-spec`, `-callback`,
  `-record` (tuple-based and native), `-export_type`, `-export_record`,
  `-import_record`. This is the layer our macros emit.
- **The type-expression layer** — the grammar of a *type* itself (`integer()`,
  `{ok, T} | error`, `#{atom() => term()}`, …). This appears on the right of
  `::` and inside specs. This is the layer our macros must let users *write*.

Section 2 covers the declaration layer; Section 3 the type-expression layer.

---

## 2. The declaration layer

### 2.1 Type aliases: `-type`, `-opaque`, `-nominal`

All three share one syntactic shape and differ only in **type-equivalence
semantics**:

```erlang
-type     my_struct_type()      :: Type.
-opaque   my_opaque_type()      :: Type.
-nominal  my_nominal_type()     :: Type.
-type     orddict(Key, Val)     :: [{Key, Val}].   %% parameterized
```

Dispatch is by attribute atom — the parser accepts `type | opaque | nominal` in
exactly one production (`erl_parse.yrl` `build_typed_attribute/2`, the
`Attr =:= 'type' ; Attr =:= 'opaque' ; Attr =:= 'nominal'` clause). Parameters
are type variables (uppercase, must each appear on the RHS; a bare `_` parameter
is a compile error — "bad type variable").

| Attribute | Equivalence discipline | Who enforces |
|---|---|---|
| `-type` | **Structural.** `meter()` and `foot()` both `:: integer()` are *the same type*. | n/a (names erased) |
| `-opaque` | **Structural inside the module; sealed outside.** Consumers must not pattern-match/inspect; only `=:=`/`=/=` against same-name or `any()` is allowed. | Dialyzer (since OTP 28, checked in-module like nominal; `opaque_union` option warns on union leakage). Runtime does **not** enforce — opacity is "skin-deep." |
| `-nominal` | **Nominal.** `meter()` and `foot()` are distinct *even with identical structure*. A nominal is still compatible with the bare structural type (`meter()` ↔ `integer()`), and with another nominal only by *derivation* (`-nominal s() :: t().`). | Dialyzer only; the compiler does **no** nominal checking. |

**This is the single most important finding for ADTs.** Erlang now ships three
distinct equivalence disciplines. A "statically typed LFE with ADTs" is, in
large part, a question of *which discipline our constructors get* — and `-nominal`
gives us name-based distinctness for free, which is exactly what a sum-type tag
wants.

- **LFE need:** a `deftype`-family form taking a name, optional type params, and
  a body type. Three variants (or one with a discipline keyword). Export
  handling (see 2.4).
- **Diagnostic:** nominal mismatches already produce a teachable Dialyzer
  message ("The return types do not overlap… success typing is `meter()`… spec
  is `foot()`"). Our tooling should *intercept and re-render* these in LFE terms
  rather than leak Erlang type syntax to an LFE author.

### 2.2 Function specifications: `-spec`

The richest part of the declaration layer. All forms (`typespec.md` §Specifications
for Functions; grammar `type_spec`, `spec_fun`, `type_sig`, `type_guards`):

```erlang
%% basic
-spec f(ArgType1, ..., ArgTypeN) -> ReturnType.
%% module-qualified (Module must be the current module) — documentation only
-spec m:f(A) -> B.
%% named arguments (documentation only)
-spec f(Name1 :: T1, Name2 :: T2) -> RT.
%% overloaded (multiple clauses; ';'-separated). Domains must not overlap (Dialyzer warns)
-spec f(T1, T2) -> T3;
       (T4, T5) -> T6.
%% polymorphic via type variables (relate input to output)
-spec id(X) -> X.
%% bounded quantification via 'when' subtype constraints
-spec id(X) -> X when X :: tuple().
-spec foo({X, integer()}) -> X when X :: atom();
         ([Y]) -> Y when Y :: number().
%% non-returning
-spec my_error(term()) -> no_return().
```

Constraint grammar (`erl_parse.yrl` `type_guard`):

- `var '::' top_type` → the modern `V :: Subtype` constraint (`build_constraint`).
- `atom '(' top_types ')'` → the legacy compatibility constraint
  (`build_compat_constraint`, i.e. the old `is_subtype(V, T)` form).
- `::` ("is a subtype of") is **the only** guard constraint allowed in `when`.

Notes that matter for us:

- The arity of the spec **must** match a real function in the module, or
  compilation fails. (So our macro, which owns both spec and body, can guarantee
  this by construction — a free correctness win.)
- `_` in a spec is an anonymous type variable ≡ `term()`/`any()`.
- Whether the extra information in repeated type variables (`id(X) -> X`) is
  *used* is up to the tool; the compiler just records it.

- **LFE need:** this is where the Lykn-style contract syntax lives. `:args`,
  `:returns`, plus first-class support for `:when`/constraints, overloaded
  clauses, named args, and polymorphic variables. The ergonomics challenge is
  representing overloaded specs and `when`-constraints without losing the
  "no extra cognitive load" property.
- **Diagnostic:** the overlapping-domain warning and the spec-vs-success-typing
  mismatch are the two highest-value messages to re-render in LFE form.

### 2.3 Behaviour callbacks: `-callback`

Same `type_sig` grammar as `-spec` (`erl_parse.yrl`:
`attribute -> '-' 'callback' type_spec`), but declares the contract a behaviour
*requires of its implementers* rather than describing a local function.
`-optional_callbacks([F/A,...])` marks some optional.

- **LFE need:** a typed behaviour-definition form. Lower priority than `-spec`
  but in scope eventually (OTP behaviours are how real LFE is written).
- **Diagnostic:** "module claims `-behaviour(X)` but callback `c/2` is missing or
  mis-specified" is a teachable, high-value message.

### 2.4 Exporting & remote types

```erlang
-export_type([my_struct_type/0, orddict/2]).   %% by name/arity
mod:my_struct_type()                            %% remote type reference (use site)
mod:orddict(atom(), term())
```

Only exported types may be referenced remotely; referencing an unexported remote
type is an error. Type definitions must resolve to predefined types, module-local
types, or exported remote types (enforced by the compiler — undefined local type
is a compile error).

- **LFE need:** export-type plumbing, and remote-type syntax in the
  type-expression surface (see 3.10).

### 2.5 Tuple-based records with field types

```erlang
-record(rec, {field1 :: Type1, field2, field3 = 42 :: Type3}).
```

- Untyped field defaults to `any()`.
- Default value must be *compatible with* the field type — **the compiler checks
  this and errors on violation** (one of the few real type checks the compiler
  itself performs).
- Since OTP 19, an uninitialized typed field is **no longer** silently widened
  with `'undefined'`; you must add it yourself if needed.
- A defined record is usable as a type: `#rec{}`, and refinable at the use site:
  `#rec{some_field :: Type}` (tuple-based only). In specs/types, a named type
  `person()` is preferred over `#person{}`.

### 2.6 Native records — OTP 29, experimental ⚠️

A *distinct data type* (not a tuple), introduced OTP 29 and explicitly
"may change incompatibly in OTP 30" (`ref_man_native_records.md`). Surface:

```erlang
-record #Name{Field1 [= Expr1], ...}.   %% definition (atom name, no quoting needed)
-export_record([Name1, ...]).            %% expose fields cross-module
-import_record(Module, [Name1, ...]).    %% use unqualified
#Name{f=V}            #Module:Name{f=V}  %% construction (local / external)
Expr#Name.Field       Expr#Module:Name.Field   %% access
Expr#Name{f=V}        Expr#_{f=V}        %% update (named / anonymous)
#Name{f=P}            #_{f=P}            %% pattern (named / anonymous)
is_record(T) | is_record(T,Name) | is_record(T,Module,Name)   %% guards
```

Distinctive semantics with deep ADT implications:

- **A native record is its own type** — `is_record/1` returns `false` for
  tuple-based records. This is genuine *nominal product typing baked into the
  runtime*, not a convention.
- **Captured definition** — the record definition is embedded in the value at
  construction time; all later operations use the captured definition, so values
  survive code reload / cross-node transport. (Relevant to the
  cross-process-soundness problem we deferred — a captured definition is more
  self-describing than a bare tuple.)
- Compile-time error if a local construction omits a field with no default;
  runtime exception for the external case.
- Grammar: `native_record`, `typed_native_record`, and the type forms
  `#atom:record_name{}` (`erl_parse.yrl`:163-167, 222-227).

- **LFE need:** if we adopt native records as our constructor carrier, we need
  surface for definition, construction, access, update, and **pattern matching**
  in `match`/`case`, plus export/import. The win is that exhaustiveness and tag
  distinctness get real runtime + Dialyzer support instead of relying on tuple
  tags.
- **Risk:** experimental status. A v0 should probably support *both* a tuple-tag
  representation (stable, works on OTP 26+) and native records (future-facing),
  behind one ADT surface — and the audit suggests we treat representation as a
  pluggable backend, not a baked-in choice.

### 2.7 Redefining built-in type names — OTP 26

Since OTP 26 you may define a type with the same name as a built-in; new
built-ins in later releases won't break code that already used the name (compiles
with a warning). Relevant because our generated type names must not assume the
built-in namespace is reserved.

---

## 3. The type-expression layer (the grammar)

This is the exhaustive set of type forms, taken directly from `erl_parse.yrl`
(`top_type`, `type`, `fun_type`, `map_pair_type`, `field_type`, `binary_type`).
Each row: surface → AST node → note.

### 3.1 Top-level combinators

| Surface | AST | Note |
|---|---|---|
| `T1 \| T2 \| …` | union (`lift_unions`) | The only sum mechanism in the language. Subtype absorbed by supertype in a union. |
| `Var :: T` | `ann_type` | Annotated type (named position), e.g. in args. |
| `( T )` | — | Grouping. |

**Critical for ADTs:** Erlang has **no native sum-type declaration**. A sum is
just a `|` union — typically of tagged tuples or singleton atoms. There is no
construct that says "these constructors are the *complete* set," hence no
language-level exhaustiveness. This is the central gap our ADT layer fills
(Section 5).

### 3.2 Variables, atoms, singletons

| Surface | AST | Note |
|---|---|---|
| `A` (uppercase) | `var` | Type variable. |
| `_` | `var '_'` | Anonymous var ≡ `any()`. |
| `atom()` | `build_gen_type` | The `atom` type. |
| `'foo'` | `atom` literal | **Singleton atom type** — the basis of constructor tags. |
| `name()` / `name(T,...)` | `build_type` | User/predefined type application. |

### 3.3 Numbers

| Surface | AST | Note |
|---|---|---|
| `integer()`, `float()` | builtin | |
| `42`, `$a` | `integer` / `char` | Singleton integer/char types. |
| `Lo..Hi` | `range` | Integer range. Open ranges via aliases: `0..`=`non_neg_integer()`, `1..`=`pos_integer()`, `..-1`=`neg_integer()`. |
| arithmetic in types | `mkop1`/`mkop2` | `+ - * div rem band bor bxor bsl bsr`, unary `+ - bnot`, nestable; must evaluate to an integer (also valid in ranges and bitstrings). |

### 3.4 Tuples

| Surface | AST | Note |
|---|---|---|
| `tuple()` | builtin | Tuple of any size/shape. |
| `{}` | `tuple []` | Empty-tuple singleton. |
| `{T1, …, Tn}` | `tuple [..]` | Fixed-shape tuple. The workhorse for tagged-tuple constructors: `{ok, T}`. |

### 3.5 Maps

| Surface | AST | Note |
|---|---|---|
| `#{}` | `map []` | **Empty-map singleton**, *not* `map()`. |
| `#{K => V}` | `map_field_assoc` | Optional association. |
| `#{K := V}` | `map_field_exact` | Mandatory association. |
| `map()` | alias | `#{any() => any()}`. |

Key types may overlap; leftmost association wins.

### 3.6 Lists

| Surface | AST | Note |
|---|---|---|
| `[]` | `nil` | Empty-list singleton (**not** `list()`). |
| `[T]` | `list` | Proper list, may be empty. Shorthand for `list(T)`. |
| `[T, ...]` | `nonempty_list` | Non-empty proper list. Shorthand for `nonempty_list(T)`. |
| `list()` | alias | `[any()]`. Note the bare-list type is `[_]`, not `[]`. |
| `maybe_improper_list(T1,T2)`, `nonempty_improper_list(T1,T2)`, `nonempty_maybe_improper_list/0,2` | builtins | Improper-list family (rare; long names). |

### 3.7 Binaries / bitstrings

| Surface | AST | Note |
|---|---|---|
| `<<>>` | `binary [0,0]` | Empty bitstring. |
| `<<_:M>>` | `binary [M,0]` | Exactly M bits. |
| `<<_:_*N>>` | `binary [0,N]` | k·N bits. |
| `<<_:M, _:_*N>>` | `binary [M,N]` | M + k·N bits. |

Aliases: `binary()`=`<<_:_*8>>`, `bitstring()`=`<<_:_*1>>`, plus
`nonempty_binary/0`, `nonempty_bitstring/0`.

### 3.8 Functions

| Surface | AST | Note |
|---|---|---|
| `fun()` | `'fun' []` | Any function. |
| `fun((...) -> T)` | `fun [any, T]` | Any arity returning T. |
| `fun(() -> T)` | `fun [product [], T]` | Nullary. |
| `fun((T1,…) -> T)` | `fun [product [..], T]` | Fixed arity. |

### 3.9 Records as types

| Surface | AST | Note |
|---|---|---|
| `#rec{}` / `#rec{f :: T}` | `record [name \| fields]` | Tuple-based record type, with optional field refinement. |
| `#mod:name{}` / `#mod:name{f :: T}` | `record [{tuple,[mod,name]} \| fields]` | Native record type (OTP 29). |
| field `f :: T` | `field_type` | Record field type. |

### 3.10 Remote types

| Surface | AST | Note |
|---|---|---|
| `mod:type()` / `mod:type(T,...)` | `remote_type` | Reference to an exported type in another module. |

### 3.11 The escape hatch

| Surface | Note |
|---|---|
| `dynamic()` | OTP 26. A statically-unknown type for gradual typing (like TS `any`, Python `Any`). Everything except `dynamic()` forms the lattice. Dialyzer treats `any()`/`dynamic()` identically under success typing. **This is our principled boundary type for untyped-Erlang interop.** |

---

## 4. Predefined types & built-in aliases (complete)

**Primitive / opaque-to-us:** `any()` (≡ `term()`), `none()`, `dynamic()`,
`pid()`, `port()`, `reference()`, `atom()`, `float()`, `integer()`.

**Aliases** (`typespec.md` built-in table — each is exactly a union/expansion):

| Alias | Expansion |
|---|---|
| `term()` | `any()` |
| `binary()` | `<<_:_*8>>` |
| `nonempty_binary()` | `<<_:8, _:_*8>>` |
| `bitstring()` | `<<_:_*1>>` |
| `nonempty_bitstring()` | `<<_:1, _:_*1>>` |
| `boolean()` | `'false' \| 'true'` |
| `byte()` | `0..255` |
| `char()` | `0..16#10ffff` |
| `nil()` | `[]` |
| `number()` | `integer() \| float()` |
| `list()` | `[any()]` |
| `maybe_improper_list()` | `maybe_improper_list(any(), any())` |
| `nonempty_list()` | `nonempty_list(any())` |
| `string()` | `[char()]` |
| `nonempty_string()` | `[char(), ...]` |
| `iodata()` | `iolist() \| binary()` |
| `iolist()` | `maybe_improper_list(byte() \| binary() \| iolist(), binary() \| [])` |
| `map()` | `#{any() => any()}` |
| `function()` | `fun()` |
| `module()` | `atom()` |
| `mfa()` | `{module(), atom(), arity()}` |
| `arity()` | `0..255` |
| `identifier()` | `pid() \| port() \| reference()` |
| `node()` | `atom()` |
| `timeout()` | `'infinity' \| non_neg_integer()` |
| `no_return()` | `none()` |
| `non_neg_integer()` | `0..` |
| `pos_integer()` | `1..` |
| `neg_integer()` | `..-1` |

**Coverage note:** every alias is sugar over §3 forms. Our LFE surface can treat
these as a fixed builtin lexicon mapping to keyword type-atoms (e.g. `:string`,
`:non-neg-integer`), which is exactly the low-cognitive-load mapping the Lykn
experience suggests.

---

## 5. The term lattice & what the language can't say

- **Top:** `any()`/`term()`. **Bottom:** `none()` (`no_return()` ≡ `none()`).
- `dynamic()` sits *outside* the lattice — the gradual-typing seam.
- Subtyping is **structural** by default; `-nominal` adds name-based distinctness;
  `-opaque` adds module-scoped sealing.

**What the type language genuinely cannot express** (the build-tier list — these
are the gaps an ADT layer exists to fill):

1. **No native sum-type *declaration* with a closed constructor set.** Sums are
   open `|` unions; nothing marks "this is all the cases," so the language gives
   no exhaustiveness.
2. **No exhaustiveness checking of pattern matches** against a declared sum.
3. **No data *constructors* with parametric payloads** beyond what you hand-roll
   as tagged tuples / native records + a `-type` union.
4. **No type classes / bounded polymorphism beyond `when … :: …`.** Constraints
   are subtype bounds only.
5. **No higher-kinded types**, no row polymorphism.
6. **Opacity is convention** (runtime does not enforce); nominal/opaque checks
   live **only in Dialyzer**, not the compiler.
7. **No GADTs / dependent refinement** beyond singleton + range types.

---

## 6. The crucial finding: two enforcement tiers ("free" vs "build")

Erlang ships type *syntax* and type *checking* in different places. This is the
strategic heart of the audit and directly answers "is anything uncovered?":

**The compiler enforces only:**

- Spec/function arity agreement (spec without matching function ⇒ error).
- Local/remote type definitions exist and are exported as claimed.
- Type-variable well-formedness in `-type`/`-opaque`/`-nominal`.
- Record field default values are compatible with declared field types.
- Native-record field-initialization completeness (local ⇒ compile error).

**Everything else is Dialyzer** (success typing) — and success typing is
**optimistic**: it reports only what *cannot* succeed and **never rejects** a
program that *might*. Nominal and opaque checking, spec-vs-implementation
agreement, the overlapping-domain warning: all Dialyzer, all opt-in, all
non-total.

**Implication for `typed`.** "Statically typed with ADTs" implies *rejection* —
the thing Dialyzer is designed *not* to do. So:

- The **free tier** (emit `-spec`/`-type`/`-nominal`/native records, lean on the
  compiler's five checks + Dialyzer) gets us documentation, a lot of bug-finding,
  and nominal tag distinctness — for almost no implementation cost. It is real
  value and we should harvest it.
- The **build tier** — closed sums, exhaustiveness, constructor payload checking,
  and *rejecting* programs at our own analysis pass with teaching-quality
  diagnostics — is precisely the §5 list, and it is *ours to build*. Dialyzer
  will not do it, and emitting more specs will not conjure it.

The honest read: the contract macro can ride the free tier immediately, but the
ADT thesis lives entirely in the build tier.

> **LFE-specific correction (added after review).** I originally framed Dialyzer
> as "defense in depth" behind our checker. That assumption does **not** hold for
> LFE. Per Duncan: Robert Virding (LFE's creator) invested heavily in making
> Dialyzer work for LFE, but it **breaks as soon as macros or pure-LFE includes
> are involved** — and a macro library is nothing *but* macros. So Dialyzer is
> not a dependable second line for LFE projects. This *removes the safety net*
> and makes our own checker **load-bearing, not optional**. The free tier is
> consequently weaker for LFE than for plain Erlang (we still emit specs for docs
> and for the rare Dialyzer-clean module), but essentially all the real
> type-safety value must come from the build tier we write ourselves.

---

## 7. Coverage checklist for the LFE typed layer

A concrete "are we done?" list for the surface we must eventually cover:

- [ ] `-type` / `-opaque` / `-nominal` aliases, parameterized, with export.
- [ ] `-spec`: basic, named args, polymorphic vars, `when` constraints,
      overloaded clauses, `no_return()`.
- [ ] `-callback` / `-optional_callbacks`.
- [ ] Tuple-based `-record` with field types + use-site refinement.
- [ ] Native records (OTP 29): define/construct/access/update/match/export/import.
- [ ] Full type-expression grammar §3.1–§3.11 (unions, ann types, singletons,
      ranges + arithmetic, tuples, maps `=>`/`:=`, list family, bitstring family,
      fun family, record types, remote types, `dynamic()`).
- [ ] The complete builtin alias lexicon (§4).
- [ ] Remote-type references and `-export_type`.

Anything *not* on this list is, by construction, outside Erlang's spec surface —
i.e. build-tier ADT machinery (§5), not "coverage."

---

## 8. Diagnostic-surface notes (consolidated)

Because tooling output is a first-class deliverable, the highest-value messages
to own and re-render in LFE terms:

1. **Nominal/opaque mismatch** — Dialyzer already emits a teachable form; we
   intercept and translate to LFE syntax (no raw Erlang type leakage).
2. **Spec vs success typing disagreement** — the canonical "your spec says X, the
   code does Y" message.
3. **Overlapping overloaded-spec domains.**
4. **Native-record uninitialized field** (compile-time; good model for our own
   constructor-arity errors).
5. **Our own build-tier diagnostics** — non-exhaustive match against a declared
   sum, wrong constructor arity/payload type, unknown constructor — which have no
   Erlang analog and are the ones most worth making excellent.

---

## 9. Open questions carried into design

1. **Constructor carrier:** ✅ *Decided.* **Pluggable backend.** Default =
   **native records (OTP 29+)** — a true, runtime-distinct data type; fallback =
   **tuple-based** representation for older OTP. One ADT surface over both. A
   single test suite runs the **full matrix across every supported backend** to
   prove identical ADT semantics regardless of carrier.
2. **Equivalence discipline for our ADTs:** structural, or lean on `-nominal`
   for tag distinctness?
3. **How much free tier to harvest before building the rejecting checker** — and
   whether we run *before* or *alongside* Dialyzer.
4. **`dynamic()` as the one true interop boundary** — adopt as the blessed
   coercion type for calling untyped Erlang/OTP?
5. Overloaded specs and `when`-constraints without breaking the "no extra
   cognitive load" surface promise.

---

*Sources (all in the mounted OTP 30.0-rc0 tree): `lib/stdlib/src/erl_parse.yrl`
(grammar + `build_typed_attribute/2`); `system/doc/reference_manual/typespec.md`,
`opaques.md`, `nominals.md`, `ref_man_native_records.md`.*
