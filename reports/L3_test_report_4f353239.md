# L3 Gate Test Report — HEAD `4f353239`

**Modello**: `nvidia/nvidia/nemotron-3-ultra-550b-a55b`  
**Data**: 2026-10-31  
**Mnemosyne session**: `ses_eed53867-ad7b-4855-b60f-cb7d2ec48eaf` (aperta, nessuna memoria richiamata o proposta)

---

## 1. Suite completa

### Debug build
```
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
**Totale debug**: **119 passed, 0 failed** (4+2+11+18+1+13+10+7+6+9+1+11+13+14 = 119). Include **14 test L3** in `tests/l3_memory.rs`.

### Release build
Identico output per ogni suite, **119 passed, 0 failed**.

**Esito**: ✅ **CONFORME** — Totale atteso raggiunto in entrambe le configurazioni.

---

## 2. Conformità emendamenti (citazioni file:riga)

### E1 — L3.7c: recall rifiuta `at` ≥ seq del recall → `L3Error::Recall`
**Implementazione**: `src/memory.rs:94`
```rust
if tag=="recall" && at>=e.seq {return Err(L3Error::Recall)}
```
**Test di copertura**:
- `tests/l3_memory.rs:63-75` — `l3_e1_recall_cannot_reference_future_activation`: costruisce un RECALL con `at = future_activation_seq = recall_seq + 1`; `build_memory` restituisce `L3Error::Recall`.
- `tests/l3_memory.rs:160-221` — `l3_7b_recall_without_sequela_with_valid_archive`: due sottocasi — (a) `at=1` (attivazione precedente all'archive) → `Recall`; (b) `at=unrelated_seq > archived_at` ma l'evento `at` non attiva l'identità → `Recall`. Entrambi verificano che il futuro/autoreferenziale sia `Recall`, non `Transition`.

**Promote/Archive mantengono requisito `at` nel log precedente**: invariato alla linea 93 — `tag!="recall" && !log.events().iter().take(e.seq as usize).any(|prior| prior.seq==at && (prior.payload.get("nodes").is_some() || prior.payload.get("edges").is_some()))` → `L3Error::Transition`. **Non regredito**.

**Esito**: ✅ **CONFORME**

---

### E2 — L3.3: candidati Archive al giudizio richiedono ≥1 attivazione con `seq ≤ eval seq`
**Implementazione**: `src/memory.rs:145`
```rust
let mut archives:Vec<String>=entries.iter()
    .filter(|(id,x)| x.status!=Status::Archived
        && acts.get(*id).is_some_and(|xs| xs.iter().any(|(seq,_)| *seq<=e.seq))
        && score(id,e.seq,e.tick,h,&acts)==0)
    .map(|(id,_)| id.clone()).collect();
```
Il filtro `acts.get(*id).is_some_and(|xs| xs.iter().any(|(seq,_)| *seq<=e.seq))` esclude identità senza attivazioni `≤ e.seq`.

**Test di copertura**: `tests/l3_memory.rs:41-58` — `l3_e2_future_identity_is_not_archived_before_first_activation`
- Crea identità `X` la cui **prima attivazione** è al `seq = x_activation_seq` (dopo la valutazione).
- `evaluate` non deve emettere decisioni per `X` con `at < x_activation_seq`.
- Assert: `decisions.iter().all(... if id == &x_id => *at >= x_activation_seq)`.
- Assert: esiste almeno un `Promote` per `X` con `at >= x_activation_seq` (dopo l'attivazione).

**`build_memory` (fine log) mantiene entries = identità con ≥1 attivazione nell'intero log**: la funzione `activations` (linee 62-72) raccoglie **tutte** le attivazioni del log completo; `replay` (linea 82) inserisce in `entries` ogni `id` presente in `acts` — nessuna soglia di giudizio. Semantica invariata.

**Esito**: ✅ **CONFORME**

---

### Preimage — `without_archive_hash` serializza ESATTAMENTE come `canonical` (`archived_at_seq` per primo se presente)
**Implementazione**:
- `src/memory.rs:117-121` — `without_archive_hash`:
  ```rust
  if let Some(seq)=e.archived_at_seq {
      s.push_str("\"archived_at_seq\":"); s.push_str(&seq.to_string());
      first=false
  }
  if !first { s.push(',') }
  s.push_str("\"id\":"); ...
  ```
- `src/memory.rs:123-127` — `canonical`: **stesso ordine**, `archived_at_seq` per primo, poi `id`, `origin_seq`, `status`.

**Test di copertura**: `tests/l3_memory.rs:80-89` — `l3_8_digest_matches_canonical_archived_document_preimage`
```rust
let document = memory::canonical(&view);
let prefix = format!("{{\"archive_hash\":\"{}\",", view.archive_hash);
let rest = document.strip_prefix(&prefix).expect("archive_hash è primo membro canonico");
let preimage = format!("{{{rest}");
let hex: String = Sha256::digest(preimage.as_bytes()).iter().map(|b| format!("{b:02x}")).collect();
assert_eq!(hex, view.archive_hash, "L3.8: archive_hash must hash canonical document minus its own field");
```
Il test **calcola davvero SHA-256** del preimage (non confronto autoreferenziale): estrae il documento canonico, rimuove il campo `archive_hash` iniziale, hash della stringa risultante, confronta con `view.archive_hash` memorizzato. Il test fallirebbe se l'ordine dei campi divergese.

**Esito**: ✅ **CONFORME**

---

### Soppressione re-ARCHIVE — `l3_6_archive_in_log_is_not_reemitted`
**Test**: `tests/l3_memory.rs:94-101`
```rust
let (mut log, id, _) = valid_archived_a();
let base = memory::build_memory(&log).unwrap();
nodes(&mut log, 26, &["A"]); // nuova attivazione della stessa identità
let decisions = memory::evaluate(&log, &base).unwrap();
assert!(decisions.iter().all(|d| !matches!(d, Decision3::Archive { id: archived, .. } if archived == &id)),
    "L3.6: archived identity must not be archived again before recall");
