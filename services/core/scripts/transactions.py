"""Reports network calls made inside database transactions."""

import ast
import sys
from collections.abc import Iterator, Sequence
from pathlib import Path
from typing import Protocol, final, override

import attrs

from scripts.calls import CallRoot, EagerCalls, NetworkImport

_PROBLEM = 'network call inside a transaction'
_ATOMIC = frozenset(('atomic', 'transaction.atomic'))


class Report(Protocol):
    """Lists problems of some Python code."""

    def problems(self) -> Iterator[str]:
        """Yields one line per problem."""


class Block(Protocol):
    """Statements of some Python code that run in one context."""

    def statements(self) -> Sequence[ast.stmt]:
        """Returns the statements, or nothing outside the context."""


class Marker(Protocol):
    """Python expression that may open a database transaction."""

    def opens(self) -> bool:
        """Returns true when the expression opens a transaction."""


@final
@attrs.define(slots=True, frozen=True)
class Atomic(Marker):
    """Expression such as `transaction.atomic` or `atomic()`."""

    _expression: ast.expr

    @override
    def opens(self) -> bool:
        """Returns true for `atomic`, `transaction.atomic` and their calls."""
        if isinstance(self._expression, ast.Call):
            return Atomic(self._expression.func).opens()
        return ast.unparse(self._expression) in _ATOMIC


@final
@attrs.define(slots=True, frozen=True)
class AtomicBlock(Block):
    """Statements that run inside a database transaction."""

    _node: ast.AST

    @override
    def statements(self) -> Sequence[ast.stmt]:
        """Returns the body of an atomic `with` or function, or nothing."""
        match self._node:
            case ast.With(items=contexts, body=body) if any(
                Atomic(context.context_expr).opens() for context in contexts
            ):
                return body
            case (
                ast.FunctionDef(
                    decorator_list=decorators,
                    body=body,
                )
                | ast.AsyncFunctionDef(
                    decorator_list=decorators,
                    body=body,
                )
            ) if any(Atomic(decorator).opens() for decorator in decorators):
                return body
            case _:
                return ()


@final
@attrs.define(slots=True, frozen=True)
class TransactionReport(Report):
    """Network calls inside the transactions of one Python module."""

    _path: Path
    _text: str

    @override
    def problems(self) -> Iterator[str]:
        """Yields one line per network call inside a transaction."""
        module = ast.parse(self._text)
        network = frozenset(
            name for node in ast.walk(module) for name in NetworkImport(node).names()
        )
        for node in ast.walk(module):
            for statement in AtomicBlock(node).statements():
                yield from (
                    f'{self._path}:{call.lineno}: {_PROBLEM}'
                    for call in EagerCalls(statement).calls()
                    if CallRoot(call).reaches(network)
                )


@final
@attrs.define(slots=True, frozen=True)
class TransactionTree(Report):
    """Network calls inside transactions of every module under a directory."""

    _root: Path

    @override
    def problems(self) -> Iterator[str]:
        """Yields one line per network call inside a transaction."""
        for path in sorted(self._root.rglob('*.py')):
            yield from TransactionReport(
                path.relative_to(self._root),
                path.read_text(),
            ).problems()


if __name__ == '__main__':
    problems = '\n'.join(TransactionTree(Path('server')).problems())
    sys.exit(problems or None)
