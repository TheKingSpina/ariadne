# RUN L0 — chiuso il 22 ottobre

## Esito tecnico
HEAD `a02b66a`. **80 test verdi, eseguiti** dal tester su worktree pulito a HEAD,
in debug e in release (80 passed / 0 failed, exit 0), replay a due processi 6/6.
Dei 70 a `fba6565` la suite a HEAD non era mai stata eseguita in modo indipendente:
ora lo è. Resta un rosso **processuale** (B4) e nessun rosso funzionale.
Gate §1–§5: conforme. Reviewer: **§3 VIOLATO rispetto alla SPEC come scritta**
(payload compensativo a 3 chiavi vs 2 prescritte) — risolto in codice con
l'opzione A, non ancora scritto nella SPEC.

## Commit L0 (locali, non pushati)
```
6ae990a → 8d71937   §2–§3 (formato evento, append-only, compensazioni)
038e51d → 99b9d06   engine PCG64
d2f1610 → b7543f7   genesi con stato PCG64 reale (chiude D1)
48e5529 → a345be7   State::fold
08b1536 → b2790e9   ciclo SENSE→ACTION→STATE   [+ a298cf9 fix import = B4 indebolita]
6d6eadc → 5b495d7   replay a due processi
7f76fea → 649deea   C1: compensativi autodescrittivi + genesis fallibile
        → 45ad98d   fix aspettativa errata nel test
        → 238d586   CLI fallisce bene su seed fuori range
88a68da → 5d0a652   N3: parser severo sul token stringa
1b453f9             N4: fixture canonica, golden ricalcolato con shasum
fba6565             pulizia import nel test N3
edcca4a → a02b66a   N3-bis: canonicità dell'intera riga (ricomposizione)
```

## Report
- `reports/L0_test_report.md` — §2–§3
- `reports/L0_prng_genesis_test_report.md` — PCG64 + genesi
- `reports/L0_conformance_test_report.md` — §1–§5 (a `5b495d7`, §3 superato)
- `reports/L0_commits_test_report.md` — archeologia test-first, 6 coppie
- `reports/L0_conformance_recheck_test_report.md` — §3 e invarianti a `238d586`
- `reports/L0_n3_n4_test_report.md` — N3+N4: **ROSSO in voce 2** (canonicità di riga), 7 APERTA

## Decisioni umane prese
- `state` = stringa decimale 128 bit; `stimolo` = stringa decimale u64 completa.
- Opzione A: compensativo autodescrittivo `{compensates, reason, state}`.
- `genesis` fallibile: `seed > i64::MAX` rifiutato.
- Non riscrivere la storia per cancellare il rosso di B4.

## Decisioni umane firmate (22 ottobre)
1. **N1 — seeding**: L0 è il riferimento, splitmix64 canonizzato nella SPEC. SeedSequence
   è un dettaglio di numpy; Ariadne è local-first. Chi vuole confrontarsi consuma lo
   stream canonico L0.
2. **N2 — seed**: 1000–1099 stanno in `i64`. Nessuna estensione. Chiuso.
3. **N3 — parser**: severo. `from_jsonl` rifiuta ogni encoding che il serializzatore non
   emette. Il log esterno o è canonico o non entra.
4. **N4 — fixture**: golden obsoleto sostituito con la forma B2. Il vecchio hash resta
   nei report, la storia non si riscrive.
5. **SPEC §3 — opzione A**: addendum v0.3 in `notes/L0_SPEC_v0.3_addendum_proposta.md`.
   **Da firmare** (umano + orchestratore). Senza firma, l'audit formale rileva la
   violazione §3.

## Decisioni di processo firmate
- Scratch directory fuori repo per il tester: **fatto** (permesso edit sul tmp dir).
- Whitelist git read-only per l'orchestratore: **fatto** (`status`/`log`/`diff`/`show`/`worktree list`).
- Modelli pinnati con versione nei report: **fatto** (riga `modello:` obbligatoria nei prompt).
- Mnemosyne esercitato dal prossimo run: **fatto** — i prompt nominavano `memory_store`
  e `memory_recall`, che **non esistono**. API reale a sessioni: `memory_open_session` →
  `memory_context`/`memory_search` → `memory_propose` → `memory_close_session`.


