use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use toml::Value;

use crate::error::AnalysisError;

use pep508_rs::Requirement;

use crate::project::Dependency;

fn parse_dependency_string(dep_str: &str) -> Result<Dependency, AnalysisError> {
    let requirement = Requirement::from_str(dep_str)
        .map_err(|e| AnalysisError::PyProjectTomlError(e.to_string()))?;
    Ok(Dependency::PyPI(requirement))
}

pub struct PyProjectToml {
    /// All dependencies inclusive of all extras (excluding build dependencies)
    all_dependencies: Vec<Dependency>,
    /// Top level dependencies in the pyproject.toml file
    required_dependencies: Vec<Dependency>,
    /// Optional dependencies grouped by extra name
    optional_dependencies: HashMap<String, Vec<Dependency>>,
}

impl PyProjectToml {
    pub fn new() -> Self {
        Self {
            all_dependencies: Vec::new(),
            required_dependencies: Vec::new(),
            optional_dependencies: HashMap::new(),
        }
    }

    pub fn all_dependencies(&self) -> Vec<Dependency> {
        self.all_dependencies.clone()
    }

    pub fn required_dependencies(&self) -> &Vec<Dependency> {
        &self.required_dependencies
    }

    pub fn optional_configurations(&self) -> Vec<&str> {
        self.optional_dependencies
            .keys()
            .map(|s| s.as_str())
            .collect()
    }

    pub fn get_dependencies_for_configuration(&self, configurations: &[&str]) -> Vec<Dependency> {
        // extend all each optional dependency with the required dependencies
        let mut dependencies = self.required_dependencies.clone();
        for configuration in configurations {
            if let Some(deps) = self.optional_dependencies.get(&configuration.to_string()) {
                dependencies.extend(deps.clone());
            }
        }
        dependencies
    }
}

fn parse_table(table: &Value) -> Result<PyProjectToml, AnalysisError> {
    let mut pyprojecttoml = PyProjectToml::new();

    if let Some(project_table) = table.get("project") {
        // Handle dependencies section
        if let Some(deps) = project_table.get("dependencies") {
            match deps {
                Value::Array(dep_array) => {
                    for dep in dep_array {
                        if let Value::String(dep_str) = dep {
                            let dep = parse_dependency_string(dep_str)?;
                            pyprojecttoml.all_dependencies.push(dep.clone());
                            pyprojecttoml.required_dependencies.push(dep);
                        }
                    }
                }
                Value::Table(dep_table) => {
                    for (name, version) in dep_table {
                        if let Value::String(version_str) = version {
                            let dep_str = format!("{} {}", name, version_str);
                            let dep = parse_dependency_string(&dep_str)?;
                            pyprojecttoml.all_dependencies.push(dep.clone());
                            pyprojecttoml.required_dependencies.push(dep);
                        }
                    }
                }
                _ => {
                    return Err(AnalysisError::PyProjectTomlError(
                        "Invalid dependencies format".to_string(),
                    )
                    .into());
                }
            }
        }

        // Handle optional-dependencies section
        if let Some(optional_deps) = project_table.get("optional-dependencies") {
            if let Value::Table(optional_table) = optional_deps {
                for (group, deps) in optional_table {
                    if let Value::Array(dep_array) = deps {
                        for dep in dep_array {
                            if let Value::String(dep_str) = dep {
                                let dep = parse_dependency_string(dep_str)?;
                                pyprojecttoml
                                    .optional_dependencies
                                    .entry(group.clone())
                                    .or_insert(Vec::new())
                                    .push(dep.clone());
                                pyprojecttoml.all_dependencies.push(dep);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(pyprojecttoml)
}

pub(crate) fn parse(file_path: &Path) -> Result<PyProjectToml, AnalysisError> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| AnalysisError::PyProjectTomlError(e.to_string()))?;
    parse_contents(&content)
}

pub(crate) fn parse_contents(contents: &str) -> Result<PyProjectToml, AnalysisError> {
    let toml_value: Value = contents
        .parse()
        .map_err(|_| AnalysisError::PyProjectTomlError("Invalid TOML".to_string()))?;
    parse_table(&toml_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_dependencies() -> Result<(), AnalysisError> {
        let content = r#"
[project]
dependencies = [
    "requests >= 2.8.1",
    "flask == 1.0.0",
]
"#;
        let deps = parse_contents(content)?;

        assert_eq!(deps.all_dependencies.len(), 2);
        match &deps.all_dependencies[0] {
            Dependency::PyPI(req) => {
                assert_eq!(req.name.as_ref(), "requests");
            }
            _ => panic!("Expected a PyPI dependency"),
        };
        // Check that the dependencies are correct

        Ok(())
    }

    #[test]
    fn test_parse_dependencies_with_extras() -> Result<(), Box<dyn std::error::Error>> {
        let content = r#"
[project]
dependencies = [
    "requests[security,tests] >= 2.8.1",
]

[project.optional-dependencies]
dev = ["pytest >= 6.0.0"]
"#;
        let deps = parse_contents(content)?;

        assert_eq!(deps.all_dependencies.len(), 2);
        match &deps.all_dependencies[0] {
            Dependency::PyPI(req) => {
                assert_eq!(req.name.as_ref(), "requests");
            }
            _ => panic!("Expected a PyPI dependency"),
        };

        Ok(())
    }
}
