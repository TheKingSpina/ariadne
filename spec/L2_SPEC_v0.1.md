**Ariadne — L2_SPEC v0.1**

**1. Perimetro**
- L2 Selezione: decide cosa resta nello spazio di lavoro attivo, a budget finito.
- Deterministica per contratto: stesso log → stesse decisioni di selezione, byte per byte.
- Le decisioni sono eventi: ogni entrata/uscita dal set attivo viene scritta come evento nel log L0 — così la selezione stessa è replayabile, niente magia fuori dal substrato.

**2. Input**
- Il log L0 e le viste L1: li legge, non li tocca mai. Le sue uniche scritture sono i propri eventi di selezione in coda al log.

**3. Salienza online**
- Punteggio = funzione deterministica di: frequenza di riattivazione, recency misurata in tick logici (wall-clock vietato, come in L0), peso degli archi L1 che toccano l'elemento.
- Budget: massimo K elementi attivi, K parametro esplicito dell'evento genesi; oltre budget esce il meno saliente, pari merito = seq minore vince.

**4. Consolidamento offline**
- Gira solo durante inattività, delimitato da un marker esplicito nel log (evento IDLE_BARRIER).
- Comprime gli elementi espulsi in sintesi che rientrano nel log come eventi: mai cancellare, ridurre.

**5. Criteri di uscita (checklist)**
- [ ] Replay: stesso log → stesso set attivo.
- [ ] Budget rispettato a ogni tick, test sotto carico.
- [ ] Zero wall-clock dimostrato.
- [ ] Ogni decisione di selezione ancorata a una seq di origine.
- [ ] Consolidamento offline riprodotto identico in replay.
