"""Helper clipboard."""

from __future__ import annotations

import json
from tkinter import Tk
from typing import Any


def copy_text(root: Tk, text: str) -> None:
    root.clipboard_clear()
    root.clipboard_append(text)


def copy_json(root: Tk, data: Any) -> None:
    try:
        text = json.dumps(data, indent=2)
    except (TypeError, ValueError):
        text = str(data)
    copy_text(root, text)
