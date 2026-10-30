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
- **L0 Substrato — accettato** (22/10/2026, `a02b66a`): replay bit-perfetto a due processi, 80 test verdi debug+release, audit del reviewer senza violazioni note (chiuso con la firma di `spec/L0_SPEC_v0.3.md`).
- **L1 Vista — accettato** (23/10/2026): test-first `c8c69d7` → implementazione `87bc9e9`; 11 test L1 / 91 totali; addendum F1 firmato (`d082209`): `nodes`/`edges` come chiavi stringa con array JSON canonico interno.
- **L2 Selezione — accettato** (24/10/2026, con pipeline di integrazione L0→L1→L2): legge L2.1–L2.26 (`notes/L2_legge_selezione.md`), 105 test totali verdi (14 L2), gate tester e mini-audit reviewer conformi. Eventi L2 = `ACTION` con payload `l2 = config | select | summary | barrier`; set attivo a budget k, salienza intera `2^24 >> …`, finestra offline con Summary autorevole.
- **Legge attuale in federa consolidata** (26/10/2026, `f92bde3`): la lettura canonica di ciascun livello implementato è un solo file — `spec/L0_SPEC_v0.4.md`, `spec/L1_SPEC_v0.3.md`, `spec/L2_SPEC_v0.3.md`. Le versioni precedenti e le note di legge restano reperti storici.
- **Prossimo passo**: L3 Memoria lunga (triage della SPEC, poi ciclo test-first consolidato).

**Stack e deploy**
- **Linguaggio L0**: Rust — fold deterministica, SHA-256, no-GC per latenza prevedibile e borrow checker contro l'undefined behavior.
- **Distribuzione**: binario statico unico, zero runtime.
- **Modello deploy**: local-first sul Mac mini, col substrato incorporato; eventuale modalità daemon server come estensione futura, mai come prerequisito.
- **Memoria di sviluppo**: Mnemosyne come registro dei ricordi di progetto, con approvazione in governance.
