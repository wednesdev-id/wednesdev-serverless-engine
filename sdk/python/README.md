# Wednes Python SDK

This directory contains the Python SDK scaffolding and a basic example (`app.py`) for the `wednes:function` WebAssembly component interface.

## Requirements

Ensure you have Python 3.10+ installed.

1. Install dependencies:

   ```bash
   pip install -r requirements.txt
   ```
   *or* using `pyproject.toml` via `pip install .`

## Building

To convert the example Python app into a WebAssembly component (`target.wasm`), run the following command from this directory:

```bash
componentize-py -w ../../wit componentize app -o target.wasm
```

### Explanation of the build command
- `componentize-py`: The Bytecode Alliance CLI tool to convert a Python application into a WebAssembly component.
- `-w ../../wit`: Points the tool to the `wit/` folder to resolve the WIT ABI. (Note: On recent versions of `componentize-py`, you may need to use `-d ../../wit` to specify the WIT directory instead of `-w`).
- `componentize app`: Tells the tool to target `app.py` as the application entry point.
- `-o target.wasm`: Outputs the built Wasm component to `target.wasm`.
