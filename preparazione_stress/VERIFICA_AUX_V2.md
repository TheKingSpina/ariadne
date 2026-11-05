# Verifica del profilo AUX v2

## Esito

Suite Rust eseguita in Docker sul working tree locale: **196 test passati,
0 falliti, 0 ignorati**, inclusi 14 nuovi test in `tests/aux_v2.rs`.
Non è stato eseguito lo stress test né l'esperimento L6 completo.

- Immagine: `rust:1.85.1@sha256:e51d0265072d2d9d5d320f6a44dde6b9ef13653b035098febd68cce8fa7c0bc4`.
- Rust: `rustc 1.85.1 (4eb161250 2025-03-15)`; Docker Linux arm64.
- Comando nel container: `cargo test --locked --quiet`.
- Input copiati via archivio tar: `Cargo.toml`, `Cargo.lock`, `src`, `tests`.
  Nessun mount scrivibile del repository, nessun fallback host.
- Il working tree non è ancora committato: il risultato non è riferibile
  al solo HEAD storico `f447ca4`. Per il congelamento occorre un commit
  successivo e una nuova esecuzione su quel commit.
- Warning preesistente: `unused_mut`, `tests/l6_protocol.rs:119`.

## Copertura aggiunta

JSON canonico; compatibilità byte-per-byte dei cicli L0 v1; set L2 con
origini coincidenti; summary vuoto e non vuoto; finestra aperta senza
chiusura implicita; rifiuto di batch troncato; rollback degli errori;
deduplicazione attivazioni e giudizi; promote/archive/recall; snapshot L4;
replay delle proiezioni; rifiuto di una decisione alterata anche dopo
ricalcolo della catena; fixture L5, accordo/disaccordo e secondo ordine
identità; rifiuto di riassociazione ID fixture e ancore errate.

Il test a quattro nodi verifica una sequenza di 821 eventi ai tick 0–254,
non uno stress test. Le posizioni attese sono elencate nel contratto
`AUX_V2_PROFILO_IMPLEMENTATO.md`.

## Limiti dell'attestazione

Verifica eseguita dall'implementatore, non da tester/reviewer indipendenti.
Il replay semantico riusa il generatore di decisioni; hash e checkpoint
espliciti nei test non equivalgono a un oracolo indipendente completo.
Non sono attestate tutte le proposte delle bozze: il profilo implementato
esplicita le esclusioni, tra cui fonti multiple, finestre con più select,
compensazioni v2 e runner completo. Manifest e oracoli dello stress test
restano da allineare, revisionare e approvare prima della sua esecuzione.
