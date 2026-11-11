"""Pytest configuration, fixtures and plugins of core."""

pytest_plugins = [
    # Should be the first custom one:
    'plugins.django_settings',
    'plugins.tracecov_map',
    'plugins.hypothesis_profiles',
    # TODO: add your own plugins here!
    'plugins.main.main_templates',
    'plugins.greetings.hello_server',
    'plugins.greetings.nats_events',
]
