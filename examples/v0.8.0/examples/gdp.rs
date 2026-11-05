//! Disjunct blocks, Big-M/Hull, logical selection, planning, and transformed copies.
//!
//! Run: cargo run --example gdp

use oximo::prelude::*;
use oximo::solvers::Highs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for method in [
        GdpMethod::from(BigM::default()),
        GdpMethod::from(Hull::default()),
    ] {
        unit_selection(method)?;
    }
    nested_selection()?;
    inclusive_hull()?;
    nonlinear_reference()?;
    Ok(())
}

fn unit_selection(method: GdpMethod) -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new("unit selection");
    variable!(model, 0.0 <= flow <= 10.0);
    variable!(model, 0.0 <= cost <= 2.0);
    let small = disjunct!(model, small, |d| {
        constraint!(d, capacity, flow <= 3.0);
        constraint!(d, fixed_cost, cost == 0.0);
    });
    let large = disjunct!(model, large, |d| {
        constraint!(d, operating_range, 5.0 <= flow <= 8.0);
        constraint!(d, fixed_cost, cost == 2.0);
    });
    disjunction!(model, unit, [small, large]);
    objective!(model, Max, flow - cost);
    let options = GdpReformulationOptions::new(method);

    // Planning validates the expansion without transforming the source.
    let plan = model.plan_gdp(options.clone())?;
    assert!(model.has_unreformulated_gdp());
    println!(
        "planned size: {:?}; Hull copies: {}",
        plan.total, plan.hull_copies
    );

    // Independent copies keep the source available for other methods/data.
    let transformed = model.to_reformulated_gdp_model(options.clone())?;
    assert!(model.has_unreformulated_gdp());
    let transformed_flow =
        transformed.variable_handle(flow.var_id().ok_or("flow is not a variable")?);
    let transformed_large = transformed.boolean_handle(large.id());
    let raw = Highs.solve(&transformed, &HighsOptions::default())?;
    assert!(
        (raw.value_of(transformed_flow)?
            .ok_or("missing transformed flow")?
            - 8.0)
            .abs()
            < 1e-6
    );
    assert_eq!(raw.boolean_value_of(transformed_large)?, Some(true));
    assert!(raw.value_of(flow).is_err());

    // The solver wrapper transforms in place and returns source-facing results.
    let result = Highs
        .with_gdp(options)
        .solve(&model, &HighsOptions::default())?;
    assert_eq!(result.termination, TerminationStatus::Optimal);
    assert!((result.objective().ok_or("missing objective")? - 6.0).abs() < 1e-6);
    assert!((result.value_of(flow)?.ok_or("missing flow")? - 8.0).abs() < 1e-6);
    assert!((result.value_of(cost)?.ok_or("missing cost")? - 2.0).abs() < 1e-6);
    assert_eq!(result.boolean_value_of(large)?, Some(true));
    println!(
        "objective {:?}; large selected {:?}",
        result.objective(),
        result.boolean_value_of(large)?
    );
    for report in model.gdp_reformulations().iter() {
        println!("{}", report.display(&model));
        for (copy, activation) in report.hull_variables.iter().zip(report.hull_activations()) {
            println!(
                "copy {:?}: activation {:?}",
                copy.generated,
                result.hull_activation_value_of(activation)?
            );
        }
    }
    Ok(())
}

fn nested_selection() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new("nested choices");
    variable!(model, 0.0 <= flow <= 10.0);
    let present = disjunct!(model, present, |d| {
        let a = disjunct!(d, technology_a, |child| {
            constraint!(child, flow <= 3.0);
        });
        let b = disjunct!(d, technology_b, |child| {
            constraint!(child, flow <= 8.0);
        });
        disjunction!(d, technology, [a, b]);
    });
    let absent = disjunct!(model, absent, |d| {
        constraint!(d, flow == 0.0);
    });
    let unit = disjunction!(model, unit, [present, absent]);
    boolean_variable!(model, enabled);
    logical_constraint!(model, present.implies(enabled));
    objective!(model, Max, flow);
    // Start with Big-M; configure this group's method explicitly.
    let options = GdpReformulationOptions::new(BigM::default()).with_method(unit, Hull::default());
    let result = Highs
        .with_gdp(options)
        .solve(&model, &HighsOptions::default())?;
    assert!((result.value_of(flow)?.ok_or("missing nested flow")? - 8.0).abs() < 1e-6);
    assert_eq!(result.boolean_value_of(enabled)?, Some(true));
    Ok(())
}

fn inclusive_hull() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new("inclusive Hull");
    variable!(model, 0.0 <= flow <= 4.0);
    boolean_variable!(model, left);
    boolean_variable!(model, right);
    disjunct_constraint!(model, left, flow <= 2.0);
    disjunct_constraint!(model, right, flow >= 1.0);
    disjunction!(model, [left, right], AtLeastOne);
    logical_constraint!(model, at_least(1, [left, right]));
    objective!(model, Max, flow);

    // Inclusive selection requires opt-in pattern enumeration for Hull.
    assert!(matches!(
        model.plan_gdp(Hull::default()),
        Err(GdpError::InclusiveHull(_))
    ));
    let hull =
        Hull::default().with_inclusive_policy(HullInclusivePolicy::Enumerate { max_patterns: 3 });
    let options = GdpReformulationOptions::new(hull).with_size_limits(GdpSizeLimits {
        variables: Some(10_000),
        constraints: Some(50_000),
        expression_nodes: Some(100_000),
    });
    let result = Highs
        .with_gdp(options)
        .solve(&model, &HighsOptions::default())?;
    assert!((result.value_of(flow)?.ok_or("missing inclusive flow")? - 4.0).abs() < 1e-6);
    assert_eq!(result.boolean_value_of(right)?, Some(true));
    Ok(())
}

fn nonlinear_reference() -> Result<(), Box<dyn std::error::Error>> {
    let model = Model::new("logarithmic Hull reference");
    variable!(model, 1.0 <= flow <= 4.0);
    boolean_variable!(model, active);
    disjunct_constraint!(model, active, flow.ln() <= 3.0_f64.ln());
    // Zero is outside log's domain. A positive reference keeps the perspective safe.
    let options =
        GdpReformulationOptions::new(Hull::default()).with_hull_reference_coordinate(flow, 1.0);
    let report = model.reformulate_gdp(options)?;
    println!("nonlinear reformulation (construction only): {report}");
    // A nonlinear backend is needed to solve the transformed model.
    Ok(())
}
