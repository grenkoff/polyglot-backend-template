from typing import final

import attrs
import structlog

from server.apps.greetings.logic.value_objects import Greeted
from server.contracts.hello.v1 import hello_pb2


@final
@attrs.define(slots=True, frozen=True)
class LogJournal:
    def record(self, payload: bytes) -> Greeted:
        message = hello_pb2.Greeted.FromString(payload)
        greeted = Greeted(name=message.name, text=message.text)
        structlog.get_logger(__name__).info(
            'greeting received',
            name=greeted.name,
            text=greeted.text,
        )
        return greeted
