from __future__ import annotations

from tkinter import ttk

from ...core.models import RunSpec
from .. import theme
from ..widgets.form_field import FormField
from .base import BaseTab


class AuthTab(BaseTab):
    title = "Auth"
    service_key = "auth"

    def __init__(self, parent, ctx) -> None:
        super().__init__(parent, ctx)
        self._build_body()

    def _build_body(self) -> None:
        self.email = FormField(self.body, "Email")
        self.email.pack(fill="x")
        self.password = FormField(self.body, "Password", password=True)
        self.password.pack(fill="x")

        row = ttk.Frame(self.body)
        row.pack(fill="x", pady=theme.PAD_M)

        ttk.Button(row, text="Login", command=self._on_login).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(row, text="Logout", command=self._on_logout).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(row, text="Info", command=self._on_info).pack(
            side="left", padx=theme.PAD_S
        )

    def _on_login(self) -> None:
        email = self.email.get()
        password = self.password.get()
        if not email or not password:
            self.log.append("[validate] email dan password wajib diisi")
            return
        self.submit(
            RunSpec(
                args=["auth", "login"],
                extra_env={
                    "LIBRCEKUNIT_EMAIL": email,
                    "LIBRCEKUNIT_PASSWORD": password,
                },
            )
        )

    def _on_logout(self) -> None:
        self.submit(RunSpec(args=["auth", "logout"]))

    def _on_info(self) -> None:
        self.submit(RunSpec(args=["auth", "info"]))
