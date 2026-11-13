"""Tests of the timeouts of the database connection."""

import pytest
from django.db import connection


def _setting(name: str) -> str:
    """Reads one runtime parameter of the database connection."""
    with connection.cursor() as cursor:
        cursor.execute('SELECT current_setting(%s)', [name])
        return str(cursor.fetchone()[0])


@pytest.mark.django_db
def test_lock_timeout_is_five_seconds() -> None:
    """Ensures that a query gives up on a lock held longer than 5 seconds."""
    assert _setting('lock_timeout') == '5s'


@pytest.mark.django_db
def test_statement_timeout_is_fifteen_seconds() -> None:
    """Ensures that a statement running longer than 15 seconds is stopped."""
    assert _setting('statement_timeout') == '15s'