```
Verifica **asserzionalmente** che dopo un `Archive` in log, `evaluate` non emetta un secondo `Archive` per la stessa `id` prima di un `RECALL`. Il cutoff `decision_cutoff` (linea 138) include l'`archive` in log, quindi la nuova attivazione non produce decisioni d'archivio.

**Esito**: ✅ **CONFORME**

---

## 3. Regressione difetti chiusi (audit 30/10)

| Difetto audit | Test che lo sancisce | Esito |
|---------------|---------------------|-------|
| **Sequela futura** (L3.7c): `at` ≥ seq del recall accettato come valido | `l3_e1_recall_cannot_reference_future_activation` (linee 63-75) + `l3_7b_recall_without_sequela_with_valid_archive` (linee 160-221) | ✅ Chiuso — entrambi i test passano, `L3Error::Recall` emesso |
| **Preimage** (L3.8/L3.10): ordine campi `archived_at_seq` non primo → hash non corrispondente | `l3_8_digest_matches_canonical_archived_document_preimage` (linee 80-89) | ✅ Chiuso — SHA-256 calcolato su preimage con ordine corretto, assert passa |
| **Identità future archiviate prima dell'esistenza** (L3.3 E2): identità mai attivate ricevono `Archive`/`Promote` al giudizio | `l3_e2_future_identity_is_not_archived_before_first_activation` (linee 41-58) | ✅ Chiuso — identità `X` non riceve decisioni prima della sua prima attivazione |

Tutti e tre i difetti hanno test di regressione dedicati che passano.

**Esito**: ✅ **CONFORME**

---

## 4. Stabilità suite preesistente

- **115 test precedenti** (105 L0/L1/L2 + 10 L3 pre-`0f01211`) restano verdi.
- **Commit `0f01211`**: `git show --stat 0f01211` → **solo `tests/l3_memory.rs` (+82 linee)**, nessun test esistente modificato.
- **Commit `4f353239`**: `git show --stat 4f353239` → **solo `src/memory.rs` (+3/−2)**, nessun test toccato.
- **Diff `4f353239~2..4f353239 --stat`**: conferma che i due commit di lavoro toccano solo `src/memory.rs` e `tests/l3_memory.rs` (più file agenti non di codice).

**Esito**: ✅ **CONFORME**

---

## 5. Provenienza

| Commit | File toccati | Natura |
|--------|--------------|--------|
| `0f01211` | `tests/l3_memory.rs` (+82) | **Solo test** — 4 nuovi test: `l3_e2_future_identity_is_not_archived_before_first_activation`, `l3_e1_recall_cannot_reference_future_activation`, `l3_8_digest_matches_canonical_archived_document_preimage`, `l3_6_archive_in_log_is_not_reemitted` |
| `4f353239` | `src/memory.rs` (+3/−2) | **Solo fix** — tre modifiche puntuali: (1) linea 94 `at>=e.seq → Recall`, (2) linea 119 `archived_at_seq` primo in serializzazione, (3) linea 145 filtro `acts.get(...).is_some_and(... <= e.seq)` |

**Il test è la prova**: nessun test modificato nel commit di fix (`4f353239`).

**Esito**: ✅ **CONFORME**

---

## 6. Invarianze

`git diff aed10e8..4f353239 -- src/view.rs src/selection.rs src/log.rs src/event.rs src/payload.rs src/lib.rs spec/` → **nessun output** (zero modifiche).

File `notes/L3_legge_memoria.md` singolo (non due), invariato nel range.

Agenti (`0defe20`): modifiche solo a `.opencode/agents/*.md` — **fuori dalla mappa del codice**.

**Esito**: ✅ **CONFORME**

---

## 7. Giudizio finale

| Voce | Esito |
|------|-------|
| 1. Suite completa (119/119 debug & release) | ✅ CONFORME |
| 2. E1 — Recall future/autoreferenziale → `Recall` | ✅ CONFORME |
| 2. E2 — Archive solo identità con attivazione ≤ giudizio | ✅ CONFORME |
| 2. Preimage — `archived_at_seq` primo, SHA-256 verificato | ✅ CONFORME |
| 2. Soppressione re-ARCHIVE — assert no nuova decisione | ✅ CONFORME |
| 3. Regressione tre difetti audit 30/10 | ✅ CONFORME |
| 4. Stabilità 115 test preesistenti | ✅ CONFORME |
| 5. Provenienza commit (test-only / fix-only) | ✅ CONFORME |
| 6. Invarianze codice/note/agenti | ✅ CONFORME |

### **GIUDIZIO COMPLESSIVO: CONFORME** rispetto alla legge emendata `aed10e8`.

---

## Difetti residui / Domande

- **Nessun difetto residuo** rilevato.
- **Nessuna domanda** aperta.

---

*Report generato dal Tester (gate L3, terzo passaggio) su worktree pulito `/Users/spina/.local/share/opencode/worktree/f2d28e/l3-gate-test-4f353239`. Mnemosyne session chiusa.*