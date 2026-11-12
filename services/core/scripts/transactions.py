"""Reports network calls made inside database transactions."""

from collections.abc import Iterator
from pathlib import Path
from typing import Protocol, final

import attrs


class Report(Protocol):
    """Lists problems of some Python code."""

    def problems(self) -> Iterator[str]:
        """Yields one line per problem."""


@final
@attrs.define(slots=True, frozen=True)
class TransactionReport:
    """Network calls inside the transactions of one Python module."""

    _path: Path
    _text: str

    def problems(self) -> Iterator[str]:
        """Yields one line per network call inside a transaction."""
        yield from ()
