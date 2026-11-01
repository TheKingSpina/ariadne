# Triage L4 — domande da risolvere prima dei test

Documento di triage in sola lettura: non modifica né interpreta in via
normativa le SPEC vigenti. Le proposte indicate come default sono scelte da
sottoporre a decisione; non autorizzano da sole implementazione o deviazione
dalle leggi L0–L3. Segue il pattern `notes/L3_domande_memoria.md`; le
citazioni «riga N» si riferiscono ai file indicati nella singola voce. HEAD
al momento del
triage: `fa0950c` (`fa0950c05491d7d004f7c440267a1740dbadf0ff`, verificato).

## Q1 — Tipo logico degli eventi L4 o discriminante `l4`?

- **L4:** §2, riga 9, «Scritture solo eventi propri: NARRATE,
  IDENTITY_SNAPSHOT»; §4, riga 17, «ogni revisione è un nuovo
  IDENTITY_SNAPSHOT»; §1, riga 6, «decisioni sono eventi in coda a L0».
- **Leggi che toccano il punto:** L0 §2, riga 35, enumera `type` come
  `SENSE | ACTION | STATE` — enumerazione chiusa; L2.1, righe 42–47, e
  L3.1, righe 24–31 (`notes/L3_legge_memoria.md`), stabiliscono il pattern
  vigente: nessun tipo logico L0 nuovo, ogni scritta di livello è un
  `ACTION` con discriminante `l2`/`l3` a dominio chiuso.
- **Indeterminato/opzioni:** la SPEC L4 non dice se NARRATE e
  IDENTITY_SNAPSHOT siano (a) due nuovi tipi logici L0 — che richiederebbe
  emendamento firmato di L0 §2 e riverifica di parser/preimage (L0 §9,
  righe 152–166) — oppure (b) eventi `ACTION` con discriminante `l4`, come
  già fatto per L2 e L3. L'opzione (a) collide con L0 §2 così come firmato.
- **Default da deliberare:** proposta: `ACTION` con discriminante `l4`,
  dominio chiuso `narrate | snapshot` (pattern L2.1/L3.1); l'alternativa con
  nuovi tipi logici richiede firma umana esplicita perché modifica la legge
  L0, non solo la SPEC L4.
- **Tocca:** nuovo `src/identity.rs`; `event.rs`/`log.rs` solo se
  emendamento L0, cosa che la presente nota non autorizza.

## Q2 — Schema payload L4 e bibliografia delle seq

- **L4:** §3, riga 14, «Ogni affermazione narrativa cita le seq di origine:
  il sé con la sua bibliografia»; §2, riga 9; §4, riga 17.
- **Leggi:** L0 §3, righe 46–56, payload flat con solo `i64` e stringhe;
  L1 §3 (F1), righe 34–49, e L2.1, righe 44–46, risolvono le liste come
  stringhe con JSON canonico interno; L2.21, righe 48–53, e L3.1, righe
  27–31, impongono schema chiuso per tag (chiavi extra → `Error::Schema`).
- **Indeterminato:** la SPEC non specifica né nomi né chiavi né tipi dei
  payload `l4`: che campi porta un `narrate` (testo? lista di seq? hash del
  documento?), che campi uno `snapshot` (hash del documento, `head`?,
  `schema_version`?), e come si codifica la «bibliografia» di seq.
- **Default da deliberare:** proposta: schema chiuso pattern L2.21/L3.1
  (es. `l4="narrate"` con testo + bibliografia come stringa JSON canonico
  di seq, stile L2.10/L2.16; `l4="snapshot"` con hash del documento e
  ancoraggio di testa) — nomi e set esatti vanno firmati: gli eventi sono
  API pubblica del log (vedi NON decido io). Qui nessun set è fissato dalla
  SPEC: i nomi citati sono solo esempi.
- **Tocca:** `src/identity.rs`; `payload.rs` (solo se servisse un helper
  nuovo).

## Q3 — Forma canonica del documento d'identità e suo hash

- **L4:** §5, riga 22; §6, righe 25–26 («Replay bit-perfetto della
  narrazione», «Ogni snapshot ancorato a un hash di testa»); §1, riga 5
  («vista con revisioni»).
