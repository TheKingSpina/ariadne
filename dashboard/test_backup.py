"""Test del backup verificabile: export, digest, ripristino non distruttivo.

Richiede, come `dashboard.test_dashboard`, che siano già compilati i binari:
`cargo build --locked --bins` (l0_replay e ariadne_mcp_backend).
"""
from contextlib import closing
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

from dashboard.backup import BackupError, canonical, canonical_digest, export, restore, verify
from dashboard.server import Store

ROOT = Path(__file__).resolve().parent.parent
UTC_FORMAT = re.compile(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z\Z")


class BackupTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "ariadne.sqlite3"
        self.store = Store(self.path)
        runtime = str(ROOT / "target" / "debug" / "l0_replay")
        self.genesis = subprocess.check_output([runtime, "--seed", "1042", "--ticks", "0"], text=True)
        self.cycle = subprocess.check_output([runtime, "--seed", "1042", "--ticks", "1"], text=True)
        self.fixture, self.binding = self.make_v2()

    def make_v2(self):
        """Catena v2 con fixture L5, come prodotta da Store.append con l'oracolo."""
        fixture = '{"observations":[{"agente":"a","confidenza":1000,"ordine_tom":0,"proposizione":"1","soggetto":"a","sources":[0]}]}'
        digest = hashlib.sha256(fixture.encode()).hexdigest()
        prefix = json.loads(self.genesis)["hash"]
        payload = {"data": fixture, "kind": "observation", "level": "L5", "protocol_version": "2",
                   "provenance": canonical({"external": {"fixture_id": "a", "sha256": digest}, "members": [], "refs": []}),
                   "sources": "[0]"}
        preimage = "\x1f".join(["1", "0", "AUX", canonical(payload), prefix])
        event = {"seq": 1, "tick": 0, "type": "AUX", "payload": payload, "prev_hash": prefix,
                 "hash": hashlib.sha256(preimage.encode()).hexdigest()}
        return self.genesis + json.dumps(event, ensure_ascii=False, separators=(",", ":")) + "\n", \
            {"fixture_id": "a", "sha256": digest, "bytes": fixture, "anchor": 0}

    def seed_full(self):
        self.store.append("main", {"version": "1", "log_jsonl": self.genesis})
        self.store.append("main", {"version": "1", "log_jsonl": self.cycle})
        self.store.append("social", {"version": "2", "log_jsonl": self.fixture, "fixtures": [self.binding]})
        self.store.remember({"content": "Preferisco Rust", "source": "utente", "tags": ["progetto"]})
        self.store.remember({"content": "Usiamo Docker"})
        return Store(self.path)

    def export_to(self, name="backup.json", database=None, force=False):
        output = Path(self.temp.name) / name
        export(database or self.path, output, force=force)
        return output

    def rewrite(self, document, name):
        target = Path(self.temp.name) / name
        target.write_text(canonical(document) + "\n", encoding="utf-8")
        return target

    expected_counts = {"logs": 2, "events": 6, "fixtures": 1, "memories": 2}

    # ---------------------------------------------------------------- export

    def test_export_format_digest_canonical_zero600_and_verify(self):
        self.seed_full()
        output = self.export_to()
        document = json.loads(output.read_text(encoding="utf-8"))
        self.assertEqual(document["schema"], "ariadne-backup/1")
        self.assertTrue(UTC_FORMAT.fullmatch(document["created_at"]))
        self.assertEqual(document["source"], "ariadne.sqlite3")  # solo nome file, non percorso
        integrity = document["integrity"]
        self.assertEqual(integrity["algorithm"], "sha256")
        # digest indipendente: SHA-256 del JSON canonico di data
        recomputed = hashlib.sha256(json.dumps(document["data"], sort_keys=True,
                                              separators=(",", ":"), ensure_ascii=False).encode()).hexdigest()
        self.assertEqual(integrity["digest"], recomputed)
        self.assertEqual(output.stat().st_mode & 0o777, 0o600)
        # il conteggio dev'essere ricostruibile senza il file: dallo store atteso
        counts = verify(output)
        self.assertEqual(counts["schema"], "ariadne-backup/1")
        self.assertEqual(counts["counts"], self.expected_counts)
        # due esporti dello stesso archivio hanno lo stesso digest: riproducibilità
        again = self.export_to("backup2.json")
        self.assertEqual(json.loads(again.read_text())["integrity"]["digest"], counts["digest"])

    def test_export_includes_every_log_sorted_by_id(self):
        self.seed_full()
        self.store.append("tardi", {"version": "1", "log_jsonl": self.genesis})
        output = self.export_to()
        self.assertEqual(verify(output)["counts"]["logs"], 3)
        self.assertEqual([item["id"] for item in json.loads(output.read_text())["data"]["logs"]],
                         ["main", "social", "tardi"])

    def test_export_rejects_existing_output_without_force(self):
        self.seed_full()
        output = self.export_to()
        with self.assertRaises(BackupError) as caught:
            export(self.path, output)
        self.assertIn("--force", str(caught.exception))
        export(self.path, output, force=True)  # sovrascribile con --force
        self.assertEqual(verify(output)["counts"], self.expected_counts)

    def test_export_rejects_missing_source_and_never_writes_token(self):
        with self.assertRaises(BackupError):
            export(self.path.parent / "assente.sqlite3", self.path.parent / "backup.json")
        self.assertFalse((self.path.parent / "assente.sqlite3").exists())
        self.assertFalse((self.path.parent / "backup.json").exists())

    # ---------------------------------------------------------------- verify

    def test_tampered_payload_fails_digest(self):
        self.seed_full()
        output = self.export_to()
        document = json.loads(output.read_text())
        document["data"]["memories"][0]["content"] = "memoria alterata"
        tampered = self.rewrite(document, "tampered.json")
        with self.assertRaises(BackupError) as caught:
            verify(tampered)
        self.assertIn("digest", str(caught.exception))
        with self.assertRaises(BackupError):
            restore(tampered, self.path.parent / "restored.sqlite3")

    def test_unknown_schema_rejected(self):
        self.seed_full()
        output = self.export_to()
        for schema in ("ariadne-backup/2", "ariadne-backup"):
            document = json.loads(output.read_text())
            document["schema"] = schema
            candidate = self.rewrite(document, "schema.json")
            with self.assertRaises(BackupError) as caught:
                verify(candidate)
            self.assertIn("schema ignoto", str(caught.exception))
            with self.assertRaises(BackupError):
                restore(candidate, self.path.parent / "restored.sqlite3")

    # ---------------------------------------------------------------- restore

    def test_roundtrip_export_verify_restore_and_store_reopens(self):
        self.seed_full()
        output = self.export_to()
        result = verify(output)
        self.assertEqual(result["counts"], self.expected_counts)
        target = self.path.parent / "restored.sqlite3"
        restore(output, target)
        reopened = Store(target)
        with closing(reopened.connect()) as db:
            self.assertEqual([row[0] for row in db.execute("SELECT id FROM logs ORDER BY id")],
                             ["main", "social"])
        self.assertEqual(reopened.get("main"),
                         {"version": "1", "log_jsonl": self.cycle, "fixtures": []})
        self.assertEqual(reopened.get("social"),
                         {"version": "2", "log_jsonl": self.fixture, "fixtures": [self.binding]})
        memories = sorted(reopened.recall(""), key=lambda m: m["id"])
        self.assertEqual([m["content"] for m in memories], ["Preferisco Rust", "Usiamo Docker"])
        self.assertEqual(sorted(memories[0]["tags"]), ["progetto"])  # il tags mantiene la lista
        self.assertEqual(len(reopened.list()), 2)
        self.assertEqual([row["events"] for row in reopened.list()], [4, 2])

    def test_roundtrip_preserves_memory_corrections(self):
        self.seed_full()
        original = next(m for m in self.store.recall() if m["content"] == "Preferisco Rust")
        correction = self.store.remember({"content": "Preferisco Go", "tags": ["progetto"],
                                          "supersedes": original["id"]})
        output = self.export_to()
        target = self.path.parent / "restored-corrections.sqlite3"
        restore(output, target)
        rows = {m["id"]: m for m in Store(target).recall()}
        self.assertEqual(rows[correction["id"]]["supersedes"], original["id"])
        self.assertEqual(rows[original["id"]]["superseded_by"], correction["id"])
        self.assertEqual([m["id"] for m in Store(target).recall(current=True)], [3, 2])
        document = json.loads(output.read_text(encoding="utf-8"))
        exported = {row["id"]: row for row in document["data"]["memories"]}
        self.assertEqual(exported[correction["id"]]["supersedes"], original["id"])

    def test_restore_into_nonempty_destination_requires_replace(self):
        self.seed_full()
        output = self.export_to()
        with self.assertRaises(BackupError) as caught:
            restore(output, self.path)
        self.assertIn("--replace", str(caught.exception))
        self.assertEqual(Store(self.path).get("main")["log_jsonl"], self.cycle)
        self.assertEqual(len(Store(self.path).list()), 2)
        # con --replace lo stesso archivio viene ricostruito senza errori
        restore(output, self.path, replace=True)
        self.assertEqual(Store(self.path).get("main")["log_jsonl"], self.cycle)

    def test_replace_preserves_old_data_in_pre_restore_copy(self):
        self.seed_full()
        output = self.export_to()
        self.store.append("vecchio", {"version": "1", "log_jsonl": self.genesis})
        restore(output, self.path, replace=True)
        copies = list(self.path.parent.glob("ariadne.sqlite3.pre-restore-*"))
        self.assertEqual(len(copies), 1)
        preserved = Store(copies[0])
        self.assertEqual(preserved.get("vecchio")["log_jsonl"], self.genesis)  # dati vecchi conservati
        self.assertEqual(preserved.get("main")["log_jsonl"], self.cycle)
        restored = Store(self.path)
        self.assertEqual(restored.get("main")["log_jsonl"], self.cycle)
        with self.assertRaises(KeyError):
            restored.get("vecchio")
        # il database vecchio resta nella copia: il ripristino non ha lasciato file temporanei
        self.assertFalse(self.path.with_name("ariadne.sqlite3.restore-tmp").exists())

    def test_write_token_never_exported_never_touched(self):
        secret = "segreto-0123456789abcdef0123456789abcdef\n"
        other = "altro-0123456789abcdef0123456789abcdef\n"
        destination = self.path.parent / "restore-dir" / "restored.sqlite3"
        destination.parent.mkdir()
        source_token = self.path.parent / ".write-token"
        destination_token = destination.parent / ".write-token"
        source_token.write_text(secret, encoding="ascii")
        destination_token.write_text(other, encoding="ascii")
        self.seed_full()
        output = self.export_to()
        self.assertNotIn("segreto-0123456789abcdef", output.read_text(encoding="utf-8"))
        self.assertEqual(source_token.read_text(encoding="ascii"), secret)  # export non lo legge né lo tocca
        restore(output, destination)
        self.assertEqual(source_token.read_text(encoding="ascii"), secret)  # restore non lo modifica
        self.assertEqual(destination_token.read_text(encoding="ascii"), other)

    def test_broken_chain_aborts_restore_and_keeps_existing_database(self):
        self.store.append("salvato", {"version": "1", "log_jsonl": self.genesis})
        source = self.path.parent / "source.sqlite3"
        other = Store(source)  # export da un archivio distinto dal database di destinazione
        other.append("main", {"version": "1", "log_jsonl": self.cycle})
        output = self.export_to(database=source)
        document = json.loads(output.read_text())
        event = json.loads(document["data"]["events"][0]["line"])  # seq 0 di main
        event["hash"] = "0" * 64  # catena rotta, digest però ricalcolato
        document["data"]["events"][0]["line"] = json.dumps(event, ensure_ascii=False, separators=(",", ":")) + "\n"
        document["integrity"]["digest"] = canonical_digest(document["data"])
        tampered = self.rewrite(document, "broken.json")
        self.assertEqual(verify(tampered)["counts"],
                         {"logs": 1, "events": 4, "fixtures": 0, "memories": 0})  # digest coerente
        with self.assertRaises(BackupError) as caught:
            restore(tampered, self.path, replace=True)
        self.assertIn("verifica del registro 'main'", str(caught.exception))
        self.assertEqual(Store(self.path).get("salvato")["log_jsonl"], self.genesis)  # DB invariato
        with self.assertRaises(KeyError):
            Store(self.path).get("main")
        self.assertFalse(self.path.with_name("ariadne.sqlite3.restore-tmp").exists())
        self.assertFalse(list(self.path.parent.glob("ariadne.sqlite3.pre-restore-*")))

    # ---------------------------------------------------------------- CLI

    def test_cli_exit_codes_and_stderr_messages(self):
        self.seed_full()
        output = self.path.parent / "cli-backup.json"
        good = [sys.executable, str(ROOT / "dashboard" / "backup.py"),
                "export", "--database", str(self.path), "--output", str(output)]
        result = subprocess.run(good, capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(result.returncode, 0, result.stderr)
        result = subprocess.run([sys.executable, str(ROOT / "dashboard" / "backup.py"),
                                 "verify", "--input", str(output)], capture_output=True, text=True, cwd=ROOT)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("ariadne-backup/1", result.stdout)
        self.assertIn("digest", result.stdout)
        document = json.loads(output.read_text())
        document["integrity"]["digest"] = "0" * 64  # digest alterato
        broken = self.rewrite(document, "cli-broken.json")
        result = subprocess.run([sys.executable, str(ROOT / "dashboard" / "backup.py"),
                                 "verify", "--input", str(broken)], capture_output=True, text=True, cwd=ROOT)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertIn("digest", result.stderr)
        result = subprocess.run(good, capture_output=True, text=True, cwd=ROOT)  # --output esistente
        self.assertEqual(result.returncode, 1)
        self.assertIn("--force", result.stderr)
        self.assertEqual(result.stdout, "")


if __name__ == "__main__":
    unittest.main()
