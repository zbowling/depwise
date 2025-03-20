# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "numpy",
#     "torch",
# ]
# ///

import sys
import json
import site
import pkgutil

import_dump = {}

# get the standard library path
std_lib_path = sys.stdlib_module_names

# get all the modules in the standard library
import_dump["stdlib"] = sorted(std_lib_path)

import_dump["builtin"] = sorted(sys.builtin_module_names)

import_dump["site-packages"] = []
import_dump["user-site-packages"] = []


def load_all_modules_from_dir(key, dirname):
    for pkg in pkgutil.iter_modules(dirname):
        if pkg.ispkg:
            import_dump[key].append(pkg.name)


# get current site-packages path
site_packages_paths = site.getsitepackages()

load_all_modules_from_dir("site-packages", site_packages_paths)

print(json.dumps(import_dump, indent=4))
