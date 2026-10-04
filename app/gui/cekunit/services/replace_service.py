"""Service untuk replace (destructive)."""

from __future__ import annotations

from pathlib import Path

from ..core.models import RunSpec
from .base import Service


class ReplaceService(Service):
    def dry_run(self, path: Path, on_done=None) -> bool:
        return self.run(RunSpec(args=["replace", str(path)]), on_done)

    def execute(
        self,
        path: Path,
        no_backup: bool,
        on_done=None,
    ) -> bool:
        args = ["replace", str(path), "--execute"]
        if no_backup:
            args.append("--no-backup")
        spec = RunSpec(
            args=args,
            destructive=True,
            needs_confirmation=True,
            confirm_phrase="REPLACE",
            description=f"Backup + delete-all + upload {path.name}",
        )
        return self.run(spec, on_done)
