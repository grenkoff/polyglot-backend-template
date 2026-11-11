"""Management command that consumes greeting events from NATS."""

from typing import final, override

from django.core.management.base import BaseCommand


@final
class Command(BaseCommand):
    """Listens to greeting events and records them in the journal."""

    help = 'Listens to hello.v1.greeted events from NATS and records them.'

    @override
    def handle(self, *args: object, **kwargs: object) -> None:
        """Records greeting events until the process receives SIGTERM."""
