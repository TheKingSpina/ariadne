# Lettura del risultato L6 — 5 novembre 2026

**Comando eseguito:** `cargo run --release --locked --bin l6_experiment`
**Report generato:** `reports/l6_esperimento.md`
**Log delle run:** `runs/l6/` (200 file, non committati)

## Risultato

- 100 coppie appaiate, seed `1000..=1099`, nessuna esclusione.
- 0 violazioni di protocollo (timeout, replay, forme).
- 200/200 log ricostruiti byte per byte.
- Wilcoxon appaiato una coda: `W = 5050.0`, `n = 100`, `p = 0.000000`,
  `Δ̄ = 0.6647` unità reward.
- Verdetto previsto dalla legge L6: **SUCCESSO**.

## Cosa significa

Il risultato riguarda il corpus sintetico e il criterio preregistrato
dell'harness: nel confronto appaiato, il braccio con memoria ottiene reward
di test più alti del controllo, con replay integro e nessuna violazione.

## Cosa non significa

- **Non è lo stress test L0–L6** descritto da
  `manifest_stress_L0_L6_bozza.md`: quel manifest è ancora una bozza con
  `TBD` e non è stato eseguito.
- **Non dimostra l'efficacia causale di L1–L5.** L6 è un esperimento
  sintetico separato; il suo esito non si trasferisce automaticamente ai
  livelli inferiori né a dati reali.
- **Non autentica fatti, persone o agenti.** Il replay verifica integrità e
  coerenza del protocollo, non la verità delle dichiarazioni.
- Il corpus e le reward sono generati dall'harness; il p-value dipende dal
  piano sintetico e dal criterio, non da un campione esterno.
- Un `p` stampato come `0.000000` è un valore arrotondato, non una prova di
  probabilità nulla.

## Riproduzione

```sh
cargo run --release --locked --bin l6_experiment
```

L'harness riscrive `runs/l6/` e `reports/l6_esperimento.md`. Il confronto va
letto insieme a questa nota; il verdetto non è una firma di accettazione.
