mod condayml;
mod pixitoml;
mod pyprojecttoml;
mod requirementstxt;

use crate::error::AnalysisError;
pub use pep508_rs::Requirement as PyPIRequirement;

use std::path::PathBuf;
/// Implements a match spec for Conda packages. Follows the rules in https://github.com/conda/conda/blob/main/conda/models/match_spec.py#L569
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CondaMatchSpec {
    /// The package name
    name: String,
    /// The raw spec string that was parsed
    raw_spec: String,
}

impl CondaMatchSpec {
    /// Create a new CondaMatchSpec from a spec string
    pub fn new(spec: &str) -> Self {
        // Extract just the package name from the spec
        // Handle cases like:
        // - package
        // - package=1.0
        // - package>=1.0
        // - package[build=*]
        // - channel::package
        let raw_spec = spec.trim().to_string();
        let name = Self::extract_name(&raw_spec);

        Self { name, raw_spec }
    }

    /// Extract the package name from a conda spec string
    fn extract_name(spec: &str) -> String {
        // Remove any channel prefix (everything before ::)
        let without_channel = spec.split("::").last().unwrap_or(spec);

        // Remove any version constraints and build specs
        let name_part = without_channel
            .split(|c: char| c == '=' || c == '>' || c == '<' || c == '~' || c == '[')
            .next()
            .unwrap_or(without_channel)
            .trim()
            .to_string();

        name_part
    }

    /// Get the package name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the original raw spec string
    pub fn raw_spec(&self) -> &str {
        &self.raw_spec
    }
}

/// Represents a Python package dependency with its version requirements
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dependency {
    /// A dependency on a PyPI package
    PyPI(PyPIRequirement),
    /// A dependency on a Conda package
    Conda(CondaMatchSpec),
    /// Explict URL dependency where we know for sure what the package name is (e.g. https://example.com/package-1.0.0-py3-none-any.whl)
    PackageUrl(String),
    /// Explict path dependency where we don't know for sure what the package name is (e.g. /path/to/package-1.0.0-py3-none-any.whl)
    PackagePath(PathBuf),
}

pub fn parse_dependency_file(
    source: crate::EnvironmentBuilderSource,
) -> Result<Vec<Dependency>, AnalysisError> {
    // If the file is a pyproject.toml, use the PyProjectTomlParser
    match source {
        crate::EnvironmentBuilderSource::PyProjectToml(path) => {
            pyprojecttoml::parse_dependencies_file(&path)
        }
        crate::EnvironmentBuilderSource::RequirementsTxt(path) => {
            requirementstxt::parse_dependencies_file(&path)
        }
        //EnvironmentBuilderSource::CondaEnvironmentYml => condayml::parse_dependencies_file(file_path),
        //EnvironmentBuilderSource::PixiToml => pixitoml::parse_dependencies_file(file_path),
        _ => Err(AnalysisError::UnsupportedDependencyFile(todo!())),
    }
}
