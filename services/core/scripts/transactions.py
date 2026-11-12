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


@final
@attrs.define(slots=True, frozen=True)
class AtomicBlock:
    """Statements that run inside a `transaction.atomic()` block."""

    _node: ast.AST

    def statements(self) -> Sequence[ast.stmt]:
        """Returns the body of an atomic `with`, or nothing for other nodes."""
        match self._node:
            case ast.With(
                items=[
                    ast.withitem(
                        context_expr=ast.Call(
                            func=ast.Attribute(
                                value=ast.Name(id='transaction'),
                                attr='atomic',
                            ),
                        ),
                    ),
                    *_,
                ],
                body=body,
            ):
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
