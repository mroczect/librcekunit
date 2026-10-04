"""Main window."""

from __future__ import annotations

from dataclasses import dataclass
from tkinter import BOTH, Tk, X, ttk

from ..core.client import CliClient
from ..core.config import (
    get_gui_log_path,
    get_prefs_path,
    load_env_file,
)
from ..services.auth_service import AuthService
from ..services.csv_service import CsvService
from ..services.dashboard_service import DashboardService
from ..services.input_user_service import InputUserService
from ..services.replace_service import ReplaceService
from ..services.report_service import ReportService
from ..utils.async_runner import AsyncRunner
from ..utils.event_bus import TOPIC_RESULT, EventBus
from ..utils.json_store import JsonStore
from ..utils.logger import GuiLogger
from . import shortcuts, theme
from .clipboard import copy_json
from .tabs import (
    AuthTab,
    CsvTab,
    DashboardTab,
    InfoTab,
    InputDataTab,
    InputUserTab,
    ReplaceTab,
    ReportTab,
)
from .widgets.safety_badge import SafetyBadge
from .widgets.status_bar import StatusBar
from .widgets.toolbar import Toolbar


@dataclass
class Context:
    root: Tk
    runner: AsyncRunner
    bus: EventBus
    logger: GuiLogger
    prefs: JsonStore
    services: dict

    def service(self, key: str):
        return self.services.get(key)


class App:
    def __init__(self) -> None:
        self.root = Tk()
        self.root.title("Cek Unit")
        self.root.geometry("1100x820")
        theme.apply(self.root)

        # Bootstrap
        self.logger = GuiLogger(get_gui_log_path())
        self.prefs = JsonStore(get_prefs_path())
        self.bus = EventBus()
        self.client = CliClient()
        self.runner = AsyncRunner(self.root, self.client)

        self.services = {
            "auth": AuthService(self.runner, self.bus, self.logger),
            "dashboard": DashboardService(self.runner, self.bus, self.logger),
            "input_user": InputUserService(self.runner, self.bus, self.logger),
            "csv": CsvService(self.runner, self.bus, self.logger),
            "report": ReportService(self.runner, self.bus, self.logger),
            "replace": ReplaceService(self.runner, self.bus, self.logger),
        }

        self.ctx = Context(
            root=self.root,
            runner=self.runner,
            bus=self.bus,
            logger=self.logger,
            prefs=self.prefs,
            services=self.services,
        )

        self._build_layout()
        self._bind_shortcuts()
        self._wire_bus()
        self._schedule_status()

    # ------------------------------------------------------------------

    def _build_layout(self) -> None:
        head = ttk.Frame(self.root)
        head.pack(fill=X, padx=theme.PAD_L, pady=theme.PAD_M)

        ttk.Label(head, text="Cek Unit", font=("", 16, "bold")).pack(side="left")
        self.safety = SafetyBadge(head)
        self.safety.pack(side="right", padx=theme.PAD_M)

        self.toolbar = Toolbar(
            self.root,
            on_refresh=self.on_refresh,
            on_reload_env=self.on_reload_env,
            on_clear_log=self.on_clear_log,
            on_copy_json=self.on_copy_json,
        )
        self.toolbar.pack(fill=X, padx=theme.PAD_L, pady=theme.PAD_S)

        notebook = ttk.Notebook(self.root)
        notebook.pack(fill=BOTH, expand=True, padx=theme.PAD_L, pady=theme.PAD_S)

        self.tabs = {
            "info": InfoTab(notebook, self.ctx),
            "auth": AuthTab(notebook, self.ctx),
            "dashboard": DashboardTab(notebook, self.ctx),
            "input-user": InputUserTab(notebook, self.ctx),
            "input-data": InputDataTab(notebook, self.ctx),
            "csv": CsvTab(notebook, self.ctx),
            "report": ReportTab(notebook, self.ctx),
            "replace": ReplaceTab(notebook, self.ctx),
        }
        for key, tab in self.tabs.items():
            notebook.add(tab, text=key)

        self.notebook = notebook

        self.status = StatusBar(self.root)
        self.status.pack(fill=X, padx=theme.PAD_L, pady=theme.PAD_S)

    def _bind_shortcuts(self) -> None:
        shortcuts.bind(self.root, self)

    def _wire_bus(self) -> None:
        self.bus.subscribe(TOPIC_RESULT, lambda _r: self.status.refresh())

    def _schedule_status(self) -> None:
        self.status.set_busy(self.runner.is_busy())
        self.safety.refresh()
        self.root.after(1500, self._schedule_status)

    # ------------------------------------------------------------------
    # Shortcut handlers
    # ------------------------------------------------------------------

    def on_refresh(self) -> None:
        self.status.refresh()
        self.safety.refresh()

    def on_clear_log(self) -> None:
        current = self._current_tab()
        if current is not None:
            current.log.clear()

    def on_copy_json(self) -> None:
        current = self._current_tab()
        if current is None:
            return
        data = current.current_json()
        if data is None:
            return
        copy_json(self.root, data)

    def on_reload_env(self) -> None:
        from pathlib import Path

        load_env_file(Path.cwd() / ".env")
        self.status.refresh()
        self.safety.refresh()
        self.logger.info("reload env")

    def on_quit(self) -> None:
        self.root.destroy()

    def _current_tab(self):
        try:
            tab_id = self.notebook.select()
            widget = self.notebook.nametowidget(tab_id)
        except Exception:  # noqa: BLE001
            return None
        return widget

    # ------------------------------------------------------------------

    def run(self) -> None:
        self.root.mainloop()
