# Test project details

flat source test details:

* Project has a flat source layout with no pyproject.toml file.
* Some of the source has a dependency on requests which is declared in requirements.txt. This should not be reported as an error.
* import_pandas.py imports pandas which is declared in requirements-dev.txt but not in requirements.txt. This should report an error.
* import_pandas_in_try.py imports pandas which is declared in requirements-dev.txt but not in requirements.txt. This represents a case where a dependency is optional and should not be reported as an error.
* import_non_existant_module.py tries to import a non-existant module. This should report an error.
