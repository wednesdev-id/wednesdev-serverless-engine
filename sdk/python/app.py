"""Basic example implementing the exported handle function from wednes:function WIT."""

try:
    from wit_world.imports.types import Header, Request, Response  # type: ignore[import-not-found]
except ImportError:
    try:
        from function.imports.types import Header, Request, Response  # type: ignore[import-not-found]
    except ImportError:
        from dataclasses import dataclass
        from typing import List

        @dataclass
        class Header:
            name: str
            value: str

        @dataclass
        class Request:
            method: str
            path: str
            headers: List[Header]
            body: bytes

        @dataclass
        class Response:
            status: int
            headers: List[Header]
            body: bytes


class Function:
    def handle(self, req: Request) -> Response:
        return Response(
            status=200,
            headers=[Header(name="content-type", value="text/plain")],
            body=b"Hello from Wednes Python SDK!\n",
        )


# Exported class alias for componentize-py target world
WitWorld = Function
