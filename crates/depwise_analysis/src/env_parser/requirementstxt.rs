use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::env_parser::{Dependency, PyPIRequirement};
use crate::error::AnalysisError;

enum RequirementLine {
    Dependency(Dependency),
    RequirementFile(PathBuf),
    Url(String),
    Path(PathBuf),
    Noop,
}

/// Parse a single line from a requirements.txt file
fn parse_requirement_line(line: &str) -> Result<RequirementLine, AnalysisError> {
    let trimmed = line.trim();

    // trim off any trailing comments
    let trimmed = trimmed.rsplit_once("#").unwrap_or((trimmed, "")).0.trim();

    // Skip empty lines and comments
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return Ok(RequirementLine::Noop);
    }

    // if the line starts with -r, it's a requirement file
    if trimmed.starts_with("-r ") {
        return Ok(RequirementLine::RequirementFile(PathBuf::from(
            trimmed.split_whitespace().nth(1).unwrap(),
        )));
    }

    // ignore other - and -- options
    if trimmed.starts_with("-") {
        return Ok(RequirementLine::Noop);
    }

    // Parse the requirement
    match PyPIRequirement::from_str(trimmed) {
        Ok(requirement) => Ok(RequirementLine::Dependency(Dependency::PyPI(requirement))),
        Err(error) => {
            // If we can't parse the line as a PyPI requirement, check if it's a url or path

            // if the line starts with a protocol then it's a url
            if trimmed.starts_with("http:")
                || trimmed.starts_with("https:")
                || trimmed.starts_with("ftp:")
                || trimmed.starts_with("file:")
            {
                return Ok(RequirementLine::Url(trimmed.to_string()));
            }

            // if the line looks like a path then it's a path
            if trimmed.starts_with("/")
                || trimmed.starts_with(".")
                || trimmed.ends_with(".whl")
                || trimmed.ends_with(".tar.gz")
                || trimmed.ends_with(".zip")
                || trimmed.ends_with(".tar.bz2")
                || trimmed.ends_with(".tar")
                || trimmed.ends_with(".egg")
                || trimmed.ends_with(".tar.xz")
            {
                return Ok(RequirementLine::Path(PathBuf::from(trimmed)));
            }

            Err(error.into())
        }
    }
}

/// Parse a requirements.txt file and return a list of dependencies
pub(crate) fn parse_dependencies_file(file_path: &Path) -> Result<Vec<Dependency>, AnalysisError> {
    parse_dependencies_file_with_visited(&file_path, &mut Vec::new())
}

/// Helper function that tracks visited files to prevent infinite recursion
fn parse_dependencies_file_with_visited(
    file_path: &Path,
    visited: &mut Vec<PathBuf>,
) -> Result<Vec<Dependency>, AnalysisError> {
    // Check if we've already visited this file to prevent infinite recursion
    if visited.contains(&file_path.to_path_buf()) {
        return Err(AnalysisError::DependencyParseError(format!(
            "Circular dependency detected in requirements file: {}",
            file_path.display()
        )));
    }

    // Add this file to the visited list
    visited.push(file_path.to_path_buf());

    let content = fs::read_to_string(file_path).map_err(|e| {
        AnalysisError::FileReadError(file_path.to_string_lossy().to_string(), e.to_string())
    })?;

    parse_dependencies_with_visited(
        &content,
        file_path.parent().unwrap_or_else(|| Path::new(".")),
        visited,
    )
}

/// Parse requirements.txt content and return a list of dependencies
pub(crate) fn parse_dependencies(content: &str) -> Result<Vec<Dependency>, AnalysisError> {
    parse_dependencies_with_visited(content, Path::new("."), &mut Vec::new())
}

/// Helper function that tracks visited files to prevent infinite recursion
fn parse_dependencies_with_visited(
    content: &str,
    base_dir: &Path,
    visited: &mut Vec<PathBuf>,
) -> Result<Vec<Dependency>, AnalysisError> {
    let mut dependencies = Vec::new();

    for line in content.lines() {
        match parse_requirement_line(line)? {
            RequirementLine::Dependency(dep) => dependencies.push(dep),
            RequirementLine::RequirementFile(rel_path) => {
                let abs_path = base_dir.join(&rel_path);
                let deps = parse_dependencies_file_with_visited(&abs_path, visited)?;
                dependencies.extend(deps);
            }
            RequirementLine::Url(url) => {
                dependencies.push(Dependency::PackageUrl(url));
            }
            RequirementLine::Path(path) => {
                dependencies.push(Dependency::PackagePath(path));
            }
            RequirementLine::Noop => {}
        }
    }

    Ok(dependencies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_parse_simple_requirements() -> Result<(), AnalysisError> {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("requirements.txt");
        let mut file = File::create(&file_path).unwrap();

        writeln!(file, "requests==2.28.1").unwrap();
        writeln!(file, "# This is a comment").unwrap();
        writeln!(file, "flask>=2.0.0").unwrap();
        writeln!(file, "-r other-requirements.txt").unwrap();
        writeln!(file, "pandas~=1.5.0").unwrap();

        let other_file_path = dir.path().join("other-requirements.txt");
        let mut other_file = File::create(&other_file_path).unwrap();
        writeln!(other_file, "torch==2.6.0").unwrap();

        let deps = parse_dependencies_file(&file_path)?;
        assert_eq!(deps.len(), 4);

        // Test that we can parse the content directly
        let content = "requests==2.28.1\n# Comment\nflask>=2.0.0\n\npandas~=1.5.0";
        let deps = parse_dependencies(content)?;
        assert_eq!(deps.len(), 3);

        Ok(())
    }

    #[test]
    fn test_parse_complex_requirements() -> Result<(), AnalysisError> {
        let content = r#"
# This is a comment
requests==2.28.1    # Inline comment
flask>=2.0.0,<3.0.0
pandas~=1.5.0
numpy>=1.20.0; python_version>="3.8"
wxPathon @ http://wxpython.org/Phoenix/snapshot-builds/wxPython_Phoenix-3.0.3.dev1820+49a8884-cp34-none-win_amd64.whl
"#;
        let deps = parse_dependencies(content)?;
        assert_eq!(deps.len(), 5); // Should skip the -r line

        Ok(())
    }
}
