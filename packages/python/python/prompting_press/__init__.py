"""Prompting Press — Python distribution.

The public API is provided by the compiled Rust extension (the PyO3 binding crate
``crates/prompting-press-py``), built and merged into this package by maturin. In the
mixed Rust/Python layout the extension lands as the submodule ``prompting_press.prompting_press``;
this package ``__init__`` re-exports its public names so callers use ``prompting_press.render`` etc.

The full facade (``__all__`` + package-metadata ``__version__``) is finalized in T022; this
re-export is the minimum needed for the US1 render surface to be importable.
"""

from .prompting_press import (  # the compiled extension submodule
    FieldError,
    GuardConfig,
    LoadError,
    PromptingPressError,
    PromptRenderError,
    PromptValidationError,
    Registry,
    RenderResult,
    UnknownPromptError,
    core_version,
    get_source,
    render,
)

# The generated Pydantic prompt-definition shape (codegen'd from the JSON Schema — C-07).
from .generated import PromptDefinition

__version__ = "0.0.0"  # placeholder until sourced from package metadata (T022)

__all__ = [
    "Registry",
    "RenderResult",
    "GuardConfig",
    "FieldError",
    "render",
    "get_source",
    "core_version",
    "PromptDefinition",
    "PromptingPressError",
    "PromptValidationError",
    "PromptRenderError",
    "UnknownPromptError",
    "LoadError",
]
