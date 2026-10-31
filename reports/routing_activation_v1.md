# Attivazione routing v1 — 31 ottobre 2026

modello: openai/gpt-6.1-sol

Autorizzazione umana: «direi di attivarlo (ma fai un backup del sistema attuale,
non si sa mai)». Attivazione locale della configurazione, non collaudo dei modelli.

## Backup

Percorso: `/Users/spina/Ariadne-agent-backups/pre-routing-20261031-195101`.
Manifest SHA-256 verificato prima e dopo le modifiche: 13 file, snapshot del
working tree e baseline committata a `a1d88ebff3d7408d22b751b6dcce9cd66ed6901c`.
Istruzioni di ripristino in `RESTORE.md`, fuori dalla discovery `.opencode/`.

## Verifiche eseguite

Comando di discovery: `opencode debug agents`, esito 0. Output JSON letto
e controllato senza effettuare chiamate a modelli:

- `orchestrator`: GLM 5.3 Flash free, variante high; può delegare solo a Router.
- `router`: MiMo V2.6 Flash free; può delegare solo a producer/tester/reviewer.
- Producer/tester/reviewer: nessun modello configurato; dispatch esplicito
  richiesto dalla policy. Delega ad altri agenti negata.
- Scrittura Router negata per codice e policy, consentita nei report;
  `cargo test` negato al Router.
- Policy v1 ATTIVA nel working tree; `git diff --check` senza errori.
- HEAD invariato; nessuna modifica a codice, test Rust o SPEC.

Controlli dei permessi effettuati sull'elenco effettivamente caricato dal CLI,
valutando le regole in ordine (ultima corrispondenza prevalente).

## Limiti e stato di consegna

Nessun dispatch modello, consumo di quota, cargo, commit o push. Disponibilità
dei modelli e delega annidata con selezione runtime esplicita NON collaudate.
I figli senza pin erediterebbero il modello parent in assenza del parametro
`model`: la policy impone il parametro, ma questa verifica non ne prova
l'applicazione end-to-end. La selezione del modello di una sessione primaria
già aperta non cambia automaticamente selezionando un altro agente.

Seriale per default. Parallelismo solo con mandato completo di limiti e
isolamento; nessun fallback automatico; passaggi onerosi soggetti a consenso.
Span non identificato e selezione downstream di Free Models Router restano
bloccati dalle verifiche specifiche mancanti, non dalla policy globale.
