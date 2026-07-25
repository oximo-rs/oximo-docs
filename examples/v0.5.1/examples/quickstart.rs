//! Quickstart.
//!
//! Runnable companion to the docs "Quickstart" page.
//!
//! ```text
//! max  3x + 4y
//! s.t. x + 2y <= 14
//!      3x >= y
//!      x <= y + 2
//!      x >= 0, 0 <= y <= 4
//! ```
//!
//! Optimum: obj = 34, x = 6, y = 4.
//!
//!   cargo run --example quickstart

use oximo::prelude::*;
use oximo::solvers::Highs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("transport");

    variable!(m, x >= 0.0);
    variable!(m, 0.0 <= y <= 4.0);

    constraint!(m, c1, x + 2.0 * y <= 14.0);
    constraint!(m, c2, 3.0 * x >= y);
    constraint!(m, c3, x <= y + 2.0);
    objective!(m, Max, 3.0 * x + 4.0 * y);

    let result = Highs.solve(&m, &HighsOptions::default())?;
    println!("obj = {:?}", result.objective()); // Some(34.0)
    println!("x   = {:?}", result.value_of(x)); // Some(6.0)
    println!("y   = {:?}", result.value_of(y)); // Some(4.0)

    // Human-readable dump of the whole solution.
    print!("{}", result.report(&m));
    Ok(())
}
