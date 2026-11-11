"""Tests of the command that consumes greeting events from NATS."""

import io
import threading
from typing import final, override

from django.core.management import call_command


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


def test_listen_greetings_records_until_sigterm(
    listening: threading.Event,
    recorded: threading.Event,
) -> None:
    call_command('listen_greetings', stdout=_Announced(listening))

    assert recorded.is_set()
