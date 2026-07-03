"""Run every CrewAI docs sample under ``examples/`` as a standalone program.

Mirrors docs/site/samples/python/tests/test_examples.py. Kept as a separate
project because CrewAI's chromadb/Pydantic-v1 stack cannot import on Python 3.14
(this project pins 3.13). Each file under ``examples/`` is the verbatim artifact
a docs page shows via ``?raw`` and is executed here; assertions live inside each
example, so the faithful way to run them is as a script.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

EXAMPLES_DIR = Path(__file__).resolve().parent.parent / "examples"

EXAMPLES = sorted(p for p in EXAMPLES_DIR.glob("*.py") if not p.name.startswith("_"))

assert EXAMPLES, f"no example programs found under {EXAMPLES_DIR}"


@pytest.mark.parametrize("example", EXAMPLES, ids=lambda p: p.name)
def test_example_runs(example: Path) -> None:
    result = subprocess.run(
        [sys.executable, str(example)],
        cwd=EXAMPLES_DIR,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, (
        f"{example.name} exited {result.returncode}\n"
        f"--- stdout ---\n{result.stdout}\n"
        f"--- stderr ---\n{result.stderr}"
    )
