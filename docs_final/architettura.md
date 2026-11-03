# Architettura e livelli

## Principio generale

Il substrato è un log append-only. Gli eventi registrati hanno sequenza,
tick logico, tipo, payload e collegamento crittografico alla storia
precedente. I livelli superiori ricostruiscono viste o producono decisioni a
partire dal log; l'apposizione delle decisioni resta responsabilità del
chiamante tramite L0. La catena degli hash attesta integrità rispetto ai
byte registrati, non la verità degli eventi né l'identità di chi li ha
immessi.

L'implementazione è una libreria Rust con binari diagnostici e una
dipendenza di produzione `sha2`; vedi `Cargo.toml` e `src/lib.rs`.

## L0 — Substrato

- **Codice:** `src/log.rs`, `src/event.rs`, `src/payload.rs`,
  `src/prng.rs`, `src/state.rs`, `src/cycle.rs`.
- **Contratto:** `spec/L0_SPEC_v0.4.md`.
- Tipi evento `SENSE`, `ACTION`, `STATE`; payload piatto con valori interi
  o stringhe, serializzazione canonica e catena SHA-256. `Log::verify`
  controlla la catena; JSONL permette esportazione e ricostruzione.
- Il ciclo dimostrativo usa PCG64: uno stimolo per tick e una policy stub
  deterministica. Non è un'interfaccia completa verso un mondo reale.

## L1 — Vista semantica

- **Codice:** `src/view.rs`; **fonti:** `spec/L1_SPEC_v0.3.md` e addendum
  F1 vigente.
- `view::build` deriva nodi e archi da estrazioni nel log. Identità e
  provenienza mantengono la prima sequenza di origine; i cicli nel grafo
  sono ammessi.
- Le collezioni strutturate nel payload L0 usano stringhe con JSON interno
  canonico, non array come valori diretti del payload.

## L2 — Selezione

- **Codice:** `src/selection.rs`; **fonti:** `spec/L2_SPEC_v0.3.md`,
  `notes/L2_legge_selezione.md` e L2.27.
- Ricostruisce un insieme attivo a budget finito e genera decisioni di
  selezione/consolidamento. Gli eventi L2 sono `ACTION` con discriminante
  `l2`; codifica e append sono separate.
- La riprova di un arco riattiva gli endpoint dichiarati **dallo stesso
  evento**, secondo L2.27; l'origine L1 resta distinta dall'attivazione.

## L3 — Memoria lunga

- **Codice:** `src/memory.rs`; **fonti:** `spec/L3_SPEC_v0.1.md`,
  `notes/L3_legge_memoria.md`, `notes/L3_provenienza.md`.
- Deriva stato e archivio delle identità: promozione, archiviazione e
  richiamo sono eventi `ACTION` con discriminante `l3`. L'archiviazione non
  cancella la storia del log.
- Le regole E1/E2 e l'emendamento E3 fanno parte del contesto normativo:
  per i dettagli prevalgono i documenti firmati e i test del livello.

## L4 — Documento del sé

- **Codice:** `src/identity.rs`; **fonti:** `spec/L4_SPEC_v0.1.md`,
  `notes/L4_legge_identita.md`.
- La narrazione è deterministica e riguarda prime dichiarazioni e specifiche
  decisioni L2/L3. Le revisioni usano eventi `l4=narrate` e `l4=snapshot`;
  il `self_hash` usa il preimage canonico privo del proprio campo hash.
- Il documento è ricostruito dal log, non conservato in un file separato.
  `head_hash` ancora la vista alla testa: log con narrazioni equivalenti
  possono comunque produrre documenti completi diversi se cambia la testa.

## L5 — Modelli sociali

- **Codice:** `src/social.rs`; **fonti:** `spec/L5_SPEC_v0.1.md`,
  `notes/L5_legge_sociale.md`.
- Le osservazioni sociali sono lette da eventi `SENSE` con payload conforme
  alla legge L5. Il modello usa contatori interi per comportamento e una
  previsione deterministica; `predict` e `model_update` sono eventi L5.
- Il ciclo stub L0 non genera automaticamente osservazioni sociali. Il
  secondo ordine implementato è una composizione specificata in v0.1, non
  una dimostrazione generale di teoria della mente o accesso ad altri agenti.

## L6 — Valutazione

- **Codice:** `src/valuation.rs`, `src/bin/l6_experiment.rs`; **fonti:**
  `spec/L6_SPEC_v0.2.md`, `notes/L6_legge_valutazione.md`.
- L'harness esegue un esperimento sintetico appaiato: due braccia ricevono
  gli stessi task, una ricava le risposte dalla storia del proprio log e
  l'altra risponde sempre con la classe 0. Sono previste 100 coppie di seed
  e una statistica Wilcoxon a una coda.
- La SPEC parla di valutazione; la descrizione «L6 Meta ricalibra i livelli»
  nel README storico non descrive una funzione di ricalibrazione attualmente
  implementata. L'esperimento non prova di per sé il beneficio delle viste
  L1–L5: vedere [verifiche.md](verifiche.md).
