import asyncio
from typing import Any, Final, final, override

import nats
from django.conf import settings
from django.core.management.base import BaseCommand

from server.apps.greetings.infra.journal import LogJournal

_SUBJECT: Final = 'hello.v1.greeted'


@final
class Command(BaseCommand):
    help = 'Listens to hello.v1.greeted events from NATS and records them.'

    @override
    def handle(self, *args: Any, **options: Any) -> None:  # noqa: WPS110
        asyncio.run(self._listen())  # pragma: no cover

    async def _listen(self) -> None:  # pragma: no cover
        journal = LogJournal()
        client = await nats.connect(settings.NATS_URL)
        subscription = await client.subscribe(_SUBJECT)
        self.stdout.write(f'listening to {_SUBJECT}')
        async for message in subscription.messages:
            journal.record(message.data)
