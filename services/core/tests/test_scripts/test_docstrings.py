from pathlib import Path

from scripts.docstrings import ModuleReport


def test_accepts_one_line_docstrings() -> None:
    source = '\n'.join((
        '"""Module."""',
        'class Thing:',
        '    """Thing."""',
        '    def act(self):',
        '        """Acts."""',
        'def run():',
        '    """Runs."""',
    ))

    assert not list(ModuleReport(Path('a.py'), source).problems())


def test_reports_missing_private_docstring() -> None:
    source = '\n'.join((
        '"""Module."""',
        'def _hidden():',
        '    return 1',
    ))

    problems = list(ModuleReport(Path('a.py'), source).problems())

    assert problems == ['a.py:2: missing docstring']


def test_reports_multiline_docstring() -> None:
    source = '\n'.join((
        '"""Module."""',
        'def run():',
        '    """Runs.',
        '',
        '    And explains why.',
        '    """',
    ))

    problems = list(ModuleReport(Path('a.py'), source).problems())

    assert problems == ['a.py:2: docstring has more than one line']


def test_reports_module_without_docstring() -> None:
    problems = list(ModuleReport(Path('a.py'), 'VALUE = 1').problems())

    assert problems == ['a.py:1: missing docstring']


def test_skips_empty_module() -> None:
    assert not list(ModuleReport(Path('__init__.py'), '').problems())
