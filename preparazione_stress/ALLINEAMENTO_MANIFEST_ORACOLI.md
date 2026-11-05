# Allineamento manifest e oracoli allo stress test

**Data:** 5 novembre 2026.
**Stato:** analisi preliminare. Nessuna modifica a
`manifest_stress_L0_L6_bozza.md` o `oracoli_stress_L0_L6_bozza.md`; nessuna
esecuzione dello stress test L0–L6.

> **Aggiornamento del 5 novembre 2026.** Le cinque ambiguità elencate sotto
> sono state ratificate e incorporate in
> `AUX_V2_PROFILO_IMPLEMENTATO.md`, sezione «Decisioni ratificate». Il
> documento resta la guida per compilare manifest e oracoli; le decisioni
> non trasformano le bozze in documenti eseguibili. L'esperimento L6 è stato
> eseguito separatamente con `src/bin/l6_experiment.rs`; il suo report non è
> lo stress test L0–L6 e non dimostra l'efficacia causale di L1–L5.

## Esito in breve

I due documenti sono **dichiaratamente bozze non eseguibili** e contengono
molti `TBD`. Non sono “quasi pronti”: mancano sequenza, tick, seq, payload
canonici, schema degli artefatti e commit. Allinearli al profilo implementato
richiede decisioni normative, non solo compilazione di valori.

L'oracolo indipendente `preparazione_stress/aux_v2_oracle.py` fornisce già un
esempio completo e verificabile per uno scenario piccolo (121 eventi, L1–L4),
ma **non sostituisce** il manifest dello stress test: copre un solo scenario,
non L5/L6, e non è approvato come preregistrazione.

## Lacune del manifest rispetto al profilo implementato

| Area | Stato nella bozza | Cosa serve per allinearla |
| --- | --- | --- |
| Collocazione eventi L2–L5 | proposta AUX tra cicli, da ratificare | decisione normativa: AUX dopo il `STATE` del ciclo, tick corrente; nessun draw consumato |
| Sequenza L1 | nodi e arco proposti, seq `TBD` | fissare tick, seq, ordine di `data.nodes`/`data.edges`, fixture e revisioni L4 |
| Configurazioni L2/L3 | `k=3, tau=5, T=1, H=10` | fissare le seq dei due eventi di config e la loro collocazione |
| Ciclo L0 | “un ciclo valido per tick”, schema non fissato | usare il ciclo reale `SENSE→ACTION→STATE` con seed 1042 e tick espliciti |
| Finestra L2 | “ordine logico proposto”, non eseguibile | preregistrare `barrier open`, `select`, `barrier close`, `summary` e le quattro revisioni L4 con seq esatte |
| L3 promote/archive/recall | tick e stato attivazioni `TBD` | calcolare le soglie con `T` e `H` reali e indicare le seq delle transizioni |
| L4 | cutoff, `at`, testi e hash `TBD` | fissare i testi deterministici e i `self_hash` attesi, o dichiarare che si verificano per ricalcolo indipendente |
| L5 | sequenza proposta, `at/sources/tick/seq` `TBD` | completare osservazioni, previsioni, accordo/disaccordo e `model_update` |
| L6 | 100 coppie, 120 task, timeout | resta un esperimento separato; non è necessario per il replay L1–L4 |
| Fixture del gate | byte e output `TBD` | riusare il formato dei negativi dell'oracolo, ma con payload dello scenario approvato |
| Schema artefatti | campi e canonizzazione `TBD` | definire JSON canonico, digest e riferimenti al manifest |
| Commit | non fissato | fissare il commit **dopo** l'emendamento e la revisione, non prima |

## Lacune degli oracoli

| Area | Stato | Cosa serve |
| --- | --- | --- |
| Oracolo L1–L4 | non esiste come codice | riusare l'architettura dell'oracolo indipendente, con fixture preregistrato separato |
| Oracolo L5 | formule e valori candidati | definire identità, classi, `model_update` e sorgenti in modo verificabile |
| Oracolo L6 | procedura proposta | definire replay delle azioni, reward e criterio di confronto; resta separato |
| Gate | esiti e motivi `TBD` | codificare gli errori attesi e la prova che nessuna valutazione sia partita |
| Indipendenza | dichiarata | dimostrare che l'oracolo non importa la logica di produzione e che il fixture è congelato |

## Ambiguità da ratificare prima di compilare i valori

1. **Ordine L3 nella stessa classe**: il profilo non fissa il comparatore tra
   più `archive` o più `promote` nello stesso giudizio.
2. **Formula L3**: l'implementazione adotta l'adattamento di L2 (`2^24`,
   `tau→H`, `seq→tick`, soglia 25); il profilo non lo scrive letteralmente.
3. **Insieme valutato da L3**: tutte le identità con attivazioni, oppure solo
   quelle attivate dall'ultima estrazione.
4. **Nuovi archi e L4**: l'implementazione narra anche i nuovi archi; il
   profilo parla di “prime dichiarazioni”.
5. **L5 e collocazione AUX**: la bozza coordinata propone AUX fuori dalle
   triplette e fixture esterna; va ratificato come legge, non dato per assunto.

Senza queste decisioni, compilare il manifest significherebbe inventare
semantica. L'oracolo indipendente ha evitato i pareggi L3 nello scenario
principale proprio per non imporre una scelta non ratificata.

## Ordine proposto

1. Ratificare le cinque ambiguità sopra.
2. Fissare il profilo e il commit del codice; aggiornare
   `AUX_V2_PROFILO_IMPLEMENTATO.md` se le decisioni lo cambiano.
3. Compilare il manifest con tick, seq, payload e testi esatti; niente `TBD`.
4. Scrivere l'oracolo L1–L4 separato, con fixture congelato e digest.
5. Aggiungere L5 solo dopo la ratifica della collocazione AUX.
6. Preparare il gate positivo/negativo e la prova che il negativo non avvia
   valutazioni.
7. Revisione indipendente di manifest e oracoli, poi esecuzione.
8. L6 resta l'ultimo passo e non dimostra l'efficacia causale di L1–L5.

## Cosa è già disponibile

- `preparazione_stress/aux_v2_oracle.py`: generatore/verificatore indipendente
  per lo scenario `small`, con 121 eventi e 11 negativi.
- `tests/aux_v2_independent.rs`: confronto byte per byte con il fixture
  congelato.
- `preparazione_stress/ORACOLO_INDIPENDENTE.md`: regole, ambiguità e limiti.
- `AUX_V2_PROFILO_IMPLEMENTATO.md`: contratto del profilo effettivamente
  implementato.

Questi artefatti sono un modello di metodo, non il manifest approvato.