## Gate N3 + N4 + N3-bis (chiuso)
- Tester: `reports/L0_n3_n4_test_report.md` §0–§11 (N3+N4) e §12 (N3-bis).
  **80 test verdi eseguiti** su worktree pulito a `a02b66a`, debug e release, exit 0;
  rosso riprodotto a `edcca4a` (7 ok / 3 failed, exit 101). Nessun accettato-e-non-canonico.
  Cinque casi limite provati: chiavi duplicate (rifiuto), CRLF (rifiuto, guardia
  preesistente), newline finale opzionale (contratto invariato), hash incoerente
  (accettato dal parse, rifiutato da `verify()` — schema ≠ catena), una riga canonica
  per ogni tipo di evento (round-trip identico).
- Reviewer: `reports/L0_review_report.md`. **RESPINTO** — una sola violazione,
  §3 della v0.2: payload compensativo a 3 chiavi contro 2 prescritte
  (`src/log.rs:145-149` vs `spec/L0_SPEC_v0.2.md:19`). Tutti gli altri vincoli CONFORME.
- **Il respingimento si scioglie con una firma**: la violazione è quella che l'addendum
  v0.3 emenda. Firmato l'addendum, L0 non ha violazioni note.
- Debito noto, non riesaminato: B4 (coppia `08b1536`→`b2790e9` con test mai compilato
  contro la sua implementazione) e i due test corretti dopo (`a298cf9`, `45ad98d`).

## Le proposte non approvate non sono memoria
Il reviewer ha cercato le sei memorie citate nel dispatch con `memory_context` e non le
ha trovate: ha concluso che «non esistono». **L'esistenza è dimostrata**: sono tutte in
`memory_admin_memories(lifecycle: "pending_approval")`. Il meccanismo è un altro, ed è
il punto: `memory_context` restituisce **solo le memorie approvate**, quindi una proposta
in attesa è invisibile al richiamo di ogni run successivo.
Conseguenza da tenere presente: **tredici proposte in attesa valgono zero**, qualunque
cosa dicano. Non sono decorazione in attesa di approvazione: sono, oggi, assenza.
L'approvazione umana è l'unico passaggio che le rende esercitabili.


- **Il commit era l'ultima voce**: 3 dispatch su 6 del producer sono chiusi con
  lavoro non committato. Aggiunta la "riserva di commit" al prompt.
- **Il tester non può costruire controesempi**: scrive solo sotto `reports/`, quindi
  non può verificare che una suite fallirebbe se il codice fosse sbagliato.
  Serve una directory di scratch scrivibile fuori dal repository.
- **L'orchestratore non vede il repository**: senza `git status/log/diff` può solo
  credere a ciò che gli agenti dichiarano. Proposta: whitelist read-only.
- **Mnemosyne non esercitato**: zero `memory_recall` in tutto il run.
- **Un modello è sparito a metà run** (minimax-m3.1): i modelli come stringa nel
  frontmatter sono un punto di rottura.
- **Comandi composti e permessi** (`22 ottobre`): `cargo *` era in whitelist, ma il
  tester ha scritto `cd … && cargo test`. Il permesso è sul comando intero, quindi è
  stato negato due volte e il gate è chiuso senza esecuzione. Regola aggiunta: mai
  `cd`/`&&`/`;`/`|`, si passa `workdir` allo strumento.
- **Il tester esaurisce gli step sull'analisi e arriva all'esecuzione per ultima**
  (due gate su due). Regola aggiunta: la suite sul worktree pulito è il **primo** passo.
- **Gli agenti ereditano l'indice git**: il producer ha committato dieci file staged
  altrui (`.opencode/`, `notes/`, `reports/`, `readme.md`). Corretto con `reset --soft`.
  Finché quei file restano in stage, ogni `git commit` senza `-- <paths>` li inghiotte.
- **Il reviewer è morto per timeout a monte** (504, primo tentativo di audit finale):
  aveva letto molto e non aveva scritto nulla. Regola aggiunta: l'artefatto si crea nei
  primi passi e si aggiorna a ogni passo. Un report parziale è un risultato, un'analisi
  non salvata è zero.
- **Un modello assente dalla lista dei più recenti non è un modello sparito**: il tool
  `models` senza `all` mostra solo l'ultima versione per famiglia, e l'audit ha rischiato
  di dichiarare `nemotron-3-ultra-free` «sparito» quando era `active`. Prima di sostituire
  un modello pinnato, verificare con `all: true`.
