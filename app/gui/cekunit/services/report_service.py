"""Service untuk report."""

from __future__ import annotations

from pathlib import Path

from ..core.models import RunSpec
from .base import Service


class ReportService(Service):
    def run_report(
        self,
        date_args: list[str],
        dedup_by: list[str],
        pivot_rows: list[str],
        pivot_values: str,
        pivot_agg: str,
        out: Path | None,
        force: bool,
        on_done=None,
    ) -> bool:
        args = [
            "report",
            *date_args,
            "--dedup-by",
            ",".join(dedup_by),
            "--pivot-rows",
            ",".join(pivot_rows),
            "--pivot-values",
            pivot_values,
            "--pivot-agg",
            pivot_agg,
        ]
        if out:
            args += ["--out", str(out)]
        if force:
            args.append("--force")
        return self.run(RunSpec(args=args), on_done)
