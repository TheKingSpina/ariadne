# Stress test L0–L6 — manifest v1

**Data:** 5 novembre 2026.
**Commit verificato:** `e5485225b1c26b2c70b36ecd4d8e2384c194d3b8` (working tree può contenere modifiche non committate).
**Scenario:** `stress_l0_l6_v1`, seed 1042.
**Comando:** `python3 preparazione_stress/esegui_stress_l0_l6.py`.

## Parametri

- L0: ciclo `SENSE → ACTION → STATE`, seed 1042.
- L2: `k=3`, `tau=5` in seq.
- L3: `T=1`, `H=10` tick.
- L4: revisioni dopo ogni dichiarazione e transizione.
- L5: quattro fixture esterne, ancora GENESIS `seq=0`, ordine 0 e ordine 1.
- L6: esperimento appaiato separato, 100 coppie.

## L0–L5

| Voce | Valore |
| --- | --- |
| Eventi | 847 |
| Hash di testa | `8d114b99ddacea0eb6960e27a7824c1269d4848aac9e8d83699d82084f5ea8c4` |
| SHA-256 del JSONL | `43accb3a73fc7d11e22e66db895785e2af2a7c61556f93c7174aac6e39f60168` |
| Fixture | 4 |
| L2 summary | 5 |
| L3 promote | alpha, beta |
| L3 archive | alpha, gamma, delta |
| L3 recall | alpha |
| Stato L4 finale | alpha `circuito`, beta `memoria_lunga` |
| Grafo finale | 4 nodi, 1 arco `alfa → beta` |

Verifiche superate: catena hash e canonicità con implementazione Python
indipendente; digest e associazione delle fixture; osservazioni L5, predict e
`model_update`; ordine `archive` prima di `promote`; espulsione di gamma;
proiezioni `graph`, `active`, `memory`, `self` ricostruite dal backend.

## L6

- Coppie: 100 (seed 1000..=1099), nessuna esclusione.
- Violazioni di protocollo: 0.
- Statistica: W = 5050.0, n = 100, p = 0.000000, Δ̄ = 0.6647.
- Verdetto previsto: **SUCCESSO**.
- Report: `reports/l6_esperimento.md`; lettura critica: `reports/L6_LETTURA.md`.

## Limiti

- Lo scenario L0–L5 è sintetico e definito da questo manifest v1, non dal
  vecchio `manifest_stress_L0_L6_bozza.md`.
- La catena e le fixture sono verificate in modo indipendente; la semantica
  L1–L4 è coperta dall'oracolo `aux_v2_small` su uno scenario distinto, non
  da un replay indipendente di questo intero log.
- L6 non dimostra l'efficacia causale di L1–L5 e non autentica fatti o
  persone.
- I dati e le reward sono generati dall'harness; il risultato vale per
  questo corpus e questo criterio.
