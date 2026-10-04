"""Kebijakan safety untuk operasi write."""

from __future__ import annotations

from dataclasses import dataclass

from .config import write_allowed


@dataclass(frozen=True)
class SafetyDecision:
    allowed: bool
    reason: str
    requires_confirmation: bool
    confirm_phrase: str = ""


def check_write(
    needs_confirmation: bool = True, phrase: str = "HAPUS"
) -> SafetyDecision:
    """Putuskan apakah write boleh dilakukan.

    Aturan:
    1. Kalau env CEKUNIT_ALLOW_WRITE tidak di-set → tolak.
    2. Kalau di-set → izinkan, tapi tetap minta konfirmasi kalau
       `needs_confirmation=True`.
    """
    if not write_allowed():
        return SafetyDecision(
            allowed=False,
            reason=(
                "CEKUNIT_ALLOW_WRITE tidak di-set ke 1. "
                "Operasi write ditolak oleh policy."
            ),
            requires_confirmation=False,
        )

    return SafetyDecision(
        allowed=True,
        reason="write diizinkan oleh environment",
        requires_confirmation=needs_confirmation,
        confirm_phrase=phrase,
    )
