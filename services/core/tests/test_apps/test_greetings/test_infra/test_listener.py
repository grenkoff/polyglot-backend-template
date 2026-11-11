"""Tests of the listener that records greeting events from NATS."""

import asyncio
import io
import socket
from collections.abc import Iterator

import pytest
from django.conf import LazySettings, settings
from django.core.management.base import OutputWrapper

from server.apps.greetings.infra.journal import LogJournal
from server.apps.greetings.infra.listener import NatsListener

_SHORT_TIMEOUT = 0.2


@pytest.fixture
def silent_nats(settings: LazySettings) -> Iterator[None]:
    """Points core at a server that accepts connections and never answers."""
    with socket.socket() as silent:
        silent.bind(('127.0.0.1', 0))
        silent.listen()
        settings.NATS_URL = f'nats://127.0.0.1:{silent.getsockname()[1]}'
        settings.NATS_TIMEOUT = _SHORT_TIMEOUT
        yield


@pytest.mark.timeout(1)
@pytest.mark.usefixtures('silent_nats')
def test_nats_listener_gives_up_on_silent_nats() -> None:
    listener = NatsListener(settings, LogJournal(), OutputWrapper(io.StringIO()))

    with pytest.raises(TimeoutError):
        asyncio.run(listener.run(asyncio.Event()))
