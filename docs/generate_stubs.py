"""Generate documentation from .pyi stubs."""

import os
import sys
from pathlib import Path


def generate_stubs_docs():
    """Copy .pyi files to a location mkdocstrings can find them."""
    project_root = Path(__file__).parent.parent
    source_dir = project_root / "python" / "structpath"

    docs_dir = project_root / "docs"

    # Create a stubs directory in the Python path
    stubs_dir = docs_dir / "structpath"
    os.makedirs(stubs_dir, exist_ok=True)

    for pyi_file in source_dir.glob("*.pyi"):
        py_file = stubs_dir / f"{pyi_file.stem}.py"
        with open(pyi_file, "r") as f_in, open(py_file, "w") as f_out:
            f_out.write(f_in.read())

    print(f"Generated stubs in: {stubs_dir}")
    print(f"Current Python path: {sys.path}")


if __name__ == "__main__":
    generate_stubs_docs()
