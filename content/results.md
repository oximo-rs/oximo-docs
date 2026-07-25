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

- [Solvers](/solvers/): choose a backend and set its options
- [Printing & Debugging](/debugging/): inspect the model that produced a result
- [I/O](/io/): export a model for inspection in another tool

[SolverResult]: https://docs.rs/oximo/latest/oximo/prelude/struct.SolverResult.html
[TerminationStatus]: https://docs.rs/oximo/latest/oximo/prelude/enum.TerminationStatus.html
[PrimalStatus]: https://docs.rs/oximo/latest/oximo/prelude/enum.PrimalStatus.html
[Highs]: https://docs.rs/oximo/latest/oximo/solvers/struct.Highs.html
[HighsOptions]: https://docs.rs/oximo/latest/oximo/prelude/struct.HighsOptions.html
