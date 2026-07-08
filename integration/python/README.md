# prompting-press Python integration tests

This directory is the pre-release gate for the Python binding. It is a real downstream consumer
project that depends on `packages/python` via an editable path dep and exercises the entire public
API surface of `prompting-press` as modular test files — one file per feature — so adding coverage
for a new feature is always one new `tests/test_<feature>.py` file.

To run: `cd integration/python && uv sync && uv run pytest -q`
