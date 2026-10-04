"""Client wrapper untuk binary cekunit."""

from __future__ import annotations

import json
import os
import subprocess
import time

from .config import get_bin
from .exit_codes import describe
from .models import ErrorBody, Result, RunSpec


class CliNotFoundError(Exception):
    """Binary cekunit tidak ditemukan di PATH."""


class CliClient:
    """Wrapper sinkron. Async diurus AsyncRunner."""

    def __init__(self, timeout_secs: int = 600) -> None:
        self._timeout = timeout_secs

    def run(self, spec: RunSpec) -> Result:
        """Jalankan CLI, kembalikan Result."""
        env = dict(os.environ)
        env.update(spec.extra_env)

        started = time.monotonic()
        try:
            proc = subprocess.run(
                [get_bin(), *spec.args],
                capture_output=True,
                text=True,
                env=env,
                timeout=self._timeout,
                check=False,
            )
        except FileNotFoundError as exc:
            raise CliNotFoundError(f"binary tidak ditemukan: {get_bin()}") from exc
        except subprocess.TimeoutExpired:
            duration = int((time.monotonic() - started) * 1000)
            return Result(
                ok=False,
                command=self._guess_command(spec.args),
                error=ErrorBody(
                    kind="timeout",
                    code=124,
                    message=f"proses tidak selesai dalam {self._timeout}s",
                    retryable=True,
                ),
                exit_code=124,
                duration_ms=duration,
            )

        duration = int((time.monotonic() - started) * 1000)
        stdout = proc.stdout.strip()
        stderr = proc.stderr.strip()

        parsed = self._parse_envelope(stdout)
        if parsed is None:
            info = describe(proc.returncode)
            return Result(
                ok=False,
                command=self._guess_command(spec.args),
                error=ErrorBody(
                    kind="parse_error",
                    code=proc.returncode or 1,
                    message=f"stdout bukan JSON envelope ({info.name})",
                    retryable=False,
                ),
                exit_code=proc.returncode,
                stderr=stderr,
                raw_stdout=stdout,
                duration_ms=duration,
            )

        ok = bool(parsed.get("ok", False))
        command = str(parsed.get("command", self._guess_command(spec.args)))

        if ok:
            data = parsed.get("data")
            if not isinstance(data, dict):
                data = {"value": data}
            return Result(
                ok=True,
                command=command,
                data=data,
                exit_code=proc.returncode,
                stderr=stderr,
                raw_stdout=stdout,
                duration_ms=duration,
            )

        err_raw = parsed.get("error") or {}
        if not isinstance(err_raw, dict):
            err_raw = {"kind": "unknown", "code": 1, "message": str(err_raw)}
        return Result(
            ok=False,
            command=command,
            error=ErrorBody.from_json(err_raw),
            exit_code=proc.returncode,
            stderr=stderr,
            raw_stdout=stdout,
            duration_ms=duration,
        )

    @staticmethod
    def _parse_envelope(text: str) -> dict | None:
        if not text:
            return None
        for line in reversed(text.splitlines()):
            line = line.strip()
            if not line.startswith("{"):
                continue
            try:
                parsed = json.loads(line)
            except ValueError:
                continue
            if isinstance(parsed, dict) and "ok" in parsed:
                return parsed
        return None

    @staticmethod
    def _guess_command(args: list[str]) -> str:
        return args[0] if args else "unknown"
