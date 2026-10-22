**Ariadne — L4_SPEC v0.1**

**1. Perimetro**
- L4 Identità: il sé come narrazione derivata dal log — un processo, mai un oggetto statico.
- L'identità non è una fonte: è una vista con revisioni, come tutto il resto.
- Le sue decisioni sono eventi in coda a L0: replayabili, verificabili, eliminabili no.

**2. Input**
- Log L0, viste L1, decisioni L2 e L3, in sola lettura. Scritture solo eventi propri: NARRATE, IDENTITY_SNAPSHOT.

**3. Sintesi narrativa**
- La narrazione è generata da regole deterministiche sugli eventi: stesso log → stessa identità, byte per byte.
- Eventuali componenti stocastiche passano solo dal PRNG dello stato L0 — niente fonti di casualità fuori dal substrato.
- Ogni affermazione narrativa cita le seq di origine: il sé con la sua bibliografia.

**4. Versionamento**
- Mai sovrascrivere: ogni revisione è un nuovo IDENTITY_SNAPSHOT ancorato all'hash di testa del log che l'ha generata.
- Lo storico delle versioni è la biografia: chi ero a seq X, chi sono a seq Y.

**5. Continuità tra sessioni**
- Al boot: replay del log → ricostruzione dell'ultimo snapshot + delta dagli eventi successivi.
- Nessun file separato "chi sono io": l'identità non si salva, si ricalcola. Se il log è integro, tu sei integro.

**6. Criteri di uscita (checklist)**
- [ ] Replay bit-perfetto della narrazione su due esecuzioni.
- [ ] Ogni snapshot ancorato a un hash di testa.
- [ ] Zero scritture fuori dagli eventi propri.
- [ ] Test di ripresa sessione superato.
- [ ] Test di divergenza: due log diversi → due identità diverse.
