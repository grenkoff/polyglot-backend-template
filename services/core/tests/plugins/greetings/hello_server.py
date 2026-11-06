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


@final
class _FakeHello(hello_pb2_grpc.HelloServiceServicer):
    @override
    def Greet(
        self,
        request: hello_pb2.GreetRequest,
        context: grpc.ServicerContext,
    ) -> hello_pb2.GreetResponse:
        name = request.name.strip()
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
    def __init__(self, released: threading.Event) -> None:
        self._released = released

    @override
    def Greet(
        self,
        request: hello_pb2.GreetRequest,
        context: grpc.ServicerContext,
    ) -> hello_pb2.GreetResponse:
        self._released.wait(timeout=_SLOW_ANSWER)
        return hello_pb2.GreetResponse(text='too late')


@contextmanager
def _serving(
    servicer: hello_pb2_grpc.HelloServiceServicer,
) -> Generator[str]:
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
    with _serving(_FakeHello()) as address:
        settings.HELLO_GRPC_ADDR = address
        yield address


@pytest.fixture
def hello_slow(settings: LazySettings) -> Iterator[str]:
    released = threading.Event()
    with _serving(_SlowHello(released)) as address:
        settings.HELLO_GRPC_ADDR = address
        settings.HELLO_GRPC_TIMEOUT = _SHORT_TIMEOUT
        yield address
        released.set()
        time.sleep(_SHORT_TIMEOUT)


@pytest.fixture
def hello_down(settings: LazySettings) -> str:
    address = '127.0.0.1:1'
    settings.HELLO_GRPC_ADDR = address
    return address
