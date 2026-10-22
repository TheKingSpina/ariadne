**Ariadne — L1_SPEC v0.1**

**1. Perimetro**
- L1 è una vista derivata: il grafo è una proiezione del log, mai una fonte separata.
- Cicli ammessi e liberi — l'aciclicità è legge solo in L0, qui il grafo può collegarsi come vuole.
- Zero scritture: L1 non tocca mai il log.

**2. Input**
- Solo il log L0 (eventi SENSE/ACTION/STATE), letto in ordine di seq. Nient'altro.

**3. Nodi**
- Nodo = entità estratta dal payload: {id, tipo, origine: seq}.
- id canonico e deterministico dall'hash del contenuto: stesso payload → stesso id, ovunque.

**4. Archi**
- Arco = relazione con prova: {sorgente, destinazione, tipo, seq_di_origine}.
- Un arco esiste solo se c'è un evento che lo prova — niente archi "a memoria diScripts", la provenienza è obbligatoria.

**5. Ricostruzione**
- Replay del log → estrazione deterministica: stesso log → stesso grafo, byte per byte.
- Ogni vista è versionata e ancorata all'hash di testa del log che l'ha generata: il grafo porta con sé la sua verità.

**6. Criteri di uscita (checklist)**
- [ ] Ricostruzione deterministica su due esecuzioni indipendenti.
- [ ] Ogni arco tracciato alla sua seq di origine.
- [ ] Vista ancorata all'hash di testa del log.
- [ ] Zero scritture al log dimostrate.
- [ ] Test con grafo ciclico valido superato.
