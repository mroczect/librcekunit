"""Service untuk CSV lokal."""

from __future__ import annotations

from pathlib import Path

from ..core.models import RunSpec
from .base import Service


class CsvService(Service):
    def check(self, path: Path, on_done=None) -> bool:
        return self.run(RunSpec(args=["csv", "check", str(path)]), on_done)

    def dedup(
        self,
        path: Path,
        by: list[str],
        keep: str,
        out: Path | None,
        force: bool,
        on_done=None,
    ) -> bool:
        args = ["csv", "dedup", str(path), "--by", ",".join(by), "--keep", keep]
        if out:
            args += ["--out", str(out)]
        if force:
            args.append("--force")
        return self.run(RunSpec(args=args), on_done)

    def pivot(
        self,
        path: Path,
        rows: list[str],
        values: str,
        agg: str,
        out: Path | None,
        force: bool,
        on_done=None,
    ) -> bool:
        args = [
            "csv",
            "pivot",
            str(path),
            "--rows",
            ",".join(rows),
            "--values",
            values,
            "--agg",
            agg,
        ]
        if out:
            args += ["--out", str(out)]
        if force:
            args.append("--force")
        return self.run(RunSpec(args=args), on_done)
