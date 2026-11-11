"""Use case that reads a blog post."""

from __future__ import annotations

from typing import TYPE_CHECKING, final

import attrs

from server.apps.main.logic.value_objects import BlogPostFullPayload

if TYPE_CHECKING:
    from server.apps.main.infra import mappers, repository


@final
@attrs.define(slots=True, frozen=True)
class GetBlogPost:
    """Get ``BlogPost`` models by primary key."""

    _repository: repository.BlogPostRepo
    _mapper: mappers.BlogPostMapper

    def __call__(self, blog_post_id: int) -> BlogPostFullPayload:
        """Returns the blog post with the given id as a value object."""
        return self._mapper(self._repository.get_by_id(blog_post_id))
