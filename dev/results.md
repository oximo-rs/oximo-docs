+++
title = "Results"
description = "Inspect solver status, solutions, duals, and solution pools."
weight = 5
+++

Every oximo backend returns the same [`SolverResult`][SolverResult]. Read it
the same way independently of the solver.
It is recommended to first check what stopped the solve and whether a usable point is available, then
inspect the values relevant to your application.

## Reading results

The quickest option is the built-in report, which renders a model-aware summary:

```rust
print!("{}", result.report(&m));
```

For programmatic access, [`SolverResult`][SolverResult] separates _why the
solver stopped_ from _whether a usable point came back_. That split matters because,
for example, a run that hits a time limit can still carry a good incumbent.

```rust
let result = Highs.solve(&m, &HighsOptions::default())?;

match result.termination {
    TerminationStatus::Optimal => {
        // `objective()` is Option, since a model may have no objective.
        if let Some(obj) = result.objective() {
            println!("optimal: {obj}");
        }
    }
    TerminationStatus::Infeasible => println!("infeasible"),
    TerminationStatus::TimeLimit if result.has_solution() => {
        println!("time limit, best = {:?}", result.objective());
    }
    _ => {}
}

let x_val = result.value_of(x);            // Option<f64>
let dual  = result.dual_of(constraint_id); // Option<f64>
```

## Status

[`TerminationStatus`][TerminationStatus] says why the solver stopped:
`Optimal`, `LocallyOptimal`, `Feasible`, `Infeasible`, `Unbounded`,
`InfeasibleOrUnbounded`, `IterationLimit`, `TimeLimit`, `NodeLimit`,
`Interrupted`, `NumericError`, `NotSolved`, or `Other(String)` for an unmapped
backend status.

[`PrimalStatus`][PrimalStatus] says what you actually got: `NoSolution`,
`FeasiblePoint`, or `OptimalPoint`. `result.has_solution()` is the shorthand.
Always check it before trusting a value.

## Fields and accessors

| Item                | Type                                     | What information is available                  |
| ------------------- | ---------------------------------------- | ---------------------------------------------- |
| `termination`       | [`TerminationStatus`][TerminationStatus] | Why the solver stopped                         |
| `primal_status`     | [`PrimalStatus`][PrimalStatus]           | Whether a usable point is present              |
| `objective()`       | `Option<f64>`                            | Objective of the best solution                 |
| `value_of(expr)`    | `Option<f64>`                            | Primal value for a variable or indexed element |
| `values_of(&var)`   | iterator of `(&IndexKey, f64)`           | Every element of an indexed variable           |
| `dual_of(id)`       | `Option<f64>`                            | Shadow price (continuous models)               |
| `reduced_costs`     | map keyed by `VarId`                     | Reduced costs (continuous models)              |
| `best_bound`, `gap` | `Option<f64>`                            | Populated by branch-and-bound backends         |
| `solve_time`        | `Duration`                               | Wall-clock time in the backend                 |
| `iterations`        | `u64`                                    | Iteration count when the backend reports one   |
| `raw_log`           | `Option<String>`                         | Backend log, when captured                     |

## Constraint IDs

Constraint declarations return handles that let you query their rows after a
solve. A scalar relation returns a [`ConstraintId`][ConstraintId]. A two-sided
range returns [`RangeConstraintIds`][RangeConstraintIds], which identifies either
one interval row or its separate lower and upper rows.

Capture an indexed declaration when you need to query its rows. The returned
[`IndexedConstraint`][IndexedConstraint] maps each typed key to its registered
ID, so dual queries do not need to reconstruct generated names:

```rust
let cover: IndexedConstraint<usize> =
    constraint!(m, cover[i in 0..n_items], x[i] >= demand[i]);

let cid = cover.get(0).expect("cover row exists");
let dual = result.dual_of(cid);

for (i, cid) in cover.iter() {
    println!("cover[{i}] dual = {:?}", result.dual_of(cid));
}
```

`get(key)` also works for sparse, filtered, string, and tuple domains, and
`iter()` yields typed `(key, ConstraintId)` pairs in domain order. Handles own
their keys and IDs and remain usable after the model declaration block.

Indexed two-sided ranges return an [`IndexedRangeConstraint`][IndexedRangeConstraint].
Each key maps to `RangeConstraintIds::Interval(cid)` when the row stays a native
interval, or to `RangeConstraintIds::Split { lower, upper }` when symbolic bounds
or a nonlinear body require two rows. Match the value and query each row ID with
`result.dual_of` separately; a family can contain both forms.

```rust
let bands: IndexedRangeConstraint<usize> =
    constraint!(m, bands[i in 0..n_items], 1.0 <= x[i] <= 4.0);

for (i, ids) in bands.iter() {
    match ids {
        RangeConstraintIds::Interval(cid) => {
            println!("bands[{i}] = {:?}", result.dual_of(cid));
        }
        RangeConstraintIds::Split { lower, upper } => {
            println!("bands[{i}] lower = {:?}", result.dual_of(lower));
            println!("bands[{i}] upper = {:?}", result.dual_of(upper));
        }
    }
}
```

## Indexed variables

`values_of` walks an indexed family without a manual key loop:

```rust
for (key, value) in result.values_of(&x) {
    println!("x[{}] = {value:.2}", display_index_key(key));
}
```

## Solution pools

Backends that can return multiple points expose them, best-first:

```rust
for i in 0..result.result_count() {
    let point = result.solution(i).unwrap();
    println!("objective {:?}", point.objective);
}
```

## Next steps

- [Solvers](../solvers/): choose a backend and set its options
- [Printing & Debugging](../debugging/): inspect the model that produced a result
- [I/O](../io/): export a model for inspection in another tool

[SolverResult]: https://docs.rs/oximo/latest/oximo/prelude/struct.SolverResult.html
[TerminationStatus]: https://docs.rs/oximo/latest/oximo/prelude/enum.TerminationStatus.html
[PrimalStatus]: https://docs.rs/oximo/latest/oximo/prelude/enum.PrimalStatus.html
[Highs]: https://docs.rs/oximo/latest/oximo/solvers/struct.Highs.html
[HighsOptions]: https://docs.rs/oximo/latest/oximo/prelude/struct.HighsOptions.html
[ConstraintId]: https://docs.rs/oximo/latest/oximo/prelude/struct.ConstraintId.html
[RangeConstraintIds]: https://docs.rs/oximo/latest/oximo/prelude/enum.RangeConstraintIds.html
[IndexedConstraint]: https://docs.rs/oximo/latest/oximo/prelude/struct.IndexedConstraint.html
[IndexedRangeConstraint]: https://docs.rs/oximo/latest/oximo/prelude/struct.IndexedRangeConstraint.html
