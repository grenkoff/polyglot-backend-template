"""Greetings built by the hello service over gRPC."""

from __future__ import annotations

from typing import TYPE_CHECKING, final

import attrs
import grpc

from server.apps.greetings.logic.greetings import (
    BlankNameError,
    GreetingsUnavailableError,
)
from server.apps.greetings.logic.value_objects import GreetingText
from server.contracts.hello.v1 import hello_pb2, hello_pb2_grpc

if TYPE_CHECKING:
    from django.conf import LazySettings

    from server.apps.greetings.logic.value_objects import GreetingRequest


@final
@attrs.define(slots=True, frozen=True)
class RemoteGreetings:
    """Adapter: asks the hello service for greetings over gRPC."""

    _settings: LazySettings

    def text(self, request: GreetingRequest) -> GreetingText:
        """Asks the hello service for a greeting, raising domain errors."""
        with grpc.insecure_channel(self._settings.HELLO_GRPC_ADDR) as channel:
            try:
                response = hello_pb2_grpc.HelloServiceStub(channel).Greet(
                    hello_pb2.GreetRequest(
                        name=request.name,
                        polite=request.polite,
                    ),
                    timeout=self._settings.HELLO_GRPC_TIMEOUT,
                )
            except grpc.RpcError as exc:
                if exc.code() == grpc.StatusCode.INVALID_ARGUMENT:
                    raise BlankNameError(exc.details()) from exc
                raise GreetingsUnavailableError(str(exc)) from exc
        return GreetingText(text=response.text)
