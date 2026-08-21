+++
title = "Installation"
description = "Add oximo to a Rust project and configure an optional solver backend."
weight = 1
+++

For most projects, installation is one command. The default `oximo` build
provides the modeling layer and file I/O. Solver backends are opt-in.

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

This default dependency is enough to construct models and use the MPS, LP, and
NL I/O APIs. Choose a solver feature before solving a model.

### HiGHS

HiGHS is a bundled LP/MILP/QP solver. Enable it by doing:

```bash
cargo add oximo --features highs
```

The HiGHS build requires a C/C++ compiler:

- On Windows, install the MSVC C++ build tools.
- On macOS, run `xcode-select --install`.
- On Linux, install your distribution's standard C/C++ build tools.

Then follow the [Quickstart](/quickstart/) to solve the example model.

### Clarabel

Clarabel is a pure-Rust solver for continuous LP, QP, and SOCP models:

```bash
cargo add oximo --features clarabel
```

Use this path when you want a solver without a C compiler or external solver
installation. Clarabel does not solve mixed-integer models.

## Other backends

The [Solvers][Solvers] guide compares model-kind support and summarizes the
installation requirements of every backend. Enable a backend with its feature,
for example:

```toml
[dependencies]
oximo = { version = "0.5", features = ["pounce"] }
```

With no solver feature, you can still construct models and export them through
the default `io` feature.

## Advanced: exact nonlinear derivatives

`pounce` uses finite-difference derivatives for nonlinear expressions by
default. The nightly-only `pounce-enzyme` feature provides exact gradients,
Jacobians, and Hessians through [Enzyme](https://enzyme.mit.edu/).

It requires a nightly Rust toolchain with the `enzyme` component,
`RUSTFLAGS="-Zautodiff=Enable"`, and a fat-LTO profile:

```bash
RUSTFLAGS="-Zautodiff=Enable" cargo +nightly build --profile enzyme --features pounce-enzyme
```

## Feature reference

| Feature         | Included by default | Purpose                                                              |
| --------------- | :-----------------: | -------------------------------------------------------------------- |
| `highs`         |         no          | Bundled [`Highs`][Highs] solver for LP, MILP, and QP.                |
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
