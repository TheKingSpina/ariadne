#!/usr/bin/env python3
"""Coordinated launcher for the dashboard and the MCP HTTP endpoint.

One write token is loaded once and passed to both children. If a child exits
unexpectedly the other is terminated and the supervisor exits non-zero. This
is a test-environment supervisor, not a service manager: no restarts, no
readiness beyond a TCP port check.
"""
import argparse
import os
import signal
import socket
import subprocess
import sys
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "dashboard"))
from server import load_write_token  # noqa: E402


def wait_http(host, port, timeout=30):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection((host, port), timeout=1):
                return True
        except OSError:
            time.sleep(0.25)
    raise TimeoutError(f"porta {host}:{port} non risponde")


def main():
    parser = argparse.ArgumentParser(description="Avvio e arresto coordinati di dashboard e MCP HTTP.")
    parser.add_argument("--host", default="0.0.0.0", help="bind del server MCP HTTP")
    parser.add_argument("--port", type=int, default=8080, help="porta del server MCP HTTP")
    parser.add_argument("--dashboard-host", default=None, help="bind della dashboard (default: come --host)")
    parser.add_argument("--dashboard-port", type=int, default=8090)
    parser.add_argument("--database", default=os.environ.get("ARIADNE_DATABASE", "/data/ariadne.sqlite3"))
    args = parser.parse_args()
    dashboard_host = args.dashboard_host or args.host
    try:
        token = load_write_token(args.database, os.environ.get("ARIADNE_WRITE_TOKEN"))
    except (OSError, ValueError) as exc:
        raise SystemExit(f"impossibile inizializzare token: {exc}") from exc
    env = dict(os.environ)
    env.update(
        ARIADNE_WRITE_TOKEN=token,
        ARIADNE_STORE_URL=f"http://127.0.0.1:{args.dashboard_port}",
        ARIADNE_STORE_HOST=f"127.0.0.1:{args.dashboard_port}",
    )
    children = []
    lock = threading.Lock()
    running = threading.Event()
    failure = threading.Event()

    def stop():
        if not running.is_set():
            return
        running.clear()
        sys.stderr.write("supervisor: arresto in corso\n"); sys.stderr.flush()
        with lock:
            procs = list(children)
        for _, process in procs:
            if process.poll() is None:
                process.terminate()
        for _, process in procs:
            if process.poll() is None:
                try:
                    process.wait(15)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(5)
        os._exit(1 if failure.is_set() else 0)

    def spawn(name, argv):
        process = subprocess.Popen(argv, env=env)
        with lock:
            children.append((name, process))
        def watch():
            code = process.wait()
            if not running.is_set():
                return
            failure.set()
            sys.stderr.write(f"{name}: terminato inattesamente (codice {code}); arresto dell'altro figlio\n")
            stop()
        threading.Thread(target=watch, daemon=True).start()

    def on_signal(signum, _frame):
        sys.stderr.write(f"supervisor: ricevuto segnale {signum}\n")
        stop()

    signal.signal(signal.SIGTERM, on_signal)
    signal.signal(signal.SIGINT, on_signal)
    running.set()
    spawn("dashboard", [sys.executable, str(ROOT / "dashboard" / "server.py"),
                        "--host", dashboard_host, "--port", str(args.dashboard_port),
                        "--database", args.database])
    wait_http("127.0.0.1", args.dashboard_port)
    spawn("mcp", [sys.executable, str(ROOT / "mcp" / "http_server.py"),
                  "--host", args.host, "--port", str(args.port)])
    wait_http("127.0.0.1", args.port)
    sys.stderr.write("supervisor: dashboard e MCP HTTP attivi; SIGTERM o SIGINT per arrestare\n")
    while running.is_set():
        time.sleep(1)
    os._exit(1 if failure.is_set() else 0)


if __name__ == "__main__":
    main()
