# Report patch L4 — validazione l2/l3 uniforme nel replay `build_self`

- **modello**: opencode/glm-5.3-flash-free (classe light, variante: nessuna), canale «Free-first», 31/10/2026.
- **sessione mnemosyne**: vedi resoconto finale; esse sono state aperte e chiuse per consolidamento (`memory_open_session` / `memory_close_session`).

## Esito: PATCH COMPLETATA

Rilievo unico (non bloccante) di `reports/audit_l4_impl.md`: il replay di
`build_self` scavalca `validate_decision_fields` per gli eventi non-ACTION
con chiavi `l2`/`l3`, mentre `evaluate_self` li valora senza discriminare il
tipo → divergenza fra le due API sullo stesso log patologico
(`build_self` → Ok, `evaluate_self` → `SelfError::Schema`).

**Direzione firmata dall'orchestratore sotto accettazione umana** («Accetto»
sul ciclo L4, readme `adc0b84`, «patch immediata» dal rilievo, 31/10/2026):
la validazione vale per OGNI evento che il piano L4 legge, a prescindere dal
tipo L0; un payload decisionale su un evento non-ACTION è scritta
decisionale fuori dal proprio dominio (L4.1 domini chiusi; L4.2 mappatura
errori → `SelfError::Schema`) → Schema anche nel replay di `build_self`.

## Commit (test-first, nessun push)

| commit | hash | contenuto |
|---|---|---|
| Commit 1 (rosso) | `48daac3` | solo test: `l4_check4_replay_non_action_with_l2_or_l3_is_schema` in `tests/l4_identity.rs` (log di genesi + evento `SENSE` con payload `l2 select` a chiavi decisionali presenti) — rosso onesto: `22 passed; 1 failed` |
| Commit 2 (verde) | `30adda8` | fix minimo in `src/identity.rs`: `validate_decision_fields` eseguito per ogni evento non-l4 nel replay + `Schema` per non-ACTION con chiavi `l2`/`l3`; nessun altro cambiamento |

## Perimetro file

- `tests/l4_identity.rs` (solo 1 nuovo test, commit 1);
- `src/identity.rs` (solo fix minimo, commit 2 — 9+/6−).
Nessuna modifica a `src/memory.rs`, `src/selection.rs`, `src/lib.rs`, note, policy, config, altri test.

## Verifica finale

- `cargo test` completo (debug): **145/145** verdi (144 pre-esistenti + 1 nuovo).
- `cargo test --release` completo: **145/145** verdi.
- Test nuovo: ROSSO prima (`build_self` → Ok, divergenza documentata), VERDE dopo (Schema uniforme nelle due API).
- Nessun effetto collaterale: tutti i test pre-esistenti L0–L3 e L4 restano verdi in entrambe le modalità.

Rilievo audit chiuso. Checklist non dichiarata dalle autorità: gate spettiente al tester, accettazione all'umano — qui solo lo stato di fatto sopra riportato.

## Aggiunta copertura ramo `l3` (re-audit, rilievo informativo (a), «test subito» 31/10/2026)

- **commit**: `8d82f47` — unico commit di copertura, sopra `30adda8`; **solo test** nuovo `l4_check4_replay_non_action_with_l3_is_schema` (specchio `l3` del test `l2` a `48daac3`: genesi + evento `SENSE` con payload `l3` a chiavi decisionali L3 valide → `build_self` → `Err(SelfError::Schema)`).
- **pattern dichiarato nel messaggio**: copertura post-gate, NON test-first — la patch è già in vigore e il test è **verde alla stesura**; nessun cambiamento al codice di produzione.
- **esito**: `cargo test` completo (debug) **146/146** verdi (145 + 1).
- **modello**: opencode/glm-5.3-flash-free (classe light, variante nessuna), canale «Free-first», 31/10/2026.
- **sessione mnemosyne di questo mandato**: vedi resoconto finale (`memory_open_session` → propose → `memory_close_session`).
