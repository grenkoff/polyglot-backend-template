"""Checks that every Python object starts with a one-line docstring."""

import ast
from collections.abc import Iterator
from pathlib import Path
from typing import Protocol, final

import attrs

_DOCUMENTED = (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)


class Report(Protocol):
    """Lists docstring problems of some Python code."""

    def problems(self) -> Iterator[str]:
        """Yields one line per problem."""


@final
@attrs.define(slots=True, frozen=True)
class ModuleReport:
    """Docstring problems of one Python module."""

    _path: Path
    _text: str

    def problems(self) -> Iterator[str]:
        """Yields one line per problem."""
        for node in ast.walk(ast.parse(self._text)):
            if isinstance(node, _DOCUMENTED):
                docstring = ast.get_docstring(node, clean=False)
                if not docstring:
                    yield f'{self._path}:{node.lineno}: missing docstring'
                elif '\n' in docstring:
                    yield (
                        f'{self._path}:{node.lineno}: '
                        'docstring has more than one line'
                    )
