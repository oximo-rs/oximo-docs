//! Solver results: status, model-aware reports, indexed values, and row handles.
//!
//! Runnable companion to the development docs "Results" page.
//!
//!   cargo run --example results

use oximo::prelude::*;
use oximo::solvers::Highs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("cover");
    let demand = [2.0, 3.0, 1.0];
    variable!(m, x[i in 0..demand.len()] >= 0.0);
    let cover: IndexedConstraint<usize> =
        constraint!(m, cover[i in 0..demand.len()], x[i] >= demand[i]);
    constraint!(m, total, sum!(x[i] for i in 0..demand.len()) <= 10.0);
    objective!(m, Min, sum!(x[i] for i in 0..demand.len()));

    let result = Highs.solve(&m, &HighsOptions::default())?;
    print!("{}", result.report(&m)?);

    match result.termination {
        TerminationStatus::Optimal => println!("optimal: {:?}", result.objective()),
        TerminationStatus::Infeasible => println!("infeasible"),
        TerminationStatus::TimeLimit if result.has_solution() => {
            println!("time limit, best = {:?}", result.objective());
        }
        _ => println!("stopped: {:?}", result.termination),
    }

    let first_cover = cover.get(0).expect("cover row exists");
    println!("cover[0] dual = {:?}", result.dual_of(first_cover)?);
    for (key, value) in result.values_of(&x)? {
        println!("x[{}] = {value:.2}", display_index_key(key));
    }

    Ok(())
}
