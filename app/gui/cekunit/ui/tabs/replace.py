from __future__ import annotations

from tkinter import ttk

from ...core.models import RunSpec
from .. import theme
from ..widgets.file_picker import FilePicker
from ..widgets.form_field import FormField
from .base import BaseTab


class ReplaceTab(BaseTab):
    title = "Replace"
    service_key = "replace"

    def __init__(self, parent, ctx) -> None:
        super().__init__(parent, ctx)
        self._build_body()

    def _build_body(self) -> None:
        warn = ttk.Label(
            self.body,
            text=(
                "DESTRUKTIF: backup + delete-all + upload. "
                "Butuh CEKUNIT_ALLOW_WRITE=1 dan konfirmasi frasa."
            ),
            foreground=theme.ERROR,
            wraplength=600,
            justify="left",
        )
        warn.pack(anchor="w", pady=theme.PAD_M)

        self.picker = FilePicker(
            self.body,
            "CSV file",
            filetypes=[("CSV", "*.csv"), ("All", "*.*")],
        )
        self.picker.pack(fill="x")

        self.no_backup = FormField(self.body, "Skip backup (0/1)", default="0", width=4)
        self.no_backup.pack(fill="x")

        row = ttk.Frame(self.body)
        row.pack(fill="x", pady=theme.PAD_M)
        ttk.Button(row, text="Dry-run", command=self._on_dry).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(row, text="Execute", command=self._on_exec).pack(
            side="left", padx=theme.PAD_S
        )

    def _on_dry(self) -> None:
        path = self.picker.get()
        if not path:
            self.log.append("[validate] file kosong")
            return
        self.submit(RunSpec(args=["replace", path]))

    def _on_exec(self) -> None:
        path = self.picker.get()
        if not path:
            self.log.append("[validate] file kosong")
            return
        args = ["replace", path, "--execute"]
        if self.no_backup.get() in ("1", "true", "yes"):
            args.append("--no-backup")
        self.submit(
            RunSpec(
                args=args,
                destructive=True,
                needs_confirmation=True,
                confirm_phrase="REPLACE",
                description=(f"Backup (atau skip) + delete-all + upload {path}"),
            )
        )
