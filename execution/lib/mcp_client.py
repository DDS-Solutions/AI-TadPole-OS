"""
@docs ARCHITECTURE:Core

### AI Context Alignment
- **Subsystem**: Infrastructure Automation / execution/lib/mcp_client
- **Primary Entrypoints**: `TadpoleClient`, `TadpoleClientError`

### ⚠️ Invariants & Non-Negotiables
- `[Structural]` Zero external dependencies — standard library only.
- `[Structural]` Platform-agnostic IPC abstraction (Named Pipe on Windows, UDS on Unix).

### 🔍 Debugging & Observability
- **Local Errors**: `TadpoleClientError`
- **Telemetry Targets**: none declared
- **Witness Tests**: none declared

Zero-dependency Python client for the TadpoleOS IPC Bridge.

Connects to the Sovereign Engine's Named Pipe (Windows) or Unix Domain Socket (Linux/macOS)
and provides a simple API for listing tools, querying schemas, and calling tools.

Usage:
    from execution.lib.mcp_client import TadpoleClient

    client = TadpoleClient.auto_connect()
    tools = client.list_tools()
    schema = client.get_tool_schema("read_file")
    client.close()

Protocol: Newline-delimited JSON-RPC 2.0 (NDJSON).
Dependencies: NONE (stdlib only: socket, json, os, sys, hashlib, struct).
"""

import json
import os
import socket
import struct
import sys
import hashlib
from pathlib import Path
from typing import Any, Dict, List, Optional


class TadpoleClientError(Exception):
    """Raised when the IPC bridge returns a JSON-RPC error."""
    def __init__(self, code: int, message: str, data: Any = None):
        self.code = code
        self.rpc_message = message
        self.data = data
        super().__init__(f"JSON-RPC Error {code}: {message}")


