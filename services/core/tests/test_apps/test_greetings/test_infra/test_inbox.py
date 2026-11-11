"""Tests of the inbox that receives greeting events from NATS."""

import asyncio

import nats
from django.conf import settings

from server.apps.greetings.infra.inbox import SUBJECT, NatsInbox


async def _delivered(payload: bytes) -> bytes:
    """Publishes the payload and returns what the inbox received."""
    async with await nats.connect(
        settings.NATS_URL,
        connect_timeout=settings.NATS_TIMEOUT,
    ) as client:
        payloads = await NatsInbox(client, settings).payloads()
        await client.publish(SUBJECT, payload)
        return await asyncio.wait_for(anext(payloads), settings.NATS_TIMEOUT)


def test_nats_inbox_delivers_published_events() -> None:
    assert asyncio.run(_delivered(b'Ann')) == b'Ann'
