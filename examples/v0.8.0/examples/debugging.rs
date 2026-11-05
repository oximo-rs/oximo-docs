//! Original constraint handles, stored rows, ownership checks, and SDP kinds.
//!
//! Run: cargo run --example debugging

use oximo::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("display adapters");
    variable!(m, x >= 0.0);
    variable!(m, y >= 0.0);
    constraint!(m, c1, x * y <= y + 3.0);
    let handle = m.constraint_handle("c1").ok_or("missing c1")?;
    println!("original: {}", m.display_constraint(handle)?);
    let row = m.constraint_id("c1").ok_or("missing c1 row")?;
    println!("stored row: {}", m.display_row(row));
    assert_eq!(m.display_expr(2.0 * x - y).to_string(), "2 x - y");

    constraint!(m, supply[i in 0..2], x + i as f64 * y <= 5.0);
    let supply = m
        .constraint_handle("supply[1]")
        .ok_or("missing indexed row")?;
    println!("indexed: {}", m.display_constraint(supply)?);
    let foreign = Model::new("foreign");
    variable!(foreign, z);
    let foreign_row = constraint!(foreign, z <= 1.0);
    assert!(m.display_constraint(foreign_row).is_err());

    let sdp = Model::new("PSD classification");
    symmetric_variable!(sdp, matrix[2]);
    let positivity = psd_constraint!(sdp, positivity, &matrix);
    objective!(sdp, Min, matrix.trace());
    assert_eq!(sdp.kind(), ModelKind::SDP);
    println!("PSD model kind: {:?}", sdp.kind());
    sdp.set_psd_active(positivity, false)?;
    assert_eq!(sdp.kind(), ModelKind::LP);
    sdp.set_psd_active(positivity, true)?;
    println!("{sdp}");
    Ok(())
}
