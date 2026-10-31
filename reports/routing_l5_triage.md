# Registro Router — triage L5 Sociale (mandato 7)

## Identificazione

- **Mandato:** triage L5 Sociale (una sola fase: il triage non tocca
  codice), emesso dall'orchestratore al Router.
- **Policy di routing:** `.opencode/routing.md` v1 (ATTIVO, 31/10/2026),
  verificato per lettura prima del dispatch («Stato: ATTIVO»).
- **Revisione Git:** HEAD dichiarata `1a479c6`; verificata per lettura
  `.git/refs/heads/main` → `1a479c6d4e093e36b1eb1bc4dfde5f6ae8f9147b`.
- **Contratto:** `spec/L5_SPEC_v0.1.md` (intera; perimetro, non estensibile)
  + L0–L4 intoccabili (leggi `notes/L2_legge_selezione.md`,
  `notes/L3_legge_memoria.md`, `notes/L4_legge_identita.md` incluse le
  emendazioni; codice `src/{log,event,view,prng,selection,memory,identity}.rs`;
  suite 146/146). Vincoli trasversali richiamati: D18 L1, L2.27 (Lettura A),
  E1/E2/E3 (L3), L4 self_hash preimage, cutoff L2.11/L3.9/L4.6, schema
  chiuso L2.21/L3.1/L4.1; NO nuovi file esterni; niente «eventi notizie».
- **Canale:** Free-first (v1); canale autoriale per il triage: producer
  free prescritto. Nessuna spesa in autonomia: punti che richiedono canali
  differenzianti → solo segnalazione, STOP, umano decide.

## Piano di dispatch (registrato prima della chiamata)

| # | Fase | Ruolo | Classe | Modello richiesto (ID completo) | Variante | Sessione | Motivazione |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Triage L5 (una sola fase) | producer | light | `opencode/glm-5.3-flash-free` | nessuna prescritta → default del catalogo (dichiarato) | `ses_e6c53a29affea4AdUkfEiSmo0M` (settimo mandato, stessa sessione) | Prescritto dal mandato; stesso free autoriale dei triage/dispatch L4 |

- **Serialità/foreground:** un figlio alla volta; nessun fallback; blocco
  reale → fermata e resoconto con session ID.
- **Perimetro scrittura figlio:** SOLO `notes/L5_domande_sociale.md` +
  `reports/l5_triage.md` (entrambi NON committati: l'orchestratore
  committerà dopo la firma umana). NESSUNA scrittura in `src/` né
  `tests/`, NESSUN commit, NESSUN push.
- **Deliverable:** triage nello STESO formato di
  `notes/L4_domande_identita.md` (Q1..Qn con proposte marcate
  «proposta» + sezione «NON decido io» + sezione «termini indefiniti»),
  che NON propone modifiche alla SPEC.
- Nessun tester né reviewer: fase unica triage (gate/audit valgono per
  il ciclo test-first→accettazione, non per il triage).

## Esiti

- **Figlio 1 — producer (triage L5) — ESITO: TRIAGE COMPLETATO.**
  - Ruolo/modello/classe: producer, `opencode/glm-5.3-flash-free`,
    classe light, variante: nessuna (default del catalogo), canale free
    «Free-first». Sessione `ses_e6c53a29affea4AdUkfEiSmo0M` (settimo
    mandato).
  - Mnemosyne: `ses_e0cf60b3-c9a6-4638-b65a-8853f6988c36` chiusa
    (job `job_7b93d361-c31e-4c30-ac9c-430519a44717`); proposte pending
    `mem_12674ce4-fd50-460b-92b2-0af345281423`,
    `mem_e53f9bad-38c5-4cc6-aed1-78bd4d9b5c7a`,
    `mem_db6e47b8-c705-41cc-a78f-479e331b18d1`.
  - **Deliverable (verificati presenti, NESSUN commit):**
    `notes/L5_domande_sociale.md` (Q1–Q15 + termini indefiniti 13 voci +
    «non procedere prima delle risposte» 7 voci) e
    `reports/l5_triage.md` (report compatto).
  - 15 domande, tutte con proposta marcata tranne Q4 (→ «NON decido io»);
    anomalie SPEC 6 voci segnalate; **nessun segnale non-free** (nessuno
    STOP: tutte le proposte sono testuali/strutturali sui pattern vigenti).
  - Nessuna scrittura in `src/`/`tests/`, nessun tocco a leggi/spec,
    nessuna modifica proposta alla SPEC.
- **Fasi B/C non previste** per il triage (fase unica; gate/audit
  valgono per il ciclo test-first → accettazione della legge futura).
- **Prossima decisione:** firma umana sui default Q1–Q15 (raccolti in
  «NON decido io» le voci che solo l'umano può firmare) → committ da
  parte dell'orchestratore dei due file → stesura legge L5.
