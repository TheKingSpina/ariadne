# Uso e superficie pubblica

## Prerequisiti e verifiche

Servono Rust e Cargo compatibili con l'edizione 2021 del crate. Dalla radice:

```sh
cargo test
cargo test --release
```

La compilazione e i test usano la directory `target/`; non è necessario
eseguire l'esperimento completo L6 per eseguire la suite.

## Elementi pubblici

`src/lib.rs` riesporta `Log`, `Event`, `EventType`, `Payload`, `Value`,
`Error` e altre primitive L0. I moduli pubblici comprendono `cycle`, `prng`,
`state`, `view`, `selection`, `memory`, `identity`, `social`, `valuation`.
I moduli interni `event`, `log`, `payload` non vanno importati direttamente:
si usano le riesportazioni alla radice del crate.

Punti di accesso principali, da consultare nelle firme Rust per dettagli su
tipi, errori e precondizioni:

| Livello | Operazioni principali |
| --- | --- |
| L0 | `Log::genesis`, `append`, `verify`, `to_jsonl`, `from_jsonl`; `cycle::run_cycle` |
| L1 | `view::build` |
| L2 | `selection::build_active`, `evaluate`, `encode` |
| L3 | `memory::build_memory`, `evaluate`, `encode`, `encode_recall` |
| L4 | `identity::build_self`, `evaluate_self`, `encode4` |
| L5 | `social::build_social`, `predict_social`, `evaluate_social`, `encode5`, `encode5_predict` |
| L6 | `valuation::generate_pair`, `run_arm`, `run_pair`, `fold_run`, `wilcoxon_one_sided_greater` |

Le funzioni di valutazione dei livelli non equivalgono automaticamente a
un'append: verificare il contratto del singolo livello, codificare le
decisioni e apporle tramite le API L0 quando richiesto.

## Binari diagnostici

```sh
cargo run --bin l0_replay -- --seed 1042 --ticks 10
```

`l0_replay` produce JSONL su stdout. Consultare `src/bin/l0_replay.rs` per
il contratto esatto degli argomenti e dell'output.

```sh
cargo run --bin l6_experiment
```

**Attenzione:** il secondo comando scrive e sovrascrive i file per seed in
`runs/l6/` e `reports/l6_esperimento.md`, relativamente alla directory
corrente. `runs/` è ignorata da Git; il report è tracciato. Preservare gli
artefatti esistenti prima di rieseguirlo. Il programma misura anche il tempo
con wall-clock e il report contiene dati di esecuzione variabili.

Non assumere che i binari diagnostici costituiscano già un daemon o una
distribuzione pronta per il deploy: tali forme operative non sono dimostrate
dai comandi sopra.
