+++
title = "Solvers"
description = "Solve oximo models with a variety of solvers."
weight = 4
+++

oximo is solver-agnostic, and provides a variety of backends to solve your optimization models.

Every backend implements the same [`Solver`][Solver] trait, so swapping engines is a one-line change. The general pattern is:

```rust
let result = Backend.solve(&model, &BackendOptions::default())?;
```

`result` is a [`SolverResult`][SolverResult], the same struct regardless of backend.

## Choosing a backend

To determine what backend to use, consider what each backend _solves_ (by model kind) and what it _costs and offers_ (license, install, diagnostics). Each backend's Cargo feature flag is named in its own section below.

### Model kind support

Each row is what [`Solver::supports`][Solver] accepts for that backend, against every [`ModelKind`][ModelKind]:

| Backend                |  LP   | MILP  |  QP   | MIQP  |  QCP  | MIQCP | SOCP  | MISOCP |  NLP  | MINLP |
| ---------------------- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :----: | :---: | :---: |
| [`Highs`][Highs]       | **✓** | **✓** | **✓** |   —   |   —   |   —   |   —   |   —    |   —   |   —   |
| [`Clarabel`][Clarabel] | **✓** |   —   | **✓** |   —   |   —   |   —   | **✓** |   —    |   —   |   —   |
| [`Pounce`][Pounce]     | **✓** |   —   | **✓** |   —   | **✓** |   —   |   —   |   —    | **✓** |   —   |
| [`Gurobi`][Gurobi]     | **✓** | **✓** | **✓** | **✓** | **✓** | **✓** | **✓** | **✓**  | **✓** | **✓** |
| `Mosek`                | **✓** | **✓** | **✓** | **✓** | **✓** | **✓** | **✓** | **✓**  |   —   |   —   |
| [`Baron`][Baron]       | **✓** | **✓** | **✓** | **✓** | **✓** | **✓** | **✓** | **✓**  | **✓** | **✓** |
| [`Gams`][Gams]         | **✓** | **✓** | **✓** | **✓** | **✓** | **✓** | **✓** | **✓**  | **✓** | **✓** |

`Gams` accepts every kind at the oximo layer, but the actual coverage is the GAMS sub-solver's. Pick one that handles the kind you emit.

### Requirements & capabilities

Deployment cost and diagnostic/solve capability per backend.

**✓** = yes/supported; — = no.

| Backend                | License | Separate install | C compiler | Direct interface |  IIS  | Warm start | Sol. pool | Duals | Parallel |
| ---------------------- | :-----: | :--------------: | :--------: | :--------------: | :---: | :--------: | :-------: | :---: | :------: |
| [`Highs`][Highs]       |  **—**  |      **—**       |     ✓      |      **✓**       |  —†   |   **✓**    |     —     | **✓** |  **✓**   |
| [`Clarabel`][Clarabel] |  **—**  |      **—**       |   **—**    |      **✓**       |   —   |   **✓**    |     —     | **✓** |  **✓**   |
| [`Pounce`][Pounce]     |  **—**  |      **—**       |   **—**    |      **✓**       |   —   |   **✓**    |     —     | **✓** |    —     |
| [`Gurobi`][Gurobi]     |    ✓    |        ✓         |   **—**    |      **✓**       | **✓** |   **✓**    |   **✓**   | **✓** |  **✓**   |
| `Mosek`                |    ✓    |        ✓         |   **—**    |      **✓**       |   —   |   **✓**    |     —     | **✓** |  **✓**   |
| [`Baron`][Baron]       |    ✓    |        ✓         |   **—**    |        —         | **✓** |     —      |   **✓**   | **✓** |  **✓**   |
| [`Gams`][Gams]         |    ✓    |        ✓         |   **—**    |        —         |   —   |     —      |  **✓**¶   | **✓** |  **✓**   |

Column meanings:

- **License:** the underlying solver is commercial and needs a license at runtime. The oximo wrapper crates are all MIT OR Apache-2.0.
- **Direct interface:** talks to the solver in-process. C API/FFI for HiGHS, Gurobi, and MOSEK. Pure Rust for Clarabel and Pounce. BARON and GAMS instead exchange model/result files (`.bar`, `.gms`) with an external executable.
- **C compiler:** whether a C/C++ compiler is required at build time.
- **IIS:** computes an irreducible infeasible set to explain an infeasible model.
- **Warm start:** persistent handle for incremental re-solves.
- **Sol. pool:** returns multiple solutions, best-first.
- **Duals:** shadow prices/reduced costs.
- **Parallel:** solver can solve in parallel.

