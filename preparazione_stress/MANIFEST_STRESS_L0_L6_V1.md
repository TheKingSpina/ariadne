# Manifest stress test L0–L6 — v1 eseguibile

**Data:** 5 novembre 2026.
**Stato:** v1, compilato e usato per la run. Sostituisce, per questa
esecuzione, la bozza `manifest_stress_L0_L6_bozza.md`, che resta invariata.
**Autorizzazione:** l'umano ha chiesto di risolvere le ambiguità e di
eseguire lo stress test. Le decisioni normative sono in
`AUX_V2_PROFILO_IMPLEMENTATO.md`, sezione «Decisioni ratificate».

## Provenienza

| Voce | Valore |
| --- | --- |
| Scenario | `stress_l0_l6_v1` |
| Seed L0 | `1042` |
| Immagine / runtime | host locale e Docker per la dashboard; run su host |
| Commit di esecuzione | registrato in `reports/stress_L0_L6_v1.md` |
| Log principale | `runs/stress_l0_l6/main.jsonl` (non committato) |
| Fixture L5 | `runs/stress_l0_l6/l5-*.json` (non committate) |
| Report | `reports/stress_L0_L6_v1.md` |
| Esecutore | `preparazione_stress/esegui_stress_l0_l6.py` |

## Parametri

- L0: un ciclo `SENSE → ACTION → STATE` per tick, PRNG PCG64 seed 1042,
  genesi senza draw.
- L2: `k=3`, `tau=5` **in seq**.
- L3: `T=1`, `H=10` tick.
- L5: fixture esterne, ancora GENESIS `seq=0`; l'ordine 1 non aggiorna i
  contatori.
- L6: esperimento appaiato separato, 100 coppie seed `1000..=1099`.

## Sequenza L0–L5

Gli AUX stanno tra cicli completi, al tick dell'ultimo `STATE`. La tabella
riporta le seq effettive della run; `runs/stress_l0_l6/main.jsonl` è la fonte
byte-esatta.

| Seq | Tick | Evento | Contenuto |
| ---: | ---: | --- | --- |
| 0 | 0 | GENESIS | seed 1042 |
| 1 | 0 | L2/config | `k=3`, `tau=5` |
| 2 | 0 | L3/config | `H=10`, `T=1` |
| 3 | 0 | L5/observation | `l5-genesis-01`, alpha=1 |
| 4 | 0 | L5/predict | alpha, `previsto=1`, `at=3` |
| 5 | 0 | L5/observation | `l5-genesis-01`, accordo |
| 6 | 0 | L5/observation | `l5-beta-01`, beta=2 |
| 7 | 0 | L5/predict | alpha, `previsto=1`, `at=5` |
| 8 | 0 | L5/observation | `l5-alpha-02`, alpha=2, disaccordo |
| 9 | 0 | L5/model_update | `at=8`, `previsto=1`, `osservato=2` |
| 10 | 0 | L5/observation | `l5-perspective-01`, ordine 1 |
| 11–13 | 1 | ciclo L0 | SENSE/ACTION/STATE |
| 14 | 1 | L1/extract | 4 nodi, arco alfa→beta |
| 15–16 | 1 | L4 | narrazione e snapshot dell'estrazione |
| 17–20 | 1 | L2 | open, select, close, summary; nessuna espulsione |
| 21–24 | 1 | L4 | revisioni di select e summary |
| 25–27 | 2 | ciclo L0 | |
| 28 | 2 | L1/extract | alfa |
| 29–32 | 2 | L2 | open, select, close, summary; **gamma espulso** (`expulsions=[11]`) |
| 33–36 | 2 | L4 | revisioni di select e summary |
| 37 | 2 | L3/promote | alpha (attivazioni ai tick 1 e 2) |
| 38–39 | 2 | L4 | revisione del promote |
| 40–60 | 3–6 | L0 + L1/L2 | tre cicli, poi estrazione di alpha al tick 6; nessuna nuova espulsione |
| 61–810 | 7–256 | 250 cicli L0 | nessun AUX; il ciclo del tick 256 occupa 808–810 |
| 811 | 256 | L1/extract | beta |
| 812–815 | 256 | L2 | select e summary; nessuna espulsione |
| 816–819 | 256 | L4 | revisioni |
| 820 | 256 | L3/archive | alpha (score 0) |
| 821–822 | 256 | L4 | revisione |
| 823 | 256 | L3/archive | gamma (score 0) |
| 824–825 | 256 | L4 | revisione |
| 826 | 256 | L3/archive | delta (score 0) |
| 827–828 | 256 | L4 | revisione |
| 829 | 256 | L3/promote | beta (2 attivazioni) |
| 830–831 | 256 | L4 | revisione |
| 832–834 | 257 | ciclo L0 | |
| 835 | 257 | L1/extract | alfa |
| 836–839 | 257 | L2 | select e summary |
| 840–843 | 257 | L4 | revisioni |
| 844 | 257 | L3/recall | alpha (fonte 835 > archive 820) |
| 845–846 | 257 | L4 | revisione |

