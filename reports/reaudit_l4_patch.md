# Re-audit L4 — patch «validazione l2/l3 uniforme nel replay build_self»

- **modello**: `opencode/glm-5.3-flash-free` — classe `light`, variante: nessuna (default del catalogo), canale free «Free-first», 31/10/2026. Policy di routing **v1 ATTIVA** (mandato del Router).
- **sessione mnemosyne**: `ses_dd8f8e00-646d-477c-8940-42b4e5f82d4e` (progetto `opencode:Ariadne`, corpusRevision `efda4e4f-fbc4-48b3-abd5-0fb9ee3a5dc4:79`).
- **memorie esaminate**: `mem_ffe6a92e-6bb0-4d6c-9d43-5b344e3053bf` (legge L4 in vigore), `mem_df416075-11a9-4cbd-bb65-917e3b72ec32`, `mem_1fc3a41f-8618-4257-b486-0319cc6092a6`, `mem_1eca95a3-bd5a-481c-8407-138c3a2eb125`; proposta **in attesa**: `mem_7118d6a8-d63f-4cfc-a9c7-ae756c3f42b4` (resconto patch, vedi rilievo 3).
- **perimetro**: SOLO LA PATCH. L'audit L4 precedente (`reports/audit_l4_impl.md`, CONFORME su `280412f`) resta CHIUSO: nessun punto E1/E3/preimage/provenienza/L2.27/R1/R2 riaperto.
- **revisione**: HEAD `30adda8892a1a2786e52703632816fba17ecbf4d` confermato con `git log -1 --format=%H`; sopra `48daac3` (test) sopra `adc0b84`. Gate tester `reports/gate_l4_patch.md` VERDE sulla stessa revisione (145/145 debug e release). Non eseguo test (non compito del reviewer): solo letture Git/file.

## 1. Punto 1 — fix conforme alla direzione uniforme: **CONFORME**

Diff `git show 30adda8 -- src/identity.rs`: 9+/6−, **un solo hunk** dentro `fn replay` (righe 496–511 del file corrente). Il vecchio `continue` per `e.r#type != EventType::Action` (che saltava `validate_decision_fields` per i non-ACTION) è rimosso; il nuovo codice chiama `validate_decision_fields(e)?` (src/identity.rs:506) su **ogni** evento non-`l4` del loop e aggiunge `if (e.r#type != EventType::Action) && (e.payload.get("l2").is_some() || e.payload.get("l3").is_some()) { return Err(SelfError::Schema); }` (src/identity.rs:507–511).

- Direzione applicata esattamente come firmata: un evento non-`ACTION` con chiavi `l2`/`l3` → `SelfError::Schema` anche nel replay di `build_self` (src/identity.rs:507–511; `build_self` a src/identity.rs:521–535 chiama `replay` a 522). Domini chiusi L4.1 + mappatura errori L4.2, niente di più.
- **Nessun effetto oltre la patch** (verifica della rimozione del vecchio `continue`): le affermazioni nel replay nascono **solo** da eventi `narrate` nel ramo `l4_tag`, che termina col suo `continue` a src/identity.rs:494; per gli eventi non-l4 non c'è raccolta di statements dopo le righe 506–511 (il loop finisce a 512). Cambio comportamento limitato ai non-ACTION non-l4: (a) saltati prima, ora validati da `validate_decision_fields` (no-op per eventi senza chiavi l2/l3: src/identity.rs:237–272), (b) con chiavi l2/l3 → Schema. Nessun cambiamento di cutoff/replay/canonicalità: cutoff inalterato (src/identity.rs:541–549), digest e ordinamento statements inalterati (src/identity.rs:521–535).
- Coerenza col resto: il ramo `l4` del replay non è toccato (src/identity.rs:441–494) e non subisce la nuova validazione (il suo `continue` a 494 precede la riga 506). `evaluate_self` condivide lo stesso `replay` (src/identity.rs:548): le due API non rappresentano più esiti divergenti sullo stesso log.

## 2. Punto 2 — nessun effetto collaterale: **CONFORME**

