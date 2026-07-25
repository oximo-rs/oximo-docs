//! Solvers.
//!
//! Runnable companion to the docs "Solvers" page. The default build uses HiGHS.
//! The other backends are shown behind their Cargo feature flags so this file
//! compiles with default features and still documents the swap.
//!
//!   cargo run --example solvers
//!   cargo run --example solvers --features clarabel
//!   cargo run --example solvers --features mosek

use oximo::prelude::*;
use oximo::solvers::Highs;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("solvers");

    set!(items = 0..3);
    let cost = [2.0, 3.0, 1.5];
    variable!(m, x[i in items] >= 0.0);
    constraint!(m, budget, sum!(x[i] for i in items) <= 10.0);
    objective!(m, Max, sum!(cost[i] * x[i] for i in items));

    // Common HiGHS options, all optional.
    let opts = HighsOptions::default()
        .time_limit(Duration::from_secs(60))
        .threads(4)
        .mip_gap(0.01)
        .method(HighsMethod::Ipm);

    let result = Highs.solve(&m, &opts)?;

    // The quickest read: the built-in, model-aware report.
    print!("{}", result.report(&m));

    // Programmatic access: why the solver stopped vs. whether a point came back.
    match result.termination {
        TerminationStatus::Optimal => {
            if let Some(obj) = result.objective() {
                println!("optimal: {obj}");
            }
        }
        TerminationStatus::Infeasible => println!("infeasible"),
        TerminationStatus::TimeLimit if result.has_solution() => {
            println!("time limit, best = {:?}", result.objective());
        }
        _ => {}
    }

    // A single element, and a constraint's shadow price.
    let budget_id = m.constraint_id("budget").unwrap();
    println!("x[0]        = {:?}", result.value_of(x[0]));
    println!("budget dual = {:?}", result.dual_of(budget_id));

    // Walk an indexed family without a manual key loop.
    for (key, value) in result.values_of(&x) {
        println!("x[{}] = {value:.2}", display_index_key(key));
    }

    // Solution pools: backends that return multiple points expose them best-first.
    for i in 0..result.result_count() {
        if let Some(point) = result.solution(i) {
            println!("solution {i}: objective {:?}", point.objective);
        }
    }

    // Every backend implements the same `Solver` trait, so swapping is one line.
    #[cfg(feature = "clarabel")]
    {
        use oximo::solvers::Clarabel;
        let r = Clarabel.solve(&m, &ClarabelOptions::default())?;
        println!("clarabel obj = {:?}", r.objective());
    }
    #[cfg(feature = "pounce")]
    {
        use oximo::pounce::Pounce;
        let r = Pounce.solve(&m, &PounceOptions::default())?;
        println!("pounce obj = {:?}", r.objective());
    }
    #[cfg(feature = "gurobi")]
    {
        use oximo::solvers::Gurobi;
        let r = Gurobi.solve(
            &m,
            &GurobiOptions::default()
                .time_limit(Duration::from_secs(120))
                .seed(101),
        )?;
        println!("gurobi obj = {:?}", r.objective());
    }
    #[cfg(feature = "mosek")]
    {
        use oximo::solvers::Mosek;
        let r = Mosek.solve(&m, &MosekOptions::default().mio_tol_rel_gap(1e-4))?;
        println!("mosek obj = {:?}", r.objective());
    }
    #[cfg(feature = "baron")]
    {
        use oximo::solvers::Baron;
        let r = Baron::new().solve(&m, &BaronOptions::default())?;
        println!("baron obj = {:?}", r.objective());
    }
    #[cfg(feature = "gams")]
    {
        use oximo::solvers::Gams;
        let r = Gams.solve(&m, &GamsOptions::default())?;
        println!("gams obj = {:?}", r.objective());
    }

    Ok(())
}
