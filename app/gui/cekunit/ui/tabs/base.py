"""Base tab dengan validasi + confirm + async."""

from __future__ import annotations

from tkinter import ttk

from ...core.models import Result, RunSpec
from ...core.validators import ValidationError
from ...utils.event_bus import TOPIC_RESULT
from .. import theme
from ..widgets.confirm_dialog import confirm
from ..widgets.json_tree import JsonTree
from ..widgets.log_panel import LogPanel


class BaseTab(ttk.Frame):
    title: str = "Tab"
    service_key: str = ""

    def __init__(self, parent, ctx) -> None:
        """
        ctx: objek context yang menyediakan:
          runner, bus, logger, service(name), root, prefs
        """
        super().__init__(parent)
        self.ctx = ctx
        self.runner = ctx.runner
        self.bus = ctx.bus
        self.logger = ctx.logger
        self._build()

    def _build(self) -> None:
        head = ttk.Frame(self)
        head.pack(fill="x", padx=theme.PAD_L, pady=theme.PAD_M)
        ttk.Label(head, text=self.title, font=("", 14, "bold")).pack(anchor="w")

        body = ttk.Frame(self)
        body.pack(fill="x", padx=theme.PAD_L, pady=theme.PAD_S)
        self.body = body

        result_frame = ttk.LabelFrame(self, text="Result")
        result_frame.pack(fill="both", expand=True, padx=theme.PAD_L, pady=theme.PAD_S)
        self.tree = JsonTree(result_frame)
        self.tree.pack(fill="both", expand=True)

        log_frame = ttk.LabelFrame(self, text="Log")
        log_frame.pack(fill="x", padx=theme.PAD_L, pady=theme.PAD_S)
        self.log = LogPanel(log_frame)
        self.log.pack(fill="both", expand=True)

    def service(self):
        """Ambil service yang cocok dengan tab ini."""
        return self.ctx.service(self.service_key)

    def validate(self, fn, *args, **kwargs):
        """Wrapper validasi: log error kalau gagal."""
        try:
            return fn(*args, **kwargs)
        except ValidationError as exc:
            self.log.append(f"[validate] {exc}")
            return None

    def submit(self, spec: RunSpec) -> None:
        """Kirim RunSpec ke runner dengan safety confirm kalau perlu."""
        if spec.needs_confirmation:
            ok = confirm(
                self.ctx.root,
                "Konfirmasi operasi destruktif",
                spec.description or "Operasi ini tidak bisa di-undo.",
                spec.confirm_phrase or "HAPUS",
            )
            if not ok:
                self.log.append("[abort] konfirmasi dibatalkan")
                return

        if self.runner.is_busy():
            self.log.append("[busy] masih ada task berjalan")
            return

        self.log.append(f"$ {spec.command_line()}")
        self.logger.info(f"tab={self.title} cmd={spec.command_line()}")
        self.bus.publish("log", f"$ {spec.command_line()}")

        started = self.runner.submit(spec, self._on_result)
        if not started:
            self.log.append("[busy] gagal submit")

    def _on_result(self, result: Result) -> None:
        for line in result.stderr.splitlines():
            self.log.append(f"[stderr] {line}")

        if result.ok:
            self.log.append(f"[ok] {result.summary}")
            self.tree.set_data(result.data)
        else:
            self.log.append(f"[fail] {result.summary}")
            if result.error is not None:
                self.tree.set_data(
                    {
                        "error_kind": result.error.kind,
                        "error_code": result.error.code,
                        "error_message": result.error.message,
                        "retryable": result.error.retryable,
                    }
                )
                if result.is_write_refused:
                    self.log.append(
                        "[hint] set CEKUNIT_ALLOW_WRITE=1 di .env, restart GUI"
                    )
            else:
                self.tree.set_data({"raw": result.raw_stdout})

        self.bus.publish(TOPIC_RESULT, result)

    def current_json(self):
        """Untuk tombol Copy JSON."""
        return getattr(self, "_last_data", None)

    def clear(self) -> None:
        self.tree.clear()
        self.log.clear()
