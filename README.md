# Depwise

A fast, comprehensive dependency analyzer for Python projects that detects unused, missing, and optional dependencies across multiple environments. Supports requirements.txt, conda, pyproject.toml, pixi, and setup.py without requiring installation. Features intelligent pattern detection for optional dependencies, synthetic fast-pass analysis, and validation against actual environments. Designed for pre-commit hooks and CI/CD pipelines.

![Depwise Logo](depwise_logo.png)

## Usage


To check a project, you can use the `depwise check` command.

```bash
depwise check --project <path-to-pyproject.toml> <path to source code>
depwise check --requirements <path-to-requirements.txt> <path to source code>
depwise check --condayml <path-to-environment.yml> <path to source code>
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

To make it easier to test multiple environments, Depwise can be configured using a `depwise.toml` or `pyproject.toml` file.

```toml
[depwise]
# Ignore these module imports
ignore = ["dep1", "dep2"]

[depwise.environment.myproject]
type = "pyproject" # or "requirements", "conda", "pixi"
# We will try to infer the path from the type given common conventions,
# but you can specify it here
path = "path/to/pyproject.toml"

# Test with and without specified optional-dependencies
extras = ["*"] # or specify a list of extras to test ["extra1", "extra2"]
```
