use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use toml::Value;

use crate::error::AnalysisError;

use pep508_rs::Requirement;

use crate::env_parser::Dependency;

fn parse_dependency_string(dep_str: &str) -> Result<Dependency, AnalysisError> {
    let requirement = Requirement::from_str(dep_str)
        .map_err(|e| AnalysisError::PyProjectTomlError(e.to_string()))?;
    Ok(Dependency::PyPI(requirement))
}

fn extract_dependencies_from_table(table: &Value) -> Result<Vec<Dependency>, AnalysisError> {
    let mut dependencies = Vec::new();
    let mut required_dependencies = Vec::new();
    let mut extra_dependencies: HashMap<String, Vec<Dependency>> = HashMap::new();

    if let Some(project_table) = table.get("project") {
        // Handle dependencies section
        if let Some(deps) = project_table.get("dependencies") {
            match deps {
                Value::Array(dep_array) => {
                    for dep in dep_array {
                        if let Value::String(dep_str) = dep {
                            let dep = parse_dependency_string(dep_str)?;
                            dependencies.push(dep.clone());
                            required_dependencies.push(dep);
                        }
                    }
                }
                Value::Table(dep_table) => {
                    for (name, version) in dep_table {
                        if let Value::String(version_str) = version {
                            let dep_str = format!("{} {}", name, version_str);
                            let dep = parse_dependency_string(&dep_str)?;
                            dependencies.push(dep.clone());
                            required_dependencies.push(dep);
                        }
                    }
                }
                _ => {
                    return Err(AnalysisError::PyProjectTomlError(
                        "Invalid dependencies format".to_string(),
                    )
                    .into())
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
                                extra_dependencies
                                    .entry(group.clone())
                                    .or_insert(Vec::new())
                                    .push(dep.clone());
                                dependencies.push(dep);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(dependencies)
}

pub(crate) fn parse_dependencies_file(file_path: &Path) -> Result<Vec<Dependency>, AnalysisError> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| AnalysisError::PyProjectTomlError(e.to_string()))?;
    parse_dependencies(&content)
}

pub(crate) fn parse_dependencies(contents: &str) -> Result<Vec<Dependency>, AnalysisError> {
    let toml_value: Value = contents
        .parse()
        .map_err(|_| AnalysisError::PyProjectTomlError("Invalid TOML".to_string()))?;
    extract_dependencies_from_table(&toml_value)
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
        let deps = parse_dependencies(content)?;

        assert_eq!(deps.len(), 2);
        match &deps[0] {
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
        let deps = parse_dependencies(content)?;

        assert_eq!(deps.len(), 2);
        match &deps[0] {
            Dependency::PyPI(req) => {
                assert_eq!(req.name.as_ref(), "requests");
            }
            _ => panic!("Expected a PyPI dependency"),
        };

        Ok(())
    }
}
