# Getting Started with `lfe/typed`

> A guide to adding static types and ADTs to your LFE project.

## Prerequisites

- Erlang/OTP 28+
- LFE 2.2.1+
- Rust toolchain (for building the checker)
- rebar3

## Project Setup

1. Clone `lfe/typed` alongside your project.
2. Build the Rust checker:

```sh
cd typed/checker && cargo build
```

3. Add `typed` as a dependency in your `rebar.config` and ensure LFE is available.

## Writing a Typed Module

Typed LFE files use the `.lfet` extension. A typed module looks like this:

```lisp
(defmodule orders
  (export (status-label 1)
          (decode-order-status 1)
          (validate-order-status 2)))

;; Define an algebraic data type
(deftype order-status
  (repr tagged-tuple)
  (Pending)
  (Shipped (tracking string))
  (Cancelled (reason string)))

;; A typed function with a contract
(defun/typed status-label
  :args ((s order-status))
  :returns string
  :body (case/typed s
          ((Pending) "pending")
          ((Shipped t) (++ "shipped: " t))
          ((Cancelled r) (++ "cancelled: " r))))
```

### Key forms

- **`deftype`** declares an algebraic data type with named-field constructors.
  Optional `(repr <backend>)` clause selects the runtime representation.
- **`defun/typed`** declares a function with a type contract (`:args`, `:returns`,
  `:body`). The checker verifies the body against the contract.
- **`case/typed`** is exhaustive pattern matching over an ADT. The checker rejects
  non-exhaustive matches, naming every missing constructor.
- **Constructor calls** like `(Shipped :tracking "TRK123")` construct ADT values.

### Type annotations

Types in `:args` and `:returns` can be:
- **Built-in:** `integer`, `float`, `number`, `atom`, `boolean`, `binary`, `string`,
  `list`, `map`, `dynamic`
- **ADT names:** any `deftype`-declared type (e.g. `order-status`)

### Representation backends

| Backend | When | Runtime shape |
|---------|------|--------------|
| `tagged-tuple` (default <29) | General sum types | `{tag, field1, ...}` |
| `enum` | All-nullary sums | atoms |
| `transparent` | Single-constructor newtypes | the payload itself |
| `native-record` (29+) | True distinct type | `#Ctor{...}` |

### Generated functions

For each `deftype`, the checker generates:
- `validate-<typename>/2` — deep recursive validator
- `decode-<typename>/1` — graceful `dynamic → T` boundary

**You must manually export these** in your module's `(export ...)`.

## Typed Records (`defrecord/typed`)

Records are single-constructor ADTs with named fields and generated accessors:

```lisp
(defmodule inventory
  (export (make-item 3) (item-name 1) (item-qty 1) (item-price 1)
          (set-item-qty 2)
          (validate-item 2) (decode-item 1)))

(defrecord/typed item
  (name string)
  (qty integer)
  (price integer))

(defun/typed restock
  :args ((it item) (n integer))
  :returns item
  :body (set-item-qty it (+ (item-qty it) n)))
```

### Generated functions

For each `defrecord/typed`:
- `make-<rec>/N` — constructor (one arg per field, in declared order); each arg
  guarded by its field type
- `<rec>-<field>/1` — accessor for each field
- `set-<rec>-<field>/2` — functional updater (returns a **new** record); the new
  value is guarded by the field type

All field-type guards use the M4 always-on posture: wrong type raises
`{type_error, #{expected => T, got => V, ...}}`.

### Type-aware accessors

The checker synthesizes accessor return types from the record definition:
`(item-qty it)` is known to return `integer`, not `dynamic`. This enables
full type checking of functions that use record accessors.

## Cross-Module Types

Types declared in one `.lfet` module can be referenced from another. The checker
scans sibling `.lfet` files in the same directory, building a combined type
registry before checking any module.

### Qualified `mod:type`

Use `module-name:type-name` in type positions:

```lisp
;; orders_web.lfet — consumes the `order` record from orders.lfet
(defmodule orders_web
  (export (get-order-total 1)))

(defun/typed get-order-total
  :args ((o orders:order))
  :returns integer
  :body (element 4 o))
```

### `import-types`

Import types by name to use bare references:

```lisp
(defmodule orders_web
  (export (get-order-total 1)))

(import-types (from orders (order)))

(defun/typed get-order-total
  :args ((o order))
  :returns integer
  :body (element 4 o))
```

Both forms produce the same result. The checker statically rejects:
- Unknown module: `bogus:foo` → *"unknown module `bogus`"*
- Unknown type in a known module: `orders:nonexistent` → *"unknown type `nonexistent` in module `orders`"*
- Bad import: `(import-types (from orders (nonexistent)))` → same

## Running the Checker

```sh
# Check a single file (auto-scans sibling .lfet for cross-module types)
typed/checker/target/debug/typed-check your-module.lfet --output your-module.eetf

# Then compile through the driver
erl -noshell -pa typed/ebin -pa lfe/ebin -eval '
  {ok, Bin} = file:read_file("your-module.eetf"),
  Forms = binary_to_term(Bin),
  typed_driver:compile_forms(Forms, "your-module.lfet", "."),
  halt().
'
```

## Reading Type Errors

### Compile-time: wrong return type

```
error[E001]: body returns `integer`, but contract declares `:returns binary`
  --> greeting.lfet:3:1
     |
   3 | (defun/typed oops
     | ^
```

### Compile-time: non-exhaustive match

```
error[E100]: non-exhaustive pattern match on type `order-status`
  --> orders.lfet:10:1
      |
   10 | (case/typed s
      | ^
   |
   = These values are not matched:
       - Shipped
       - Cancelled
   = Hint: add clauses for the missing constructor(s), or use `_` as a catch-all.
```

### Runtime: wrong-typed argument (guard crash)

```erlang
{type_error, #{expected => integer, got => "hello",
               function => double, arg => 1, path => []}}
```

### Runtime: invalid decode input (graceful error)

```erlang
{error, {type_error, #{expected => string, got => 999,
                        path => [tracking]}}}
```

Use `typed_rt:render_type_error/1` to turn either into a teaching-grade string:

```
type error: expected string at .tracking, got 999
```

## The `dynamic` Boundary

Calls to untyped/unknown functions synthesize `dynamic`. `dynamic` is compatible
with any expected type — the gradual escape hatch. This means typed code can call
untyped Erlang/LFE freely; the checker just can't verify those calls.

Runtime enforcement at the boundary uses `decode-<typename>/1`:

```lisp
;; At your HTTP handler / message boundary:
(case (orders:decode-order-status user-input)
  (#(ok status) (process-order status))
  (#(error te) (respond-400 (typed_rt:render_type_error te))))
```

## Current Limitations

See `docs/design/M5-gap-inventory.md` for the full list. Key ones:
- No cross-module *function* imports (types cross modules; functions don't yet)
- No `when` guards in `case/typed` patterns
- No `let` type annotations (bindings typed by synthesis)
- Binary literals (`#"..."`) not yet supported by the parser
- Native-record backend needs OTP 29+
