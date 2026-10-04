"""Panel progress parsing stderr log CLI."""

from __future__ import annotations

import json
from tkinter import DoubleVar, StringVar, ttk

from .. import theme


class ProgressPanel(ttk.Frame):
    def __init__(self, parent) -> None:
        super().__init__(parent)
        self._value = DoubleVar(value=0.0)
        self._label = StringVar(value="idle")

        ttk.Label(self, textvariable=self._label).pack(anchor="w")
        self._bar = ttk.Progressbar(
            self, orient="horizontal", mode="determinate", variable=self._value
        )
        self._bar.pack(fill="x", pady=theme.PAD_S)

    def reset(self) -> None:
        self._value.set(0.0)
        self._label.set("idle")

    def feed_stderr_line(self, line: str) -> None:
        """Terima satu baris JSON dari stderr CLI.

        Contoh: {"level":"info","msg":"download_progress","mb":3,"bytes":3145728}
        """
        s = line.strip()
        if not s.startswith("{"):
            return
        try:
            parsed = json.loads(s)
        except ValueError:
            return
        if not isinstance(parsed, dict):
            return
        if parsed.get("msg") == "download_progress":
            mb = parsed.get("mb", 0)
            self._label.set(f"downloaded {mb} MiB")
            self._value.set(min(float(mb) * 10, 100))