class TadpoleClient:
    """
    Zero-dependency IPC client for the TadpoleOS Sovereign Engine.
    
    Communicates over Named Pipe (Windows) or Unix Domain Socket (Unix)
    using newline-delimited JSON-RPC 2.0.
    """

    def __init__(self, pipe_path: str):
        self._pipe_path = pipe_path
        self._sock: Optional[socket.socket] = None
        self._file = None
        self._request_id = 0
        self._tool_cache: Dict[str, dict] = {}
        self._connect()

    def _connect(self):
        """Establish connection to the IPC bridge."""
        if sys.platform == "win32":
            # Windows Named Pipe — connect via Win32 file API
            import ctypes
            import ctypes.wintypes

            GENERIC_READ = 0x80000000
            GENERIC_WRITE = 0x40000000
            OPEN_EXISTING = 3
            INVALID_HANDLE_VALUE = ctypes.wintypes.HANDLE(-1).value

            handle = ctypes.windll.kernel32.CreateFileW(
                self._pipe_path,
                GENERIC_READ | GENERIC_WRITE,
                0,  # no sharing
                None,  # default security
                OPEN_EXISTING,
                0,  # default attributes
                None,
            )

            if handle == INVALID_HANDLE_VALUE:
                err = ctypes.get_last_error() or ctypes.windll.kernel32.GetLastError()
                raise ConnectionError(
                    f"Cannot connect to IPC bridge at {self._pipe_path} (Win32 error {err}). "
                    f"Is the Sovereign Engine running?"
                )

            # Wrap the Win32 handle in a Python file object for read/write
            import msvcrt
            fd = msvcrt.open_osfhandle(handle, os.O_RDWR)
            self._file = os.fdopen(fd, "r+b", buffering=0)
        else:
            # Unix Domain Socket
            self._sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            try:
                self._sock.connect(self._pipe_path)
                self._sock.settimeout(30.0)
            except (ConnectionRefusedError, FileNotFoundError) as e:
                raise ConnectionError(
                    f"Cannot connect to IPC bridge at {self._pipe_path}. "
                    f"Is the Sovereign Engine running? ({e})"
                )

    def _send_request(self, method: str, params: Optional[dict] = None) -> Any:
        """Send a JSON-RPC 2.0 request and return the result."""
        self._request_id += 1
        request = {
            "jsonrpc": "2.0",
            "id": self._request_id,
            "method": method,
        }
        if params:
            request["params"] = params

        payload = json.dumps(request, separators=(",", ":")) + "\n"
        payload_bytes = payload.encode("utf-8")

        if self._file:
            # Windows named pipe
            self._file.write(payload_bytes)
            self._file.flush()
            response_line = b""
            while True:
                chunk = self._file.read(1)
                if not chunk or chunk == b"\n":
                    break
                response_line += chunk
        else:
            # Unix socket
            self._sock.sendall(payload_bytes)
            response_line = b""
            while True:
                chunk = self._sock.recv(1)
                if not chunk or chunk == b"\n":
                    break
                response_line += chunk

        if not response_line:
            raise ConnectionError("IPC bridge closed connection unexpectedly")

        response = json.loads(response_line.decode("utf-8"))

        if "error" in response and response["error"]:
            err = response["error"]
            raise TadpoleClientError(
                code=err.get("code", -1),
                message=err.get("message", "Unknown error"),
                data=err.get("data"),
            )

        return response.get("result")

    # ── Public API ────────────────────────────────────────────

    def ping(self) -> str:
        """Health check — returns 'pong' if the bridge is alive."""
        return self._send_request("ping")

    def list_tools(self, use_cache: bool = True) -> List[dict]:
        """
        List all registered tools with their metadata.
        Results are cached after the first call.
        """
        if use_cache and self._tool_cache:
            return list(self._tool_cache.values())

        tools = self._send_request("list_tools")
        self._tool_cache = {t["name"]: t for t in tools}
        return tools

    def get_tool_schema(self, tool_name: str) -> dict:
        """Get full schema for a specific tool."""
        return self._send_request("get_tool_schema", {"name": tool_name})

    def close(self):
        """Close the IPC connection."""
        try:
            if self._file:
                self._file.close()
                self._file = None
            if self._sock:
                self._sock.close()
                self._sock = None
        except Exception:
            pass

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()

    def __del__(self):
        self.close()

    # ── Factory ───────────────────────────────────────────────

    @staticmethod
    def pipe_path_for(workspace_root: str) -> str:
        """
        Compute the deterministic pipe path for a workspace.
        Must match the Rust implementation in ipc_bridge.rs.
        """
        # Rust uses std::hash::DefaultHasher which is SipHash-1-3.
        # We replicate the same hash here using Python's built-in hash.
        # IMPORTANT: Python's hash() is NOT stable across processes,
        # so we use the same algorithm as Rust: hash the path bytes.
        import ctypes

        # Replicate Rust's DefaultHasher (SipHash) behavior:
        # We use a simpler deterministic approach — hash the UTF-8 bytes
        # with a stable algorithm. Since Rust's hash is architecture-dependent,
        # we write the pipe path to a well-known discovery file instead.
        #
        # Fallback: read the pipe path from the discovery file.
        discovery_file = os.path.join(workspace_root, ".tmp", "ipc_bridge_path.txt")
        if os.path.exists(discovery_file):
            with open(discovery_file, "r") as f:
                return f.read().strip()

        # If no discovery file, compute a portable hash
        path_hash = hashlib.sha256(workspace_root.encode("utf-8")).hexdigest()[:16]

        if sys.platform == "win32":
            return rf"\\.\pipe\tadpoleos-ipc-{path_hash}"
        else:
            return f"/tmp/tadpoleos-ipc-{path_hash}.sock"

    @classmethod
    def auto_connect(cls, workspace_root: Optional[str] = None) -> "TadpoleClient":
        """
        Auto-detect the workspace root and connect to the IPC bridge.
        
        Resolution order:
        1. Explicit workspace_root parameter
        2. TADPOLE_WORKSPACE env var
        3. Walk up from CWD looking for .tadpole/ or server-rs/
        """
        if workspace_root is None:
            workspace_root = os.environ.get("TADPOLE_WORKSPACE")

        if workspace_root is None:
            # Walk up from CWD
            current = Path.cwd()
            for parent in [current, *current.parents]:
                if (parent / ".tadpole").exists() or (parent / "server-rs").exists():
                    workspace_root = str(parent)
                    break

        if workspace_root is None:
            raise ConnectionError(
                "Cannot determine workspace root. Set TADPOLE_WORKSPACE env var "
                "or run from within the project directory."
            )

        pipe_path = cls.pipe_path_for(workspace_root)
        return cls(pipe_path)


# ── CLI Entry Point ───────────────────────────────────────────

if __name__ == "__main__":
    """Quick CLI for testing: python -m execution.lib.mcp_client [ping|list|schema <name>]"""
    import sys

    try:
        client = TadpoleClient.auto_connect()
    except ConnectionError as e:
        print(f"❌ {e}", file=sys.stderr)
        sys.exit(1)

    cmd = sys.argv[1] if len(sys.argv) > 1 else "ping"

    try:
        if cmd == "ping":
            result = client.ping()
            print(f"✅ {result}")
        elif cmd == "list":
            tools = client.list_tools()
            for t in tools:
                mutating = "🔴" if t.get("is_mutating") else "🟢"
                print(f"  {mutating} {t['name']}: {t.get('description', '')[:80]}")
            print(f"\n  Total: {len(tools)} tools")
        elif cmd == "schema" and len(sys.argv) > 2:
            schema = client.get_tool_schema(sys.argv[2])
            print(json.dumps(schema, indent=2))
        else:
            print("Usage: python -m execution.lib.mcp_client [ping|list|schema <name>]")
    except TadpoleClientError as e:
        print(f"❌ RPC Error: {e}", file=sys.stderr)
        sys.exit(1)
    finally:
        client.close()
