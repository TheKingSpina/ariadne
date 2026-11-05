#!/usr/bin/env python3
"""Esportazione verificabile e ripristino non distruttivo dell'archivio.

Strumento CLI autonomo: usa solo la standard library più i moduli dello
stesso repository necessari per coerenza (`dashboard.server.Store` per lo
schema e i trigger esatti dell'archivio, `mcp.ariadne_server.tool` per la
convalida `ariadne_verify` dei log prima che un ripristino diventi attivo).

Il token di scrittura (`/data/.write-token`) non viene mai letto, esportato
né modificato: il tool lavora solo sull'archivio SQLite e sul file di export.

Formato dell'export (`ariadne-backup/1`): un singolo file JSON con
`schema`, `created_at`, `source` (solo nome del file del database),
`data` (le tabelle `logs`, `events`, `fixtures`, `memories`) e
`integrity.digest`, lo SHA-256 del JSON canonico di `data`
(`sort_keys=True`, separatori `","`/`":"`, `ensure_ascii=False`).
"""
import argparse
from contextlib import closing
from datetime import datetime, timezone
import hashlib
import hmac
import json
import os
from pathlib import Path
import sqlite3
import sys
import tempfile

ROOT = Path(__file__).resolve().parent.parent
for _path in (str(ROOT), str(ROOT / "mcp")):
    if _path not in sys.path:
        sys.path.insert(0, _path)

from ariadne_server import tool  # noqa: E402: stessa convalida usata dal server
from dashboard.server import NAME, Store  # noqa: E402: schema e trigger condivisi

SCHEMA = "ariadne-backup/1"
TABLES = ("logs", "events", "fixtures", "memories")
ALGORITHM = "sha256"


class BackupError(Exception):
    """Errore previsto: messaggio chiaro e codice d'uscita non zero."""


def canonical(value):
    """JSON canonico di `data`: l'unico formato di cui si calcola il digest."""
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def canonical_digest(value):
    return hashlib.sha256(canonical(value).encode("utf-8")).hexdigest()


def utc_now():
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def utc_stamp():
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def sidecars(path):
    base = str(path)
    return [Path(base + suffix) for suffix in ("-wal", "-shm", "-journal")]


def read_rows(source):
    """Legge tutte le tabelle da una copia coerente del database attivo."""
    if not source.is_file():
        raise BackupError(f"database di origine inesistente: {source.name}")
    fd, tempfile_name = tempfile.mkstemp(prefix="ariadne-export-", suffix=".sqlite3")
    os.close(fd)
    temp = Path(tempfile_name)
    try:
        with closing(sqlite3.connect(str(source), timeout=10)) as src:
            with closing(sqlite3.connect(temp, timeout=10)) as snapshot:
                src.backup(snapshot)
                rows = {
                    "logs": [{"id": r[0], "version": r[1]} for r in snapshot.execute(
                        "SELECT id,version FROM logs ORDER BY id")],
                    "events": [{"log_id": r[0], "seq": r[1], "line": r[2]} for r in snapshot.execute(
                        "SELECT log_id,seq,line FROM events ORDER BY log_id,seq")],
                    "fixtures": [{"log_id": r[0], "fixture_id": r[1],
                                  "binding": _load_json_column(r[2], f"fixture {r[0]}/{r[1]}")}
                                 for r in snapshot.execute(
                        "SELECT log_id,fixture_id,binding FROM fixtures ORDER BY log_id,fixture_id")],
                    "memories": [{"id": r[0], "content": r[1], "source": r[2],
                                  "tags": _load_json_column(r[3], f"memoria {r[0]}"), "created_at": r[4],
                                  "supersedes": r[5]}
                                 for r in snapshot.execute(
                        "SELECT id,content,source,tags,created_at,supersedes FROM memories ORDER BY id")],
                }
        return rows
    except (OSError, sqlite3.Error) as exc:
        raise BackupError(f"database di origine incompleto o illeggibile: {exc}") from exc
    finally:
        for leftover in [temp, *sidecars(temp)]:
            leftover.unlink(missing_ok=True)


def _load_json_column(text, label):
    try:
        value = json.loads(text)
    except (TypeError, ValueError) as exc:
        raise BackupError(f"{label}: contenuto JSON non leggibile") from exc
    return value


