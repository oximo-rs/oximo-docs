//! Validate complete models
//!
//! Run: cargo run --example solvers
//!      cargo run --example solvers --features clarabel,scip

use oximo::prelude::*;
use oximo::solvers::Highs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("validated LP");
    variable!(m, 0.0 <= x <= 10.0);
    objective!(m, Max, 2.0 * x);
    let options = HighsOptions::default();
    let mut solver = Highs;
    let validation = solver.validate_model(&m, &options);
    for diagnostic in &validation.errors {
        eprintln!("{}: {}", diagnostic.location(), diagnostic.reason);
    }
    validation.into_result()?;
    let result = solver.solve(&m, &options)?;
    assert_eq!(result.objective(), Some(20.0));
    print!("{}", result.report(&m)?);

    // HiGHS accepts MILP and continuous QP separately, but rejects MIQP.
    let miqp = Model::new("unsupported combination");
    variable!(miqp, 0.0 <= n <= 5.0, Int);
    objective!(miqp, Min, (n - 2.0).square());
    assert_eq!(miqp.kind(), ModelKind::MIQP);
    let rejected = solver.validate_model(&miqp, &options);
    assert!(!rejected.errors.is_empty());
    for diagnostic in &rejected.errors {
        println!(
            "expected rejection: {}: {}",
            diagnostic.location(),
            diagnostic.reason
        );
    }

    // PSD construction is independent of whether the backend supports SDP.
    let sdp = Model::new("unsupported PSD");
    symmetric_variable!(sdp, matrix[2]);
    psd_constraint!(sdp, &matrix);
    objective!(sdp, Min, matrix.trace());
    assert!(!solver.validate_model(&sdp, &options).errors.is_empty());

    #[cfg(feature = "clarabel")]
    {
        use oximo::solvers::Clarabel;
        Clarabel
            .validate_model(&m, &ClarabelOptions::default())
            .into_result()?;
        let result = Clarabel.solve(&m, &ClarabelOptions::default())?;
        assert!((result.objective().ok_or("missing Clarabel objective")? - 20.0).abs() < 1e-5);
        println!("Clarabel objective: {:?}", result.objective());
    }
    #[cfg(feature = "scip")]
    {
        use oximo::ScipOptions;
        use oximo::solvers::Scip;
        let mut scip = Scip::new();
        scip.validate_model(&miqp, &ScipOptions::default())
            .into_result()?;
        let result = scip.solve(&miqp, &ScipOptions::default())?;
        assert!(result.objective().ok_or("missing SCIP objective")?.abs() < 1e-5);
        println!("SCIP MIQP objective: {:?}", result.objective());
    }
    Ok(())
}
