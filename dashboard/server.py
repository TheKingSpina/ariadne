#!/usr/bin/env python3
"""Local HTTP dashboard and append-only SQLite store; not an MCP server."""
import argparse
from contextlib import closing
from datetime import datetime, timezone
import hashlib
import hmac
import json
import os
from pathlib import Path
import re
import secrets
import sqlite3
import sys
import threading
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "mcp"))
from ariadne_server import tool  # noqa: E402: same read-only validation backend

MAX_BODY = 2_000_000
NAME = re.compile(r"[a-zA-Z0-9_-]{1,64}\Z")
MCP_HEALTH_URL = os.environ.get("ARIADNE_MCP_HEALTH_URL", "http://127.0.0.1:8080/healthz")


class Conflict(Exception):
    pass


class ChangeFeed:
    """In-process commit notifications; initial event and reconnect resync clients."""
    def __init__(self):
        self.condition = threading.Condition()
        self.revision = 0
        self.slots = threading.BoundedSemaphore(16)

    def notify(self):
        with self.condition:
            self.revision += 1
            self.condition.notify_all()


class Store:
    def __init__(self, path):
        self.path = Path(path)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        with closing(self.connect()) as db:
            db.executescript("""
                PRAGMA journal_mode=WAL;
                CREATE TABLE IF NOT EXISTS logs (id TEXT PRIMARY KEY, version TEXT NOT NULL);
                CREATE TABLE IF NOT EXISTS events (log_id TEXT NOT NULL, seq INTEGER NOT NULL, line TEXT NOT NULL,
                    PRIMARY KEY (log_id, seq), FOREIGN KEY (log_id) REFERENCES logs(id));
                CREATE TABLE IF NOT EXISTS fixtures (log_id TEXT NOT NULL, fixture_id TEXT NOT NULL, binding TEXT NOT NULL,
                    PRIMARY KEY (log_id, fixture_id), FOREIGN KEY (log_id) REFERENCES logs(id));
                CREATE TABLE IF NOT EXISTS memories (id INTEGER PRIMARY KEY AUTOINCREMENT, content TEXT NOT NULL,
                    source TEXT NOT NULL, tags TEXT NOT NULL, created_at TEXT NOT NULL, supersedes INTEGER);
                CREATE TRIGGER IF NOT EXISTS logs_no_update BEFORE UPDATE ON logs BEGIN SELECT RAISE(ABORT,'append-only'); END;
                CREATE TRIGGER IF NOT EXISTS logs_no_delete BEFORE DELETE ON logs BEGIN SELECT RAISE(ABORT,'append-only'); END;
                CREATE TRIGGER IF NOT EXISTS events_no_update BEFORE UPDATE ON events BEGIN SELECT RAISE(ABORT,'append-only'); END;
                CREATE TRIGGER IF NOT EXISTS events_no_delete BEFORE DELETE ON events BEGIN SELECT RAISE(ABORT,'append-only'); END;
                CREATE TRIGGER IF NOT EXISTS fixtures_no_update BEFORE UPDATE ON fixtures BEGIN SELECT RAISE(ABORT,'append-only'); END;
                CREATE TRIGGER IF NOT EXISTS fixtures_no_delete BEFORE DELETE ON fixtures BEGIN SELECT RAISE(ABORT,'append-only'); END;
                CREATE TRIGGER IF NOT EXISTS memories_no_update BEFORE UPDATE ON memories BEGIN SELECT RAISE(ABORT,'append-only'); END;
                CREATE TRIGGER IF NOT EXISTS memories_no_delete BEFORE DELETE ON memories BEGIN SELECT RAISE(ABORT,'append-only'); END;
            """)
            # Migrazione additiva per gli archivi creati prima delle rettifiche.
            # ALTER TABLE aggiunge solo una colonna nullable: nessuna riga è
            # riscritta e i trigger append-only restano attivi.
            columns = {row[1] for row in db.execute("PRAGMA table_info(memories)")}
            if "supersedes" not in columns:
                db.execute("ALTER TABLE memories ADD COLUMN supersedes INTEGER")

    def connect(self):
        db = sqlite3.connect(self.path, timeout=10)
        db.execute("PRAGMA foreign_keys=ON")
        db.execute("PRAGMA synchronous=FULL")
        return db

    @staticmethod
    def name(name):
        if not isinstance(name, str) or not NAME.fullmatch(name):
            raise ValueError("ID non valido: usa 1–64 lettere, cifre, _ o -")
        return name

    @staticmethod
    def _read(db, name):
        row = db.execute("SELECT version FROM logs WHERE id=?", (name,)).fetchone()
        if row is None:
            raise KeyError(name)
        events = [line for (line,) in db.execute("SELECT line FROM events WHERE log_id=? ORDER BY seq", (name,))]
        fixtures = [json.loads(binding) for (binding,) in db.execute("SELECT binding FROM fixtures WHERE log_id=? ORDER BY fixture_id", (name,))]
        return {"version": row[0], "log_jsonl": "".join(events), "fixtures": fixtures}

    def list(self):
        with closing(self.connect()) as db:
            return [{"id": row[0], "version": row[1], "events": row[2]} for row in db.execute(
                "SELECT l.id,l.version,COUNT(e.seq) FROM logs l LEFT JOIN events e ON e.log_id=l.id GROUP BY l.id,l.version ORDER BY l.id")]

    def get(self, name):
        self.name(name)
        with closing(self.connect()) as db:
            return self._read(db, name)

    def remember(self, data):
        if not isinstance(data, dict) or set(data) - {"content", "source", "tags", "supersedes"}:
            raise ValueError("schema memoria non valido")
        content, source, tags = data.get("content"), data.get("source", ""), data.get("tags", [])
        supersedes = data.get("supersedes")
        if not isinstance(content, str) or not content.strip() or len(content) > 4000:
            raise ValueError("contenuto richiesto, massimo 4000 caratteri")
        if not isinstance(source, str) or len(source) > 500:
            raise ValueError("fonte dichiarata non valida")
        if not isinstance(tags, list) or len(tags) > 10 or any(not isinstance(t, str) or not t.strip() or len(t) > 64 for t in tags) or len(set(tags)) != len(tags):
            raise ValueError("etichette non valide")
        if supersedes is not None and (type(supersedes) is not int or supersedes < 1):
            raise ValueError("supersedes deve essere l'ID intero di una memoria esistente")
        created = datetime.now(timezone.utc).isoformat(timespec="seconds")
        with closing(self.connect()) as db:
            db.execute("BEGIN IMMEDIATE")
            try:
                if supersedes is not None:
                    if db.execute("SELECT 1 FROM memories WHERE id=?", (supersedes,)).fetchone() is None:
                        raise ValueError(f"memoria da rettificare non trovata: #{supersedes}")
                    if db.execute("SELECT 1 FROM memories WHERE supersedes=?", (supersedes,)).fetchone() is not None:
                        raise ValueError(f"memoria #{supersedes} ha già una rettifica")
                result = db.execute("INSERT INTO memories(content,source,tags,created_at,supersedes) VALUES (?,?,?,?,?)",
                                    (content, source, json.dumps(tags, ensure_ascii=False, separators=(",", ":")), created, supersedes))
                memory_id = result.lastrowid
                db.commit()
            except Exception:
                db.rollback()
                raise
        return {"id": memory_id, "content": content, "source": source, "tags": tags,
                "created_at": created, "supersedes": supersedes, "superseded_by": None}

    @staticmethod
    def _date(value, label):
        if value is None or value == "":
            return None
        if not isinstance(value, str) or not re.fullmatch(r"\d{4}-\d{2}-\d{2}", value):
            raise ValueError(f"{label}: usa il formato YYYY-MM-DD")
        try:
            datetime.strptime(value, "%Y-%m-%d")
        except ValueError as exc:
            raise ValueError(f"{label}: data non valida") from exc
        return value

    def recall(self, query="", limit=50, tag=None, source=None, date_from=None, date_to=None, current=False):
        if not isinstance(query, str) or len(query) > 200 or type(limit) is not int or not 1 <= limit <= 100:
            raise ValueError("ricerca o limite non validi")
        if tag is not None and (not isinstance(tag, str) or not tag.strip() or len(tag) > 64):
            raise ValueError("etichetta non valida")
        if source is not None and (not isinstance(source, str) or len(source) > 500):
            raise ValueError("fonte non valida")
        start, end = self._date(date_from, "from"), self._date(date_to, "to")
        if start and end and start > end:
            raise ValueError("intervallo date invertito")
        if type(current) is not bool:
            raise ValueError("current deve essere booleano")
        with closing(self.connect()) as db:
            rows = db.execute("SELECT id,content,source,tags,created_at,supersedes FROM memories "
                              "WHERE instr(lower(content),lower(?))>0 OR instr(lower(source),lower(?))>0 OR instr(lower(tags),lower(?))>0 "
                              "ORDER BY id DESC", (query, query, query)).fetchall()
            successors = {target: source_id for source_id, target in
                          db.execute("SELECT id,supersedes FROM memories WHERE supersedes IS NOT NULL")}
        found = []
        for row in rows:
            tags = json.loads(row[3])
            created = row[4]
            if tag is not None and tag not in tags:
                continue
            if source is not None and source.lower() not in row[2].lower():
                continue
            day = created[:10]
            if start and day < start:
                continue
            if end and day > end:
                continue
            superseded_by = successors.get(row[0])
            if current and superseded_by is not None:
                continue
            found.append({"id": row[0], "content": row[1], "source": row[2], "tags": tags,
                          "created_at": created, "supersedes": row[5], "superseded_by": superseded_by})
            if len(found) >= limit:
                break
        return found

    def diagnostics(self):
        """Reachability and local state. Never a statement of verified conformity."""
        with closing(self.connect()) as db:
            logs, events = db.execute(
                "SELECT (SELECT COUNT(*) FROM logs),(SELECT COUNT(*) FROM events)").fetchone()
            versions = [row[0] for row in db.execute("SELECT DISTINCT version FROM logs ORDER BY version")]
        try:
            info = self.path.stat()
            updated = datetime.fromtimestamp(info.st_mtime, timezone.utc).isoformat(timespec="seconds")
            size = info.st_size
        except OSError:
            updated, size = None, None
        return {
            "archive": {"reachable": True, "database": self.path.name, "logs": logs, "events": events,
                        "size_bytes": size, "updated_at": updated},
            "protocol_versions": versions,
            "mcp": mcp_health(),
            "conformity": {"status": "not_verified",
                           "note": "Raggiungibilità e integrità locale non dimostrano conformità, verità o autenticità."},
        }

    def append(self, name, data):
        self.name(name)
        if not isinstance(data, dict) or set(data) - {"version", "log_jsonl", "fixtures"}:
            raise ValueError("schema richiesta non valido")
        version, text, fixtures = data.get("version"), data.get("log_jsonl"), data.get("fixtures", [])
        if version not in ("1", "2") or not isinstance(text, str) or not text.endswith("\n") or not isinstance(fixtures, list):
            raise ValueError("versione, JSONL o fixture non valide")
        lines = text.splitlines(keepends=True)
        if not lines or any(not line.endswith("\n") or line == "\n" for line in lines):
            raise ValueError("JSONL incompleto")
        with closing(self.connect()) as db:
            db.execute("BEGIN IMMEDIATE")
            try:
                try:
                    old = self._read(db, name)
                except KeyError:
                    old = None
                if old and (old["version"] != version or not text.startswith(old["log_jsonl"])):
                    raise Conflict("il log proposto non estende esattamente il prefisso archiviato")
                bindings = {f["fixture_id"]: f for f in old["fixtures"]} if old else {}
                for fixture in fixtures:
                    if not isinstance(fixture, dict) or set(fixture) != {"fixture_id", "sha256", "bytes", "anchor"}:
                        raise ValueError("fixture non valida")
                    fid = fixture["fixture_id"]
                    if not isinstance(fid, str) or not fid:
                        raise ValueError("ID fixture non valido")
                    if not isinstance(fixture["bytes"], str) or not isinstance(fixture["sha256"], str) or hashlib.sha256(fixture["bytes"].encode("utf-8")).hexdigest() != fixture["sha256"]:
                        raise ValueError("digest fixture non corrispondente")
                    if fid in bindings and bindings[fid] != fixture:
                        raise Conflict("fixture esistente non modificabile")
                    bindings[fid] = fixture
                args = {"version": version, "log_jsonl": text, "fixtures": list(bindings.values())}
                verified = tool("ariadne_verify", args)
                if not old:
                    db.execute("INSERT INTO logs(id,version) VALUES (?,?)", (name, version))
                existing = {f["fixture_id"] for f in old["fixtures"]} if old else set()
                for fid, binding in bindings.items():
                    if fid not in existing:
                        db.execute("INSERT INTO fixtures(log_id,fixture_id,binding) VALUES (?,?,?)", (name, fid, json.dumps(binding, ensure_ascii=False, sort_keys=True, separators=(",", ":"))))
                count = len(old["log_jsonl"].splitlines()) if old else 0
                db.executemany("INSERT INTO events(log_id,seq,line) VALUES (?,?,?)", ((name, i, line) for i, line in enumerate(lines[count:], count)))
                db.commit()
                return verified
            except Exception:
                db.rollback()
                raise


