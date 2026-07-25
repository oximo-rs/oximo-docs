+++
title = "I/O"
description = "Export oximo models to MPS, LP or NL files."
weight = 7
+++

The [`oximo-io`][oximo_io] crate writes [`Model`][Model]s to the standard text formats: **MPS**, **LP**, and **NL**. All three writers are gated on the `io` Cargo feature, which is on by default.

Use this when you want to:

- Hand a [`Model`][Model] to a solver oximo doesn't bundle (COPT, SCIP, CPLEX, ...)
- Feed a nonlinear model to an AMPL-compatible solver via NL
- Reproduce a bug report against a third-party tool
- Archive the exact problem instance for later inspection
- Inspect the model

## To a string

```rust
use oximo::io;

let mps = io::to_mps_string(&m)?;
let lp  = io::to_lp_string(&m)?;
let nl  = io::to_nl_string(&m)?;
```

Each returns a `String` you can log, hash, or feed into something else in memory.

## To a file

```rust
use oximo::io;

io::write_mps(&m, "model.mps")?;
io::write_lp(&m, "model.lp")?;
io::write_nl(&m, "model.nl")?;
```

All three accept anything that implements `AsRef<Path>`.

## Picking a format

MPS and LP describe linear and quadratic models. In general, you should use LP and reach for NL when the model has
nonlinear expressions that MPS and LP cannot represent.

| Format | Pros                                                     | When to pick                              |
| ------ | -------------------------------------------------------- | ----------------------------------------- |
| MPS    | Universal, column-oriented, fixed historical format      | Maximum solver compatibility, archival    |
| LP     | Human-readable, row-oriented, mirrors algebraic notation | Quick inspection, sharing in bug reports  |
| NL     | Compact, carries nonlinear structure                     | NLP/MINLP models, AMPL-compatible solvers |

## NL options

The NL writer is the most configurable of the three. [`write_nl_with`][write_nl_with] and [`to_nl_string_with`][to_nl_string_with] take a [`WriteOptions`][WriteOptions] to select the [`NlFormat`][NlFormat] (binary or ASCII) and attach solver metadata: suffixes, defined variables, imported functions, and complementarity pairs.

```rust
use oximo::io::{NlFormat, WriteOptions, write_nl_with};

let opts = WriteOptions::default().format(NlFormat::Ascii);
write_nl_with(&m, "model.nl", &opts)?;
```

[`write_nl_files`][write_nl_files] emits the `.nl` alongside its companion `.col`/`.row` name files, which is what most AMPL-compatible solvers expect when you want readable names in the solution.

## Names round-trip

All writers preserve the [`Variable`][Variable] and constraint names from your model, so exported files cross-reference cleanly with [`SolverResult`][SolverResult] lookups such as `dual_of` and `reduced_costs` (see [Results](/results/)).

## Skipping the writers

If you don't need file export, opt out of the `io` feature to drop the dependency:

```toml
[dependencies]
oximo = { version = "0.5", default-features = false, features = ["highs"] }
```

[Model]: https://docs.rs/oximo/latest/oximo/prelude/struct.Model.html
[Variable]: https://docs.rs/oximo/latest/oximo/prelude/struct.Variable.html
[SolverResult]: https://docs.rs/oximo/latest/oximo/prelude/struct.SolverResult.html
[oximo_io]: https://docs.rs/oximo-io/latest/oximo_io/
[WriteOptions]: https://docs.rs/oximo-io/latest/oximo_io/nl/struct.WriteOptions.html
[NlFormat]: https://docs.rs/oximo-io/latest/oximo_io/nl/enum.NlFormat.html
[write_nl_with]: https://docs.rs/oximo-io/latest/oximo_io/nl/fn.write_nl_with.html
[to_nl_string_with]: https://docs.rs/oximo-io/latest/oximo_io/nl/fn.to_nl_string_with.html
[write_nl_files]: https://docs.rs/oximo-io/latest/oximo_io/nl/fn.write_nl_files.html
