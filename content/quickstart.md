+++
title = "Quickstart"
description = "Build, solve, and inspect a small linear program with oximo."
weight = 2

[extra]
math = true
+++

This guide builds a small linear program one piece at a time. You’ll declare
variables, add constraints and an objective, then solve the model and inspect
the result. For a full setup walkthrough, see [Installation](../installation/).

## 1. Create a project and enable a solver

This example requires Rust 1.89 or later. Create a project first:

```bash
cargo new lp-model --edition 2024
cd lp-model
```

> **Important:** `cargo add oximo` by itself does not enable a solver. Add oximo with a backend feature so this program can solve the model. Choose one of the following commands and run it from the project directory.

### HiGHS

HiGHS is a bundled solver for LP, MILP, and QP. It needs a C/C++ compiler at
build time:

```bash
cargo add oximo --features highs
```

### Clarabel

If you don’t have a C/C++ compiler, use Clarabel instead. It is pure Rust and
solves continuous LP, QP, and SOCP models, but does not solve mixed-integer
models:

```bash
cargo add oximo --features clarabel
```

The modeling code is the same with either backend. See [Solvers](../solvers/)
for the full backend list and compatibility comparison.

## 2. Understand the model

We’ll maximize the value of two decision variables, `x` and `y`, subject to a
few constraints:

$$
\begin{aligned}
\max \quad & 3x + 4y \\\\
\text{s.t.} \quad & x + 2y \le 14 \\\\
                  & 3x \ge y \\\\
                  & x \le y + 2 \\\\
                  & x \ge 0 \\\\
                  & 0 \le y \le 4
\end{aligned}
$$

The objective rewards larger values of `x` and `y`. The constraints limit their
combination, require `x` to be nonnegative, and keep `y` between zero and four.

## 3. Start a Rust program

Open `src/main.rs`. The following lines create the model. This version uses
HiGHS; if you chose Clarabel, use the alternative shown in [Choose the solver](#choose-the-solver).

```rust
use oximo::prelude::*;
use oximo::solvers::Highs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("example LP");
```

The `prelude` brings in the modeling macros and common types. `Model::new`
creates an empty model; the `?` in `main` lets errors from solving and querying
results propagate to the caller.

## 4. Declare decision variables

To create the variables, add these lines inside `main`, after `Model::new`:

```rust
    variable!(m, x >= 0.0);
    variable!(m, 0.0 <= y <= 4.0);
```

Each `variable!` call registers a variable on the model and makes its name
available as an expression in Rust. Bounds are part of the declaration, so the
solver cannot return a value outside them.

## 5. Add constraints and an objective

Next, add the three relationships and tell oximo what to maximize:

```rust
    constraint!(m, capacity, x + 2.0 * y <= 14.0);
    constraint!(m, ratio, 3.0 * x >= y);
    constraint!(m, balance, x <= y + 2.0);

    objective!(m, Max, 3.0 * x + 4.0 * y);
```

A constraint has a name followed by an algebraic relation. The names make solver output easier to read later. `objective!` records the expression and direction.

## 6. Solve and inspect the result

To query the results, we can use print statements, then close `main`:

```rust
    let result = Highs.solve(&m, &HighsOptions::default())?;
    println!("objective = {:?}", result.objective());
    println!("x = {:?}", result.value_of(x)?);
    println!("y = {:?}", result.value_of(y)?);

    Ok(())
}
```

Run the program from the project directory:

```bash
cargo run
```

The optimum is `x = 6`, `y = 4`, with objective value `34`. The
result accessors return `Option` values because a solve might not produce a
usable solution; for real applications, check the solver status before relying
on them. See [Results](../results/) for status handling and more result queries.

### Choose the solver

If you installed Clarabel instead of HiGHS, change the solver import and solve
line in `main.rs`:

```rust
use oximo::solvers::Clarabel;

let result = Clarabel.solve(&m, &ClarabelOptions::default())?;
```

Remove `use oximo::solvers::Highs;` and replace the HiGHS solve line with the Clarabel line. All the model declarations stay the same. Both backends implement the same [`Solver`][Solver] interface.

## Next steps

- [Modeling](../modeling/): indexed variables, sets, parameters, and nonlinear expressions
- [Solvers](../solvers/): compare backends and configure solver options
- [Results](../results/): check termination status, values, duals, and reduced costs

[Solver]: https://docs.rs/oximo/latest/oximo/prelude/trait.Solver.html
