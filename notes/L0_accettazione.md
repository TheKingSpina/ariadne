# L0 — accettazione

**Livello:** L0 — substrato. Log append-only aciclico, replay deterministico dalla
genesi, ciclo senso→azione→stato, zero I/O esterna, zero wall-clock, zero float.

**Legge vigente:** `spec/L0_SPEC_v0.2.md` **emendata** da `spec/L0_SPEC_v0.3.md`
(firmato il 22 ottobre 2026). In caso di divergenza prevale la v0.3.

**Codice accettato:** `a02b66a` (implementazione L0). La storia di chiusura —
`a314d54` legge, `4469f0f` infrastruttura agenti, `886ba57` documentazione,
`07abca5` note — e il commit che contiene questo documento non toccano il codice.

---

## Cosa è stato verificato, e da chi

| Ruolo | Contributo | Modello |
|---|---|---|
| Producer | Implementazione + test, test-first su ogni coppia di commit | `glm-5.3-flash-free#high` |
| Tester | Gate indipendenti su worktree pulito, report in `reports/` | `deepseek-v4.1-flash-free#high` |
| Reviewer | Audit in sola lettura a `a02b66a`, verdetto e citazioni `file:riga` | `nemotron-3-ultra-free` |
| Umano | Decisioni N1–N4, §3 (opzione A), approvazione delle memorie, deroga una tantum | — |

## Prova

- **80 test verdi**, eseguiti dal tester su worktree pulito a `a02b66a`, in debug e in
  release (exit 0), conteggio per binario riportato in `reports/L0_n3_n4_test_report.md` §12.1.
- **Replay bit-perfetto a due processi**: `tests/l0_replay_process.rs`, due processi
  distinti, stdout confrontato byte per byte. 6/6.
- **Catena di hash verificabile end-to-end**, immutabilità di un evento vecchio, evento
  senza `seq` rifiutato: `tests/l0_event_log.rs:316-338`, `:287-314`, `:258-266`.
- **Canonicità del parser**: `serialize(parse(s)) == s` sull'intera riga, per costruzione
  (`src/log.rs:220,254-265`); verificata dal reviewer su ogni percorso `Ok`.
- **Golden della genesi** (seed 1042) ricalcolato esternamente con `shasum -a 256`:
  `4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7`.
- **Audit del reviewer**: nessuna violazione dopo la firma della v0.3.
  `reports/L0_review_report.md`.

## Debiti noti, accettati consapevolmente

1. **B4 — rosso processuale.** La coppia `08b1536`→`b2790e9` (ciclo) ha un test che non
   ha mai compilato contro la sua implementazione; il verde arriva in `a298cf9`
   modificando il test. Altri due test corretti dopo l'implementazione (`45ad98d`,
   aspettativa aritmetica errata: il codice era giusto). Le altre coppie hanno diff dei
   test vuoto. La proprietà test-first di L0 è **indebolita, non falsificata**. La storia
   non si riscrive: il valore dell'esercizio è che il cedimento è registrato.
2. **`from_jsonl` non ha chiamanti di produzione.** Il parser severo è per ora una
   funzione senza clienti: la severità è verificata dai test, non esercitata in esercizio.
3. **Seed > `i64::MAX`**: rifiutati con `Error::Seed`. Il corpus L0 (1000–1099) rientra
   largamente in `i64`; nessuna estensione a `u64` pieno è prevista in L0.
4. **CRLF**: qualunque `\r` grezzo è rifiutato dal parser (guardia preesistente a N3).
   Un log scritto con terminatori Windows non entra. Coerente con E4, ma va saputo.
5. **Deroga registrata**: l'orchestratore ha committato su istruzione esplicita
   dell'umano, in due occasioni il 22 ottobre (i commit di chiusura e questo). La regola
   resta: prepara l'indice, il commit è un gesto umano — e ogni deroga va scritta nel
   messaggio di commit, così il registro non mente su chi ha premuto il tasto.

## Dichiarazione

Con la firma qui sotto, **L0 è accettato** e il livello successivo (L1 — vista) può
iniziare. L'accettazione è riservata all'umano: nessun agente la firma, e nessun agente
l'ha firmata.

- [x] **umano** — 22 ottobre 2026

> Firma dichiarata dall'umano in conversazione il 22 ottobre 2026 e trascritta
> dall'orchestratore su sua istruzione esplicita. La firma è dell'umano; la
> trascrizione no. Nessun agente ha accettato L0 e nessun agente l'ha firmata.
