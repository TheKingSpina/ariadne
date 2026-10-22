# L0 Audit Report

**modello:** opencode/nemotron-3-ultra-free

**Session ID:** ses_8ffaa936-d3ce-40ff-9ff4-640491f30a94

**HEAD audited:** a02b66a

**Date:** 2026-10-22

---

## 1. Canonicità del Parser — Completezza del criterio `s == event_line(parse(s))`

**CONFORME**

Il criterio di canonicità è completo e copre **ogni** percorso `Ok` in `from_jsonl`.

**Prova (`src/log.rs`):**
- `from_jsonl` (riga 206–226) itera su ogni riga dell'input. Per ogni riga non vuota, alla riga **220**:
  ```rust
  events.push(canonical_line_or_err(line, parse_event(line)?)?);
  ```
  Non c'è **nessun** `early return`, nessun ramo condizionale che salti il confronto, nessun valore parsato che non finisca nella riserializzazione.
- `canonical_line_or_err` (righe 254–265) ricompone l'evento con `event_line` (righe 231–243) e confronta **byte per byte** col token sorgente:
  ```rust
  let recomposed = event_line(&event);
  if recomposed.as_bytes() != line.as_bytes() {
      return Err(Error::Schema(...));
  }
  Ok(event)
  ```
- `event_line` emette i membri top-level **esattamente nell'ordine SPEC §2** (`seq, tick, type, payload, prev_hash, hash`) e usa `payload.canonical()` che, tramite `BTreeMap`, ordina le chiavi alfabeticamente.
- `parse_event` (righe 314–379) valida **tutti e sei** i membri obbligatori; il `match` finale (righe 366–378) restituisce `Err` se **qualsiasi** membro manca. Quindi un evento parziale non raggiunge mai `canonical_line_or_err`.

Di conseguenza: per ogni riga che `from_jsonl` accetta (`Ok`), vale identicamente `serialize(parse(s)) == s`. Un «accettato e non canonico» è **impossibile per costruzione**.

---

## 2. Esecuzione dei Test — Coerenza interna del report

**CONFORME**

Il report `reports/L0_n3_n4_test_report.md` (§12.1) mostra l'output **verbatim** di `cargo test` e `cargo test --release` sul worktree a `a02b66a`:

| Profilo | Binari eseguiti | Totale test | Passati | Falliti |
|---|---|---|---|---|
| debug | 10 (lib, 8 integration, 1 bin) | **80** | 80 | 0 |
| release | 10 | **80** | 80 | 0 |

Il conteggio per binario coincide (4+2+11+18+13+**10**+7+6+9 = 80; i 10 nuovi sono `l0_n3_line_canonical`). Exit code 0 in entrambi i casi.

Il **rosso su `edcca4a`** è documentato in §12.3 con output integrale: `cargo test --test l0_n3_line_canonical` → **7 passed; 3 failed**, exit **101**. I tre fallimenti corrispondono esattamente ai tre difetti dichiarati (ordine chiavi payload, ordine membri top-level, `-0` nel payload). Il primo errore è riportato verbatim alla riga 862–864 del report.

Gli output sono **output reali**, non ricostruzioni: ogni blocco `running N tests` … `test result: ok. N passed` proviene dall'esecuzione osservata.

---

## 3. Test-First sui Sei Commit

**CONFORME** — Ogni commit della catena rispetta la regola: il primo commit del task contiene **solo test**, l'implementazione tocca **solo `src/`**.

| Commit | Messaggio (troncato) | `git show --stat` — file toccati | Solo test? | Solo `src/`? |
|---|---|---|---|---|
| `88a68da` | L0 N3: test ROSSI per il parser severo | `tests/l0_n3_canonical_string.rs` (+231) | **SÌ** | — |
| `5d0a652` | L0 N3: parser severo | `src/log.rs` (+29/-5) | — | **SÌ** |
| `1b453f9` | L0 N4: test maintenance — fixture genesi | `src/event.rs` (+7/-4) | — | **SÌ** (vedi §4) |
| `fba6565` | L0 N3-fix: import EventType inutilizzato | `tests/l0_n3_canonical_string.rs` (1 riga `use`) | **SÌ** | — |
| `edcca4a` | L0 N3-bis: test ROSSI per canonicità di RIGA | `tests/l0_n3_line_canonical.rs` (+213) | **SÌ** | — |
| `a02b66a` | L0 N3-bis: canonicità di RIGA | `src/log.rs` (+36/-4) | — | **SÌ** |

Inoltre:
- `git diff --stat 88a68da 5d0a652 -- tests/` → vuoto (test immutati tra rosso e implementazione N3)
- `git diff --stat edcca4a a02b66a -- tests/` → vuoto (test immutati tra rosso e implementazione N3-bis)

---

## 4. N4 — Commit `1b453f9` su `src/event.rs`

