# Analisi della sequenza integrata L1–L4 e di AUX

**Aggiornamento successivo:** questa analisi descriveva il blocco prima
dell'implementazione. Il profilo ora attuato risolve i select mediante ID
espliciti e usa una sequenza senza prefisso L5. Contratto e copertura reali:
`AUX_V2_PROFILO_IMPLEMENTATO.md`; test: `tests/aux_v2.rs`. Le affermazioni
sull'assenza di implementazione qui sotto sono quindi storiche, non lo stato
attuale. Le esclusioni del nuovo profilo restano esplicite nel contratto.

**Stato: analisi statica, bozza non approvata.** Non è un'esecuzione né una
dimostrazione di conformità. La proposta AUX non è ancora implementata: la
presente analisi non modifica le SPEC, le leggi firmate o il codice.
Riferimenti: `bozza_coordinata_L0_L6_AUX.md` §§12–13,
`manifest_stress_L0_L6_bozza.md`, leggi L2–L4 e `spec/L0_SPEC_v0.4.md`.

## Prefisso determinato dalla bozza

Questa sequenza assume esattamente il prefisso L5 candidato del manifest:
genesi a seq 0, AUX L5 a seq 1–8, tutti tick 0. Alla sua fine la finestra
L2 è chiusa. Nessun evento aggiuntivo è inserito nel prefisso.

| Seq | Tick | Evento candidato | Effetto |
| ---: | ---: | --- | --- |
| 0 | 0 | GENESIS, seed 1042 | Nessun draw; hash golden L0 |
| 1–8 | 0 | AUX L5 del manifest | Osservazioni e decisioni solo L5 |
| 9 | 0 | AUX L2/config, `k=3`, `tau=5` | Configura L2 |
| 10 | 0 | AUX L3/config, `T=1`, `H=10` | Configura L3 |
| 11 | 1 | SENSE, `stimolo` del primo draw | Fonte L0 dell'estrazione |
| 12 | 1 | ACTION, `mossa=stimolo mod 3` | Secondo evento del ciclo |
| 13 | 1 | STATE del ciclo | Completa la tripletta contigua |
| 14 | 1 | AUX L1/extract | Quattro nodi e arco della tabella del manifest, ciascun membro con fonte `[11]` |

`sources` dell'estrazione è `[11]`; `provenance.members` contiene cinque
voci indicizzate 0–4, tutte con `sources=[11]`, prima i quattro nodi e poi
l'arco. `refs=[]`. Tutti i quattro nodi hanno `origin_seq=11`, che è diversa
dalla seq 14 dell'AUX. Il rapporto fra il numero PRNG e i nomi è la tabella
sintetica dichiarata nel manifest, non un'affermazione sul mondo esterno.
I valori dello stimolo, gli hash degli eventi e i byte degli AUX non sono
qui dichiarati risultati di una run.

## Verifica aritmetica del primo giudizio L2

La tabella degli ID candidati negli oracoli dà, in ordine crescente:

1. delta `0b742e7b07b05582b4afb020715c9fff31e0bd3e107df888b0aa2cb4a49414f1`;
2. beta `45aba7c6bf7186d413becacb7f88e22665b4b50b25829e668df7435020664ee0`;
3. gamma `781246dd55b21c887bc8c3e7badd05c350dc931eebf5c7b98338ff5f727048dc`;
4. alfa `c735556705ac4d4485024861e77e1c1b2d569542c8d5a4359f5ed96334aa8feb`.

Ogni identità ha una sola attivazione dalla fonte seq 11, compresi gli
endpoint dell'arco perché la menzione si deduplica per identità e fonte.
I punteggi e le origini sono uguali; applicando la scala L2.6/L2.24 e
`k=3`, il set candidato risultante contiene delta, beta e gamma, non alfa.
Questa conclusione sul set non stabilisce ancora la forma valida di
`select`: tutte le identità condividono `origin_seq=11`.

## Conflitto da chiudere per un select verificabile

L2 usa liste `ups`/`downs` di sole `origin_seq`, non di ID. Nel caso sopra,
tre ingressi distinti avrebbero tutti origine 11. Occorre stabilire se e
come preservare la molteplicità nei due array e distinguere un candidato
scartato prima dell'ingresso da un membro precedentemente attivo espulso.
La proposta di summary si rifà a L2.26, che deduce gli espulsi dallo stato
attivo precedente al select: nel primo giudizio alfa non era già attivo.
Non si può quindi dedurre automaticamente che `downs=[11]` sia un'espulsione
da inserire nel summary o che i tre ingressi siano rappresentabili in modo
non ambiguo. Va definita una regola v2 completa e verificata con un esempio
di selezione e summary prima di assegnare seq ai successivi AUX.

Anche la finestra L2 non consente eventi non-L2 mentre è aperta (§12.3):
un'estrazione L1 o una narrazione L4 non può essere apposta nella finestra.
Perciò il piano deve specificare quali select pendenti vengono apposti dopo
l'apertura, quali fonti precedenti li generano, e il batch consecutivo
`close → summary` prima di qualsiasi evento non-L2. Non basta indicare
«estrazione durante la finestra».

## Giudizi successivi da completare

- Tick 2: la tabella propone una seconda fonte L0 distinta per alfa; con
  `T=1`, due attivazioni distinte soddisfano la soglia stretta di promote,
  a condizione che il replay v2 e la decisione apposta siano coerenti.
- Tick 253: con `H=10`, i contributi delle attivazioni di alfa ai tick 1 e
  2 hanno età 252 e 251 e quindi decadimento zero. Un giudizio valido può
  proporre archive, ma occorre fissare l'ordine di tutte le decisioni
  contestuali e le rispettive seq.
- Tick 254: la nuova fonte L0 deve seguire la seq dell'archive apposto;
  solo allora può sostenere un recall che conserva `origin_seq=11`.
- L4: le revisioni narrano gli AUX rilevanti e devono essere coppie
  consecutive narrate/snapshot, con `at`, `sources`, testo e `self_hash`
  derivati dalle seq e dagli hash reali del prefisso. Non è corretto
  inventarli prima di risolvere select e ordine degli eventi.

## Stato dell'implementazione AUX

L'enum `EventType` in `src/event.rs` ammette soltanto `SENSE`, `ACTION`,
`STATE`; parser e serializzatore non supportano AUX. L'implementazione L2
esistente in `src/selection.rs` applica la rappresentazione storica e la
chiusura implicita della finestra, non il batch v2. È quindi impossibile
attestare il replay v2 o il comportamento del batch sul codice attuale.
La suite Docker già eseguita sul commit `f447ca4` verifica la versione
esistente, non le nuove regole. Non sono stati eseguiti nuovi test in questa
analisi.

Prima di implementazione e test restano necessari testo v2 completo e
ratificato, una sequenza coerente e manifest/oracoli approvati, secondo i
vincoli di lavoro già stabiliti. L'analisi individua il punto specifico da
risolvere, non certifica che il resto del progetto sia conforme.
