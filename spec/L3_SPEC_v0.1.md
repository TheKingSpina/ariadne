**Ariadne — L3_SPEC v0.1**

**1. Perimetro**
- L3 Memoria lunga: promozione, dimenticanza attiva e archivio.
- Deriva tutto dal log: le sue decisioni sono eventi scritti in coda a L0, replayabili come il resto.
- Mai cancellare: la dimenticanza è spostamento, non distruzione.

**2. Input**
- Log L0, viste L1 e decisioni L2, in sola lettura. Le sue uniche scritture sono i propri eventi: PROMOTE, ARCHIVE, RECALL.

**3. Promozione**
- Un elemento consolidato da L2 che supera la soglia T di riattivazioni (parametro esplicito in genesi) diventa memoria lunga: evento PROMOTE con seq di origine.

**4. Dimenticanza attiva**
- Punteggio di decadimento calcolato su tick logici e frequenza di accesso — wall-clock vietato, sempre.
- Sotto soglia: evento ARCHIVE, l'elemento va in archivio marcato con archived_at_seq, resta intero e raggiungibile.

**5. Archivio e richiamo**
- L'archivio è una vista separata ancorata agli hash del log, riproducibile al 100% in replay.
- Un evento RECALL riporta un elemento in circuito con la sua provenienza completa: niente ritorni dalle tombe senza sequela.

**6. Criteri di uscita (checklist)**
- [ ] Replay: stesso log → stesso archivio, byte per byte.
- [ ] Zero cancellazioni dimostrate: nessun contenuto diventa irraggiungibile.
- [ ] Ogni PROMOTE/ARCHIVE/RECALL ancorato a una seq di origine.
- [ ] Decadimento deterministico su tick logici, zero wall-clock.
- [ ] Test di richiamo dall'archivio superato.

