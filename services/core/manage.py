#!/usr/bin/env python

"""Runs Django management commands of core."""

import os
import sys


def main() -> None:
    """Sets the default settings module and executes the given command."""
    os.environ.setdefault('DJANGO_SETTINGS_MODULE', 'server.settings')

    try:
        from django.core import management  # noqa: PLC0415
    except ImportError:
        raise ImportError(
            "Couldn't import Django. Are you sure it's installed and "
            + 'available on your PYTHONPATH environment variable? Did you '
            + 'forget to activate a virtual environment?',
        ) from None

    management.execute_from_command_line(sys.argv)


if __name__ == '__main__':
    main()