- `git diff --name-only adc0b84..30adda8` → solo `src/identity.rs` e `tests/l4_identity.rs`. Nessun tocco a `src/memory.rs`, `src/selection.rs`, `src/lib.rs`, L2/L3, policy o altri test (conferma del perimetro del gate, `reports/gate_l4_patch.md` §4).
- Per i 22 test L4 pre-esistenti e i test L0–L3, caso per caso: (a) eventi `l4` → percorso invariato (src/identity.rs:441–494); (b) eventi ACTION l2/l3 → stesso flusso (validate, poi fine loop), esito invariato; (c) eventi non-ACTION **senza** chiavi l2/l3 → `validate_decision_fields` no-op (src/identity.rs:271), esito invariato; (d) test L4 con preimage/hash → `replay`/`doc_digest` inalterati. I 145/145 del gate = 144 pre-esistenti + 1 test nuovo, invariati in debug e release (`reports/gate_l4_patch.md` §2/§3).

## 3. Punto 3 — test nuovo discriminante: **CONFORME**

- `git show --stat 48daac3` → **solo** `tests/l4_identity.rs` (+17, 0−), nessun codice di produzione nel primo commit del task: la regola test-first è rispettata senza eccezioni da registrare.
- `tests/l4_identity.rs:646–656` `l4_check4_replay_non_action_with_l2_or_l3_is_schema`: log di genesi + `EventType::Sense` con `("l2","select"), ("at",0), ("ups","[0]"), ("downs","[]")` — decisionale l2 **valido nelle forme** (il ramo "select" esigente su `ups`/`downs` passa: src/identity.rs:245–249) — assert `build_self(&log).unwrap_err() == SelfError::Schema`.
- Esercita esattamente il caso della patch: evento non-ACTION, senza `l4` nel payload, con chiave `l2`; pre-patch → `continue` sulla vecchia linea del diff → `Ok(SelfDocument)`, post-patch → Schema per es. a righe 507–508. Rosso ricostruito a `48daac3` documentato dal gate (`reports/gate_l4_patch.md` §5: `unwrap_err()` su valore `Ok`, `FAILED. 22 passed; 1 failed`): rosso *sostantivo* (divergenza API), non errore di fixture. Non rieseguo i run: ne verifico solo il tecnicismo del codice e della fixture.

## VERDETTO: **CONFORME**

Il diff `30adda8` implementa esclusivamente la direzione uniforme firmata: la validazione `l2`/`l3` vale per ogni evento letto dal piano L4, a prescindere dal tipo L0, mappata con `Schema` per i non-ACTION con `l2`/`l3`. Nessun effetto collaterale sui 145 test; nessun altro file toccato. Il test nuovo è test-first, ben formato e discriminante. Sessione mnemosyne chiusa al termine.

## Rilievi (3/3, informativi — non violazioni)

1. **Disgiunzione esercitata solo per `l2`**: la condizione della patch copre `l2` **oppure** `l3` (src/identity.rs:507–508), ma il nuovo test (tests/l4_identity.rs:646–656) esercita solo `l2` su un evento `Sense`; il ramo `l3` su evento non-ACTION non ha test diretto (coperto solo per simmetria da `validate_decision_fields` e dal caso ACTION preesistente `l4_check4_malformed_l3_fields_are_schema`, tests/l4_identity.rs:660 segg.). Segnalo, non risolvo: eventuale test complementare alla valutazione del producer.
2. **Ordine dei controlli, identico in esito**: per un non-ACTION con payload decisionale *malformato* (es. `l2`+`l3` insieme, o id `l3` non hex-64) l'errore esce da `validate_decision_fields` (src/identity.rs:238–271) **prima** del nuovo controllo di tipo (507–511). Entrambi i percorsi → `SelfError::Schema`, quindi nessuna differenza osservabile per API né test; citato solo per completezza.
3. **Proposta memoria pendente**: `mem_7118d6a8-d63f-4cfc-a9c7-ae756c3f42b4` (resconto della patch) è in `pending_approval` e riguarda questo livello. Per protocollo non ne prendo decisione: l'approvazione spettano all'orchestratore (delega umana). Nessun conflitto irrisolto (`memory_list_conflicts` → 0) e nessuna memoria approvata in contraddizione col codice: la legge L4 in `mem_ffe6a92e` (dominio chiuso `l4=ACTION`, errori → Schema) è coerente con la patch.
