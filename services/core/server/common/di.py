"""Dependency injection for controllers."""

import importlib
from typing import Any, final

import punq


class HasContainer:
    """Mixin that resolves dependencies; must be the first base class."""

    __slots__ = ('_container',)

    def __init__(self, *args: Any, **kwargs: Any) -> None:
        """Create container with dependencies for this class."""
        super().__init__(*args, **kwargs)
        # Imported lazily so `server.common` does not statically depend on
        # the project-wide DI bootstrap. This also preserves Import Linter
        # app-independence checks.
        implemented = importlib.import_module('server.implemented')
        self._container = implemented.populate_dependencies(punq.Container())

    @final
    def resolve[Thing](self, thing: type[Thing]) -> Thing:
        """Resolve a dependency."""
        return self._container.resolve(thing)  # type: ignore[no-any-return]
