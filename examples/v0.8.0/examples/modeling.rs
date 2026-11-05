//! Changed modeling APIs: typed expressions, semantic constraints, and builders.
//!
//! Run: cargo run --example modeling

use oximo::prelude::*;
use oximo::solvers::Highs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    constraint_updates()?;
    parameter_updates()?;
    typed_expressions_and_aggregations()?;
    affine_builders()?;
    reformulation_reports()?;
    Ok(())
}

fn constraint_updates() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("constraint updates");
    variable!(m, x >= 0.0);
    variable!(m, y >= 0.0);
    let cap = m.add_constraint(
        "capacity",
        Constraint::new((2.0 * x + y).into_function(), LessThan(100.0)),
    );
    m.set_constraint_set(cap, LessThan(8.0))?;
    m.set_constraint_function(cap, (2.0 * x).into_function())?;
    m.set_constraint_active(cap, false)?;
    m.set_constraint_active(cap, true)?;
    objective!(m, Max, x);

    println!("original declaration: {}", m.display_constraint(cap)?);
    println!("stored row: {}", m.display_row(cap.row_id()));
    let result = Highs.solve(&m, &HighsOptions::default())?;
    assert!((result.value_of(x)?.ok_or("missing x")? - 4.0).abs() < 1e-6);
    Ok(())
}

fn parameter_updates() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("checked parameter updates");
    param!(m, price = 1.0);
    variable!(m, 0.0 <= quantity <= 10.0);
    objective!(m, Max, price * quantity);
    for value in [1.0, 1.6, 2.0] {
        price.set_param_value(value)?;
        let result = Highs.solve(&m, &HighsOptions::default())?;
        println!("price {value}: objective {:?}", result.objective());
    }
    // Invalid writes fail without replacing the last valid binding.
    assert!(price.set_param_value(f64::NAN).is_err());
    Ok(())
}

fn typed_expressions_and_aggregations() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("typed expressions");
    variable!(m, x[i in 0..3] >= 0.0);
    variable!(m, t >= 0.0);
    let component = x[0]
        .powi(1)
        .try_affine()
        .map_err(|class| format!("expected affine component, found {class:?}"))?;
    soc_constraint!(m, cone, [component] <= t);
    assert!(x[0].square().try_affine().is_err());
    let quadratic = x[0]
        .powi(2)
        .try_quadratic()
        .map_err(|class| format!("expected quadratic expression, found {class:?}"))?;
    println!("quadratic: {}", m.display_expr(quadratic));

    // A model-owned empty sum is the additive identity.
    let maybe_empty = Vec::<usize>::new();
    constraint!(m, empty_balance, sum!(x[i] for i in &maybe_empty) == 0.0);
    let zero = sum!(m, x[i] for i in &maybe_empty);
    println!("empty sum: {}", m.display_expr(zero));
    let least = min!(x[i] for i in 0..3);
    let peak = max!(x[i] for i in 0..3 if i != 1);
    println!(
        "extrema: {}, {}",
        m.display_expr(least),
        m.display_expr(peak)
    );
    Ok(())
}

fn affine_builders() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("incremental terms");
    variable!(m, x[i in 0..3] >= 0.0);
    let mut builder = m.affine_builder();
    for (i, weight) in [2.0, 3.0, 4.0].into_iter().enumerate() {
        builder.add_term(weight, x[i]);
    }
    builder.add_constant(5.0);
    constraint!(m, capacity, builder.build() <= 100.0);
    // build() resets pending terms and retains scratch capacity.
    builder.add_term(1.0, x[0]);
    constraint!(m, reused, builder.build() <= 3.0);

    let mut compensated = m.compensated_affine_builder();
    for weight in [1e16, 1.0, -1e16] {
        compensated.add_term(weight, x[0]);
    }
    let retained = compensated.build();
    objective!(m, Max, x[0]);
    let result = Highs.solve(&m, &HighsOptions::default())?;
    assert!((result.value_of(retained)?.ok_or("missing retained value")? - 3.0).abs() < 1e-6);
    println!("compensated expression: {}", m.display_expr(retained));
    Ok(())
}

fn reformulation_reports() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("portable indicators and SOS");
    variable!(m, enabled, Binary);
    variable!(m, 0.0 <= x <= 10.0);
    variable!(m, 0.0 <= y <= 10.0);
    indicator_constraint!(m, capacity, enabled == 1 => x <= 8.0);
    indicator_constraint!(m, shutdown, enabled == 0 => x == 0.0);
    let choice = sos_constraint!(m, choice, SOS1, [x, y]);
    objective!(m, Max, 2.0 * x + y);

    for report in m.reformulate_indicators(IndicatorReformulationOptions::default())? {
        println!("{}", report.display(&m));
    }
    // A single SOS handle can be reformulated independently.
    let report = choice
        .reformulate(&m, SosReformulationOptions::default())?
        .ok_or("SOS was already inactive")?;
    println!("{}", report.display(&m));
    let result = Highs.solve(&m, &HighsOptions::default())?;
    assert!((result.objective().ok_or("missing objective")? - 16.0).abs() < 1e-6);
    Ok(())
}
