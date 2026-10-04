"""Entry point GUI `cekunit`.

Jalankan dengan:
    python main.py
atau:
    python -m cekunit
"""

from __future__ import annotations

import sys
from pathlib import Path


def _bootstrap_path() -> None:
    """Pastikan paket `cekunit` bisa di-import walau main.py dijalankan
    dari lokasi mana pun."""
    here = Path(__file__).resolve().parent
    if str(here) not in sys.path:
        sys.path.insert(0, str(here))


def _load_env() -> None:
    from cekunit.core.config import load_env_file

    candidates = [
        Path.cwd() / ".env",
        Path(__file__).resolve().parent / ".env",
    ]
    for path in candidates:
        if path.is_file():
            load_env_file(path)
            return


def main() -> int:
    _bootstrap_path()
    _load_env()

    from cekunit.ui.app import App

    app = App()
    app.run()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
