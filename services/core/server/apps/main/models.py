"""Database models of the main app."""

import textwrap
from typing import final, override

from django.db import models

from server.apps.main.logic.constants import POST_TITLE_MAX_LENGTH


@final
class BlogPost(models.Model):
    """Example blog post model with fixtures, factories and migration tests."""

    title = models.CharField(max_length=POST_TITLE_MAX_LENGTH)
    body = models.TextField()

    created_at = models.DateTimeField(auto_now_add=True)
    updated_at = models.DateTimeField(auto_now=True)

    class Meta:
        # You can use `gettext` to translate model names:
        """Names of the model in the admin panel."""

        verbose_name = 'BlogPost'
        verbose_name_plural = 'BlogPosts'

    @override
    def __str__(self) -> str:
        """All django models should have this method."""
        return textwrap.wrap(self.title, POST_TITLE_MAX_LENGTH // 4)[0]
