# L2 — Legge di selezione (default firmati dall'orchestratore, 23/10/2026)

Emenda operativa a `spec/L2_SPEC_v0.1.md`; approvata dall'umano nel messaggio
«producer L2 test-first con i default firmati». Da audit del reviewer, NON
pre-auditate nel codice. Dove questa legge tace, il producer SÍ ferma: niente
estrapolazione; chiede all'orchestratore.

**L2.1 — Tipi evento (Q1).** Nessun tipo logico L0 nuovo. Ogni scritta L2 è un
evento `ACTION` con chiave payload `l2` (stringa) discriminante:
`"config" | "select" | "summary" | "barrier"`. Payload L0 resta flat (solo
`i64`/stringa); le liste sono stringhe contenenti JSON canonico interno (come
F1 di `spec/L1_SPEC_v0.2.md`). Gli eventi senza `l2` non sono di L2.

**L2.2 — Config (Q5).** L'evento `l2="config"` porta `k` (i64 ≥ 0) e `tau`
(i64 ≥ 1). Vale dalla propria seq in poi; config successive ragpiano la
storia. Se la selection operates senza config / con config malformata →
errore, non aggiorna default impliciti.

**L2.3 — Attivazione (Q2).** Un evento L0 con estrazioni L1 attiva ogni
elemento che dichiara (ragso diretto) o menzionato (endpoint di arco) nello
stesso evento; una sola attivazione per elemento per seq, mai in replay.

**L2.4 — Salienza (Q3).** score(e, t) = Σ_att contributi interi:
contribution(a) = `2^24 >> m`, con `m = min(⌊(t − seq_a) / tau⌋, 24)`,
`seq_a` = seq dell'evento che ha generato l'attivazione, `t` = seq
dell'evento di valutazione. Puro `u64`/shift, nessun float.

**L2.5 — Peso archi (Q4).** Peso fisso `1.0` per ogni arco, indipendente da
`rel`. Contributo dell'arco = un attivazione al tick dell'evento che ha
dichiarato l'arco (seq di dichiarazione minimo, D18 L1); senza doppio
ricalcolo per riprova. Gli archi non attivano se stesso.

**L2.6 — Budget e parità (Q8).** Dopo ogni valutazione: se il set eccede `k`,
esce l'elemento con punteggio minimo; pari merito, rimane quello con seq di
origine minore. Se il set è sotto `k` dopo un evento: entra l'elemento non
attivo con punteggio massimo; pari merito, entra quello con seq di origine
minore. `k = 0` ammesso (nessun attivo).

**L2.7 — Momento di decisione (Q9).** La valutazione scatta dopo ogni evento
L0 con estrazioni; le selezioni scritte in quel momento ereditano il tick
dell'evento di valutazione. La replay segue l'ordine di seq.

**L2.8 — Barrier e offline (Q7).** `l2="barrier"` apre la finestra offline;
la chiude il primo evento L0 non-`l2` successivo. Se il payload contiene
`phase="close"` si chiude su quello; una barrier aperta al termine del log
resta valida. Barrier nidificate/duplicate durante una finestra attiva →
errore.

**L2.9 — Summary (Q11).** Evento `l2="summary"` con `expulsions` (stringa
JSON canonico, array di seq degli elementi espulsi in quella finestra) e
`text` (stringa deterministica: id degli espulsi in ordine di id, uniti da
`|`). Il consolidamento copre gli elementi **espulsi** nella finestra; le
sinstesi già registrate non si ri-emettono in replay. Nessun tema di
"espulsione pop" (rimozione dal log o dalla vista).

**L2.10 — Select (Q6).** Evento `l2="select"` con `ups` e `downs`, ciascuno
stringa JSON canonico di seq (i64) degli elementi che entrano / escono in
quella decisione. Ancoraggio = le seq degli eventi che hannogenerato la
valutazione (stessa lista su cui è calcolato il punteggio).

**L2.11 — Replay non ri-appende (Q12).** In replay il builder è a sola scrittura:
legge gli eventi `l2` già nel log e li tratta come **autoritativi**; non ri-emette
select/summary/barrier. Niente verifica del ricalcolo: le decisioni diverse dal
ralcolamento sono un mismatch dichiarato, non un errore catchable.

**L2.12 — Legge con L1 (Q10).** L2 invoca `view::build(&Log)` e verifica che
`view.head_hash` corrisponda all'hash dell'ultimo evento del log prima di
valutare; mi dash se non coincide. Zero scritture su log e vista oltre agli
eventi L2 propri (L2.1).
