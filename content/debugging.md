+++
title = "Printing & Debugging"
description = "Print an oximo model as readable algebra and track down what it actually says."
weight = 6
+++

Macros that generate families of constraints are convenient right up until the model doesn't say what you thought it said. [`Model`][Model] implements `Display`, so the fastest way to check is to print it.

## Printing

You can print the complete model as a readable algebra block to the console with `println!("{m}");`.

```rust
let m = Model::new("diet");
variable!(m, x >= 0.0);
variable!(m, y >= 0.0);
constraint!(m, c1, x + 2.0 * y <= 14.0);
constraint!(m, c2, 3.0 * x - y >= 0.0);
objective!(m, Min, 3.0 * x + 4.0 * y);

println!("{m}");
```

```txt
Model 'diet' (LP)
min 3 x + 4 y
s.t.
  c1: x + 2 y <= 14
  c2: 3 x - y >= 0
vars
  x >= 0
  y >= 0
```

The header carries the inferred [`ModelKind`][ModelKind], so a single print answers both "did my constraints come out right?" and "why is this backend refusing my model?". Ranges, cones, and domains all render in the algebraic form you wrote:

```txt
max x + y
  band: 1 <= x + t <= 4
  disk: ||x, t|| <= x + 1
  0 <= y <= 1, binary
```

## Debugging

Printing a 10,000-constraint model is not debugging. Display adapters render a single piece, which is what you want inside an assertion or a targeted `dbg!`:

| Adapter                           | Renders               |
| --------------------------------- | --------------------- |
| [`display_expr(e)`][d_expr]       | A bare [`Expr`][Expr] |
| [`display_constraint(id)`][d_con] | One named constraint  |
| [`display_objective()`][d_obj]    | The objective         |
| [`display_soc(id)`][d_soc]        | One second-order cone |

Look an id up by name with [`constraint_id`][cid] (or `soc_constraint_id`), then render it:

```rust
let c = m.constraint_id("c").unwrap();
assert_eq!(m.display_constraint(c).to_string(), "c: -y + x * y <= 3");
assert_eq!(m.display_expr(2.0 * x - y).to_string(), "2 x - y");
```

Because these return `Display` adapters rather than `String`, they're cheap to leave in test assertions. The rendering only happens when something formats them.

## Rule-generated names

Indexed constraints are auto-named `base[key]`, and that name is exactly what `constraint_id` expects. This is the usual way to confirm a [rule](/modeling/#rule-style-constraints) expanded over the keys you intended:

```rust
constraint!(m, supply[p in plants], sum!(x[p, q] for q in markets) <= supply_of(&p));

let c = m.constraint_id("supply[seattle]").unwrap();
println!("{}", m.display_constraint(c));
```

To render a key the same way the auto-namer does, use [`display_index_key`][dik].

## Parameters show their current binding

A printed model resolves [parameters](/modeling/#parameters) to whatever they're bound to _right now_, which makes it easy to confirm a scenario sweep is re-binding what you think:

```rust
param!(m, price = 4.0);
objective!(m, Min, price * x);

println!("{m}"); // min 4 x   ... params: price = 4

m.set_param(price, 7.5);
println!("{m}"); // min 7.5 x ... params: price = 7.5
```

## Checking the model kind

oximo infers the [`ModelKind`][ModelKind] from the expressions you write. You never declare it. A backend that rejects your model is usually telling you an expression is a different kind than you assumed, so you can pin it down in a test:

```rust
assert_eq!(m.kind(), ModelKind::LP);
```

Compare the result against the [backend table](/solvers/#choosing-a-backend) to see which solvers accept it.

## Diagnosing infeasibility

Sometimes the backend accepts the model's kind and still returns _infeasible_. Each constraint is fine on its own, but together they can't all hold. Backends built on a solver with a native conflict refiner can express why by computing an **irreducible infeasible subsystem (IIS)**, a minimal set of constraints and variable bounds that are jointly infeasible, where dropping any one member makes the rest feasible.

Call [`compute_iis`][iis_compute] from the [`InfeasibilityDiagnosis`][infd] trait, then render the result against the model with [`Iis::report`][iis_report].

```rust
use oximo::solvers::Gurobi;

let m = Model::new("iis");
variable!(m, x >= 0.0);
constraint!(m, floor, x >= 2.0);
constraint!(m, ceil, x <= 1.0);
objective!(m, Min, x);

let iis = Gurobi.compute_iis(&m, &GurobiOptions::default())?;
println!("{}", iis.report(&m));
```

```txt
irreducible infeasible subsystem (2 members)

constraints (2)
  floor
  ceil
```

The report uses the model's own names, so `floor` and `ceil` point straight back to the `constraint!` lines that conflict. The `x >= 0` bound isn't part of the conflict, so it isn't listed. Reach the members programmatically through [`Iis`][iis]'s `constraints`, `soc_constraints`, and `var_bounds` fields (each bound tagged [`VarBoundKind`][vbk] `Lower`/`Upper`). `compute_iis` returns a [`SolverError`][SolverError] if the model turns out feasible, so it doubles as an "is this actually infeasible?" assertion.

> For more details on IIS support, see [Solvers > Requirements & capabilities](/solvers/#requirements-capabilities).

## When printing isn't enough

| Error                         | Debugging steps                                                                          |
| ----------------------------- | ---------------------------------------------------------------------------------------- |
| Backend rejects the model     | Check [`Model::kind()`][Model] against the [backend table](/solvers/#choosing-a-backend) |
| Model is too big to read      | Export it with [`io::write_lp`](/io/) (if linear)                                        |
| Solver returns nothing usable | Inspect `termination` and `primal_status`, not just `objective()`                        |
| Model reports infeasible      | Compute an [IIS](#diagnosing-infeasibility) (if supported by backend)                    |
| Need the backend's own logs   | Set `.verbose(true)` in the options, or read `result.raw_log`                            |

After a solve, [`result.report(&m)`](/results/#reading-results) pairs the model with its solution in one print-out.

## Next steps

- [I/O](/io/): export your model to MPS, LP, or NL for inspection in other tools
- [Modeling](/modeling/): back to variables, sets, and rule-style constraints

[Model]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html
[ModelKind]: https://docs.rs/oximo/latest/oximo/prelude/enum.ModelKind.html
[Expr]: https://docs.rs/oximo/latest/oximo/prelude/struct.Expr.html
[d_expr]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html#method.display_expr
[d_con]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html#method.display_constraint
[d_obj]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html#method.display_objective
[d_soc]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html#method.display_soc
[cid]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html#method.constraint_id
[dik]: https://docs.rs/oximo/latest/oximo/prelude/fn.display_index_key.html
[infd]: https://docs.rs/oximo/latest/oximo/prelude/trait.InfeasibilityDiagnosis.html
[iis_compute]: https://docs.rs/oximo/latest/oximo/prelude/trait.InfeasibilityDiagnosis.html#tymethod.compute_iis
[iis]: https://docs.rs/oximo/latest/oximo/prelude/struct.Iis.html
[iis_report]: https://docs.rs/oximo/latest/oximo/prelude/struct.Iis.html#method.report
[vbk]: https://docs.rs/oximo/latest/oximo/prelude/enum.VarBoundKind.html
[SolverError]: https://docs.rs/oximo/latest/oximo/prelude/enum.SolverError.html
