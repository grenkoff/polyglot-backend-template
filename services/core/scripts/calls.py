"""Calls in Python code that reach the network."""

import ast
from collections.abc import Iterator
from typing import Protocol, final

import attrs

_NETWORK_PACKAGES = frozenset(('grpc', 'requests', 'httpx', 'urllib', 'nats'))
_GENERATED_GRPC = '_pb2_grpc'


class Names(Protocol):
    """Names that some Python code binds."""

    def names(self) -> Iterator[str]:
        """Yields the bound names."""


class Root(Protocol):
    """Name that some Python expression starts from."""

    def name(self) -> str:
        """Returns the name, or an empty string when there is none."""


@final
@attrs.define(slots=True, frozen=True)
class NetworkImport:
    """Names an import statement binds to network clients."""

    _node: ast.AST

    def names(self) -> Iterator[str]:
        """Yields the names bound to network packages or gRPC stubs."""
        match self._node:
            case ast.Import(names=aliases):
                bound = [
                    (alias.name, alias.asname or alias.name.partition('.')[0])
                    for alias in aliases
                ]
            case ast.ImportFrom(module=module, names=aliases):
                bound = [
                    (f'{module}.{alias.name}', alias.asname or alias.name)
                    for alias in aliases
                ]
            case _:
                bound = []
        yield from (
            name
            for source, name in bound
            if source.partition('.')[0] in _NETWORK_PACKAGES
            or _GENERATED_GRPC in source
        )


@final
@attrs.define(slots=True, frozen=True)
class CallRoot:
    """Name a call starts from, such as `grpc` in `grpc.insecure_channel()`."""

    _call: ast.Call

    def name(self) -> str:
        """Returns the first name of the called expression."""
        target = self._call.func
        while isinstance(target, ast.Attribute):
            target = target.value
        match target:
            case ast.Name(id=name):
                return name
            case _:
                return ''
