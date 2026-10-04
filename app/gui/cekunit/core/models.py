"""Model data."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any


@dataclass
class ErrorBody:
    kind: str
    code: int
    message: str
    retryable: bool

    @classmethod
    def from_json(cls, data: dict[str, Any]) -> ErrorBody:
        return cls(
            kind=str(data.get("kind", "unknown")),
            code=int(data.get("code", 1)),
            message=str(data.get("message", "")),
            retryable=bool(data.get("retryable", False)),
        )


@dataclass
class Result:
    ok: bool
    command: str
    data: dict[str, Any] | None = None
    error: ErrorBody | None = None
    exit_code: int = 0
    stderr: str = ""
    raw_stdout: str = ""
    duration_ms: int = 0

    @property
    def is_write_refused(self) -> bool:
        return self.error is not None and self.error.kind == "write_refused"

    @property
    def summary(self) -> str:
        if self.ok:
            return f"ok: {self.command} ({self.duration_ms}ms)"
        if self.error is not None:
            return f"{self.error.kind}: {self.error.message}"
        return f"failed: {self.command}"


@dataclass
class RunSpec:
    args: list[str] = field(default_factory=list)
    extra_env: dict[str, str] = field(default_factory=dict)
    destructive: bool = False
    needs_confirmation: bool = False
    confirm_phrase: str = ""
    description: str = ""

    def command_line(self) -> str:
        return "cekunit " + " ".join(self.args)
