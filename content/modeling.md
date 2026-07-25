+++
title = "Modeling"
description = "Variables, index sets, expressions, and constraints in oximo."
weight = 3
+++

oximo's modeling layer lets you describe an optimization problem with idiomatic Rust.
You write models using declarative macros that mirror algebraic notation, set algebra for indices, operator-overloaded expressions, and rule-style constraint generation.

Everything on this page is re-exported from [`oximo::prelude`][prelude].

## Variables

Variables are the quantities a solver may choose. [`variable!`][variable]
registers one on the model and binds a Rust variable of the same name, so the
symbol is immediately usable in later expressions. Bounds are written the way
you'd write them on paper.

```rust
use oximo::prelude::*;

let m = Model::new("my_model");

variable!(m, x >= 0.0);                 // continuous, x ≥ 0
variable!(m, 0.0 <= y <= 10.0);         // continuous, 0 ≤ y ≤ 10
variable!(m, z);                        // free, unbounded by default
variable!(m, b, Bin);                   // binary {0, 1}   (also Binary)
variable!(m, n >= 0.0, Int);            // general integer (also Integer)
variable!(m, s <= 10.0, SemiCont(2.0)); // semicontinuous: 0 or in [2, 10]
```

Bounds, domain, warm start, and fixing can also be passed as keyword arguments after the name:

```rust
variable!(m, x, lb = 0.0, ub = 1.0);       // same as `0.0 <= x <= 1.0`
variable!(m, n, lb = 0.0, domain = Int);   // keyword domain
variable!(m, w, lb = 0.0, ub = 10.0, Int); // mixed with a positional domain token
variable!(m, p, lb = 0.0, initial = 3.0);  // warm start (scalar only)
variable!(m, q, fix = 5.0);                // fixed to 5.0 (scalar only)
```

| Domain token  | Meaning                           |
| ------------- | --------------------------------- |
| _(none)_      | Continuous                        |
| `Bin`         | Binary `{0, 1}` (alias `Binary`)  |
| `Int`         | General integer (alias `Integer`) |
| `SemiCont(t)` | Zero, or continuous in `[t, ub]`  |
| `SemiInt(t)`  | Zero, or integer in `[t, ub]`     |

## Parameters

A parameter is data that can change while the model structure stays fixed. It
stays _symbolic_ in the model, so `param!` lets you build once and re-bind its
value between solves without rebuilding variables, constraints, or the
objective. This is useful for scenario sweeps.

```rust
param!(m, p1 = 0.0);

variable!(m, x1 >= 0.0);
objective!(m, Max, p1 * x1);

for price in [1.0, 1.6, 2.0] {
    p1.set_param_value(price);
    let result = Highs.solve(&m, &HighsOptions::default())?;
    println!("{price} -> {:?}", result.objective());
}
```

A parameter times a variable stays linear, so the model kind is unchanged by re-binding.

For parameter sweeps, it is recommended to use a `Persistent` solver if available. See [Solvers](/solvers/) for details.

## Index sets

A [`Set`][Set] is the modeling-layer container for an ordered, finite index set
over integers, strings, or tuples. Use one to name the domain of an indexed
variable, a generated constraint, or a sum.

Most domains need no explicit [`Set`][Set], since an integer range is already
a domain (`x[i in 0..5]`, `sum!(… for i in 0..n)`).
Reach for [`Set`][Set] when keys are strings, tuples, sparse, or a
subset reused across statements.

The [`set!`][set-macro] macro binds a named set. A plain right side is normalized to an owned set, while a `pat in domain [if cond]` comprehension builds and optionally filters one.

```rust
use oximo::prelude::*;

let plants = Set::strings(["seattle", "san-diego"]);

set!(items = 0..5);             // range normalized to Set<usize>
set!(routes = plants * plants); // Cartesian product

// Comprehension: product domain + by-value `if`. These two are equivalent.
set!(arcs = (p, q) in &plants * &plants if p != q); // single tuple pattern
set!(arcs = i in plants, j in plants if i != j);    // multi-bind product

// The typed filter is also a Set method (the receiver pins the key type):
let diag = (&plants * &plants).filter_typed(|(p, q)| p == q);

// Sparse / string leaf sets:
let sparse = Set::from_ints([0, 2, 4, 8]);
```

## Indexed variables

Many optimization quantities vary by period, product, or route. The indexed
form, `variable!(m, x[k in set])`, registers one scalar per key and auto-names
it like `x[seattle,nyc]`. Bounds apply uniformly by default. A multi-index
family ranges over a Cartesian product.

```rust
let m = Model::new("transport");

variable!(m, x[r in routes] >= 0.0);        // one var per route
variable!(m, y[k in items] >= 0.0, Int);    // integer family
variable!(m, z[a in rows, b in cols], Bin); // multi-index (Cartesian product)

// Scalar lookup: any type that converts to IndexKey works.
let e1 = x[("seattle", "nyc")];
let e2 = z[a, b];

// Per-key bounds may reference the index.
variable!(m, 0.0 <= w[(p, q) in routes] <= capacity_for(&p, &q));
variable!(m, v[k in items], lb = 0.0, ub = cap[k]);

// Filtered family: keep only matching keys (no trivial elements built).
variable!(m, d[(i, j) in rc if i == j] >= 0.0);
```

`x[key]` returns the [`Expr`][Expr] for that element, ready to drop into a constraint or objective.

