"""Service untuk dashboard."""

from __future__ import annotations

from ..core.models import RunSpec
from .base import Service


class DashboardService(Service):
    def list(
        self, pages: int, sort: str, direction: str, search: str, on_done=None
    ) -> bool:
        args = ["dashboard", "list", "--pages", str(pages)]
        if sort:
            args += ["--sort", sort]
        if direction:
            args += ["--direction", direction]
        if search:
            args += ["--search", search]
        return self.run(RunSpec(args=args), on_done)

    def unique(self, column: str, on_done=None) -> bool:
        return self.run(RunSpec(args=["dashboard", "unique", column]), on_done)

    def delete_category(
        self, column: str, value: str, execute: bool, on_done=None
    ) -> bool:
        pair = f"{column}={value}"
        args = ["dashboard", "delete-category", pair]
        if execute:
            args.append("--execute")
        spec = RunSpec(
            args=args,
            destructive=execute,
            needs_confirmation=execute,
            confirm_phrase="HAPUS",
            description=f"Hapus baris dengan {pair}",
        )
        return self.run(spec, on_done)

    def delete_all(self, execute: bool, on_done=None) -> bool:
        args = ["dashboard", "delete-all"]
        if execute:
            args.append("--execute")
        spec = RunSpec(
            args=args,
            destructive=execute,
            needs_confirmation=execute,
            confirm_phrase="HAPUS SEMUA",
            description="Hapus SEMUA baris di dashboard",
        )
        return self.run(spec, on_done)
