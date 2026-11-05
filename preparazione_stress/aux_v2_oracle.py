#!/usr/bin/env python3
"""Oracolo indipendente per il replay AUX v2 (mandato WP3, 2026-11-05).

Genera e verifica il fixture preregistrato
`preparazione_stress/fixtures/aux_v2_small/expected.jsonl` per lo scenario
`small` fissato nel mandato: seed 1042, k=1, tau=5, T=1, H=1,
nodi elemento/alfa e elemento/beta, arco alfa->beta "conosce".

Indipendenza dall'implementazione:
- nessun import da `ariadne`, nessuna chiamata al backend Rust;
- PCG64, JSON canonico, seal SHA-256 e semantica L1-L4 sono reimplementati
  qui in seconda battuta a partire dal profilo documentato
  (AUX_V2_PROFILO_IMPLEMENTATO.md), non portati dal codice di produzione;
- il driver Rust `tests/aux_v2_independent.rs` NON rigenera l'atteso:
  legge il fixture congelato e ne confronta i byte con il log costruito
  tramite la sola API pubblica `V2Log`.

Sottocomandi:
  build   rigenera/sovrascrive expected.jsonl, manifest.json,
          expected.sha256 e i fixture negativi, prima di ogni run dei test;
  verify  non riscrive nulla: ricalcola dal profilo, confronta byte con
          il fixture letto dal disco e verifica catena, digest e negativi.

Dipendenze: solo standard library (json, hashlib, argparse, pathlib).
"""

import argparse
import hashlib
import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
FIXDIR = REPO / "preparazione_stress" / "fixtures" / "aux_v2_small"
NEGDIR = FIXDIR / "negative"

GENERATOR = {
    "name": "aux_v2_oracle.py",
    "version": 1,
    "data": "2026-11-05",
    "mandato": "2026-11-05-wp3-oracolo.md",
}

# Costanti PCG64 fissate dalla SPEC L0 e dalle decisioni di governance
# documentate in src/prng.rs; trascritte come contratto, non importate.
MASK64 = (1 << 64) - 1
MASK128 = (1 << 128) - 1
MULTIPLIER = 0x2360ED051FC65DA44385DF649FCCF645
SM_GOLD = 0x9E3779B97F4A7C15
SM_M1 = 0xBF58476D1CE4E5B9
SM_M2 = 0x94D049BB133111EB
SEP = chr(0x1F)
GENESIS_PREV = "0" * 64
PROTOCOL_VERSION = "2"
DOMAIN = ("SENSE", "ACTION", "STATE", "AUX")

# Punti preregistrati dal mandato: servono da legami di controllo della
# codifica (hash genesi e id nodo), mai da fonte dell'atteso (che è la
# simulazione del profilo).
GOLD_NODE_ALFA = "c735556705ac4d4485024861e77e1c1b2d569542c8d5a4359f5ed96334aa8feb"
GOLD_NODE_BETA = "45aba7c6bf7186d413becacb7f88e22665b4b50b25829e668df7435020664ee0"
GOLD_GENESIS_HASH = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7"
GOLD_SCORE_ALFA_20 = 18_874_368
GOLD_SCORE_BETA_20 = 2_097_152

SCHEMA_ERROR = "Schema"
CHAIN_ERROR = "Chain"
MISMATCH_ERROR = "Mismatch"
INCOMPLETE_ERROR = "IncompleteBatch"

# Fixtures negativi, con errore atteso preregistrato qui.
# prefix_dopo_barrier_select.jsonl è ACCETTATO dal replay (prefisso
# open/select ancora aperto): è coperto da un caso separato non in lista.
NEGATIVES = [
    {"file": "select_ups_ricalcolata.jsonl", "atteso": MISMATCH_ERROR},
    {"file": "un_byte_alterato.jsonl", "atteso": CHAIN_ERROR},
    {"file": "crlf.jsonl", "atteso": SCHEMA_ERROR},
    {"file": "senza_lf_finale.jsonl", "atteso": SCHEMA_ERROR},
    {"file": "riga_vuota_intermedia.jsonl", "atteso": SCHEMA_ERROR},
    {"file": "protocol_version_errato.jsonl", "atteso": SCHEMA_ERROR},
    {"file": "summary_text_ricalcolata.jsonl", "atteso": MISMATCH_ERROR},
    {"file": "snapshot_self_hash_ricalcolata.jsonl", "atteso": MISMATCH_ERROR},
    {"file": "prefix_dopo_barrier_open.jsonl", "atteso": INCOMPLETE_ERROR},
    {"file": "prefix_dopo_barrier_close.jsonl", "atteso": MISMATCH_ERROR},
]

# ------------------------------------------------------------------ JSON canonico


def canon(value):
    """JSON canonico del profilo v2: chiavi ordinate per byte UTF-8,
    nessuno spazio, interi e stringhe con escape minimo, array/oggetti."""
    if isinstance(value, bool):
        raise TypeError("bool non ammesso nel profilo v2")
    if isinstance(value, str):
        return _jstring(value)
    if isinstance(value, int):
        return f"{value}"
    if isinstance(value, list):
        return "[" + ",".join(canon(v) for v in value) + "]"
    if isinstance(value, dict):
        keys = sorted(value.keys(), key=lambda k: k.encode("utf-8"))
        return "{" + ",".join(_jstring(k) + ":" + canon(value[k]) for k in keys) + "}"
    raise TypeError(f"valore non canonico: {type(value)!r}")


def _jstring(s):
    out = ['"']
    for ch in s:
        if ch == '"':
            out.append('\\"')
        elif ch == "\\":
            out.append("\\\\")
        elif ch == "\b":
            out.append("\\b")
        elif ch == "\t":
            out.append("\\t")
        elif ch == "\n":
            out.append("\\n")
        elif ch == "\f":
            out.append("\\f")
        elif ch == "\r":
            out.append("\\r")
        elif ord(ch) < 0x20:
            out.append("\\u%04x" % ord(ch))
        else:
            out.append(ch)
    return "".join(out) + '"'


