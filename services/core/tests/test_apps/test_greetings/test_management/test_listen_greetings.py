"""Tests of the command that consumes greeting events from NATS."""

import asyncio
import io
import logging
import os
import signal
import threading
from typing import Final, final, override

import nats
from django.conf import settings
from django.core.management import call_command

from server.apps.greetings.infra.inbox import SUBJECT
from server.contracts.hello.v1 import hello_pb2

_PATIENCE: Final = 2.0
_PAYLOAD: Final = hello_pb2.Greeted(
    name='Ann',
    text='Hello, Ann!',
).SerializeToString()


@final
class _Announced(io.StringIO):
    """Command output that signals when the command starts listening."""

    def __init__(self, listening: threading.Event) -> None:
        """Signals the given event on the first write."""
        super().__init__()
        self._listening = listening

    @override
    def write(self, text: str) -> int:
        """Keeps the text and signals that the command is listening."""
        self._listening.set()
        return super().write(text)


@final
class _Recorded(logging.Handler):
    """Log handler that signals when a greeting is recorded."""

    def __init__(self, recorded: threading.Event) -> None:
        """Signals the given event on every record."""
        super().__init__()
        self._recorded = recorded

    @override
    def emit(self, record: logging.LogRecord) -> None:
        """Signals that a greeting was recorded."""
        self._recorded.set()


async def _publish() -> None:
    """Publishes one greeting event to NATS."""
    async with await nats.connect(
        settings.NATS_URL,
        connect_timeout=settings.NATS_TIMEOUT,
    ) as client:
        await client.publish(SUBJECT, _PAYLOAD)


def _drive(listening: threading.Event, recorded: threading.Event) -> None:
    """Publishes once the command listens and stops it once it records."""
    if listening.wait(_PATIENCE):
        asyncio.run(_publish())
        recorded.wait(_PATIENCE)
        os.kill(os.getpid(), signal.SIGTERM)


def test_listen_greetings_records_until_sigterm() -> None:
    listening, recorded = threading.Event(), threading.Event()
    server_logger = logging.getLogger('server')
    handler = _Recorded(recorded)
    server_logger.addHandler(handler)
    driver = threading.Thread(target=_drive, args=(listening, recorded))
    driver.start()
    try:
        call_command('listen_greetings', stdout=_Announced(listening))
    finally:
        driver.join()
        server_logger.removeHandler(handler)

    assert recorded.is_set()
