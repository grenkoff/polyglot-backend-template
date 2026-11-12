"""Tests of the report of network calls inside database transactions."""

from pathlib import Path

from scripts.transactions import TransactionReport, TransactionTree

_MODULE = Path('a.py')
_PROBLEM = 'network call inside a transaction'
_TRANSACTION = 'from django.db import transaction'
_CONNECT = '    nats.connect()'


def _source(*lines: str) -> str:
    """Joins lines into the text of a Python module."""
    return '\n'.join(lines)


def test_reports_grpc_calls_inside_atomic() -> None:
    source = _source(
        'import grpc',
        _TRANSACTION,
        'from server.contracts.hello.v1 import hello_pb2_grpc',
        'with transaction.atomic():',
        "    channel = grpc.insecure_channel('hello:50051')",
        '    hello_pb2_grpc.HelloServiceStub(channel)',
    )

    problems = list(TransactionReport(_MODULE, source).problems())

    assert problems == [f'a.py:5: {_PROBLEM}', f'a.py:6: {_PROBLEM}']


def test_reports_http_and_nats_in_atomic() -> None:
    source = _source(
        'from urllib.request import urlopen',
        'import urllib.request',
        'import httpx',
        'import requests',
        'from django.db.transaction import atomic',
        'from nats.aio.client import Client',
        'with atomic():',
        "    requests.get('http://hello')",
        "    httpx.post('http://hello')",
        "    urlopen('http://hello')",
        "    urllib.request.urlopen('http://hello')",
        '    Client()',
    )

    problems = list(TransactionReport(_MODULE, source).problems())

    assert problems == [f'a.py:{line}: {_PROBLEM}' for line in range(8, 13)]


def test_reports_calls_in_atomic_functions() -> None:
    source = _source(
        'import nats',
        _TRANSACTION,
        '@transaction.atomic',
        'def save():',
        _CONNECT,
        '@transaction.atomic()',
        'async def store():',
        '    await nats.connect()',
    )

    problems = list(TransactionReport(_MODULE, source).problems())

    assert problems == [f'a.py:5: {_PROBLEM}', f'a.py:8: {_PROBLEM}']


def test_accepts_calls_outside_and_on_commit() -> None:
    source = _source(
        'import nats',
        _TRANSACTION,
        'from django.db.transaction import on_commit',
        'from functools import cache',
        'nats.connect()',
        'with transaction.atomic():',
        '    transaction.on_commit(lambda: nats.connect())',
        '    on_commit(lambda: nats.connect())',
        "    len('local work')",
        "    ''.join(())",
        'with nats.connect() as client:',
        _CONNECT,
        '@cache',
        'def ready():',
        _CONNECT,
        'nats.connect()',
    )

    problems = list(TransactionReport(_MODULE, source).problems())

    assert not problems


def test_reports_problems_under_a_directory(tmp_path: Path) -> None:
    package = tmp_path / 'apps'
    package.mkdir()
    (package / 'store.py').write_text(
        _source(
            'import nats',
            _TRANSACTION,
            'with transaction.atomic():',
            _CONNECT,
        ),
    )

    problems = list(TransactionTree(tmp_path).problems())

    assert problems == [f'apps/store.py:4: {_PROBLEM}']


def test_accepts_the_code_of_core() -> None:
    problems = list(TransactionTree(Path('server')).problems())

    assert not problems
