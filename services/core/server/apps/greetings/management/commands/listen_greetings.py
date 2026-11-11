"""Management command that consumes greeting events from NATS."""

import asyncio
import signal
from typing import final, override

from django.conf import settings
from django.core.management.base import BaseCommand

from server.apps.greetings.infra.journal import LogJournal
from server.apps.greetings.infra.listener import NatsListener


@final
class Command(BaseCommand):
    """Listens to greeting events and records them in the journal."""

    help = 'Listens to hello.v1.greeted events from NATS and records them.'

    @override
    def handle(self, *args: object, **kwargs: object) -> None:
        """Records greeting events until the process receives SIGTERM."""
        asyncio.run(self._listen())

    async def _listen(self) -> None:
        """Runs the NATS listener until SIGTERM sets the stop event."""
        stopped = asyncio.Event()
        asyncio.get_running_loop().add_signal_handler(
            signal.SIGTERM,
            stopped.set,
        )
        await NatsListener(settings, LogJournal(), self.stdout).run(stopped)
