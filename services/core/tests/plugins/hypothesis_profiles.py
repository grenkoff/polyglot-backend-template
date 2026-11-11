"""Hypothesis profiles used by the test runs."""

from hypothesis import HealthCheck, settings

settings.register_profile(
    'mutmut',
    suppress_health_check=[HealthCheck.differing_executors],
)
