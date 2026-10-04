"""Konfigurasi dan loading .env."""

from __future__ import annotations

import os
from pathlib import Path

ENV_CEKUNIT_BIN = "CEKUNIT_BIN"
ENV_BASE_URL = "LIBRCEKUNIT_BASE_URL"
ENV_EMAIL = "LIBRCEKUNIT_EMAIL"
ENV_PASSWORD = "LIBRCEKUNIT_PASSWORD"
ENV_COOKIE_FILE = "LIBRCEKUNIT_COOKIE_FILE"
ENV_TIMEOUT_SECS = "LIBRCEKUNIT_TIMEOUT_SECS"
ENV_LOG = "CEKUNIT_LOG"
ENV_ALLOW_WRITE = "CEKUNIT_ALLOW_WRITE"
ENV_GUI_LOG = "CEKUNIT_GUI_LOG"
ENV_GUI_PREFS = "CEKUNIT_GUI_PREFS"

DEFAULT_BIN = "cekunit"
DEFAULT_GUI_LOG = "cekunit_gui.log"
DEFAULT_PREFS = "prefs.json"


def load_env_file(path: Path) -> None:
    if not path.is_file():
        return
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return

    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, _, value = line.partition("=")
        key = key.strip()
        value = value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in ("'", '"'):
            value = value[1:-1]
        if key and key not in os.environ:
            os.environ[key] = value


def get_bin() -> str:
    return os.environ.get(ENV_CEKUNIT_BIN, DEFAULT_BIN)


def get_gui_log_path() -> Path:
    return Path(os.environ.get(ENV_GUI_LOG, DEFAULT_GUI_LOG))


def get_prefs_path() -> Path:
    return Path(os.environ.get(ENV_GUI_PREFS, DEFAULT_PREFS))


def write_allowed() -> bool:
    v = os.environ.get(ENV_ALLOW_WRITE, "")
    return v == "1" or v.lower() == "true"


def env_snapshot() -> dict[str, str]:
    keys = [
        ENV_BASE_URL,
        ENV_EMAIL,
        ENV_COOKIE_FILE,
        ENV_TIMEOUT_SECS,
        ENV_LOG,
        ENV_ALLOW_WRITE,
    ]
    return {k: os.environ.get(k, "") for k in keys}
