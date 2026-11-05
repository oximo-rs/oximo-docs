//! Solve-time parameters, scenario queries, captured formulas, and row handles.
//!
//! Run: cargo run --example results

use oximo::prelude::*;
use oximo::solvers::Highs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    parameter_snapshots()?;
    semantic_rows()?;
    Ok(())
}

fn parameter_snapshots() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("parameter snapshots");
    param!(m, price = 2.0);
    variable!(m, 0.0 <= quantity <= 10.0);
    let revenue = price * quantity;
    let captured = revenue.capture();
    objective!(m, Max, revenue);
    let mut solver = Highs.persistent();
    let options = HighsOptions::default();
    let first = solver.solve(&m, &options)?;
    assert_eq!(first.termination, TerminationStatus::Optimal);
    assert!(first.has_solution());
    let pooled = first.solution(0).ok_or("missing solution")?.clone();

    price.set_param_value(3.0)?;
    // Saved results use the solve-time snapshot, even after a parameter write.
    assert_eq!(first.value_of(revenue)?, Some(20.0));
    assert_eq!(first.value_of_captured(&captured)?, Some(20.0));
    assert_eq!(pooled.value_of(revenue)?, Some(20.0));
    // Scenario queries evaluate the saved point under the current bindings.
    assert_eq!(first.value_of_with_current_parameters(revenue)?, Some(30.0));
    assert_eq!(
        pooled.value_of_with_current_parameters(revenue)?,
        Some(30.0)
    );
    assert_eq!(first.objective(), Some(20.0));
    let second = solver.solve(&m, &options)?;
    assert_eq!(second.objective(), Some(30.0));
    println!(
        "saved objective {:?}; new objective {:?}",
        first.objective(),
        second.objective()
    );
    print!("{}", second.report(&m)?);
    Ok(())
}

fn semantic_rows() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("semantic row queries");
    let demand = [2.0, 3.0, 1.0];
    variable!(m, x[i in 0..demand.len()] >= 0.0);
    let cover = constraint!(m, cover[i in 0..demand.len()], 2.0 * x[i] >= 2.0 * demand[i]);
    let bands: IndexedRangeConstraint<usize> =
        constraint!(m, bands[i in 0..demand.len()], 1.0 <= x[i] <= 4.0);
    objective!(m, Min, sum!(x[i] for i in 0..demand.len()));
    let result = Highs.solve(&m, &HighsOptions::default())?;
    for (i, handle) in cover.iter() {
        println!(
            "cover[{i}] original multiplier: {:?}",
            result.dual_of(handle)?
        );
        println!("stored row: {}", m.display_row(handle.row_id()));
        println!(
            "evaluation: {:?}",
            result.constraint_evaluation(&m, handle)?
        );
    }
    for (i, handles) in bands.iter() {
        match handles {
            RangeConstraintHandles::Interval(handle) => {
                println!("bands[{i}] multiplier: {:?}", result.dual_of(handle)?);
            }
            RangeConstraintHandles::Split { lower, upper } => {
                println!(
                    "bands[{i}] multipliers: {:?}, {:?}",
                    result.dual_of(lower)?,
                    result.dual_of(upper)?
                );
            }
        }
    }
    for (key, value) in result.values_of(&x)? {
        println!("x[{}] = {value}", display_index_key(key));
    }
    let foreign = Model::new("foreign");
    variable!(foreign, y);
    assert!(result.value_of(y).is_err());
    Ok(())
}
