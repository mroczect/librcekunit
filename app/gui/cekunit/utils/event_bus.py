"""Event bus sederhana."""

from __future__ import annotations

from collections import defaultdict
from collections.abc import Callable
from typing import Any

Handler = Callable[[Any], None]


class EventBus:
    """Pub-sub antar tab tanpa coupling langsung."""

    def __init__(self) -> None:
        self._handlers: dict[str, list[Handler]] = defaultdict(list)

    def subscribe(self, topic: str, handler: Handler) -> None:
        self._handlers[topic].append(handler)

    def publish(self, topic: str, payload: Any = None) -> None:
        for handler in list(self._handlers.get(topic, [])):
            try:
                handler(payload)
            except Exception:  # noqa: BLE001
                pass


# Topik standar
TOPIC_AUTH_CHANGED = "auth_changed"
TOPIC_SESSION_CHANGED = "session_changed"
TOPIC_RESULT = "result"
TOPIC_LOG = "log"
TOPIC_WRITE_STATE_CHANGED = "write_state_changed"
