"""Badge status safety."""

from __future__ import annotations

from tkinter import ttk

from ...core.config import write_allowed
from .. import theme


class SafetyBadge(ttk.Frame):
    def __init__(self, parent) -> None:
        super().__init__(parent)
        self._label = ttk.Label(self, text="")
        self._label.pack()
        self.refresh()

    def refresh(self) -> None:
        if write_allowed():
            self._label.configure(
                text="  WRITE ENABLED  ",
                background=theme.WARN,
                foreground="#000000",
            )
        else:
            self._label.configure(
                text="  READ-ONLY  ",
                background=theme.OK,
                foreground="#000000",
            )