**CONFORME**

Il diff di `1b453f9` tocca **esclusivamente** il modulo `#[cfg(test)]` (righe 124–163 di `src/event.rs`):

```diff
@@ -127,9 +127,12 @@ mod tests {
 
     #[test]
     fn compute_hash_matches_documented_preimage() {
-        // preimage indipendente, costruito dal test, contro il golden precomputato
-        // esternamente (Python hashlib) per la genesi a seed 1042.
-        let payload = r#"{"prng":"PCG64","seed":1042,"state":1042}"#;
+        // preimage indipendente, costruito dal test, contro il golden ricalcolato
+        // esternamente con `shasum -a 256` (comando e output verbatim nel
+        // messaggio del commit N4); mai ricalcolato con compute_hash o to_jsonl.
+        // Formato canonico attuale (B2): `state` è la STRINGA decimale dello
+        // stato interno PCG64, non l'intero del payload (forma pre-B2 eliminata).
+        let payload = r#"{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"}"#;
         let mut expected_pre = Vec::new();
         expected_pre.extend_from_slice(b"0");
         expected_pre.push(0x1F);
@@ -140,7 +143,7 @@ mod tests {
         expected_pre.extend_from_slice(payload.as_bytes());
         expected_pre.push(0x1F);
         expected_pre.extend_from_slice(GENESIS_PREV_HASH.as_bytes());
-        let golden = "e73ef6c4d12742137235c98bf4427abc43bc123de12614806929b5e101721e97";
+        let golden = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7";
         assert_eq!(to_hex(&Sha256::digest(&expected_pre)), golden);
         assert_eq!(
             compute_hash(0, 0, EventType::State, payload, GENESIS_PREV_HASH),
```

- **Zero righe** di implementazione (`compute_hash`, `seal`, `to_hex`, `EventType`, `Event::seal`) sono modificate.
- `git show --stat 1b453f9` elenca **solo** `src/event.rs` (7+/4−).
- Il vecchio golden `e73ef6c4…` è **assente** da `src/` e `tests/` (`grep` negativo), presente solo nei report storici.

---

## 5. Tre Test Critici — Esistenza ed Esito

Tutti e tre **ESISTONO**, sono citati con `file:riga`, e il tester li riporta **VERDI** nell'esecuzione a `a02b66a` (§12.1, binario `l0_event_log` 18/18).

| Test preteso | File:riga | Descrizione | Esito tester |
|---|---|---|---|
| Replay del prefisso storico byte-per-byte | `tests/l0_event_log.rs:316-338` (`test_replay_prefix_byte_per_byte`) | Costruisce due volte lo stesso log indipendentemente, confronta `jsonl_a == jsonl_b` byte-per-byte, poi `from_jsonl` + `replay.to_jsonl() == jsonl_a` + `verify()` | **VERDE** (§12.1) |
| Immutabilità: fallisce se un evento vecchio cambia | `tests/l0_event_log.rs:287-314` (`test_log_is_append_only_no_mutation_truncation_or_reorder`) | (1) Cambia payload vecchio → `verify().is_err()`; (2) Cambia `hash` memorizzato → `verify().is_err()`; (3) Cambia `prev_hash` memorizzato → `verify().is_err()`; (4) Log intatto resta valido | **VERDE** (§12.1) |
| Vieta evento senza `seq` di origine | `tests/l0_event_log.rs:258-266` (`test_event_without_seq_is_rejected`) | Rimuove `"seq":0,` dalla riga genesi → `is_err()`; variante `"seq":,` (membro presente senza valore) → `is_err()`; rifiuto confermato nel codice da `set_slot!` + match finale `src/log.rs:366-378` | **VERDE** (§12.1) |

---

## 6. §3 — Verdetto sulla SPEC v0.2 vigente

**VIOLATO**

La SPEC v0.2 §3 (riga 19 del file `spec/L0_SPEC_v0.2.md`) prescrive testualmente:

> Ogni errore si corregge con evento compensativo: **payload {compensates: seq, reason}**, la storia resta intatta.

**Due chiavi** prescritte: `compensates` e `reason`.

