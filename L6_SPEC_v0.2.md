**Ariadne — L6_SPEC v0.2**

**1. Perimetro**
- L6 Valutazione: esperimento appaiato memoria vs controllo sotto regola nascosta stabile.
- Domanda: la memoria accumulata (L0–L5) migliora il reward delle decisioni rispetto a un braccio senza memoria?

**2. Disegno sperimentale**
- Due braccia appaiate: stessa sequenza di task, stesso seed — braccio memoria vs braccio controllo (no memoria).
- Regola nascosta stabile durante tutto il run: individuabile solo accumulando esperienza.
- Cue e test proof: le prove finali verificano la regola che i cue consentivano di inferire — niente richieste non inferibili.

**3. Parametri fissati**
- 100 run, seed pre-fissati 1000–1099 su PCG64/MT19937.
- Timeout per run: 5,0 secondi.
- Log unificato JSONL per ogni run.

**4. Statistica e criterio di successo**
- Wilcoxon appaiato a una coda sulle differenze di reward per run, α = 0,01.
- Successo solo con: p < 0,01, Δ̄ ≥ 0,30, zero violazioni di protocollo o corruzioni d'integrità su tutti i 100 run.
- Benchmark passivi diagnostici registrati: oracolo (reward 1,00) e random uniforme (reward 0,50).

**5. Validità**
- Replay bit-perfetto di ogni run dal suo log.
- Stesso log → stesso esito, ovunque, in locale o daemon.

**6. Criteri di uscita (checklist)**
- [ ] 100 run completati sui seed 1000–1099.
- [ ] Wilcoxon p < 0,01 e Δ̄ ≥ 0,30.
- [ ] Zero violazioni d'integrità su tutto il corpus.
- [ ] Oracolo e random registrati come benchmark passivi.
- [ ] Replay bit-perfetto dimostrato.

