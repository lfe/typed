# Audit 2 — The Erlang Data-Type Taxonomy

> **Project:** `typed` — an experiment in a statically typed LFE with algebraic
> data types.
> **Audit question:** *What is the complete data-type surface area and taxonomy
> in Erlang?*
> **Status:** complete, pending review.
> **Companion to:** [Audit 1 — spec surface area](01-erlang-spec-surface-area.md).
> Where Audit 1 inventoried the *type language* (what you can **say** about
> terms), this audit inventories the *terms themselves* (what actually **exists**
> at runtime) and maps the two together.

## Provenance

Ground truth from the mounted OTP **30.0-rc0** tree and the project's
erlang-guidelines substrate — not memory:

| Source | Used for |
|---|---|
| `system/doc/reference_manual/data_types.md` | The runtime term taxonomy |
| `system/doc/reference_manual/expressions.md` §Term Comparisons | Total term order + comparison semantics (`:837`) |
| `system/doc/reference_manual/ref_man_native_records.md` | Native records (OTP 29) |
| `lib/stdlib/src/erl_parse.yrl` | Literal/term grammar cross-check |
| erlang-guidelines `guides/04-data-and-types.md` (DT-01…DT-19) | Idiomatic representation choices |

---

## 1. The keystone: it's terms all the way down

The single most important sentence in `data_types.md` (`:28`):

> *"Erlang has no user-defined types, only composite types (data structures)
> made of Erlang terms. This means that any function testing for a composite
> type, typically named `is_type/1`, might return `true` for a term that
> coincides with the chosen representation."*

This is the ground on which the entire project stands. At **runtime** there is
only a fixed, closed set of term kinds. The **type language** of Audit 1 does not
add new runtime entities — it merely *carves named subsets* out of the universe
of terms. Therefore:

> **An ADT is a type-level fiction with no runtime counterpart except the
> representation we choose for it.** `Result(T,E) = Ok(T) | Error(E)` does not
> exist at runtime; what exists is whatever term we pick to *stand for* `Ok(x)` —
> a tagged tuple, a native record, an atom. The "algebraic" structure lives
> entirely in our checker and our diagnostics.

And the corollary, straight from the keystone sentence: **structural predicates
can false-positive.** A hand-written `{ok, 42}` is indistinguishable from our
`Ok(42)` if both compile to the same tuple. This is precisely the hazard that
native records (a *true* data type, §3.5) and `-nominal` (Audit 1 §2.1) exist to
close, and a central reason they matter to us.

---

## 2. Primitive (atomic) term types

Each row: what it is · literal forms · runtime predicate · Audit-1 spec type ·
relevance to ADTs.

### 2.1 Integer

- **Arbitrary precision** (bignums) — no overflow. Literals: decimal with `_`
  separators (`1_234`), `$char` (codepoint, e.g. `$A` = 65), `base#digits`
  (base 2–36, e.g. `16#1f`, `36#hello`). No `0x`/`077` prefixes.
- Predicate: `is_integer/1`. Spec types: `integer()`, singleton `42`, ranges
  `Lo..Hi`, and the open-range aliases (`non_neg_integer()` etc.).
- **ADT:** singleton-integer and range types are usable as constructor payload
  refinements and as "enum-like" carriers.

### 2.2 Float

- **64-bit**, base-2. **No `Inf`/`NaN`** — any operation that would produce them
  raises `badarith`. `0.0` and `-0.0` are the *same number* (`==`) but *distinct
  terms* (`=:=`) — a bug-prone edge fixed/clarified in OTP 27.
- Predicate: `is_float/1`. Spec type: `float()`.
- **ADT:** the `±0.0` and no-`NaN` facts matter for *equality of constructor
  values* — our derived equality must decide `=:=` vs `==` semantics deliberately
  (see §5).

### 2.3 Atom

- A named constant. Quoting required unless it begins lowercase and is
  alphanumeric/`_`/`@`. The **atom table is bounded (~1M default) and never
  garbage-collected** → **never mint atoms from untrusted input** (DT-13;
  use `binary_to_existing_atom/2`).
- Predicate: `is_atom/1`. Spec types: `atom()`, singleton `'foo'`, `boolean()`.
- **ADT:** *the* representation for **nullary constructors**. A singleton atom
  `'None'` is the natural runtime value for a payload-free case, and the leading
  atom tag is what makes tagged-tuple constructors dispatchable. Booleans (`true`/
  `false`), `nil`, `undefined`, `ok`, `error` are all just atoms — so our sum
  encodings live in the same namespace as ordinary Erlang control atoms (a
  collision hazard, §8).

### 2.4 Reference

