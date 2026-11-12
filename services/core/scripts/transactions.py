"""Reports network calls made inside database transactions."""

import ast
from collections.abc import Iterator, Sequence
from pathlib import Path
from typing import Protocol, final

import attrs

from scripts.calls import CallRoot, NetworkImport

_PROBLEM = 'network call inside a transaction'


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
class Atomic:
    """Expression such as `transaction.atomic` or `atomic()`."""

    _expression: ast.expr

    def opens(self) -> bool:
        """Returns true for `atomic`, `transaction.atomic` and their calls."""
        match self._expression:
            case ast.Call(func=func):
                return Atomic(func).opens()
            case (
                ast.Attribute(value=ast.Name(id='transaction'), attr='atomic')
                | ast.Name(id='atomic')
            ):
                return True
            case _:
                return False


@final
@attrs.define(slots=True, frozen=True)
class AtomicBlock:
    """Statements that run inside a database transaction."""

    _node: ast.AST

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
class TransactionReport:
    """Network calls inside the transactions of one Python module."""

    _path: Path
    _text: str

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
                    for call in ast.walk(statement)
                    if isinstance(call, ast.Call) and CallRoot(call).name() in network
                )
