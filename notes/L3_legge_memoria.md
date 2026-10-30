# L3 — Legge di memoria lunga (firmata dall'umano, 30/10/2026)

Emenda operativamente `spec/L3_SPEC_v0.1.md` alle condizioni firmate
dall'umano in risposta al triage `notes/L3_domande_memoria.md` (Q1–Q8, 30
ottobre 2026):

> «Q1, Q4, Q5, Q7 e Q8 le firmo coi default così come sono, sono nel solco
> L2. Q2: contano solo le attivazioni L2 ancorate a eventi L0 distinti,
> soglia stretta > T, e il conteggio sopravvive all'ARCHIVE — la memoria non
> muore per decreto, solo per oblio calcolato. Q3: decadimento esponenziale
> su tick logici come la salienza L2, half-life H dichiarata nell'evento
> config L3 insieme a T (via più economica che emendare la genesi
> sigillata), aritmetica intera satura senza overflow, valutazione al tick
> di decisione prima dell'ARCHIVE, tie-break con la scala L2 completa
> (score → origin_seq → id). Q6: sequela = almeno una nuova attivazione L2
> dell'identità con seq > archived_at_seq, distinta dal RECALL stesso, e il
> ritorno conserva l'origin_seq L1 originale.»

Firma orchestratore: presente, 30/10/2026. Dove questa legge tace vale il
pattern L0–L2 in federe; il producer SI ferma e chiede, niente intuizioni.

---

**L3.1 — Eventi L3.** Nessun tipo logico L0 nuovo. Ogni scritta L3 è un
evento `ACTION` con chiave payload `l3` (stringa) discriminante:
`"config" | "promote" | "archive" | "recall"`. Payload L0 resta flat (solo
`i64`/stringa); gli id sono stringhe hex minuscolo a 64. Chiavi extra
rispetto a quelle prescritte → `L3Error::Schema` (pattern L2.21):
config ha solo `T`,`H`; promote/archive solo `at`,`id`,`origin_seq`;
recall solo `at`,`id`,`origin_seq`. Eventi senza `l3` sono fuori da L3;
un evento con `l2` non è un evento L3 (i domini sono separati e chiusi).
**Interazione con la finestra L2 (conseguenza dedicata):** gli eventi L3
sono eventi L0 non-`l2`, quindi chiudono implicitamente una finestra L2
aperta (L2.25); sono privi di estrazioni, quindi non sono eventi di
valutazione L2. L2 non legge `l3`; L3 legge le attivazioni L2 (L2.3/L2.13)
e non muta alcuna decisione L2.

**L3.2 — Config Q1.** `l3="config"` porta `T` (`i64 ≥ 0`, soglia di
riattivazioni) e `H` (`i64 ≥ 1`, half-life del decadimento in tick
logici). Vale dalla propria seq in poi, per il conteggio e per il
decadimento; un config successivo supersede il precedente a partire dalla
propria seq (pattern L2.2). La geni L0 resta **invariata** (payload
sigillato `{prng,seed,state}`, L0 §4): T ed H vivono nell'evento config,
non nella genesi. Valutare senza config o con config malformata →
`L3Error::Config`; niente default impliciti.

**L3.3 — Conteggio delle riattivazioni Q2.** Per identità: il conteggio è
il numero di **eventi L0 distinti** che l'hanno attivata per L2.3/L2.13
(dichiarazione diretta o menzione, una sola attivazione per elemento per
seq), contati dall'intera storia del log. Persistente: **mai azzerato**
da ARCHIVE, RECALL o da qualunque transizione — la memoria non muore per
decreto, solo per oblio calcolato. La promozione richiede **soglia
stretta**: conteggio `> T`.

**L3.4 — Decadimento Q3 (metà del punto).** Punteggio di decadimento
dell'identità al giudizio:

```
contribution(a) = 0                       se m ≥ 25
                = 2^24 >> m               altrimenti
m = ⌊(tick_t − tick_a) / H⌋
```