def parse_line(text):
    """Parser canonico di una riga JSONL: struttura fissata
    seq,tick,type,payload,prev_hash,hash. Rifiuta bool/float/null,
    chiavi duplicate, controlli grezzi, annidamento oltre 32."""
    value, pos = _parse_value(text, 0, 0)
    if pos != len(text):
        raise SyntaxError("testo non consumato interamente")
    if not isinstance(value, dict) or len(value) != 6:
        raise SyntaxError("il record ha esattamente sei chiavi")
    for key in ("seq", "tick", "type", "payload", "prev_hash", "hash"):
        if key not in value:
            raise SyntaxError(f"chiave mancante: {key}")
    if not isinstance(value["type"], str) or value["type"] not in DOMAIN:
        raise SyntaxError("type fuori dal dominio")
    for key in ("seq", "tick"):
        n = value[key]
        if isinstance(n, bool) or not isinstance(n, int) or n < 0:
            raise SyntaxError(f"{key} non è un intero non negativo")
    for key in ("prev_hash", "hash"):
        h = value[key]
        if not isinstance(h, str) or len(h) != 64 \
                or not all(c in "0123456789abcdef" for c in h):
            raise SyntaxError(f"{key} non è esadecimale minuscolo a 64")
    return value


def _parse_value(text, pos, depth):
    if depth > 32:
        raise SyntaxError("annidamento oltre 32")
    if pos >= len(text):
        raise SyntaxError("testo esaurito")
    ch = text[pos]
    if ch == '"':
        return _parse_string(text, pos)
    if ch == "[":
        pos += 1
        out = []
        if pos < len(text) and text[pos] != "]":
            while True:
                v, pos = _parse_value(text, pos, depth + 1)
                out.append(v)
                if pos < len(text) and text[pos] == ",":
                    pos += 1
                    continue
                break
        if pos >= len(text) or text[pos] != "]":
            raise SyntaxError("array non chiuso")
        return out, pos + 1
    if ch == "{":
        pos += 1
        out = {}
        if pos < len(text) and text[pos] != "}":
            while True:
                k, pos = _parse_string(text, pos)
                if pos >= len(text) or text[pos] != ":":
                    raise SyntaxError("manca ':'")
                v, pos = _parse_value(text, pos + 1, depth + 1)
                if k in out:
                    raise SyntaxError("chiave duplicata")
                out[k] = v
                if pos < len(text) and text[pos] == ",":
                    pos += 1
                    continue
                break
        if pos >= len(text) or text[pos] != "}":
            raise SyntaxError("oggetto non chiuso")
        return out, pos + 1
    if ch == "-" or ch.isdigit():
        start = pos
        pos += 1
        while pos < len(text) and text[pos].isdigit():
            pos += 1
        return int(text[start:pos]), pos
    raise SyntaxError(f"byte non valido a {pos}: {ch!r}")


def _parse_string(text, pos):
    if pos >= len(text) or text[pos] != '"':
        raise SyntaxError("manca '\"'")
    pos += 1
    out = []
    while True:
        if pos >= len(text):
            raise SyntaxError("stringa non chiusa")
        ch = text[pos]
        if ch == '"':
            return "".join(out), pos + 1
        if ch == "\\":
            pos += 1
            if pos >= len(text):
                raise SyntaxError("escape incompleto")
            esc = text[pos]
            if esc == '"':
                out.append('"')
            elif esc == "\\":
                out.append("\\")
            elif esc == "b":
                out.append("\b")
            elif esc == "f":
                out.append("\f")
            elif esc == "n":
                out.append("\n")
            elif esc == "r":
                out.append("\r")
            elif esc == "t":
                out.append("\t")
            elif esc == "u":
                try:
                    code = int(text[pos + 1 : pos + 5], 16)
                except ValueError as err:
                    raise SyntaxError("escape \\u non valido") from err
                pos += 4
                out.append(chr(code))
            else:
                raise SyntaxError(f"escape non ammesso: \\{esc}")
            pos += 1
            continue
        if ord(ch) < 0x20:
            raise SyntaxError("carattere di controllo grezzo in stringa")
        out.append(ch)
        pos += 1


def parse_extra(text):
    value, pos = _parse_value(text, 0, 0)
    if pos != len(text):
        raise SyntaxError("testo extra dopo il valore")
    return value


def int_list(text):
    value = parse_extra(text)
    if not isinstance(value, list) or not all(
            isinstance(v, int) and not isinstance(v, bool) and v >= 0 for v in value):
        raise ValueError("lista attesa di interi non negativi")
    return value


def sha256_hex(data):
    return hashlib.sha256(data).hexdigest()


def seal_hash(seq, tick, kind, payload_text, prev):
    """Preimage congelato: seq SP tick SP tipo SP payload canonico SP prev_hash."""
    preimage = f"{seq}{SEP}{tick}{SEP}{kind}{SEP}{payload_text}{SEP}{prev}"
    return sha256_hex(preimage.encode("utf-8"))


def format_record(seq, tick, kind, payload_text, prev, h):
    """Ordine dei campi di livello superiore congelato dal profilo:
    seq,tick,type,payload,prev_hash,hash (non ordine dizionario)."""
    return ('{{"seq":{seq},"tick":{tick},"type":{t},"payload":{p},'
            '"prev_hash":{ph},"hash":{h}}}').format(
        seq=seq, tick=tick, t=_jstring(kind), p=payload_text,
        ph=_jstring(prev), h=_jstring(h))


# ------------------------------------------------------------------------- PCG64


class Pcg64:
    """PCG XSL RR 128/64 (O'Neill), seeding splitmix64 come da SPEC L0:
    initstate/initseq da due step splitmix64, increment dispari,
    stato iniziale (initstate+increment)*M+increment."""

    def __init__(self, seed):
        counter = seed & MASK64
        initstate, counter = self._splitmix(counter)
        initseq, counter = self._splitmix(counter)
        self.increment = ((initseq << 1) | 1) & MASK128
        self.state = ((initstate + self.increment) * MULTIPLIER + self.increment) & MASK128

    @staticmethod
    def _splitmix(counter):
        """Un passo splitmix64: incrementa il contatore, poi lo difonde
        (valori 0xBF58476D1CE4E5B9 e 0x94D049BB133111EB)."""
        counter = (counter + SM_GOLD) & MASK64
        z = counter
        z = ((z ^ (z >> 30)) * SM_M1) & MASK64
        z = ((z ^ (z >> 27)) * SM_M2) & MASK64
        return z ^ (z >> 31), counter

    def next_u64(self):
        self.state = (self.state * MULTIPLIER + self.increment) & MASK128
        s = self.state
        rot = s >> 122
        xsl = ((s >> 64) ^ s) & MASK64
        return ((xsl >> rot) | (xsl << (64 - rot))) & MASK64

    def state_decimal(self):
        return str(self.state)


