//! Verify the install: a one-variable model solved with HiGHS.
//!
//! Runnable companion to the docs "Installation" page.
//! If this compiles and prints `obj = Some(10.0)`, the default build works.
//!
//!   cargo run --example installation

use oximo::prelude::*;
use oximo::solvers::Highs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("smoke-test");
    variable!(m, 0.0 <= x <= 10.0);
    objective!(m, Max, x);

    let result = Highs.solve(&m, &HighsOptions::default())?;
    println!("obj = {:?}", result.objective()); // Some(10.0)
    println!("x   = {:?}", result.value_of(x)); // Some(10.0)
    Ok(())
}
