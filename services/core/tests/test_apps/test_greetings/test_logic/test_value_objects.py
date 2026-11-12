"""Tests of greeting value objects."""

import msgspec

from server.apps.greetings.logic.value_objects import GreetingRequest


def test_greeting_name_schema_spells_out_spaces() -> None:
    schema = msgspec.json.schema(GreetingRequest)

    name = schema['$defs']['GreetingRequest']['properties']['name']
    pattern = name['pattern']
    assert pattern.startswith('[^')
    assert r'\s' not in pattern.lower()