# --------------------------------------------------------------------- profilo


def node_id(kind, key):
    return sha256_hex(canon({"key": key, "kind": kind}).encode("utf-8"))


def ordered_l3(changes, nodes):
    """Ordine deterministico delle transizioni L3: archive (0), recall (1),
    promote (2); archive con origine decrescente e id decrescente, le altre
    con origine crescente e id crescente."""
    changes = sorted(changes, key=lambda item: (
        0 if item[0] == "archive" else (1 if item[0] == "recall" else 2),
        nodes[item[1]]["origin_seq"], item[1]))
    out, i = [], 0
    while i < len(changes):
        j = i
        level = 0 if changes[j][0] == "archive" else (1 if changes[j][0] == "recall" else 2)
        while j < len(changes) and (
                0 if changes[j][0] == "archive" else (1 if changes[j][0] == "recall" else 2)) == level:
            j += 1
        group = changes[i:j]
        if level == 0:
            group.reverse()
        out.extend(group)
        i = j
    return out


class Oracle:
    """Registrazione dal solo profilo v2. Nessun import di produzione."""

    def __init__(self, seed):
        self.seed = seed
        self.rng = Pcg64(seed)
        self.tick = 0
        self.events = []
        self.nodes = {}          # id -> {kind, key, origin_seq}
        self.edge_origins = {}   # (src_id, dst_id, rel) -> origin_seq
        self.activations = {}    # id -> set di seq sorgente (dedup per identità/seq)
        self.active = set()
        self.memory = {}         # id -> "Circuit" | "Long" | ("Archived", seq)
        self.l2 = None           # (k, tau)
        self.l3 = None           # (T, H)
        self.window = None
        self.statements = []     # dati L4 osservabili, in ordine di apposizione
        self.score_probe = {}    # retroscena per il manifest (estr. 20)
        self._push_record(0, 0, "STATE",
                          {"prng": "PCG64", "seed": seed,
                           "state": self.rng.state_decimal()}, GENESIS_PREV)

    # --- eventi apporsi

    def _head(self):
        return self.events[-1]["hash"]

    def _push_record(self, seq, tick, kind, payload, prev):
        payload_text = canon(payload)
        h = seal_hash(seq, tick, kind, payload_text, prev)
        self.events.append({"seq": seq, "tick": tick, "type": kind, "payload": payload,
                            "prev_hash": prev, "hash": h})
        return self.events[-1]

    def line(self, record):
        return format_record(record["seq"], record["tick"], record["type"],
                             canon(record["payload"]), record["prev_hash"], record["hash"])

    def to_jsonl(self):
        return "".join(self.line(r) + "\n" for r in self.events)

    def _push(self, kind, tick, payload):
        seq = len(self.events)
        self._push_record(seq, tick, kind, payload, self._head())
        return seq

    def _aux(self, level, kind, data, sources, provenance):
        payload = {
            "data": canon(data),
            "kind": kind,
            "level": level,
            "protocol_version": PROTOCOL_VERSION,
            "provenance": canon(provenance),
            "sources": canon(sources),
        }
        return self._push("AUX", self.tick, payload)

    def _outside(self):
        if self.window is not None:
            raise RuntimeError("operazione dentro una finestra L2 aperta")

    # --- ciclo L0

    def cycle(self):
        self._outside()
        tick = self.tick + 1
        draw = self.rng.next_u64()
        at = self._push("SENSE", tick, {"stimolo": str(draw)})
        self._push("ACTION", tick, {"mossa": draw % 3})
        self._push("STATE", tick, {"action": tick, "sense": tick,
                                   "state": self.rng.state_decimal(), "tick": tick})
        self.tick = tick
        return at

    # --- configurazioni

    def configure(self, k, tau, threshold, half_life):
        self._outside()
        if k < 0 or tau < 1 or threshold < 0 or half_life < 1:
            raise ValueError("config fuori intervallo")
        self._aux("L2", "config", {"k": k, "tau": tau}, [],
                  {"members": [], "refs": []})
        self._aux("L3", "config", {"H": half_life, "T": threshold}, [],
                  {"members": [], "refs": []})
        self.l2 = (k, tau)
        self.l3 = (threshold, half_life)

    # --- L1

    def extract(self, source, nodes, edges):
        self._outside()
        if not 0 <= source < len(self.events) \
                or self.events[source]["type"] != "SENSE" \
                or self.events[source]["tick"] != self.tick:
            raise ValueError("sorgente non è un SENSE del ciclo corrente")
        declared, mentioned, node_data, edge_data, new_edges = [], set(), [], [], []
        for kind, key in nodes:
            id_ = node_id(kind, key)
            mentioned.add(id_)
            if id_ not in self.nodes:
                declared.append((kind, key))
                self.nodes[id_] = {"kind": kind, "key": key, "origin_seq": source}
            self.memory.setdefault(id_, "Circuit")
            node_data.append({"kind": kind, "key": key})
        for sk, sv, dk, dv, rel in edges:
            a, b = node_id(sk, sv), node_id(dk, dv)
            if a not in self.nodes or b not in self.nodes:
                raise ValueError("endpoint non dichiarati")
            mentioned.add(a)
            mentioned.add(b)
            edge_data.append({"dst_kind": dk, "dst_key": dv, "rel": rel,
                              "src_kind": sk, "src_key": sv})
            if (a, b, rel) not in self.edge_origins:
                self.edge_origins[(a, b, rel)] = source
                new_edges.append((sk, sv, dk, dv, rel))
        if not node_data and not edge_data:
            raise ValueError("estrazione vuota")
        members = [{"index": i, "sources": [source]}
                   for i in range(len(node_data) + len(edge_data))]
        at = self._aux("L1", "extract", {"edges": edge_data, "nodes": node_data},
                       [source], {"members": members, "refs": []})
        for id_ in sorted(mentioned):
            self.activations.setdefault(id_, set()).add(source)
        texts = [f'a seq {at} ho dichiarato l\'elemento {kind} "{key}"'
                 for kind, key in sorted(declared)]
        for sk, sv, dk, dv, rel in sorted(
                new_edges, key=lambda e: (e[4], node_id(e[0], e[1]), node_id(e[2], e[3]))):
            texts.append(f'a seq {at} ho dichiarato l\'arco {rel} da {sk} "{sv}" a {dk} "{dv}"')
        if texts:
            self._narrate(at, [at], "\n".join(texts), [source])
        return at

    # --- saliency L2: somma intera saturante 2^24 >> (anagrafica in seq / tau)

    def _score_l2(self, id_, at, tau):
        total = 0
        for a in sorted(self.activations[id_]):
            if a <= at:
                total += (1 << 24) >> min((at - a) // tau, 24)
        return total

    def _recorded_decisions(self, level):
        """Copia dei data.at di decisioni già apposte per level."""
        found = set()
        for record in self.events:
            if record["type"] != "AUX" or record["payload"]["level"] != level:
                continue
            if record["payload"]["kind"] in ("select", "promote", "archive", "recall"):
                found.add(parse_extra(record["payload"]["data"])["at"])
        return found

    def _open_selection_window(self, extraction):
        self._outside()
        if self.l2 is None:
            raise RuntimeError("config L2 assente")
        k, tau = self.l2
        self._latest_extraction(extraction)
        if any(at >= extraction for at in self._recorded_decisions("L2")):
            raise ValueError("una decisione L2 copre già questa estrazione")
        event = self.events[extraction]
        if event["tick"] != self.tick or event["type"] != "AUX" \
                or event["payload"]["level"] != "L1":
            raise ValueError("estrazione L1 non al tick corrente")
        sources = int_list(event["payload"]["sources"])
        source = sources[0]
        if self.events[source]["type"] != "SENSE" or self.events[source]["tick"] != self.tick:
            raise ValueError("sorgente L0 non del tick corrente")
        ranked = set(self.active)
        for id_, acts in self.activations.items():
            if source in acts:
                ranked.add(id_)
        if extraction == 20:
            self.score_probe = {i: self._score_l2(i, extraction, tau) for i in ranked}
        ranked = sorted(ranked, key=lambda i: (
            -self._score_l2(i, extraction, tau), self.nodes[i]["origin_seq"], i))
        nxt = set(ranked[:k])
        ups = sorted(nxt - self.active)
        downs = sorted(self.active - nxt)
        self._aux("L2", "barrier", {"phase": "open"}, [], {"members": [], "refs": []})
        select = self._aux("L2", "select", {"at": extraction, "downs": downs, "ups": ups},
                           sources, {"members": [], "refs": [extraction]})
        self.active = set(nxt)
        self.window = {"select": select, "ups": ups, "downs": downs, "sources": sources}

    def _close_window(self):
        win = self.window
        if win is None:
            raise RuntimeError("nessuna finestra L2 aperta")
        select, ups, downs, sources = win["select"], win["ups"], win["downs"], win["sources"]
        self._aux("L2", "barrier", {"phase": "close"}, [], {"members": [], "refs": []})
        expulsions = sorted({self.nodes[i]["origin_seq"] for i in downs})
        summary = self._aux("L2", "summary",
                            {"expulsions": expulsions, "text": "|".join(downs)},
                            sources, {"members": [], "refs": [select]})
        self.window = None
        up_orig = sorted({self.nodes[i]["origin_seq"] for i in ups})
        down_orig = sorted({self.nodes[i]["origin_seq"] for i in downs})
        up_csv = ",".join(str(v) for v in up_orig)
        down_csv = ",".join(str(v) for v in down_orig)
        cites = sorted({select} | {self.nodes[i]["origin_seq"] for i in ups + downs})
        self._narrate(select, cites,
                      f"a seq {select} ho selezionato: entrate [{up_csv}], uscite [{down_csv}]",
                      sources)
        cites = sorted(set(expulsions) | {summary})
        self._narrate(summary, cites,
                      f"a seq {summary} ho consolidato: espulsi "
                      f"[{','.join(str(v) for v in expulsions)}]", sources)

    def close_selection_window(self, extraction):
        self._open_selection_window(extraction)
        self._close_window()

    # --- L3

    def _tick_of(self, seq):
        return self.events[seq]["tick"]

    def _latest_extraction(self, extraction):
        for record in reversed(self.events):
            if record["type"] == "AUX" and record["payload"]["level"] == "L1":
                if record["seq"] != extraction:
                    raise ValueError("estrazione non è l'ultima del log")
                return
        raise ValueError("nessuna estrazione presente")

    def evaluate_memory(self, extraction):
        self._outside()
        if self.l3 is None:
            raise RuntimeError("config L3 assente")
        threshold, h = self.l3
        self._latest_extraction(extraction)
        if any(at >= extraction for at in self._recorded_decisions("L3")):
            raise ValueError("una decisione L3 copre già questa estrazione")
        event = self.events[extraction]
        if event["tick"] != self.tick or event["type"] != "AUX" \
                or event["payload"]["level"] != "L1":
            raise ValueError("estrazione L1 non al tick corrente")
        sources = int_list(event["payload"]["sources"])
        changes = []
        for id_ in sorted(self.activations):
            acts = self.activations[id_]
            score = 0
            for a in sorted(acts):
                age = (self.tick - self._tick_of(a)) // h
                if age < 25:
                    score += (1 << 24) >> age
            status = self.memory[id_]
            if isinstance(status, tuple):
                if status[0] != "Archived":
                    raise ValueError(f"stato non noto per {id_}")
                at_archived = status[1]
                if any(s > at_archived and s in acts for s in sources):
                    changes.append(("recall", id_))
            elif score == 0:
                changes.append(("archive", id_))
            elif status == "Circuit" and len(acts) > threshold:
                changes.append(("promote", id_))

        changes = ordered_l3(changes, self.nodes)
        for kind, id_ in changes:
            origin = self.nodes[id_]["origin_seq"]
            at = self._aux("L3", kind,
                           {"at": extraction, "id": id_, "origin_seq": origin},
                           sources, {"members": [], "refs": [extraction]})
            if kind == "archive":
                self.memory[id_] = ("Archived", at)
                text = f"a seq {at} ho archiviato l'identità {id_}"
                cites = [at]
            elif kind == "recall":
                self.memory[id_] = "Circuit"
                text = f"a seq {at} ho richiamato l'identità {id_} (origin_seq {origin})"
                cites = sorted([at, origin])
            else:
                self.memory[id_] = "Long"
                text = f"a seq {at} ho promosso l'identità {id_}"
                cites = [at]
            self._narrate(at, cites, text, sources)

    # --- L4

    def _narrate(self, at, cites, text, sources):
        self._outside()
        data = {"at": at, "sources": list(cites), "text": text}
        self.statements.append(data)
        narrate = self._aux("L4", "narrate", data, sources, {"members": [], "refs": [at]})
        preimage = {"head_hash": self._head(), "schema_version": PROTOCOL_VERSION,
                    "statements": list(self.statements)}
        self_hash = sha256_hex(canon(preimage).encode("utf-8"))
        self._aux("L4", "snapshot", {"at": at, "self_hash": self_hash}, sources,
                  {"members": [], "refs": [narrate]})


# ---------------------------------------------------------------- scenario small


def build_expected():
    oracle = Oracle(1042)
    schedule(oracle)
    return oracle


def schedule(oracle):
    """Operazioni pubbliche dello scenario `small`, nell'ordine del mandato."""
    oracle.configure(1, 5, 1, 1)
    # tick 1
    s1 = oracle.cycle()
    e1 = oracle.extract(s1,
                        [("elemento", "alfa"), ("elemento", "beta")],
                        [("elemento", "alfa", "elemento", "beta", "conosce")])
    oracle.close_selection_window(e1)
    # tick 2
    s2 = oracle.cycle()
    e2 = oracle.extract(s2, [("elemento", "alfa")], [])
    oracle.evaluate_memory(e2)
    oracle.close_selection_window(e2)
    # tick 3..26: 24 cicli L0
    for _ in range(24):
        oracle.cycle()
    # tick 27
    s27 = oracle.cycle()
    e27 = oracle.extract(s27, [("elemento", "beta")], [])
    oracle.evaluate_memory(e27)
    # tick 28
    s28 = oracle.cycle()
    e28 = oracle.extract(s28, [("elemento", "alfa")], [])
    oracle.evaluate_memory(e28)


TABLE = [
    {"da": 0, "a": 0, "tick": [0, 0], "operazione": "GENESIS",
     "atteso": "seed 1042, hash 4649d1f4…26b7"},
    {"da": 1, "a": 1, "tick": [0, 0], "operazione": "L2/config", "atteso": "k=1, tau=5"},
    {"da": 2, "a": 2, "tick": [0, 0], "operazione": "L3/config", "atteso": "H=1, T=1"},
    {"da": 3, "a": 5, "tick": [1, 1], "operazione": "ciclo L0"},
    {"da": 6, "a": 6, "tick": [1, 1], "operazione": "L1/extract",
     "atteso": "alfa, beta, arco; sources=[3]"},
    {"da": 7, "a": 8, "tick": [1, 1], "operazione": "L4 narrate/snapshot", "atteso": "at=6"},
    {"da": 9, "a": 9, "tick": [1, 1], "operazione": "L2 barrier open"},
    {"da": 10, "a": 10, "tick": [1, 1], "operazione": "L2/select",
     "atteso": "at=6; ups=[beta]; downs=[]"},
    {"da": 11, "a": 11, "tick": [1, 1], "operazione": "L2 barrier close"},
    {"da": 12, "a": 12, "tick": [1, 1], "operazione": "L2/summary",
     "atteso": "expulsions=[]; text=\"\""},
    {"da": 13, "a": 16, "tick": [1, 1], "operazione": "L4 select + summary"},
    {"da": 17, "a": 19, "tick": [2, 2], "operazione": "ciclo L0"},
    {"da": 20, "a": 20, "tick": [2, 2], "operazione": "L1/extract",
     "atteso": "alfa; sources=[17]; nessun L4"},
    {"da": 21, "a": 21, "tick": [2, 2], "operazione": "L3/promote",
     "atteso": "alfa, at=20, origin_seq=3"},
    {"da": 22, "a": 23, "tick": [2, 2], "operazione": "L4 promote"},
    {"da": 24, "a": 24, "tick": [2, 2], "operazione": "L2 barrier open"},
    {"da": 25, "a": 25, "tick": [2, 2], "operazione": "L2/select",
     "atteso": "at=20; ups=[alfa]; downs=[beta]"},
    {"da": 26, "a": 26, "tick": [2, 2], "operazione": "L2 barrier close"},
    {"da": 27, "a": 27, "tick": [2, 2], "operazione": "L2/summary",
     "atteso": "expulsions=[3]; text=beta"},
    {"da": 28, "a": 31, "tick": [2, 2], "operazione": "L4 select + summary"},
    {"da": 32, "a": 103, "tick": [3, 26], "operazione": "24 cicli L0", "atteso": "72 eventi"},
    {"da": 104, "a": 106, "tick": [27, 27], "operazione": "ciclo L0"},
    {"da": 107, "a": 107, "tick": [27, 27], "operazione": "L1/extract",
     "atteso": "beta; sources=[104]"},
    {"da": 108, "a": 108, "tick": [27, 27], "operazione": "L3/archive",
     "atteso": "alfa, at=107"},
    {"da": 109, "a": 110, "tick": [27, 27], "operazione": "L4 archive"},
    {"da": 111, "a": 111, "tick": [27, 27], "operazione": "L3/promote",
     "atteso": "beta, at=107"},
    {"da": 112, "a": 113, "tick": [27, 27], "operazione": "L4 promote"},
    {"da": 114, "a": 116, "tick": [28, 28], "operazione": "ciclo L0"},
    {"da": 117, "a": 117, "tick": [28, 28], "operazione": "L1/extract",
     "atteso": "alfa; sources=[114]"},
    {"da": 118, "a": 118, "tick": [28, 28], "operazione": "L3/recall",
     "atteso": "alfa, at=117"},
    {"da": 119, "a": 120, "tick": [28, 28], "operazione": "L4 recall"},
]


def assert_against_mandate(oracle):
    """Legami con la tabella preregistrata nel mandato. Una derivazione che
    li contraddice ferma il build: mai piegata all'implementazione in silenzio."""
    lines = [oracle.line(r) for r in oracle.events]
    problems = []

    def expect(cond, message):
        if not cond:
            problems.append(message)

    expect(len(oracle.events) == 121, f"totale eventi {len(oracle.events)} != 121")
    r0 = parse_line(lines[0])
    expect(r0["payload"]["prng"] == "PCG64" and r0["payload"]["seed"] == 1042,
           "payload genesi non conforme")
    expect(r0["hash"] == GOLD_GENESIS_HASH,
           f"hash genesi {r0['hash']} != preregistrato {GOLD_GENESIS_HASH}")
    expect(node_id("elemento", "alfa") == GOLD_NODE_ALFA, "node_id alfa != preregistrato")
    expect(node_id("elemento", "beta") == GOLD_NODE_BETA, "node_id beta != preregistrato")

    def level_kind(seq):
        p = parse_line(lines[seq])
        return p["type"], p["payload"].get("level"), p["payload"].get("kind")

    def data_at(seq):
        return parse_extra(parse_line(lines[seq])["payload"]["data"])

    expect(level_kind(1)[1:] == ("L2", "config"), "seq 1 non è L2/config")
    expect(data_at(1) == {"k": 1, "tau": 5}, f"L2/config a seq 1: {data_at(1)}")
    expect(level_kind(2)[1:] == ("L3", "config"), "seq 2 non è L3/config")
    expect(data_at(2) == {"H": 1, "T": 1}, f"L3/config a seq 2: {data_at(2)}")
    expect(level_kind(3)[:1] == ("SENSE",) and level_kind(5)[:1] == ("STATE",),
           "ciclo tick 1 non contiguo 3-5")
    expect(level_kind(6)[1:] == ("L1", "extract"), "seq 6 non è L1/extract")
    expect(data_at(6)["nodes"] == [{"kind": "elemento", "key": "alfa"},
                                   {"kind": "elemento", "key": "beta"}],
           f"nodi a seq 6: {data_at(6)['nodes']}")
    expect(data_at(6)["edges"] == [{"dst_kind": "elemento", "dst_key": "beta",
                                    "rel": "conosce", "src_kind": "elemento",
                                    "src_key": "alfa"}],
           f"arco a seq 6: {data_at(6).get('edges')}")
    expect(parse_line(lines[6])["payload"]["sources"] == "[3]", "sources seq 6 != [3]")
    expect(data_at(9) == {"phase": "open"}, "seq 9 non è barrier open")
    expect(level_kind(10)[1:] == ("L2", "select"), "seq 10 non è select")
    expect(data_at(10) == {"at": 6, "downs": [], "ups": [GOLD_NODE_BETA]},
           f"select tick 1: {data_at(10)}")
    expect(data_at(11) == {"phase": "close"}, "seq 11 non è barrier close")
    expect(data_at(12) == {"expulsions": [], "text": ""},
           f"summary tick 1: {data_at(12)}")
    expect(level_kind(17)[:1] == ("SENSE",), "ciclo tick 2 non a 17-19")
    expect(data_at(20)["nodes"] == [{"kind": "elemento", "key": "alfa"}],
           "seq 20 non estrae solo alfa")
    expect(parse_line(lines[20])["payload"]["sources"] == "[17]", "sources seq 20 != [17]")
    expect(level_kind(21)[1:] == ("L3", "promote"), "seq 21 non è promote")
    expect(data_at(21) == {"at": 20, "id": GOLD_NODE_ALFA, "origin_seq": 3},
           f"L3/promote a seq 21: {data_at(21)}")
    probe = oracle.score_probe
    expect(probe.get(GOLD_NODE_ALFA) == GOLD_SCORE_ALFA_20,
           f"score(alfa,20) = {probe.get(GOLD_NODE_ALFA)} != {GOLD_SCORE_ALFA_20}")
    expect(probe.get(GOLD_NODE_BETA) == GOLD_SCORE_BETA_20,
           f"score(beta,20) = {probe.get(GOLD_NODE_BETA)} != {GOLD_SCORE_BETA_20}")
    expect(data_at(25) == {"at": 20, "downs": [GOLD_NODE_BETA], "ups": [GOLD_NODE_ALFA]},
           f"select tick 2: {data_at(25)}")
    expect(data_at(27) == {"expulsions": [3], "text": GOLD_NODE_BETA},
           f"summary tick 2: {data_at(27)}")
    for seq in range(32, 104):
        expect(parse_line(lines[seq])["type"] in ("SENSE", "ACTION", "STATE"),
               f"seq {seq} non è ciclo L0")
    expect(level_kind(107)[1:] == ("L1", "extract"), "seq 107 non è extract")
    expect(data_at(107)["nodes"] == [{"kind": "elemento", "key": "beta"}],
           "seq 107 non estrae solo beta")
    expect(parse_line(lines[107])["payload"]["sources"] == "[104]", "sources seq 107 != [104]")
    expect(level_kind(108)[1:] == ("L3", "archive"), "seq 108 non è archive alfa")
    expect(data_at(108) == {"at": 107, "id": GOLD_NODE_ALFA, "origin_seq": 3},
           f"L3/archive a seq 108: {data_at(108)}")
    expect(level_kind(111)[1:] == ("L3", "promote"), "seq 111 non è promote beta")
    expect(data_at(111) == {"at": 107, "id": GOLD_NODE_BETA, "origin_seq": 3},
           f"L3/promote a seq 111: {data_at(111)}")
    expect(level_kind(114)[:1] == ("SENSE",), "ciclo tick 28 non a 114-116")
    expect(level_kind(117)[1:] == ("L1", "extract"), "seq 117 non è extract")
    expect(parse_line(lines[117])["payload"]["sources"] == "[114]", "sources seq 117 != [114]")
    expect(level_kind(118)[1:] == ("L3", "recall"), "seq 118 non è recall alfa")
    expect(data_at(118) == {"at": 117, "id": GOLD_NODE_ALFA, "origin_seq": 3},
           f"L3/recall a seq 118: {data_at(118)}")
    cursor = 0
    for row in TABLE:
        if row["da"] != cursor:
            problems.append(f"tabella: buco a seq {cursor} prima di {row}")
        cursor = row["a"] + 1
    expect(cursor == 121, f"tabella seq copre fino a {cursor - 1}, non 120")
    return problems


# --------------------------------------------------------------- fixture negativi


def recompute_chain(lines, from_seq):
    """Risigilla le righe da from_seq in poi con l'oracolo, dopo una mutazione."""
    prev = parse_line(lines[from_seq - 1])["hash"] if from_seq > 0 else GENESIS_PREV
    for seq in range(from_seq, len(lines)):
        record = parse_line(lines[seq])
        payload_text = canon(record["payload"])
        h = seal_hash(seq, record["tick"], record["type"], payload_text, prev)
        lines[seq] = format_record(seq, record["tick"], record["type"],
                                   payload_text, prev, h)
        prev = h
    return lines


def mutate_line(seq, payload_mutator):
    """Trasforma il payload canonico di una riga e risigilla la catena a valle."""
    def apply(lines):
        record = parse_line(lines[seq])
        payload_new = payload_mutator(record["payload"])
        payload_text = canon(payload_new)
        prev = parse_line(lines[seq - 1])["hash"] if seq > 0 else GENESIS_PREV
        h = seal_hash(seq, record["tick"], record["type"], payload_text, prev)
        lines[seq] = format_record(seq, record["tick"], record["type"],
                                   payload_text, prev, h)
        return recompute_chain(lines, seq + 1)
    return apply


def inner_data_mutator(transform):
    """Trasforma l'JSON interiore consiglato in payload.data (stringa canonica)."""
    def apply(payload):
        data = parse_extra(payload["data"])
        return {**payload, "data": canon(transform(data))}
    return apply


def replace_unique(text, old, new, label):
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: attesa una occorrenza, trovate {count}")
    return text.replace(old, new, 1)


def flip_one_byte(lines):
    """Alterazione di un byte in un SENSE senza risigillare: Chain."""
    line = lines[3]
    prefix = '"stimolo":"'
    start = line.index(prefix) + len(prefix)
    end = start
    while line[end] != '"':
        end += 1
    last = line[end - 1]
    new_digit = "0" if last != "0" else "1"
    lines[3] = line[:end - 1] + new_digit + line[end:]
    return lines


def write_negative_fixtures(base_text):
    NEGDIR.mkdir(parents=True, exist_ok=True)
    base_lines = base_text[:-1].split("\n")

    def older_or(newer, old):
        return newer if old != newer else "ee" * 64

    fixtures = {
        "select_ups_ricalcolata.jsonl":
            "\n".join(mutate_line(10, inner_data_mutator(
                lambda d: {**d, "ups": [GOLD_NODE_ALFA]}))(base_lines[:])) + "\n",
        "un_byte_alterato.jsonl":
            "\n".join(flip_one_byte(base_lines[:])) + "\n",
        "crlf.jsonl": base_text.replace("\n", "\r\n"),
        "senza_lf_finale.jsonl": base_text[:-1],
        "riga_vuota_intermedia.jsonl":
            "\n".join(base_lines[:3] + [""] + base_lines[3:]) + "\n",
        "protocol_version_errato.jsonl":
            "\n".join(mutate_line(6, lambda p: {**p, "protocol_version": "3"})(base_lines[:])) + "\n",
        "summary_text_ricalcolata.jsonl":
            "\n".join(mutate_line(12, inner_data_mutator(
                lambda d: {**d, "text": "x"}))(base_lines[:])) + "\n",
        "snapshot_self_hash_ricalcolata.jsonl":
            "\n".join(mutate_line(8, inner_data_mutator(
                lambda d: {**d, "self_hash": older_or("ff" * 64, d["self_hash"])}))(base_lines[:])) + "\n",
        "prefix_dopo_barrier_open.jsonl":
            "".join(l + "\n" for l in base_lines[:10]),
        "prefix_dopo_barrier_select.jsonl":
            "".join(l + "\n" for l in base_lines[:11]),
        "prefix_dopo_barrier_close.jsonl":
            "".join(l + "\n" for l in base_lines[:12]),
    }
    for name, fixture_text in fixtures.items():
        with open(NEGDIR / name, "w", encoding="utf-8", newline="") as fh:
            fh.write(fixture_text)
    return fixtures


# --------------------------------------------------------------------- manifest


def manifest_doc(fixtures_digest, oracle):
    negatives = []
    for item in NEGATIVES:
        path = NEGDIR / item["file"]
        negative = dict(item)
        negative["sha256"] = sha256_hex(path.read_bytes())
        negatives.append(negative)
    negatives.append({
        "file": "prefix_dopo_barrier_select.jsonl",
        "atteso": "accettato: prefisso open/select ancora aperto",
        "sha256": sha256_hex((NEGDIR / "prefix_dopo_barrier_select.jsonl").read_bytes()),
    })
    return {
        "generator": GENERATOR,
        "scenario": "aux_v2_small",
        "parametri": {"seed": 1042, "k": 1, "tau": 5, "T": 1, "H": 1},
        "nodi": {"alfa": GOLD_NODE_ALFA, "beta": GOLD_NODE_BETA},
        "attesi": {
            "hash_genesis": GOLD_GENESIS_HASH,
            "score_l2_tick2": {"alfa": GOLD_SCORE_ALFA_20, "beta": GOLD_SCORE_BETA_20},
            "select_tick1": {"at": 6, "ups": [GOLD_NODE_BETA], "downs": []},
            "select_tick2": {"at": 20, "ups": [GOLD_NODE_ALFA], "downs": [GOLD_NODE_BETA]},
            "summary_tick2": {"expulsions": [3], "text": GOLD_NODE_BETA},
            "l3_tick27": {"archive": {"id": GOLD_NODE_ALFA, "at": 107, "origin_seq": 3},
                          "promote": {"id": GOLD_NODE_BETA, "at": 107, "origin_seq": 3}},
            "l3_tick28": {"recall": {"id": GOLD_NODE_ALFA, "at": 117, "origin_seq": 3}},
        },
        "tabella_seq": TABLE,
        "fixture": {
            "percorso": "preparazione_stress/fixtures/aux_v2_small/expected.jsonl",
            "sha256": fixtures_digest,
            "eventi": len(oracle.events),
            "byte": len(oracle.to_jsonl().encode("utf-8")),
        },
        "negativi": negatives,
        "note": [
            "expected.jsonl è congelato a build; tests/aux_v2_independent.rs non lo",
            "rigenera e fallisce se il file manca o il digest non coincide.",
            "prefix_dopo_barrier_select.jsonl è accettato dal replay come prefisso",
            "ancora aperto ed è coperto dal test Rust a parte la lista NEGATIVES.",
        ],
    }


# --------------------------------------------------------------------- verifica


def verify_lines(text):
    """Verifica strutturale indipendente delle righe: canonicità, catena, seq."""
    problems = []
    if not text.endswith("\n"):
        problems.append("manca il LF finale")
    if "\r" in text:
        problems.append("trovato CR nel testo")
    body = text[:-1].split("\n")
    prev = GENESIS_PREV
    for seq, line_text in enumerate(body):
        if line_text == "":
            problems.append(f"seq {seq}: riga vuota")
            continue
        try:
            record = parse_line(line_text)
        except SyntaxError as err:
            problems.append(f"seq {seq}: schema non valido ({err})")
            continue
        if record["seq"] != seq:
            problems.append(f"seq {seq}: campo seq {record['seq']}")
        if record["prev_hash"] != prev:
            problems.append(f"seq {seq}: prev_hash non allineato")
        payload_text = canon(record["payload"])
        rebuilt = format_record(seq, record["tick"], record["type"], payload_text,
                                record["prev_hash"], record["hash"])
        if rebuilt != line_text:
            problems.append(f"seq {seq}: riga non canonica")
        expected_hash = seal_hash(seq, record["tick"], record["type"], payload_text,
                                  record["prev_hash"])
        if expected_hash != record["hash"]:
            problems.append(f"seq {seq}: hash non coincide")
        prev = record["hash"]
    return problems


def first_diff(a, b):
    for seq, (x, y) in enumerate(zip(a.split("\n"), b.split("\n"))):
        if x != y:
            return f"seq {seq}: attesa «{x[:200]}» ottenuta «{y[:200]}»"
    if len(a) != len(b):
        return f"lunghezze {len(a)} vs {len(b)} byte"
    return "irnoto"


def cmd_build(_args):
    FIXDIR.mkdir(parents=True, exist_ok=True)
    oracle = build_expected()
    problems = assert_against_mandate(oracle) + verify_lines(oracle.to_jsonl())
    if problems:
        for p in problems:
            print(f"ERRORE di derivazione: {p}", file=sys.stderr)
        print("build interrotto: il fixture congerebbe una derivazione difforme", file=sys.stderr)
        return 2
    text = oracle.to_jsonl()
    with open(FIXDIR / "expected.jsonl", "w", encoding="utf-8", newline="") as fh:
        fh.write(text)
    digest = sha256_hex(text.encode("utf-8"))
    with open(FIXDIR / "expected.sha256", "w", encoding="utf-8", newline="") as fh:
        fh.write(digest + "\n")
    write_negative_fixtures(text)
    manifest = manifest_doc(digest, oracle)
    with open(FIXDIR / "manifest.json", "w", encoding="utf-8", newline="") as fh:
        fh.write(json.dumps(manifest, indent=2, ensure_ascii=False, sort_keys=True) + "\n")
    print(f"expected.jsonl: {len(oracle.events)} eventi, {len(text)} byte, sha256 {digest}")
    print(f"negativi: {len(NEGATIVES) + 1} fixture in {NEGDIR}")
    return 0


def cmd_verify(_args):
    problems = []
    expected_path = FIXDIR / "expected.jsonl"
    if not expected_path.exists():
        print("ERRORE: expected.jsonl mancante: il test Rust deve fallire", file=sys.stderr)
        return 2
    text = expected_path.read_text(encoding="utf-8")
    digest = sha256_hex(text.encode("utf-8"))
    sha_file = FIXDIR / "expected.sha256"
    if not sha_file.exists():
        problems.append("expected.sha256 mancante")
    elif sha_file.read_text(encoding="utf-8").strip() != digest:
        problems.append("expected.sha256 non coincide con il digest reale")
    manifest_path = FIXDIR / "manifest.json"
    if manifest_path.exists():
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        if manifest.get("fixture", {}).get("sha256") != digest:
            problems.append("manifest.fixture.sha256 non coincide con il digest reale")
        params = manifest.get("parametri", {})
        wanted = {"seed": 1042, "k": 1, "tau": 5, "T": 1, "H": 1}
        if params != wanted:
            problems.append(f"parametri nel manifest {params} != {wanted}")
        if manifest.get("tabella_seq") != TABLE:
            problems.append("tabella_seq nel manifest diverge da quella preregistrata")
        for negative in manifest.get("negativi", []):
            path = NEGDIR / negative["file"]
            if not path.exists():
                problems.append(f"negativo mancante: {negative['file']}")
            elif sha256_hex(path.read_bytes()) != negative["sha256"]:
                problems.append(f"digest del negativo {negative['file']} divergente")
    else:
        problems.append("manifest.json mancante")
    problems.extend(verify_lines(text))
    oracle = build_expected()
    problems.extend(assert_against_mandate(oracle))
    if oracle.to_jsonl() != text:
        problems.append("rigenerazione dal profilo diverge dal fixture: "
                        + first_diff(oracle.to_jsonl(), text))
    if problems:
        for p in problems:
            print(f"ERRORE: {p}", file=sys.stderr)
        return 1
    print(f"verifica ok: {len(oracle.events)} eventi, sha256 {digest}, "
          "catena, semantica e negativi riconfermati in modo indipendente")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Oracolo indipendente AUX v2 (WP3)")
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("build", help="genera/sovrascrive fixture, manifest e negativi")
    sub.add_parser("verify", help="verifica catena, digest, semantica e negativi")
    args = parser.parse_args()
    if args.command == "build":
        return cmd_build(args)
    return cmd_verify(args)


if __name__ == "__main__":
    sys.exit(main())
