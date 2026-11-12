"""Checks that every Python object starts with a one-line docstring."""

import ast
import sys
from collections.abc import Iterator
from pathlib import Path
from typing import Protocol, final, override

import attrs

type _Documented = ast.Module | ast.ClassDef | ast.FunctionDef | ast.AsyncFunctionDef

_DEFINITIONS = (ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)
_TEST_PREFIX = 'test_'
_EXCLUDED = (
    '.venv/**',
    'mutants/**',
    'server/contracts/**',
    'server/apps/*/migrations/*',
)


class Report(Protocol):
    """Lists docstring problems of some Python code."""

    def problems(self) -> Iterator[str]:
        """Yields one line per problem."""


@final
@attrs.define(slots=True, frozen=True)
class DocstringReport(Report):
    """Docstring problems of one module, class or function."""

    _place: str
    _node: _Documented

    @override
    def problems(self) -> Iterator[str]:
        """Yields one line per problem."""
        docstring = ast.get_docstring(self._node)
        if not docstring:
            yield f'{self._place}: missing docstring'
        elif '\n' in docstring:
            yield f'{self._place}: docstring has more than one line'


@final
@attrs.define(slots=True, frozen=True)
class ModuleReport(Report):
    """Docstring problems of one Python module."""

    _path: Path
    _text: str

    @override
    def problems(self) -> Iterator[str]:
        """Yields one line per problem."""
        module = ast.parse(self._text)
        if module.body:
            yield from DocstringReport(f'{self._path}:1', module).problems()
        for node in ast.walk(module):
            if isinstance(node, _DEFINITIONS) and not node.name.startswith(
                _TEST_PREFIX,
            ):
                yield from DocstringReport(
                    f'{self._path}:{node.lineno}',
                    node,
                ).problems()


@final
@attrs.define(slots=True, frozen=True)
class TreeReport(Report):
    """Docstring problems of every Python module under a directory."""

    _root: Path
    _excluded: tuple[str, ...]

    @override
    def problems(self) -> Iterator[str]:
        """Yields one line per problem."""
        for path in sorted(self._root.rglob('*.py')):
            relative = path.relative_to(self._root)
            if not any(map(relative.full_match, self._excluded)):
                yield from ModuleReport(relative, path.read_text()).problems()


if __name__ == '__main__':
    problems = '\n'.join(TreeReport(Path(), _EXCLUDED).problems())
    sys.exit(problems or None)
