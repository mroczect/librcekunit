"""Entry point: python -m cekunit."""

from __future__ import annotations

from pathlib import Path

from .core.config import load_env_file
from .ui.app import App


def main() -> int:
    env_path = Path.cwd() / ".env"
    if not env_path.is_file():
        env_path = Path(__file__).resolve().parent.parent / ".env"
    load_env_file(env_path)

    app = App()
    app.run()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
