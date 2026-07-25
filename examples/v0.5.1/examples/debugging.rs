//! Printing & Debugging.
//!
//! Runnable companion to the docs "Printing & Debugging" page. The IIS section
//! needs Gurobi, so it is gated behind the `gurobi` feature and skipped in the
//! default build.
//!
//!   cargo run --example debugging
//!   cargo run --example debugging --features gurobi

use oximo::prelude::*;

fn supply_of(_p: &str) -> f64 {
    500.0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    whole_model();
    one_piece_at_a_time();
    rule_generated_names();
    parameters_show_binding();
    checking_the_kind();
    diagnosing_infeasibility()?;
    Ok(())
}

/// `Model` implements `Display`.
fn whole_model() {
    println!("== printing ==");
    let m = Model::new("diet");
    variable!(m, x >= 0.0);
    variable!(m, y >= 0.0);
    constraint!(m, c1, x + 2.0 * y <= 14.0);
    constraint!(m, c2, 3.0 * x - y >= 0.0);
    objective!(m, Min, 3.0 * x + 4.0 * y);

    println!("{m}\n");
}

/// Display adapters render a single piece, for an assertion or a targeted dbg!.
fn one_piece_at_a_time() {
    println!("== debugging ==");
    let m = Model::new("pieces");
    variable!(m, x >= 0.0);
    variable!(m, y >= 0.0);
    constraint!(m, c, x * y <= y + 3.0);

    let c = m.constraint_id("c").unwrap();
    println!("constraint c: {}", m.display_constraint(c));
    println!("expr        : {}", m.display_expr(2.0 * x - y));
    println!("objective   : {}\n", m.display_objective());
}

/// Indexed constraints are auto-named `base[key]`, exactly what `constraint_id`
/// expects, so it confirms a rule expanded over the keys you intended.
fn rule_generated_names() {
    println!("== rule-generated names ==");
    let m = Model::new("transport");
    let plants = Set::strings(["seattle", "san-diego"]);
    let markets = Set::strings(["nyc", "chicago"]);
    set!(routes = plants * markets);
    variable!(m, x[r in routes] >= 0.0);

    constraint!(m, supply[p in plants], sum!(x[p, q] for q in markets) <= supply_of(&p));

    let c = m.constraint_id("supply[seattle]").unwrap();
    println!("{}\n", m.display_constraint(c));
}

/// A printed model resolves parameters to their current binding.
fn parameters_show_binding() {
    println!("== parameters show their binding ==");
    let m = Model::new("sweep");
    variable!(m, x >= 0.0);
    constraint!(m, cap, x <= 10.0);
    param!(m, price = 4.0);
    objective!(m, Min, price * x);

    println!("{m}"); // ... params: price = 4

    m.set_param(price, 7.5);
    println!("{m}"); // ... params: price = 7.5
    println!();
}

/// oximo infers the kind from the expressions you write.
fn checking_the_kind() {
    println!("== checking the model kind ==");
    let m = Model::new("kind");
    variable!(m, x >= 0.0);
    variable!(m, y >= 0.0);
    constraint!(m, c1, x + 2.0 * y <= 14.0);
    objective!(m, Min, 3.0 * x + 4.0 * y);

    assert_eq!(m.kind(), ModelKind::LP);
    println!("kind = {:?}\n", m.kind());
}

/// Compute an irreducible infeasible subsystem to explain an infeasible model.
/// Needs a native conflict refiner (Gurobi or BARON).
#[cfg(feature = "gurobi")]
fn diagnosing_infeasibility() -> Result<(), Box<dyn std::error::Error>> {
    use oximo::solvers::Gurobi;

    println!("== diagnosing infeasibility ==");
    let m = Model::new("iis");
    variable!(m, x >= 0.0);
    constraint!(m, floor, x >= 2.0);
    constraint!(m, ceil, x <= 1.0);
    objective!(m, Min, x);

    let iis = Gurobi.compute_iis(&m, &GurobiOptions::default())?;
    println!("{}", iis.report(&m));
    Ok(())
}

#[cfg(not(feature = "gurobi"))]
fn diagnosing_infeasibility() -> Result<(), Box<dyn std::error::Error>> {
    println!("== diagnosing infeasibility ==");
    println!("(enable --features gurobi to compute an IIS)");
    Ok(())
}
