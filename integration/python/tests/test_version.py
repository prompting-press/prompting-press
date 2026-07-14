# Copyright (C) 2024-2026 Sjors Robroek
# SPDX-License-Identifier: AGPL-3.0-only

"""Integration: core_version() returns a non-empty string.

Principle I: the Rust engine version is exposed via core_version() so downstream
consumers can log/trace which engine version produced a given render.
"""

from __future__ import annotations

from prompting_press import core_version


def test_core_version_returns_non_empty_string() -> None:
    v = core_version()
    assert isinstance(v, str)
    assert len(v) > 0


def test_core_version_is_callable_multiple_times() -> None:
    # Idempotent — no side effects
    assert core_version() == core_version()
