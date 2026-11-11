"""Tests of greetings from the hello service over gRPC."""

import pytest
from django.conf import settings

from server.apps.greetings.infra.remote import RemoteGreetings
from server.apps.greetings.logic.greetings import (
    BlankNameError,
    GreetingsUnavailableError,
)
from server.apps.greetings.logic.value_objects import (
    GreetingRequest,
    GreetingText,
)


@pytest.mark.usefixtures('hello_server')
def test_remote_greetings_text() -> None:
    greeting = RemoteGreetings(settings).text(GreetingRequest(name='Ann'))

    assert greeting == GreetingText(text='Hello, Ann!')


@pytest.mark.usefixtures('hello_server')
def test_remote_greetings_blank_name() -> None:
    with pytest.raises(BlankNameError, match=r'^name must not be blank$'):
        RemoteGreetings(settings).text(GreetingRequest(name=' '))


@pytest.mark.usefixtures('hello_down')
def test_remote_greetings_unavailable() -> None:
    with pytest.raises(GreetingsUnavailableError, match='UNAVAILABLE'):
        RemoteGreetings(settings).text(GreetingRequest(name='Ann'))


@pytest.mark.usefixtures('hello_slow')
def test_remote_greetings_timeout() -> None:
    with pytest.raises(GreetingsUnavailableError, match='DEADLINE_EXCEEDED'):
        RemoteGreetings(settings).text(GreetingRequest(name='Ann'))
