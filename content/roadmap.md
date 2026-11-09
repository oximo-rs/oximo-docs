+++
title = "Roadmap"
description = "Where oximo is headed, including planned backends, modeling features, and tooling."
weight = 100
render = true

[extra]
hide_from_toc = true
hide_from_nav = true
+++

This is a living document that sketches the direction of the project, and items can move, merge, or drop as priorities shift. If something here matters to you, or you want to help build it, open an issue on [GitHub](https://github.com/oximo-rs/oximo).

Status labels: **Planned** is on the near-term list, **Exploring** is under
active design, and **Considering** is an idea we like but have not committed to.

## Solvers

- [ ] **IIS for HiGHS.** Provide Irreducible Infeasible Subsystem for the HiGHS backend. See [highs#47](https://github.com/rust-or/highs/issues/47). _(Planned)_
- [x] **SCIP backend.** Available through `russcip`, with bundled and system
      installation options, mixed-integer nonlinear models, and persistent solves.
      See [Solvers > SCIP](../solvers/#scip).
- [ ] **IPOPT backend.** Planned but waiting on support for Windows and a new release of `ipopt-rs`. _(Planned)_
- [ ] **CPLEX backend.** Planned but waiting on more basic features in `cplex-rs` like QP/MIQP support and duals. See [cplex-rs#7](https://github.com/cplex-rs/cplex-rs/pull/7). _(Planned)_
- [ ] **Support other versions of MOSEK.** Currently `oximo-mosek` only supports MOSEK 11.2. Support for other versions is planned. _(Planned)_

## Modeling

- [x] **SOS constraints.** Scalar and indexed SOS1/SOS2 macros, native backend
      support, and explicit MILP reformulations. See [SOS constraints](../modeling/#sos-constraints).
- [x] **Native indicator constraints.** Ergonomic scalar and indexed macros,
      native Gurobi and MOSEK support, and selected GAMS sub-solvers.
- [x] **Indicator reformulations.** Explicit Big-M reformulations for backends
      without native indicator support.
- [x] **Semidefinite constraints.** Symmetric matrix variables and affine PSD
      constraints, with Clarabel and MOSEK backends and matrix primal/dual queries.
      See [Semidefinite constraints](../modeling/#semidefinite-constraints).
- [x] **Typed constraint handles and updates.** Preserve the original function
      and set, update compatible declarations, and query original constraints and
      their duals. See [Handles and updates](../modeling/#handles-and-updates).
- [x] **Incremental affine construction.** Reusable affine builders, including
      opt-in compensated summation. See [Incremental affine construction](../modeling/#incremental-affine-construction).

## I/O

- [x] **Model readers.** Read MPS, LP, and NL files back into a model for
      round-tripping and importing supported models built elsewhere. See [I/O](../io/).

## Ecosystem

- [ ] **More worked examples.** Additional end-to-end case studies. _(Planned)_
- [x] **Generalized Disjunctive Programming (GDP).** Boolean decisions,
      disjunct blocks, logical constraints, nested disjunctions, and Big-M/Hull
      reformulations. See [GDP](../gdp/).
- [ ] **Unit-based models.** Create models with unit-based variables and constraints. _(Considering)_