- **Leggi:** L1 §5–§6, righe 74–97 (membri in ordine lessicografico,
  `schema_version`, `head_hash` = hash dell'ultimo evento, `view_hash` =
  SHA-256 del documento escluso il proprio campo); L3.8, righe 143–155
  (`spec/L3_SPEC_v0.1.md` + legge: stesso pattern per l'archivio memoria);
  L1 §7, righe 99–103.
- **Indeterminato:** la SPEC L4 non definisce lo schema del documento
  identità (campi della narrazione, ordinamenti, serializzazione) né quale
  campo hash esista né come si calcola; «ancorato a un hash di testa» non
  specifica quale hash (in L1/L3 è sempre l'hash dell'ultimo evento).
- **Default da deliberare:** proposta: documento canonico con membri in
  ordine lessicografico, `schema_version:"1"`, `head_hash` = hash
  dell'ultimo evento del log, campo hash dedicato calcolato col pattern
  esclude-se-stesso (D16/L3.8). Stesso log → stesso documento, byte per
  byte. Forma esatta da firmare in legge L4.
- **Tocca:** `src/identity.rs`.

## Q4 — Trigger di emissione di narrate/snapshot e cutoff

- **L4:** §4, righe 17–18 (ogni revisione nuovo snapshot; lo storico è la
  biografia «chi ero a seq X, chi sono a seq Y»); §5, riga 21 («ultimo
  snapshot + delta dagli eventi successivi»).
- **Leggi:** L2.7, righe 106–108, e L3.4 (in `notes/L3_legge_memoria.md`,
  righe 74–75): i giudizi L2/L3 scattano per ogni evento L0 con
  estrazioni; L2.23, righe 163–171, e L3.9, righe 160–164, definiscono il
  cutoff su decisioni già in log.
- **Indeterminato:** la SPEC non dice quando si emette NARRATE e quando
  IDENTITY_SNAPSHOT, il rapporto revisione↔evento di valutazione
  (una revisione per giudizio? a soglie?), né se esista un cutoff L4.
- **Default da deliberare:** proposta: giudizi L4 legati agli stessi
  eventi di valutazione L2/L3 (eventi con estrazioni), una revisione per
  giudizio, cutoff al pattern L3.9 sull'ultimo `l4` decisionale. Attenzione:
  il trigger è anche semantica di business («quando cambia il sé») → vedi
  NON decido io; il default è soltanto coerenza coi pattern, non
  autorizzazione.
- **Tocca:** `src/identity.rs`; letture su `selection.rs`/`memory.rs`.

## Q5 — Regole deterministiche di narrazione: qual è il loro contenuto?

- **L4:** §3, riga 12, «La narrazione è generata da regole deterministiche
  sugli eventi: stesso log → stessa identità, byte per byte».
- **Leggi:** L0 §7, righe 97–103, garantisce solo determinismo dello
  stato; L1 §3, righe 64–65, «nessun contratto semantico a livello L0» —
  le leggi non prescrivono contenuto.
- **Indeterminato:** che cosa afferma la narrazione (quali eventi la
  riguardano, in quale ordine o formato di testo), se descrive anche lo
  stato L3 (memoria_lunga/archiviata), in che forma si esprimano: nessuna
  regola è scritta.
- **Default da deliberare:** questa è semantica di business per
  eccellenza → NON decido io. Unica forma vincolante che le leggi
  impongono già ora: funzione pura deterministica eventi-sotto-seq →
  documento, zero I/O, zero wall-clock (L0 §1, righe 19–23).
- **Tocca:** `src/identity.rs` (modulo, non contenuto firmabile qui).

## Q6 — Componenti stocastiche e budget di draw

- **L4:** §3, riga 13, «Eventuali componenti stocastiche passano solo dal
  PRNG dello stato L0 — niente fonti di casualità fuori dal substrato».
- **Leggi:** L0 §4, righe 70–71, budget dei draw: genesi 0; **1 draw per
  tick dal SENSE; 0 dalla policy**; L0 §6, righe 92–93, invariante: il PRNG
  vive negli STATE e ogni STATE porta lo snapshot; L4 §2, riga 9, scritture
  consentite: solo NARRATE/IDENTITY_SNAPSHOT.
- **Indeterminato/contraddizione:** un draw L4 eccede il budget L0 e
  desincronizzerebbe in replay lo stream che il SENSE si aspetta; per
  avanzare il PRNG servirebbe un evento `STATE` — vietato dallo stesso §2
  della SPEC. L4 è in tensione interna (§3 riga 13 vs §2 riga 9) e con
  L0 §4.
- **Opzioni:** (a) L4 senza componenti stocastiche nella v0.1 — la parola
  «eventuali» lo ammette; (b) derivazione deterministica dallo snapshot
  PCG64 senza avanzamento di stato; (c) emendamento L0 (budget e
  meccanismo) prima di qualunque implementazione.
- **Default da deliberare:** proposta: L4 v0.1 senza componenti
  stocastiche; qualunque stocasticità futura richiede emendamento firmato
  di L0. L'opzione (b) è un'interpretazione non ancora legge.
- **Tocca:** `src/prng.rs` (solo in caso di emendamento), `identity.rs`.

## Q7 — Omonimia «identità»: id L1/L2/L3 vs il sé L4

- **L4:** §1, riga 5, «L'identità non è una fonte: è una vista con
  revisioni»; §3, riga 12, «stessa identità».
- **Leggi:** L2.3, righe 66–67, «L'identità dell'elemento è l'id L1:
  SHA-256 canonico json di `{kind,key}`»; L3.3, righe 51–56, «Per
  identità: il conteggio …»; L3.5–L3.7 usano «identità» per l'elemento.
  Il termine è già portato da L2/L3 con senso diverso.
- **Indeterminato:** doppio uso dello stesso nome nel canone, con rischio
  di collisione in codice, test, leggi e nella futura L5 (che legge
  «viste L1–L4»).
- **Default da deliberare:** proposta: riservare «identità»/«id»
  all'id L1 (uso vigente L2/L3) e denominare l'oggetto L4 in modo distinto
  (es. «sé», «documento del sé», «identità narrativa») — convenzione da
  firmare, non la scelgo io; qui cito soltanto l'opzione.
- **Tocca:** naming in tutta la legge L4 e nell'implementazione.

## Q8 — Come L4 legge le decisioni L2/L3

- **L4:** §2, riga 9, input «Log L0, viste L1, decisioni L2 e L3, in sola
  lettura».
- **Leggi:** le decisioni L2/L3 vivono solo come eventi nel log: l'accesso
  canonico è `build_active` (L2.20, righe 221–224) e `build_memory`
  (L3.10, righe 180–184), con verifica `head_hash` (L2.12, righe 208–211;
  L3.9, righe 157–159); L2.22, righe 242–253, definisce la coerenza di
  prefisso per una base parziale.
- **Indeterminato:** L4 ri-parsa gli eventi `l2`/`l3` (duplicando logica
  altrui) oppure si appoggia alle viste consolidate? E come si coordina
  il check di testa fra le tre fonti?
- **Default da deliberare:** proposta: L4 legge il log e ricava L1 con
  `view::build`, L2 con `build_active`, L3 con `build_memory`; nessun
  re-parse dei domini `l2`/`l3`; `head_hash` confrontata una sola volta
  (coerenza di prefisso stile L2.22 anche per la base d'identità). Riuso
  API senza cambiarne contratto.
- **Tocca:** `src/identity.rs`; `view.rs`/`selection.rs`/`memory.rs` solo
  come chiamate pubbliche esistenti.

## Q9 — Test di divergenza: proprietà enunciata non universalmente vera

- **L4:** §6, riga 29, «Test di divergenza: due log diversi → due identità
  diverse».
- **Leggi:** nessun precedente L1–L3 richiede iniettività: L1 §7, righe
  100–103, e L3 checklist, righe 22–23, chiedono solo determinismo
  (stesso log → stesso output); regole deterministiche su eventi
  generano la stessa identità anche da log che differiscono solo per
  eventi irrilevanti alla narrazione — la proprietà data per universale
  è falsa.
- **Indeterminato:** se il test voglia iniettività universale (non
  ottenibile in generale), o divergenza scoped (differenza su eventi
  «rilevanti» per la narrazione — criterio che dipende da Q5). Analoghi
  enunciati anche in L5, riga 27.
- **Default da deliberare:** proposta: riformulare in legge L4 la
  proprietà in forma scoped (log diversi che mutano eventi rilevanti →
  revisioni diverse); la definizione di «rilevante» segue la semantica di
  narrazione firmata in Q5. Nel frattempo nessun test rosso che assuma
  iniettività.
- **Tocca:** `tests/` (futuro modulo L4), legge L4.

## Q10 — «Boot», «sessione», «log integro»

- **L4:** §5, righe 21–22, «Al boot: replay del log → ricostruzione …»,
  «Nessun file separato "chi sono io" … Se il log è integro, tu sei
  integro»; §6, riga 28, «Test di ripresa sessione».
- **Leggi:** in L0–L3 non esiste «boot» né «sessione»: il sistema è una
  libreria zero I/O (L0 §1, righe 19–23); L0 §7, righe 100–102, la fold
  assume log verificato (`verify() == Ok` prima della chiamata);
  L2.12/L3.9 verificano solo l'ancoraggio di testa.
- **Indeterminato:** «boot» può essere solo una chiamata di funzione; che
  cosa succede su log corrotto o non verificato (errore esplicito?
  rifiuto? comportamento indefinito?); «sessione» è una nozione nuova
  senza nessuna definizione (ridefinisce il process model? file? I/O?).
- **Default da deliberare:** proposta: boot = ogni chiamata pubblica L4 su
  `&Log`; «ripresa sessione» = nessun file di identità, tutto ricalcolato
  dal log (§5, riga 22, già lo vieta); log non integro → errore esplicito
  nella scala d'errori L4 (pattern L2.12/L3.9), mai fallback. La tassonomia
  degli errori è API pubblica → firma umana (NON decido io).
- **Tocca:** `src/identity.rs`, errori.

## Q11 — Interazione con la finestra L2 e il cutoff L3

- **L4:** §1, riga 6, «Le sue decisioni sono eventi in coda a L0».
- **Leggi:** L2.25, righe 134–155, il primo evento L0 non-`l2` chiude una
  finestra L2 aperta; L3.1, righe 32–36, ne fa conseguenza dedicata per
  gli eventi `l3`;
  L2.23, righe 163–171, e L3.9, righe 160–164, definiscono i cutoff.
- **Indeterminato:** se gli eventi `l4` (essendo eventi L0 non-`l2`, a
  seguito di Q1) chiudono implicitamente una finestra L2 aperta; se sono
  eventi di valutazione (no, se privi di estrazioni, ma la SPEC non lo
  dice); se entrano nei cutoff L2/L3 (L2 non legge `l4`; L3.9 guarda solo
  eventi `l3` decisionali).
- **Default da deliberare:** proposta: conseguenze identiche a L3.1 —
  `l4` chiude implicitamente una finestra L2 aperta, non è evento di
  valutazione L2/L3, non entra nei cutoff L2/L3; il cutoff L4 va definito
  in legge (vedi Q4) e l'interazione scritta esplicitamente nella legge
  L4 per ispezione, come fatto in L3.1 secondo capoverso.
- **Tocca:** legge L4, `src/identity.rs`.

## NON decido io

Richiedono firma umana (semantica di business, soglie, costi/concurrenza,
cambi di API pubbliche) — la presente nota NON è una decisione:

1. **Contenuto delle regole di narrazione** (Q5, e criterio di
   «rilevanza» in Q9): che cosa afferma il sé, quali eventi lo generano,
   formato e lingua del testo. Semantica di business.
2. **Trigger di revisione e politica di emissione** narrate/snapshot
   (Q4): quante revisioni, quando, soglie di emissione. Semantica di
   business e soglie.
3. **Qualsiasi componente stocastica** e eventuale modifica del budget di
   draw (Q6): tocca la legge L0 (§4) e l'invariante STATE — solo con
   emendamento firmato. Nessuna decisione qui.
4. **Dominio del discriminante `l4` e schema esatto dei payload** (Q1, Q2):
   gli eventi sono API pubblica del log; a L2 e L3 lo schema è passato da
   pacchetti Q firmati dall'umano. Stesso iter necessario.
5. **Forma canonica definitiva del documento d'identità** e il suo campo
   hash (Q3): nuova API pubblica; default indicato è solo pattern L1/L3.
6. **Semantica di boot/sessione e tassonomia d'errori L4** (Q10): firme
   pubbliche e comportamento in errore.
7. **Formulazione della checklist di uscita** (Q9): la proprietà così come
   scritta non è dimostrabile in generale; va riformulata prima del gate.
8. **Convenzione dei nomi** per disambiguare «identità» (Q7): convenzione
   che attraversa leggi, codice e documenti.

## Verifica di coerenza pre-legge (solo segnalazioni, nessuna decisione)

Punti in cui la SPEC L4 potrebbe toccare o estendere i livelli vigenti:

- **L0:** dominio chiuso dei tipi (§2, riga 35) — NARRATE/IDENTITY_SNAPSHOT
  letteralmente letti come tipi nuovi violerebbero l'enum; payload flat
  (§3, righe 46–56) e parser severo (§9, righe 152–166) vincolano
  qualunque evento `l4`; budget di draw (§4, righe 70–71) — la stocasticità
  L4 (§3, riga 13) non ha un posto legale senza emendamento:
  **estenderebbe L0**; append-only (§6, righe 84–95) e determinismo
  (§7, righe 97–103) sono compatibili con il perimetro L4.
- **L1:** il documento d'identità si colloca come **ulteriore vista
  derivata** (pattern §1, righe 18–24, §7–§8): nessuna estensione prevista
  della vista L1, solo riuso di `view::build`; le convenzioni di
  `head_hash` (D10/D14, righe 83–84), hash esclude-campo (D16, righe
  93–97) e `origin_seq` dei nodi/archi (§3, righe 52–65) sono i punti di
  riuso naturale; ricostruire la bibliografia deve passare dagli id L1
  vigenti, non da nuovi identificatori.
- **L2:** eventi `l4` sono eventi L0 non-`l2` → (se futura legge conferma)
  **chiudono una finestra L2 aperta** (L2.25, righe 139–141) e, privi di
  estrazioni, non sono eventi di valutazione (L2.7); non entrano nel
  cutoff L2 (L2.23); L2 non legge `l4` (L2.1, righe 46–47). Omonimia
  «identità» con L2.3 (righe 66–67) da disambiguare. Riuso di
  `build_active`/`evaluate` come input (L2.20) senza mutare il contratto
  pubblico.
- **L3:** stessa interazione (L3.1, righe 32–36); cutoff L3.9 (righe
  160–164) ignorerà `l4`; il documento d'identità dovrà rappresentare gli
  stati L3 (circuito/memoria_lunga/archiviata) **senza duplicare la
  logica** di L3.5–L3.8 (uso di `build_memory`/`MemoryView`, L3.10); l'ordine
  delle decisioni e l'emendamento E3 (L3.5: Promote su archiviata è
  violazione) sono formula finita che la narrazione deve riflettere
  coerentemente senza re-interpretare.
- **L5 (downstream, solo segnale):** L5 §2, riga 9, legge «viste L1–L4»:
  il contratto L4 diventerà input vincolante di L5; la forma canonica del
  documento L4 (Q3) va fissata prima che L5 ne dipenda.
- **Procedura:** test-first — i test rossi L4 non possono assumere schema
  di eventi, nome di dominio, forma del documento né semantica di
  narrazione prima delle firme (pattern «Non procedere» della nota L3).

## Termini nuovi non definiti in `spec/L4_SPEC_v0.1.md`

Elenco dei nomi usati dalla SPEC senza definizione normativa; nessuna
definizione è qui proposta:

- NARRATE
- IDENTITY_SNAPSHOT
- narrazione (in quanto documento generato rilegibile «byte per byte»)
- affermazione narrativa
- revisione (di chi, a quale innesco)
- biografia
- boot
- sessione / ripresa sessione
- delta (dagli eventi successivi)
- componente stocastica
- divergenza (nella proprietà della checklist)
- integrità («Se il log è integro») — L0 ha `verify()`, ma il termine
  come condizione L4 resta da definire
- processo («un processo, mai un oggetto statico») — retorica di perimetro
  senza operazionalizzazione
- vista con revisioni — costrutto L4 distinto dalle viste L1–L3, non
  specificato

## Non procedere prima delle risposte

- Non dichiarare NARRATE/IDENTITY_SNAPSHOT né tipi logici L0 nuovi né
  dominio `l4` senza firma.
- Non codificare schema payload, bibliografia o forma canonica del
  documento d'identità.
- Non implementare regole di narrazione né trigger di revisione.
- Non introdurre componenti stocastiche né consumare draw oltre al budget
  L0 (questo richiederebbe comunque un emendamento).
- Non scrivere test che assumano iniettività universale dei log (Q9).
- Non definire boot/sessione/ripristino con I/O o file d'identità separati.
- Non usare il termine «identità» per l'oggetto L4 prima della convenzione
  firmata (rischia collisione con id L1/L2/L3).

---

**Footer di provenienza.**

- Modello: `opencode/glm-5.3-flash-free` (classe `light`; variante: nessuna
  — default del catalogo) assegnato dal Router (policy
  `.opencode/routing.md` v1, ATTIVA 31/10/2026), mandato in
  `reports/routing_l4_triage.md`.
- Session ID strumento subagent: non ricavabile dall'interno del figlio
  (nessun parametro esposto; il registro è stato compilato dal Router).
  Session ID memoria (Mnemosyne): `ses_c1d8285d-56d9-41ea-87e5-286d8d797ddb`.
- Data: 31/10/2026. HEAD: `fa0950c`
  (`fa0950c05491d7d004f7c440267a1740dbadf0ff`, verificato).
- **Documento di TRIAGE, non di decisione:** ogni «proposta» indica un
  default da sottoporre a firma; nessun deliverable qui autorizza
  implementazione, legge o deviazione da L0–L3 (vedi NON decido io).
