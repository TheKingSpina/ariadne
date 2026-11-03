# Ariadne — documentazione di riferimento

Questa cartella descrive il repository così come implementato alla revisione
`94e6eb6`. È una guida tecnica, **non sostituisce né modifica** le SPEC, le
leggi firmate o i report storici. In caso di divergenza, consultare le fonti
originali e verificare il comportamento del codice; non considerare questa
guida un emendamento normativo.

## Indice

1. [Architettura e livelli L0–L6](architettura.md)
2. [Uso, API e comandi](uso.md)
3. [Test, esperimento L6 e limiti delle verifiche](verifiche.md)
4. [Fonti, decisioni e governance](fonti_governance.md)

## Stato in breve

- La suite esistente comprende **182 test**, passati in debug e release
  nell'ultima esecuzione riportata in questa sessione.
- I livelli hanno test propri e alcune interazioni sono testate; **non è
  presente un singolo test end-to-end che percorra L0→L6**. La pipeline
  integrata esplicita arriva a L0→L1→L2.
- L6 implementato è un **esperimento di valutazione**. Non va confuso con
  una funzione già implementata di ricalibrazione automatica dei livelli.
- Il report sperimentale storico L6 va interpretato entro i limiti descritti
  in [verifiche.md](verifiche.md).
