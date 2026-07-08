"""Provenance attributes tests for the Python binding — spec 018 (T008, T012, T013).

T008 (US1): `result.provenance_attributes()` is a 4-entry dict[str, str] with the
            fixed library-owned `prompting_press.prompt.*` keys; values equal the
            result fields.
T012 (US2): exactly 4 keys; map excludes text/guard/metadata/output_model (per binding).
T013 (US3): the four provenance fields remain publicly readable for custom key maps.

These tests require the Python wheel to be installed (`maturin develop` or `pip install -e .`).
"""

from __future__ import annotations

import re

from prompting_press import Prompt
from pydantic import BaseModel

# ── Key constants (spec 018 FR-003, library-owned — NOT OTel gen_ai.* keys) ─────────────

KEY_NAME = "prompting_press.prompt.name"
KEY_VARIANT = "prompting_press.prompt.variant"
KEY_TEMPLATE_HASH = "prompting_press.prompt.template_hash"
KEY_RENDER_HASH = "prompting_press.prompt.render_hash"

EXPECTED_KEYS = frozenset({KEY_NAME, KEY_VARIANT, KEY_TEMPLATE_HASH, KEY_RENDER_HASH})

# A lowercase 64-char hex string — the SHA-256 provenance hash shape.
HEX64 = re.compile(r"\A[0-9a-f]{64}\Z")

# ── Vars model ───────────────────────────────────────────────────────────────────────────


class Greeting(BaseModel):
    """Minimal Pydantic model for the greeting prompt."""

    name: str
    count: int


# ── Prompt fixture ────────────────────────────────────────────────────────────────────────

GREET_YAML = """
name: greet
role: user
body: "Hi {{ name }}, you have {{ count }} messages"
variables:
  name:  { type: string,  trusted: true }
  count: { type: integer, trusted: true }
"""


def render_greet() -> object:
    """Render the greeting prompt and return the RenderResult."""
    p = Prompt.from_yaml(GREET_YAML)
    return p.render(Greeting, data={"name": "Ada", "count": 3})


# ── T008 (US1) ───────────────────────────────────────────────────────────────────────────


def test_provenance_attributes_returns_dict() -> None:
    """provenance_attributes() returns a dict."""
    result = render_greet()
    attrs = result.provenance_attributes()
    assert isinstance(attrs, dict), f"expected dict, got {type(attrs)}"


def test_provenance_attributes_has_exactly_four_entries() -> None:
    """Map contains exactly 4 entries (FR-002, SC-004)."""
    attrs = render_greet().provenance_attributes()
    assert len(attrs) == 4, f"expected 4 entries, got {len(attrs)}: {list(attrs.keys())}"


def test_provenance_attributes_contains_all_expected_keys() -> None:
    """Map contains all four library-owned keys (FR-003)."""
    attrs = render_greet().provenance_attributes()
    missing = EXPECTED_KEYS - set(attrs.keys())
    assert not missing, f"missing keys: {missing}"


def test_provenance_attributes_values_are_strings() -> None:
    """All values are str (dict[str, str] contract)."""
    attrs = render_greet().provenance_attributes()
    for k, v in attrs.items():
        assert isinstance(v, str), f"value for {k!r} must be str, got {type(v)}"


def test_provenance_attributes_values_equal_result_fields() -> None:
    """Values equal the corresponding result fields (FR-001)."""
    result = render_greet()
    attrs = result.provenance_attributes()

    assert attrs[KEY_NAME] == result.name, "KEY_NAME must equal result.name"
    assert attrs[KEY_VARIANT] == result.variant, "KEY_VARIANT must equal result.variant"
    assert attrs[KEY_TEMPLATE_HASH] == result.template_hash, (
        "KEY_TEMPLATE_HASH must equal result.template_hash"
    )
    assert attrs[KEY_RENDER_HASH] == result.render_hash, (
        "KEY_RENDER_HASH must equal result.render_hash"
    )


def test_provenance_attributes_default_variant_is_default(  # INV-3
) -> None:
    """When no variant is selected, KEY_VARIANT == 'default'."""
    attrs = render_greet().provenance_attributes()
    assert attrs[KEY_VARIANT] == "default"


def test_provenance_attributes_hashes_are_64_hex() -> None:
    """template_hash and render_hash are lowercase 64-char hex."""
    attrs = render_greet().provenance_attributes()
    assert HEX64.match(attrs[KEY_TEMPLATE_HASH]), (
        f"KEY_TEMPLATE_HASH is not 64-hex: {attrs[KEY_TEMPLATE_HASH]!r}"
    )
    assert HEX64.match(attrs[KEY_RENDER_HASH]), (
        f"KEY_RENDER_HASH is not 64-hex: {attrs[KEY_RENDER_HASH]!r}"
    )


def test_provenance_attributes_is_deterministic() -> None:
    """Two identical renders produce identical attribute maps (SC-003)."""
    p = Prompt.from_yaml(GREET_YAML)
    g = Greeting(name="Ada", count=3)
    attrs1 = p.render(g).provenance_attributes()
    attrs2 = p.render(g).provenance_attributes()
    assert attrs1 == attrs2, "identical renders must produce identical attribute maps"


# ── T012 (US2) — exclusions and purity ──────────────────────────────────────────────────


def test_provenance_attributes_excludes_text_guard_metadata_output_model() -> None:
    """Map must NOT include text, guard, output_model, or metadata (FR-007, SC-004)."""
    attrs = render_greet().provenance_attributes()

    assert "text" not in attrs, "rendered body text must be excluded"
    assert "guard" not in attrs, "guard text must be excluded"
    assert "output_model" not in attrs, "output_model must be excluded"
    assert "metadata" not in attrs, "metadata must be excluded"
    # Confirm still exactly 4 (not just the four exclusions).
    assert len(attrs) == 4


def test_provenance_attributes_is_pure_does_not_mutate_result() -> None:
    """Calling provenance_attributes() twice produces equal maps; result is unchanged."""
    result = render_greet()
    a = result.provenance_attributes()
    b = result.provenance_attributes()
    assert a == b, "calling provenance_attributes() twice must yield equal maps"
    # Result attributes are unchanged after calling the helper.
    assert result.name == "greet"
    assert result.variant == "default"


# ── T013 (US3) — custom key map ──────────────────────────────────────────────────────────


def test_public_fields_allow_custom_keyed_attribute_map() -> None:
    """Fields remain publicly readable; consumer can build a custom-keyed map (FR-008)."""
    result = render_greet()

    # Consumer-chosen keys — nothing to do with prompting_press.prompt.*.
    custom = {
        "my.prompt.name": result.name,
        "my.prompt.variant": result.variant,
        "my.prompt.template_hash": result.template_hash,
        "my.prompt.render_hash": result.render_hash,
    }

    assert len(custom) == 4
    assert custom["my.prompt.name"] == "greet"
    assert custom["my.prompt.variant"] == "default"
    assert HEX64.match(custom["my.prompt.template_hash"])
    assert HEX64.match(custom["my.prompt.render_hash"])
