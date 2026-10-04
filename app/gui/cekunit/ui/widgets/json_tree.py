"""Tree widget untuk JSON."""

from __future__ import annotations

import json
from tkinter import ttk
from typing import Any


class JsonTree(ttk.Frame):
    def __init__(self, parent) -> None:
        super().__init__(parent)
        self._tree = ttk.Treeview(self, show="tree")
        self._tree.pack(fill="both", expand=True, side="left")
        scroll = ttk.Scrollbar(self, orient="vertical", command=self._tree.yview)
        scroll.pack(fill="y", side="right")
        self._tree.configure(yscrollcommand=scroll.set)

    def set_data(self, data: Any) -> None:
        for item in self._tree.get_children():
            self._tree.delete(item)
        self._walk(data, "")

    def clear(self) -> None:
        self.set_data(None)

    def _walk(self, node: Any, parent: str) -> None:
        if isinstance(node, dict):
            for key, value in node.items():
                node_id = self._tree.insert(parent, "end", text=str(key), open=True)
                self._walk(value, node_id)
        elif isinstance(node, list):
            for index, value in enumerate(node):
                node_id = self._tree.insert(parent, "end", text=f"[{index}]", open=True)
                self._walk(value, node_id)
        else:
            self._tree.insert(parent, "end", text=_scalar(node))


def _scalar(value: Any) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        return value
    try:
        return json.dumps(value)
    except (TypeError, ValueError):
        return str(value)