dove `tick_t` = tick dell'**evento di valutazione** (L2.7: il giudizio
avviene per eventi L0 che portano estrazioni) e `tick_a` = tick
dell'evento attivante. `tick` L0 è non decrescente: la differenza è ≥ 0.
Il sum è `saturating_add` intero: **nessun overflow, no float, no
wall-clock**. Il decadimento è lo stesso meccanismo esponenziale della
saliienza L2 (L2.4), con `tick` al posto di `seq` e `H` al posto di
`tau`; il cap simbolicamente a 25 rende naturale lo zero (il contributo
muore). Valutazione al tick di decisione, **prima** della scrittura
dell'ARCHIVE: l'evento archive non partecipa al proprio giudizio. La
soglia `= 0`: identità con `score == 0` è in oblio.

**L3.5 — PROMOTE Q2/Q4.** Al giudizio, per identità tracciate con stato ≠
`memoria_lunga` e conteggio `> T`: decisione `Promote`. Payload encode:
`l3="promote"`, `at` (`i64` = seq dell'evento di valutazione; anchoring
alla seq di origine della decisione, pattern L2.18), `id` (hex),
`origin_seq` (`i64`: origine L1 dell'identità, se L1 la conosce). Il tick
dell'evento di valutazione è il tick dell'evento apposto.

**L3.6 — ARCHIVE Q3/Q4.** Al giudizio, per identità con stato ≠
`archiviata` e `score == 0`: decisione `Archive` con lo stesso payload di
promote (`l3="archive"`, `at`, `id`, `origin_seq`). `archived_at_seq`
della vista L3 è la **seq dell'evento archive** quando apposto in log.
Le identità già archiviate non si ri-valutano: la ri-archiviazione è
soppressa in replay (pattern L2.9), finché una sequela non le riporti nel
circuito e un nuovo oblio a 0 le ri-archivi con nuova `archived_at_seq`.
**Ordine deterministico delle decisioni emesse alla stessa valutazione:**
Archive in scala di espulsione L2 (score minore, poi origin_seq
maggiore, poi id maggiore — il più obsoleto prima, come L2.6→L2.24);
Promote in scala d'ingresso L2 (origin_seq minore, poi id minore — come
L2.17). Una decisione per identità per valutazione; al massimo un'Archive
e un Promote per identità nello stesso giudizio (in ordine di emissioni
date le regole sopra).

**L3.7 — RECALL e sequela Q6.** `l3="recall"` con `at` (`i64`: seq
dell'evento L0 attivante che costituisce la sequela), `id`, `origin_seq`.
Vincoli verificati in replay autorevole, violare uno è `L3Error::Recall`:
(a) lo stato dell'identità è `archiviata` al momento del recall;
(b) `at` esiste ed è un evento L0 con estrazioni che attiva l'identità
per L2.3/L2.13;
(c) **sequela**: `at.seq > archived_at_seq` dell'ultima archiviazione,
ed è distinto dall'evento recall stesso (sempre vero: `at` è nel log
precedente). RECALL senza sequela è **vietato**: niente ritorni dalle
tombe senza sequela. Il richiamo riporta lo stato a `circuito`;
l'**origin_seq L1 originale è conservato** (nessuna nuova origine).
Re-richiamo di un'in-circuito o in-memoria-lunga → `L3Error::Transition`.

**L3.8 — Stato L3, proiezione e append-only Q7/Q8.** Append-only puro
(L0 §6): ogni transizione è un evento in coda, nulla si cancella né si
muta retroattivamente; lo stato L3 si ricostruisce in ordine di seq.
Stato: `circuito` (origine e default), `memoria_lunga` (PROMOTE),
`archiviata` (ARCHIVE). Transizioni legali: circuito→memoria_lunga
(promote), circuito/memoria_lunga→archiviata (archive), archiviata→
circuito (recall); altri passi o ripetizioni → `L3Error::Transition`
(pattern L2.19). In replay gli eventi `l3` in log sono **autoritativi**
(L2.11); le identità tracciate sono quelle attivate almeno una volta.
Proiezione canonica (documenta `build_memory`):