Footnotes:

- **†** The `highs` crate does not support IIS yet.
- **¶** GAMS's pool and duals come from the underlying sub-solver's GDX output (e.g. CPLEX `solnpool`).

A backend rejects model kinds it can't handle, so check [`Model::kind()`][Model] if a solve returns [`SolverError::UnsupportedKind`][SolverError].

## External solver setup

Gurobi, MOSEK, BARON, and GAMS need software installed outside Cargo. Add the
corresponding feature only after that software is installed and licensed.

### Gurobi

Enable `gurobi`, set `GUROBI_HOME` to the Gurobi installation, and make sure a
Gurobi license is active. The backend links through the
[`grb`](https://crates.io/crates/grb) crate.

```toml
[dependencies]
oximo = { version = "0.5", features = ["gurobi"] }
```

### MOSEK

Enable `mosek`, install MOSEK 11.2, and configure a valid license. Set
`MOSEK_BINDIR_112` to MOSEK's platform `bin` directory when the installation is
not in its default location. The backend links through the
[`mosek`](https://crates.io/crates/mosek) crate.

> Note for Windows: the current `mosek` crate build script emits an unquoted
> linker flag, so setting `MOSEK_BINDIR_112` directly to a path containing
> spaces can fail. Create a junction with a space-free path, then point the
> environment variable at that junction. See [mosek.rust#1](https://github.com/MOSEK/mosek.rust/issues/1).

> Note: We currently support only MOSEK 11.2.
> Support for other versions of MOSEK is planned.

```toml
[dependencies]
oximo = { version = "0.5", features = ["mosek"] }
```

### BARON and GAMS

Enable `baron` or `gams` after placing the respective executable on `PATH`.
Both backends exchange files with an external program rather than linking the
solver into your process.

```toml
[dependencies]
oximo = { version = "0.5", features = ["baron"] }
```

Use `features = ["gams"]` instead for GAMS.

## HiGHS

[`Highs`][Highs] is bundled by default via the `highs` Cargo feature. No external install required, but a C/C++ compiler is needed at build time.

```rust
use oximo::prelude::*;
use oximo::solvers::Highs;
use std::time::Duration;

let result = Highs.solve(&m, &HighsOptions::default()
    .time_limit(Duration::from_secs(60))
    .threads(4)
    .mip_gap(0.01)
    .verbose(true)
    .method(HighsMethod::Ipm))?;
```

Common [`HighsOptions`][HighsOptions]:

| Method           | Effect                                                              |
| ---------------- | ------------------------------------------------------------------- |
| `.time_limit(d)` | Stop after `d` (`std::time::Duration`)                              |
| `.threads(n)`    | Cap parallelism                                                     |
| `.mip_gap(g)`    | Relative MIP optimality gap (e.g. `0.01` = 1%)                      |
| `.verbose(b)`    | Stream the solver log                                               |
| `.method(m)`     | LP algorithm via [`HighsMethod`][HighsMethod] (`Simplex`, `Ipm`, …) |

## Clarabel

[`Clarabel`][Clarabel] is a pure-Rust conic interior-point solver. No install, no license. It handles continuous LP, QP (convex quadratic objectives), and SOCP models.

```rust
use oximo::prelude::*;
use oximo::solvers::Clarabel;

let result = Clarabel.solve(&m, &ClarabelOptions::default())?;
```

## POUNCE

[`Pounce`][Pounce] is a pure-Rust port of IPOPT, covering continuous LP/QP/QCP/NLP. Enable the `pounce` feature.

```rust
use oximo::prelude::*;
use oximo::pounce::Pounce;

let result = Pounce.solve(&m, &PounceOptions::default())?;
```

Purely linear/quadratic models solve with exact analytic derivatives. Models containing a nonlinear function fall back to finite differences and an L-BFGS Hessian unless you enable the nightly-only `pounce-enzyme` feature, which supplies exact gradients plus sparse Jacobians and Hessians.

For any nonlinear model, enabling `pounce-enzyme` is **highly recommended**. Exact derivatives are faster and far more accurate than the finite-difference fallback, which improves both convergence and robustness. It requires a nightly toolchain and a fat-LTO build (see [Installation > Advanced: exact nonlinear derivatives](/installation/#advanced-exact-nonlinear-derivatives)).

## Gurobi

[`Gurobi`][Gurobi] requires the `gurobi` Cargo feature plus a licensed Gurobi installation reachable via `GUROBI_HOME`.

> Note: Only Gurobi v12 and later are supported.

```rust
use oximo::prelude::*;
use oximo::solvers::Gurobi;
use std::time::Duration;

let result = Gurobi.solve(&m, &GurobiOptions::default()
    .time_limit(Duration::from_secs(120))
    .mip_focus(1)
    .seed(101))?;
```

Common [`GurobiOptions`][GurobiOptions]:

| Method           | Effect                                                         |
| ---------------- | -------------------------------------------------------------- |
| `.time_limit(d)` | Wall-clock limit                                               |
| `.mip_focus(n)`  | Gurobi `MIPFocus` (1 = feasibility, 2 = optimality, 3 = bound) |
| `.mip_gap(g)`    | Relative MIP optimality gap                                    |
| `.seed(n)`       | Random seed for reproducible runs                              |

## MOSEK

`Mosek` supports LP, MILP, convex QP/MIQP, convex QCP/MIQCP, and
SOCP/MISOCP models. It requires the `mosek` Cargo feature, a licensed MOSEK
11.2 installation, and `MOSEK_BINDIR_112` when MOSEK is outside its default
location. MOSEK validates the convexity of quadratic data.

```rust
use oximo::prelude::*;
use oximo::solvers::Mosek;
use std::time::Duration;

let result = Mosek.solve(&m, &MosekOptions::default()
    .time_limit(Duration::from_secs(120))
    .threads(4)
    .mio_tol_rel_gap(1e-4))?;
```

`MosekOptions` provides builders for every MOSEK 11.2 parameter. Universal
options such as `time_limit`, `threads`, and `verbose` are applied first.
MOSEK-specific parameter builders are then applied in call order.

## BARON

[`Baron`][Baron] is a global solver for nonconvex LP/MILP/QP/MIQP/NLP/MINLP. Requires the `baron` feature and a licensed BARON install on `PATH`.

```rust
use oximo::prelude::*;
use oximo::solvers::Baron;

let result = Baron::new().solve(&m, &BaronOptions::default())?;
```

## GAMS

[`Gams`][Gams] requires the `gams` Cargo feature plus a GAMS install on `PATH`. Useful when you want to route a model through GAMS-managed solvers (CPLEX, BARON, IPOPT, KNITRO, ...).

```rust
use oximo::prelude::*;
use oximo::solvers::Gams;

let result = Gams.solve(&m, &GamsOptions::default())?;
```

See [`GamsOptions`][GamsOptions] and the per-solver option structs in [`oximo::gams`][gams_mod] (`GamsCplexOptions`, `GamsBaronOptions`, `GamsIpoptOptions`, ...) for tuning the underlying solver.

## Next steps

- [Results](/results/): inspect solver status, values, duals, and solution pools
- [Printing & Debugging](/debugging/): print a model as algebra and track down what it actually says
- [I/O](/io/): write your model to MPS, LP or NL for use with external tools

[Solver]: https://docs.rs/oximo/latest/oximo/prelude/trait.Solver.html
[SolverResult]: https://docs.rs/oximo/latest/oximo/prelude/struct.SolverResult.html
[SolverError]: https://docs.rs/oximo/latest/oximo/prelude/enum.SolverError.html
[Model]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html
[ModelKind]: https://docs.rs/oximo/latest/oximo/prelude/enum.ModelKind.html
[Highs]: https://docs.rs/oximo/latest/oximo/solvers/struct.Highs.html
[HighsOptions]: https://docs.rs/oximo/latest/oximo/prelude/struct.HighsOptions.html
[HighsMethod]: https://docs.rs/oximo/latest/oximo/prelude/enum.HighsMethod.html
[Clarabel]: https://docs.rs/oximo-clarabel/latest/oximo_clarabel/struct.Clarabel.html
[Pounce]: https://docs.rs/oximo-pounce/latest/oximo_pounce/struct.Pounce.html
[Gurobi]: https://docs.rs/oximo-gurobi/latest/oximo_gurobi/struct.Gurobi.html
[GurobiOptions]: https://docs.rs/oximo-gurobi/latest/oximo_gurobi/struct.GurobiOptions.html
[Baron]: https://docs.rs/oximo-baron/latest/oximo_baron/struct.Baron.html
[Gams]: https://docs.rs/oximo-gams/latest/oximo_gams/struct.Gams.html
[GamsOptions]: https://docs.rs/oximo-gams/latest/oximo_gams/struct.GamsOptions.html
[gams_mod]: https://docs.rs/oximo/latest/oximo/gams/index.html
