from typing import Annotated, final

import msgspec

from server.apps.greetings.logic.constants import NAME_MAX_LENGTH


@final
class GreetingRequest(msgspec.Struct, frozen=True):
    name: Annotated[
        str,
        msgspec.Meta(min_length=1, max_length=NAME_MAX_LENGTH),
    ]
    polite: bool = False


@final
class GreetingText(msgspec.Struct, frozen=True):
    text: str


@final
class Greeted(msgspec.Struct, frozen=True):
    name: str
    text: str
