mod condayml;
mod pixitoml;
mod pyprojecttoml;
mod requirementstxt;

use crate::error::AnalysisError;
pub use pep508_rs::Requirement as PyPIRequirement;

use crate::EnvironmentBuilderSource;

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

/// Represents a configuration of dependencies from the project
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    /// The dependencies for the configuration
    dependencies: Vec<Dependency>,

    /// The name of the configuration
    name: String,

    /// The source of the configuration
    source: EnvironmentBuilderSource,
}

impl Configuration {
    pub fn new(
        dependencies: Vec<Dependency>,
        name: String,
        source: EnvironmentBuilderSource,
    ) -> Self {
        Self {
            dependencies,
            name,
            source,
        }
    }
}

/// Extract the the different configurations of dependencies from the project
pub fn extract_configurations(
    source: EnvironmentBuilderSource,
) -> Result<Vec<Configuration>, AnalysisError> {
    // If the file is a pyproject.toml, use the PyProjectTomlParser
    match &source {
        EnvironmentBuilderSource::PyProjectToml(path) => {
            let pyproject = pyprojecttoml::parse(&path)?;
            let mut configurations = Vec::new();

            let configuration = Configuration::new(
                pyproject.required_dependencies().clone(),
                format!("{}", path.display().to_string()),
                source.clone(),
            );
            configurations.push(configuration);

            // Add all optional configurations
            for configuration in pyproject.optional_configurations() {
                let dependencies = pyproject.get_dependencies_for_configuration(&[configuration]);
                configurations.push(Configuration::new(
                    dependencies,
                    format!("{}[{}]", path.display().to_string(), configuration),
                    source.clone(),
                ));
            }
            Ok(configurations)
        }
        EnvironmentBuilderSource::RequirementsTxt(path) => {
            let dependencies = requirementstxt::parse(&path)?;
            let configuration =
                Configuration::new(dependencies, path.display().to_string(), source.clone());
            Ok(vec![configuration])
        }
        //EnvironmentBuilderSource::CondaEnvironmentYml => condayml::parse_dependencies_file(file_path),
        //EnvironmentBuilderSource::PixiToml => pixitoml::parse_dependencies_file(file_path),
        _ => Err(AnalysisError::UnsupportedProjectFormat(todo!())),
    }
}
