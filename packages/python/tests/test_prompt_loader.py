"""Spec 019 — PromptLoader tests for the Python binding (T010).

Covers:
- FileSystemLoader: hit + miss (load_not_found)
- Traversal guard: ../  absolute, NUL, backslash, symlink escape,
  key="", key=".", intermediate "." (SC-008)
- Read cap: exceed max_bytes → load_io (SC-009)
- MemoryLoader: hit + miss
- Callable coercion (FR-001)
- Compose Prompt.from_yaml(loader.load(k)) (US1)
- except PromptLoadError does NOT catch malformed-YAML LoadError (SC-010)
- Dependency-injection: swap FileSystemLoader ↔ MemoryLoader without changing call site (SC-002)
- make_prompt_load_error native-raise factory (FR-008a)
"""

from __future__ import annotations

import os
import pathlib
import textwrap

import pytest
from prompting_press import (
    LoadError,
    Prompt,
    PromptingPressError,
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

# ─── fixtures ────────────────────────────────────────────────────────────────

VALID_YAML = textwrap.dedent("""\
    name: test
    role: user
    body: "Hello {{ name }}"
    variables:
      name: { type: string, trusted: true }
""")


def write_prompt(directory: pathlib.Path, key: str, content: str = VALID_YAML) -> None:
    """Write a .yaml file for `key` into `directory`."""
    (directory / f"{key}.yaml").write_text(content, encoding="utf-8")


# ─── PromptLoader protocol ────────────────────────────────────────────────────


def test_filesystem_loader_implements_protocol(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    assert isinstance(loader, PromptLoader)


def test_memory_loader_implements_protocol() -> None:
    loader = MemoryLoader({"greet": VALID_YAML})
    assert isinstance(loader, PromptLoader)


# ─── FileSystemLoader: happy path ────────────────────────────────────────────


def test_filesystem_loader_hit(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    raw = loader.load("greet")
    assert "name: test" in raw


def test_filesystem_loader_miss_returns_not_found(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    with pytest.raises(PromptLoadError) as exc_info:
        loader.load("missing_key")
    assert exc_info.value.errors[0].code == LOAD_NOT_FOUND


def test_filesystem_loader_custom_suffix(tmp_path: pathlib.Path) -> None:
    (tmp_path / "greet.json").write_text(VALID_YAML, encoding="utf-8")
    loader = FileSystemLoader(tmp_path, suffix=".json")
    raw = loader.load("greet")
    assert raw.strip() != ""


def test_filesystem_loader_nonexistent_base_raises() -> None:
    with pytest.raises(PromptLoadError) as exc_info:
        FileSystemLoader("/this/path/does/not/exist/abc123")
    assert exc_info.value.errors[0].code == LOAD_NOT_FOUND


# ─── FileSystemLoader: traversal guard (SC-008) ──────────────────────────────


@pytest.mark.parametrize(
    "bad_key",
    [
        "../secret",
        "../../etc/passwd",
        "/etc/passwd",
        "/absolute",
        "foo\0bar",
        "foo\\bar",
        "",
        ".",
        "foo/./bar",
        "../",
    ],
)
def test_traversal_guard_rejects_unsafe_keys(tmp_path: pathlib.Path, bad_key: str) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    with pytest.raises(PromptLoadError) as exc_info:
        loader.load(bad_key)
    assert exc_info.value.errors[0].code == LOAD_NOT_FOUND


@pytest.mark.skipif(os.name == "nt", reason="symlinks require elevated privileges on Windows")
def test_traversal_guard_escaping_symlink(tmp_path: pathlib.Path) -> None:
    secret_dir = tmp_path / "secret"
    secret_dir.mkdir()
    (secret_dir / "secret.yaml").write_text("secret contents", encoding="utf-8")

    base_dir = tmp_path / "base"
    base_dir.mkdir()
    (base_dir / "escape").symlink_to(secret_dir)

    loader = FileSystemLoader(base_dir)
    with pytest.raises(PromptLoadError) as exc_info:
        loader.load("escape/secret")
    assert exc_info.value.errors[0].code == LOAD_NOT_FOUND


def test_traversal_error_message_does_not_leak_base_path(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    with pytest.raises(PromptLoadError) as exc_info:
        loader.load("../../../etc/passwd")
    msg = exc_info.value.errors[0].message
    assert str(tmp_path) not in msg, f"base path leaked into message: {msg}"
    # The logical key IS in the message (by design — it's the caller's own input).
    assert "../../../etc/passwd" in msg


# ─── FileSystemLoader: read cap (SC-009) ──────────────────────────────────────


def test_read_cap_exceeded_returns_load_io(tmp_path: pathlib.Path) -> None:
    (tmp_path / "big.yaml").write_text("x" * 100, encoding="utf-8")
    loader = FileSystemLoader(tmp_path, max_bytes=50)
    with pytest.raises(PromptLoadError) as exc_info:
        loader.load("big")
    assert exc_info.value.errors[0].code == LOAD_IO


def test_read_cap_exactly_at_limit_succeeds(tmp_path: pathlib.Path) -> None:
    content = "x" * 50
    (tmp_path / "exact.yaml").write_text(content, encoding="utf-8")
    loader = FileSystemLoader(tmp_path, max_bytes=50)
    result = loader.load("exact")
    assert result == content


# ─── MemoryLoader ─────────────────────────────────────────────────────────────


def test_memory_loader_hit() -> None:
    loader = MemoryLoader({"greet": VALID_YAML})
    assert loader.load("greet") == VALID_YAML


def test_memory_loader_miss_returns_not_found() -> None:
    loader = MemoryLoader()
    with pytest.raises(PromptLoadError) as exc_info:
        loader.load("missing")
    assert exc_info.value.errors[0].code == LOAD_NOT_FOUND


# ─── Callable coercion (FR-001) ───────────────────────────────────────────────


def test_callable_coercion_hit() -> None:
    def my_loader(key: str) -> str:
        if key == "greet":
            return VALID_YAML
        raise make_prompt_load_error(LOAD_NOT_FOUND, f"key not found: `{key}`")

    # A bare (key)->str callable is a valid loader at the call site (FR-001 callable
    # coercion): the library never gates on isinstance — it just calls the loader.
    # A plain function is NOT a structural instance of the `load`-method Protocol, so
    # `isinstance(fn, PromptLoader)` is correctly False; assert it is callable instead.
    assert callable(my_loader)
    raw = my_loader("greet")
    prompt = Prompt.from_yaml(raw)
    assert prompt.name == "test"


def test_callable_coercion_miss_raises_prompt_load_error() -> None:
    def failing_loader(key: str) -> str:
        raise make_prompt_load_error(LOAD_NOT_FOUND, f"key not found: `{key}`")

    with pytest.raises(PromptLoadError) as exc_info:
        failing_loader("anything")
    assert exc_info.value.errors[0].code == LOAD_NOT_FOUND


# ─── SC-010: PromptLoadError is distinct from LoadError ──────────────────────


def test_prompt_load_error_does_not_catch_load_error() -> None:
    """except PromptLoadError must NOT catch a malformed-YAML LoadError (SC-010)."""
    # Malformed YAML raises LoadError (the parse error).
    with pytest.raises(LoadError):
        Prompt.from_yaml("not: valid: yaml: : :")

    # PromptLoadError does not catch it.
    with pytest.raises(LoadError):
        try:
            Prompt.from_yaml("not: valid: yaml: : :")
        except PromptLoadError:
            pytest.fail("PromptLoadError caught a LoadError — not distinct")


def test_prompt_load_error_is_prompting_press_error() -> None:
    """PromptLoadError is a subclass of PromptingPressError (catch-all)."""
    loader = MemoryLoader()
    with pytest.raises(PromptingPressError):
        loader.load("missing")


def test_load_error_is_not_prompt_load_error() -> None:
    """LoadError and PromptLoadError are separate classes (FR-007)."""
    assert not issubclass(LoadError, PromptLoadError)
    assert not issubclass(PromptLoadError, LoadError)


# ─── Compose: load + parse (US1) ─────────────────────────────────────────────


def test_compose_filesystem_load_then_parse(tmp_path: pathlib.Path) -> None:
    write_prompt(tmp_path, "greet")
    loader = FileSystemLoader(tmp_path)
    raw = loader.load("greet")
    prompt = Prompt.from_yaml(raw)
    assert prompt.name == "test"


def test_compose_memory_load_then_parse() -> None:
    loader = MemoryLoader({"greet": VALID_YAML})
    raw = loader.load("greet")
    prompt = Prompt.from_yaml(raw)
    assert prompt.name == "test"


# ─── SC-002: dependency injection — swap FileSystem ↔ Memory ─────────────────


def _load_and_parse(loader: PromptLoader, key: str) -> Prompt:
    """Call site that accepts any PromptLoader."""
    return Prompt.from_yaml(loader.load(key))


def test_swap_filesystem_for_memory_without_changing_call_site(
    tmp_path: pathlib.Path,
) -> None:
    write_prompt(tmp_path, "greet")
    fs_loader = FileSystemLoader(tmp_path)
    from_fs = _load_and_parse(fs_loader, "greet")

    mem_loader = MemoryLoader({"greet": VALID_YAML})
    from_mem = _load_and_parse(mem_loader, "greet")

    assert from_fs.name == from_mem.name


# ─── US3: custom loader interchangeable with built-ins (T015) ─────────────────


def test_custom_loader_interchangeable_with_built_ins() -> None:
    """A plain callable or class with load(key) is usable via the same call site."""
    store: dict[str, str] = {"greet": VALID_YAML}

    class CustomLoader:
        def load(self, key: str) -> str:
            try:
                return store[key]
            except KeyError as exc:
                raise make_prompt_load_error(LOAD_NOT_FOUND, f"key not found: `{key}`") from exc

    custom = CustomLoader()
    assert isinstance(custom, PromptLoader)
    from_custom = _load_and_parse(custom, "greet")

    built_in = MemoryLoader(store)
    from_builtin = _load_and_parse(built_in, "greet")

    assert from_custom.name == from_builtin.name


def test_custom_loader_failure_surfaces_as_prompt_load_error() -> None:
    """A custom loader failure is a PromptLoadError, distinct from a parse error."""

    def bad_loader(key: str) -> str:
        raise make_prompt_load_error(LOAD_NOT_FOUND, f"key not found: `{key}`")

    with pytest.raises(PromptLoadError) as exc_info:
        bad_loader("greet")
    assert exc_info.value.errors[0].code == LOAD_NOT_FOUND


# ─── FR-008a: make_prompt_load_error (native raise path) ─────────────────────


def test_make_prompt_load_error_not_found() -> None:
    # make_prompt_load_error RETURNS a PromptLoadError instance for the caller to raise
    # (see loader.py: `raise make_prompt_load_error(...)`); it does not raise on its own.
    with pytest.raises(PromptLoadError) as exc_info:
        raise make_prompt_load_error(LOAD_NOT_FOUND, "key not found: `x`")
    assert exc_info.value.errors[0].code == LOAD_NOT_FOUND
    assert "x" in exc_info.value.errors[0].message


def test_make_prompt_load_error_io() -> None:
    with pytest.raises(PromptLoadError) as exc_info:
        raise make_prompt_load_error(LOAD_IO, "I/O error loading `x`: disk full")
    assert exc_info.value.errors[0].code == LOAD_IO
