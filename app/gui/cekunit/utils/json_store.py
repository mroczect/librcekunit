"""Simpan/load state ke JSON file."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


class JsonStore:
    """Simple key-value store dengan backend JSON."""

    def __init__(self, path: Path) -> None:
        self._path = path
        self._cache: dict[str, Any] = {}
        self._load()

    def _load(self) -> None:
        if not self._path.is_file():
            return
        try:
            data = json.loads(self._path.read_text(encoding="utf-8"))
        except (OSError, ValueError):
            return
        if isinstance(data, dict):
            self._cache = data

    def get(self, key: str, default: Any = None) -> Any:
        return self._cache.get(key, default)

    def set(self, key: str, value: Any) -> None:
        self._cache[key] = value
        self._save()

    def _save(self) -> None:
        try:
            self._path.write_text(
                json.dumps(self._cache, indent=2),
                encoding="utf-8",
            )
        except OSError:
            pass
