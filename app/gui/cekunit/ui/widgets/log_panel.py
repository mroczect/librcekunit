"""Panel log."""

from __future__ import annotations

from tkinter import END, Text, ttk

from .. import theme


class LogPanel(ttk.Frame):
    def __init__(self, parent) -> None:
        super().__init__(parent)
        self._text = Text(
            self,
            height=8,
            wrap="word",
            bg=theme.BG,
            fg=theme.FG,
            insertbackground=theme.FG,
            font=(theme.MONO_FAMILY, theme.MONO_SIZE),
            borderwidth=0,
        )
        self._text.pack(fill="both", expand=True)

    def append(self, line: str) -> None:
        self._text.insert(END, line + "\n")
        self._text.see(END)

    def get_text(self) -> str:
        return self._text.get("1.0", END)

    def clear(self) -> None:
        self._text.delete("1.0", END)
