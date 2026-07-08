"""Integration: prompt.check() → CheckReport findings.

Covers: clean prompt passes, untrusted_without_guard finding, guard presence satisfies
lint, check is pure (no mutation), construction errors are not lint findings.
"""

from __future__ import annotations

import pytest

from prompting_press import CheckReport, Prompt, PromptingPressError

KIND_UNTRUSTED = "untrusted_without_guard"


def _kinds(report: CheckReport) -> list[str]:
    return [f.kind for f in report.findings]


# ─── clean prompt passes ─────────────────────────────────────────────────────


def test_clean_prompt_passes_with_empty_report() -> None:
    p = Prompt(
        {
            "name": "greet",
            "role": "user",
            "body": "Hi {{ name }}",
            "variables": {"name": {"type": "string", "trusted": True}},
        }
    )
    report = p.check()
    assert isinstance(report, CheckReport)
    assert report.passed() is True
    assert report.is_empty() is True
    assert len(report) == 0
    assert not bool(report)


def test_no_variables_prompt_passes() -> None:
    p = Prompt({"name": "bare", "role": "user", "body": "Hello, world!"})
    assert p.check().passed() is True


# ─── untrusted_without_guard finding ─────────────────────────────────────────


def test_untrusted_variable_without_guard_flagged() -> None:
    p = Prompt(
        {
            "name": "search",
            "role": "user",
            "body": "Query: {{ q }}",
            "variables": {"q": {"type": "string", "trusted": False}},
        }
    )
    report = p.check()
    assert not report.passed()
    assert _kinds(report) == [KIND_UNTRUSTED]
    finding = report.findings[0]
    assert finding.kind == KIND_UNTRUSTED
    assert finding.prompt == "search"
    assert finding.variant is None  # prompt-level finding
    assert "q" in finding.detail


def test_guard_key_in_metadata_satisfies_lint() -> None:
    p = Prompt(
        {
            "name": "search",
            "role": "user",
            "body": "Query: {{ q }}",
            "variables": {"q": {"type": "string", "trusted": False}},
            "metadata": {"guard": "sanitized upstream"},
        }
    )
    assert p.check().passed()


# ─── CheckReport collection protocol ─────────────────────────────────────────


def test_check_report_collection_protocol() -> None:
    p_clean = Prompt({"name": "clean", "role": "user", "body": "hi"})
    p_untrusted = Prompt(
        {
            "name": "untrusted",
            "role": "user",
            "body": "{{ q }}",
            "variables": {"q": {"type": "string", "trusted": False}},
        }
    )
    clean = p_clean.check()
    assert clean.passed() is True
    assert len(clean) == 0
    assert not bool(clean)

    flagged = p_untrusted.check()
    assert flagged.passed() is False
    assert len(flagged) == 1
    assert bool(flagged)


# ─── findings are read-only ───────────────────────────────────────────────────


def test_finding_attributes_are_read_only() -> None:
    p = Prompt(
        {
            "name": "search",
            "role": "user",
            "body": "Query: {{ q }}",
            "variables": {"q": {"type": "string", "trusted": False}},
        }
    )
    finding = p.check().findings[0]
    with pytest.raises(AttributeError):
        finding.kind = "tampered"  # type: ignore[misc]


# ─── check is pure ────────────────────────────────────────────────────────────


def test_check_is_pure_and_repeatable() -> None:
    p = Prompt(
        {
            "name": "greet",
            "role": "user",
            "body": "Hi {{ name }}",
            "variables": {"name": {"type": "string", "trusted": True}},
        }
    )
    report_a = p.check()
    report_b = p.check()
    assert report_a.passed() == report_b.passed()
    assert [(f.kind, f.prompt) for f in report_a.findings] == [
        (f.kind, f.prompt) for f in report_b.findings
    ]


# ─── construction errors are not lint findings ───────────────────────────────


def test_undeclared_variable_is_construction_error_not_lint() -> None:
    with pytest.raises(PromptingPressError):
        Prompt(
            {
                "name": "ghosty",
                "role": "user",
                "body": "{{ ghost }}",
                "variables": {"name": {"type": "string", "trusted": True}},
            }
        )


def test_excluded_feature_is_construction_error() -> None:
    with pytest.raises(PromptingPressError):
        Prompt({"name": "ae", "role": "user", "body": '{% include "x" %}'})
