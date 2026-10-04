from __future__ import annotations

from tkinter import ttk

from ...core.models import RunSpec
from ...core.validators import columns_list
from .. import theme
from ..widgets.file_picker import FilePicker
from ..widgets.form_field import FormField
from .base import BaseTab


class CsvTab(BaseTab):
    title = "CSV Tools"
    service_key = "csv"

    def __init__(self, parent, ctx) -> None:
        super().__init__(parent, ctx)
        self._build_body()

    def _build_body(self) -> None:
        self.picker = FilePicker(
            self.body,
            "File CSV",
            filetypes=[("CSV", "*.csv"), ("All", "*.*")],
        )
        self.picker.pack(fill="x")

        ttk.Button(self.body, text="Check", command=self._on_check).pack(
            anchor="w", padx=theme.PAD_S, pady=theme.PAD_S
        )

        ttk.Separator(self.body).pack(fill="x", pady=theme.PAD_M)

        ttk.Label(self.body, text="Dedup", font=("", 11, "bold")).pack(anchor="w")
        self.dedup_by = FormField(self.body, "By (col,col)", default="userID,nopol")
        self.dedup_by.pack(fill="x")
        self.keep = FormField(self.body, "Keep (first|last)", default="first", width=10)
        self.keep.pack(fill="x")
        ttk.Button(self.body, text="Dedup", command=self._on_dedup).pack(
            anchor="w", padx=theme.PAD_S, pady=theme.PAD_S
        )

        ttk.Separator(self.body).pack(fill="x", pady=theme.PAD_M)

        ttk.Label(self.body, text="Pivot", font=("", 11, "bold")).pack(anchor="w")
        self.rows = FormField(self.body, "Rows (col,col)", default="nama")
        self.rows.pack(fill="x")
        self.values = FormField(self.body, "Values", default="nopol")
        self.values.pack(fill="x")
        self.agg = FormField(
            self.body, "Agg (count|count-distinct|sum)", default="count-distinct"
        )
        self.agg.pack(fill="x")
        ttk.Button(self.body, text="Pivot", command=self._on_pivot).pack(
            anchor="w", padx=theme.PAD_S, pady=theme.PAD_S
        )

    def _on_check(self) -> None:
        path = self.picker.get()
        if not path:
            self.log.append("[validate] file kosong")
            return
        self.submit(RunSpec(args=["csv", "check", path]))

    def _on_dedup(self) -> None:
        path = self.picker.get()
        if not path:
            self.log.append("[validate] file kosong")
            return
        cols = self.validate(columns_list, self.dedup_by.get())
        if cols is None:
            return
        args = [
            "csv",
            "dedup",
            path,
            "--by",
            ",".join(cols),
            "--keep",
            self.keep.get() or "first",
            "--force",
        ]
        self.submit(RunSpec(args=args))

    def _on_pivot(self) -> None:
        path = self.picker.get()
        if not path:
            self.log.append("[validate] file kosong")
            return
        cols = self.validate(columns_list, self.rows.get())
        if cols is None:
            return
        args = [
            "csv",
            "pivot",
            path,
            "--rows",
            ",".join(cols),
            "--values",
            self.values.get(),
            "--agg",
            self.agg.get() or "count-distinct",
            "--force",
        ]
        self.submit(RunSpec(args=args))
