"""Management command that consumes greeting events from NATS."""

import asyncio
import signal
from collections.abc import AsyncIterator
from typing import final, override

import attrs
import nats
from django.conf import settings
from django.core.management.base import BaseCommand, OutputWrapper

from server.apps.greetings.infra.inbox import SUBJECT, NatsInbox
from server.apps.greetings.infra.journal import LogJournal
from server.apps.greetings.logic.greetings import Inbox
from server.apps.greetings.logic.usecases.listen import Listen


@final
@attrs.define(slots=True, frozen=True)
class _AnnouncedInbox:
    """Decorator: announces on the output once another inbox listens."""

    _origin: Inbox
    _output: OutputWrapper

    async def payloads(self) -> AsyncIterator[bytes]:
        """Starts the other inbox and announces that it listens."""
        payloads = await self._origin.payloads()
        self._output.write(f'listening to {SUBJECT}')
        return payloads


@final
class Command(BaseCommand):
    """Listens to greeting events and records them in the journal."""

    help = 'Listens to hello.v1.greeted events from NATS and records them.'

    @override
    def handle(self, *args: object, **kwargs: object) -> None:
        """Records greeting events until the process receives SIGTERM."""
        asyncio.run(self._listen())

    async def _listen(self) -> None:
        """Records events until SIGTERM drains the NATS connection."""
        client = await nats.connect(
            settings.NATS_URL,
            connect_timeout=settings.NATS_TIMEOUT,
            drain_timeout=settings.NATS_TIMEOUT,
        )
        stopped = asyncio.Event()
        asyncio.get_running_loop().add_signal_handler(
            signal.SIGTERM,
            stopped.set,
        )
        listening = asyncio.create_task(
            Listen(
                _AnnouncedInbox(NatsInbox(client, settings), self.stdout),
                LogJournal(),
            )(),
        )
        await stopped.wait()
        await client.drain()
        await listening
