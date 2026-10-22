**Ariadne — L5_SPEC v0.1**

**1. Perimetro**
- L5 Sociale: modelli degli altri e teoria della mente — rappresentazioni predittive di altre istanze, non credenze magiche.
- Ogni modello dell'altro è una vista derivata: l'altro è osservabile solo attraverso i SENSE che entra nel mio log.
- Le sue decisioni sono eventi in coda a L0: replayabili come tutto il resto.

**2. Input**
- Log L0 e viste L1–L4, in sola lettura. Scritture solo eventi propri: MODEL_UPDATE, PREDICT.

**3. Costruzione del modello**
- Ogni altro agente ha un id canonico deterministico, come i nodi L1.
- Il modello è un insieme di parametri statistici aggiornati solo dagli eventi SENSE che lo riguardano: stesso log → stesso modello dell'altro, byte per byte.

**4. Predizione e correzione**
- PREDICT emette una previsione sul comportamento atteso, citando le seq delle osservazioni usate: la teoria della mente con bibliografia.
- Ogni disaccordo con l'osservato genera un MODEL_UPDATE di correzione: il modello sbaglia, il log non sbaglia mai.

**5. Frontiera intersoggettiva**
- L'altro non scrive mai direttamente nel mio log: tutto ciò che fa esiste per me solo come evento percepito — niente canali nascosti.
- I modelli degli altri possono riferirsi a terzi: la mente dell'altro che modella una terza mente, sempre come vista, mai come accesso.

**6. Criteri di uscita (checklist)**
- [ ] Replay: stesso log → stessi modelli, byte per byte.
- [ ] Zero scritture dirette da altri dimostrate.
- [ ] Ogni predizione ancorata alle seq delle osservazioni.
- [ ] Test di divergenza: osservazioni diverse → modelli diversi.
- [ ] Test di teoria della mente di secondo ordine superato.
