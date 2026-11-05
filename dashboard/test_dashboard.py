import hashlib
import http.client
import json
from datetime import datetime, timezone
from pathlib import Path
import sqlite3
import subprocess
import tempfile
import threading
import unittest

from dashboard.server import ChangeFeed, Conflict, Handler, Store, ThreadingHTTPServer, load_write_token

ROOT = Path(__file__).resolve().parent.parent

NODES = lambda *keys: [{"kind": "elemento", "key": key} for key in keys]
EDGE = lambda sk, dk, rel: {"src_kind": "elemento", "src_key": sk, "dst_kind": "elemento", "dst_key": dk, "rel": rel}


class GraphToolsTests(unittest.TestCase):
    """Read-only and add_graph MCP tools against a controlled store."""
    def setUp(self):
        import ariadne_server
        self.ariadne = ariadne_server
        self.store_server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        port = self.store_server.server_address[1]
        self.path = self.mkpath = Path(tempfile.mkdtemp()) / "data.sqlite3"
        self.store_server.store = Store(self.path)
        self.store_server.feed = ChangeFeed()
        self.store_server.write_token = "a" * 32
        self.store_server.allowed_hosts = {f"127.0.0.1:{port}"}
        threading.Thread(target=self.store_server.serve_forever, daemon=True).start()
        self.addCleanup(self.store_server.server_close)
        self.addCleanup(self.store_server.shutdown)
        self._previous = (ariadne_server.STORE_URL, ariadne_server.STORE_HOST, ariadne_server.WRITE_TOKEN)
        ariadne_server.STORE_URL = f"http://127.0.0.1:{port}"
        ariadne_server.STORE_HOST = f"127.0.0.1:{port}"
        ariadne_server.WRITE_TOKEN = "a" * 32
        self.addCleanup(lambda: setattr(self.ariadne, "STORE_URL", self._previous[0]))
        self.addCleanup(lambda: setattr(self.ariadne, "STORE_HOST", self._previous[1]))
        self.addCleanup(lambda: setattr(self.ariadne, "WRITE_TOKEN", self._previous[2]))

    def seed(self, rid="demo", *extra_edges):
        edges = [EDGE("alfa", "beta", "conosce"), EDGE("gamma", "alfa", "invia")] + list(extra_edges)
        return self.ariadne.tool("ariadne_add_graph", {"id": rid, "nodes": NODES("alfa", "beta", "gamma"), "edges": edges})

    def test_add_graph_edges_cycle_self_loop_and_existing_node(self):
        first = self.seed()
        self.assertTrue(first["persisted"])
        self.assertEqual(first["version"], "2")
        self.assertEqual(first["warnings"], [])
        second = self.ariadne.tool("ariadne_add_graph", {
            "id": "demo", "nodes": [], "edges": [EDGE("beta", "alfa", "conosce"), EDGE("alfa", "alfa", "auto"), EDGE("alfa", "gamma", "risponde")]})
        self.assertGreater(second["extraction_seq"], first["extraction_seq"])
        graph = self.ariadne.tool("ariadne_saved_view", {"id": "demo", "view": "graph"})
        self.assertEqual(len(graph["nodes"]), 3)
        self.assertEqual(len(graph["edges"]), 5)
        self.assertIn("auto", {e["rel"] for e in graph["edges"]})
        aa = [e for e in graph["edges"] if e["rel"] == "auto"]
        self.assertEqual(aa[0]["src_id"], aa[0]["dst_id"])

    def test_add_graph_duplicate_node_and_edge_produce_warnings(self):
        reply = self.ariadne.tool("ariadne_add_graph", {
            "id": "duplicita", "nodes": NODES("alfa", "alfa", "beta"), "edges": [EDGE("alfa", "beta", "conosce"), EDGE("alfa", "beta", "conosce")]})
        self.assertEqual(len(reply["warnings"]), 2)
        self.assertTrue(any("nodo duplicato" in w for w in reply["warnings"]))
        self.assertTrue(any("arco duplicato" in w for w in reply["warnings"]))
        graph = self.ariadne.tool("ariadne_saved_view", {"id": "duplicita", "view": "graph"})
        self.assertEqual(len(graph["nodes"]), 2)
        self.assertEqual(len(graph["edges"]), 1)

    def test_add_graph_endpoint_and_rel_errors_name_the_values(self):
        with self.assertRaises(ValueError) as caught:
            self.ariadne.tool("ariadne_add_graph", {"id": "demo", "nodes": NODES("alfa"), "edges": [EDGE("alfa", "sconosciuto", "conosce")]})
        self.assertIn("elemento/sconosciuto", str(caught.exception))
        for bad_rel in ("Conosce", "co-nosce"):
            with self.assertRaises(ValueError) as caught:
                self.ariadne.tool("ariadne_add_graph", {"id": "demo", "nodes": NODES("alfa", "beta"), "edges": [EDGE("alfa", "beta", bad_rel)]})
            self.assertIn(f'"{bad_rel}"', str(caught.exception))

    def test_find_nodes_query_kind_limit_and_degree(self):
        self.seed()
        result = self.ariadne.tool("ariadne_find_nodes", {"id": "demo", "query": "AL"})
        self.assertTrue(result["read_only"])
        self.assertTrue(result["persisted"])
        self.assertEqual([n["key"] for n in result["nodes"]], ["alfa"])
        everything = self.ariadne.tool("ariadne_find_nodes", {"id": "demo", "query": ""})
        self.assertEqual([n["key"] for n in everything["nodes"]], ["alfa", "beta", "gamma"])
        by_key = {n["key"]: n for n in everything["nodes"]}
        self.assertEqual(by_key["alfa"]["degree"], 2)
        self.assertEqual(by_key["beta"]["degree"], 1)
        self.assertEqual(by_key["gamma"]["degree"], 1)
        self.assertTrue(all(isinstance(n["origin_seq"], int) and n["origin_seq"] > 0 for n in everything["nodes"]))
        filtered = self.ariadne.tool("ariadne_find_nodes", {"id": "demo", "kind": "artefatto"})
        self.assertEqual(filtered["total"], 0)
        self.assertEqual(filtered["nodes"], [])
        limited = self.ariadne.tool("ariadne_find_nodes", {"id": "demo", "limit": 1})
        self.assertEqual(limited["total"], 3)
        self.assertEqual(len(limited["nodes"]), 1)

    def test_node_neighbors_directions_and_missing_node(self):
        self.seed()
        out = self.ariadne.tool("ariadne_node_neighbors", {"id": "demo", "kind": "elemento", "key": "alfa", "direction": "out"})
        self.assertEqual([e["rel"] for e in out["outgoing"]], ["conosce"])
        self.assertEqual(out["outgoing"][0]["node"]["key"], "beta")
        self.assertEqual(out["incoming"], [])
        inc = self.ariadne.tool("ariadne_node_neighbors", {"id": "demo", "kind": "elemento", "key": "alfa", "direction": "in"})
        self.assertEqual([e["rel"] for e in inc["incoming"]], ["invia"])
        self.assertEqual(inc["incoming"][0]["node"]["key"], "gamma")
        both = self.ariadne.tool("ariadne_node_neighbors", {"id": "demo", "kind": "elemento", "key": "alfa"})
        self.assertEqual(len(both["outgoing"]), 1)
        self.assertEqual(len(both["incoming"]), 1)
        self.assertEqual(both["node"]["kind"], "elemento")
        self.assertGreater(both["node"]["origin_seq"], 0)
        with self.assertRaises(ValueError) as caught:
            self.ariadne.tool("ariadne_node_neighbors", {"id": "demo", "kind": "elemento", "key": "assente"})
        self.assertIn("elemento/assente", str(caught.exception))

    def test_has_edge_exists_missing_endpoints_and_rel_filter(self):
        self.seed()
        reply = self.ariadne.tool("ariadne_has_edge", {"id": "demo", "src_kind": "elemento", "src_key": "gamma", "dst_kind": "elemento", "dst_key": "alfa", "rel": "invia"})
        self.assertTrue(reply["exists"])
        self.assertEqual(len(reply["edges"]), 1)
        self.assertEqual(reply["edges"][0]["src"]["key"], "gamma")
        self.assertEqual(reply["edges"][0]["dst"]["key"], "alfa")
        self.assertGreater(reply["edges"][0]["origin_seq"], 0)
        wrong_rel = self.ariadne.tool("ariadne_has_edge", {"id": "demo", "src_kind": "elemento", "src_key": "gamma", "dst_kind": "elemento", "dst_key": "alfa", "rel": "conosce"})
        self.assertFalse(wrong_rel["exists"])
        self.assertEqual(wrong_rel["edges"], [])
        unknown = self.ariadne.tool("ariadne_has_edge", {"id": "demo", "src_kind": "elemento", "src_key": "alfa", "dst_kind": "elemento", "dst_key": "assente"})
        self.assertFalse(unknown["exists"])
        self.assertEqual(unknown["edges"], [])
        any_rel = self.ariadne.tool("ariadne_has_edge", {"id": "demo", "src_kind": "elemento", "src_key": "alfa", "dst_kind": "elemento", "dst_key": "beta"})
        self.assertTrue(any_rel["exists"])

    def test_read_tools_fail_for_missing_or_v1_register(self):
        with self.assertRaises(ValueError) as caught:
            self.ariadne.tool("ariadne_find_nodes", {"id": "inesistente"})
        self.assertIn("log non trovato", str(caught.exception))
        self.store_server.store.append("v1log", {"version": "1", "log_jsonl": self.genesis()})
        with self.assertRaises(ValueError) as caught:
            self.ariadne.tool("ariadne_has_edge", {"id": "v1log", "src_kind": "elemento", "src_key": "a", "dst_kind": "elemento", "dst_key": "b"})
        self.assertIn("v2", str(caught.exception))

    def genesis(self):
        binary = str(ROOT / "target" / "debug" / "l0_replay")
        return subprocess.check_output([binary, "--seed", "1042", "--ticks", "0"], text=True)

    def test_add_graph_survives_store_reopen(self):
        self.seed()
        reopened = Store(self.path)
        saved = reopened.get("demo")
        self.assertEqual(saved["version"], "2")
        self.assertGreater(len(saved["log_jsonl"]), 0)
        graph = self.ariadne.tool("ariadne_saved_view", {"id": "demo", "view": "graph"})
        self.assertEqual({n["key"] for n in graph["nodes"]}, {"alfa", "beta", "gamma"})
        self.assertEqual(len(graph["edges"]), 2)


class StoreTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "data.sqlite3"
        self.store = Store(self.path)
        binary = str(ROOT / "target" / "debug" / "l0_replay")
        self.genesis = subprocess.check_output([binary, "--seed", "1042", "--ticks", "0"], text=True)
        self.cycle = subprocess.check_output([binary, "--seed", "1042", "--ticks", "1"], text=True)

    def test_persistence_restart_append_only_and_conflicts(self):
        self.store.append("main", {"version": "1", "log_jsonl": self.genesis})
        reopened = Store(self.path)
        self.assertEqual(reopened.get("main")["log_jsonl"], self.genesis)
        reopened.append("main", {"version": "1", "log_jsonl": self.cycle})
        self.assertEqual(Store(self.path).get("main")["log_jsonl"], self.cycle)
        with self.assertRaises(Conflict):
            reopened.append("main", {"version": "1", "log_jsonl": self.genesis})
        with self.assertRaises(Conflict):
            reopened.append("main", {"version": "2", "log_jsonl": self.cycle})
        with self.assertRaises(ValueError):
            reopened.append("main", {"version": "1", "log_jsonl": self.cycle + "invalid\n"})
        self.assertEqual(Store(self.path).get("main")["log_jsonl"], self.cycle)
        with self.assertRaises(sqlite3.IntegrityError):
            with reopened.connect() as db:
                db.execute("DELETE FROM events")
        self.assertEqual(reopened.list()[0]["events"], 4)

    def test_fixture_binding_and_log_are_persisted_together(self):
        fixture = '{"observations":[{"agente":"a","confidenza":1000,"ordine_tom":0,"proposizione":"1","soggetto":"a","sources":[0]}]}'
        digest = hashlib.sha256(fixture.encode()).hexdigest()
        canon = lambda obj: json.dumps(obj, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        previous = json.loads(self.genesis)["hash"]
        payload = {"data": fixture, "kind": "observation", "level": "L5", "protocol_version": "2",
                   "provenance": canon({"external": {"fixture_id": "a", "sha256": digest}, "members": [], "refs": []}), "sources": "[0]"}
        preimage = "\x1f".join(["1", "0", "AUX", canon(payload), previous])
        event = {"seq": 1, "tick": 0, "type": "AUX", "payload": payload, "prev_hash": previous,
                 "hash": hashlib.sha256(preimage.encode()).hexdigest()}
        log = self.genesis + json.dumps(event, ensure_ascii=False, separators=(",", ":")) + "\n"
        binding = {"fixture_id": "a", "sha256": digest, "bytes": fixture, "anchor": 0}
        with self.assertRaises(ValueError):
            self.store.append("social", {"version": "2", "log_jsonl": log})
        self.assertEqual(self.store.list(), [])
        self.store.append("social", {"version": "2", "log_jsonl": log, "fixtures": [binding]})
        saved = Store(self.path).get("social")
        self.assertEqual(saved["log_jsonl"], log)
        self.assertEqual(saved["fixtures"], [binding])
        with self.assertRaises(Conflict):
            self.store.append("social", {"version": "2", "log_jsonl": log, "fixtures": [{**binding, "anchor": 1}]})

    def test_local_http_read_and_token_origin_protection(self):
        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        port = server.server_address[1]
        server.store = self.store
        server.feed = ChangeFeed()
        server.write_token = "a" * 32
        server.allowed_hosts = {f"127.0.0.1:{port}"}
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)

        def call(method, path, data=None, headers=None):
            conn = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
            body = json.dumps(data) if data is not None else None
            conn.request(method, path, body, headers or {})
            response = conn.getresponse()
            result = (response.status, response.read())
            conn.close()
            return result

        payload = {"version": "1", "log_jsonl": self.genesis}
        self.assertEqual(call("POST", "/api/logs/main", payload, {"Content-Type": "application/json"})[0], 403)
        headers = {"Content-Type": "application/json", "X-Ariadne-Token": "a" * 32}
        self.assertEqual(call("POST", "/api/logs/main", payload, {**headers, "Origin": "https://evil.example"})[0], 403)
        self.assertEqual(call("POST", "/api/logs/main", payload, headers)[0], 200)
        status, body = call("GET", "/api/logs")
        self.assertEqual(status, 200)
        self.assertEqual(json.loads(body)["logs"][0]["id"], "main")
        self.assertEqual(call("GET", "/api/logs/main/views/graph")[0], 200)
        status, body = call("GET", "/api/diagnostics")
        self.assertEqual(status, 200)
        diagnostics = json.loads(body)
        self.assertEqual(diagnostics["conformity"]["status"], "not_verified")
        self.assertTrue(diagnostics["archive"]["reachable"])
        self.assertEqual(diagnostics["archive"]["logs"], 1)
        self.assertIn("mcp", diagnostics)
        self.assertEqual(call("GET", "/", headers={"Host": "evil.example"})[0], 403)
        self.assertIn(b"ARIADNE", call("GET", "/")[1])

    def test_text_memories_are_append_only_and_survive_reopen(self):
        first = self.store.remember({"content": "Preferisco Rust", "source": "utente", "tags": ["progetto"]})
        second = self.store.remember({"content": "Usiamo Docker"})
        self.assertGreater(second["id"], first["id"])
        reopened = Store(self.path)
        self.assertEqual(reopened.recall("Rust")[0]["content"], "Preferisco Rust")
        self.assertEqual(len(reopened.recall("progetto")), 1)
        self.assertEqual([m["id"] for m in reopened.recall(tag="progetto")], [first["id"]])
        self.assertEqual([m["id"] for m in reopened.recall(source="Utente")], [first["id"]])
        self.assertEqual([m["id"] for m in reopened.recall(tag="assente")], [])
        with self.assertRaises(ValueError):
            reopened.remember({"content": "   "})
        with self.assertRaises(sqlite3.IntegrityError):
            with reopened.connect() as db:
                db.execute("UPDATE memories SET content='alterato'")
        with self.assertRaises(sqlite3.IntegrityError):
            with reopened.connect() as db:
                db.execute("DELETE FROM memories")
        self.assertEqual(len(reopened.recall()), 2)

    def test_memory_correction_is_append_only_and_marked(self):
        original = self.store.remember({"content": "Il colore è verde", "tags": ["colore"]})
        correction = self.store.remember({"content": "Il colore è blu", "tags": ["colore"], "supersedes": original["id"]})
        self.assertEqual(correction["supersedes"], original["id"])
        rows = {row["id"]: row for row in self.store.recall("colore")}
        self.assertEqual(rows[original["id"]]["superseded_by"], correction["id"])
        self.assertIsNone(rows[correction["id"]]["superseded_by"])
        current = self.store.recall("colore", current=True)
        self.assertEqual([row["id"] for row in current], [correction["id"]])
        self.assertEqual(len(self.store.recall("colore")), 2)
        with self.assertRaises(ValueError) as caught:
            self.store.remember({"content": "seconda rettifica", "supersedes": original["id"]})
        self.assertIn("già una rettifica", str(caught.exception))
        with self.assertRaises(ValueError) as caught:
            self.store.remember({"content": "rettifica fantasma", "supersedes": 9999})
        self.assertIn("non trovata", str(caught.exception))
        with self.assertRaises(ValueError):
            self.store.remember({"content": "x", "supersedes": 0})
        with self.assertRaises(sqlite3.IntegrityError):
            with self.store.connect() as db:
                db.execute("UPDATE memories SET supersedes=NULL WHERE content='Il colore è blu'")

    def test_memory_date_filters_use_utc_day(self):
        self.store.remember({"content": "memoria datata"})
        today = datetime.now(timezone.utc).date().isoformat()
        self.assertEqual(len(self.store.recall("datata", date_from=today, date_to=today)), 1)
        self.assertEqual(self.store.recall("datata", date_to="1970-01-01"), [])
        with self.assertRaises(ValueError):
            self.store.recall(date_from="2026-13-01")
        with self.assertRaises(ValueError):
            self.store.recall(date_from="2026-02-01", date_to="2026-01-01")

    def test_old_memory_schema_is_migrated_without_rewriting_rows(self):
        legacy = Path(self.temp.name) / "legacy.sqlite3"
        with sqlite3.connect(legacy) as db:
            db.execute("CREATE TABLE memories (id INTEGER PRIMARY KEY AUTOINCREMENT, content TEXT NOT NULL, source TEXT NOT NULL, tags TEXT NOT NULL, created_at TEXT NOT NULL)")
            db.execute("INSERT INTO memories(content,source,tags,created_at) VALUES ('vecchia','fonte','[]','2026-01-01T00:00:00+00:00')")
        migrated = Store(legacy)
        columns = {row[1] for row in migrated.connect().execute("PRAGMA table_info(memories)")}
        self.assertIn("supersedes", columns)
        rows = migrated.recall("vecchia")
        self.assertEqual(rows[0]["content"], "vecchia")
        self.assertIsNone(rows[0]["supersedes"])

    def test_diagnostics_separates_reachability_from_conformity(self):
        self.store.remember({"content": "memoria diagnostica"})
        result = self.store.diagnostics()
        self.assertTrue(result["archive"]["reachable"])
        self.assertEqual(result["archive"]["logs"], 0)
        self.assertEqual(result["archive"]["events"], 0)
        self.assertEqual(result["archive"]["database"], self.path.name)
        self.assertIn("mcp", result)
        self.assertIn("reachable", result["mcp"])
        self.assertEqual(result["conformity"]["status"], "not_verified")
        self.assertIn("note", result["conformity"])

    def test_write_token_persists_across_restarts(self):
        token = load_write_token(self.path)
        self.assertEqual(load_write_token(self.path), token)
        self.assertEqual((self.path.parent / ".write-token").stat().st_mode & 0o777, 0o600)
        with self.assertRaises(ValueError):
            load_write_token(self.path, "too-short")

    def test_change_stream_notifies_after_committed_write(self):
        server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        port = server.server_address[1]
        server.store = self.store
        server.feed = ChangeFeed()
        server.write_token = "a" * 32
        server.allowed_hosts = {f"127.0.0.1:{port}"}
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        stream = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
        self.addCleanup(stream.close)
        stream.request("GET", "/api/stream")
        response = stream.getresponse()
        self.assertEqual(response.status, 200)
        self.assertEqual(response.readline(), b"event: change\n")
        self.assertEqual(response.readline(), b"data: {}\n")
        self.assertEqual(response.readline(), b"\n")
        writer = http.client.HTTPConnection("127.0.0.1", port, timeout=5)
        writer.request("POST", "/api/memories", json.dumps({"content": "notifica"}),
                       {"Content-Type": "application/json", "X-Ariadne-Token": "a" * 32})
        result = writer.getresponse()
        self.assertEqual(result.status, 201)
        result.read(); writer.close()
        self.assertEqual(response.readline(), b"event: change\n")
        self.assertEqual(response.readline(), b"data: {}\n")
        self.assertEqual(response.readline(), b"\n")


if __name__ == "__main__":
    unittest.main()
