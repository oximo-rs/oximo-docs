+++
title = "Installation"
description = "Add oximo to a Rust project and configure an optional solver backend."
weight = 1
+++

For most projects, installation is one command. oximo includes the [`Highs`][Highs]
solver by default, so you can build and solve linear, mixed-integer linear, and
quadratic models without installing a solver or obtaining a license.

> This site is a guide for tutorials and worked examples. For the complete API
> reference, see [docs.rs/oximo](https://docs.rs/oximo).

## Start here

oximo requires Rust **1.85** or later (edition 2024). Install Rust with
[rustup](https://rustup.rs/) if you do not already have it.

Create a project and add oximo:

```bash
cargo new my-oximo-model --edition 2024
cd my-oximo-model
cargo add oximo
```

That is enough to use [`Highs`][Highs] and write models to MPS, LP, or NL files.
The bundled HiGHS build needs a C compiler:

- On Windows, install the MSVC C++ build tools,
- on macOS, run `xcode-select --install`,
- on Linux, install your distribution's standard C/C++ build tools.

Next, follow the [Quickstart](/quickstart/) to create and solve your first
model. If the project builds and the Quickstart prints an objective value, your
installation is ready.

## Choose another backend

If you need another backend, you can use the provided features flags.

The [Solvers][Solvers] guide compares model-kind support, shows the different
requirements of each backend, and covers [external solver setup](/solvers/#external-solver-setup).
It is the best place to choose an engine when you are unsure.

For example, to add the Clarabel backend:

```bash
cargo add oximo --features clarabel --no-default-features
```

Or, in `Cargo.toml`:

```toml
[dependencies]
oximo = { version = "0.5", features = ["clarabel"], default-features = false }
```

## Minimal builds

The default `highs` and `io` features are convenient for most users. If you want
to avoid compiling bundled HiGHS, turn them off explicitly and select the backend you want:

```toml
[dependencies]
oximo = { version = "0.5", default-features = false, features = ["clarabel"] }
```

With no solver feature, you can still construct models. You will need a solver
feature before solving them.

## Advanced: exact nonlinear derivatives

`pounce` uses finite-difference derivatives for nonlinear expressions by
default. The nightly-only `pounce-enzyme` feature provides exact gradients,
Jacobians, and Hessians through [Enzyme](https://enzyme.mit.edu/). Use it for
nonlinear POUNCE models when your toolchain can support it. **It is highly
recommended to use it.**

It requires a nightly Rust toolchain with the `enzyme` component,
`RUSTFLAGS="-Zautodiff=Enable"`, and a fat-LTO profile. The oximo workspace
provides an `enzyme` profile:

```bash
RUSTFLAGS="-Zautodiff=Enable" cargo +nightly build --profile enzyme --features pounce-enzyme
```

See the [Enzyme installation instructions](https://rustc-dev-guide.rust-lang.org/autodiff/installation.html)
for configuring the toolchain.

## Feature reference

| Feature         | Included by default | Purpose                                                              |
| --------------- | :-----------------: | -------------------------------------------------------------------- |
| `highs`         |         yes         | Bundled [`Highs`][Highs] solver for LP, MILP, and QP.                |
| `io`            |         yes         | MPS, LP, and NL file writers.                                        |
| `baron`         |         no          | [`Baron`][Baron] global-optimization backend.                        |
| `clarabel`      |         no          | Pure-Rust [`Clarabel`][Clarabel] solver for LP, QP, and SOCP.        |
| `clarabel-faer` |         no          | Use Clarabel's `faer` linear-algebra backend.                        |
| `gurobi`        |         no          | [`Gurobi`][Gurobi] backend.                                          |
| `gams`          |         no          | [`Gams`][Gams] bridge.                                               |
| `pounce`        |         no          | Pure-Rust [`Pounce`][Pounce] solver for continuous nonlinear models. |
| `pounce-enzyme` |         no          | Exact Enzyme derivatives for POUNCE; requires nightly Rust.          |
| `mosek`         |         no          | [`Mosek`][Mosek] backend for MOSEK 11.2.                             |

[Highs]: https://docs.rs/oximo/latest/oximo/solvers/struct.Highs.html
[Clarabel]: https://docs.rs/oximo-clarabel/latest/oximo_clarabel/struct.Clarabel.html
[Pounce]: https://docs.rs/oximo-pounce/latest/oximo_pounce/struct.Pounce.html
[Gurobi]: https://docs.rs/oximo-gurobi/latest/oximo_gurobi/struct.Gurobi.html
[Gams]: https://docs.rs/oximo-gams/latest/oximo_gams/struct.Gams.html
[Baron]: https://docs.rs/oximo-baron/latest/oximo_baron/struct.Baron.html
[Mosek]: https://docs.rs/oximo-mosek/latest/oximo_mosek/struct.Mosek.html
[Solvers]: /solvers/
