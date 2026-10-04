"""Base service — semua service extend ini."""

from __future__ import annotations

from ..core.models import RunSpec


class Service:
    """Base class service."""

    def __init__(self, runner, bus, logger) -> None:
        self.runner = runner
        self.bus = bus
        self.logger = logger

    def run(self, spec: RunSpec, on_done=None) -> bool:
        """Submit RunSpec ke runner."""

        def _default_callback(result):
            self.bus.publish("result", result)

        cb = on_done or _default_callback
        ok = self.runner.submit(spec, cb)
        self.logger.info(f"submit: {spec.command_line()} ok={ok}")
        return ok
