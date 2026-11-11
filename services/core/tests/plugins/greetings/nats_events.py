"""Fixtures that publish greeting events to NATS and watch them recorded."""

import asyncio
import logging
import os
import signal
import threading
from collections.abc import Iterator
from typing import Final, final, override

import nats
import pytest
from django.conf import settings

from server.apps.greetings.infra.inbox import SUBJECT
from server.contracts.hello.v1 import hello_pb2

_PATIENCE: Final = 2.0
_PAYLOAD: Final = hello_pb2.Greeted(
    name='Ann',
    text='Hello, Ann!',
).SerializeToString()


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
    """Publishes once a listener listens and stops it once it records."""
    assert listening.wait(_PATIENCE), 'the command never started listening'
    asyncio.run(_publish())
    recorded.wait(_PATIENCE)
    os.kill(os.getpid(), signal.SIGTERM)


@pytest.fixture
def recorded() -> Iterator[threading.Event]:
    """Signals when the server logger records a greeting."""
    signal_of_record = threading.Event()
    recording = _Recorded(signal_of_record)
    server_logger = logging.getLogger('server')
    server_logger.addHandler(recording)
    yield signal_of_record
    server_logger.removeHandler(recording)


@pytest.fixture
def listening(recorded: threading.Event) -> Iterator[threading.Event]:
    """Publishes a greeting once set, then sends SIGTERM once it is recorded."""
    signal_of_listening = threading.Event()
    driver = threading.Thread(
        target=_drive,
        args=(signal_of_listening, recorded),
    )
    driver.start()
    yield signal_of_listening
    driver.join()
