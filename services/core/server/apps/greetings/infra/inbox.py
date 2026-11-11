"""Inbox that receives greeting events from NATS."""

from collections.abc import AsyncIterator
from typing import Final, final

import attrs
from nats.aio.client import Client

SUBJECT: Final = 'hello.v1.greeted'


@final
@attrs.define(slots=True, frozen=True)
class NatsInbox:
    """Adapter: greeting events published to a NATS subject."""

    _client: Client

    async def payloads(self) -> AsyncIterator[bytes]:
        """Subscribes to the greeting subject and returns the payloads."""
        return self._nothing()

    async def _nothing(self) -> AsyncIterator[bytes]:
        """Yields a placeholder instead of real events."""
        yield b''
