"""Calls in Python code that reach the network."""

import ast
from collections.abc import Iterator
from typing import Protocol, final

import attrs

_NETWORK_PACKAGES = frozenset(('grpc', 'requests', 'httpx', 'urllib', 'nats'))
_GENERATED_GRPC = '_pb2_grpc'
_ON_COMMIT = 'on_commit'


class Names(Protocol):
    """Names that some Python code binds."""

    def names(self) -> Iterator[str]:
        """Yields the bound names."""


class Target(Protocol):
    """Python call that may reach one of some names."""

    def reaches(self, names: frozenset[str]) -> bool:
        """Returns true when the call starts from one of the names."""


class Calls(Protocol):
    """Calls that some Python code makes."""

    def calls(self) -> Iterator[ast.Call]:
        """Yields the calls in the order they appear."""


@final
@attrs.define(slots=True, frozen=True)
class EagerCalls:
    """Calls that code makes at once, leaving out code given to `on_commit`."""

    _node: ast.AST

    def calls(self) -> Iterator[ast.Call]:
        """Yields the calls of the node and its children, outside `on_commit`."""
        if isinstance(self._node, ast.Call):
            if ast.unparse(self._node.func).rpartition('.')[2] == _ON_COMMIT:
                return
            yield self._node
        for child in ast.iter_child_nodes(self._node):
            yield from EagerCalls(child).calls()


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
    """Call seen by the name it starts from, as `grpc` in `grpc.connect()`."""

    _call: ast.Call

    def reaches(self, names: frozenset[str]) -> bool:
        """Returns true when the called expression starts from one of the names."""
        target = self._call.func
        while isinstance(target, ast.Attribute):
            target = target.value
        return isinstance(target, ast.Name) and target.id in names