def mcp_health():
    """Best-effort reachability check of the co-located MCP HTTP endpoint."""
    try:
        with urllib.request.urlopen(MCP_HEALTH_URL, timeout=2) as response:
            payload = json.loads(response.read(65536))
        return {"reachable": True, "status": payload.get("status"),
                "sessions": payload.get("sessions"),
                "protocol_versions": payload.get("protocol_versions")}
    except (OSError, ValueError, urllib.error.URLError) as exc:
        return {"reachable": False, "error": type(exc).__name__}


class Handler(BaseHTTPRequestHandler):
    server_version = "AriadneDashboard/0.2"
    def log_message(self, fmt, *args):
        sys.stderr.write("dashboard: " + fmt % args + "\n")

    def send_bytes(self, status, body, content_type="application/json; charset=utf-8"):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.send_header("X-Content-Type-Options", "nosniff")
        self.send_header("Content-Security-Policy", "default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'")
        self.end_headers()
        self.wfile.write(body)

    def send_json(self, status, value):
        self.send_bytes(status, json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode())

    def dispatch(self):
        host = self.headers.get("Host", "")
        if host not in self.server.allowed_hosts:
            self.send_json(403, {"error": "Host non consentito"}); return
        origin = self.headers.get("Origin")
        if origin and origin != f"http://{host}":
            self.send_json(403, {"error": "Origin non consentita"}); return
        parts = urlsplit(self.path).path.strip("/").split("/")
        if self.command == "GET" and parts == [""]:
            self.send_bytes(200, (ROOT / "dashboard" / "index.html").read_bytes(), "text/html; charset=utf-8"); return
        if self.command == "GET" and len(parts) == 1 and parts[0] in ("app.js", "style.css"):
            mime = "text/javascript" if parts[0].endswith(".js") else "text/css"
            self.send_bytes(200, (ROOT / "dashboard" / parts[0]).read_bytes(), mime + "; charset=utf-8"); return
        if self.command == "GET" and parts == ["api", "logs"]:
            self.send_json(200, {"logs": self.server.store.list()}); return
        if self.command == "GET" and parts == ["api", "stream"]:
            self.stream(); return
        if self.command == "GET" and parts == ["api", "diagnostics"]:
            self.send_json(200, self.server.store.diagnostics()); return
        if parts == ["api", "memories"]:
            if self.command == "GET":
                query = parse_qs(urlsplit(self.path).query, keep_blank_values=True)
                if set(query) - {"q", "limit", "tag", "source", "from", "to", "current"} or any(len(values) != 1 for values in query.values()):
                    raise ValueError("parametri ricerca non validi")
                current = query.get("current", ["false"])[0]
                if current not in ("true", "false"):
                    raise ValueError("current deve essere true o false")
                self.send_json(200, {"memories": self.server.store.recall(
                    query.get("q", [""])[0], int(query.get("limit", ["50"])[0]),
                    query.get("tag", [None])[0], query.get("source", [None])[0],
                    query.get("from", [None])[0], query.get("to", [None])[0],
                    current == "true")}); return
            if self.command == "POST":
                self.authorize_write()
                result = self.server.store.remember(self.read_json())
                self.server.feed.notify()
                self.send_json(201, result); return
        if len(parts) == 3 and parts[:2] == ["api", "logs"]:
            name = parts[2]
            if self.command == "GET":
                self.send_json(200, self.server.store.get(name)); return
            if self.command == "POST":
                self.authorize_write()
                result = self.server.store.append(name, self.read_json())
                self.server.feed.notify()
                self.send_json(200, result); return
        if self.command == "GET" and len(parts) == 5 and parts[:2] == ["api", "logs"] and parts[3] == "views":
            saved = self.server.store.get(parts[2])
            saved["view"] = parts[4]
            self.send_json(200, tool("ariadne_view", saved)); return
        self.send_json(404, {"error": "risorsa non trovata"})

    def authorize_write(self):
        token = self.headers.get("X-Ariadne-Token", "")
        if not hmac.compare_digest(token, self.server.write_token):
            raise PermissionError("token di scrittura non valido")

    def stream(self):
        feed = self.server.feed
        if not feed.slots.acquire(blocking=False):
            self.send_json(503, {"error": "troppe connessioni realtime"}); return
        try:
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream; charset=utf-8")
            self.send_header("Cache-Control", "no-cache, no-store")
            self.send_header("X-Accel-Buffering", "no")
            self.send_header("X-Content-Type-Options", "nosniff")
            self.end_headers()
            with feed.condition:
                revision = feed.revision
            self.wfile.write(b"event: change\ndata: {}\n\n")
            self.wfile.flush()
            while True:
                with feed.condition:
                    feed.condition.wait_for(lambda: feed.revision != revision, timeout=15)
                    changed = feed.revision != revision
                    revision = feed.revision
                self.wfile.write(b"event: change\ndata: {}\n\n" if changed else b": keepalive\n\n")
                self.wfile.flush()
        except (BrokenPipeError, ConnectionResetError, OSError):
            pass
        finally:
            feed.slots.release()

    def read_json(self):
        if self.headers.get_content_type() != "application/json":
            raise ValueError("Content-Type deve essere application/json")
        try:
            size = int(self.headers.get("Content-Length", ""))
        except ValueError:
            raise ValueError("Content-Length richiesto")
        if not 0 <= size <= MAX_BODY:
            raise ValueError("richiesta troppo grande")
        raw = self.rfile.read(size)
        if len(raw) != size:
            raise ValueError("corpo incompleto")
        return json.loads(raw)

    def handle_request(self):
        try:
            self.dispatch()
        except KeyError:
            self.send_json(404, {"error": "log non trovato"})
        except Conflict as exc:
            self.send_json(409, {"error": str(exc)})
        except PermissionError:
            self.send_json(403, {"error": "token di scrittura non valido"})
        except (ValueError, UnicodeError, json.JSONDecodeError) as exc:
            self.send_json(400, {"error": str(exc)})
        except sqlite3.Error:
            self.send_json(503, {"error": "archivio non disponibile"})

    do_GET = handle_request
    do_POST = handle_request


