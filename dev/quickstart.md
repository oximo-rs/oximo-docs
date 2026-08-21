+++
title = "Quickstart"
description = "Build and solve your first LP with oximo in under a minute."
weight = 2

[extra]
math = true
+++

This walkthrough builds a small linear program end-to-end. HiGHS is an opt-in
backend, so enable it in the project before running this example:

```bash
cargo add oximo --features highs
```

## The problem

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

Run it with `cargo run`.

## Step by step

`Model::new` creates the model container. The `variable!` macro registers the
variables and their bounds, while `constraint!` adds relations written with
`<=`, `>=`, or `==`. Finally, `objective!` declares the expression to maximize.

Every backend implements the same [`Solver`][Solver] trait, so switching from
HiGHS to another compatible backend changes only the solver type and options.
For a C-free continuous LP/QP/SOCP path, enable Clarabel instead:

```bash
cargo add oximo --features clarabel
```

See [Modeling](/modeling/) for indexed variables and nonlinear expressions,
[Solvers](/solvers/) for backend capabilities, and [Results](/results/) for
status, values, duals, and reduced costs.

[Solver]: https://docs.rs/oximo/latest/oximo/prelude/trait.Solver.html
