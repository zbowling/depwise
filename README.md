# Depwise

> Depwise is pre-alpha software. It is not ready.

A fast, comprehensive dependency analyzer for Python projects that detects unused, missing, and optional dependencies across multiple environments. Supports requirements.txt, conda, pyproject.toml, pixi, and setup.py without requiring installation. Features intelligent pattern detection for optional dependencies, synthetic fast-pass analysis, and validation against actual environments. Designed for pre-commit hooks and CI/CD pipelines.

![Depwise Logo](depwise_logo.png)

## Usage


To check a project, you can use the `depwise check` command.

```bash
# Check a project using a pyproject.toml file
depwise check --project <path-to-pyproject.toml> <path to source code>

# Check a project using a requirements.txt file. Explictly treat the path as a module and not recursively check subdirectories.
depwise check --requirements <path-to-requirements.txt> --module <path to source code>

# Check a project using a conda environment file. Explictly treat the path as a project and recursively check subdirectories.
depwise check --condayml <path-to-environment.yml> --project <path to source code>
```

To check source code in the currently active Python environment, you can use the `depwise check` command with the `--current` flag.

```bash
depwise check --current <path to source code>
```

To check a wheel, sdist, or conda package, you can use the `depwise check-package` command.

```bash
depwise check-package <path-to-package>
```

## Configuration

To make it easier to test multiple environments, Depwise can be configured using a `depwise.toml` or `pyproject.toml` (using the `[tool.depwise]` prefix) file.

```toml
[depwise]
# Ignore these module imports

# Configure a project
[depwise.project.my-project]
type = "pyproject" # or "requirements", "conda", "pixi"
# We will try to infer the path from the type given common conventions,
# but you can specify it here
project = "path/to/pyproject.toml"

# Test with and without specified optional-dependencies/extras
extras = ["*"] # or specify a list of extras to test ["extra1", "extra2"]

# Array of paths to sources to check
source = ["path/to/source"]

# List of regexes to ignore imports matching these patterns
ignore = ["^test_.*"]

# Ignore modules by name
ignore-imports = ["dep1", "dep2"]

[depwise.package.default]
# Test with and without specified optional-dependencies/extras
extras = ["*"] # or specify a list of extras to test ["extra1", "extra2"]

# List of regexes to ignore imports matching these patterns
ignore = ["^test_.*"]

# Ignore modules by name
ignore-imports = ["dep1", "dep2"]



```
