# Report implementazione L4 — legge del sé (identità narrativa)

> **Nota di trascrizione (orchestratore, 31/10/2026).** Il ruolo producer ha
> il permesso `edit reports/**: deny` a livello harness
> (`.opencode/agents/producer.md:10-12`): il contenuto sotto è trascritto
> INTEGRALMENTE dal resoconto testuale finale del producer
> (`opencode/glm-5.3-flash-free`, subagent `ses_e6c53a29affea4AdUkfEiSmo0M`),
> consegnato al Router come da provvedimento opzione 2 in bozza
> (`notes/agenti_riorganizzazione.md`). Trascrizione fedele, nessuna
> modifica di sostanza.

- **modello**: opencode/glm-5.3-flash-free, classe light, variante nessuna;
  canale «Free-first», 31/10/2026.
- **contratto**: `notes/L4_legge_identita.md` — base `2ec39a0` («scegli tu»,
  delega umana) + precisazione L4.6 `sources` e provvedimenti `302e7e2` ×2
  (diagnosi + precisazione) e autorizzazioni correzioni test 1–9
  (31/10/2026).
- **session ID**: `ses_e6c53a29affea4AdUkfEiSmo0M` (subagent); Mnemosyne
  ripresa 4: `ses_6d99e3e8-bc15-41ab-b936-ee06ca9d3bbd`.

**Commit**: 1) `c8ae8de` — `tests/l4_identity.rs`, 22 test rossi per assenza
di `ariadne::identity` (E0432/E0433). 2) `280412f` — `src/identity.rs`
completo, riga `pub mod identity;`, 8 correzioni test autorizzate.

**I 22 test l4 — esito finale (verdi 22/22)**:
`l4_check1_genesis_only_empty_statements_head_hash_and_preimage` ✓ ·
`l4_check2_i_irrelevant_difference_produces_identical_narrated_statements` ✓ ·
`l4_check2_ii_relevant_difference_produces_divergent_narrated_statements` ✓ ·
`l4_check3_all_templates_exact_texts_internal_order_sources_and_join` ✓ ·
`l4_check3_control_rementions_and_no_extraction_events_are_not_relevant` ✓ ·
`l4_check3_select_list_rendering_sorted_dedup_and_empty` ✓ ·
`l4_check4_broken_pairs_are_schema` ✓ ·
`l4_check4_extra_keys_and_bad_shapes_are_schema` ✓ ·
`l4_check4_l4_with_l2_or_l3_is_schema` ✓ ·
`l4_check4_lone_narrate_as_last_event_is_tolerated_and_authoritative` ✓ ·
`l4_check4_malformed_l3_fields_are_schema` ✓ ·
`l4_check4_narrate_at_inexistent_or_l4_is_schema` ✓ ·
`l4_check4_replay_rebuilds_document_and_snapshots_verified` ✓ ·
`l4_check4_self_hash_mismatch_is_schema` ✓ ·
`l4_check4_state_with_l4_is_schema` ✓ ·
`l4_check4_unpaired_snapshot_is_schema` ✓ ·
`l4_check5_cutoff_tail_only_and_prefix_coherence` ✓ ·
`l4_check6_l4_event_closes_l2_window_without_side_effects` ✓ ·
`l4_check7_determinism_two_evaluations_identical` ✓ ·
`l4_check8_l4_apis_do_not_advance_prng_state` ✓ ·
`l4_encode4_narrate_and_snapshot_payload_pairs` ✓ ·
`l4_encode4_payload_appendable_as_l0_event` ✓

**File toccati (nel commit 2)**: `src/identity.rs` (nuovo, solo file
producer di logica), `src/lib.rs` (una riga `pub mod identity;`),
`tests/l4_identity.rs` (le 8 prescrizioni autorizzate). Non toccati:
`src/memory.rs`, `src/selection.rs`, note, policy, config, altri test;
`examples/l4scratch.rs` cancellato (perimetro); scrittura in `reports/`
NON tentata (divieto reale).

**Conteggi suite**: debug **144/144** (16 harness: L0 80, L1 ~16, L2 11,
L3 15, L4 22 — asse esatti aggregati 144), release **144/144**.

**Interpretazioni di legge applicate** (nessuna nuova in autonomia):
1. `sources` payload narrate = stringa F1 `[i64,…]`, dedup crescente,
   altrimenti `Schema` (L4.1/L4.6/L4.7).
2. `sources` della revisione (precisazione `302e7e2`): unione dedup-crescente
   delle seq CITATE nei template — seq generatrice + seq in
   `{ups}`/`{downs}`/`{expulsions}`/`{origin}` (implementata in
   `decision_statements`/`extraction_statements` con `sort_dedup`).
3. `{seq}` dei template = seq dell'evento generatore (L4.3 letterale);
   `{tick}` decisioni = tick dell'evento apposto (L2.18).
4. Snapshot `self_hash` a decision-time = SIMULAZIONE dell'apposizione della
   coppia narrate+snapshot in coda con `Event::seal` in catena
   (seq = len eventi a stadi `log.events().len()+2k`, tick host, `ACTION`,
   payload `encode4`, `prev_hash` = ultimo hash simulato; a partire dalla
   coda reale del log) — per il self_hash dell'ancora l'hash del narrate
   esiste solo post-append (L4.4 + L4.9).
5. `build_self`: head = ultimo evento reale; appaiamento stretto; ogni
   snapshot verificato duramente contro il preimage ricostruito dai narrate
   REALI del log; narrate solitario in coda tollerato e autorevole (L2.11);
   autoreferenzialità / `at` inesistente / chiavi extra / l4+l2-l3 / lista
   non canonica / non-ACTION → `SelfError::Schema`.
6. Cut-off = `max(seq ultimo evento l4 del log, seq evento in posizione
   base.log_len−1)` (L4.6/L4.8 + pattern L2.22); nessuna decisione per
   `seq ≤ cutoff`; prefix-base con HeadMatch su len/hash errati o base vuota.
7. Documento L4.4: chiavi lessicografiche, `schema_version:"1"`, statements
   per revisione (ordine di `at`; testo = affermazioni congiunte `\n`;
   membro `{"at":…,"sources":[…],"text":…}`), `self_hash` = SHA-256 del
   preimage privo del campo (principio D16/L3.8).
8. Rilevanza L4.3(a/b) via `view::build` (id L1 = SHA-256; `origin_seq` D18
   = prima dichiarazione); un evento → più affermazioni, ordine interno
   lessicografico nodi `(kind,key)` poi archi `(rel,src_id,dst_id)`;
   checkpoint 2(i): il documento intero non può coincidere per la catena L0
   → asserta la coincidenza scoped di statements + head/self_hash differiti;
   UN evento `l4` chiude la finestra L2 (L4.9, nessun side-effect, test
   verificato).
9. Zero stocastica (L4.5): nessun draw/PRNG/I-O/wall-clock/RNG in
   `identity.rs`; verificato dal test check8.

**Il gate spetta al tester, l'accettazione all'umano**: il producer NON
dichiara la checklist verde né la conformità; il commit `280412f` è pronto
per il gate L4 e per l'audit reviewer con le sue prove (22 test rossi nel
commit 1, messaggio commit con autorizzazioni citate).
