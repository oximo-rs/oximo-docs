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

- [ ] **IIS for HiGHS.** Provide Irreducible Infeasibility System for the HiGHS backend. See [highs#47](https://github.com/rust-or/highs/issues/47). _(Planned)_
- [ ] **SCIP backend.** Currently blocked by the fact that `russcip` does not yet support non-linear constraints. See [russcip#280](https://github.com/scipopt/russcip/issues/280), [oximo#8](https://github.com/oximo-rs/oximo/issues/8). _(Planned)_
- [ ] **IPOPT backend.** Planned but waiting on support for Windows and a new release of `ipopt-rs`. _(Planned)_
- [ ] **CPLEX backend.** Planned but waiting on more basic features in `cplex-rs` like QP/MIQP support and duals. See [cplex-rs#7](https://github.com/cplex-rs/cplex-rs/pull/7). _(Planned)_
- [ ] **Support other versions of MOSEK.** Currently `oximo-mosek` only supports MOSEK 11.2. Support for other versions is planned. _(Planned)_

## Modeling

- [ ] **Indicator and SOS constraints.** Ergonomic macros for indicator constraints
  and special-ordered sets. _(Planned)_

## I/O

- [ ] **Model readers.** oximo writes MPS, LP, and NL today (see [I/O](/io/)).
  Add parsers so those formats can be read back into a [`Model`][Model], which
  enables round-tripping and importing models built elsewhere [oximo#42](https://github.com/oximo-rs/oximo/issues/42). _(Exploring)_

## Ecosystem

- [ ] **More worked examples.** Additional end-to-end case studies. _(Planned)_
- [ ] **Generalized Disjunctive Programming (GDP).** Create models with disjunctive constraints and provide automatic reformulations [oximo#20](https://github.com/oximo-rs/oximo/issues/20). _(Exploring)_
- [ ] **Unit-based models.** Create models with unit-based variables and constraints. _(Considering)_

[Model]: https://docs.rs/oximo/latest/oximo/struct.Model.html
