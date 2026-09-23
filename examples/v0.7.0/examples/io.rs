//! I/O: export a model to MPS, LP, and NL.
//!
//! Runnable companion to the development docs "I/O" page, including file
//! export and import for MPS, LP, and NL.
//!
//!   cargo run --example io

use oximo::io;
use oximo::prelude::*;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = Model::new("blend");
    variable!(m, x >= 0.0);
    variable!(m, 0.0 <= y <= 4.0);
    constraint!(m, cap, x + 2.0 * y <= 14.0);
    objective!(m, Max, 3.0 * x + 4.0 * y);

    // To a string: each returns a `String` you can log, hash, or pass on.
    let mps = io::to_mps_string(&m)?;
    let lp = io::to_lp_string(&m)?;
    let nl = io::to_nl_string(&m)?;
    println!(
        "mps: {} bytes, lp: {} bytes, nl: {} bytes",
        mps.len(),
        lp.len(),
        nl.len()
    );
    println!("\n--- LP ---\n{lp}");

    // To a file: these writers accept any `Write` sink.
    let dir = std::env::temp_dir();
    let mps_path = dir.join("model.mps");
    let lp_path = dir.join("model.lp");
    let nl_path = dir.join("model.nl");
    io::write_mps(&m, &mut File::create(&mps_path)?)?;
    io::write_lp(&m, &mut File::create(&lp_path)?)?;
    io::write_nl(&m, &mut File::create(&nl_path)?)?;

    // NL options: pick the format and attach solver metadata.
    let opts = io::WriteOptions {
        format: io::NlFormat::Ascii,
        ..Default::default()
    };
    io::write_nl_with(&m, &mut File::create(dir.join("model_ascii.nl"))?, &opts)?;

    // Companion `.col` / `.row` name files for AMPL-compatible solvers.
    io::write_nl_files(&m, &dir.join("model"), &opts)?;

    // The readers accept paths directly; LP and MPS carry this linear model.
    let _mps_roundtrip = io::read_mps_file(&mps_path)?;
    let _lp_roundtrip = io::read_lp_file(&lp_path)?;
    let _nl_roundtrip = io::read_nl_file(&nl_path)?;

    println!("\nwrote MPS/LP/NL files to {}", dir.display());
    Ok(())
}
