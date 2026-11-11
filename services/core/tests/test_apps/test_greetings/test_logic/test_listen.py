"""Tests of the use case that records every received greeting event."""

import asyncio
from collections.abc import AsyncIterator
from typing import final

import attrs

from server.apps.greetings.logic.usecases.listen import Listen
from server.apps.greetings.logic.value_objects import Greeted


@final
@attrs.define(slots=True, frozen=True)
class _MemoryInbox:
    """Inbox that delivers a fixed list of payloads."""

    _delivered: tuple[bytes, ...]

    async def payloads(self) -> AsyncIterator[bytes]:
        """Returns the payloads one by one."""
        return self._each()

    async def _each(self) -> AsyncIterator[bytes]:
        """Yields every payload in order."""
        for payload in self._delivered:
            yield payload


@final
@attrs.define(slots=True, frozen=True)
class _MemoryJournal:
    """Journal that keeps recorded payloads in a list."""

    recorded: list[bytes] = attrs.field(factory=list)

    def record(self, payload: bytes) -> Greeted:
        """Keeps the payload and returns it as a greeting."""
        self.recorded.append(payload)
        return Greeted(name=payload.decode(), text='')


def test_listen_records_events_in_order() -> None:
    journal = _MemoryJournal()

    asyncio.run(Listen(_MemoryInbox((b'Ann', b'Bob')), journal)())

    assert journal.recorded == [b'Ann', b'Bob']
