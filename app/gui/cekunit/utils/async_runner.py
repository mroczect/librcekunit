"""Async runner dengan callback di UI thread."""

from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from queue import Empty, Queue
from threading import Thread

from ..core.client import CliClient, CliNotFoundError
from ..core.models import ErrorBody, Result, RunSpec


@dataclass
class _Outcome:
    result: Result
    on_done: Callable[[Result], None]


class AsyncRunner:
    """Worker thread + antrian hasil."""

    def __init__(self, root, client: CliClient, poll_ms: int = 80) -> None:
        self._root = root
        self._client = client
        self._poll_ms = poll_ms
        self._queue: Queue[_Outcome] = Queue()
        self._worker: Thread | None = None
        self._schedule()

    def is_busy(self) -> bool:
        return self._worker is not None and self._worker.is_alive()

    def submit(self, spec: RunSpec, on_done: Callable[[Result], None]) -> bool:
        if self.is_busy():
            return False
        self._worker = Thread(target=self._run, args=(spec, on_done), daemon=True)
        self._worker.start()
        return True

    def _run(self, spec: RunSpec, on_done: Callable[[Result], None]) -> None:
        try:
            result = self._client.run(spec)
        except CliNotFoundError as exc:
            result = Result(
                ok=False,
                command=spec.args[0] if spec.args else "unknown",
                error=ErrorBody(
                    kind="cli_not_found",
                    code=127,
                    message=str(exc),
                    retryable=False,
                ),
                exit_code=127,
            )
        except Exception as exc:  # noqa: BLE001
            result = Result(
                ok=False,
                command=spec.args[0] if spec.args else "unknown",
                error=ErrorBody(
                    kind="internal",
                    code=1,
                    message=str(exc),
                    retryable=False,
                ),
                exit_code=1,
            )
        self._queue.put(_Outcome(result=result, on_done=on_done))

    def _schedule(self) -> None:
        self._drain()
        self._root.after(self._poll_ms, self._schedule)

    def _drain(self) -> None:
        while True:
            try:
                outcome = self._queue.get_nowait()
            except Empty:
                return
            try:
                outcome.on_done(outcome.result)
            except Exception:  # noqa: BLE001
                pass
