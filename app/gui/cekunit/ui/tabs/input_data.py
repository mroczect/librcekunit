from __future__ import annotations

from tkinter import END, Text, ttk

from ...core.models import RunSpec
from .. import theme
from ..widgets.file_picker import FilePicker
from .base import BaseTab


class InputDataTab(BaseTab):
    title = "Input Data"
    service_key = "input_data"

    def __init__(self, parent, ctx) -> None:
        super().__init__(parent, ctx)
        self._build_body()

    def _build_body(self) -> None:
        ttk.Label(self.body, text="Single record", font=("", 11, "bold")).pack(
            anchor="w"
        )

        self.fields = Text(
            self.body,
            height=6,
            wrap="word",
            bg=theme.BG,
            fg=theme.FG,
            insertbackground=theme.FG,
            font=(theme.MONO_FAMILY, theme.MONO_SIZE),
        )
        self.fields.pack(fill="x")
        self.fields.insert(
            "1.0",
            "# Format: satu per baris, key=value\n"
            "# Contoh:\n"
            "# no_perjanjian=SP-25-001\n"
            "# nama_nasabah=SADIMAN\n",
        )

        ttk.Button(self.body, text="Insert single", command=self._on_single).pack(
            anchor="w", padx=theme.PAD_S, pady=theme.PAD_S
        )

        ttk.Separator(self.body).pack(fill="x", pady=theme.PAD_M)

        ttk.Label(self.body, text="Upload CSV", font=("", 11, "bold")).pack(anchor="w")
        self.picker = FilePicker(
            self.body,
            "CSV file",
            filetypes=[("CSV", "*.csv"), ("All", "*.*")],
        )
        self.picker.pack(fill="x")
        ttk.Button(self.body, text="Upload CSV", command=self._on_csv).pack(
            anchor="w", padx=theme.PAD_S, pady=theme.PAD_S
        )

    def _parse_kv(self) -> list[tuple[str, str]]:
        text = self.fields.get("1.0", END)
        result = []
        for raw in text.splitlines():
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            if "=" not in line:
                continue
            k, _, v = line.partition("=")
            result.append((k.strip(), v.strip()))
        return result

    def _on_single(self) -> None:
        fields = self._parse_kv()
        if not fields:
            self.log.append("[validate] minimal satu field")
            return
        args = ["input-data", "single", "--execute"]
        for k, v in fields:
            args += [f"--{k}", v]
        self.submit(
            RunSpec(
                args=args,
                destructive=True,
                needs_confirmation=True,
                confirm_phrase="INSERT-1",
                description=f"Insert 1 record dengan {len(fields)} field",
            )
        )

    def _on_csv(self) -> None:
        path = self.picker.get()
        if not path:
            self.log.append("[validate] file kosong")
            return
        self.submit(
            RunSpec(
                args=["input-data", "csv", path, "--execute"],
                destructive=True,
                needs_confirmation=True,
                confirm_phrase="INSERT",
                description=f"Upload CSV dari {path}",
            )
        )