Il codice e i test implementano **tre chiavi**: `compensates`, `reason`, **`state`** (snapshot dell'ultimo STATE).

**Prove:**
- `src/log.rs:145-149` — costruzione del payload compensativo:
  ```rust
  let payload = Payload::from_pairs(vec![
      ("compensates", Value::Int(comp_int)),
      ("reason", Value::Str(reason.to_string())),
      ("state", Value::Str(snapshot)),  // <-- TERZA CHIAVE non prevista dalla SPEC v0.2
  ]);
  ```
- `tests/l0_event_log.rs:354-357` — asserzione esplicita del test `test_compensating_event_payload_and_history_intact`:
  ```rust
  assert_eq!(
      comp.payload.canonical(),
      r#"{"compensates":1,"reason":"gain errato: il valore era 6","state":"43149974864253806286336044685702662359"}"#,
      "payload compensativo: compensates seq, reason, E lo snapshot dell'ultimo STATE (opzione A)"
  );
  ```
- `tests/l0_event_log.rs:390-392` — `test_compensating_payload_carries_most_recent_state_snapshot_exact` conferma le tre chiavi.
- `tests/l0_event_log.rs:413-415` — `test_compensating_state_on_genesis_only_log_copies_genesis_snapshot` idem.

L'**addendum proposto** (`notes/L0_SPEC_v0.3_addendum_proposta.md`) prevede di aggiungere `state` al payload compensativo, ma **non è firmato** («proposta non firmata, non è legge» per le istruzioni dell'audit). Rispetto alla **legge vigente (v0.2)**, la violazione è **aperta e confermata**.

---

## Memorie Esaminate

Memorie richiamate via `memory_context(sessionId, "L0 parser canonico")` (sessione `ses_8ffaa936-d3ce-40ff-9ff4-640491f30a94`):

- `mem_851b8059-a7f0-4890-873d-31ee18021c94` — payload canonico: chiavi alfabetiche, escape standard
- `mem_6cadd392-a84e-464e-a301-317f3678f90c` — formato log: ordine membri top-level {seq,tick,type,payload,prev_hash,hash}, parser rigoroso
- `mem_4e59a7ca-ff5c-491b-98a1-808b6fbecc2c` — literal Value::Str con .to_string()
- `mem_58656f04-b0c6-402d-a362-d59232d861aa` — schema State L0, contatori i64
- `mem_3adbd62d-d5b0-4aad-b13a-1d8661222ea9` — preimage hash con separatore 0x1F
- `mem_08516980-3666-4abf-a8dc-e0f97f96eecc` — genesi L0: seq 0, STATE, prev_hash 64 zeri, payload {prng, seed, state=seed}
- `mem_97b6089c-27f0-450f-9082-bc606df15d5e` — dipendenza sha2, serializzazione JSON a mano

Le **6 memorie specifiche** citate nelle istruzioni (`mem_a4e1c04c`, `mem_e3378b2c`, `mem_e3d4ec8f`, `mem_9a97634e`, `mem_8cd8db39`, `mem_a4902419`) **non risultano nel corpus** (richiesta `memory_context` con quei termini → 0 risultati). Nessuna di esse è in conflitto col codice attuale perché non esistono.

---

## Conflitti Memoria

`memory_list_conflicts()` → **nessun conflitto** (array vuoto).

Nessun pattern di violazione preesistente su L0 in conflitto col codice a `a02b66a`.

---

## Debito Noto (B4)

Registrato come debito noto (senza riesaminarlo): il rosso processuale **B4** — la coppia di commit `08b1536`→`b2790e9` ha un test che **non ha mai compilato contro la sua implementazione**, più due test corretti dopo (`a298cf9`, `45ad98d`). Non rilevanza per l'audit corrente.

---

## Esito Finale

| Punto | Esito | Citazione chiave |
|---|---|---|
| 1. Completezza criterio canonicità | **CONFORME** | `src/log.rs:220,254-265,231-243,366-378` |
| 2. Esecuzione test / coerenza report | **CONFORME** | `reports/L0_n3_n4_test_report.md` §12.1, §12.3 |
| 3. Test-first sui 6 commit | **CONFORME** | `git show --stat` su ciascuno (vedi tabella §3) |
| 4. N4 solo `#[cfg(test)]` su `src/event.rs` | **CONFORME** | `git show 1b453f9 -- src/event.rs` (hunk dentro `mod tests`) |
| 5. Tre test critici esistono e passano | **CONFORME** | `tests/l0_event_log.rs:316-338, 287-314, 258-266` |
| 6. SPEC v0.2 §3 — payload compensativo | **VIOLATO** | `spec/L0_SPEC_v0.2.md:19` vs `src/log.rs:145-149`, `tests/l0_event_log.rs:354-357, 390-392, 413-415` |

---

**VERDETTO COMPLESSIVO: RESPINTO**

**Elenco integrale delle violazioni:**

1. **VIOLAZIONE SPEC v0.2 §3** — Il payload dell'evento compensativo contiene **tre chiavi** (`compensates`, `reason`, `state`) contro le **due prescritte** (`compensates`, `reason`). File: `src/log.rs:145-149`, `tests/l0_event_log.rs:354-357`, `tests/l0_event_log.rs:390-392`, `tests/l0_event_log.rs:413-415`. L'addendum v0.3 proposto non è legge.

---

**Nota:** L'audit **non dichiara l'accettazione**. La firma spetta all'umano.

---

**Mnemosyne:** sessione chiusa con `memory_close_session(ses_8ffaa936-d3ce-40ff-9ff4-640491f30a94)`.