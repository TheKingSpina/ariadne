#!/usr/bin/env python3
"""Esegue e verifica lo stress test L0–L6, manifest v1 (5 novembre 2026).

L0–L5: log deterministico costruito da `stress_l0_l6`; qui la catena, le
fixture, le transizioni e le proiezioni sono verificate con un
implementazione Python indipendente del sigillo e del parsing canonico.
L6: eseguito separatamente da `l6_experiment`.

Il report non è una prova di verità dei fatti né di efficacia causale di
L1–L5. Lo scenario è sintetico.
"""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RUN = ROOT / "runs" / "stress_l0_l6"
REPORT = ROOT / "reports" / "stress_L0_L6_v1.md"
sys.path.insert(0, str(ROOT / "preparazione_stress"))
import aux_v2_oracle as oracle  # noqa: E402: oracolo indipendente, nessun import ariadne

AGENT_ALPHA = "b7b51403fc2c3c26314fc288bfa3b12b381deeed809870d5b78fca03c8784af3"
NODE_ALPHA = "c735556705ac4d4485024861e77e1c1b2d569542c8d5a4359f5ed96334aa8feb"
NODE_BETA = "45aba7c6bf7186d413becacb7f88e22665b4b50b25829e668df7435020664ee0"
NODE_GAMMA = "781246dd55b21c887bc8c3e7badd05c350dc931eebf5c7b98338ff5f727048dc"
NODE_DELTA = "0b742e7b07b05582b4afb020715c9fff31e0bd3e107df888b0aa2cb4a49414f1"
FIXTURES = {
    "l5-genesis-01": "c66da3195548417fcbcaea078e8bab66b98b539a68ddde6011223800c117fbac",
    "l5-beta-01": "b0a0fdb92c0dfbe7bfaf9cd56bbbd78e4cfef58378aeacc4037036a975fa77c5",
    "l5-alpha-02": "7d4d7aa8059739b6b4c09c15a565254a05b208e662f51b3ee31f81307688f099",
    "l5-perspective-01": "9a5b3c1cd79054c46f205684e034fc3c96248cf8dc58843f7fd793c76386cc79",
}


def fail(message):
    raise SystemExit(f"STRESS FAIL: {message}")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def run_binary(name):
    path = ROOT / "target" / "release" / name
    if not path.exists():
        fail(f"binario mancante: {path}")
    return subprocess.run([str(path)], cwd=ROOT, capture_output=True, text=True, timeout=120)


def load_log():
    text = (RUN / "main.jsonl").read_text(encoding="utf-8")
    if not text.endswith("\n") or "\r" in text:
        fail("JSONL senza LF finale o con CR")
    lines = text[:-1].split("\n")
    records = []
    previous = "0" * 64
    for index, line in enumerate(lines):
        record = oracle.parse_line(line)
        if record["seq"] != index:
            fail(f"seq {record['seq']} diversa dall'indice {index}")
        payload_text = oracle.canon(record["payload"])
        if oracle.format_record(record["seq"], record["tick"], record["type"],
                                payload_text, record["prev_hash"], record["hash"]) != line:
            fail(f"riga {index} non canonica o campi fuori ordine")
        expected = oracle.seal_hash(record["seq"], record["tick"], record["type"],
                                    payload_text, previous)
        if record["prev_hash"] != previous:
            fail(f"prev_hash errato a seq {index}")
        if record["hash"] != expected:
            fail(f"hash errato a seq {index}")
        previous = record["hash"]
        records.append(record)
    return records, text


def aux_data(record):
    return json.loads(record["payload"]["data"])


