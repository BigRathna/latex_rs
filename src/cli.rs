use clap::Parser;
use std::path::PathBuf;

/// Command-line arguments for the `latexc` binary.
#[derive(Parser)]
#[command(name = "latexc")]
#[command(author = "Your Name <you@example.com>")]
#[command(version = "0.1.0")]
#[command(about = "Compile a .tex file to PDF using latex_rs library", long_about = None)]
pub struct Cli {
    /// Input TeX source file (.tex)
    #[arg(short, long, value_name = "FILE")]
    pub input: PathBuf,

    /// Output PDF file
    #[arg(short, long, value_name = "PDF")]
    pub output: PathBuf,
}
