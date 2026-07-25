//! Modeling.
//!
//! Variables, parameters, index sets, indexed variables, expressions, sums,
//! constraints, objectives, rule-style constraints, and nonlinear terms.
//! Snippets that reference undeclared data in the docs are filled in here with
//! concrete sets and helper functions so the whole thing compiles and runs.
//!
//!   cargo run --example modeling

#![allow(unused)]

use oximo::prelude::*;
use oximo::solvers::Highs;

// Stand-ins for the data the docs reference symbolically.
fn supply_of(_p: &str) -> f64 {
    500.0
}
fn unit_cost(_p: &str, _q: &str) -> f64 {
    1.0
}
fn capacity_for(_p: &str, _q: &str) -> f64 {
    100.0
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    variables();
    parameters()?;
    index_sets();
    indexed_variables();
    expressions();
    summing();
    constraints();
    objectives();
    rule_constraints();
    nonlinear();
    Ok(())
}

/// Variables.
fn variables() {
    println!("== variables ==");
    let m = Model::new("my_model");

    variable!(m, x >= 0.0); // continuous, x >= 0
    variable!(m, 0.0 <= y <= 10.0); // continuous, 0 <= y <= 10
    variable!(m, z); // free
    variable!(m, b, Bin); // binary {0, 1}
    variable!(m, n >= 0.0, Int); // general integer
    variable!(m, s <= 10.0, SemiCont(2.0)); // 0 or in [2, 10]

    // Keyword arguments after the name.
    variable!(m, xk, lb = 0.0, ub = 1.0);
    variable!(m, nk, lb = 0.0, domain = Int);
    variable!(m, wk, lb = 0.0, ub = 10.0, Int);
    variable!(m, pk, lb = 0.0, initial = 3.0);
    variable!(m, qk, fix = 5.0);

    println!("{m}\n");
}

/// Parameters.
fn parameters() -> Result<(), Box<dyn std::error::Error>> {
    println!("== parameters ==");
    let m = Model::new("pricing");
    param!(m, p1 = 0.0);

    variable!(m, 0.0 <= x1 <= 10.0);
    objective!(m, Max, p1 * x1);

    for price in [1.0, 1.6, 2.0] {
        p1.set_param_value(price);
        let result = Highs.solve(&m, &HighsOptions::default())?;
        println!("{price} -> {:?}", result.objective());
    }
    println!();
    Ok(())
}

/// Index sets.
fn index_sets() {
    println!("== index sets ==");
    let plants = Set::strings(["seattle", "san-diego"]);

    set!(items = 0..5); // range normalized to Set<usize>
    set!(routes = plants * plants); // Cartesian product

    // Comprehension: product domain + by-value `if`. These two are equivalent.
    set!(arcs = (p, q) in &plants * &plants if p != q); // single tuple pattern
    set!(arcs2 = i in plants, j in plants if i != j); // multi-bind product

    // The typed filter is also a Set method (the receiver pins the key type).
    let diag = (&plants * &plants).filter_typed(|(p, q)| p == q);

    // Sparse integer leaf set.
    let sparse = Set::from_ints([0, 2, 4, 8]);

    println!(
        "items={}, routes={}, arcs={}, arcs2={}, diag={}, sparse={}\n",
        items.len(),
        routes.len(),
        arcs.len(),
        arcs2.len(),
        diag.len(),
        sparse.len()
    );
}

/// Indexed variables.
fn indexed_variables() {
    println!("== indexed variables ==");
    let plants = Set::strings(["seattle", "san-diego"]);
    let markets = Set::strings(["nyc", "chicago"]);
    let m = Model::new("transport");
    set!(routes = plants * markets);

    variable!(m, x[r in routes] >= 0.0); // one var per route

    // Per-key bounds may reference the index.
    variable!(m, 0.0 <= w[(p, q) in routes] <= capacity_for(&p, &q));

    set!(items = 0..5);
    let cap = [3.0, 5.0, 2.0, 8.0, 4.0];
    variable!(m, v[k in items], lb = 0.0, ub = cap[k]);

    // Filtered family: keep only matching keys.
    set!(mm = markets * markets);
    variable!(m, d[(i, j) in mm if i == j] >= 0.0);

    // Scalar lookup: any type that converts to IndexKey works.
    let e1 = x[("seattle", "nyc")];
    println!("x[seattle,nyc] = {}\n", m.display_expr(e1));
}

/// Expressions.
fn expressions() {
    println!("== expressions ==");
    let m = Model::new("expr");
    variable!(m, x >= 0.0);
    variable!(m, y >= 0.0);
    variable!(m, z >= 0.0);

    let lhs = x + 2.0 * y - 3.0 * z;
    let rhs = 4.0 * x + 5.0;

    println!("lhs = {}", m.display_expr(lhs));
    println!("rhs = {}\n", m.display_expr(rhs));
}

