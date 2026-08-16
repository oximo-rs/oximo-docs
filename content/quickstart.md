+++
title = "Quickstart"
description = "Build and solve your first LP with oximo in under a minute."
weight = 2

[extra]
math = true
+++

This walkthrough builds a small linear program end-to-end: variables, constraints, objective, solve, read result. By the end you'll have run [`Highs`][Highs] on a real model and inspected the solution.

## The problem

<div>
\[
\begin{aligned}
\max \quad & 3x + 4y \\
\text{s.t.} \quad & x + 2y \le 14 \\
                  & 3x \ge y \\
                  & x \le y + 2 \\
                  & x \ge 0 \\
                  & 0 \le y \le 4
\end{aligned}
\]
</div>

## The full program

```rust
use oximo::prelude::*;
use oximo::solvers::Highs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("transport");

    variable!(m, x >= 0.0);
    variable!(m, 0.0 <= y <= 4.0);

    constraint!(m, c1, x + 2.0 * y <= 14.0);
    constraint!(m, c2, 3.0 * x >= y);
    constraint!(m, c3, x <= y + 2.0);
    objective!(m, Max, 3.0 * x + 4.0 * y);

    let result = Highs.solve(&m, &HighsOptions::default())?;
    println!("obj = {:?}", result.objective()); // Some(34.0)
    println!("x   = {:?}", result.value_of(x)); // Some(6.0)
    println!("y   = {:?}", result.value_of(y)); // Some(4.0)
    Ok(())
}
```

Run it with `cargo run` and you should see the optimum: `obj = Some(34.0)`.

## Step by step

### 1. Create a model

```rust
let m = Model::new("transport");
```

[`Model`][Model] is the container that holds variables, constraints, and an objective. The name is used when exporting or printing it and as a label in solver logs.

### 2. Declare variables

```rust
variable!(m, x >= 0.0);
variable!(m, 0.0 <= y <= 4.0);
```

The [`variable!`][variable] macro registers a variable on `m` and binds a Rust
binding of the same name, so you can use `x` directly in later expressions. Write
the bounds the way you'd write them on paper: `x >= 0.0`, `0.0 <= y <= 4.0`, or
just `variable!(m, z)` for a free variable. See [Modeling](/modeling/) for
integer, binary, and indexed variables.

### 3. Add constraints

```rust
constraint!(m, c1, x + 2.0 * y <= 14.0);
```

[`constraint!`][constraint] takes the model, a name, and a relation written with
`<=`, `>=`, or `==`. Expressions use standard Rust operators.

### 4. Set an objective

```rust
objective!(m, Max, 3.0 * x + 4.0 * y);
```

[`objective!`][objective] takes a sense (`Max` or `Min`, also `Maximize`/`max`,
`Minimize`/`min`) and the expression. A [`Model`][Model] has one objective.

If your problem is a feasibility problem, you can use `Feas`/`Feasibility` as the sense.

### 5. Solve

```rust
let result = Highs.solve(&m, &HighsOptions::default())?;
```

Each backend implements the [`Solver`][Solver] trait, so switching engines is a
one-line change. [`Highs.solve(&model, &options)`][Highs] returns a
[`SolverResult`][SolverResult].

### 6. Read the result

```rust
println!("obj = {:?}", result.objective());
println!("x   = {:?}", result.value_of(x));
```

[`SolverResult::objective()`][SolverResult] returns `Option<f64>`, `None` when
the model has no objective or no solution was found.
[`value_of(expr)`][SolverResult] gives a variable's value in the best solution.

For a quick human-readable dump of the whole solution, print the built-in report:

```rust
print!("{}", result.report(&m));
```

For status checking, duals, and reduced costs, see [Results](/results/).

For more examples, see the [examples](/examples/) directory.

## Next steps

- [Modeling](/modeling/): variables, index sets, indexed variables, summation, rule-style constraints
- [Solvers](/solvers/): backend options and result inspection
- [Printing & Debugging](/debugging/): print a model as algebra when it doesn't say what you expected
- [I/O](/io/): export models to MPS, LP, or NL

[Model]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html
[Solver]: https://docs.rs/oximo/latest/oximo/prelude/trait.Solver.html
[SolverResult]: https://docs.rs/oximo/latest/oximo/prelude/struct.SolverResult.html
[Highs]: https://docs.rs/oximo/latest/oximo/solvers/struct.Highs.html
[HighsOptions]: https://docs.rs/oximo/latest/oximo/prelude/struct.HighsOptions.html
[variable]: https://docs.rs/oximo/latest/oximo/prelude/macro.variable.html
[constraint]: https://docs.rs/oximo/latest/oximo/prelude/macro.constraint.html
[objective]: https://docs.rs/oximo/latest/oximo/prelude/macro.objective.html
[examples]: https://github.com/oximo-rs/oximo/tree/main/crates/oximo/examples
