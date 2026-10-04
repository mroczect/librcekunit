"""Service untuk input-user."""

from __future__ import annotations

from pathlib import Path

from ..core.models import RunSpec
from .base import Service


class InputUserService(Service):
    def list(
        self, pages: int, sort: str, direction: str, search: str, on_done=None
    ) -> bool:
        args = ["input-user", "list", "--pages", str(pages)]
        if sort:
            args += ["--sort", sort]
        if direction:
            args += ["--direction", direction]
        if search:
            args += ["--search", search]
        return self.run(RunSpec(args=args), on_done)

    def export_today(self, out: Path | None, force: bool, on_done=None) -> bool:
        args = ["input-user", "export", "today"]
        if out:
            args += ["--out", str(out)]
        if force:
            args.append("--force")
        return self.run(RunSpec(args=args), on_done)

    def export_range(
        self,
        start: str,
        end: str,
        out: Path | None,
        force: bool,
        on_done=None,
    ) -> bool:
        args = ["input-user", "export", "range", start, end]
        if out:
            args += ["--out", str(out)]
        if force:
            args.append("--force")
        return self.run(RunSpec(args=args), on_done)

    def report(
        self,
        pages: int,
        by: str,
        csv_out: Path | None,
        on_done=None,
    ) -> bool:
        args = ["input-user", "report", "--pages", str(pages)]
        if by:
            args += ["--by", by]
        if csv_out:
            args += ["--csv", str(csv_out)]
        return self.run(RunSpec(args=args), on_done)
