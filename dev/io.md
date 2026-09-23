+++
title = "I/O"
description = "Export and import oximo models with MPS, LP or NL files."
weight = 7
+++

The [`oximo-io`][oximo_io] crate writes [`Model`][Model]s to the standard text formats **MPS**, **LP**, and **NL**, and can read all three formats back into a [`Model`][Model]. All I/O is gated on the `io` Cargo feature, which is on by default.

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
use std::fs::File;

io::write_mps(&m, &mut File::create("model.mps")?)?;
io::write_lp(&m, &mut File::create("model.lp")?)?;
io::write_nl(&m, &mut File::create("model.nl")?)?;
```

Each writer accepts anything that implements `std::io::Write`.

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
use std::fs::File;

let opts = WriteOptions { format: NlFormat::Ascii, ..Default::default() };
write_nl_with(&m, &mut File::create("model.nl")?, &opts)?;
```

[`write_nl_files`][write_nl_files] emits the `.nl` alongside its companion `.col`/`.row` name files, which is what most AMPL-compatible solvers expect when you want readable names in the solution.

## NL operator support

The NL writer emits supported nonlinear operations as exact prefix opcode
trees. The reader uses the same mapping for ASCII and little-endian binary NL:

| Opcode | Operation     | oximo support                           |
| ------ | ------------- | --------------------------------------- |
| `o13`  | `floor`       | Not represented by the expression model |
| `o14`  | `ceil`        | Not represented by the expression model |
| `o15`  | `abs`         | Read and written                        |
| `o16`  | unary `-`     | Read and written                        |
| `o34`  | logical `not` | Logical expressions are not represented |
| `o37`  | `tanh`        | Read and written                        |
| `o38`  | `tan`         | Read and written                        |
| `o39`  | `sqrt`        | Read and written                        |
| `o40`  | `sinh`        | Read and written                        |
| `o41`  | `sin`         | Read and written                        |
| `o42`  | `log10`       | Read and written                        |
| `o43`  | `log`         | Read and written                        |
| `o44`  | `exp`         | Read and written                        |
| `o45`  | `cosh`        | Read and written                        |
| `o46`  | `cos`         | Read and written                        |
| `o47`  | `atanh`       | Read and written                        |
| `o49`  | `atan`        | Read and written                        |
| `o50`  | `asinh`       | Read and written                        |
| `o51`  | `asin`        | Read and written                        |
| `o52`  | `acosh`       | Read and written                        |
| `o53`  | `acos`        | Read and written                        |

`exp2(x)` is written exactly as the power expression `2 ^ x` (`o5`), because
NL has no separate `exp2` opcode.

The oximo operations `cbrt`, `expm1`, `log1p`, and `log2` also have no direct
NL opcode. Writing one returns
[`IoError::UnsupportedNonlinearOperator`][IoError].
Reading `floor`, `ceil`, logical `not`, or another
well-formed but unrepresentable opcode returns [`IoError::UnsupportedNl`][IoError].

## Names round-trip

All writers preserve the [`Variable`][Variable] and constraint names from your model, so exported files cross-reference cleanly with [`SolverResult`][SolverResult] lookups such as `dual_of` and `reduced_costs` (see [Results](../results/)).

## Reading MPS models

Use [read_mps_file][read_mps_file] for a path or [read_mps][read_mps] for any
text stream:

```rust
use oximo::io::{read_mps, read_mps_file};
use std::fs::File;

let model = read_mps_file("model.mps")?;
let model_from_stream = read_mps(File::open("model.mps")?)?;
```

The reader accepts the standard linear sections, range rows, integer markers,
binary and semi-variable bounds, SOS sections, `INDICATORS` records, and the
`QUADOBJ`, `QMATRIX`, `QCMATRIX`, and `QSECTION` quadratic extensions. An MPS
indicator references an ordinary affine row using `IF row binary 0|1`; ranged
indicator bodies are represented as two rows. MPS does not identify the
coefficient scaling used by quadratic constraints, so the default is the
Gurobi convention. Select CPLEX or MOSEK scaling explicitly when needed:

```rust
use oximo::io::{
    MpsQuadraticFormat, MpsReadOptions, read_mps_file_with,
};

let options = MpsReadOptions {
    quadratic_format: MpsQuadraticFormat::Cplex,
};
let model = read_mps_file_with("cplex-model.mps", &options)?;
```

Malformed input returns [IoError::InvalidMps][IoError]. Multiple alternative
RHS, range, or bounds vectors and unsupported MPS constructs return
[IoError::UnsupportedMps][IoError].

## Reading NL models

The NL reader imports models produced by oximo or compatible
AMPL-style tools. Use [read_nl_file][read_nl_file] for a path or [read_nl][read_nl]
for any byte stream:

```rust
use oximo::io::{read_nl, read_nl_file};
use std::fs::File;

let model = read_nl_file("model.nl")?;
let model_from_stream = read_nl(File::open("model.nl")?)?;
```

Both ASCII and little-endian binary NL encodings are accepted. When a .row or
.col sidecar exists beside the file, it supplies the original row and column
names; otherwise deterministic names are generated. Interval rows and initial
values are preserved when they can be represented by the core model.

The reader rejects malformed input with [IoError::InvalidNl][IoError] and
well-formed NL sections that the core model cannot represent with
[IoError::UnsupportedNl][IoError]. Imported functions, defined variables,
logical/network constraints, complementarity sections, and unsupported expression
opcodes are intentionally rejected.

## Reading LP models

LP files can be imported from a byte stream or a path with [read_lp][read_lp]
and [read_lp_file][read_lp_file]:

```rust
use oximo::io::{read_lp, read_lp_file};
use std::fs::File;

let model = read_lp_file("model.lp")?;
let model_from_stream = read_lp(File::open("model.lp")?)?;
```

The reader supports the CPLEX LP linear and quadratic subset represented by the
core model, including objectives, constraints, bounds, integer/binary and
semicontinuous domains, quadratic terms, SOS sections, and inline indicator
rows of the form `binary = 0|1 -> affine relation`. Ranged indicator bodies are
written as two indicator rows. Malformed input returns
[IoError::InvalidLp][IoError] with its source line and column. Unsupported LP
sections return [IoError::UnsupportedLp][IoError].

NL has no native segment for the indicator representation used by oximo.
Writing a model with an active indicator returns `IoError::UnsupportedNl`.

## Skipping the writers

If you don't need file export, opt out of the `io` feature to drop the dependency:

```toml
[dependencies]
oximo = { version = "0.6", default-features = false, features = ["highs"] }
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
[read_nl]: https://docs.rs/oximo-io/latest/oximo_io/nl/fn.read_nl.html
[read_nl_file]: https://docs.rs/oximo-io/latest/oximo_io/nl/fn.read_nl_file.html
[read_mps]: https://docs.rs/oximo-io/latest/oximo_io/fn.read_mps.html
[read_mps_file]: https://docs.rs/oximo-io/latest/oximo_io/fn.read_mps_file.html
[read_mps_file_with]: https://docs.rs/oximo-io/latest/oximo_io/fn.read_mps_file_with.html
[read_lp]: https://docs.rs/oximo-io/latest/oximo_io/fn.read_lp.html
[read_lp_file]: https://docs.rs/oximo-io/latest/oximo_io/fn.read_lp_file.html
[IoError]: https://docs.rs/oximo-io/latest/oximo_io/enum.IoError.html