- A term unique among connected nodes (`make_ref/0`). Predicate: `is_reference/1`.
  Spec type: `reference()`. ADT relevance: low (opaque identity token); but useful
  as a carrier for *generative* identity if we ever need un-forgeable tags.

### 2.5 Fun

- A functional object (closure). `is_function/1,2`. Spec types: `fun()`,
  `fun((...) -> T)`, etc. ADT relevance: payloads can be funs; higher-order
  constructors are expressible but unusual.

### 2.6 Port identifier & 2.7 Pid

- System-level identities (`is_port/1`, `is_pid/1`; `port()`, `pid()`,
  `identifier()`). ADT relevance: low — these are the untyped seams of the BEAM
  (processes, ports), exactly the boundary where `dynamic()` (Audit 1 §3.11)
  earns its keep.

### 2.8 Bit string & binary

- A **bit string** is an area of untyped memory. A **binary** is a bit string
  whose length is divisible by 8. Built/destructured with the **bit syntax**
  (`<<Version:8, Len:16, Payload:Len/binary, Rest/binary>>` — DT-12).
- Predicates: `is_bitstring/1` (any), `is_binary/1` (8-divisible). Spec types:
  the `<<_:M, _:_*N>>` family and aliases `binary()`/`bitstring()`.
- **ADT:** binaries are the idiomatic **text** carrier (DT-10) and the wire
  format (`term_to_binary/1`). Constructor payloads are frequently binaries;
  serialization of ADT values crosses here.

---

## 3. Compound (composite) term types

### 3.1 Tuple

- Fixed-size ordered collection `{T1,…,Tn}`. `element/2`, `setelement/3`,
  `tuple_size/1`, `is_tuple/1`. Spec types: `tuple()`, `{}`, `{T1,…,Tn}`.
- **ADT — the classic carrier.** A **tagged tuple** `{Ctor, F1, …, Fn}` is
  Erlang's de-facto algebraic encoding: the leading atom is the constructor tag,
  the rest is the (positional) payload; a `|` union of tagged tuples is the sum
  (DT-01). This is how `{ok,V} | {error,R}` already works.
- **Limitation:** a tuple has no identity beyond its shape, so `is_tuple/1` (and
  even tag matching) can't tell *our* `{Ok, V}` from a coincidental `{'Ok', V}`
  built elsewhere. Tag distinctness is by convention only.

### 3.2 Map

- Variable-size key→value associations `#{K => V}`. `=>` upserts; `:=` updates an
  existing key only (else `badkey`) — use `:=` so typos fail loudly (DT-08).
  `maps:*`, `map_size/1`, `is_map/1`. Spec types: `#{}`, `#{K => V}`, `#{K := V}`,
  `map()`. (Maps OTP 17/18.)
- **ADT:** a possible carrier (`#{'__struct__' => Ctor, …}`, à la Elixir structs)
  but it sacrifices compile-time field safety and isn't idiomatic for sums; better
  suited to *open/extensible* data crossing system boundaries (DT-06). Considered
  and not recommended as the default constructor carrier (§7).

### 3.3 List

- Either `[]` (nil) or a cons `[H|T]` whose tail is (usually) a list. **Proper**
  list ends in `[]`; **improper** ends in a non-list (`[a|b]`). `is_list/1`,
  `length/1`, `lists:*`. Spec types: `[]`/`nil()`, `[T]`, `[T,...]`, the improper
  family. **Note the term order treats `nil` as a separate type from `list`**
  (§5).
- **ADT:** lists are the natural payload for *sequence* fields and the structural
  model for recursive ADTs (`List(T) = Nil | Cons(T, List(T))`) — though Erlang
  lists are themselves the obvious representation for that particular ADT.

### 3.4 Record (tuple-based) — **not a true data type**

- `-record(person,{name,age})` → at runtime a **tuple** `{person, Name, Age}`
  (`data_types.md:709`). Access via `#rec{}`/`Var#rec.field` (DT-02). Records are
  *syntax over tuples*; the shell needs help to see them.
- **ADT:** a tuple-based record is just a tagged tuple with named-field sugar — a
  viable, ergonomic constructor representation on **any** OTP version, and the
  natural **fallback backend**. But it inherits §3.1's identity weakness
  (`is_tuple/1` is `true`).

### 3.5 Native record (OTP 29, experimental ⚠️) — **a true data type**

- `-record #person{name, age}` → a **distinct runtime type**:
  `is_record(P)` ⇒ `true`, `is_tuple(P)` ⇒ `false`, `is_map(P)` ⇒ `false`
  (`data_types.md:750`). Distinct construction/access/update/match/guard surface,
  `#Module:Name{}` external form, **captured definition** (the definition is
  embedded in the value at construction, so values survive code reload and
  cross-node transport — see Audit 1 §2.6).
