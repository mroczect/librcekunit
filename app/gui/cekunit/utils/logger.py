"""Logger rotating sederhana untuk GUI."""

from __future__ import annotations

import time
from pathlib import Path


class GuiLogger:
    """Tulis ke file + stdout opsional."""

    def __init__(self, path: Path, max_bytes: int = 1_000_000) -> None:
        self._path = path
        self._max_bytes = max_bytes

    def log(self, level: str, message: str) -> None:
        line = f"{time.strftime('%Y-%m-%dT%H:%M:%S')} [{level}] {message}\n"
        try:
            self._rotate_if_needed()
            with self._path.open("a", encoding="utf-8") as fh:
                fh.write(line)
        except OSError:
            pass

    def info(self, message: str) -> None:
        self.log("info", message)

    def error(self, message: str) -> None:
        self.log("error", message)

    def _rotate_if_needed(self) -> None:
        if not self._path.is_file():
            return
        try:
            if self._path.stat().st_size > self._max_bytes:
                backup = self._path.with_suffix(self._path.suffix + ".1")
                if backup.exists():
                    backup.unlink()
                self._path.rename(backup)
        except OSError:
            pass
