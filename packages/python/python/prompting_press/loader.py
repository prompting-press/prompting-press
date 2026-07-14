# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Pluggable prompt loader — spec 019.

:class:`PromptLoader` is the loader interface (a :pep:`544` ``Protocol``). The two built-in
implementations are :class:`FileSystemLoader` and :class:`MemoryLoader`.

The loader is a **pure I/O leaf**: ``load(key)`` returns raw text only — never a
:class:`~prompting_press.Prompt`, never parsed. Construction and loading remain separate,
composable steps::

    loader = FileSystemLoader("/prompts")
    prompt = Prompt.from_yaml(loader.load("greet"))

Use :class:`MemoryLoader` to dependency-inject prompt text in tests, replacing the
filesystem loader without changing call sites.

Custom loaders implement the :class:`PromptLoader` protocol or are any callable
``(key: str) -> str``. Failures must raise :class:`~prompting_press.PromptLoadError` with
the appropriate code (``load_not_found`` or ``load_io``) via
:func:`~prompting_press.make_prompt_load_error`.

Error codes
-----------
Both codes are in the ``PromptLoadError`` exception (``exc.errors[0].code``):

- ``load_not_found`` — key absent from backing store.
- ``load_io`` — I/O error or ``max_bytes`` exceeded.

Security (traversal guard + read cap)
--------------------------------------
:class:`FileSystemLoader` validates the final resolved path (including suffix) against a
canonicalized base directory. Rejected keys:

- contain ``..`` components
- are absolute
- contain NUL bytes or backslashes
- are empty or equal to ``"."``
- contain an intermediate ``"."`` component (e.g. ``"foo/./bar"``)
- resolve (after symlink expansion) to a path outside the base

