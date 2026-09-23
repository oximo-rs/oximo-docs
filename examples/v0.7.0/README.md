# oximo v0.7.0 documentation examples

These examples match the runnable snippets in the development documentation.
The `oximo` dependency uses the Git repository until v0.7.0 is available on
crates.io. `Cargo.lock` pins the Git revision used for verification.

HiGHS is enabled in the dependency so the examples that solve models run with
the default feature set. A C/C++ compiler is required to build HiGHS.

From this directory, run an example with:

```sh
cargo run --example quickstart
```

The example targets are `installation`, `quickstart`, `modeling`, `solvers`,
`results`, `debugging`, and `io`. Commercial solver examples need their
corresponding feature and solver installation, for example:

```sh
cargo run --example debugging --features gurobi
```
