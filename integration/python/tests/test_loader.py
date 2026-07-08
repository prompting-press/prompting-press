"""Integration: loader surface — MemoryLoader, FileSystemLoader, callable, make_prompt_load_error.

Spec 019: MemoryLoader hit/miss; FileSystemLoader hit + traversal rejection + read-cap;
callable as loader; make_prompt_load_error; PromptLoadError distinct from LoadError;
loader.load() returns raw text (compose with Prompt.from_yaml).
"""

from __future__ import annotations

import pathlib
import textwrap

import pytest

from prompting_press import (
    LoadError,
    Prompt,
    PromptLoadError,
    make_prompt_load_error,
)
from prompting_press.loader import (
    LOAD_IO,
    LOAD_NOT_FOUND,
    FileSystemLoader,
    MemoryLoader,
    PromptLoader,
)

VALID_YAML = textwrap.dedent("""\
    name: test
    role: user
    body: "Hello {{ name }}"
    variables:
      name: { type: string, trusted: true }
""")


def write_prompt(directory: pathlib.Path, key: str, content: str = VALID_YAML) -> None:
    (directory / f"{key}.yaml").write_text(content, encoding="utf-8")


# ─── MemoryLoader ─────────────────────────────────────────────────────────────


def test_memory_loader_hit() -> None:
    loader = MemoryLoader({"greet": VALID_YAML})
    assert loader.load("greet") == VALID_YAML


def test_memory_loader_miss_raises_prompt_load_error() -> None:
    loader = MemoryLoader({"greet": VALID_YAML})
    with pytest.raises(PromptLoadError) as excinfo:
        loader.load("missing")
    assert excinfo.value.errors[0].code == LOAD_NOT_FOUND


def test_memory_loader_implements_protocol() -> None:
    loader = MemoryLoader({"greet": VALID_YAML})
    assert isinstance(loader, PromptLoader)


# ─── FileSystemLoader ─────────────────────────────────────────────────────────


def test_filesystem_loader_hit(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    text = loader.load("greet")
    assert text == VALID_YAML


def test_filesystem_loader_miss_raises_prompt_load_error(
    tmp_path: pathlib.Path,
) -> None:
    loader = FileSystemLoader(tmp_path)
    with pytest.raises(PromptLoadError) as excinfo:
        loader.load("missing")
    assert excinfo.value.errors[0].code == LOAD_NOT_FOUND


def test_filesystem_loader_implements_protocol(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    assert isinstance(loader, PromptLoader)


def test_filesystem_loader_traversal_dotdot_rejected(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    with pytest.raises(PromptLoadError):
        loader.load("../greet")


def test_filesystem_loader_traversal_absolute_rejected(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    with pytest.raises(PromptLoadError):
        loader.load("/etc/passwd")


def test_filesystem_loader_traversal_nul_byte_rejected(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    with pytest.raises(PromptLoadError):
        loader.load("greet\x00etc/passwd")


def test_filesystem_loader_traversal_empty_key_rejected(
    tmp_path: pathlib.Path,
) -> None:
    loader = FileSystemLoader(tmp_path)
    with pytest.raises(PromptLoadError):
        loader.load("")


def test_filesystem_loader_read_cap_exceeded(tmp_path: pathlib.Path) -> None:
    # Write a file bigger than the cap
    big_content = "x" * 100
    (tmp_path / "big.yaml").write_text(big_content, encoding="utf-8")
    loader = FileSystemLoader(tmp_path, max_bytes=10)
    with pytest.raises(PromptLoadError) as excinfo:
        loader.load("big")
    assert excinfo.value.errors[0].code == LOAD_IO


# ─── callable as loader (FR-001) ─────────────────────────────────────────────


def test_callable_as_loader() -> None:
    # A plain callable (key)->str satisfies the loader contract without needing
    # to be isinstance(fn, PromptLoader) — the Protocol method is `load`, not __call__
    store = {"greet": VALID_YAML}

    def my_loader(key: str) -> str:
        if key not in store:
            raise make_prompt_load_error(LOAD_NOT_FOUND, f"key not found: {key}")
        return store[key]

    # Pass the callable directly — works without wrapping in a class
    text = my_loader("greet")
    p = Prompt.from_yaml(text)
    assert p.name == "test"


# ─── make_prompt_load_error ───────────────────────────────────────────────────


def test_make_prompt_load_error_returns_promptloaderror() -> None:
    # make_prompt_load_error RETURNS (does not self-raise); caller must raise it
    err = make_prompt_load_error(LOAD_NOT_FOUND, "test message")
    assert isinstance(err, PromptLoadError)
    assert err.errors[0].code == LOAD_NOT_FOUND


def test_make_prompt_load_error_io_code() -> None:
    err = make_prompt_load_error(LOAD_IO, "io error message")
    assert isinstance(err, PromptLoadError)
    assert err.errors[0].code == LOAD_IO


def test_make_prompt_load_error_must_be_raised() -> None:
    # Demonstrates the raise pattern
    with pytest.raises(PromptLoadError):
        raise make_prompt_load_error(LOAD_NOT_FOUND, "demo")


# ─── PromptLoadError distinct from LoadError ─────────────────────────────────


def test_prompt_load_error_is_distinct_from_load_error() -> None:
    # PromptLoadError is for loader failures; LoadError is for parse/schema failures
    assert PromptLoadError is not LoadError


def test_catching_prompt_load_error_does_not_catch_load_error() -> None:
    # Malformed YAML raises LoadError, NOT PromptLoadError
    with pytest.raises(LoadError):
        Prompt.from_yaml("name: [unterminated")

    # Catching PromptLoadError alone must not catch this LoadError
    caught = False
    try:
        Prompt.from_yaml("name: [unterminated")
    except PromptLoadError:
        caught = True
    except LoadError:
        pass
    assert not caught, "PromptLoadError must not catch LoadError"


# ─── compose: loader.load + Prompt.from_yaml ─────────────────────────────────


def test_compose_memory_loader_with_from_yaml() -> None:
    loader = MemoryLoader({"greet": VALID_YAML})
    text = loader.load("greet")
    p = Prompt.from_yaml(text)
    assert p.name == "test"
    assert p.body == "Hello {{ name }}"


def test_compose_filesystem_loader_with_from_yaml(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    text = loader.load("greet")
    p = Prompt.from_yaml(text)
    assert p.name == "test"
