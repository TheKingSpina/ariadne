# L3 — Addendum di provenienza (31/10/2026)

Regola del progetto: la storia è append-only e i messaggi di commit non si
riscrivono. Questo note completa la citazione mancante segnalata come NOTA
(4) dal reviewer nell'audit del 30/10/2026 (modulo `openai/gpt-6.1-terra`,
una tantum autorizzata) e riconfermata aperta dal re-audit del 31/10/2026
(modulo `opencode/nemotron-3.5-super-vl-free`, `reports/L3_reaudit_4f353239.md`).

## NOTA (4) — integrata

Il primo commit del task L3, `a337652` («test: add red L3 memory contract
tests and API stubs»), contiene i test rossi `tests/l3_memory.rs`, gli stub
in `src/memory.rs` marcati `TODO L3: stub test-only` (pattern di test-first
consuetudinario, ammesso dalla stessa prassi in L2 e ratificato dai gate del
tester) e la riga di export in `src/lib.rs`:

```
+pub mod memory;
```

La riga di export era espressamente prevista come «unico scostamento
ammesso» dal dispatch del 30/10/2026, ma il messaggio di `a337652` non
la citava. Questo note ne fede la citazione complementare: nessun codice
di produzione è entrato in quel commit (gli stub non hanno logica e
falliscono a runtime per costruzione; la logica arriva a `3822396`,
commit successivo al test ed ispezionabile in `git log`).

## Trail di governance del livello L3 (sintesi per l'atto di accettazione)

- legge: `5f6e3d0` (Q1–Q8) → `a46e9b2` (L3.8/L3.10 `canonical`) →
  `3b3482a` (replay richiede config) → `aed10e8` (emendamenti E1/E2
  post-audit, firmati dall'umano);
- test-first: `a337652` + `907d8b0` (rossi) → `3822396` (implementazione,
  solo `src/memory.rs`) → `acfdf82` (NOTA 1 del gate tester) →
  `0f01211` (test rossi E1/E2/digest/soppressione) → `4f353239`
  (fix, solo `src/memory.rs`, +3/−2);
- suite finale: **119/119** debug e release (14 test L3);
- gate tester: CONFORME su `3822396` (114/114) e su `4f353239` (119/119,
  `reports/L3_test_report_4f353239.md`);
- audit reviewer: NON CONFORME il 30/10 (4 rilievi, verificati e reali) →
  re-audit CONFORME il 31/10 su `4f353239` (`reports/L3_reaudit_4f353239.md`),
  limiti dichiarati (lettura `git` negata all'agente, provenienza attestata
  dal report tester).
