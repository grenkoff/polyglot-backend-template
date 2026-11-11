"""Interfaces and errors of the greetings domain."""

from collections.abc import AsyncIterator
from typing import Protocol, final

from server.apps.greetings.logic.value_objects import (
    Greeted,
    GreetingRequest,
    GreetingText,
)


@final
class BlankNameError(ValueError):
    """Raised when the name to greet has no visible characters."""

    __slots__ = ()


@final
class GreetingsUnavailableError(RuntimeError):
    """Raised when greetings cannot be built right now."""

    __slots__ = ()


class Greetings(Protocol):
    """Source of greetings."""

    def text(self, request: GreetingRequest) -> GreetingText:
        """Returns the greeting for the request."""


class Journal(Protocol):
    """Destination of received greeting events."""

    def record(self, payload: bytes) -> Greeted:
        """Records one encoded greeting event and returns it decoded."""


class Inbox(Protocol):
    """Source of encoded greeting events."""

    async def payloads(self) -> AsyncIterator[bytes]:
        """Starts receiving and returns the payloads as they arrive."""
