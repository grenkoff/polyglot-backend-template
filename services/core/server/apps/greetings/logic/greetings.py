from typing import Protocol, final

from server.apps.greetings.logic.value_objects import (
    Greeted,
    GreetingRequest,
    GreetingText,
)


@final
class BlankNameError(ValueError):
    __slots__ = ()


@final
class GreetingsUnavailableError(RuntimeError):
    __slots__ = ()


class Greetings(Protocol):
    def text(self, request: GreetingRequest) -> GreetingText: ...


class Journal(Protocol):
    def record(self, payload: bytes) -> Greeted: ...
