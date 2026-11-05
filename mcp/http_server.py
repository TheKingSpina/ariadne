#!/usr/bin/env python3
"""Local MCP Streamable HTTP transport; test environment only.

Exposes the same Protocol as the stdio adapter over POST /mcp. There is no
client authentication and CORS is open by default: never expose beyond the
loopback interface or a trusted test network. Diagnostics go to stderr only;
stdout stays empty.
"""
import argparse
import json
import os
import pathlib
import sys
import threading
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import urlsplit

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from ariadne_server import MAX_LINE, Protocol, VERSIONS, error_response  # noqa: E402

ALLOW_ORIGIN = os.environ.get("ARIADNE_MCP_ALLOW_ORIGIN", "*")
CORS_METHODS = "POST, GET, DELETE, OPTIONS"
CORS_HEADERS = "Content-Type, Accept, Mcp-Session-Id, MCP-Protocol-Version, Last-Event-ID, Authorization"
MAX_SESSIONS = 256


class State:
    """Session registry plus the shared anonymous protocol."""

    def __init__(self):
        self.lock = threading.Lock()
        self.sessions = {}
        self.shared = Protocol()

    def open_session(self):
        """Fresh session for a client that supplies no recognizable one."""
        with self.lock:
            if len(self.sessions) >= MAX_SESSIONS:
                raise RuntimeError("session limit reached")
            session_id = uuid.uuid4().hex
            protocol = Protocol()
            self.sessions[session_id] = protocol
            return session_id, protocol

    def get(self, session_id):
        with self.lock:
            return self.sessions.get(session_id)

    def drop(self, session_id):
        with self.lock:
            return self.sessions.pop(session_id, None) is not None

    def health(self):
        with self.lock:
            return {"sessions": len(self.sessions),
                    "initialized": self.shared.initialized or any(p.initialized for p in self.sessions.values())}


class Handler(BaseHTTPRequestHandler):
    server_version = "AriadneMcpHttp/0.5"

    def log_message(self, fmt, *args):
        sys.stderr.write("mcp: " + fmt % args + "\n")

    def cors(self):
        return {"Access-Control-Allow-Origin": ALLOW_ORIGIN,
                "Access-Control-Allow-Methods": CORS_METHODS,
                "Access-Control-Allow-Headers": CORS_HEADERS,
                "Access-Control-Allow-Private-Network": "true",
                "Access-Control-Max-Age": "600",
                "Access-Control-Expose-Headers": "Mcp-Session-Id"}

    def send_bytes(self, status, body, content_type="application/json; charset=utf-8", extra=None):
        self.send_response(status)
        for key, value in self.cors().items():
            self.send_header(key, value)
        for key, value in (extra or {}).items():
            self.send_header(key, value)
        if content_type is not None:
            self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        if body:
            self.wfile.write(body)

    def send_json(self, status, value, extra=None):
        body = "" if value is None else json.dumps(value, ensure_ascii=False, separators=(",", ":"))
        self.send_bytes(status, body.encode("utf-8"), extra=extra)

    def do_OPTIONS(self):
        if urlsplit(self.path).path != "/mcp":
            self.send_json(404, {"error": "risorsa non trovata"}); return
        self.send_bytes(204, b"", content_type=None)

    def do_GET(self):
        path = urlsplit(self.path).path
        if path == "/healthz":
            health = self.server.state.health()
            self.send_json(200, {"status": "ok", "protocol_versions": sorted(VERSIONS), **health}); return
        if path == "/mcp":
            self.send_json(405, {"error": "metodo non permesso; usare POST o DELETE"}, extra={"Allow": CORS_METHODS}); return
        self.send_json(404, {"error": "risorsa non trovata"})

    def do_DELETE(self):
        if urlsplit(self.path).path != "/mcp":
            self.send_json(404, {"error": "risorsa non trovata"}); return
        session_id = self.headers.get("Mcp-Session-Id", "")
        if not session_id or not self.server.state.drop(session_id):
            self.send_json(404, {"error": "Mcp-Session-Id sconosciuto"}); return
        self.send_bytes(204, b"", content_type=None)

    def do_POST(self):
        if urlsplit(self.path).path != "/mcp":
            self.send_json(404, {"error": "risorsa non trovata"}); return
        try:
            length = int(self.headers.get("Content-Length", ""))
        except ValueError:
            length = -1
        if length > MAX_LINE:
            # Drain part of the declared body so well-behaved oversized clients
            # get the status instead of a broken pipe; then close the channel.
            remaining = min(length, 4 * MAX_LINE)
            while remaining > 0:
                chunk = self.rfile.read(min(remaining, 65536))
                if not chunk:
                    break
                remaining -= len(chunk)
            self.close_connection = True
            self.send_json(413, {"error": "richiesta troppo grande"}); return
        if length < 0:
            self.send_json(400, {"error": "Content-Length richiesto"}); return
        body = self.rfile.read(length)
        if len(body) != length:
            self.send_json(400, {"error": "corpo incompleto"}); return
        if self.headers.get_content_type() != "application/json":
            self.send_json(415, {"error": "Content-Type deve essere application/json"}); return
        try:
            payload = json.loads(body)
        except (ValueError, UnicodeDecodeError):
            self.send_json(400, error_response(None, -32700, "parse error")); return
        requests_list = [payload] if isinstance(payload, dict) else payload if isinstance(payload, list) else None
        if requests_list is None or any(not isinstance(item, dict) for item in requests_list):
            self.send_json(400, error_response(None, -32600, "invalid request")); return
        state = self.server.state
        header_sid = self.headers.get("Mcp-Session-Id", "")
        session_extra = {}
        if any(item.get("method") == "initialize" for item in requests_list):
            protocol = state.get(header_sid) if header_sid else None
            if protocol is None:
                try:
                    session_extra["Mcp-Session-Id"], protocol = state.open_session()
                except RuntimeError:
                    self.send_json(503, {"error": "troppe sessioni"}); return
            if not header_sid:
                # Header-less clients never see Mcp-Session-Id: they keep using
                # the shared anonymous protocol, so mirror the handshake there.
                for item in requests_list:
                    state.shared.handle(dict(item))
        elif header_sid:
            protocol = state.get(header_sid)
            if protocol is None:
                self.send_json(404, {"error": "Mcp-Session-Id sconosciuto"}); return
        else:
            protocol = state.shared
        outputs = [outcome for item in requests_list if (outcome := protocol.handle(item)) is not None]
        if not outputs:
            self.send_bytes(202, b"", content_type=None, extra=session_extra); return
        accept = self.headers.get("Accept", "")
        if "text/event-stream" in accept and "application/json" not in accept:
            stream = "".join("event: message\ndata: " + json.dumps(outcome, ensure_ascii=False, separators=(",", ":")) + "\n\n" for outcome in outputs)
            self.send_bytes(200, stream.encode("utf-8"), content_type="text/event-stream", extra=session_extra); return
        value = outputs[0] if len(outputs) == 1 else outputs
        self.send_json(200, value, extra=session_extra)


def make_server(host="127.0.0.1", port=8080):
    server = ThreadingHTTPServer((host, port), Handler)
    server.daemon_threads = True
    server.state = State()
    return server


def main():
    parser = argparse.ArgumentParser(description="Endpoint MCP Streamable HTTP locale; solo test.")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8080)
    args = parser.parse_args()
    server = make_server(args.host, args.port)
    print(f"MCP HTTP: http://{args.host}:{args.port}/mcp (CORS {ALLOW_ORIGIN})", file=sys.stderr, flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