A missing-target canonicalize failure yields ``load_not_found`` (not ``load_io``). A file
exceeding ``max_bytes`` yields ``load_io``.
"""

from __future__ import annotations

import pathlib
from collections.abc import Callable
from typing import Protocol, runtime_checkable

# Import from the compiled extension submodule directly to avoid a circular
# import: loader.py is part of the prompting_press package, so importing from
# 'prompting_press' at the top level would be circular; the compiled extension
# lives at 'prompting_press.prompting_press' (the PyO3 cdylib).
from .prompting_press import (  # type: ignore[attr-defined]
    PromptLoadError,
    make_prompt_load_error,
)

__all__ = [
    "LOAD_IO",
    "LOAD_NOT_FOUND",
    "FileSystemLoader",
    "MemoryLoader",
    "PromptLoader",
]

# Stable code constants (mirrors Rust ``error::code``).
LOAD_IO: str = "load_io"
LOAD_NOT_FOUND: str = "load_not_found"

# Default max file size (1 MiB) — mirrors Rust ``FileSystemLoader::DEFAULT_MAX_BYTES``.
_DEFAULT_MAX_BYTES: int = 1 << 20


# ─── PromptLoader protocol ────────────────────────────────────────────────────


@runtime_checkable
class PromptLoader(Protocol):
    """Pluggable source of raw prompt text (spec 019, FR-001).

    A single ``load(key)`` operation maps a logical key to the raw text of a prompt
    definition. The returned text is **never parsed or validated** — that belongs to the
    construct-from-text path (``Prompt.from_yaml`` etc.).

    Any callable ``(key: str) -> str`` satisfies this protocol (FR-001 callable coercion):
    no struct is required.

    Failures must raise :class:`~prompting_press.PromptLoadError` — never return empty
    string or ``None`` for a missing key (FR-006).
    """

    def load(self, key: str) -> str:
        """Return the raw text for ``key``, or raise :class:`~prompting_press.PromptLoadError`.

        Parameters
        ----------
        key:
            A logical identifier for the prompt (e.g. ``"greet"``). Not a file path.

        Returns
        -------
        str
            The raw text content (YAML/JSON/TOML — callers decide the format).

        Raises
        ------
        PromptLoadError
            ``load_not_found`` when the key is absent; ``load_io`` on I/O error or
            ``max_bytes`` exceeded.
        """
        ...


# Callable coercion: a plain function/lambda also satisfies the loader contract.
# Type alias for documentation purposes.
LoaderCallable = Callable[[str], str]
AnyLoader = PromptLoader | LoaderCallable


def _raise_not_found(key: str) -> PromptLoadError:
    """Raise ``PromptLoadError`` with code ``load_not_found``."""
    raise make_prompt_load_error(LOAD_NOT_FOUND, f"key not found: `{key}`")


def _raise_io(key: str, detail: str) -> PromptLoadError:
    """Raise ``PromptLoadError`` with code ``load_io``."""
    raise make_prompt_load_error(LOAD_IO, f"I/O error loading `{key}`: {detail}")


# ─── Key validation (traversal guard) ────────────────────────────────────────


def _validate_key(key: str) -> None:
    """Raise :class:`~prompting_press.PromptLoadError` (``load_not_found``) for unsafe keys.

    Validates key before path construction (FR-002a/FR-002b/SC-008):
    - NUL bytes
    - Backslash (Windows separator / UNC)
    - Absolute paths
    - Empty key or bare ``"."``
    - Any path component that is not a plain name (``".."``, ``"."``, root, drive)
    """
    if "\0" in key:
        _raise_not_found(key)
    if "\\" in key:
        _raise_not_found(key)
    if not key or key == ".":
        _raise_not_found(key)
    # Check each component is a plain Normal name.
    p = pathlib.PurePosixPath(key)
    if p.is_absolute():
        _raise_not_found(key)
    for part in p.parts:
        if part in ("..", "."):
            _raise_not_found(key)


# ─── FileSystemLoader ─────────────────────────────────────────────────────────


class FileSystemLoader:
    """Load prompt files from a configured base directory (spec 019, FR-002).

    Maps a logical key to ``{base}/{key}{suffix}`` and returns the file's raw text.

    Parameters
    ----------
    base:
        The directory under which prompt files live. Canonicalized at construction.
    suffix:
        File name suffix appended to every key (default ``".yaml"``).
    max_bytes:
        Maximum file size in bytes (default 1 MiB). A file exceeding this limit raises
        ``PromptLoadError`` with code ``load_io``.

    Raises
    ------
    PromptLoadError
        ``load_not_found`` if ``base`` does not exist; ``load_io`` on other OS errors.
    """

    DEFAULT_MAX_BYTES: int = _DEFAULT_MAX_BYTES

    def __init__(
        self,
        base: str | pathlib.Path,
        suffix: str = ".yaml",
        max_bytes: int = _DEFAULT_MAX_BYTES,
    ) -> None:
        raw = pathlib.Path(base)
        try:
            self._base: pathlib.Path = raw.resolve(strict=True)
        except FileNotFoundError:
            _raise_not_found(str(raw))
        except OSError:
            _raise_io(str(raw), "cannot canonicalize base directory")
        self._suffix = suffix
        self._max_bytes = max_bytes

    @property
    def base(self) -> pathlib.Path:
        """The canonicalized base directory."""
        return self._base

    @property
    def suffix(self) -> str:
        """The file name suffix (e.g. ``".yaml"``)."""
        return self._suffix

    @property
    def max_bytes(self) -> int:
        """The maximum file size in bytes."""
        return self._max_bytes

    def load(self, key: str) -> str:
        """Load the prompt source for ``key``.

        Parameters
        ----------
        key:
            Logical identifier; treated as a relative path under :attr:`base`.

        Returns
        -------
        str
            The raw file contents.

        Raises
        ------
        PromptLoadError
            - ``load_not_found``: key not found, traversal rejected, or missing target.
            - ``load_io``: OS error, metadata read failure, or ``max_bytes`` exceeded.
        """
        # --- traversal guard (FR-002a/FR-002b/SC-008) ---
        _validate_key(key)

        # Build candidate: {base}/{key}{suffix}.
        candidate = self._base / (key + self._suffix)

        # Canonicalize — resolves symlinks; missing target → not_found.
        try:
            resolved = candidate.resolve(strict=True)
        except FileNotFoundError:
            _raise_not_found(key)
        except OSError:
            _raise_io(key, "path resolution failed")

        # Symlink-escape check: resolved must be a descendant of self._base.
        try:
            resolved.relative_to(self._base)
        except ValueError:
            _raise_not_found(key)

        # --- read cap (FR-016/SC-009) ---
        try:
            file_size = resolved.stat().st_size
        except OSError:
            _raise_io(key, "cannot read file metadata")

        if file_size > self._max_bytes:
            _raise_io(
                key,
                f"file size ({file_size} bytes) exceeds max_bytes ({self._max_bytes})",
            )

        # --- read ---
        try:
            return resolved.read_text(encoding="utf-8")
        except OSError:
            _raise_io(key, "failed to read file")

        # Unreachable — all paths above either return or raise.
        raise AssertionError("unreachable")  # pragma: no cover


# ─── MemoryLoader ─────────────────────────────────────────────────────────────


class MemoryLoader:
    """Load prompt source from an in-memory key→text mapping (spec 019, FR-003).

    The primary use case is **dependency injection in tests**: production code uses a
    :class:`FileSystemLoader` or a custom loader; tests substitute a :class:`MemoryLoader`
    with hard-coded prompt text. No filesystem access is performed.

    Parameters
    ----------
    prompts:
        A mapping of logical key → raw prompt text.
    """

    def __init__(self, prompts: dict[str, str] | None = None) -> None:
        self._map: dict[str, str] = dict(prompts) if prompts is not None else {}

    def load(self, key: str) -> str:
        """Return the mapped text for ``key``, or raise ``PromptLoadError(load_not_found)``.

        Parameters
        ----------
        key:
            Logical identifier; matched exactly against the mapping keys.

        Raises
        ------
        PromptLoadError
            ``load_not_found`` when ``key`` is absent from the mapping.
        """
        try:
            return self._map[key]
        except KeyError:
            _raise_not_found(key)
        raise AssertionError("unreachable")  # pragma: no cover
