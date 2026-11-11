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
        subscription = await self._client.subscribe(SUBJECT)
        await self._client.flush()
        return (message.data async for message in subscription.messages)
