"""Use case that greets a person."""

from __future__ import annotations

from typing import TYPE_CHECKING, final

import attrs

if TYPE_CHECKING:
    from server.apps.greetings.logic.greetings import Greetings
    from server.apps.greetings.logic.value_objects import (
        GreetingRequest,
        GreetingText,
    )


@final
@attrs.define(slots=True, frozen=True)
class Greet:
    """Greets a person with the configured greetings."""

    _greetings: Greetings

    def __call__(self, request: GreetingRequest) -> GreetingText:
        """Returns the greeting for the request."""
        return self._greetings.text(request)
