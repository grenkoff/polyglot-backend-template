import logging
from collections.abc import Iterator

import pytest
from structlog.testing import capture_logs

from server.apps.greetings.infra import journal
from server.apps.greetings.logic.value_objects import Greeted
from server.contracts.hello.v1 import hello_pb2

_PAYLOAD = hello_pb2.Greeted(name='Ann', text='Hello, Ann!').SerializeToString()


@pytest.fixture
def server_records(
    caplog: pytest.LogCaptureFixture,
) -> Iterator[pytest.LogCaptureFixture]:
    server_logger = logging.getLogger('server')
    server_logger.addHandler(caplog.handler)
    yield caplog
    server_logger.removeHandler(caplog.handler)


def test_journal_decodes_events() -> None:
    greeted = journal.LogJournal().record(_PAYLOAD)

    assert greeted == Greeted(name='Ann', text='Hello, Ann!')


def test_journal_logs_events() -> None:
    with capture_logs() as logs:
        journal.LogJournal().record(_PAYLOAD)

    assert logs == [
        {
            'event': 'greeting received',
            'log_level': 'info',
            'name': 'Ann',
            'text': 'Hello, Ann!',
        },
    ]


def test_journal_logs_through_server_logger(
    server_records: pytest.LogCaptureFixture,
) -> None:
    journal.LogJournal().record(_PAYLOAD)

    records = server_records.records
    assert [record.name for record in records] == [journal.__name__]
