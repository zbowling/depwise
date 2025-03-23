# Test project details

pyproject_package test details:

* Uses pyproject.toml project with a src layout.
* Project has a declared dependency numpy and it's used in __init__.py which should not be reported as an error.
* Project has imports a non-existant hello module in hello.py which should be an error with depwise since it's not declared in pyproject.toml or exists in the project.
* Project has a child module that is imported in a few ways in hello.py which should not be reported as an error.