def load_write_token(database, configured=None):
    if configured is not None:
        token = configured
    else:
        path = Path(database).parent / ".write-token"
        path.parent.mkdir(parents=True, exist_ok=True)
        try:
            fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        except FileExistsError:
            token = path.read_text(encoding="ascii").strip()
        else:
            token = secrets.token_urlsafe(32)
            with os.fdopen(fd, "w", encoding="ascii") as stream:
                stream.write(token + "\n")
                stream.flush()
                os.fsync(stream.fileno())
    if len(token) < 32 or not token.isascii() or any(ord(c) < 33 or ord(c) > 126 for c in token):
        raise ValueError("token deve avere almeno 32 caratteri ASCII stampabili")
    return token


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8090)
    parser.add_argument("--database", default=os.environ.get("ARIADNE_DATABASE", "./data/ariadne.sqlite3"))
    args = parser.parse_args()
    try:
        token = load_write_token(args.database, os.environ.get("ARIADNE_WRITE_TOKEN"))
    except (OSError, ValueError) as exc:
        raise SystemExit(f"impossibile inizializzare token: {exc}") from exc
    server = ThreadingHTTPServer((args.host, args.port), Handler)
    server.store = Store(args.database)
    server.feed = ChangeFeed()
    server.write_token = token
    server.allowed_hosts = {f"127.0.0.1:{args.port}", f"localhost:{args.port}"}
    print(f"Dashboard: http://127.0.0.1:{args.port}/", file=sys.stderr, flush=True)
    fingerprint = hashlib.sha256(token.encode("ascii")).hexdigest()[:12]
    token_path = Path(args.database).parent / ".write-token"
    print(f"Token di scrittura: configurato in {token_path} (impronta {fingerprint})", file=sys.stderr, flush=True)
    server.serve_forever()


if __name__ == "__main__":
    main()
