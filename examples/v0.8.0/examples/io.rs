//! Active PSD blocks are rejected by MPS, LP, and NL; inactive blocks are skipped.
//!
//! Run: cargo run --example io

use oximo::io;
use oximo::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("PSD export limits");
    symmetric_variable!(m, matrix[2]);
    let positivity = psd_constraint!(m, positivity, &matrix);
    objective!(m, Min, matrix.trace());
    assert!(matches!(io::to_mps_string(&m), Err(io::IoError::Conic)));
    assert!(matches!(io::to_lp_string(&m), Err(io::IoError::Conic)));
    assert!(matches!(io::to_nl_string(&m), Err(io::IoError::Conic)));
    println!("all three writers rejected the active PSD block");

    m.set_psd_active(positivity, false)?;
    for (format, text) in [
        ("MPS", io::to_mps_string(&m)?),
        ("LP", io::to_lp_string(&m)?),
        ("NL", io::to_nl_string(&m)?),
    ] {
        println!("{format}: {} bytes after deactivating PSD", text.len());
    }
    Ok(())
}
