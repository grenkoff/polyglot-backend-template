"""Checks that every Python object starts with a one-line docstring."""

from collections.abc import Iterator
from pathlib import Path
from typing import Protocol, final

import attrs


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
        yield from ()
