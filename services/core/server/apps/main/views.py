"""Pages of the main app."""

from django.http import HttpRequest, HttpResponse
from django.shortcuts import render


def index(request: HttpRequest) -> HttpResponse:
    """Renders the index page."""
    return render(request, 'main/index.html')
