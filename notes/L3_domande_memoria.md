# Triage L3 — domande da risolvere prima dei test

Documento di triage in sola lettura: non modifica né interpreta in via normativa le SPEC vigenti. Le proposte indicate come default sono scelte da sottoporre a decisione; non autorizzano da sole implementazione o deviazione dalle leggi L0–L2.

## Q1 — Dove e come si configura T alla genesi?

- **L3:** §3, riga 12: «soglia T di riattivazioni (parametro esplicito in genesi)».
- **Leggi che toccano il punto:** L0 §4, righe 62–71, sigilla genesi a payload `{prng,seed,state}`; L0 §3, righe 46–60, limita i valori payload a i64/stringhe; L2.1–2.2, righe 42–60, definisce invece config separato `l2="config"`, valido dalla propria seq, senza default; L2.21, righe 48–53, vieta chiavi extra nei payload L2 prescritti.
- **Indeterminato/opzioni:** aggiungere T al payload di genesi contraddice il payload esatto L0 §4; un evento `l3="config"` non è proibito esplicitamente, ma «in genesi» non dice che possa essere un evento successivo; allargare `l2="config"` a T collide con L2.21 (chiave extra) e ne cambia la semantica.
- **Default da deliberare:** preservare la genesi L0 e definire un evento config L3 dedicato, collocato subito dopo la genesi; conseguenza: testano payload/config, seq e validità da una seq esplicita, ma tale scelta non soddisfa letteralmente «in genesi» senza chiarimento formale. Alternativa: emendamento della legge L0; conseguenza: cambia il golden byte/hash della genesi e richiede aggiornare la legge, non solo L3.
- **Tocca:** `log.rs`, `payload.rs`, `selection.rs`.

## Q2 — Qual è il conteggio delle riattivazioni che supera T?

- **L3:** §3, riga 12: «elemento consolidato da L2 che supera la soglia T di riattivazioni».
- **Leggi:** L2.3 e L2.13, righe 64–77, definiscono attivazione/menzione per evento, deduplicata per elemento e seq; L2.20, righe 221–232, espone `build_active` come replay autorevole. Non definiscono riattivazione, finestra di conteggio, inclusività della soglia, né se il conteggio riparta dopo archive/recall.
- **Opzioni:** contare tutte le attivazioni L2 dall’origine; contare solo quelle successive all’ingresso nel set attivo; contare accessi successivi a ciascun recall. Ogni alternativa produce differenti seq di PROMOTE. **Default da deliberare:** un conteggio per evento L0 distinto secondo L2.3, soglia stretta `> T`, persistente e non azzerato da ARCHIVE; conseguenza: testare i confini T−1/T/T+1 e duplicazione nello stesso evento. È un’interpretazione, non fissata dalle leggi.
- **Tocca:** `selection.rs`, `view.rs`.

## Q3 — Formula e parametri del decadimento

- **L3:** §4, riga 15: «Punteggio di decadimento calcolato su tick logici e frequenza di accesso»; riga 16: «Sotto soglia».
- **Leggi:** L0 §1 e §2, righe 19–40, vieta wall-clock e definisce `tick` logico; L2.4, righe 79–82, definisce un punteggio intero basato su seq d’attivazione e `tau`, non su tick o frequenza aggregata. Nessuna legge stabilisce la formula L3, scala, larghezza/tipo, saturazione, soglia, arrotondamento o momento di valutazione.
- **Opzioni:** formula intera a decadimento per tick con contributo accessi, oppure altra funzione intera; usare `seq` non equivale a usare `tick` quando i tick si ripetono. **Default da deliberare:** formula intera esplicitamente approvata, con tipi e overflow dichiarati; non è possibile fissarne una senza intuizione. Una volta scelta, i test byte-per-byte devono includere valori ai confini, tick ripetuti, accessi ripetuti e overflow.
- **Parità:** L2.6, L2.17 e L2.24, righe 89–104, fissano parità su score, origin_seq e id per il set attivo/candidati L2; non dicono esplicitamente che tale ordine governi eviction/ingresso L3. **Default:** riusare l’ordine L2 completo (id lessicograficamente minore prevale); conseguenza: casi test con score e origin_seq pari.
- **Tocca:** `selection.rs`.

