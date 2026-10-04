"""Status bar bawah."""

from __future__ import annotations

from tkinter import StringVar, ttk

from ...core.config import env_snapshot, write_allowed
from .. import theme


class StatusBar(ttk.Frame):
    def __init__(self, parent) -> None:
        super().__init__(parent)

        self.base = StringVar(value="-")
        self.write = StringVar(value="-")
        self.busy = StringVar(value="idle")

        ttk.Label(self, textvariable=self.base).pack(side="left", padx=theme.PAD_M)
        ttk.Separator(self, orient="vertical").pack(side="left", fill="y")
        ttk.Label(self, textvariable=self.write).pack(side="left", padx=theme.PAD_M)
        ttk.Separator(self, orient="vertical").pack(side="left", fill="y")
        ttk.Label(self, textvariable=self.busy).pack(side="left", padx=theme.PAD_M)

        self.refresh()

    def refresh(self) -> None:
        snap = env_snapshot()
        self.base.set(f"base_url={snap.get('LIBRCEKUNIT_BASE_URL', '') or '(unset)'}")
        self.write.set("write=on" if write_allowed() else "write=off")

    def set_busy(self, busy: bool) -> None:
        self.busy.set("busy" if busy else "idle")
