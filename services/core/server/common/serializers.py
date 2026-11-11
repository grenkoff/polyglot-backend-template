"""Serializers shared by the HTTP APIs."""

from collections.abc import Mapping
from typing import Any, final, override

from dmr.plugins.msgspec import MsgspecSerializer


@final
class StrictMsgspecSerializer(MsgspecSerializer):
    """Msgspec serializer that parses request bodies strictly by default."""

    @override
    @classmethod
    def from_python(
        cls,
        unstructured: Any,
        model: Any,
        *,
        strict: bool | None,
        extra_namespace: Mapping[str, Any] | None = None,
    ) -> Any:
        """Converts data to the model, strictly unless told otherwise."""
        return super().from_python(
            unstructured,
            model,
            strict=strict is not False,
            extra_namespace=extra_namespace,
        )