def main():
    result = run_binary("stress_l0_l6")
    if result.returncode != 0:
        fail(f"stress_l0_l6: {result.stderr.strip()}")
    summary = json.loads((RUN / "summary.json").read_text(encoding="utf-8"))
    records, text = load_log()

    # Fixture: byte, digest e associazione agli eventi di osservazione.
    for fixture_id, expected_digest in FIXTURES.items():
        path = RUN / f"{fixture_id}.json"
        data = path.read_bytes()
        if sha256(data) != expected_digest:
            fail(f"digest fixture {fixture_id}")
    observations = {3: "l5-genesis-01", 5: "l5-genesis-01", 6: "l5-beta-01",
                    8: "l5-alpha-02", 10: "l5-perspective-01"}
    for seq, fixture_id in observations.items():
        record = records[seq]
        payload = record["payload"]
        if payload.get("level") != "L5" or payload.get("kind") != "observation":
            fail(f"seq {seq}: attesa osservazione L5")
        raw = (RUN / f"{fixture_id}.json").read_text(encoding="utf-8")
        if payload.get("data") != raw:
            fail(f"seq {seq}: data diversa dai byte della fixture {fixture_id}")
        provenance = json.loads(payload["provenance"])
        external = provenance.get("external", {})
        if external.get("fixture_id") != fixture_id or external.get("sha256") != FIXTURES[fixture_id]:
            fail(f"seq {seq}: external della fixture non corrisponde")
        if payload.get("sources") != "[0]":
            fail(f"seq {seq}: ancora attesa [0]")

    # L5: predict e model_update.
    if records[4]["payload"].get("kind") != "predict" or aux_data(records[4]).get("previsto") != 1:
        fail("seq 4: predict alpha 1")
    if records[7]["payload"].get("kind") != "predict" or aux_data(records[7]).get("previsto") != 1:
        fail("seq 7: predict alpha 2")
    update = records[9]["payload"]
    update_data = aux_data(records[9])
    if update.get("kind") != "model_update" or update_data.get("id") != AGENT_ALPHA:
        fail("seq 9: model_update alpha mancante")
    if update_data.get("previsto") != 1 or update_data.get("osservato") != 2:
        fail("seq 9: model_update con classi inattese")
    if update_data.get("sources") != [7, 8]:
        fail("seq 9: model_update senza fonti predict/osservazione")

    # L2: cinque summary, espulsione di gamma al tick 2.
    summaries = [r for r in records if r["payload"].get("level") == "L2" and r["payload"].get("kind") == "summary"]
    if len(summaries) != 5:
        fail(f"attese 5 summary L2, trovate {len(summaries)}")
    expulsion = aux_data(summaries[1]).get("expulsions")
    if expulsion != [11] or aux_data(summaries[1]).get("text") != NODE_GAMMA:
        fail(f"espulsione attesa [11]/gamma, trovata {expulsion}")
    if aux_data(summaries[0]).get("expulsions") != []:
        fail("la prima select non deve espellere")

    # L3: transizioni attese e ordine archive -> promote al tick 256.
    l3 = [(i, r) for i, r in enumerate(records) if r["payload"].get("level") == "L3"
          and r["payload"].get("kind") in ("promote", "archive", "recall")]
    by_kind = {}
    for index, record in l3:
        by_kind.setdefault(record["payload"]["kind"], []).append((index, aux_data(record)["id"]))
    if not any(node == NODE_ALPHA for _, node in by_kind.get("promote", [])):
        fail("promote alpha mancante")
    if not any(node == NODE_BETA for _, node in by_kind.get("promote", [])):
        fail("promote beta mancante")
    if not any(node == NODE_ALPHA for _, node in by_kind.get("recall", [])):
        fail("recall alpha mancante")
    archived = by_kind.get("archive", [])
    if [node for _, node in archived] != [NODE_ALPHA, NODE_GAMMA, NODE_DELTA]:
        fail(f"ordine archive inatteso: {[node for _, node in archived]}")
    promote_beta = next(index for index, node in by_kind["promote"] if node == NODE_BETA)
    if max(index for index, _ in archived) > promote_beta:
        fail("archive deve precedere promote nello stesso giudizio")

    # Proiezioni funzionali tramite backend (non è un oracolo indipendente).
    fixtures = [{"fixture_id": fid, "sha256": digest,
                 "bytes": (RUN / f"{fid}.json").read_text(encoding="utf-8"), "anchor": 0}
                for fid, digest in FIXTURES.items()]
    for operation in ("graph", "active", "memory", "self"):
        request = {"operation": operation, "version": "2", "log_jsonl": text, "fixtures": fixtures}
        completed = subprocess.run([str(ROOT / "target" / "release" / "ariadne_mcp_backend")],
                                   input=json.dumps(request, ensure_ascii=False, sort_keys=True, separators=(",", ":")),
                                   capture_output=True, text=True, timeout=30)
        if completed.returncode != 0:
            fail(f"vista {operation}: {completed.stderr.strip()}")
        view = json.loads(completed.stdout)
        if operation == "graph" and (len(view["nodes"]) != 4 or len(view["edges"]) != 1):
            fail("grafo finale inatteso")
        if operation == "active" and NODE_ALPHA not in view["ids"]:
            fail("alpha non attivo nel documento active")
        if operation == "memory":
            states = {entry["id"]: entry["status"] for entry in view["entries"]}
            if states.get(NODE_ALPHA) != "circuito" or states.get(NODE_BETA) != "memoria_lunga":
                fail(f"stati L4 finali inattesi: {states}")

    if summary.get("events") != len(records):
        fail("conteggio eventi del summary diverso dal log")
    if summary.get("head_hash") != records[-1]["hash"]:
        fail("head_hash del summary diverso dalla testa")
    log_digest = sha256(text.encode("utf-8"))
    if summary.get("log_sha256") != log_digest:
        fail("digest del log diverso dal summary")

    # L6: esperimento separato.
    l6 = run_binary("l6_experiment")
    if l6.returncode != 0:
        fail(f"l6_experiment: {l6.stderr.strip()}")
    l6_report = (ROOT / "reports" / "l6_esperimento.md").read_text(encoding="utf-8")
    statistics = re.search(r"Statistica: W = ([0-9.]+), n = (\d+), p = ([0-9.]+), Δ̄ = ([0-9.]+)", l6_report)
    if not statistics:
        fail("statistica L6 non trovata nel report")
    verdict = re.search(r"VERDICT \(§4\): (.+)", l6_report)
    verdict_text = verdict.group(1).strip().strip('*').strip() if verdict else 'non letto'
    violations = re.search(r"Violazioni di protocollo: (\d+)", l6_report)

    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    report = f"""# Stress test L0–L6 — manifest v1

**Data:** 5 novembre 2026.
**Commit verificato:** `{commit}` (working tree può contenere modifiche non committate).
**Scenario:** `stress_l0_l6_v1`, seed 1042.
**Comando:** `python3 preparazione_stress/esegui_stress_l0_l6.py`.

## Parametri

- L0: ciclo `SENSE → ACTION → STATE`, seed 1042.
- L2: `k=3`, `tau=5` in seq.
- L3: `T=1`, `H=10` tick.
- L4: revisioni dopo ogni dichiarazione e transizione.
- L5: quattro fixture esterne, ancora GENESIS `seq=0`, ordine 0 e ordine 1.
- L6: esperimento appaiato separato, 100 coppie.

## L0–L5

| Voce | Valore |
| --- | --- |
| Eventi | {len(records)} |
| Hash di testa | `{records[-1]['hash']}` |
| SHA-256 del JSONL | `{log_digest}` |
| Fixture | {len(FIXTURES)} |
| L2 summary | 5 |
| L3 promote | alpha, beta |
| L3 archive | alpha, gamma, delta |
| L3 recall | alpha |
| Stato L4 finale | alpha `circuito`, beta `memoria_lunga` |
| Grafo finale | 4 nodi, 1 arco `alfa → beta` |

Verifiche superate: catena hash e canonicità con implementazione Python
indipendente; digest e associazione delle fixture; osservazioni L5, predict e
`model_update`; ordine `archive` prima di `promote`; espulsione di gamma;
proiezioni `graph`, `active`, `memory`, `self` ricostruite dal backend.

## L6

- Coppie: 100 (seed 1000..=1099), nessuna esclusione.
- Violazioni di protocollo: {violations.group(1) if violations else 'non lette'}.
- Statistica: W = {statistics.group(1)}, n = {statistics.group(2)}, p = {statistics.group(3)}, Δ̄ = {statistics.group(4)}.
- Verdetto previsto: **{verdict_text}**.
- Report: `reports/l6_esperimento.md`; lettura critica: `reports/L6_LETTURA.md`.

## Limiti

- Lo scenario L0–L5 è sintetico e definito da questo manifest v1, non dal
  vecchio `manifest_stress_L0_L6_bozza.md`.
- La catena e le fixture sono verificate in modo indipendente; la semantica
  L1–L4 è coperta dall'oracolo `aux_v2_small` su uno scenario distinto, non
  da un replay indipendente di questo intero log.
- L6 non dimostra l'efficacia causale di L1–L5 e non autentica fatti o
  persone.
- I dati e le reward sono generati dall'harness; il risultato vale per
  questo corpus e questo criterio.
"""
    REPORT.write_text(report, encoding="utf-8")
    print(f"stress L0–L6 ok: {len(records)} eventi, sha256 {log_digest}, L6 {statistics.group(3)}")


if __name__ == "__main__":
    main()
