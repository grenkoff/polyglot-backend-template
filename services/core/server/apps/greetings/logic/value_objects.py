"""Value objects of the greetings domain."""

from typing import Annotated, final

import msgspec

from server.apps.greetings.logic.constants import NAME_MAX_LENGTH, NAME_PATTERN


@final
class GreetingRequest(msgspec.Struct, frozen=True):
    """Request for a greeting: a name and whether to be polite."""

    name: Annotated[
        str,
        msgspec.Meta(
            min_length=1,
            max_length=NAME_MAX_LENGTH,
            pattern=NAME_PATTERN,
        ),
    ]
    polite: bool = False


@final
class GreetingText(msgspec.Struct, frozen=True):
    """Text of a greeting."""

    text: str


@final
class Greeted(msgspec.Struct, frozen=True):
    """Fact that someone was greeted, received from the hello service."""

    name: str
    text: str
