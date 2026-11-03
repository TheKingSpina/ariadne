# Domande per l'emendamento dello scenario integrato L0→L6

**Stato: aperto, nessuna risposta o firma apposta.** Le opzioni sono
alternative da sottoporre a kingspina, non raccomandazioni. Il documento
organizza le domande di `bozza_emendamento.md`, basata sulla revisione
`493041addcd00df2cddfab65cbdd7e4c609be2ad`; per fonti e conflitti completi
si rimanda alla bozza. Fino al completamento e alla firma non costituisce
autorizzazione a eseguire lo scenario integrato.

## L0–L1 · Log e dati dello scenario

1. **Seed e genesi — TBD-01.** Quale seed usiamo per il log principale?
   Un seed ammesso da L0, un seed collegato a L6, oppure un'altra regola?

2. **Eventi e tick — TBD-02.** Vuoi specificare l'intera sequenza degli
   eventi e dei tick, oppure una regola deterministica che la generi?

3. **Ciclo L0 e task L6 — TBD-C2.** L0 descrive `SENSE → ACTION → STATE`;
   L6 descrive task `SENSE → ACTION → SENSE`. Come si relazionano nello
   scenario integrato: perimetri distinti, emendamento esplicito o altra
   risoluzione?

4. **Estrazioni L1 — TBD-03.** Quali nodi e archi usiamo? Occorrono `kind`,
   `key`, endpoint, `rel` e ordine, oppure una regola deterministica che li
   definisca.

## L2–L3 · Selezione e memoria

5. **Configurazione L2 — TBD-04.** Quali valori e quale posizione nel log
   per `k` e `tau`? Vuoi una sola configurazione o configurazioni successive?

6. **Copertura L2 — TBD-05.** Quali casi devono essere esercitati: `select`,
   apertura e chiusura della finestra, emissione di `summary`? Puoi indicare
   un sottoinsieme oppure tutti, con le condizioni attese.

7. **Configurazione L3 — TBD-06.** Quali valori e quale posizione nel log
   per soglia `T` e half-life `H`? Una sola configurazione o successive?

8. **Transizioni L3 — TBD-07.** Quali transizioni vuoi esercitare tra
   `promote`, `archive` e `recall`? Per il richiamo occorre definire anche
   la sequela di attivazione.

## L4–L5 · Sé e modello sociale

9. **Revisioni L4 — TBD-08.** Quali eventi rilevanti devono essere narrati
   e quali coppie `narrate`/`snapshot` verificate? Puoi indicare gli eventi
   o una regola per sceglierli.

10. **Uguaglianza del documento L4 — TBD-C1.** La checklist richiede un
    documento identico per differenze irrilevanti, ma il documento include
    `head_hash`, che può cambiare. Vuoi chiarire quale parte confrontare,
    emendare la checklist o scegliere un'altra risoluzione?

11. **Osservazioni L5 — TBD-09.** Quali label degli altri agenti,
    comportamenti e ordine delle osservazioni usiamo? In alternativa,
    quale regola deterministica li genera?

12. **Copertura L5 — TBD-10.** Quali casi devono essere controllati:
    previsione, osservazione concordante o discordante, `model_update` e
    secondo ordine? Quali output sono attesi?

## Collegamento complessivo e L6

13. **Topologia dei log — TBD-11.** Come colleghiamo i livelli? Per esempio,
    un log principale L0–L5 più i log sperimentali L6, oppure un'altra
    topologia esplicitamente definita?

14. **Contributo L1–L5 — TBD-C4.** La prova L6 deve usare obbligatoriamente
    le viste L1–L5, oppure mantenere il protocollo L6.4 che usa la storia del
    log e rende facoltative quelle API? Quale conclusione sarà ammessa?

15. **Benchmark casuale — TBD-C3.** La SPEC L6 indica `0,50`; la legge L6
    indica `333` milli per tre classi. Quale formulazione e quale perimetro
    devono valere nella prova?

16. **Timeout e log — TBD-C5.** Confermi il limite di 5 secondi per coppia?
    Come devono relazionarsi i log dei singoli bracci con l'unico log
    verificabile della dimostrazione?

17. **Criteri di PASS/FAIL — TBD-12.** Quali valori attesi e invarianti
    devono essere verificati a ogni passaggio L0–L6? Quali differenze o
    errori fanno fallire la prova?

18. **Replay delle decisioni L6 — TBD-13.** Come deve essere ricalcolata e
    confrontata l'intera sequenza di risposte e reward per ciascun braccio?

## Ambiente e gate

19. **Ambiente — TBD-14.** Quali versioni e digest dell'immagine devono
    essere fissati per la prova firmata? Come e dove vanno conservati log e
    altri artefatti? L'immagine usata nella verifica recente era
    `rust:1.85.1` con digest
    `sha256:e51d0265072d2d9d5d320f6a44dde6b9ef13653b035098febd68cce8fa7c0bc4`;
    è un fatto dell'esecuzione, non una scelta già firmata per questa prova.

20. **Gate positivo e negativo — TBD-15.** Quali input devono rappresentare
    una run valida e una non conforme? Quali output dimostrano
    rispettivamente che passa e che viene bloccata?

Le risposte possono essere fornite a gruppi, mantenendo la numerazione.
