mod check;
mod check_package;

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
/// CLI for depwise
#[derive(Debug, Parser)]
#[command(name = "depwise", version, author, about)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Check(CheckArgs),
    CheckPackage(CheckPackageArgs),
}

#[derive(Debug, Args)]
#[group(required = false, multiple = false)]
pub struct Environment {
    /// Path to the pyproject.toml file
    #[arg(short, long, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    pyproject: Option<PathBuf>,

    /// Path to the requirements.txt file
    #[arg(short, long, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    requirements: Option<PathBuf>,

    /// Path to Conda environment file
    #[arg(short, long, value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    condayml: Option<PathBuf>,

    /// Current environment to use for validation.
    /// A `python3` bin from the environment must be on the $PATH.
    #[arg(short = 'e', long)]
    current_environment: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum EnvironmentBackend {
    /// Automatically choose the best backend for the environment file
    /// and tools installed.
    Auto,

    /// Use a simulated in-memory environment to validate all dependencies.
    /// This is the fastest option but not as accurate.
    Simulated,

    /// Use UV to create a new environment and validate all dependencies.
    /// Fast but only supports pypi dependencies.
    UV,

    /// Use Pixi to create a new environment and validate all dependencies.
    /// This allows testing conda, pypi, and pip dependencies.
    Pixi,

    /// Use the current python environment to validate all dependencies.
    /// This is useful if you want to test the dependencies already activated
    /// and installed in your current environment.
    Current,
}

impl From<EnvironmentBackend> for depwise_analysis::EnvironmentBackend {
    fn from(backend: EnvironmentBackend) -> Self {
        match backend {
            EnvironmentBackend::Auto => depwise_analysis::EnvironmentBackend::Auto,
            EnvironmentBackend::Simulated => depwise_analysis::EnvironmentBackend::Simulated,
            EnvironmentBackend::UV => depwise_analysis::EnvironmentBackend::UV,
            EnvironmentBackend::Pixi => depwise_analysis::EnvironmentBackend::Pixi,
            EnvironmentBackend::Current => depwise_analysis::EnvironmentBackend::Current,
        }
    }
}

/// Check a wheel, sdist, or conda package that all declared dependencies match what is used in the package.
#[derive(Debug, Parser)]
#[command(name = "check-package")]
#[command(about = "Check a wheel, sdist, or conda package")]
pub struct CheckPackageArgs {
    /// Path to the package
    #[arg(value_hint = clap::ValueHint::FilePath, value_name = "FILE", required = true)]
    package: PathBuf,

    /// Backend to use for checking dependencies
    #[arg(long, value_enum, default_value = "auto")]
    backend: EnvironmentBackend,

    /// Package extras (python wheel or sdist only)
    #[arg(long, name = "extra", value_name = "EXTRA")]
    extras: Vec<String>,
}

/// Subcommand for checking dependencies
#[derive(Debug, Parser)]
#[command(name = "check")]
#[command(about = "Check a project")]
pub struct CheckArgs {
    /// Path to the project src root
    #[arg(default_value = ".")]
    path: PathBuf,

    #[command(flatten)]
    environment: Environment,

    /// Backend to use for checking dependencies
    #[arg(long, value_enum, default_value = "auto")]
    backend: EnvironmentBackend,
}

pub fn execute(args: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match args.command {
        Commands::Check(check_args) => check::execute(check_args),
        Commands::CheckPackage(check_package_args) => check_package::execute(check_package_args),
    }
}
