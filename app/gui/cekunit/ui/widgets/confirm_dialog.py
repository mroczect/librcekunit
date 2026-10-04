"""Dialog konfirmasi dengan frasa."""

from __future__ import annotations

from tkinter import StringVar, Toplevel, ttk

from .. import theme


class ConfirmDialog(Toplevel):
    """Dialog modal: user harus ketik frasa yang tepat."""

    def __init__(
        self,
        parent,
        title: str,
        description: str,
        phrase: str,
    ) -> None:
        super().__init__(parent)
        self.title(title)
        self.geometry("500x260")
        self.transient(parent)
        self.grab_set()

        self.result: bool = False
        self._phrase = phrase
        self._var = StringVar(value="")

        ttk.Label(
            self,
            text="KONFIRMASI DIPERLUKAN",
            foreground=theme.ERROR,
            font=("", 12, "bold"),
        ).pack(anchor="w", padx=theme.PAD_L, pady=theme.PAD_M)

        ttk.Label(self, text=description, wraplength=440, justify="left").pack(
            anchor="w", padx=theme.PAD_L, pady=theme.PAD_S
        )

        ttk.Label(
            self,
            text=f'Ketik frasa persis: "{phrase}"',
            foreground=theme.WARN,
        ).pack(anchor="w", padx=theme.PAD_L, pady=theme.PAD_M)

        entry = ttk.Entry(self, textvariable=self._var)
        entry.pack(fill="x", padx=theme.PAD_L)
        entry.focus_set()

        row = ttk.Frame(self)
        row.pack(fill="x", padx=theme.PAD_L, pady=theme.PAD_L)
        ttk.Button(row, text="Batal", command=self._on_cancel).pack(
            side="right", padx=theme.PAD_S
        )
        ttk.Button(row, text="Konfirmasi", command=self._on_ok).pack(
            side="right", padx=theme.PAD_S
        )

        self.bind("<Return>", lambda e: self._on_ok())
        self.bind("<Escape>", lambda e: self._on_cancel())

    def _on_ok(self) -> None:
        if self._var.get().strip() == self._phrase:
            self.result = True
            self.destroy()

    def _on_cancel(self) -> None:
        self.result = False
        self.destroy()


def confirm(parent, title: str, description: str, phrase: str) -> bool:
    """Buka dialog dan kembalikan True kalau dikonfirmasi."""
    dlg = ConfirmDialog(parent, title, description, phrase)
    parent.wait_window(dlg)
    return dlg.result