/// Summing over sets.
fn summing() {
    println!("== summing over sets ==");
    let m = Model::new("summing");

    set!(items = 0..4);
    let weights = [1.0, 2.0, 1.5, 0.5];
    let capacity = 10.0;
    variable!(m, x[i in items] >= 0.0);
    constraint!(m, cap, sum!(weights[i] * x[i] for i in items) <= capacity);

    let plants = Set::strings(["seattle", "san-diego"]);
    let markets = Set::strings(["nyc", "chicago"]);
    set!(routes = plants * markets);
    variable!(m, f[r in routes] >= 0.0);
    let total_cost = sum!(unit_cost(&p, &q) * f[p, q] for p in plants, q in markets);

    // Filtered sum.
    let online = [true, false, true, true];
    let active = sum!(x[i] for i in 0..4 if online[i]);

    objective!(m, Min, total_cost + active);
    println!("obj = {}\n", m.display_objective());
}

/// Constraints.
fn constraints() {
    println!("== constraints ==");
    let m = Model::new("constraints");
    variable!(m, x >= 0.0);
    variable!(m, y >= 0.0);

    constraint!(m, cap, 2.0 * x + 3.0 * y <= 100.0);
    constraint!(m, demand, x >= 5.0);
    constraint!(m, balance, x - y == 0.0);
    constraint!(m, band, 1.0 <= x + y <= 10.0); // two-sided range -> one constraint

    println!("{m}\n");
}

/// A model has exactly one objective.
fn objectives() {
    println!("== objectives ==");
    let m = Model::new("obj");
    variable!(m, x >= 0.0);
    variable!(m, y >= 0.0);
    constraint!(m, cap, x + y <= 10.0);

    objective!(m, Min, 3.0 * x + 5.0 * y); // also Max / Minimize/min / Maximize/max
    println!("obj = {}\n", m.display_objective());
}

/// Rule-style constraints.
fn rule_constraints() {
    println!("== rule-style constraints ==");
    let m = Model::new("rules");
    let plants = Set::strings(["seattle", "san-diego"]);
    let markets = Set::strings(["nyc", "chicago"]);
    set!(routes = plants * markets);
    variable!(m, x[r in routes] >= 0.0);

    // Scalar set: one constraint per period.
    const T: usize = 3;
    let periods = Set::range(0..T);
    let capacity = 100.0;
    variable!(m, prod[t in periods] >= 0.0);
    variable!(m, run[t in periods], Bin);
    constraint!(m, setup[t in periods], prod[t] <= capacity * run[t]);

    // Tuple set + inner sum builds the LHS expression.
    constraint!(m, supply[p in plants], sum!(x[p, q] for q in markets) <= supply_of(&p));

    // Filtered family: only keys passing the guard are built.
    set!(mm = markets * markets);
    variable!(m, g[(i, j) in mm] >= 0.0);
    constraint!(m, diag[(i, j) in mm if i == j], g[i, j] <= 1.0);

    // Computed run-time name.
    let p = "seattle";
    variable!(m, inflow[k in plants] >= 0.0);
    variable!(m, outflow[k in plants] >= 0.0);
    constraint!(m, name = format!("bal_{p}"), inflow[p] - outflow[p] == 0.0);

    let c = m.constraint_id("supply[seattle]").unwrap();
    println!("supply[seattle]: {}\n", m.display_constraint(c));
}

/// Nonlinear expressions.
fn nonlinear() {
    println!("== nonlinear expressions ==");

    // Rosenbrock NLP.
    let m = Model::new("rosenbrock");
    variable!(m, x);
    variable!(m, y);
    objective!(m, Min, (1.0 - x).powi(2) + 100.0 * (y - x.powi(2)).powi(2));
    println!("rosenbrock -> {:?}", m.kind());

    // Quadratic constraint (QCP).
    let m2 = Model::new("disk");
    variable!(m2, x);
    variable!(m2, y);
    constraint!(m2, disk, x.powi(2) + y.powi(2) <= 1.0);
    objective!(m2, Max, x + y);
    println!("disk       -> {:?}", m2.kind());

    // Second-order cone ||(x, y)|| <= t (SOCP).
    let m3 = Model::new("cone");
    variable!(m3, x);
    variable!(m3, y);
    variable!(m3, t >= 0.0);
    soc_constraint!(m3, cone, [x, y] <= t);
    objective!(m3, Min, t);
    println!("cone       -> {:?}", m3.kind());

    // Transcendental utility (NLP; MINLP when any variable is integer).
    let m4 = Model::new("utility");
    set!(items = 0..3);
    let u = [1.0, 2.0, 1.5];
    let w = [0.5, 0.3, 0.8];
    variable!(m4, x[i in items] >= 0.0);
    constraint!(m4, budget, sum!(x[i] for i in items) <= 10.0);
    objective!(
        m4,
        Max,
        sum!(u[i] * (1.0 + w[i] * x[i]).log() for i in items)
    );
    println!("utility    -> {:?}", m4.kind());
}