## Expressions

[`Expr`][Expr]s are built with standard Rust operators. Scalars are `f64`.

```rust
let lhs = x + 2.0 * y - 3.0 * z;
let rhs = 4.0 * x + 5.0;
```

Behind the scenes oximo uses arena-allocated expression trees ([`oximo-expr`][oximo-expr]), so combining large [`Expr`][Expr]s stays cheap.

## Summing over sets

When an expression has one term per key, use `sum!` instead of building a Rust
loop. It produces one [`Expr`][Expr] that can go anywhere an expression is
accepted.

`sum!(body for k in set)` reads as \\(\sum_{k \in \text{set}} \text{body}\\).

```rust
// Single sum: sum over i in items of weights[i] * x[i]
constraint!(m, cap, sum!(weights[i] * x[i] for i in items) <= capacity);

// Double sum, flat: sum over (p, q) in plants x markets
let total_cost = sum!(c[p, q] * x[p, q] for p in plants, q in markets);

// Filtered sum.
let active = sum!(x[i] for i in 0..n if online[i]);
```

For the plain dot-product case there is also [`dot`][dot].

## Constraints

Constraints define which combinations of variable values are allowed. Give each
one a stable name for readable solver output and diagnostics, then use
[`constraint!`][constraint] with a relation written as `<=`, `>=`, or `==`.
A two-sided range becomes a single constraint.

```rust
constraint!(m, cap, 2.0 * x + 3.0 * y <= 100.0);
constraint!(m, demand, x >= 5.0);
constraint!(m, balance, x - y == 0.0);
constraint!(m, band, 1.0 <= x + y <= 10.0); // two-sided range -> one constraint
```

For SOC constraints, use the [`soc_constraint!`][soc_constraint] macro.

## Objectives

A [`Model`][Model] has exactly one objective function, the quantity it minimizes or
maximizes among feasible solutions. Add it after defining the expressions it
uses, with the sense written next to the model.

```rust
objective!(m1, Min, 3.0 * x + 5.0 * y);
objective!(m2, Max, x + 2.0 * y); // also Minimize/min, Maximize/max
```

If you have a feasibility problem, use the `feas`/`Feasibility` sense.

## Rule-style constraints

Use the indexed form of [`constraint!`][constraint] when the same rule applies
across a set. It emits one constraint per key, auto-named like
`supply[seattle]`, without an explicit Rust loop. A trailing `if` filters the
keys, and `name = expr` gives a computed run-time name.

```rust
// Scalar set: one constraint per period.
let periods = Set::range(0..T);
constraint!(m, setup[t in periods], x[t] <= capacity * s[t]);

// Tuple set + inner sum builds the LHS expression (key types inferred).
constraint!(m, supply[p in plants], sum!(x[p, q] for q in markets) <= supply_of(&p));

// Filtered family: only the keys passing the guard are built.
constraint!(m, diag[(i, j) in arcs if i == j], x[i, j] <= 1.0);

// Computed run-time name.
constraint!(m, name = format!("bal_{p}"), inflow[p] - outflow[p] == 0.0);
```

## Nonlinear expressions

`Pow`, `Sin`, `Cos`, `Exp`, `Log`, `Abs`, and bilinear products are first-class,
so you can write nonlinear algebra in the same expressions as linear terms.
The model's kind is inferred from what you write.

```rust
// Rosenbrock NLP
objective!(m1, Min, (1.0 - x).powi(2) + 100.0 * (y - x.powi(2)).powi(2));

// Quadratic constraint (model kind: QCP)
constraint!(m2, disk, x.powi(2) + y.powi(2) <= 1.0);

// Second-order cone ||(x, y)|| <= t (model kind: SOCP)
soc_constraint!(m3, cone, [x, y] <= t);

// Transcendental utility (MINLP when any variable is integer/binary)
objective!(m4, Max, sum!(u[i] * (1.0 + w[i] * x[i]).log() for i in items));
```

Check the inferred kind with [`Model::kind()`][Model], which returns a [`ModelKind`][ModelKind]. Backends reject kinds they don't support. See [Printing & Debugging](/debugging/#checking-the-model-kind).

## Next steps

- [Solvers](/solvers/): pick a backend, set options, read the solution
- [Printing & Debugging](/debugging/): print a model as algebra and check what it actually says
- [I/O](/io/): export your model to MPS, LP, or NL

[prelude]: https://docs.rs/oximo/latest/oximo/prelude/index.html
[Model]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html
[ModelKind]: https://docs.rs/oximo/latest/oximo/prelude/enum.ModelKind.html
[Set]: https://docs.rs/oximo/latest/oximo/prelude/struct.Set.html
[Expr]: https://docs.rs/oximo/latest/oximo/prelude/struct.Expr.html
[dot]: https://docs.rs/oximo/latest/oximo/prelude/fn.dot.html
[variable]: https://docs.rs/oximo/latest/oximo/prelude/macro.variable.html
[constraint]: https://docs.rs/oximo/latest/oximo/prelude/macro.constraint.html
[set-macro]: https://docs.rs/oximo/latest/oximo/prelude/macro.set.html
[oximo-expr]: https://docs.rs/oximo-expr/latest/oximo_expr/
[soc_constraint]: https://docs.rs/oximo/latest/oximo/prelude/macro.soc_constraint.html
