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
