
import websocket
import sys

def test_ws(url, protocols):
    try:
        print(f"Connecting to {url} with protocols {protocols}...")
        ws = websocket.create_connection(url, subprotocols=protocols, timeout=5)
        print("Success! Handshake complete.")
        print(f"Server selected protocol: {ws.subprotocol}")
        ws.close()
        return True
    except Exception as e:
        print(f"Failure: {e}")
        return False

if __name__ == "__main__":
    token = "tadpole-dev-token-2026"
    url = "ws://localhost:8000/v1/engine/ws"
    # Port 8000 on localhost
    test_ws(url, [f"bearer.{token}", "tadpole-pulse-v1"])
    # Also test 127.0.0.1
    url_ip = "ws://127.0.0.1:8000/v1/engine/ws"
    test_ws(url_ip, [f"bearer.{token}", "tadpole-pulse-v1"])