def export(database, output, force=False):
    """Scrive il file di export con permessi 0600; mai `.write-token`."""
    database = Path(database)
    output = Path(output)
    if output.exists() and not force:
        raise BackupError(f"file di output già presente: {output.name} (usa --force per sovrascrivere)")
    rows = read_rows(database)
    document = {
        "schema": SCHEMA,
        "created_at": utc_now(),
        "source": database.name,
        "data": rows,
        "integrity": {"algorithm": ALGORITHM, "digest": canonical_digest(rows)},
    }
    payload = (canonical(document) + "\n").encode("utf-8")
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC
    try:
        fd = os.open(output, flags, 0o600)
    except OSError as exc:
        raise BackupError(f"impossibile creare il file di output: {exc}") from exc
    with os.fdopen(fd, "wb") as stream:
        os.fchmod(stream.fileno(), 0o600)
        stream.write(payload)
        stream.flush()
        os.fsync(stream.fileno())
    counts = {name: len(rows[name]) for name in TABLES}
    return {"digest": document["integrity"]["digest"], "counts": counts}


def load(path):
    """Legge e convalida schema e integrità del file di export."""
    path = Path(path)
    try:
        raw = json.loads(path.read_bytes())
    except OSError as exc:
        raise BackupError(f"file di export illeggibile: {exc}") from exc
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise BackupError(f"file di export non è JSON valido: {exc}") from exc
    if not isinstance(raw, dict) or raw.get("schema") != SCHEMA:
        found = raw.get("schema") if isinstance(raw, dict) else None
        raise BackupError(f"schema ignoto: {found!r}, atteso {SCHEMA!r}")
    integrity = raw.get("integrity")
    if not isinstance(integrity, dict) or integrity.get("algorithm") != ALGORITHM:
        raise BackupError("integrità assente o algoritmo non supportato")
    digest = integrity.get("digest")
    data = raw.get("data")
    if not isinstance(digest, str) or len(digest) != 64 or any(c not in "0123456789abcdef" for c in digest):
        raise BackupError("digest mancante o mal formato")
    if not isinstance(data, dict):
        raise BackupError("campo data assente o non valido")
    computed = canonical_digest(data)
    if not hmac.compare_digest(computed, digest):
        raise BackupError(f"digest non corrisponde: atteso {digest}, calcolato {computed}")
    return raw, data


def validate_content(data):
    """Controllo strutturale del contenuto prima del ripristino."""
    if set(data) != set(TABLES):
        raise BackupError("il campo data non contiene esattamente logs, events, fixtures e memories")
    for name in TABLES:
        if not all(isinstance(item, dict) for item in data[name]):
            raise BackupError(f"tabella {name}: voci non valide")
    ids = set()
    for row in data["logs"]:
        if set(row) != {"id", "version"}:
            raise BackupError("registro con campi inattesi")
        if not isinstance(row["id"], str) or not NAME.fullmatch(row["id"]):
            raise BackupError("ID registro non valido")
        if row["version"] not in ("1", "2"):
            raise BackupError(f"versione {row['version']!r} non valida per il registro {row['id']}")
        ids.add(row["id"])
    seen = set()
    for row in data["events"]:
        if set(row) != {"log_id", "seq", "line"}:
            raise BackupError("evento con campi inattesi")
        if row["log_id"] not in ids:
            raise BackupError(f"evento collegato al registro sconosciuto {row['log_id']!r}")
        if type(row["seq"]) is not int or row["seq"] < 0:
            raise BackupError(f"sequenza non valida nel registro {row['log_id']!r}")
        if not isinstance(row["line"], str) or not row["line"].endswith("\n") or row["line"] == "\n":
            raise BackupError(f"linea evento incompleta nel registro {row['log_id']!r}")
        if (row["log_id"], row["seq"]) in seen:
            raise BackupError(f"evento duplicato seq {row['seq']} nel registro {row['log_id']!r}")
        seen.add((row["log_id"], row["seq"]))
    seen = set()
    for row in data["fixtures"]:
        if set(row) != {"log_id", "fixture_id", "binding"}:
            raise BackupError("fixture con campi inattesi")
        if row["log_id"] not in ids:
            raise BackupError(f"fixture collegata al registro sconosciuto {row['log_id']!r}")
        if not isinstance(row["fixture_id"], str) or not row["fixture_id"]:
            raise BackupError("ID fixture non valido")
        if not isinstance(row["binding"], dict):
            raise BackupError(f"binding non valido per la fixture {row['fixture_id']!r}")
        if (row["log_id"], row["fixture_id"]) in seen:
            raise BackupError(f"fixture duplicata {row['fixture_id']!r} nel registro {row['log_id']!r}")
        seen.add((row["log_id"], row["fixture_id"]))
    memory_ids = set()
    for row in data["memories"]:
        if set(row) not in ({"id", "content", "source", "tags", "created_at"},
                            {"id", "content", "source", "tags", "created_at", "supersedes"}):
            raise BackupError("memoria con campi inattesi")
        if type(row["id"]) is not int or row["id"] < 1:
            raise BackupError("ID memoria non valido")
        if row["id"] in memory_ids:
            raise BackupError(f"ID memoria duplicato: {row['id']}")
        memory_ids.add(row["id"])
        if not isinstance(row["content"], str) or not isinstance(row["source"], str) \
                or not isinstance(row["created_at"], str):
            raise BackupError("memoria con campi testuali non validi")
        if not isinstance(row["tags"], list) or any(not isinstance(tag, str) for tag in row["tags"]):
            raise BackupError(f"etichette non valide per la memoria {row['id']}")
        supersedes = row.get("supersedes")
        if supersedes is not None and (type(supersedes) is not int or supersedes < 1):
            raise BackupError(f"supersedes non valido per la memoria {row['id']}")
    targets = set()
    for row in data["memories"]:
        target = row.get("supersedes")
        if target is None:
            continue
        if target not in memory_ids:
            raise BackupError(f"memoria {row['id']} rettifica un ID inesistente: {target}")
        if target == row["id"]:
            raise BackupError(f"memoria {row['id']} non può rettificare se stessa")
        if target in targets:
            raise BackupError(f"memoria {target} ha più di una rettifica")
        targets.add(target)
    return {name: len(data[name]) for name in TABLES}


