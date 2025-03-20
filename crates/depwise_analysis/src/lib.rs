pub mod env_backend;
pub mod env_parser;
pub mod error;
pub mod parser;

pub use error::AnalysisError;
use std::path::Path;
use std::path::PathBuf;
use toml::Value;
/// A file that can be used to extract dependencies from to build up an environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentBuilderSource {
    CondaEnvironmentYml(PathBuf),
    PixiToml(PathBuf),
    PyProjectToml(PathBuf),
    RequirementsTxt(PathBuf),
}

impl EnvironmentBuilderSource {
    pub fn infer_from_source_path(path: &Path) -> Result<Self, AnalysisError> {
        if path.is_dir() {
            let pyproject_toml = path.join("pyproject.toml");
            if pyproject_toml.exists() {
                // Check if the pyproject.toml is a poetry project or has a [project] section
                if let Ok(pyproject_toml_content) = std::fs::read_to_string(&pyproject_toml) {
                    if let Ok(toml_value) = pyproject_toml_content.parse::<Value>() {
                        if toml_value.get("project").is_some_and(|v| v.is_table()) {
                            return Ok(Self::PyProjectToml(pyproject_toml));
                        }
                    }
                }
            }
            let requirements_txt = path.join("requirements.txt");
            if requirements_txt.exists() {
                return Ok(Self::RequirementsTxt(requirements_txt));
            }
            let conda_environment_yml = path.join("environment.yml");
            if conda_environment_yml.exists() {
                return Ok(Self::CondaEnvironmentYml(conda_environment_yml));
            }
        }

        Err(AnalysisError::NoProjectOrRequirementsFile(
            path.to_string_lossy().to_string(),
        ))
    }
}

pub enum EnvironmentBackend {
    Auto,
    Simulated,
    UV,
    Pixi,
    Current,
}

pub struct Analysis {
    found_imports: Vec<String>,
    unused_imports: Vec<String>,
    missing_imports: Vec<String>,
}

impl Default for Analysis {
    fn default() -> Self {
        Self {
            found_imports: vec![],
            unused_imports: vec![],
            missing_imports: vec![],
        }
    }
}

pub fn analyze_project(
    mut environment_builder_source: Option<EnvironmentBuilderSource>,
    backend: EnvironmentBackend,
    path: &Path,
) -> Result<Analysis, AnalysisError> {
    // If the environment_builder_source is None we can try to infer it from the path
    if environment_builder_source.is_none() {
        match EnvironmentBuilderSource::infer_from_source_path(path) {
            Ok(inferred_source) => {
                environment_builder_source = Some(inferred_source);
            }
            Err(e) => {
                println!("Error inferring environment builder source: {:?}", e);
            }
        }
    }

    if let Some(environment) = environment_builder_source {
        let dependencies = env_parser::parse_dependency_file(environment)?;
        println!("dependencies: {:?}", dependencies);
    }

    let analysis = Analysis::default();
    Ok(analysis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::fmt::format::FmtSpan;

    fn init_tracing() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .with_span_events(FmtSpan::CLOSE)
            .try_init();
    }

    #[test]
    fn test_parse_relative_imports() -> Result<(), AnalysisError> {
        init_tracing();
        Ok(())
    }
}
