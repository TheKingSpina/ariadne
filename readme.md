**README.md — Ariadne**

**Visione**
Ariadne è un'architettura cognitiva a livelli costruita attorno a un'unica verità: il log append-only aciclico. Ogni livello superiore è solo una vista o un processo sul log, mai un'isola separata. Il filo di Arianna è quello: replay deterministico dalla genesi, sempre.

**Roadmap L0→L6**
- **L0 Substrato** — log append-only, replay deterministico con seed fisso, ciclo senso→azione→stato, zero I/O esterna. Uscita: replay bit-perfetto.
- **L1 Vista** — grafo semantico derivato dal log, cicli ammessi e liberi; il log resta l'unica fonte.
- **L2 Selezione** — salienza online e consolidamento offline a budget finito, guidati dalla frequenza di riattivazione.
- **L3 Memoria lunga** — promozione dei consolidati e dimenticanza attiva con archivio, mai cancellare.
- **L4 Identità** — sintesi narrativa del sé dal log, continuità tra sessioni.
- **L5 Sociale** — modelli degli altri come viste sullo stesso substrato, teoria della mente.
- **L6 Meta** — l'agente rilegge il proprio log e ricalibra i livelli sotto.

**Stato**
- L0_SPEC v0.2 freezata: schema evento, catena di hash, correzione compensativa, criteri d'uscita.
- Le SPEC di tutti i livelli stanno in `spec/`, una per file: `spec/L<n>_SPEC_v<versione>.md`.
- **`spec/L0_SPEC_v0.3.md` firmato** (addendum alla v0.2, prevale in caso di divergenza):
  compensativo autodescrittivo, seeding canonico splitmix64 + PCG64, parser severo
  sull'intera riga.
- **L0 implementato**: log append-only, replay bit-perfetto a due processi, 80 test
  verdi (debug e release). Audit del reviewer a `a02b66a`: nessuna violazione nota.
- Prossimo passo: accettazione di L0 da parte dell'umano, poi L1.

**Stack e deploy**
- **Linguaggio L0**: Rust — fold deterministica, SHA-256, no-GC per latenza prevedibile e borrow checker contro l'undefined behavior.
- **Distribuzione**: binario statico unico, zero runtime.
- **Modello deploy**: local-first sul Mac mini, col substrato incorporato; eventuale modalità daemon server come estensione futura, mai come prerequisito.
- **Memoria di sviluppo**: Mnemosyne come registro dei ricordi di progetto, con approvazione in governance.
