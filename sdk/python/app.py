"""Wednes Python SDK function handler."""

try:
    from wit_world.imports.types import Header, Request, Response
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

class WitWorld:
    def handle(self, req: Request) -> Response:
        return Response(
            status=200,
            headers=[Header(name="content-type", value="text/plain")],
            body=b"Hello from Wednes Python SDK!\n",
        )
