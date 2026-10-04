"""Service untuk auth."""

from __future__ import annotations

from ..core.models import RunSpec
from .base import Service


class AuthService(Service):
    def login(self, email: str, password: str, on_done=None) -> bool:
        spec = RunSpec(
            args=["auth", "login"],
            extra_env={
                "LIBRCEKUNIT_EMAIL": email,
                "LIBRCEKUNIT_PASSWORD": password,
            },
        )
        return self.run(spec, on_done)

    def logout(self, on_done=None) -> bool:
        return self.run(RunSpec(args=["auth", "logout"]), on_done)

    def info(self, on_done=None) -> bool:
        return self.run(RunSpec(args=["auth", "info"]), on_done)
