"""Content-Security-Policy headers served with django-csp."""

from typing import TypedDict, final

from csp.constants import NONE, SELF


@final
class _ContentSecurityPolicy(TypedDict):
    """Shape of the django-csp settings."""

    EXCLUDE_URL_PREFIXES: list[str]  # noqa: WPS115
    DIRECTIVES: dict[str, list[str]]  # noqa: WPS115


# These values might and will be redefined in `development.py` env:
CONTENT_SECURITY_POLICY: _ContentSecurityPolicy = {
    'EXCLUDE_URL_PREFIXES': [
        '/docs/v1/stoplight/',
        '/docs/v1/swagger/',
        '/docs/v1/scalar/',
        '/docs/v1/redoc/',
    ],
    'DIRECTIVES': {
        'default-src': [NONE],
        'script-src': [SELF],
        'style-src': [SELF],
        'img-src': [SELF],
        'font-src': [SELF],
        'connect-src': [],
    },
}
