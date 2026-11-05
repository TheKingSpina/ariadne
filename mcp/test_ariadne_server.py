"""Black-box MCP stdio and local HTTP tests. Run after cargo build --bins."""
import http.client
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parent.parent
SERVER = ROOT / "mcp" / "ariadne_server.py"
L0 = ROOT / "target" / "debug" / "l0_replay"

TOOLS_LIST = ["ariadne_verify", "ariadne_view", "ariadne_list_logs", "ariadne_read_log", "ariadne_saved_view",
              "ariadne_save_log", "ariadne_remember", "ariadne_recall", "ariadne_v2_create", "ariadne_v2_extract",
              "ariadne_add_graph", "ariadne_find_nodes", "ariadne_node_neighbors", "ariadne_has_edge"]


def request(identifier, method, params=None):
    return {"jsonrpc": "2.0", "id": identifier, "method": method, "params": params or {}}


class McpTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.log = subprocess.check_output([str(L0), "--seed", "1042", "--ticks", "1"], text=True)

    def session(self, *messages, env=None):
        payload = "".join(json.dumps(m, ensure_ascii=False) + "\n" for m in messages)
        p = subprocess.run([sys.executable, str(SERVER)], input=payload, capture_output=True, text=True, timeout=20, check=True, env=env)
        self.assertEqual(p.stderr, "")
        return [json.loads(line) for line in p.stdout.splitlines()]

    def test_initialize_list_and_verify_v1_v2(self):
        messages = [
            request(1, "initialize", {"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "test", "version": "1"}}),
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            request(2, "tools/list"),
            request(3, "tools/call", {"name": "ariadne_verify", "arguments": {"version": "1", "log_jsonl": self.log}}),
            request(4, "tools/call", {"name": "ariadne_verify", "arguments": {"version": "2", "log_jsonl": self.log}}),
            request(5, "tools/call", {"name": "ariadne_view", "arguments": {"version": "2", "log_jsonl": self.log, "view": "graph"}}),
        ]
        reply = self.session(*messages)
        self.assertEqual(len(reply), 5)
        self.assertEqual(reply[0]["result"]["protocolVersion"], "2025-03-26")
        self.assertEqual([x["name"] for x in reply[1]["result"]["tools"]], TOOLS_LIST)
        for item in reply[2:4]:
            self.assertFalse(item["result"]["isError"])
            self.assertEqual(json.loads(item["result"]["content"][0]["text"])["events"], 4)
        self.assertEqual(json.loads(reply[4]["result"]["content"][0]["text"])["nodes"], [])

    def test_invalid_log_and_unsupported_view_are_tool_errors(self):
        reply = self.session(request(1, "initialize", {"protocolVersion": "2024-11-05"}),
                             request(2, "tools/call", {"name": "ariadne_verify", "arguments": {"version": "1", "log_jsonl": self.log.replace("1042", "1043")}}),
                             request(3, "tools/call", {"name": "ariadne_view", "arguments": {"version": "1", "log_jsonl": self.log, "view": "memory"}}),
                             request(4, "tools/call", {"name": "ariadne_verify", "arguments": {"version": "2", "log_jsonl": self.log, "path": "/etc/passwd"}}))
        self.assertTrue(all(r["result"]["isError"] for r in reply[1:]))
        self.assertTrue(all(r["result"]["content"][0]["text"].startswith("ariadne_verify: ") for r in (reply[1], reply[3])))
        self.assertTrue(reply[2]["result"]["content"][0]["text"].startswith("ariadne_view: "))

    def test_transient_tools_declare_persisted_false(self):
        listing = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}), request(2, "tools/list"))
        tools = {item["name"]: item for item in listing[1]["result"]["tools"]}
        create = tools["ariadne_v2_create"]["inputSchema"]["examples"][0]
        extract = tools["ariadne_v2_extract"]["inputSchema"]["examples"][0]
        reply = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}),
                             request(2, "tools/call", {"name": "ariadne_v2_create", "arguments": create}),
                             request(3, "tools/call", {"name": "ariadne_v2_extract", "arguments": extract}))
        created = json.loads(reply[1]["result"]["content"][0]["text"])
        generated = json.loads(reply[2]["result"]["content"][0]["text"])
        self.assertFalse(created.pop("persisted"))
        self.assertFalse(generated.pop("persisted"))
        self.assertIn("warnings", generated)

    def test_protocol_errors_and_notification_do_not_pollute_stdout(self):
        reply = self.session(request(1, "tools/list"), request(2, "initialize", {"protocolVersion": "2025-03-26"}),
                             {"jsonrpc": "2.0", "method": "notifications/initialized"}, request(3, "missing"))
        self.assertEqual([r["id"] for r in reply], [1, 2, 3])
        self.assertEqual(reply[0]["error"]["code"], -32002)
        self.assertEqual(reply[2]["error"]["code"], -32601)

    def test_v2_aux_fixture_and_digest_gate(self):
        fixture = '{"observations":[{"agente":"a","confidenza":1000,"ordine_tom":0,"proposizione":"1","soggetto":"a","sources":[0]}]}'
        digest = hashlib.sha256(fixture.encode()).hexdigest()
        canon = lambda obj: json.dumps(obj, ensure_ascii=False, sort_keys=True, separators=(",", ":"))
        genesis = self.log.splitlines()[0]
        previous = json.loads(genesis)["hash"]
        payload = {
            "data": fixture, "kind": "observation", "level": "L5", "protocol_version": "2",
            "provenance": canon({"external": {"fixture_id": "fixture-a", "sha256": digest}, "members": [], "refs": []}),
            "sources": "[0]",
        }
        preimage = "\x1f".join(["1", "0", "AUX", canon(payload), previous])
        event = {"seq": 1, "tick": 0, "type": "AUX", "payload": payload, "prev_hash": previous,
                 "hash": hashlib.sha256(preimage.encode()).hexdigest()}
        log = genesis + "\n" + json.dumps(event, ensure_ascii=False, separators=(",", ":")) + "\n"
        binding = {"fixture_id": "fixture-a", "sha256": digest, "bytes": fixture, "anchor": 0}
        bad = {**binding, "bytes": fixture.replace('"proposizione":"1"', '"proposizione":"2"')}
        reply = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}),
                             request(2, "tools/call", {"name": "ariadne_verify", "arguments": {"version": "2", "log_jsonl": log, "fixtures": [binding]}}),
                             request(3, "tools/call", {"name": "ariadne_verify", "arguments": {"version": "2", "log_jsonl": log, "fixtures": [bad]}}),
                             request(4, "tools/call", {"name": "ariadne_verify", "arguments": {"version": "2", "log_jsonl": log}}))
        self.assertFalse(reply[1]["result"]["isError"])
        self.assertEqual(json.loads(reply[1]["result"]["content"][0]["text"])["events"], 2)
        self.assertTrue(reply[2]["result"]["isError"])
        self.assertTrue(reply[3]["result"]["isError"])

    def test_published_examples_are_valid_and_not_saved(self):
        listing = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}), request(2, "tools/list"))
        tools = {item["name"]: item for item in listing[1]["result"]["tools"]}
        v1, v2 = tools["ariadne_verify"]["inputSchema"]["examples"]
        graph = tools["ariadne_view"]["inputSchema"]["examples"][0]
        reply = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}),
                             request(2, "tools/call", {"name": "ariadne_verify", "arguments": v1}),
                             request(3, "tools/call", {"name": "ariadne_verify", "arguments": v2}),
                             request(4, "tools/call", {"name": "ariadne_view", "arguments": graph}))
        self.assertTrue(all(not item["result"]["isError"] for item in reply[1:]))
        self.assertEqual(json.loads(reply[1]["result"]["content"][0]["text"])["events"], 1)
        self.assertEqual(json.loads(reply[2]["result"]["content"][0]["text"])["events"], 2)
        self.assertEqual(json.loads(reply[3]["result"]["content"][0]["text"])["nodes"], [])

    def test_create_and_extract_example_build_valid_v2_graph(self):
        listing = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}), request(2, "tools/list"))
        tools = {item["name"]: item for item in listing[1]["result"]["tools"]}
        create = tools["ariadne_v2_create"]["inputSchema"]["examples"][0]
        extract = tools["ariadne_v2_extract"]["inputSchema"]["examples"][0]
        reply = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}),
                             request(2, "tools/call", {"name": "ariadne_v2_create", "arguments": create}),
                             request(3, "tools/call", {"name": "ariadne_v2_extract", "arguments": extract}))
        self.assertFalse(reply[1]["result"]["isError"])
        self.assertFalse(reply[2]["result"]["isError"])
        created = json.loads(reply[1]["result"]["content"][0]["text"])
        generated = json.loads(reply[2]["result"]["content"][0]["text"])
        self.assertEqual(created["log_jsonl"], extract["log_jsonl"])
        self.assertTrue(generated["log_jsonl"].startswith(created["log_jsonl"]))
        checked = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}),
                               request(2, "tools/call", {"name": "ariadne_view", "arguments": {"version": "2", "log_jsonl": generated["log_jsonl"], "view": "graph"}}))
        self.assertFalse(checked[1]["result"]["isError"])
        graph = json.loads(checked[1]["result"]["content"][0]["text"])
        self.assertEqual({n["key"] for n in graph["nodes"]}, {"alfa", "beta"})
        self.assertEqual(len(graph["edges"]), 1)
        self.assertEqual(graph["edges"][0]["rel"], "conosce")

    def test_extract_fails_for_undeclared_endpoint(self):
        listing = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}), request(2, "tools/list"))
        tools = {item["name"]: item for item in listing[1]["result"]["tools"]}
        extract = tools["ariadne_v2_extract"]["inputSchema"]["examples"][0]
        extract["edges"][0]["dst_key"] = "sconosciuto"
        reply = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}),
                             request(2, "tools/call", {"name": "ariadne_v2_extract", "arguments": extract}))
        self.assertTrue(reply[1]["result"]["isError"])

    def test_replay_rejection_names_first_broken_seq(self):
        configured = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}), request(2, "tools/list"))
        tools = {item["name"]: item for item in configured[1]["result"]["tools"]}
        lines = tools["ariadne_v2_extract"]["inputSchema"]["examples"][0]["log_jsonl"].splitlines()
        shuffled = "\n".join([lines[0], lines[2], lines[1]]) + "\n"
        reply = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}),
                             request(2, "tools/call", {"name": "ariadne_verify", "arguments": {"version": "2", "log_jsonl": shuffled}}),
                             request(3, "tools/call", {"name": "ariadne_verify", "arguments": {"version": "2", "log_jsonl": shuffled.replace('"k":3', '"k":4')}}))
        self.assertTrue(reply[1]["result"]["isError"])
        self.assertIn("v2 replay rejected a seq 1", reply[1]["result"]["content"][0]["text"])
        self.assertTrue(reply[2]["result"]["isError"])
        self.assertIn("v2 replay rejected a seq 1", reply[2]["result"]["content"][0]["text"])

    def test_v2_graph_can_be_saved_and_seen_from_store(self):
        from dashboard.server import ChangeFeed, Handler, Store, ThreadingHTTPServer
        with tempfile.TemporaryDirectory() as directory:
            server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
            port = server.server_address[1]
            server.store = Store(Path(directory) / "data.sqlite3")
            server.feed = ChangeFeed()
            server.write_token = "a" * 32
            server.allowed_hosts = {f"127.0.0.1:{port}"}
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                env = {**os.environ, "ARIADNE_STORE_URL": f"http://127.0.0.1:{port}", "ARIADNE_STORE_HOST": f"127.0.0.1:{port}", "ARIADNE_WRITE_TOKEN": "a" * 32}
                init = request(1, "initialize", {"protocolVersion": "2025-03-26"})
                listing = self.session(init, request(2, "tools/list"), env=env)
                tools = {item["name"]: item for item in listing[1]["result"]["tools"]}
                sample = tools["ariadne_v2_extract"]["inputSchema"]["examples"][0]
                result = self.session(init, request(2, "tools/call", {"name": "ariadne_v2_extract", "arguments": sample}), env=env)
                generated = json.loads(result[1]["result"]["content"][0]["text"])
                saved = self.session(init,
                    request(2, "tools/call", {"name": "ariadne_save_log", "arguments": {"id": "grafo", "version": "2", "log_jsonl": generated["log_jsonl"]}}),
                    request(3, "tools/call", {"name": "ariadne_saved_view", "arguments": {"id": "grafo", "view": "graph"}}), env=env)
                self.assertFalse(saved[1]["result"]["isError"])
                graph = json.loads(saved[2]["result"]["content"][0]["text"])
                self.assertEqual(len(graph["nodes"]), 2)
                self.assertEqual(len(graph["edges"]), 1)
                self.assertEqual(Store(Path(directory) / "data.sqlite3").get("grafo")["log_jsonl"], generated["log_jsonl"])
            finally:
                server.shutdown()
                server.server_close()
                thread.join()

    def test_add_graph_persists_and_extends_named_register(self):
        from dashboard.server import ChangeFeed, Handler, Store, ThreadingHTTPServer
        with tempfile.TemporaryDirectory() as directory:
            server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
            port = server.server_address[1]
            server.store = Store(Path(directory) / "data.sqlite3")
            server.feed = ChangeFeed()
            server.write_token = "a" * 32
            server.allowed_hosts = {f"127.0.0.1:{port}"}
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                env = {**os.environ, "ARIADNE_STORE_URL": f"http://127.0.0.1:{port}", "ARIADNE_STORE_HOST": f"127.0.0.1:{port}", "ARIADNE_WRITE_TOKEN": "a" * 32}
                init = request(1, "initialize", {"protocolVersion": "2025-03-26"})
                graph = {"id": "demo", "nodes": [{"kind": "elemento", "key": "alfa"}, {"kind": "elemento", "key": "beta"}],
                         "edges": [{"src_kind": "elemento", "src_key": "alfa", "dst_kind": "elemento", "dst_key": "beta", "rel": "conosce"}]}
                calls = [request(2, "tools/call", {"name": "ariadne_add_graph", "arguments": graph}),
                         request(3, "tools/call", {"name": "ariadne_add_graph", "arguments": {"id": "demo", "nodes": [{"kind": "elemento", "key": "gamma"}], "edges": []}}),
                         request(4, "tools/call", {"name": "ariadne_saved_view", "arguments": {"id": "demo", "view": "graph"}})]
                reply = self.session(init, *calls, env=env)
                self.assertTrue(all(not r["result"]["isError"] for r in reply[1:]))
                added = json.loads(reply[1]["result"]["content"][0]["text"])
                self.assertTrue(added["persisted"])
                self.assertEqual(added["warnings"], [])
                view = json.loads(reply[3]["result"]["content"][0]["text"])
                self.assertEqual({n["key"] for n in view["nodes"]}, {"alfa", "beta", "gamma"})
                self.assertEqual(len(view["edges"]), 1)
                self.assertEqual(len(Store(Path(directory) / "data.sqlite3").list()), 1)
            finally:
                server.shutdown(); server.server_close(); thread.join()

    def test_persistent_memory_and_log_tools(self):
        from dashboard.server import ChangeFeed, Handler, Store, ThreadingHTTPServer
        with tempfile.TemporaryDirectory() as directory:
            server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
            port = server.server_address[1]
            server.store = Store(Path(directory) / "data.sqlite3")
            server.feed = ChangeFeed()
            server.write_token = "a" * 32
            server.allowed_hosts = {f"127.0.0.1:{port}"}
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                env = {**os.environ, "ARIADNE_STORE_URL": f"http://127.0.0.1:{port}", "ARIADNE_STORE_HOST": f"127.0.0.1:{port}", "ARIADNE_WRITE_TOKEN": "a" * 32}
                def call(i, name, arguments):
                    return request(i, "tools/call", {"name": name, "arguments": arguments})
                reply = self.session(request(1, "initialize", {"protocolVersion": "2025-03-26"}),
                                     call(2, "ariadne_remember", {"content": "Test memoria persistente", "source": "test", "tags": ["demo"]}),
                                     call(3, "ariadne_recall", {"query": "persistente"}),
                                     call(3_1, "ariadne_recall", {"tag": "demo"}),
                                     call(3_2, "ariadne_recall", {"tag": "assente"}),
                                     call(4, "ariadne_save_log", {"id": "demo", "version": "1", "log_jsonl": self.log}),
                                     call(5, "ariadne_list_logs", {}), call(6, "ariadne_read_log", {"id": "demo"}),
                                     call(7, "ariadne_saved_view", {"id": "demo", "view": "graph"}), env=env)
                self.assertTrue(all(not r["result"]["isError"] for r in reply[1:]))
                memory = json.loads(reply[2]["result"]["content"][0]["text"])["memories"][0]
                self.assertEqual(memory["content"], "Test memoria persistente")
                self.assertEqual(json.loads(reply[3]["result"]["content"][0]["text"])["memories"][0]["id"], memory["id"])
                self.assertEqual(json.loads(reply[4]["result"]["content"][0]["text"])["memories"], [])
                self.assertEqual(json.loads(reply[6]["result"]["content"][0]["text"])["logs"][0]["id"], "demo")
                self.assertEqual(json.loads(reply[7]["result"]["content"][0]["text"])["log_jsonl"], self.log)
                self.assertEqual(json.loads(reply[8]["result"]["content"][0]["text"])["nodes"], [])
                # A fresh server instance reads the same persisted data.
                self.assertEqual(Store(Path(directory) / "data.sqlite3").recall()[0]["id"], memory["id"])
            finally:
                server.shutdown()
                server.server_close()
                thread.join()


    def test_memory_correction_and_date_filters(self):
        from datetime import datetime, timezone
        from dashboard.server import ChangeFeed, Handler, Store, ThreadingHTTPServer
        with tempfile.TemporaryDirectory() as directory:
            server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
            port = server.server_address[1]
            server.store = Store(Path(directory) / "data.sqlite3")
            server.feed = ChangeFeed()
            server.write_token = "a" * 32
            server.allowed_hosts = {f"127.0.0.1:{port}"}
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                env = {**os.environ, "ARIADNE_STORE_URL": f"http://127.0.0.1:{port}", "ARIADNE_STORE_HOST": f"127.0.0.1:{port}", "ARIADNE_WRITE_TOKEN": "a" * 32}
                init = request(1, "initialize", {"protocolVersion": "2025-03-26"})
                def call(i, name, arguments):
                    return request(i, "tools/call", {"name": name, "arguments": arguments})
                first = self.session(init, call(2, "ariadne_remember", {"content": "colore verde", "tags": ["demo"]}), env=env)
                original = json.loads(first[1]["result"]["content"][0]["text"])["id"]
                today = datetime.now(timezone.utc).date().isoformat()
                reply = self.session(init,
                    call(2, "ariadne_remember", {"content": "colore blu", "tags": ["demo"], "supersedes": original}),
                    call(3, "ariadne_recall", {"tag": "demo", "current": True}),
                    call(4, "ariadne_recall", {"query": "colore"}),
                    call(5, "ariadne_recall", {"from": today, "to": today}),
                    call(6, "ariadne_recall", {"from": "1970-01-01", "to": "1970-01-01"}), env=env)
                self.assertTrue(all(not r["result"]["isError"] for r in reply[1:]))
                correction = json.loads(reply[1]["result"]["content"][0]["text"])
                self.assertEqual(correction["supersedes"], original)
                current = json.loads(reply[2]["result"]["content"][0]["text"])["memories"]
                self.assertEqual([m["id"] for m in current], [correction["id"]])
                both = {m["id"]: m for m in json.loads(reply[3]["result"]["content"][0]["text"])["memories"]}
                self.assertEqual(both[original]["superseded_by"], correction["id"])
                self.assertEqual(len(json.loads(reply[4]["result"]["content"][0]["text"])["memories"]), 2)
                self.assertEqual(json.loads(reply[5]["result"]["content"][0]["text"])["memories"], [])
                again = self.session(init, call(2, "ariadne_remember", {"content": "terza", "supersedes": original}), env=env)
                self.assertTrue(again[1]["result"]["isError"])
                self.assertIn("già una rettifica", again[1]["result"]["content"][0]["text"])
            finally:
                server.shutdown()
                server.server_close()
                thread.join()


