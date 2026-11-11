"""Listener that records greeting events from NATS until it is stopped."""

from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING, final

import attrs
import nats

from server.apps.greetings.infra.inbox import SUBJECT, NatsInbox
from server.apps.greetings.logic.usecases.listen import Listen

if TYPE_CHECKING:
    from collections.abc import AsyncIterator

    from django.conf import LazySettings
    from django.core.management.base import OutputWrapper

    from server.apps.greetings.logic.greetings import Inbox, Journal


@final
@attrs.define(slots=True, frozen=True)
class AnnouncedInbox:
    """Decorator: announces on the output once another inbox listens."""

    _origin: Inbox
    _output: OutputWrapper

    async def payloads(self) -> AsyncIterator[bytes]:
        """Starts the other inbox and announces that it listens."""
        payloads = await self._origin.payloads()
        self._output.write(f'listening to {SUBJECT}')
        return payloads


@final
@attrs.define(slots=True, frozen=True)
class NatsListener:
    """Records greeting events from NATS until stopped, then drains."""

    _settings: LazySettings
    _journal: Journal
    _output: OutputWrapper

    async def run(self, stopped: asyncio.Event) -> None:
        """Records events until the stop event is set."""
        client = await nats.connect(
            self._settings.NATS_URL,
            connect_timeout=self._settings.NATS_TIMEOUT,
            drain_timeout=self._settings.NATS_TIMEOUT,
        )
        listening = asyncio.create_task(
            Listen(
                AnnouncedInbox(
                    NatsInbox(client, self._settings),
                    self._output,
                ),
                self._journal,
            )(),
        )
        await stopped.wait()
        await client.drain()
        await listening
