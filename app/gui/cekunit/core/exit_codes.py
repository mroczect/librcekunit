"""Mapping exit code CLI ke label manusiawi."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class CodeInfo:
    code: int
    name: str
    description: str
    retryable: bool


CODES: dict[int, CodeInfo] = {
    0: CodeInfo(0, "ok", "Sukses", False),
    1: CodeInfo(1, "internal", "Error internal", False),
    2: CodeInfo(2, "usage", "Argumen salah", False),
    3: CodeInfo(3, "config", "Konfigurasi salah", False),
    4: CodeInfo(4, "auth", "Login/session gagal", True),
    5: CodeInfo(5, "network", "Masalah jaringan", True),
    6: CodeInfo(6, "api", "Server kembalikan error", False),
    7: CodeInfo(7, "io", "I/O file gagal", True),
    8: CodeInfo(8, "csv", "CSV tidak valid", False),
    9: CodeInfo(9, "write_refused", "Write ditolak safety gate", False),
}


def describe(code: int) -> CodeInfo:
    """Ambil info kode, fallback ke internal."""
    return CODES.get(code, CODES[1])
