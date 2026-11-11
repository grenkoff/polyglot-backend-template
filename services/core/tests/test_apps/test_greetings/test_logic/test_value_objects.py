"""Tests of greeting value objects."""

import msgspec

from server.apps.greetings.logic.value_objects import GreetingRequest


def test_greeting_name_schema() -> None:
    schema = msgspec.json.schema(GreetingRequest)

    name = schema['$defs']['GreetingRequest']['properties']['name']
    assert name.get('pattern') == r'\S'
