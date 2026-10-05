# Wednes Python SDK

This directory contains the Python SDK scaffolding and a basic example (`app.py`) for the `wednes:function` WebAssembly component interface.

## Requirements

Ensure you have Python 3.10+ installed.

1. Install dependencies via a virtual environment:

   ```bash
   python3 -m venv .venv
   source .venv/bin/activate
   python3 -m pip install componentize-py
   ```

*(Optional Workaround)*: `componentize_py` currently does not ship with a `__main__.py` entry point. To run it via `python3 -m componentize_py`, add it to the package:
```bash
cat << 'MAIN' > .venv/lib/python3.11/site-packages/componentize_py/__main__.py
import sys
from componentize_py import script

if __name__ == "__main__":
    sys.exit(script())
MAIN
```

## Building

To convert the example Python app into a WebAssembly component (`app.wasm`), run the following command from this directory:

```bash
python3 -m componentize_py -d ../../wit componentize app -o app.wasm
```
*(Alternatively, simply run the CLI tool: `componentize-py -d ../../wit componentize app -o app.wasm`)*

## Validation

Verify the compiled component:

```bash
wasm-tools validate app.wasm
```

### Explanation of the build command
- `componentize_py`: The Bytecode Alliance tool to convert a Python application into a WebAssembly component.
- `-d ../../wit`: Points the tool to the `wit/` folder to resolve the WIT ABI.
- `componentize app`: Tells the tool to target `app.py`, looking for the `WitWorld` duck-typed class implementing the target world's exports.
- `-o app.wasm`: Outputs the built Wasm component to `app.wasm`.
