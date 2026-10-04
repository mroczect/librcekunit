"""File picker (entry + browse button)."""

from __future__ import annotations

from tkinter import StringVar, filedialog, ttk

from .. import theme


class FilePicker(ttk.Frame):
    def __init__(
        self,
        parent,
        label: str,
        *,
        filetypes: list[tuple[str, str]] | None = None,
        mode: str = "open",
        width: int = 45,
    ) -> None:
        super().__init__(parent)
        self.var = StringVar()
        self._filetypes = filetypes or [("All files", "*.*")]
        self._mode = mode

        ttk.Label(self, text=label).grid(row=0, column=0, sticky="w", pady=theme.PAD_S)
        ttk.Entry(self, textvariable=self.var, width=width).grid(
            row=0, column=1, sticky="we", padx=theme.PAD_M
        )
        ttk.Button(self, text="Browse", command=self._on_browse).grid(
            row=0, column=2, padx=theme.PAD_S
        )
        self.columnconfigure(1, weight=1)

    def _on_browse(self) -> None:
        if self._mode == "save":
            path = filedialog.asksaveasfilename(filetypes=self._filetypes)
        else:
            path = filedialog.askopenfilename(filetypes=self._filetypes)
        if path:
            self.var.set(path)

    def get(self) -> str:
        return self.var.get().strip()

    def set(self, value: str) -> None:
        self.var.set(value)
