"""Toolbar atas."""

from __future__ import annotations

from tkinter import ttk

from .. import theme


class Toolbar(ttk.Frame):
    def __init__(
        self, parent, on_refresh, on_reload_env, on_clear_log, on_copy_json
    ) -> None:
        super().__init__(parent)
        ttk.Button(self, text="Refresh (Ctrl+R)", command=on_refresh).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(self, text="Reload env", command=on_reload_env).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(self, text="Clear log", command=on_clear_log).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(self, text="Copy JSON", command=on_copy_json).pack(
            side="left", padx=theme.PAD_S
        )
