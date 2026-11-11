"""Tests of the greeting HTTP endpoint."""

from http import HTTPStatus

import msgspec
import pytest
from django.urls import reverse
from dmr.test import DMRClient

from server.apps.greetings.logic.value_objects import GreetingRequest


@pytest.fixture
def url() -> str:
    """Returns the URL of the greeting endpoint."""
    return reverse('api:v1:greetings:greeting_create')


@pytest.mark.usefixtures('hello_server')
@pytest.mark.parametrize(
    ('polite', 'expected'),
    [
        (False, 'Hello, Ann!'),
        (True, 'Hello, Ann! Nice to meet you.'),
    ],
)
def test_greeting_create(
    dmr_client: DMRClient,
    url: str,
    *,
    polite: bool,
    expected: str,
) -> None:
    response = dmr_client.post(
        url,
        data=msgspec.to_builtins(GreetingRequest(name=' Ann ', polite=polite)),
    )

    assert response.status_code == HTTPStatus.OK
    assert response.json() == {'text': expected}


@pytest.mark.usefixtures('hello_down')
@pytest.mark.parametrize('blank', ['   ', '\t\n', '　'])
def test_greeting_blank_name(
    dmr_client: DMRClient,
    url: str,
    blank: str,
) -> None:
    response = dmr_client.post(
        url,
        data=msgspec.to_builtins(GreetingRequest(name=blank)),
    )

    assert response.status_code == HTTPStatus.BAD_REQUEST


@pytest.mark.usefixtures('hello_server')
@pytest.mark.parametrize(
    'payload',
    [
        {'name': 'Ann', 'polite': 0},
        {'name': 'Ann', 'polite': 'true'},
        {'name': 1},
    ],
)
def test_greeting_wrong_types(
    dmr_client: DMRClient,
    url: str,
    payload: dict[str, object],
) -> None:
    response = dmr_client.post(url, data=payload)

    assert response.status_code == HTTPStatus.BAD_REQUEST


@pytest.mark.usefixtures('hello_down')
def test_greeting_hello_is_down(dmr_client: DMRClient, url: str) -> None:
    response = dmr_client.post(
        url,
        data=msgspec.to_builtins(GreetingRequest(name='Ann')),
    )

    assert response.status_code == HTTPStatus.SERVICE_UNAVAILABLE
