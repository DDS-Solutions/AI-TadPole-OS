"""
@docs ARCHITECTURE:Quality:Verification

### AI Context Alignment
- **Subsystem**: System Core / test_ws
- **Primary Entrypoints**: `test_ws`, `main`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Deterministic internal state integrity and strict interface contract compliance.

### 🔍 Debugging & Observability
- **Local Errors**: none
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared
"""

from __future__ import annotations

try:
    import websocket
except ImportError:
    websocket = None

import sys


def test_ws(url: str, protocols: list[str]) -> bool:
    if websocket is None:
        print("Failure: install 'websocket-client' to run the WebSocket smoke test.")
        return False

    try:
        print(f"Connecting to {url} with protocols {protocols}...")
        ws = websocket.create_connection(url, subprotocols=protocols, timeout=5)
        print("Success! Handshake complete.")
        print(f"Server selected protocol: {ws.subprotocol}")
        ws.close()
        return True
    except Exception as error:
        print(f"Failure: {error}")
        return False


def main() -> int:
    import os

    token = os.environ.get("NEURAL_TOKEN")
    if not token:
        print("Failure: NEURAL_TOKEN not found in environment.")
        return 1

    protocols = [f"bearer.{token}", "tadpole-pulse-v1"]
    urls = (
        "ws://localhost:8000/v1/engine/ws",
        "ws://127.0.0.1:8000/v1/engine/ws",
    )
    return 0 if all(test_ws(url, protocols) for url in urls) else 1


if __name__ == "__main__":
    sys.exit(main())
