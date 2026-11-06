from http import HTTPStatus
from typing import final, override

from django.http import HttpResponse
from dmr import Body, Controller, modify
from dmr.endpoint import Endpoint
from dmr.errors import ErrorType
from dmr.metadata import ResponseSpec

from server.apps.greetings.logic.greetings import (
    BlankNameError,
    GreetingsUnavailableError,
)
from server.apps.greetings.logic.usecases import greet
from server.apps.greetings.logic.value_objects import (
    GreetingRequest,
    GreetingText,
)
from server.common.di import HasContainer
from server.common.serializers import StrictMsgspecSerializer


@final
class GreetingCreate(
    HasContainer,
    Controller[StrictMsgspecSerializer],
):
    @modify(
        status_code=HTTPStatus.OK,
        extra_responses=[
            ResponseSpec(
                Controller.error_model,
                status_code=HTTPStatus.UNPROCESSABLE_ENTITY,
            ),
            ResponseSpec(
                Controller.error_model,
                status_code=HTTPStatus.SERVICE_UNAVAILABLE,
            ),
        ],
    )
    def post(self, parsed_body: Body[GreetingRequest]) -> GreetingText:
        return self.resolve(greet.Greet)(parsed_body)

    @override
    def handle_error(
        self,
        endpoint: Endpoint,
        controller: Controller[StrictMsgspecSerializer],
        exc: Exception,
    ) -> HttpResponse:
        if isinstance(exc, BlankNameError):
            return self.to_error(
                self.format_error(
                    'Name must not be blank',
                    error_type=ErrorType.value_error,
                ),
                status_code=HTTPStatus.UNPROCESSABLE_ENTITY,
            )
        if isinstance(exc, GreetingsUnavailableError):
            return self.to_error(
                self.format_error(
                    'Greetings are unavailable, try again later',
                    error_type=ErrorType.internal_error,
                ),
                status_code=HTTPStatus.SERVICE_UNAVAILABLE,
            )
        return super().handle_error(  # pragma: no cover
            endpoint,
            controller,
            exc,
        )
