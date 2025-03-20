use pep508_rs::Pep508Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("Failed to parse file: {0}. Error reading line {1} column {2}")]
    ParseFileError(String, String, String),
    #[error("Failed to parse pyproject.toml: {0}")]
    PyProjectTomlError(String),
    #[error("Unsupported dependency file: {0}")]
    UnsupportedDependencyFile(String),
    #[error("Failed to read file {0}: {1}")]
    FileReadError(String, String),
    #[error("Failed to parse dependency {0}")]
    DependencyParseError(String),
    #[error("No project or requirements file could be automatically discovered in {0}")]
    NoProjectOrRequirementsFile(String),
}

impl From<Pep508Error> for AnalysisError {
    fn from(error: Pep508Error) -> Self {
        AnalysisError::DependencyParseError(error.to_string())
    }
}
