"""Tests of how the NATS listener stops when events are left unrecorded."""

import asyncio
import io
from typing import final, override

import nats
import pytest
from django.conf import LazySettings, settings
from django.core.management.base import OutputWrapper

from server.apps.greetings.infra.inbox import SUBJECT
from server.apps.greetings.infra.listener import NatsListener
from server.apps.greetings.logic.value_objects import Greeted

_SHORT_TIMEOUT = 0.2


@final
class _JournalBrokenError(Exception):
    """Raised by a journal that cannot record anything."""


@final
class _BrokenJournal:
    """Journal that fails on every event."""

    def record(self, payload: bytes) -> Greeted:
        """Refuses to record the event."""
        raise _JournalBrokenError(payload)


@final
class _Announced(io.StringIO):
    """Listener output that signals when the listener starts listening."""

    def __init__(self, listening: asyncio.Event) -> None:
        """Signals the given event on every write."""
        super().__init__()
        self._listening = listening

    @override
    def write(self, text: str) -> int:
        """Keeps the text and signals that the listener is listening."""
        self._listening.set()
        return super().write(text)


async def _stop_with_unrecorded_events() -> None:
    """Publishes two events to a broken journal and stops the listener."""
    listening, stopped = asyncio.Event(), asyncio.Event()
    output = OutputWrapper(_Announced(listening))
    running = asyncio.create_task(
        NatsListener(settings, _BrokenJournal(), output).run(stopped),
    )
    await listening.wait()
    async with await nats.connect(settings.NATS_URL) as client:
        await client.publish(SUBJECT, b'first')
        await client.publish(SUBJECT, b'second')
    stopped.set()
    await running


@pytest.mark.timeout(2)
def test_listener_stops_with_unrecorded_events(
    settings: LazySettings,
) -> None:
    settings.NATS_TIMEOUT = _SHORT_TIMEOUT

    with pytest.raises(_JournalBrokenError):
        asyncio.run(_stop_with_unrecorded_events())
