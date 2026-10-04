"""Validasi input sebelum dikirim ke CLI."""

from __future__ import annotations

import re
from datetime import datetime
from pathlib import Path

_DATE_RE = re.compile(r"^\d{4}-\d{2}-\d{2}$")


class ValidationError(Exception):
    """Input tidak valid."""


def date_string(value: str) -> str:
    """Pastikan format YYYY-MM-DD dan tanggal valid."""
    v = value.strip()
    if not _DATE_RE.match(v):
        raise ValidationError(f"tanggal harus YYYY-MM-DD, dapat {value!r}")
    try:
        datetime.strptime(v, "%Y-%m-%d").date()
    except ValueError as exc:
        raise ValidationError(f"tanggal tidak valid: {exc}") from exc
    return v


def date_range(start: str, end: str) -> tuple[str, str]:
    s = date_string(start)
    e = date_string(end)
    if s > e:
        raise ValidationError(f"start {s} > end {e}")
    return s, e


def column_pair(value: str) -> tuple[str, str]:
    """Parse `column=value`."""
    v = value.strip()
    if "=" not in v:
        raise ValidationError("format harus column=value")
    col, _, val = v.partition("=")
    col = col.strip()
    val = val.strip()
    if not col:
        raise ValidationError("nama kolom tidak boleh kosong")
    if not val:
        raise ValidationError("nilai tidak boleh kosong")
    if not re.match(r"^[A-Za-z_][A-Za-z0-9_]*$", col):
        raise ValidationError(f"nama kolom tidak valid: {col!r}")
    return col, val


def existing_file(value: str) -> Path:
    p = Path(value.strip())
    if not p.is_file():
        raise ValidationError(f"file tidak ada: {p}")
    return p


def non_empty(value: str, label: str = "value") -> str:
    v = value.strip()
    if not v:
        raise ValidationError(f"{label} tidak boleh kosong")
    return v


def positive_int(value: str, label: str = "value") -> int:
    v = value.strip()
    try:
        n = int(v)
    except ValueError as exc:
        raise ValidationError(f"{label} harus angka, dapat {value!r}") from exc
    if n <= 0:
        raise ValidationError(f"{label} harus > 0")
    return n


def columns_list(value: str) -> list[str]:
    parts = [p.strip() for p in value.split(",") if p.strip()]
    if not parts:
        raise ValidationError("daftar kolom kosong")
    return parts
