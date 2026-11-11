"""Tests of the docstring report of a directory."""

from pathlib import Path

from scripts.docstrings import TreeReport


def test_skips_excluded_paths(tmp_path: Path) -> None:
    generated = tmp_path / 'contracts'
    generated.mkdir()
    (generated / 'generated.py').write_text('VALUE = 1')
    (tmp_path / 'written.py').write_text('VALUE = 1')

    problems = list(TreeReport(tmp_path, ('contracts/**',)).problems())

    assert problems == ['written.py:1: missing docstring']
