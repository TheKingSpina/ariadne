# Gate test compatto L0–L3 — revisione b2770bd

modello: opencode/deepseek-v4.1-flash-free (nessuna variante, default catalogo)
classe: light (mandato Router) — policy `.opencode/routing.md` v1, 31/10/2026, ATTIVO
session ID tester: ses_e6d5af25-24c7-4901-88eb-68d000b3af23
memorie richiamate: nessuna; memorie proposte: nessuna (i tool mnemosyne `memory_context`/`memory_search` non erano invocabili con la firma del catalogo corrente: chiamata rifiutata con `Unknown tool`)

## Revisione Git

```
$ git log -1 --format=%H
b2770bdf3e60883258038db37f36f3ea2ad6432e
```
Coincide con la HEAD attesa `b2770bd`. Nota di metodo: `git rev-parse HEAD` è stato rifiutato
dalla sandbox (`Permission denied: shell`), quindi ho usato `git log -1 --format=%H`, che
restituisce lo stesso oggetto. Nessun retry sul comando negato.

Stato albero al momento dei run (`git status --porcelain`): `M opencode.jsonc`, più file non
tracciati in `notes/` e `reports/`. Nessuna modifica a `src/` o `tests/`.

## Comandi eseguiti (testo esatto)

```
$ git log -1 --format=%H
$ git status --porcelain
$ cargo test
$ cargo test --release
$ shasum -a 256 target/debug/deps/ariadne-f4839cd99eb39cde
```

## Run debug — `cargo test`

Totale: **119 passed; 0 failed; 0 ignored** (+ 0 doc-test).

| Suite | passed | failed | ignored |
| --- | --- | --- | --- |
| unittests src/lib.rs | 4 | 0 | 0 |
| unittests src/bin/l0_replay.rs | 0 | 0 | 0 |
| tests/l0_c1_genesis.rs | 2 | 0 | 0 |
| tests/l0_cycle.rs | 11 | 0 | 0 |
| tests/l0_event_log.rs | 18 | 0 | 0 |
| tests/l0_l1_l2_pipeline.rs | 1 | 0 | 0 |
| tests/l0_n3_canonical_string.rs | 13 | 0 | 0 |
| tests/l0_n3_line_canonical.rs | 10 | 0 | 0 |
| tests/l0_prng.rs | 7 | 0 | 0 |
| tests/l0_replay_process.rs | 6 | 0 | 0 |
| tests/l0_state.rs | 9 | 0 | 0 |
| tests/l1_view.rs | 11 | 0 | 0 |
| tests/l2_selection.rs | 13 | 0 | 0 |
| tests/l3_memory.rs | 14 | 0 | 0 |
| doc-tests ariadne | 0 | 0 | 0 |

Output aggregato delle suite: `test result: ok.` su tutte le 15 righe di risultato.

L3: 14 test osservati in `tests/l3_memory.rs`. `src/memory.rs` non contiene `#[test]` né
`#[cfg(test)]` (grep senza corrispondenze), quindi i 14 L3 provengono dal solo file di
integrazione. Unica warning di build: `unused import: EventType` in `src/selection.rs:3`.

Hash SHA-256 del binario di test debug
`target/debug/deps/ariadne-f4839cd99eb39cde`:
```
73a4160e0ead4ed443b0b21fa386530af5f903241448de50211ae2262d5c7ec2
```

## Run release — `cargo test --release`

Totale: **119 passed; 0 failed; 0 ignored** (+ 0 doc-test).

| Suite | passed | failed | ignored |
| --- | --- | --- | --- |
| unittests src/lib.rs | 4 | 0 | 0 |
| unittests src/bin/l0_replay.rs | 0 | 0 | 0 |
| tests/l0_c1_genesis.rs | 2 | 0 | 0 |
| tests/l0_cycle.rs | 11 | 0 | 0 |
| tests/l0_event_log.rs | 18 | 0 | 0 |
| tests/l0_l1_l2_pipeline.rs | 1 | 0 | 0 |
| tests/l0_n3_canonical_string.rs | 13 | 0 | 0 |
| tests/l0_n3_line_canonical.rs | 10 | 0 | 0 |
| tests/l0_prng.rs | 7 | 0 | 0 |
| tests/l0_replay_process.rs | 6 | 0 | 0 |
| tests/l0_state.rs | 9 | 0 | 0 |
| tests/l1_view.rs | 11 | 0 | 0 |
| tests/l2_selection.rs | 13 | 0 | 0 |
| tests/l3_memory.rs | 14 | 0 | 0 |
| doc-tests ariadne | 0 | 0 | 0 |

`test result: ok.` su tutte le 15 righe di risultato. Stessa unica warning (`unused import:
EventType`, `src/selection.rs:3`).

## Confronto con l'atteso

| Run | atteso | osservato | L3 atteso | L3 osservato | esito |
| --- | --- | --- | --- | --- | --- |
| debug | 119 | 119 (0 failed) | 14 | 14 | conforme |
| release | 119 | 119 (0 failed) | 14 | 14 | conforme |

## Esito sintetico del gate

**VERDE** — entrambi i run (debug e release) su b2770bd: 119/119 passati, 0 falliti, 14 L3.

## Limitazioni dichiarate

1. I run sono stati eseguiti nel checkout principale di `/Users/spina/Ariadne`, non in un
   worktree dedicato: la creazione di worktree non era prevista dal mandato e
   `git rev-parse`/altri comandi sono stati bloccati dalla sandbox. L'albero era comunque
   privo di modifiche a `src/` e `tests/` al momento dei run (unica modifica tracciata:
   `opencode.jsonc`, più file non tracciati in `notes/` e `reports/`).
2. Il run release è partito con artefatti già compilati (`Finished release profile ... in
   0.02s`), quindi non documenta una ricompilazione completa da zero in release; i test
   sono stati comunque eseguiti e riportati per intero.
3. I conteggi sono presi dalla stdout integrale di ciascun run, non da riepiloghi esterni.
4. Non sono stati verificati in questa sessione gli altri requisiti del gate L0–L3
   (ordine dei commit test-first, provenienza `seq`, immutabilità): il mandato richiedeva
   solo l'esecuzione delle due suite e i conteggi. Questo report non chiude quindi il gate
   globale sulla revisione integrata.
