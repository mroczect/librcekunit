"""Keyboard shortcuts global."""

from __future__ import annotations


def bind(root, app) -> None:
    root.bind("<Control-r>", lambda e: app.on_refresh())
    root.bind("<Control-l>", lambda e: app.on_clear_log())
    root.bind("<Control-q>", lambda e: app.on_quit())
    root.bind("<F5>", lambda e: app.on_refresh())
    root.bind("<Control-Shift-C>", lambda e: app.on_copy_json())
    root.bind("<Control-Shift-R>", lambda e: app.on_reload_env())
