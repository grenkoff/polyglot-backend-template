"""Fake hello services for the greetings tests."""

import threading
import time
from collections.abc import Generator, Iterator
from concurrent import futures
from contextlib import contextmanager
from typing import Final, final, override

import grpc
import pytest
from django.conf import LazySettings

from server.contracts.hello.v1 import hello_pb2, hello_pb2_grpc

_SLOW_ANSWER: Final = 2
_SHORT_TIMEOUT: Final = 0.2
_UNICODE_SPACES: Final = ''.join(
    map(
        chr,
        (
            *range(0x9, 0xE),
            0x20,
            0x85,
            0xA0,
            0x1680,
            *range(0x2000, 0x200B),
            0x2028,
            0x2029,
            0x202F,
            0x205F,
            0x3000,
        ),
    ),
)


@final
class _FakeHello(hello_pb2_grpc.HelloServiceServicer):
    """Fake hello service that greets like the real one."""

    @override
    def Greet(
        self,
        request: hello_pb2.GreetRequest,
        context: grpc.ServicerContext,
    ) -> hello_pb2.GreetResponse:
        """Greets the name in the request or aborts when it is blank."""
        name = request.name.strip(_UNICODE_SPACES)
        if not name:
            context.abort(
                grpc.StatusCode.INVALID_ARGUMENT,
                'name must not be blank',
            )
        text = f'Hello, {name}!'
        if request.polite:
            text = f'{text} Nice to meet you.'
        return hello_pb2.GreetResponse(text=text)


@final
class _SlowHello(hello_pb2_grpc.HelloServiceServicer):
    """Fake hello service that answers only when released."""

    def __init__(self, released: threading.Event) -> None:
        """Waits for the given event before answering."""
        self._released = released

    @override
    def Greet(
        self,
        request: hello_pb2.GreetRequest,
        context: grpc.ServicerContext,
    ) -> hello_pb2.GreetResponse:
        """Answers after the release event or a timeout."""
        self._released.wait(timeout=_SLOW_ANSWER)
        return hello_pb2.GreetResponse(text='too late')


@contextmanager
def _serving(
    servicer: hello_pb2_grpc.HelloServiceServicer,
) -> Generator[str]:
    """Serves a fake hello service on a free local port."""
    server = grpc.server(futures.ThreadPoolExecutor(max_workers=2))
    hello_pb2_grpc.add_HelloServiceServicer_to_server(servicer, server)
    port = server.add_insecure_port('127.0.0.1:0')
    server.start()
    try:
        yield f'127.0.0.1:{port}'
    finally:
        server.stop(grace=None)


@pytest.fixture
def hello_server(settings: LazySettings) -> Iterator[str]:
    """Points core at a fake hello service that greets."""
    with _serving(_FakeHello()) as address:
        settings.HELLO_GRPC_ADDR = address
        yield address


@pytest.fixture
def hello_slow(settings: LazySettings) -> Iterator[str]:
    """Points core at a fake hello service that answers too late."""
    released = threading.Event()
    with _serving(_SlowHello(released)) as address:
        settings.HELLO_GRPC_ADDR = address
        settings.HELLO_GRPC_TIMEOUT = _SHORT_TIMEOUT
        yield address
        released.set()
        time.sleep(_SHORT_TIMEOUT)


@pytest.fixture
def hello_down(settings: LazySettings) -> str:
    """Points core at an address where no hello service listens."""
    address = '127.0.0.1:1'
    settings.HELLO_GRPC_ADDR = address
    return address