- **ADT — the decided default carrier (OTP 29+).** This is the closest thing
  Erlang has ever shipped to a *nominal product type with a runtime identity*:
  constructor values are genuinely their own type, not a tuple wearing a tag.
  Combined with `-nominal` it gives tag distinctness the runtime actually
  enforces, closing the §1 false-positive hazard.
- **Caveat:** experimental; "may change incompatibly in OTP 30." Hence the
  pluggable-backend decision (§7).

---

## 4. The "looks like a type but isn't" set

A naive ADT designer will mistake these for primitives; they are sugar or
encodings and must be handled as such by our reader/printer:

| Surface | Reality |
|---|---|
| **String** `"hello"` | Sugar for a list of codepoints `[$h,$e,…]`. Not a data type. Adjacent literals concatenate at compile time (must be whitespace-separated since OTP 27). **Triple-quoted strings** (OTP 27). |
| **Boolean** | No boolean type — the atoms `true`/`false`. `is_boolean/1`. |
| **Record** (tuple-based) | A tuple (§3.4). |
| **Sigil** `~b"..."`, `~s"..."`, `~"..."` (OTP 27) | A prefix on a string literal selecting a transform (→ UTF-8 binary or codepoint list), verbatim variants `~B`/`~S`. Not a data type. |

**ADT implication:** our surface's `:string`, `:boolean` keyword types must
*compile down* to their real carriers (codepoint list / `true|false` atoms),
and our pattern matcher must understand that a "string pattern" is a list
pattern. Sigils give us a clean way to denote binary vs charlist text payloads.

---

## 5. The total term order & comparison semantics

Verified from `expressions.md:837`:

```
number < atom < reference < fun < port < pid < tuple < map < nil < list < bit string
```

- `nil` (`[]`) sorts as **its own kind**, *below* non-empty `list` — a genuine
  separation, not a special case.
- Tuples ordered by size, then elementwise. Maps by size, then keys (ascending),
  then values; **in map-key order, integers sort below floats**. Bit strings
  bit-by-bit, prefix is smaller.
- **`=:=`/`=/=` (term equivalence)** vs **`==`/`/=` (arithmetic)**: `1 == 1.0` is
  `true` but `1 =:= 1.0` is `false`; likewise `0.0 =:= -0.0` is `false`. Prefer
  exact equality; `==` coerces and masks type errors (DT-15).

**ADT implications, three of them:**

1. **Derived ordering** of constructor values is *automatic and total* — any two
   ADT values are comparable, and `lists:sort/1` will order them. We get `Ord`
   "for free," but it follows the term order, not constructor-declaration order
   (a tuple-tagged `Error` may sort before/after `Ok` purely by atom spelling).
   If we want declaration-order semantics, we must derive it ourselves.
2. **Derived equality** must choose `=:=` vs `==`. The `±0.0` and `1 vs 1.0`
   cases mean a payload-`float` constructor has two defensible equalities; our
   generated equality should default to `=:=` (structural identity) and document
   it.
3. The order **differs by carrier** (a native record is not a tuple), so "compare
   two ADT values" is *not* representation-independent unless we generate our own
   comparator. This is a concrete thing the **all-backend test matrix** must pin.

---

## 6. The term ↔ type bridge (synthesis with Audit 1)

| Runtime term (Audit 2) | Spec type(s) (Audit 1) | Idiomatic ADT role |
|---|---|---|
| integer | `integer()`, `N`, `Lo..Hi` | payload; enum-ish singletons/ranges |
| float | `float()` | payload (mind equality) |
| atom | `atom()`, `'foo'`, `boolean()` | **nullary constructors & tags** |
| reference | `reference()` | identity payload (rare) |
| fun | `fun(...)` | higher-order payload (rare) |
| port / pid | `port()` / `pid()` | untyped seam → `dynamic()` |
| bitstring/binary | `<<_:M,_:_*N>>`, `binary()` | text/wire payload |
| tuple | `{...}`, `tuple()` | **tagged-tuple constructor (fallback carrier)** |
| map | `#{K=>V}`, `#{K:=V}`, `map()` | open data payload; non-default carrier |
| list / nil | `[T]`, `[T,...]`, `[]` | sequence payload; recursive ADTs |
| tuple-based record | `#rec{}` (= tuple) | named-field tagged tuple (fallback) |
| **native record** | `#mod:name{}` / `#name{}` | **default constructor carrier (29+)** |

---

## 7. ADT representation analysis (the decision, grounded)

Three candidate carriers for "a constructor `C` with payload `(p1…pn)`":