## Q4 — Schema, payload e provenienza degli eventi L3

- **L3:** §2, riga 9, nomina PROMOTE, ARCHIVE, RECALL come uniche scritture; §3 riga 12, §4 riga 16, §5 riga 20 e checklist §6 riga 25 richiedono ancoraggio a seq d’origine/`archived_at_seq`.
- **Leggi:** L0 §2–§3, righe 25–60, vincola i tre tipi L0 e payload flat canonico; L2.1 e L2.21, righe 42–53, codifica decisioni come `ACTION` con discriminante `l2`, schema chiuso per tag; L2.18, righe 115–118, definisce `at` come seq di valutazione, non seq di origine; L1 §3/§5, righe 52–60 e 85–91, conserva `origin_seq` su nodi e archi.
- **Indeterminato:** discriminante (es. `l3` in ACTION), nomi/tipi/chiavi esatte dei payload, codifica della lista, seq d’origine di un evento decisionale distinto da `at`, e trattamento di nodi/archi. L0 non stabilisce che L3 possa riusare `l2`; L2.21 vieta `l3` extra negli eventi L2, perciò un payload può essere L3 oppure L2, non entrambi senza decisione normativa.
- **Default da deliberare:** eventi `ACTION` con `l3="promote"|"archive"|"recall"`, payload esatto con id L1 e `origin_seq`; aggiungere `at` solo se si vuole anche la seq di valutazione. Conseguenza: testare encoding canonico, schema chiuso, seq di origine e distinzione `at`/origine. Questo schema è una proposta, non determinato da L0–L2.
- **Tocca:** `selection.rs`, `payload.rs`, eventualmente `event.rs` (solo se si propone un nuovo tipo L0, ipotesi che L0 §2 attualmente non ammette).

## Q5 — Relazione fra archivio, vista L1 e set attivo L2

- **L3:** §2, riga 9, riceve viste L1 e decisioni L2 in sola lettura; §4 riga 16 dice che ARCHIVE sposta l’elemento in archivio «intero e raggiungibile»; §5 righe 19–20 descrive vista separata e ritorno in circuito.
- **Leggi:** L1 §1–§2 e §7, righe 18–30 e 99–103: vista derivata dal log L0, senza scritture e con nodi/archi originati dagli eventi L0; L2.11 e L2.20, righe 202–232: replay autorevole e `build_active`; L2.16, righe 180–185, precisa che summary non rimuove nulla da log o vista.
- **Indeterminato:** se ARCHIVE rimuove solo dalla vista lunga L3, anche da `build_active`, oppure dalla vista L1 (quest’ultima alterazione è in tensione con L1 derivata integralmente dal log); come conservare accessibilità a nodi/archi e cosa significhi «in circuito».
- **Default da deliberare:** L1 resta invariata; archivio è proiezione L3 distinta e ARCHIVE non modifica i dati o le decisioni L2 già autorevoli. Conseguenza: testare che `view::build` e `build_active` restino identici, mentre la vista L3 archivia l’elemento. Se L3 deve sottrarre dal set attivo, va definita una decisione L3 separata senza riscrivere L2.
- **Tocca:** `selection.rs`, nuovo modulo/vista L3; `view.rs` solo se una scelta esplicita compatibile con L1 lo richiede.

## Q6 — Semantica di RECALL e «sequela»

