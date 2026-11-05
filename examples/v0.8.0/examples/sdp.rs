//! PSD construction and optional SDP solves with matrix values and duals.
//!
//! Run: cargo run --example sdp
//!      cargo run --example sdp --features clarabel-sdp-mkl
//!      cargo run --example sdp --features mosek

use oximo::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("minimum trace");
    symmetric_variable!(m, matrix[2]);
    let positivity = psd_constraint!(m, positivity, &matrix);
    constraint!(m, off_diagonal, matrix[0, 1] == 1.0);
    objective!(m, Min, matrix.trace());
    assert_eq!(m.kind(), ModelKind::SDP);

    // Mirrored entries refer to the same scalar variable.
    assert_eq!(matrix[(0, 1)].var_id(), matrix[(1, 0)].var_id());
    let identity = SymmetricMatrix::<f64>::identity(2);
    let shifts = [0.0, 1.0, 2.0];
    psd_constraint!(m, shifted[i in 0..shifts.len()], &matrix + shifts[i] * &identity);
    let extra = psd_constraint!(m, name = "extra positivity".to_owned(), &matrix);
    m.set_psd_active(extra, false)?;
    println!("{m}");

    // The default run constructs the model without a BLAS/LAPACK dependency.
    #[cfg(feature = "clarabel-sdp")]
    {
        use oximo::solvers::Clarabel;
        let options = ClarabelOptions::default();
        Clarabel.validate_model(&m, &options).into_result()?;
        let result = Clarabel.solve(&m, &options)?;
        inspect_matrices(&result, &matrix, positivity, 2.0)?;
    }
    #[cfg(feature = "mosek")]
    {
        use oximo::solvers::Mosek;
        let options = MosekOptions::default();
        Mosek.validate_model(&m, &options).into_result()?;
        let result = Mosek.solve(&m, &options)?;
        inspect_matrices(&result, &matrix, positivity, 2.0)?;
    }
    #[cfg(not(any(feature = "clarabel-sdp", feature = "mosek")))]
    {
        let _ = positivity;
        println!("Enable a Clarabel SDP provider or MOSEK to solve and inspect PSD duals.");
    }

    // Affine matrix inequality: min t such that t I - C is PSD.
    let eigen = Model::new("largest eigenvalue");
    variable!(eigen, t);
    let c = SymmetricMatrix::try_from_rows([[2.0, 1.0], [1.0, 2.0]])?;
    let shifted = t * identity - &c;
    let bound = psd_constraint!(eigen, eigenvalue_bound, &shifted);
    objective!(eigen, Min, t);
    println!("C = {c}; expected largest eigenvalue = 3");
    #[cfg(feature = "clarabel-sdp")]
    {
        let result = oximo::solvers::Clarabel.solve(&eigen, &ClarabelOptions::default())?;
        inspect_matrices(&result, &shifted, bound, 3.0)?;
    }
    #[cfg(feature = "mosek")]
    {
        let result = oximo::solvers::Mosek.solve(&eigen, &MosekOptions::default())?;
        inspect_matrices(&result, &shifted, bound, 3.0)?;
    }
    #[cfg(not(any(feature = "clarabel-sdp", feature = "mosek")))]
    let _ = bound;
    Ok(())
}

#[cfg(any(feature = "clarabel-sdp", feature = "mosek"))]
fn inspect_matrices(
    result: &SolverResult,
    matrix: &SymmetricMatrix<Expr<'_, Affine>>,
    cone: PsdConstraintHandle,
    expected: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    assert!(result.has_solution());
    assert!((result.objective().ok_or("missing SDP objective")? - expected).abs() < 1e-5);
    let primal = result
        .value_of_matrix(matrix)?
        .ok_or("missing primal matrix")?;
    let dual = result.psd_dual_of(cone)?.ok_or("missing PSD dual")?;
    println!("primal = {primal}; dual = {dual}");
    println!("complementarity = {}", primal.frobenius(dual));
    assert!(primal.frobenius(dual).abs() < 1e-5);
    Ok(())
}