| Carrier | Distinct runtime identity? | OTP | Field access | Verdict |
|---|---|---|---|---|
| **Native record** `#C{f1=…,…}` | **Yes** (`is_record` true, `is_tuple` false) | 29+ ⚠️ | named | **Default** |
| **Tagged tuple** `{'C', …}` / tuple-based record | No (`is_tuple` true; identity by convention) | any | positional / named-sugar | **Fallback** (older OTP) |
| **Map** `#{'__ctor__'=>'C', …}` | No | 18+ | named, runtime-only | Rejected as default (loses compile-time safety, non-idiomatic for sums) |
| **Singleton atom** `'C'` | n/a | any | — | **Nullary constructors**, both backends |

A **sum** is the `|` union of its constructors' carriers, plus singleton atoms
for nullary cases — exactly Erlang's existing idiom, but with our checker
enforcing *closedness* and *exhaustiveness* that the language itself cannot
(Audit 1 §5).

**Decided (Duncan):** a **pluggable backend** behind one ADT surface — **native
records default (OTP 29+)**, tagged-tuple/tuple-record **fallback** for older OTP.
The *surface* (constructor name + payload shape) is identical across carriers;
only the lowering differs. That invariant is what makes a **single test suite,
run across the full backend matrix**, a meaningful proof of equivalence — and §5
flags the two places (ordering, equality) where carrier-independence is *not*
free and the matrix must hold our feet to the fire.

**Open design sub-questions** (carried to design phase):

1. **Positional vs named constructor fields.** Native records are named-field;
   tagged tuples are naturally positional. The surface must pick one mental model
   and lower both ways. (Leaning: named fields, positional sugar.)
2. **Tag-collision prevention.** Atom/tuple tags share Erlang's global atom space
   and collide with hand-written control atoms (§8). Module-qualify tags? Native
   records solve this via captured module+name; the tuple fallback needs a
   convention.
3. **Native-record sort position** in the total term order is **not documented**
   in `expressions.md` (the ordering text predates the feature). *Unverified* —
   to be pinned empirically before relying on cross-value ordering.

---

## 8. What the runtime gives us vs what ADTs need (the gap, from the data side)

Mirrors Audit 1 §5 but framed in terms:

1. **No closed sums at runtime.** A union is just "any of these term shapes";
   nothing records that the set is complete.
2. **Predicates can false-positive** (the §1 keystone): `is_tuple/1`,
   tag-matching, and `is_map/1` all return `true` for coincidental shapes. Only
   native records (`is_record/1` ⇒ distinct) escape this.
3. **No exhaustiveness** — pattern matching over term shapes has no notion of "all
   constructors covered."
4. **Opacity is skin-deep** — the runtime never enforces abstraction (Audit 1
   §2.1); a determined consumer inspects the underlying term.
5. **Tags live in the global atom namespace**, colliding with ordinary control
   atoms and risking the unbounded-atom-table hazard if ever derived from input
   (DT-13).

Every one of these is build-tier work for our checker, not something the term
layer or Dialyzer supplies.

---

## 9. Diagnostic-surface notes (consolidated)

Representation-aware messages worth making excellent (the project's teaching
goal):

1. **Coincidental-shape warning** — "you matched a bare tuple `{ok, X}` that has
   the same shape as constructor `Ok/1` of type `Result`; did you mean to
   construct/deconstruct the ADT?" (Directly addresses the §1 hazard.)
2. **Wrong constructor arity / field** — model on the native-record
   "field not initialized" compile error (Audit 1).
3. **Carrier mismatch** — "this value is a tagged-tuple `Result`; you're matching
   it as a native-record `Result` (backend mismatch)."
4. **Equality/order surprise** — when a `float`-payload constructor is compared,
   surface the `=:=` vs `==` and `±0.0` subtlety rather than letting it bite
   silently.
5. **Tag collision** — "constructor tag `'ok'` collides with a standard Erlang
   result atom; consider qualifying."

---

## 10. Open questions carried into design

1. Positional vs named constructor fields (§7.1).
2. Tag-collision strategy for the tuple fallback (§7.2).
3. Native-record term-order position — **unverified**, pin empirically (§7.3).
4. Default derived-equality semantics (`=:=`) and whether to offer `==` variants
   for numeric payloads (§5).
5. Whether to derive `Ord`/`Eq`-style operations following term order or
   declaration order (§5.1).
6. `dynamic()` + `pid()`/`port()` as the typed/untyped seam for process- and
   port-carried ADT values (deferred soundness-across-processes problem).

---

*Sources (mounted OTP 30.0-rc0): `system/doc/reference_manual/data_types.md`,
`expressions.md` (§Term Comparisons, `:837`), `ref_man_native_records.md`,
`lib/stdlib/src/erl_parse.yrl`; erlang-guidelines `guides/04-data-and-types.md`
(DT-01…DT-19). Cross-references Audit 1 throughout.*
