"""Tests of the docstring report of one module."""

from pathlib import Path

from scripts.docstrings import ModuleReport

_MODULE = Path('a.py')


def _source(*lines: str) -> str:
    """Joins lines into the text of a Python module."""
    return '\n'.join(lines)


def test_accepts_one_line_docstrings() -> None:
    source = _source(
        "'''Module.'''",
        'class Thing:',
        "    '''Thing.'''",
        '    def act(self):',
        "        '''Acts.'''",
        'def run():',
        "    '''Runs.'''",
    )

    assert not list(ModuleReport(_MODULE, source).problems())


def test_reports_missing_private_docstring() -> None:
    source = _source(
        "'''Module.'''",
        'def _hidden():',
        '    return 1',
    )

    problems = list(ModuleReport(_MODULE, source).problems())

    assert problems == ['a.py:2: missing docstring']


def test_reports_multiline_docstring() -> None:
    source = _source(
        "'''Module.'''",
        'def run():',
        "    '''Runs.",
        '',
        '    And explains why.',
        "    '''",
    )

    problems = list(ModuleReport(_MODULE, source).problems())

    assert problems == ['a.py:2: docstring has more than one line']


def test_reports_module_without_docstring() -> None:
    problems = list(ModuleReport(_MODULE, 'VALUE = 1').problems())

    assert problems == ['a.py:1: missing docstring']


def test_skips_empty_module() -> None:
    assert not list(ModuleReport(Path('__init__.py'), '').problems())


def test_skips_test_functions() -> None:
    source = _source(
        "'''Tests of something.'''",
        'def test_something():',
        '    assert True',
    )

    assert not list(ModuleReport(Path('test_a.py'), source).problems())
