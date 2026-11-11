"""Addresses and timeouts of the services core talks to."""

from server.settings.components import config

HELLO_GRPC_ADDR = config('HELLO_GRPC_ADDR', default='localhost:50051')
HELLO_GRPC_TIMEOUT = config('HELLO_GRPC_TIMEOUT', cast=float, default=5.0)

NATS_URL = config('NATS_URL', default='nats://localhost:4222')