- **L3:** §5, riga 20: «Un evento RECALL riporta un elemento in circuito con la sua provenienza completa: niente ritorni dalle tombe senza sequela»; §6 riga 27 richiede test di richiamo.
- **Leggi:** L0 §2 e §7, righe 25–42 e 97–103, garantisce seq/hash e replay; L1 §3, righe 52–60, stabilisce `origin_seq` minimo; L2.17, righe 95–98, vieta candidati storici non attivati per L2, ma non definisce «sequela» L3.
- **Indeterminato:** significato di sequela (nuove attivazioni L2 dopo ARCHIVE, eventi di arco/menzione, o altro), minimo/massimo numero, ordinamento, se RECALL stesso conta, e comportamento su provenienza mancante o elemento mai archiviato.
- **Default da deliberare:** richiedere almeno una nuova attivazione L2 dell’identità dopo `archived_at_seq`, distinta dall’evento RECALL; conseguenza: testare richiamo senza sequela (rifiuto/nessun effetto da decidere) e con sequela, mantenendo l’`origin_seq` L1 originale. Le leggi non determinano né condizione né esito dell’errore.
- **Tocca:** `selection.rs`, modulo/vista L3.

## Q7 — Forma canonica e ancoraggio dell’archivio

- **L3:** §5, riga 19: «vista separata ancorata agli hash del log, riproducibile al 100% in replay»; §6 riga 23 impone stesso log → stesso archivio byte per byte; riga 25 richiede seq d’origine.
- **Leggi:** L0 §2 e §7, righe 27–42 e 97–103, definisce hash e replay; L1 §5–§7, righe 74–103, definisce schema canonico, `head_hash`, `view_hash` SHA-256 e ordinamenti; L2.12, righe 208–211, richiede coerenza head hash per L1.
- **Indeterminato:** schema/versione archivio, campi, hash singolo o per elemento, preimage, inclusione di `head_hash` o hash dell’evento ARCHIVE/RECALL, ordinamento, deduplica e serializzazione. «Ancorata agli hash» non specifica quale hash né equivalenza con `view_hash` L1.
- **Default da deliberare:** forma canonica separata con `schema_version`, `head_hash` e `archive_hash` calcolato come L1 escludendo il campo hash; ordinamento per id e contenuto completo con `origin_seq` e `archived_at_seq`. Conseguenza: fixture byte-per-byte e hash indipendente; proposta da approvare, non prescritta da L1.
- **Tocca:** nuovo modulo/vista L3, `view.rs` solo per riuso API senza cambiarne contratto.

## Q8 — Archivio append-only e stato in caso di ripetizioni

- **L3:** §1, righe 4–6, promozione/dimenticanza/archivio e «Mai cancellare»; §4 riga 16, «resta intero e raggiungibile».
- **Leggi:** L0 §6, righe 84–95, vieta update/delete/riscritture e corregge con evento compensativo; L1 §1 e §7, righe 20–24 e 101–103, derivazione dal log; L2.9, righe 173–178, dice che summary non è «espulsione pop» e vieta rimozione dal log/vista.
- **Indeterminato:** se PROMOTE/ARCHIVE/RECALL duplicati siano rifiutati, idempotenti o eventi ripetibili; come rappresentare archiviazioni successive e quale `archived_at_seq` prevalga; «raggiungibile» richiede lookup o solo conservazione nel log.
- **Default da deliberare:** nessuna cancellazione o mutazione retroattiva; ogni transizione è un evento in coda e lo stato d’archivio si ricostruisce in ordine di seq. Conseguenza: testare duplicati e transizioni multiple, ma la regola per validità/idempotenza e per `archived_at_seq` resta da decidere.
- **Tocca:** modulo/vista L3; `log.rs` solo tramite append pubblico esistente.

## Non procedere prima delle risposte

- Non definire T né alterare il payload sigillato della genesi L0.
- Non fissare formula, tipi, overflow, soglie o tie-break L3 per decadimento.
- Non codificare schema/payload o discriminanti PROMOTE, ARCHIVE e RECALL.
- Non rimuovere elementi da L1 o dal set autorevole L2, né interpretare «sequela».
- Non definire schema/hash/ordinamento dell’archivio né comportamento per eventi ripetuti.
- Non scrivere test byte-per-byte che assumano una delle opzioni sopra senza decisione normativa.
