"""Konstanta tema."""

BG = "#1e1e1e"
FG = "#e0e0e0"
ACCENT = "#4a9eff"
OK = "#4caf50"
WARN = "#ff9800"
ERROR = "#f44336"
MUTED = "#888888"
MONO_FAMILY = "monospace"
MONO_SIZE = 10
PAD_S = 4
PAD_M = 8
PAD_L = 16


def apply(root) -> None:
    from tkinter import ttk

    style = ttk.Style(root)
    available = style.theme_names()
    for candidate in ("clam", "alt", "default"):
        if candidate in available:
            style.theme_use(candidate)
            break
