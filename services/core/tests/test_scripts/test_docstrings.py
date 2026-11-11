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
