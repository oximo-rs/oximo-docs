//! I/O: export a model to MPS, LP, and NL.
//!
//! Runnable companion to the docs "I/O" page. The docs show the 0.5 path-based
//! writers; on oximo 0.4.0 the `write_*` functions take a `Write` sink, so this
//! example opens files in the temp dir and hands them the writer.
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
    println!("mps: {} bytes, lp: {} bytes, nl: {} bytes", mps.len(), lp.len(), nl.len());
    println!("\n--- LP ---\n{lp}");

    // To a file: 0.4.0 writers take a `Write` sink.
    let dir = std::env::temp_dir();
    io::write_mps(&m, &mut File::create(dir.join("model.mps"))?)?;
    io::write_lp(&m, &mut File::create(dir.join("model.lp"))?)?;
    io::write_nl(&m, &mut File::create(dir.join("model.nl"))?)?;

    // NL options: pick the format and attach solver metadata. `format` is a
    // public field on `WriteOptions` in 0.4.0.
    let opts = io::WriteOptions { format: io::NlFormat::Ascii, ..Default::default() };
    io::write_nl_with(&m, &mut File::create(dir.join("model_ascii.nl"))?, &opts)?;

    // Companion `.col` / `.row` name files for AMPL-compatible solvers.
    io::write_nl_files(&m, &dir.join("model"), &opts)?;

    println!("\nwrote MPS/LP/NL files to {}", dir.display());
    Ok(())
}
