"""Path helpers."""

from __future__ import annotations

import os
from pathlib import Path


def home_downloads() -> Path:
    return Path(os.environ.get("HOME", str(Path.home()))) / "Downloads"


def ensure_dir(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path
