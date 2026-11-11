"""Use case that creates a blog post."""

from __future__ import annotations

from typing import TYPE_CHECKING, final

import attrs

from server.apps.main.logic.value_objects import (
    BlogPostCreatePayload,
    BlogPostFullPayload,
)

if TYPE_CHECKING:
    from server.apps.main.infra import mappers, repository


@final
@attrs.define(slots=True, frozen=True)
class CreateBlogPost:
    """Creates ``BlogPost`` instances."""

    _repository: repository.BlogPostRepo
    _mapper: mappers.BlogPostMapper

    def __call__(
        self,
        parsed_body: BlogPostCreatePayload,
    ) -> BlogPostFullPayload:
        """Creates a blog post and returns it as a value object."""
        return self._mapper(self._repository.create(parsed_body))
