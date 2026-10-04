from __future__ import annotations

from tkinter import ttk

from ...core.models import RunSpec
from ...core.validators import date_range, positive_int
from .. import theme
from ..widgets.form_field import FormField
from .base import BaseTab


class InputUserTab(BaseTab):
    title = "Input User"
    service_key = "input_user"

    def __init__(self, parent, ctx) -> None:
        super().__init__(parent, ctx)
        self._build_body()

    def _build_body(self) -> None:
        ttk.Label(self.body, text="Export CSV", font=("", 11, "bold")).pack(anchor="w")

        self.start = FormField(self.body, "Start (YYYY-MM-DD)", width=14)
        self.start.pack(fill="x")
        self.end = FormField(self.body, "End (YYYY-MM-DD)", width=14)
        self.end.pack(fill="x")
        self.out = FormField(self.body, "Output dir")
        self.out.pack(fill="x")

        row = ttk.Frame(self.body)
        row.pack(fill="x", pady=theme.PAD_M)
        ttk.Button(row, text="Export today", command=self._on_today).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(row, text="Export range", command=self._on_range).pack(
            side="left", padx=theme.PAD_S
        )

        ttk.Separator(self.body).pack(fill="x", pady=theme.PAD_M)

        ttk.Label(self.body, text="Scan + group", font=("", 11, "bold")).pack(
            anchor="w"
        )
        self.pages = FormField(self.body, "Pages", default="5", width=6)
        self.pages.pack(fill="x")
        self.by = FormField(self.body, "Group by", width=20)
        self.by.pack(fill="x")
        self.csv_out = FormField(self.body, "CSV out")
        self.csv_out.pack(fill="x")

        ttk.Button(self.body, text="Report", command=self._on_report).pack(
            anchor="w", padx=theme.PAD_S, pady=theme.PAD_S
        )

    def _on_today(self) -> None:
        args = ["input-user", "export", "today"]
        if self.out.get():
            args += ["--out", self.out.get()]
        self.submit(RunSpec(args=args))

    def _on_range(self) -> None:
        pair = self.validate(date_range, self.start.get(), self.end.get())
        if pair is None:
            return
        start, end = pair
        args = ["input-user", "export", "range", start, end]
        if self.out.get():
            args += ["--out", self.out.get()]
        self.submit(RunSpec(args=args))

    def _on_report(self) -> None:
        pages = self.validate(positive_int, self.pages.get(), "pages")
        if pages is None:
            return
        args = ["input-user", "report", "--pages", str(pages)]
        if self.by.get():
            args += ["--by", self.by.get()]
        if self.csv_out.get():
            args += ["--csv", self.csv_out.get()]
        self.submit(RunSpec(args=args))