Totale: **847 eventi**, hash di testa
`8d114b99ddacea0eb6960e27a7824c1269d4848aac9e8d83699d82084f5ea8c4`,
SHA-256 del JSONL
`43accb3a73fc7d11e22e66db895785e2af2a7c61556f93c7174aac6e39f60168`.

## Fixture L5

| ID | SHA-256 |
| --- | --- |
| `l5-genesis-01` | `c66da3195548417fcbcaea078e8bab66b98b539a68ddde6011223800c117fbac` |
| `l5-beta-01` | `b0a0fdb92c0dfbe7bfaf9cd56bbbd78e4cfef58378aeacc4037036a975fa77c5` |
| `l5-alpha-02` | `7d4d7aa8059739b6b4c09c15a565254a05b208e662f51b3ee31f81307688f099` |
| `l5-perspective-01` | `9a5b3c1cd79054c46f205684e034fc3c96248cf8dc58843f7fd793c76386cc79` |

I byte sono quelli del manifest proposto, senza newline finale. Le fixture
positive e il caso negativo con digest errato sono già coperti dai test.

## Oracoli

- Catena, canonicità, `prev_hash` e `hash`: verifica Python indipendente in
  `preparazione_stress/esegui_stress_l0_l6.py`, che usa il parser e il
  sigillo di `aux_v2_oracle.py`.
- Fixture: digest e corrispondenza `data`/`provenance.external`.
- L5: posizioni 3–10, classi, accordo/disaccordo e `model_update`.
- L2: cinque summary; la seconda espelle gamma con origine 11.
- L3: promote alpha/beta, archive alpha/gamma/delta nell'ordine
  `archive → promote`, recall alpha.
- Proiezioni `graph`, `active`, `memory`, `self`: ricostruite dal backend
  Rust; sono un controllo funzionale, non un oracolo indipendente.
- L6: harness separato con replay proprio e Wilcoxon.

## Esecuzione

```sh
cargo build --release --locked --bins
python3 preparazione_stress/esegui_stress_l0_l6.py
```

L'esecutore rigenera `runs/stress_l0_l6/`, esegue L6, verifica e scrive
`reports/stress_L0_L6_v1.md`. `runs/` è ignorato da Git; il report sì.

## Limiti

- Scenario sintetico definito oggi; non è una preregistrazione approvata
  prima dei risultati.
- La semantica L1–L4 di questo log non è rigiocata da una seconda
  implementazione completa: è coperta dall'oracolo indipendente su un altro
  scenario. Il manifest dichiara questa asimmetria.
- L6 non dimostra l'efficacia causale di L1–L5 né la verità delle
  dichiarazioni.
- Il risultato vale per questo corpus e questo criterio.
