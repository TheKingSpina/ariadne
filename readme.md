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
- Prossimo passo: implementazione del substrato L0 e test di replay bit-perfetto.
