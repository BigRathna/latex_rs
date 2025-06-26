mod cli;
use clap::Parser;
use cli::Cli;
use latex_rs::compile;
use std::{error::Error, fs}; // so Cli::parse() is available

fn main() -> Result<(), Box<dyn Error>> {
    // parse args
    let cli = Cli::parse();

    // read input .tex
    let tex = fs::read_to_string(&cli.input)
        .map_err(|e| format!("Failed to read {}: {}", cli.input.display(), e))?;

    // compile to PDF bytes
    let pdf = compile(&tex).map_err(|e| format!("Compilation error: {}", e))?;

    // write output .pdf
    fs::write(&cli.output, &pdf)
        .map_err(|e| format!("Failed to write {}: {}", cli.output.display(), e))?;

    println!("Written PDF to {}", cli.output.display());
    Ok(())
}
