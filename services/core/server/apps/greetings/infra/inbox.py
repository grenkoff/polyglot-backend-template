"""Inbox that receives greeting events from NATS."""

from __future__ import annotations

from typing import TYPE_CHECKING, Final, final

import attrs

if TYPE_CHECKING:
    from collections.abc import AsyncIterator

    from django.conf import LazySettings
    from nats.aio.client import Client

SUBJECT: Final = 'hello.v1.greeted'


@final
@attrs.define(slots=True, frozen=True)
class NatsInbox:
    """Adapter: greeting events published to a NATS subject."""

    _client: Client
    _settings: LazySettings

    async def payloads(self) -> AsyncIterator[bytes]:
        """Subscribes to the greeting subject and returns the payloads."""
        subscription = await self._client.subscribe(SUBJECT)
        await self._client.flush(timeout=self._settings.NATS_TIMEOUT)
        return (message.data async for message in subscription.messages)
