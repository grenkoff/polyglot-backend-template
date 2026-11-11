"""Routes of the greetings HTTP API."""

from dmr.routing import Router, path

from server.apps.greetings.api import views

router = Router(
    'greetings/',
    [
        path('', views.GreetingCreate.as_view(), name='greeting_create'),
    ],
)
