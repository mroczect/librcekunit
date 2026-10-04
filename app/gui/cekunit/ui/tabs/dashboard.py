from __future__ import annotations

from tkinter import ttk

from ...core.models import RunSpec
from ...core.validators import column_pair, positive_int
from .. import theme
from ..widgets.form_field import FormField
from .base import BaseTab


class DashboardTab(BaseTab):
    title = "Dashboard"
    service_key = "dashboard"

    def __init__(self, parent, ctx) -> None:
        super().__init__(parent, ctx)
        self._build_body()

    def _build_body(self) -> None:
        self.pages = FormField(self.body, "Pages", default="2", width=6)
        self.pages.pack(fill="x")
        self.sort = FormField(self.body, "Sort", width=20)
        self.sort.pack(fill="x")
        self.direction = FormField(self.body, "Direction", default="asc", width=8)
        self.direction.pack(fill="x")
        self.search = FormField(self.body, "Search", width=30)
        self.search.pack(fill="x")

        row = ttk.Frame(self.body)
        row.pack(fill="x", pady=theme.PAD_M)
        ttk.Button(row, text="List", command=self._on_list).pack(
            side="left", padx=theme.PAD_S
        )

        ttk.Separator(self.body).pack(fill="x", pady=theme.PAD_M)

        self.unique_column = FormField(self.body, "Kolom unik", width=20)
        self.unique_column.pack(fill="x")
        ttk.Button(self.body, text="Nilai unik", command=self._on_unique).pack(
            anchor="w", padx=theme.PAD_S
        )

        ttk.Separator(self.body).pack(fill="x", pady=theme.PAD_M)

        self.delete_pair = FormField(self.body, "column=value", width=30)
        self.delete_pair.pack(fill="x")
        row2 = ttk.Frame(self.body)
        row2.pack(fill="x")
        ttk.Button(row2, text="Delete preview", command=self._on_del_preview).pack(
            side="left", padx=theme.PAD_S
        )
        ttk.Button(row2, text="Delete execute", command=self._on_del_exec).pack(
            side="left", padx=theme.PAD_S
        )

    def _on_list(self) -> None:
        pages = self.validate(positive_int, self.pages.get(), "pages")
        if pages is None:
            return
        args = ["dashboard", "list", "--pages", str(pages)]
        if self.sort.get():
            args += ["--sort", self.sort.get()]
        if self.direction.get():
            args += ["--direction", self.direction.get()]
        if self.search.get():
            args += ["--search", self.search.get()]
        self.submit(RunSpec(args=args))

    def _on_unique(self) -> None:
        col = self.unique_column.get()
        if not col:
            self.log.append("[validate] kolom kosong")
            return
        self.submit(RunSpec(args=["dashboard", "unique", col]))

    def _on_del_preview(self) -> None:
        pair = self.validate(column_pair, self.delete_pair.get())
        if pair is None:
            return
        col, val = pair
        self.submit(RunSpec(args=["dashboard", "delete-category", f"{col}={val}"]))

    def _on_del_exec(self) -> None:
        pair = self.validate(column_pair, self.delete_pair.get())
        if pair is None:
            return
        col, val = pair
        self.submit(
            RunSpec(
                args=["dashboard", "delete-category", f"{col}={val}", "--execute"],
                destructive=True,
                needs_confirmation=True,
                confirm_phrase="HAPUS",
                description=f"Hapus baris dengan {col}={val}",
            )
        )
