# Pandas is declared in requirements-dev.txt but not in requirements.txt
# This represents a case where a dependency is optional and should be reported as an error.
try:
    import pandas as pd

    print(pd.DataFrame([1, 2, 3]))
except ImportError:
    print("Expected ImportError In requirements.txt but not in requirements-dev.txt")