```
{"archive_hash":"…","entries":[…],"head_hash":"…","schema_version":"1"}
```

membri in ordine lessicografico; `head_hash` = hash dell'ultimo evento
del log (D14-pattern); `schema_version` stringa `"1"` (D15-pattern);
`archive_hash` = SHA-256 minuscolo del documento canonico **escluso il
proprio campo** (D16-pattern); `entries` in ordine di `id`, ciascuna
`{"archived_at_seq":…, "id":…, "origin_seq":…, "status":…}` con
`archived_at_seq` presente solo se `status` = "archiviata". Stesso log →
stesso documento, byte per byte.

**L3.9 — Cutoff e interazione L1/L2 Q5.** L3 legge il log e la vista L1
(invoc `view::build` con check `head_hash`, L2.12-pattern). L1 e L2
restano **invartiti**: un evento `l3` non muta né la vista L1 né il set
attivo L2. Cutoff: le decisioni nuove riguardano eventi di valutazione
con seq successiva all'ultimo evento `l3` **decisionale**
(promote/archive/recall) già in log; `config` è evento di controllo e
non partecipa al cutoff (pattern L2.23). Coerenza di prefisso su
`log_len`/`head_hash` in evaluate (L2.22-pattern) → `L3Error::HeadMatch`.

**L3.10 — API di `src/memory.rs`** (pattern L2.20; attuazione e formati
esatti in accordo ad questa legge):

- `pub struct MemoryView { archive_hash: String, config: Option<(i64,i64)>, entries: Vec<Entry>, log_len: usize, head_hash: String }`
  dove `Entry { id: String, origin_seq: u64, status: Status,
  archived_at_seq: Option<u64> }` e `pub enum Status { Circuit, Long, Archived }`;
  `archive_hash` è calcolato da `build_memory` con la regola di L3.8
  (SHA-256 minuscolo del documento canonico escluso il proprio campo);
- `pub fn canonical(view: &MemoryView) -> String` — serializzazione
  canonica del documento L3.8, membri in ordine lessicografico, nessuno
  spazio: è testo sagefa di riprodotto byte per byte; `archive_hash` di
  `view` è la SHA-256 del preimage, cioè dello stesso testo prodotto da
  `canonical` privo del campo `archive_hash` (come `view_hash` in L1/D16);
- `pub enum L3Error { Config, Schema, HeadMatch, Recall, Transition }`;
- `pub fn build_memory(log: &Log) -> Result<MemoryView, L3Error>` —
  replay autorevole a fine log (L3.8), check `head_hash` (L3.9);
- `pub fn evaluate(log: &Log, base: &MemoryView) -> Result<Vec<Decision3>, L3Error>`
  — decisioni nuove per eventi di valutazione oltre il cutoff (L3.9),
  includendo promozioni e archivii con ordine L3.6;
- `pub enum Decision3 { Promote { at: u64, tick: u64, origin_seq: u64, id: String },
  Archive { at: u64, tick: u64, origin_seq: u64, id: String } }`;
- `pub fn encode(d: &Decision3) -> Vec<(String, Value)>` — payload L0 per
  promote/archive (L3.5/L3.6);
- `pub fn encode_recall(at: u64, origin_seq: u64, id: String) -> Vec<(String, Value)>`
  — payload dell'evento recall; append via API L0 al chiamante.
  Zero I/O, zero wall-clock, zero RNG.

---

## Firma

- [x] **umano** — 30 ottobre 2026 (pacchetto Q1–Q8: default Q1/Q4/Q5/Q7/Q8
  firmati così come sono; regole puntuali per Q2 il conteggio, per Q3 la
  formula di decadimento e per Q6 la sequela, citate in epigrafe; questa
  legge trascrive quella firma)
- [x] **orchestratore** — 30 ottobre 2026 (trascrizione del pacchetto in
  legge; niente requisiti oltre le decisioni umane e i pattern L0–L2)