def has_rows(database):
    """True se il file esiste e contiene righe in almeno una tabella."""
    if not database.exists():
        return False
    try:
        read_only = sqlite3.connect(f"{database.resolve().as_uri()}?mode=ro", timeout=10, uri=True)
    except sqlite3.Error:
        return True  # file non riconosciuto come database: rifiutiamo senza --replace
    with closing(read_only) as db:
        try:
            tables = [row[0] for row in db.execute(
                "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")]
            for table in tables:
                if db.execute(f"SELECT COUNT(*) FROM \"{table}\"").fetchone()[0] > 0:
                    return True
        except sqlite3.Error:
            return True  # struttura illeggibile: meglio rifiutare
    return False


def snapshot_existing(database):
    """Copia coerente del database esistente in `<database>.pre-restore-<UTC>`."""
    stem = f"{database.name}.pre-restore-{utc_stamp()}"
    pre = database.with_name(stem)
    suffix = 2
    while pre.exists():
        pre = database.with_name(f"{stem}-{suffix}")
        suffix += 1
    with closing(sqlite3.connect(f"{database.resolve().as_uri()}?mode=ro", timeout=10, uri=True)) as src:
        with closing(sqlite3.connect(pre, timeout=10)) as dst:
            src.backup(dst)
            dst.execute("PRAGMA wal_checkpoint(TRUNCATE)")
    for leftover in sidecars(pre):
        if leftover.exists():
            leftover.unlink()
    return pre


def build(temporary, data):
    """Costruisce un nuovo database con lo schema esatto di `dashboard.server`."""
    Store(temporary)  # crea schema, trigger e WAL header come faccia la dashboard
    with closing(sqlite3.connect(str(temporary), timeout=10)) as db:
        db.execute("PRAGMA foreign_keys=ON")
        db.execute("PRAGMA journal_mode=DELETE")  # nessun WAL: la sostituzione resta un file solo
        db.execute("BEGIN IMMEDIATE")
        try:
            db.executemany("INSERT INTO logs(id,version) VALUES (?,?)",
                           ((row["id"], row["version"]) for row in data["logs"]))
            db.executemany("INSERT INTO events(log_id,seq,line) VALUES (?,?,?)",
                           ((row["log_id"], row["seq"], row["line"]) for row in data["events"]))
            db.executemany("INSERT INTO fixtures(log_id,fixture_id,binding) VALUES (?,?,?)",
                           ((row["log_id"], row["fixture_id"], canonical(row["binding"]))
                            for row in data["fixtures"]))
            db.executemany("INSERT INTO memories(id,content,source,tags,created_at,supersedes) VALUES (?,?,?,?,?,?)",
                           ((row["id"], row["content"], row["source"],
                             canonical(row["tags"]), row["created_at"], row.get("supersedes"))
                            for row in data["memories"]))
            db.commit()
        except Exception:
            db.rollback()
            raise


