from __future__ import annotations

from tkinter import ttk

from ...core.models import RunSpec
from .. import theme
from .base import BaseTab


class InfoTab(BaseTab):
    title = "Info"
    service_key = "auth"

    def __init__(self, parent, ctx) -> None:
        super().__init__(parent, ctx)
        self._build_body()

    def _build_body(self) -> None:
        row = ttk.Frame(self.body)
        row.pack(fill="x")
        ttk.Button(row, text="Probe server", command=self._on_probe).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(row, text="Clear", command=self.clear).pack(
            side="left", padx=theme.PAD_S
        )

    def _on_probe(self) -> None:
        self.submit(RunSpec(args=["info"]))
