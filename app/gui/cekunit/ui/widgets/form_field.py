"""Form field dengan validasi opsional."""

from __future__ import annotations

from collections.abc import Callable
from tkinter import StringVar, ttk

from .. import theme


class FormField(ttk.Frame):
    def __init__(
        self,
        parent,
        label: str,
        *,
        hint: str = "",
        password: bool = False,
        width: int = 40,
        default: str = "",
        on_change: Callable[[str], None] | None = None,
    ) -> None:
        super().__init__(parent)
        self.var = StringVar(value=default)
        if on_change is not None:
            self.var.trace_add("write", lambda *_: on_change(self.var.get()))

        ttk.Label(self, text=label).grid(row=0, column=0, sticky="w", pady=theme.PAD_S)
        entry = ttk.Entry(self, textvariable=self.var, width=width)
        if password:
            entry.configure(show="*")
        entry.grid(row=0, column=1, sticky="we", padx=theme.PAD_M, pady=theme.PAD_S)

        if hint:
            ttk.Label(self, text=hint, foreground=theme.MUTED).grid(
                row=1, column=1, sticky="w", padx=theme.PAD_M
            )
        self.columnconfigure(1, weight=1)

    def get(self) -> str:
        return self.var.get().strip()

    def set(self, value: str) -> None:
        self.var.set(value)