def verify_built(temporary):
    """Valida ogni log ricostruito con `ariadne_verify` prima di attivarlo."""
    with closing(sqlite3.connect(str(temporary), timeout=10)) as db:
        entries = {}
        versions = {row[0]: row[1] for row in db.execute("SELECT id,version FROM logs")}
        for log_id, seq, line in db.execute("SELECT log_id,seq,line FROM events ORDER BY log_id,seq"):
            entries.setdefault(log_id, []).append(line)
        fixture_map = {}
        for log_id, fixture_id, binding in db.execute(
                "SELECT log_id,fixture_id,binding FROM fixtures ORDER BY log_id,fixture_id"):
            fixture_map.setdefault(log_id, []).append(json.loads(binding))
    for log_id, version in sorted(versions.items()):
        args = {"version": version, "log_jsonl": "".join(entries.get(log_id, [])),
                "fixtures": fixture_map.get(log_id, [])}
        try:
            tool("ariadne_verify", args)
        except Exception as exc:
            raise BackupError(f"verifica del registro {log_id!r} fallita: {exc}") from exc


def activate(temporary, database):
    """Sostituisce in modo atomico il database attivo; mai il token `.write-token`."""
    for leftover in sidecars(database):
        if leftover.exists():
            leftover.unlink()
    os.replace(temporary, database)
    directory = os.open(database.parent, os.O_RDONLY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)


def remove_temporary(temporary):
    if temporary.exists():
        temporary.unlink()
    for leftover in sidecars(temporary):
        if leftover.exists():
            leftover.unlink()


def verify(path):
    """Calcola il digest sull'intero contenuto e restituisce schema, conteggi e digest."""
    document, data = load(path)
    return {
        "schema": document["schema"],
        "source": document.get("source"),
        "created_at": document.get("created_at"),
        "digest": document["integrity"]["digest"],
        "counts": {name: len(data[name]) for name in TABLES},
    }


def restore(path, database, replace=False):
    """Ripristino: valida, ricostruisce, verifica ogni log, poi sostituisce."""
    database = Path(database)
    temporary = Path(str(database) + ".restore-tmp")
    document, data = load(path)
    counts = validate_content(data)
    try:
        if database.exists() and has_rows(database) and not replace:
            raise BackupError(f"destinazione non vuota: {database.name} richiede --replace")
        if database.parent:
            database.parent.mkdir(parents=True, exist_ok=True)
        remove_temporary(temporary)
        build(temporary, data)
        verify_built(temporary)
        pre = None
        if replace and database.exists():
            pre = snapshot_existing(database)
        activate(temporary, database)
        for leftover in sidecars(temporary):  # i sidecar residui del tmp non devono sopravvivere
            if leftover.exists():
                leftover.unlink()
    except BaseException:
        remove_temporary(temporary)
        raise
    return {"digest": document["integrity"]["digest"], "counts": counts,
            "pre_restore": pre.name if pre else None}


def build_parser():
    parser = argparse.ArgumentParser(
        prog="dashboard/backup.py",
        description="Esportazione verificabile e ripristino dell'archivio Ariadne; il token di scrittura non viene mai toccato.")
    sub = parser.add_subparsers(dest="command", required=True)
    p_export = sub.add_parser("export", help="esporta archivio e memorie in un file JSON verificabile")
    p_export.add_argument("--database", required=True)
    p_export.add_argument("--output", required=True)
    p_export.add_argument("--force", action="store_true", help="sovrascrive il file di output esistente")
    p_verify = sub.add_parser("verify", help="ricalcola il digest e mostra conteggi e schema")
    p_verify.add_argument("--input", required=True)
    p_restore = sub.add_parser("restore", help="ricostruisce e attiva un nuovo database dopo verifica")
    p_restore.add_argument("--input", required=True)
    p_restore.add_argument("--database", required=True)
    p_restore.add_argument("--replace", action="store_true",
                           help="richiesto se il database di destinazione esiste e contiene righe")
    return parser


def main(argv=None):
    args = build_parser().parse_args(argv)
    try:
        if args.command == "export":
            result = export(args.database, args.output, force=args.force)
            counts = result["counts"]
            print(f"esportato {counts['logs']} registri, {counts['events']} eventi, "
                  f"{counts['fixtures']} fixture, {counts['memories']} memorie → {args.output} "
                  f"(schema {SCHEMA}, digest {result['digest']})")
        elif args.command == "verify":
            result = verify(args.input)
            counts = result["counts"]
            print(f"schema: {result['schema']}; creato: {result['created_at']}; sorgente: {result['source']}")
            print(f"registri: {counts['logs']}; eventi: {counts['events']}; "
                  f"fixture: {counts['fixtures']}; memorie: {counts['memories']}")
            print(f"digest: {result['digest']}")
        else:
            result = restore(args.input, args.database, replace=args.replace)
            counts = result["counts"]
            pre = f"; copia precedente: {result['pre_restore']}" if result["pre_restore"] else ""
            print(f"ripristino completato: {counts['logs']} registri, {counts['events']} eventi, "
                  f"{counts['fixtures']} fixture, {counts['memories']} memorie{pre}")
    except BackupError as exc:
        print(f"backup: {exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
