from __future__ import annotations

from tkinter import ttk

from ...core.models import RunSpec
from ...core.validators import columns_list, date_range
from .. import theme
from ..widgets.form_field import FormField
from .base import BaseTab


class ReportTab(BaseTab):
    title = "Report"
    service_key = "report"

    def __init__(self, parent, ctx) -> None:
        super().__init__(parent, ctx)
        self._build_body()

    def _build_body(self) -> None:
        self.mode = FormField(self.body, "Mode (today|date|range)", default="today")
        self.mode.pack(fill="x")
        self.start = FormField(self.body, "Start (YYYY-MM-DD)", width=14)
        self.start.pack(fill="x")
        self.end = FormField(self.body, "End (YYYY-MM-DD)", width=14)
        self.end.pack(fill="x")

        self.dedup_by = FormField(self.body, "Dedup by", default="userID,nopol")
        self.dedup_by.pack(fill="x")
        self.pivot_rows = FormField(self.body, "Pivot rows", default="nama")
        self.pivot_rows.pack(fill="x")
        self.pivot_values = FormField(self.body, "Pivot values", default="nopol")
        self.pivot_values.pack(fill="x")
        self.pivot_agg = FormField(self.body, "Agg", default="count-distinct")
        self.pivot_agg.pack(fill="x")
        self.out = FormField(self.body, "Output dir")
        self.out.pack(fill="x")

        row = ttk.Frame(self.body)
        row.pack(fill="x", pady=theme.PAD_M)
        ttk.Button(row, text="Run", command=self._on_run).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(row, text="Clear", command=self.clear).pack(
            side="left", padx=theme.PAD_S
        )

    def _date_args(self) -> list[str] | None:
        mode = self.mode.get().lower()
        if mode == "today":
            return ["today"]
        if mode == "date":
            from ...core.validators import date_string

            d = self.validate(date_string, self.start.get())
            if d is None:
                return None
            return ["date", d]
        if mode == "range":
            pair = self.validate(date_range, self.start.get(), self.end.get())
            if pair is None:
                return None
            s, e = pair
            return ["range", s, e]
        self.log.append(f"[validate] mode tidak dikenal: {mode}")
        return None

    def _on_run(self) -> None:
        date_args = self._date_args()
        if date_args is None:
            return
        dedup = self.validate(columns_list, self.dedup_by.get())
        if dedup is None:
            return
        rows = self.validate(columns_list, self.pivot_rows.get())
        if rows is None:
            return

        args = [
            "report",
            *date_args,
            "--dedup-by",
            ",".join(dedup),
            "--pivot-rows",
            ",".join(rows),
            "--pivot-values",
            self.pivot_values.get(),
            "--pivot-agg",
            self.pivot_agg.get() or "count-distinct",
            "--force",
        ]
        if self.out.get():
            args += ["--out", self.out.get()]
        self.submit(RunSpec(args=args))
