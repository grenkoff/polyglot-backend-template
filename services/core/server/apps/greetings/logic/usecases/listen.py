"""Use case that records every received greeting event."""

from __future__ import annotations

from typing import TYPE_CHECKING, final

import attrs

if TYPE_CHECKING:
    from server.apps.greetings.logic.greetings import Inbox, Journal


@final
@attrs.define(slots=True, frozen=True)
class Listen:
    """Records every event an inbox delivers in a journal."""

    _inbox: Inbox
    _journal: Journal

    async def __call__(self) -> None:
        """Records events until the inbox has no more."""
        async for payload in await self._inbox.payloads():
            self._journal.record(payload)