class HttpTransportTests(unittest.TestCase):
    """Streamable HTTP transport over the same Protocol. Test server on loopback."""
    @classmethod
    def setUpClass(cls):
        sys.path.insert(0, str(ROOT / "mcp"))
        import http_server
        cls.http_server = http_server
        cls.server = http_server.make_server("127.0.0.1", 0)
        cls.port = cls.server.server_address[1]
        cls.thread = threading.Thread(target=cls.server.serve_forever, daemon=True)
        cls.thread.start()

    @classmethod
    def tearDownClass(cls):
        cls.server.shutdown()
        cls.server.server_close()
        cls.thread.join()

    @classmethod
    def call(cls, method, path, payload=None, headers=None):
        body = None
        extra = {"Accept": "application/json"}
        if payload is not None:
            body = json.dumps(payload, ensure_ascii=False).encode()
            extra["Content-Type"] = "application/json"
            extra["Content-Length"] = str(len(body))
        request = urllib.request.Request(f"http://127.0.0.1:{cls.port}{path}", data=body,
                                         headers={**extra, **(headers or {})}, method=method)
        try:
            response = urllib.request.urlopen(request, timeout=5)
            return response.status, dict(response.headers), response.read()
        except urllib.error.HTTPError as error:
            return error.code, dict(error.headers), error.read()

    def test_initialize_returns_session_and_health(self):
        status, headers, body = self.call("POST", "/mcp", request(1, "initialize", {"protocolVersion": "2025-03-26"}))
        self.assertEqual(status, 200)
        session = headers.get("Mcp-Session-Id", "")
        self.assertTrue(session)
        reply = json.loads(body)
        self.assertEqual(reply["result"]["protocolVersion"], "2025-03-26")
        self.assertEqual(reply["result"]["serverInfo"]["version"], "0.5.0")
        self.assertEqual(reply["result"]["instructions"].count("ariadne_find_nodes"), 1)
        status, _, body = self.call("POST", "/mcp", request(2, "tools/list"), headers={"Mcp-Session-Id": session})
        self.assertEqual(status, 200)
        self.assertEqual([item["name"] for item in json.loads(body)["result"]["tools"]], TOOLS_LIST)
        status, _, body = self.call("GET", "/healthz")
        self.assertEqual(status, 200)
        health = json.loads(body)
        self.assertEqual(health["status"], "ok")
        self.assertTrue(health["initialized"])
        self.assertEqual(sorted(health["protocol_versions"]), ["2024-11-05", "2025-03-26"])
        self.assertGreaterEqual(health["sessions"], 1)
        status, _, _ = self.call("DELETE", "/mcp", headers={"Mcp-Session-Id": session})
        self.assertEqual(status, 204)
        status, _, _ = self.call("POST", "/mcp", request(2, "tools/list"), headers={"Mcp-Session-Id": session})
        self.assertEqual(status, 404)
        status, _, _ = self.call("DELETE", "/mcp", headers={"Mcp-Session-Id": session})
        self.assertEqual(status, 404)

    def test_unknown_session_is_rejected(self):
        status, _, body = self.call("POST", "/mcp", request(1, "tools/list"), headers={"Mcp-Session-Id": "noflag"})
        self.assertEqual(status, 404)
        self.assertEqual(json.loads(body)["error"], "Mcp-Session-Id sconosciuto")

    def test_valid_session_roundtrip(self):
        status, headers, _ = self.call("POST", "/mcp", request(1, "initialize", {"protocolVersion": "2025-03-26"}))
        session = headers.get("Mcp-Session-Id", "")
        status, _, body = self.call("POST", "/mcp", request(2, "ping"), headers={"Mcp-Session-Id": session})
        self.assertEqual(status, 200)
        self.assertEqual(json.loads(body)["result"], {})
        # Without the session header the same call lands on the anonymous protocol.
        status, _, _ = self.call("POST", "/mcp", request(2, "ping"))
        self.assertEqual(status, 200)
        status, _, _ = self.call("DELETE", "/mcp", headers={"Mcp-Session-Id": session})
        self.assertEqual(status, 204)

    def test_batch_and_notifications(self):
        batch = [request(1, "ping"), {"jsonrpc": "2.0", "method": "notifications/initialized"}, request(2, "ping")]
        status, _, body = self.call("POST", "/mcp", batch)
        self.assertEqual(status, 200)
        replies = json.loads(body)
        self.assertEqual([r["id"] for r in replies], [1, 2])
        status, _, body = self.call("POST", "/mcp", [{"jsonrpc": "2.0", "method": "notifications/initialized"}] * 2)
        self.assertEqual(status, 202)
        self.assertEqual(body, b"")
        status, _, body = self.call("POST", "/mcp", {"jsonrpc": "2.0", "method": "notifications/initialized"})
        self.assertEqual(status, 202)
        self.assertEqual(body, b"")

    def test_tool_errors_are_iserror_with_prefix(self):
        status, _, _ = self.call("POST", "/mcp", request(0, "initialize", {"protocolVersion": "2025-03-26"}))
        self.assertEqual(status, 200)
        status, _, body = self.call("POST", "/mcp", request(1, "tools/call", {"name": "ariadne_not_a_tool", "arguments": {}}))
        self.assertEqual(status, 200)
        reply = json.loads(body)
        self.assertTrue(reply["result"]["isError"])
        self.assertTrue(reply["result"]["content"][0]["text"].startswith("ariadne_not_a_tool: "))

    def test_transport_guards(self):
        status, _, _ = self.call("POST", "/mcp", [request(1, "ping"), {"jsonrpc": "2.0", "method": "notifications/initialized"}])
        self.assertEqual(status, 200)
        status, headers, _ = self.call("GET", "/mcp")
        self.assertEqual(status, 405)
        self.assertIn("POST", headers.get("Allow", ""))
        status, headers, _ = self.call("OPTIONS", "/mcp")
        self.assertEqual(status, 204)
        self.assertEqual(headers.get("Access-Control-Allow-Origin"), self.http_server.ALLOW_ORIGIN)
        self.assertEqual(headers.get("Access-Control-Allow-Methods"), "POST, GET, DELETE, OPTIONS")
        self.assertIn("Mcp-Session-Id", headers.get("Access-Control-Allow-Headers", ""))
        oversized = b"1" * (self.http_server.MAX_LINE + 1)
        oversize_request = urllib.request.Request(f"http://127.0.0.1:{self.port}/mcp", data=oversized,
                                         headers={"Content-Type": "application/json", "Content-Length": str(len(oversized))}, method="POST")
        try:
            response = urllib.request.urlopen(oversize_request, timeout=5)
            response.read()
            self.fail("413 expected")
        except urllib.error.HTTPError as error:
            self.assertEqual(error.code, 413)
            error.read()
        status, _, body = self.call("GET", "/mcp/altro")
        self.assertEqual(status, 404)

    def test_parse_and_invalid_request_errors(self):
        connection = http.client.HTTPConnection("127.0.0.1", self.port, timeout=5)
        connection.request("POST", "/mcp", b"not json", {"Content-Type": "application/json"})
        response = connection.getresponse(); body = response.read(); connection.close()
        self.assertEqual(response.status, 400)
        self.assertEqual(json.loads(body)["error"]["code"], -32700)
        connection = http.client.HTTPConnection("127.0.0.1", self.port, timeout=5)
        connection.request("POST", "/mcp", json.dumps({"jsonrpc": "2.0", "id": 3.14, "method": "ping"}).encode(),
                           {"Content-Type": "application/json"})
        response = connection.getresponse(); body = response.read(); connection.close()
        self.assertEqual(response.status, 200)
        self.assertEqual(json.loads(body)["error"]["code"], -32600)

    def test_content_type_must_be_json(self):
        request = urllib.request.Request(f"http://127.0.0.1:{self.port}/mcp", data=b"1", method="POST")
        try:
            response = urllib.request.urlopen(request, timeout=5)
            response.read()
            self.fail("415 expected")
        except urllib.error.HTTPError as error:
            self.assertEqual(error.code, 415)
            error.read()

    def test_sse_response_when_json_not_accepted(self):
        status, headers, body = self.call("POST", "/mcp", request(1, "initialize", {"protocolVersion": "2025-03-26"}),
                                          headers={"Accept": "text/event-stream"})
        self.assertEqual(status, 200)
        self.assertTrue(headers.get("Content-Type", "").startswith("text/event-stream"))
        self.assertTrue(body.startswith(b"event: message\ndata: "))


if __name__ == "__main__":
    unittest.main()
