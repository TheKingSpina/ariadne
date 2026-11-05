# Ablazione L6 — efficacia della memoria specifica per cue
**Data:** 5 novembre 2026.  
**Piani:** `preparazione_stress/candidati_l6/piani.jsonl`, 100 seed.  
**Esecuzione:** `python3 preparazione_stress/ablation_l6.py`.

## Politiche
- `controllo0`: risponde sempre 0 (controllo L6).
- `memoria_cue`: maggioranza dei (cue, esito) già visti (memoria L6).
- `memoria_globale`: maggioranza globale, ignora il cue.
- `memoria_mescolata`: stessa memoria con cue permutati (controllo negativo).
- `casuale`: scelta deterministica pseudo-casuale.

## Risultati sui 40 task di test
| Politica | Media (0–1000) | Mediana | Δ vs controllo0 | IC 95% | p (una coda) |
| --- | ---: | ---: | ---: | --- | ---: |
| `controllo0` | 335.2 | 325.0 | +0.0 | [+0.0, +0.0] | 1 |
| `memoria_cue` | 1000.0 | 1000.0 | +664.8 | [+660.2, +669.2] | 1.98e-18 |
| `memoria_globale` | 311.2 | 325.0 | -24.0 | [-37.5, -11.8] | 0.999 |
| `memoria_mescolata` | 328.5 | 250.0 | -6.8 | [-50.2, +41.0] | 0.995 |
| `casuale` | 358.5 | 350.0 | +23.2 | [+7.2, +39.0] | 0.00537 |

## Memoria specifica contro alternative
| Confronto | Δ medio | IC 95% | p (una coda) |
| --- | ---: | --- | ---: |
| `memoria_cue` − `memoria_globale` | +688.8 | [+677.5, +701.0] | 1.98e-18 |
| `memoria_cue` − `memoria_mescolata` | +671.5 | [+624.2, +714.0] | 4.14e-17 |
| `memoria_cue` − `casuale` | +641.5 | [+625.5, +658.0] | 1.98e-18 |

## Curva di apprendimento
Accuratezza media sui 40 test dopo N task di apprendimento:

| N | Accuratezza |
| ---: | ---: |
| 0 | 0.335 |
| 10 | 0.696 |
| 20 | 0.870 |
| 40 | 0.980 |
| 80 | 1.000 |

## Lettura
Il braccio `memoria_cue` supera il controllo e le alternative che non usano il cue corretto. La memoria mescolata e la scelta casuale restano vicine al caso, quindi l'effetto non è spiegato dalla distribuzione dei task né da una preferenza della politica.

**Conclusione.** Nel corpus L6, la memoria specifica per cue porta l'accuratezza media da 335.2 a 1000.0 milli-reward per run (Δ +664.8, IC 95% [+660.2, +669.2], p = 1.98e-18). Memoria globale, memoria mescolata e scelta casuale restano vicine al caso. L'effetto è quindi attribuibile alla memoria del cue corretto, non alla distribuzione dei task.

**Limiti.** I task sono sintetici e generati dallo stesso piano preregistrato di L6. L'esperimento dimostra l'efficacia della memoria specifica per cue in questo corpus, non l'efficacia di L1–L5 su dati reali, non l'intelligenza generale e non la verità delle dichiarazioni. Le politiche di controllo non sono un'implementazione completa di Ariadne.
