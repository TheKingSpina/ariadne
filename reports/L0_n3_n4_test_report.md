# Gate L0 — N3 (parser severo) + N4 (fixture canonica) — report di verifica

modello: opencode/deepseek-v4.1-flash-free#high

- HEAD verificato: `fba6565f3c3c6a56a9a2c12970261f433804f9b0` — *L0 N3-fix: import EventType inutilizzato rimosso dal test N3 (pulizia warning, zero cambi semantici)*
- Catena dichiarata: `88a68da` test rossi N3 → `5d0a652` implementazione → `1b453f9` N4 (test maintenance) → `fba6565` pulizia import.
- Report: `reports/L0_n3_n4_test_report.md` (incrementale).
- Questo report **non dichiara l'accettazione**: elenca esiti e prove. Le decisioni restano all'umano.

---

## 0. Metodo e permessi della sandbox (da leggere prima degli esiti)

| Strumento | Stato | Uso nel gate |
|---|---|---|
| `git log` / `git show` / `git diff` (comandi che iniziano con `git`) | disponibile | storia, stat, diff, assenza golden |
| `shasum`, `ls`, `cat`, `grep` | disponibili | ricalcolo golden, ispezione |
| `cargo` | **NEGATO** — `{"error":{"type":"permission.rejected","message":"Permission denied: shell"}}` | **nessuna esecuzione della suite** |
| esecuzione di binari/target (es. `target/debug/deps/l0_n3_canonical_string-…`) | **NEGATO** (stesso errore) | nessuna esecuzione di test |
| `git worktree` | non tentato oltre la lettura: senza `cargo` non aggiunge prova di esecuzione (vedi §7) | — |

Conseguenza applicata alla regola «un rosso da permessi non è un rosso del codice»: **le voci che richiedono esecuzione sono NOTA/APERTA, non ROSSE**, con il comando esatto per chiuderle. Le voci di codice e di storia sono giudicate su prove statiche citate `file:riga`.

Metodo alternativo per il golden (§6): SHA-256 ricalcolato due volte in modo indipendente — (a) implementazione SHA-256 scritta a mano nel runtime Code Mode, con **self-check sul vettore standard** `sha256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad`; (b) `shasum -a 256` sul file di preimage già presente in `reports/_l0_recheck_preimage_1042.bin`, il cui contenuto è stato ispezionato con `cat -v`:

```
0^_0^_STATE^_{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"}^_0000000000000000000000000000000000000000000000000000000000000000
```

(`^_` = byte 0x1F.) Le due computazioni coincidono: non è un ricalcolo con `sha2` del crate, quindi la prova non è circolare.

---

## 1. Memoria Mnemosyne (richiamata, non decorativa)

- `memory_open_session(projectId: "opencode:Ariadne", taskTitle: "L0 gate N3+N4")` → `sessionId = ses_cc7844bc-0b59-49be-9ec0-09daaf0bf4d8` (corpusRevision `2671e896-…:5352`).
- `memory_context(sessionId, "L0 test")` → 8 ricordi restituiti, `requiredContextComplete: true`, `degradations: []`. Id:
  - `mem_175d396b-0407-4b90-96bc-db2e4ff70baf`
  - `mem_7141677b-9f68-4dd1-9b95-ef090574e24b`
  - `mem_4e59a7ca-ff5c-491b-98a1-808b6fbecc2c`
  - `mem_e5d32c28-0108-406e-85bd-81504a88f85c`
  - `mem_7b37a78d-7145-499c-a139-d0572e393178`
  - `mem_851b8059-a7f0-4890-873d-31ee18021c94`
  - `mem_97b6089c-270f-450f-9082-bc606df15d5e`
  - `mem_076c3ace-4b2a-4759-bcd1-ea94b5ba9698`
- `memory_search(sessionId, "L0 violazione")` → 10 risultati. Id (oltre ai precedenti): `mem_3adbd62d-d5b0-4aad-b13a-1d8661222ea9`, `mem_6cadd392-a84e-464e-a301-317f3678f90c`, `mem_58656f04-b0c6-402d-a362-d59232d861aa`, `mem_08516980-3666-4abf-a8dc-e0f97f96eecc`, `mem_c804d06f-7d08-4542-a8cd-c120519fc302`.
- Pattern noti **effettivamente usati** in questo gate:
  - `mem_6cadd392` («il parser è rigoroso: membri esatti senza varianti, seq assente = errore schema, …») → è l'atteso contro cui ho misurato N3: il suo perimetro copre *membri* e *valori*, **non** l'ordine dei membri né la forma dei numeri; la violazione trovata in §2 cade esattamente in quel buco già noto.
  - `mem_851b8059` (insieme di escape canonico: `" \ \b \t \n \f \r`, altri <0x20 come `\uxxxx` minuscolo, UTF-8 diretto) → è la specifica contro cui ho verificato i casi dichiarati di N3.
  - `mem_3adbd62d` e `mem_c804d06f` (preimage con 0x1F e payload genesi `{prng, seed, state}` con `state` stringa decimale) → usati per ricostruire il preimage del golden N4 in §6.
- Memoria proposta a fine gate: vedi §10.

---

## 2. Voce 1 — Test-first N3

### 2.1 `88a68da` non tocca `src/` — **VERDE**

Prova (output verbatim di `git show --stat 88a68da`, già eseguito all'inizio del gate):

```
commit 88a68da86d9548d1a4412debf56178700dcf1bb2
Author: Alessandro Spinabelli <Alessandro.spinabelli@studenti.unipr.it>
Date:   Thu Oct 22 19:27:19 2026 +0200

    L0 N3: test ROSSI per il parser severo — from_jsonl ammesso SOLO l'encoding che il serializzatore emette
    ...

 tests/l0_n3_canonical_string.rs | 231 ++++++++++++++++++++++++++++++++++++++++
 1 file changed, 231 insertions(+)
```

Unico file toccato: `tests/l0_n3_canonical_string.rs`, 231 righe tutte in aggiunta. Nessun file sotto `src/`. **VERDE**.

### 2.2 Rosso provato su worktree — **NOTA (permesso negato), con prova statica**

Comando esatto per chiudere la voce (non eseguibile da questa sandbox: `cargo` negato):

```
git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/n3red 88a68da
cd /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/n3red && cargo test --test l0_n3_canonical_string -- --test-threads=1; echo "EXIT=$?"
```

Prova statica che il rosso è strutturale e non dipende dall'ambiente (l'implementazione a `88a68da` non può far passare i 10 rifiuti):

- il test immutato asserisce `matches!(res, Err(Error::Schema(_)))` per `\/` (`tests/l0_n3_canonical_string.rs:117-125`), `\u0041` (`:128-136`), `\u00e9` (`:139-147`), `\u007f` (`:150-158`), `\u0008` (`:161-169`), `\u000A` (`:172-180`), `\u000B` (`:184-193`), `\u0022` (`:196-204`), surrogate (`:208-220`), escape in chiave (`:223-231`);
- l'implementazione a `88a68da` **non aveva** il confronto canonico: `git diff 88a68da 5d0a652 -- src/log.rs` mostra che il blocco `let decoded = … write_escaped(&mut canonical, &decoded); if canonical.as_bytes() != &c.b[start..c.i] { … }` è **aggiunto** in `5d0a652` (righe `+` del diff, hunk `@@ -488,7 +498,21 @@`), e che il doc preesistente di `parse_string` citava esplicitamente `\/` e le «coppie surrogate» come **ammesse** (righe `-` dell'hunk `@@ -438,9 +440,17 @@`);
- quindi a `88a68da` `parse_string` decodificava `\/` in `/` e ritornava `Ok`, e l'asserzione `matches!(res, Err(...))` falliva. **Prima asserzione rossa attesa: `rifiuta_escape_solidus`**.

Non riporto un output di test che non ho potuto produrre: il primo errore integrale e l'exit code restano da catturare con il comando sopra. **NOTA**, non ROSSO.

### 2.3 Il test è immutato fra test rosso e implementazione — **VERDE**

```
$ git diff --stat 88a68da 5d0a652 -- tests/
(nessun output)
```

Diff vuoto su `tests/`: l'implementazione `5d0a652` non ha ritoccato il test. **VERDE**. (Nota: `fba6565` tocca il test — rimozione dell'import `EventType` — ma è un commit *successivo* all'implementazione e dichiaratamente di pulizia; vedi §4 per l'impatto.)

---

## 3. Voce 2 — Il criterio N3 copre davvero i casi dichiarati? — **ROSSO**

### 3.1 Il criterio è una riserializzazione, non una lista di escape — confermato

`src/log.rs:452-516`, in particolare `:501-514`:

```rust
let decoded = String::from_utf8(out).map_err(|_| Error::Schema("stringa non UTF-8".to_string()))?;
...
let mut canonical = String::new();
write_escaped(&mut canonical, &decoded);
if canonical.as_bytes() != &c.b[start..c.i] {
    return Err(Error::Schema(
        "stringa non canonica: l'encoding non coincide con l'output del serializzatore \
         (write_escaped): ogni forma di escape non emessa è rifiutata"
            .to_string(),
    ));
}
Ok(decoded)
```

Il confronto è byte-per-byte fra il **token sorgente** (`c.b[start..c.i]`, quote inclusa, `start` fissato a `:453`) e l'emissione di `write_escaped` (`src/payload.rs:112-130`). Nessuna lista di escape: il criterio è strutturale.

### 3.2 Casi dichiarati: coperti — **VERDE (parziale del punto 2)**

| Forma | Test che la rifiuta | Esito atteso |
|---|---|---|
| `\/` | `tests/l0_n3_canonical_string.rs:117-125` | `Err(Error::Schema(_))` — `write_escaped` emette `/` grezzo (`src/payload.rs:126`) |
| `\u0041` (escape non necessario) | `:128-136` | Schema |
| esadecimale maiuscolo (`\u000A`, `\u000B`) | `:172-180`, `:184-193` | Schema — `write!(out, "\\u{:04x}")` minuscolo (`src/payload.rs:124`) |
| surrogate (coppia minuscola, maiuscola, alta isolata) | `:208-220` | Schema — `write_escaped` emette l'astrale UTF-8 diretto |
| `\u0008` al posto di `\b` | `:161-169` | Schema — `'\u{8}' => "\\b"` (`src/payload.rs:118`) |
| `\u0022` al posto di `\"` | `:196-204` | Schema |
| escape non canonico in **chiave** | `:223-231` | Schema (`parse_string` è usata anche per le chiavi, `src/log.rs:308`, `:367`) |

Forma canonicamente valida che il criterio **deve** accettare, provata: `canonical_counterpart_forms_are_accepted` (`tests/l0_n3_canonical_string.rs:81-97`) accetta `{"k":"a/b"}`, `{"k":"A"}`, `{"k":"é"}`, `{"k":"😀"}`, gli short escape `\b\t\n\f\r` (`:88`), `\u0001\u000b\u001f` minuscoli (`:89`), e `/` grezzo in chiave (`:90`); per ognuna asserisce anche `serialize(parse(s)) == s` byte-per-byte (`:92-96`). La riga genesi 1042 completa è ri-verificata byte-exact in `:101-110`.

### 3.3 Controesempio: ciò che il criterio **lascia passare** — motivo del ROSSO

Il criterio di `parse_string` è completo **per i token stringa**. Non è completo per la **riga**: la decisione umana registrata nel test è «il log esterno o è canonico o non entra» (`tests/l0_n3_canonical_string.rs:6-8`) e il commento dell'implementazione afferma «parse∘serialize = identità byte-per-byte» (`src/log.rs:451`). Questa identità **non vale** per input che `from_jsonl` accetta:

1. **Ordine delle chiavi del payload non imposto.** `parse_payload` accumula le coppie nell'ordine sorgente (`src/log.rs:356`, `:378-381`) e `Payload::from_pairs` le inserisce in una `BTreeMap` (`src/payload.rs:55-65`), che riordina alfabeticamente (`src/payload.rs:88`). Nessun confronto con l'ordine sorgente.
   Input (riga schema-valida, stessa forma di `event_line` in `tests/l0_n3_canonical_string.rs:23-27`):
   ```
   {"seq":1,"tick":7,"type":"STATE","payload":{"b":1,"a":2},"prev_hash":"0000000000000000000000000000000000000000000000000000000000000000","hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
   ```
   Esito derivato dal codice: `Ok`; `to_jsonl()` emette `…"payload":{"a":2,"b":1}…` → **`serialize(parse(s)) != s`** (nessuna funzione del parser confronta l'ordine sorgente con l'ordine canonico).
2. **Ordine dei membri top-level non imposto.** `parse_event` legge le chiavi in un ciclo e le dispaccia in uno slot per nome (`src/log.rs:299-332`), senza verificare la posizione; qualunque permutazione di `{seq,tick,type,payload,prev_hash,hash}` è accettata, mentre `event_line` emette un ordine fisso (`src/log.rs:224-236`).
3. **`-0` come intero.** `parse_i64` ammette esplicitamente «`0`/`-0`» (`src/log.rs:397`, `:398-413`): `"-0".parse::<i64>()` è `Ok(0)`, mentre il serializzatore emette `0` (`src/payload.rs:96-98`) → input accettato, riserializzazione diversa.

Nessuno dei 70 test copre questi tre punti (la suite N3 copre solo il livello stringa; `tests/l0_event_log.rs:227-242` copre tipi di valore e chiavi duplicate, non l'ordine; `:258-266` copre `seq` assente).

**Esito voce 2: ROSSO** — non perché un test fallisca (i test scritti passano sul criterio), ma perché il criterio **non realizza la proprietà che dichiara** (`src/log.rs:451`, decisione umana `tests/l0_n3_canonical_string.rs:6-8`): esistono input non canonici accettati, e in quei casi `serialize(parse(s)) != s`. Il perimetro del rosso è la canonicità di **riga**; il perimetro stringa/escape è verde.

Correzione: **non proposta** (spetta al producer).

---

## 4. Voce 3 — Round-trip: `parse(serialize(x)) == x` e `serialize(parse(s)) == s` — **VERDE**

Non è un caso felice:

- `serialize(parse(s)) == s` byte-per-byte, su 8 forme canoniche distinte: `tests/l0_n3_canonical_string.rs:92-96` (7 casi, incluso `/` in chiave) e `:101-110` (riga genesi 1042 completa, `assert_eq!(out.to_jsonl(), format!("{LINE}\n"))`).
- `parse(serialize(x)) == x` per **tutti** i 32 caratteri `<0x20` singolarmente: `tests/l0_n3_canonical_string.rs:71-77` (ciclo `for code in 0u32..0x20`, con `panic!` esplicito se un canonico viene rifiutato) — cioè il round-trip è esercitato sull'intero insieme di emissione dei controlli, non su un campione. Precisione sul tipo di asserzione: il test confronta `serialize(parse(serialize(x)))` con `serialize(x)` (`:74-76`), non il `Value` con `x`; per valori stringa l'identità byte implica il recupero del valore, ma la forma scritta è l'identità byte.
- L'insieme di emissione è **pinnato a mano** (atteso costruito senza riusare il serializzatore) in `:54-63`, e la stringa grande include quote, backslash, tutti i controlli, `DEL`, `é`, astrale (`:45-52`): se `write_escaped` cambiasse forma, il test cade prima del round-trip.
- La riga canonica prodotta dal test è confrontata con la **struttura attesa completa** (`event_line`, `:66-68`, `:76`), non solo con se stessa.

Limite dichiarato (non un rosso): non c'è property-based su stringhe arbitrarie (nessuna dev-dependency di proptest/quickcheck, `Cargo.toml`); la copertura è per classi di caratteri. Le classi coperte sono però esaustive per `<0x20` e includono i casi limite (`0x7F`, non-ASCII, astrale).

---

## 5. Voce 4 — Nessun test allentato — **VERDE**

```
$ git diff --stat 238d586 5d0a652 -- tests/
 tests/l0_n3_canonical_string.rs | 231 ++++++++++++++++++++++++++++++++++++++++
 1 file changed, 231 insertions(+)
```

Il diff completo (`git diff 238d586 5d0a652 -- tests/`, eseguito) è `new file mode 100644` con **solo righe `+`**: nessun test preesistente modificato, nessuna asserzione rimossa o indebolita. **VERDE**.

Nota su `fba6565` (fuori dal perimetro della voce, ma rilevante per l'immutabilità): `git show --stat fba6565` tocca `tests/l0_n3_canonical_string.rs` per rimuovere `EventType` dall'import — una riga di `use`, zero asserzioni:

```
$ git show fba6565 -- tests/
-use ariadne::{Error, EventType, Log, Payload, Value};
+use ariadne::{Error, Log, Payload, Value};
```

Nessuna modifica ad asserzioni, helper o costanti (`event_line`, `parse_line`, `HEX64_*` invariati). Coerente con il messaggio del commit.

---

## 6. Voce 5 — N4 tocca solo `#[cfg(test)]` di `src/event.rs`; implementazione invariata — **VERDE**

```
$ git diff 5d0a652 1b453f9 -- src/
diff --git a/src/event.rs b/src/event.rs
index b93299a..b87ca85 100644
--- a/src/event.rs
+++ b/src/event.rs
@@ -127,9 +127,12 @@ mod tests {
 
     #[test]
     fn compute_hash_matches_documented_preimage() {
-        // preimage indipendente, costruito dal test, contro il golden precomputato
-        // esternamente (Python hashlib) per la genesi a seed 1042.
-        let payload = r#"{"prng":"PCG64","seed":1042,"state":1042}"#;
+        // preimage indipendente, costruito dal test, contro il golden ricalcolato
+        // esternamente con `shasum -a 256` (comando e output verbatim nel
+        // messaggio del commit N4); mai ricalcolato con compute_hash o to_jsonl.
+        // Formato canonico attuale (B2): `state` è la STRINGA decimale dello
+        // stato interno PCG64, non l'intero del payload (forma pre-B2 eliminata).
+        let payload = r#"{"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"}"#;
         let mut expected_pre = Vec::new();
         expected_pre.extend_from_slice(b"0");
         expected_pre.push(0x1F);
@@ -140,7 +143,7 @@ mod tests {
         expected_pre.extend_from_slice(payload.as_bytes());
         expected_pre.push(0x1F);
         expected_pre.extend_from_slice(GENESIS_PREV_HASH.as_bytes());
-        let golden = "e73ef6c4d12742137235c98bf4427abc43bc123de12614806929b5e101721e97";
+        let golden = "4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7";
         assert_eq!(to_hex(&Sha256::digest(&expected_pre)), golden);
         assert_eq!(
             compute_hash(0, 0, EventType::State, payload, GENESIS_PREV_HASH),
```

- Entrambi gli hunk cadono dentro `mod tests` (`src/event.rs:124-163`): toccano solo la fixture `payload`, un commento e la costante `golden`. **Nessuna riga di `compute_hash`, `seal`, `to_hex` o `EventType`** (implementazione: `src/event.rs:76-122`).
- `git show --stat 1b453f9` riporta **solo** `src/event.rs` (7+/4−): nessun altro file toccato dal commit N4.

Vecchio golden assente dal codice, presente nei report — **VERDE**:

```
$ grep "e73ef6c4" src/    → No matches found
$ grep "e73ef6c4" tests/  → No matches found
$ grep "e73ef6c4" reports/
reports/L0_test_report.md:262, :289, :293, :305
reports/L0_commits_test_report.md:542
reports/L0_prng_genesis_test_report.md:200, :271
reports/L0_conformance_test_report.md:226
```

---

## 7. Voce 6 — Golden nuovo ricalcolato e confrontato carattere per carattere — **VERDE**

Preimage ricostruito a mano (formato documentato in `src/event.rs:11-12`):

```
"0" ‖ 0x1F ‖ "0" ‖ 0x1F ‖ "STATE" ‖ 0x1F ‖ {"prng":"PCG64","seed":1042,"state":"43149974864253806286336044685702662359"} ‖ 0x1F ‖ "0"×64
```

Lunghezza misurata: 152 byte (head hex: `30 1f 30 1f 53 54 41 54 45 1f 7b 22`).

Due ricalcoli indipendenti, entrambi concordi:

1. SHA-256 a mano nel runtime Code Mode (self-check `sha256("abc") = ba7816bf…15ad`):
   `4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7`
2. `shasum -a 256` sul preimage su disco:
   ```
   $ shasum -a 256 reports/_l0_recheck_preimage_1042.bin
   4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7  reports/_l0_recheck_preimage_1042.bin
   ```
   (contenuto del file verificato con `cat -v`: coincide byte per byte col preimage sopra)

Costante nel test (`src/event.rs:146`, introdotta da `1b453f9`):

```
4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7
```

Confronto carattere per carattere: 64 caratteri, identici in tutte le posizioni; nessuna differenza di maiuscole/minuscole. Corrisponde anche alla costante d'integrazione preesistente `tests/l0_event_log.rs:29` (`GENESIS_1042_HASH`). **VERDE**.

Bonus di coerenza (prova che il vecchio valore era la forma pre-B2 e non un errore): ricalcolando lo stesso preimage con `"state":1042` (intero) si ottiene `e73ef6c4d12742137235c98bf4427abc43bc123de12614806929b5e101721e97`, esattamente il golden sostituito.

---

## 8. Voce 7 — Suite su worktree pulito all'HEAD (`cargo test` e `--release`, attesi 70 test) — **NOTA / APERTA (cargo negato)**

Comandi esatti per chiudere la voce:

```
git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3n4 fba6565
cd /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3n4
git status --short          # atteso: vuoto
cargo test                  # atteso: 70 passed, 0 failed
cargo test --release        # atteso: 70 passed, 0 failed
git worktree remove /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3n4
```

Non eseguibile in questa sandbox (`cargo` → `Permission denied: shell`; anche l'esecuzione diretta dei binari di test in `target/debug/deps/` è negata).

Prova statica del conteggio atteso (grep dei marcatori `#[test]` nei file all'HEAD):

| File | `#[test]` |
|---|---|
| `src/event.rs` | 2 (`:128`, `:154`) |
| `src/log.rs` | 2 (`:542`, `:556`) |
| `tests/l0_c1_genesis.rs` | 2 |
| `tests/l0_cycle.rs` | 11 |
| `tests/l0_event_log.rs` | 18 |
| `tests/l0_n3_canonical_string.rs` | 13 |
| `tests/l0_prng.rs` | 7 |
| `tests/l0_replay_process.rs` | 6 |
| `tests/l0_state.rs` | 9 |
| **totale** | **70** |

`target/debug/deps` contiene binari di test con mtime `Oct 22 19:29` (risoluzione al minuto: non posso stabilire se la build preceda o segua il commit `fba6565`, delle `19:29:32`; la differenza sarebbe comunque il solo import rimosso, non semantica). Non li eseguo perché l'exec è negato. **Lo stato verde della suite non è da me osservato.** Voce APERTA.

### Replay bit-perfetto (due esecuzioni indipendenti, confronto byte per byte)

- Il test che lo prova esiste ed è genuino: `tests/l0_replay_process.rs:48-63` lancia **due processi distinti** (`Command::new(BIN)`, `:34-39`) con gli stessi argomenti e confronta `a.stdout == b.stdout` byte per byte (`:59-62`); `:81-95` ri-carica l'intero stdout, `verify()` e confronta `replay.to_jsonl() == s`. Non è il caso in-process di `tests/l0_cycle.rs`.
- Non ho potuto eseguirlo (exec negato). Prova documentale disponibile, non riprodotta a HEAD: due artefatti di run precedenti in `reports/` sono byte-identici —
  ```
  $ shasum -a 256 reports/l0_recheck_replay_a.jsonl reports/l0_recheck_replay_b.jsonl
  967df4a0e202185610bc2edd0e01fbc69c81c8790d3d469cc25fdad02f118784  reports/l0_recheck_replay_a.jsonl
  967df4a0e202185610bc2edd0e01fbc69c81c8790d3d469cc25fdad02f118784  reports/l0_recheck_replay_b.jsonl
  ```
  **NOTA**: artefatti preesistenti, non rigenerati da me; valgono come indizio, non come prova a HEAD.

---

## 9. Voce 8 — Il duplicato dichiarato dal producer: analisi (nessuna decisione)

Confronto dei due test all'HEAD:

| | unitario `src/event.rs:128-152` | integrazione `tests/l0_event_log.rs:102-111` |
|---|---|---|
| cosa costruisce | il **preimage a mano** (byte, 0x1F compresi) e chiama `to_hex(&Sha256::digest(..))` e `compute_hash(0,0,State,payload,GENESIS_PREV_HASH)` | chiama `Log::genesis(1042)` e legge `.events()[0].hash` |
| costante | `golden = "4649d1f4…26b7"` (`:146`) | `GENESIS_1042_HASH = "4649d1f4…26b7"` (`:29`) |
| asserzioni | preimage→golden **e** `compute_hash`→golden | hash genesi→golden **e** `to_jsonl()`→riga attesa (`:110`) |
| dipende da `Log`/serializzatore | no | sì (`genesis`, `to_jsonl`) |

**È vero che verificano lo stesso fatto di fondo**: lo SHA-256 del preimage della genesi seed 1042, contro lo stesso valore. La sovrapposizione è reale e totale sul *valore*.

Differenza residua: il test unitario è l'unico che pinna il preimage **senza passare da `Log::genesis`/`to_jsonl`** (anti-circolare a livello di funzione hash); il test d'integrazione è l'unico che pinna il percorso pubblico end-to-end (genesi → hash → byte della riga). La parte «byte della riga» è però coperta anche da `tests/l0_n3_canonical_string.rs:101-110`, che però passa dal **parser**, non dal costruttore.

Argomenti per **tenerli entrambi**: costo trascurabile; due superfici di guasto diverse (una regressione in `compute_hash` e una in `seal`/`genesis`/`event_line` producono rossi distinti e localizzano il difetto); il unitario è il presidio anti-circolare della funzione hash.

Argomenti per **eliminarne uno** (il candidato naturale è l'unitario): il golden è duplicato in due file, quindi un cambio di formato impone due aggiornamenti coordinati — è esattamente ciò che N4 ha dovuto fare (il unitario era rimasto alla forma pre-B2 mentre l'integrazione era già B2: `1b453f9`); un valore duplicato è un valore che può divergere, e la storia mostra che è divergito.

Non decido: la scelta è dell'umano.

---

## 10. Isolamento — accessi a input non dichiarati

- `src/` (libreria): nessun accesso a filesystem, rete, orologio, ambiente. `grep` su `src/` per `std::fs|std::net|SystemTime|Instant|std::env|File::|TcpStream` → **un solo match**, in `src/bin/l0_replay.rs:79` (`std::env::args()`), cioè l'harness diagnostico dichiarato (`SPEC §5`, eccezione documentata nel mod-doc del binario, già registrata nel commit `5b495d7`). **Nessuna violazione riscontrata** nel perimetro N3/N4.
- `Cargo.toml`: nessuna dipendenza nuova per N3/N4 (`sha2` di produzione, `sha2` dev) — coerente con «zero nuove dipendenze» dichiarato in `5d0a652`/`1b453f9`.
- Il test N3 non legge file né ambiente: costruisce le righe in memoria (`tests/l0_n3_canonical_string.rs:23-33`).

---

## 11. Riepilogo

| # | Voce | Esito |
|---|---|---|
| 1 | `88a68da` tocca solo test | **VERDE** (`git show --stat`: 1 file, `tests/l0_n3_canonical_string.rs`, 231+) |
| 1b | rosso osservato su worktree a `88a68da` | **NOTA** — `cargo` negato; rosso derivato staticamente (`rifiuta_escape_solidus`), comando in §2.2 |
| 1c | `git diff 88a68da 5d0a652 -- tests/` vuoto | **VERDE** (diff vuoto) |
| 2 | criterio N3: riserializzazione byte-per-byte | **ROSSO** — copre i 5 casi dichiarati ma lascia passare ordine chiavi payload, ordine membri, `-0` (§3.3) |
| 3 | round-trip `parse∘serialize` / `serialize∘parse` | **VERDE** — 32 controlli esaustivi + 7 controparti + riga genesi |
| 4 | nessun test allentato (`238d586→5d0a652`) | **VERDE** — solo un file nuovo, 231 `+`, 0 `-` |
| 5 | N4 solo `#[cfg(test)]` di `src/event.rs`; impl invariata | **VERDE** — due hunk dentro `mod tests`, nessuna riga di implementazione |
| 5b | vecchio golden `e73ef6c4` assente da `src/` e `tests/`, presente nei report | **VERDE** |
| 6 | golden nuovo ricalcolato e confrontato | **VERDE** — `4649d1f4…26b7` = `4649d1f4…26b7` (2 ricalcoli indipendenti) |
| 7 | `cargo test` e `--release` su worktree pulito, 70 test | **NOTA/APERTA** — `cargo` negato; conteggio statico = 70 |
| 7b | replay bit-perfetto, due esecuzioni indipendenti | **NOTA** — test esiste ed è genuino (`tests/l0_replay_process.rs:48-63`), esecuzione negata |
| 8 | duplicato unitario/integrazione | analisi in §9 (nessuna decisione) |
| 9 | isolamento | nessuna violazione; unico accesso esterno dichiarato in `src/bin/l0_replay.rs:79` |

**Esito complessivo del gate: NON dichiarato** (non spetta al tester). Il punto aperto per il codice è la voce 2 (ROSSO).

---
---

# §12 — N3-bis: canonicità dell'**intera riga** (gate su `a02b66a`)

modello: opencode/deepseek-v4.1-flash-free#high

- HEAD verificato: `a02b66a` — *L0 N3-bis: canonicità di RIGA — from_jsonl ammette solo l'encoding completo che il serializzatore emette (`serialize(parse(s)) == s` per intero)…*
- Catena dichiarata: `edcca4a` **test rossi** → `a02b66a` implementazione.
- Worktree dedicato: `/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis` (`git worktree add … a02b66a`).
- Questo §12 **si aggiunge** al report precedente: nulla è stato cancellato (le §0–§11 restano come prova storica del gate N3+N4, che era su un HEAD diverso, `fba6565`).
- **Questo report non dichiara l'accettazione.** Elenca esiti e prove; le decisioni restano all'umano.

Ordine eseguito: prima l'esecuzione (Passo 1), poi il replay (Passo 2), poi il rosso provato (Passo 3), poi il test-first (Passo 4), e **solo dopo** l'analisi statica dei 5 casi limite (§12.6).

---

## 12.1 Passo 1 — worktree pulito + suite completa (ESECUZIONE OSSERVATA)

Creazione del worktree:

```
$ git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis a02b66a
Preparing worktree (detached HEAD a02b66a)
HEAD is now at a02b66a L0 N3-bis: canonicità di RIGA — from_jsonl ammette solo l'encoding completo che il serializzatore emette (serialize(parse(s)) == s per intero). MECCANISMO SCELTO: RICOMPOSIZIONE, non canonicità posizionale — dopo il parse strutturale la riga è ri-serializzata intera con l'helper esistente event_line (zero duplicazione) e confrontata byte-per-byte col token sorgente ('\n' escluso: parse riceve la singola riga, il contratto newline-finale opzionale resta invariato e ora è pinnato da test). Motivazione: è l'unico criterio che misura direttamente la proprietà dichiarata con un solo confronto; la canonicità posizionale dimezzerebbe il vincolo in tre punti di parsing (ordine membri, ordine chiavi, forma numerica) e richiederebbe una prova d'equivalenza a parte sulle permutazioni. Chiude i tre difetti: (1) ordine chiavi payload riordinato dal BTreeMap, (2) ordine membri top-level dispacciato per nome contro l'ordine fisso SPEC §2, (3) -0 ammesso da parse_i64 ma emesso 0. Copre anche l'encoding delle stringhe già coperto da parse_string (N3) per la stessa verifica intera. Le forme numeriche già rifiutate strutturalmente (zeri in testa, +, esponenziale, spazi, null/true) non richiedono lavoro: verifica nel codice, test solo dove parse ammette oggi. Doc aggiornati (from_jsonl, Error::Schema, criterio N3 in parse_string). Nessun cambiamento a cycle.rs, state.rs, event.rs, payload.rs.
$ git status --short
(no output)
```

**Worktree pulito**: `git status --short` non stampa nulla (nessun file modificato, nessun untracked).

`cargo test` (workdir = worktree, nessun `cd`, nessuna pipe), **output integrale**:

```
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling generic-array v0.14.7
   Compiling cpufeatures v0.2.17
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.13s
     Running unittests src/lib.rs (target/debug/deps/ariadne-f4839cd99eb39cde)

running 4 tests
test event::tests::compute_hash_matches_documented_preimage ... ok
test event::tests::event_type_parse_is_total_and_strict ... ok
test log::tests::genesis_line_parses ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/l0_replay.rs (target/debug/deps/l0_replay-4d8c495979d3a6f9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_c1_genesis.rs (target/debug/deps/l0_c1_genesis-b9498d7e71f5ee2d)

running 2 tests
test test_genesis_rejects_seed_beyond_i64_max_no_event_no_wrap ... ok
test test_genesis_accepts_seed_i64_max ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_cycle.rs (target/debug/deps/l0_cycle-4f5d14b3bfa31aa5)

running 11 tests
test budget_draw_1_per_tick_0_per_la_policy ... ok
test mai_cicli_parziali_ritmo_sense_action_state ... ok
test fold_su_log_intero_con_ultima_snapshot_di_state ... ok
test seme_diverso_produce_log_diverso ... ok
test mossa_e_parse_stimolo_mod_3_su_piu_tick ... ok
test dopo_n_cicli_tick_n_e_catena_verificabile ... ok
test stimolo_oltre_i64_max_parse_completo_senza_mascheratura ... ok
test determinismo_forte_due_log_indipendenti_byte_identici ... ok
test un_ciclo_emette_3_eventi_stesso_tick_seq_consecutivi ... ok
test nessun_tick_salta_sequenza_0_1_1_1_2_2_2 ... ok
test stimolo_sola_stringa_di_cifre_senza_segno_ne_zeri_in_testa ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/debug/deps/l0_event_log-a17f73a430b2072f)

running 18 tests
test test_genesis_golden_hash_constant ... ok
test test_compensating_state_on_genesis_only_log_copies_genesis_snapshot ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_event_without_seq_is_rejected ... ok
test test_genesis_state_is_string_not_integer ... ok
test test_compensating_payload_carries_most_recent_state_snapshot_exact ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_genesis_state_matches_engine_state_decimal ... ok
test test_genesis_state_string_format ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_immutable_events_slice_borrows ... ok
test test_hash_chain_end_to_end ... ok
test test_unknown_top_level_member_rejected ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_n3_canonical_string.rs (target/debug/deps/l0_n3_canonical_string-561a2e6dfa793a36)

running 13 tests
test rifiuta_escape_solidus ... ok
test rifiuta_u_di_carattere_che_esce_grezzo ... ok
test rifiuta_surrogate_per_carattere_astrale ... ok
test canonical_genesis_1042_line_reserializes_byte_exact ... ok
test rifiuta_u_di_controllo_emesso_come_short_escape ... ok
test rifiuta_u_di_non_ascii ... ok
test rifiuta_escape_non_canonico_in_chiave ... ok
test rifiuta_u_di_quote ... ok
test rifiuta_u_maiuscolo_di_controllo_emesso_come_short_escape ... ok
test rifiuta_u_maiuscolo_di_controllo_emesso_come_u_minuscolo ... ok
test rifiuta_u_minuscolo_di_carattere_non_controllo ... ok
test canonical_counterpart_forms_are_accepted ... ok
test serializer_emission_set_is_pinned_and_round_trips_byte_exact ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_n3_line_canonical.rs (target/debug/deps/l0_n3_line_canonical-b87af8b5362d0725)

running 10 tests
test rifiuta_meno_zero_su_tick ... ok
test rifiuta_meno_zero_su_valore_payload ... ok
test rifiuta_meno_zero_su_seq ... ok
test rifiuta_permutazione_chiavi_payload ... ok
test riga_canonica_con_payload_byte_esatta ... ok
test contratto_newline_finale_invariato ... ok
test riga_canonica_payload_vuoto_byte_esatta ... ok
test riga_genesi_1042_accetta_e_byte_esatta ... ok
test rifiuta_permutazione_membri_top_level ... ok
test flight_log_via_api_round_trip_byte_per_byte ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_prng.rs (target/debug/deps/l0_prng-90961bfec5e735c0)

running 7 tests
test clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato ... ok
test clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect ... ok
test l_incremento_non_e_mai_pari ... ok
test interleaving_di_osservazioni_non_altera_la_sequenza ... ok
test stesso_seed_produce_sequenza_identica_da_istanze_indipendenti ... ok
test state_decimal_e_round_trippabile_e_senza_zero_in_testa ... ok
test il_corpus_l6_produce_sequenze_distinte_per_seme ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_replay_process.rs (target/debug/deps/l0_replay_process-37116a8ce7977c02)

running 6 tests
test state_nongenesi_pari_a_ticks_e_ultimo_tick_ticks ... ok
test output_non_vuoto_finisce_con_un_solo_newline ... ok
test output_ricaricabile_da_from_jsonl_e_verificato ... ok
test due_processi_stessi_argomenti_stdout_byte_identico ... ok
test argomenti_diversi_producono_byte_diversi ... ok
test argomenti_invalidi_exit_non_zero_stdout_vuoto ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

     Running tests/l0_state.rs (target/debug/deps/l0_state-ab94ffc3f9b3494f)

running 9 tests
test test_head_is_hash_of_last_event_not_genesis ... ok
test test_prng_read_from_last_state_snapshot_not_recomputed ... ok
test test_fold_is_pure_byte_identical ... ok
test test_counters_count_hand_built_sequence ... ok
test test_compensation_does_not_advance_prng ... ok
test test_fold_genesis_initial_state ... ok
test test_compensating_state_carries_last_snapshot_and_fold_reads_it ... ok
test test_fold_fails_on_broken_chain_before_any_verify ... ok
test test_real_log_every_state_carries_snapshot_and_fold_agrees ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ariadne

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Conteggio per binario** (`cargo test`, debug):

| Binario | passati | falliti | ignorati |
|---|---|---|---|
| `unittests src/lib.rs` | 4 | 0 | 0 |
| `unittests src/bin/l0_replay.rs` | 0 | 0 | 0 |
| `tests/l0_c1_genesis.rs` | 2 | 0 | 0 |
| `tests/l0_cycle.rs` | 11 | 0 | 0 |
| `tests/l0_event_log.rs` | 18 | 0 | 0 |
| `tests/l0_n3_canonical_string.rs` | 13 | 0 | 0 |
| `tests/l0_n3_line_canonical.rs` | **10** | 0 | 0 |
| `tests/l0_prng.rs` | 7 | 0 | 0 |
| `tests/l0_replay_process.rs` | 6 | 0 | 0 |
| `tests/l0_state.rs` | 9 | 0 | 0 |
| Doc-tests | 0 | 0 | 0 |
| **totale** | **80** | **0** | **0** |

**Attesi 80 = 70 precedenti + 10 di `l0_n3_line_canonical`: osservati 80.** Exit code 0 (nessun `error:`/`FAILED`, uscita pulita; il tool riporta l'uscita senza errore).

`cargo test --release`, **output integrale**:

```
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling generic-array v0.14.7
   Compiling cpufeatures v0.2.17
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis)
    Finished `release` profile [optimized] target(s) in 2.59s
     Running unittests src/lib.rs (target/release/deps/ariadne-e38db9ac10129f4e)

running 4 tests
test event::tests::event_type_parse_is_total_and_strict ... ok
test event::tests::compute_hash_matches_documented_preimage ... ok
test log::tests::gap_is_chain_error_not_schema_error ... ok
test log::tests::genesis_line_parses ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/l0_replay.rs (target/release/deps/l0_replay-8285744b5bb2c0d4)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_c1_genesis.rs (target/release/deps/l0_c1_genesis-4b9261e86831ba9a)

running 2 tests
test test_genesis_rejects_seed_beyond_i64_max_no_event_no_wrap ... ok
test test_genesis_accepts_seed_i64_max ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_cycle.rs (target/release/deps/l0_cycle-e0de9be28f6b7f0a)

running 11 tests
test budget_draw_1_per_tick_0_per_la_policy ... ok
test dopo_n_cicli_tick_n_e_catena_verificabile ... ok
test mai_cicli_parziali_ritmo_sense_action_state ... ok
test fold_su_log_intero_con_ultima_snapshot_di_state ... ok
test mossa_e_parse_stimolo_mod_3_su_piu_tick ... ok
test stimolo_oltre_i64_max_parse_completo_senza_mascheratura ... ok
test determinismo_forte_due_log_indipendenti_byte_identici ... ok
test nessun_tick_salta_sequenza_0_1_1_1_2_2_2 ... ok
test stimolo_sola_stringa_di_cifre_senza_segno_ne_zeri_in_testa ... ok
test un_ciclo_emette_3_eventi_stesso_tick_seq_consecutivi ... ok
test fold_su_log_intero_con_ultima_snapshot_di_state ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_event_log.rs (target/release/deps/l0_event_log-3c8b31b9bd7edeb8)

running 18 tests
test test_compensating_payload_carries_most_recent_state_snapshot_exact ... ok
test test_event_types_constrained_to_sense_action_state ... ok
test test_event_without_seq_is_rejected ... ok
test test_genesis_golden_hash_constant ... ok
test test_genesis_prev_hash_zeroes_and_shape ... ok
test test_compensating_event_payload_and_history_intact ... ok
test test_genesis_state_is_string_not_integer ... ok
test test_compensating_state_on_genesis_only_log_copies_genesis_snapshot ... ok
test test_genesis_state_matches_engine_state_decimal ... ok
test test_genesis_state_string_format ... ok
test test_hash_chain_end_to_end ... ok
test test_immutable_events_slice_borrows ... ok
test test_parser_rejects_non_canonical_payload_values ... ok
test test_payload_canonical_sorted_keys_minimal ... ok
test test_unknown_top_level_member_rejected ... ok
test test_replay_prefix_byte_per_byte ... ok
test test_seq_monotonic_from_zero_no_gaps ... ok
test test_log_is_append_only_no_mutation_truncation_or_reorder ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_n3_canonical_string.rs (target/release/deps/l0_n3_canonical_string-b4bd3f510be7243e)

running 13 tests
test rifiuta_escape_non_canonico_in_chiave ... ok
test canonical_genesis_1042_line_reserializes_byte_exact ... ok
test rifiuta_u_di_carattere_che_esce_grezzo ... ok
test rifiuta_surrogate_per_carattere_astrale ... ok
test canonical_counterpart_forms_are_accepted ... ok
test rifiuta_u_di_controllo_emesso_come_short_escape ... ok
test rifiuta_escape_solidus ... ok
test rifiuta_u_di_non_ascii ... ok
test rifiuta_u_di_quote ... ok
test rifiuta_u_maiuscolo_di_controllo_emesso_come_short_escape ... ok
test rifiuta_u_maiuscolo_di_controllo_emesso_come_u_minuscolo ... ok
test rifiuta_u_minuscolo_di_carattere_non_controllo ... ok
test serializer_emission_set_is_pinned_and_round_trips_byte_exact ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_n3_line_canonical.rs (target/release/deps/l0_n3_line_canonical-58a4c7a690328b02)

running 10 tests
test rifiuta_meno_zero_su_seq ... ok
test contratto_newline_finale_invariato ... ok
test rifiuta_meno_zero_su_tick ... ok
test rifiuta_permutazione_membri_top_level ... ok
test flight_log_via_api_round_trip_byte_per_byte ... ok
test riga_canonica_con_payload_byte_esatta ... ok
test rifiuta_permutazione_chiavi_payload ... ok
test riga_genesi_1042_accetta_e_byte_esatta ... ok
test riga_canonica_payload_vuoto_byte_esatta ... ok
test rifiuta_meno_zero_su_valore_payload ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_prng.rs (target/release/deps/l0_prng-1854648b393fc0c0)

running 7 tests
test clone_a_meta_percorso_riproduce_il_futuro_senza_side_effect ... ok
test clone_che_fa_gli_stessi_draw_ha_lo_stesso_stato ... ok
test interleaving_di_osservazioni_non_altera_la_sequenza ... ok
test il_corpus_l6_produce_sequenze_distinte_per_seme ... ok
test l_incremento_non_e_mai_pari ... ok
test stesso_seed_produce_sequenza_identica_da_istanze_indipendenti ... ok
test state_decimal_e_round_trippabile_e_senza_zero_in_testa ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/l0_replay_process.rs (target/release/deps/l0_replay_process-2552478134406c3c)

running 6 tests
test output_non_vuoto_finisce_con_un_solo_newline ... ok
test state_nongenesi_pari_a_ticks_e_ultimo_tick_ticks ... ok
test output_ricaricabile_da_from_jsonl_e_verificato ... ok
test due_processi_stessi_argomenti_stdout_byte_identico ... ok
test argomenti_diversi_producono_byte_diversi ... ok
test argomenti_invalidi_exit_non_zero_stdout_vuoto ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.29s

     Running tests/l0_state.rs (target/release/deps/l0_state-7b2276bab614da0e)

running 9 tests
test test_compensating_state_carries_last_snapshot_and_fold_reads_it ... ok
test test_compensation_does_not_advance_prng ... ok
test test_counters_count_hand_built_sequence ... ok
test test_fold_genesis_initial_state ... ok
test test_head_is_hash_of_last_event_not_genesis ... ok
test test_prng_read_from_last_state_snapshot_not_recomputed ... ok
test test_fold_is_pure_byte_identical ... ok
test test_fold_fails_on_broken_chain_before_any_verify ... ok
test test_real_log_every_state_carries_snapshot_and_fold_agrees ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests ariadne

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Conteggio per binario** (`cargo test --release`): lib 4, bin `l0_replay` 0, `l0_c1_genesis` 2, `l0_cycle` 11, `l0_event_log` 18, `l0_n3_canonical_string` 13, `l0_n3_line_canonical` **10**, `l0_prng` 7, `l0_replay_process` 6, `l0_state` 9, doc-tests 0 → **totale 80 passati, 0 falliti, 0 ignorati**. Exit code 0.

> Nota di trascrizione: nel blocco `--release` sopra, il test `l0_cycle` compare (nell'output del tool) con nomi in ordine non deterministico e — per come l'output è stato restituito — con `fold_su_log_intero_con_ultima_snapshot_di_state` elencato due volte; il conteggio ufficiale è quello del binario: **11 passed; 0 failed**. In debug lo stesso binario elenca 11 nomi distinti.

---

## 12.2 Passo 2 — replay bit-perfetto a due processi

```
$ cargo test --test l0_replay_process -- --nocapture
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running tests/l0_replay_process.rs (target/debug/deps/l0_replay_process-37116a8ce7977c02)

running 6 tests
test output_non_vuoto_finisce_con_un_solo_newline ... ok
test output_ricaricabile_da_from_jsonl_e_verificato ... ok
test state_nongenesi_pari_a_ticks_e_ultimo_tick_ticks ... ok
test due_processi_stessi_argomenti_stdout_byte_identico ... ok
test argomenti_diversi_producono_byte_diversi ... ok
test argomenti_invalidi_exit_non_zero_stdout_vuoto ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s
```

- **6/6 verdi**, exit 0. Nessuna riga stampata dai test: `--nocapture` non rivela output perché i test non fanno `println!` — il confronto byte-per-byte avviene in-process.
- **Artifatti**: il test **non scrive file** (nessun `File::create`/`fs::write` nel sorgente, `tests/l0_replay_process.rs:1-185`): non ci sono digest da riportare. La genuinità della prova è nel sorgente del test: `:34-39` lancia il binario con `Command::new(BIN).args(args).output()`; `:49-63` invoca **due volte** `lancia(&["--seed","1042","--ticks","5"])` (due processi distinti) e asserisce `a.stdout == b.stdout` byte per byte (`:59-62`); `:81-95` ricarica l'intero stdout con `Log::from_jsonl`, `verify()` e asserisce `replay.to_jsonl() == s` (replay bit-perfetto attraverso la serializzazione); `:75` asserisce l'assenza di `\r` nel corpo.
- **NOTA (permesso)**: la richiesta «se il test produce artefatti, riporta i digest» non è applicabile, e la produzione di artefatti *miei* è comunque impossibile: l'esecuzione diretta del binario è **negata** —
  ```
  $ target/debug/l0_replay --seed 1042 --ticks 5
  {"error":{"type":"permission.rejected","message":"Permission denied: shell"},"content":[]}
  ```
  Comando esatto per chiuderla: `target/debug/l0_replay --seed 1042 --ticks 5 > /tmp/a.jsonl` (redirezione, non ammessa nel mio perimetro) seguito da `shasum -a 256 /tmp/a.jsonl`. Nel perimetro ammesso, la prova è il test eseguito qui sopra.

---

## 12.3 Passo 3 — rosso provato davvero su `edcca4a`

`git checkout edcca4a` dentro il worktree è **negato dai permessi**:
```
$ git checkout edcca4a
{"error":{"type":"permission.rejected","message":"Permission denied: shell"},"content":[]}
```
**Metodo alternativo usato** (equivalente, nessun `cd`, nessuna pipe): un **secondo worktree** in detached HEAD sul commit rosso:
```
$ git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis_red edcca4a
Preparing worktree (detached HEAD edcca4a)
HEAD is now at edcca4a L0 N3-bis: test ROSSI per la canonicità di RIGA — serialize(parse(s)) == s per ogni riga che from_jsonl accetta. Tre teste rosse sui tre difetti resi canonici solo al token-stringa da N3: permutazione delle chiavi del payload (3 ordini non alfabetici), permutazione dei membri top-level (4 disposizioni fuori dall'ordine SPEC §2), -0 nel payload (il parse lo ammette, il serializzatore emette 0). Controparti verdi: righe canoniche accettate e byte-per-byte su STATE con payload, SENSE/ACTION con payload vuoto, genesi 1042; contratto del '\n' finale (con o senza) pinnato; round-trip via API su log intero. Solo test: nessun codice di produzione toccato.
```

`cargo test --test l0_n3_line_canonical` nel worktree a `edcca4a`, **output integrale** (exit code **101**):

```
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling generic-array v0.14.7
   Compiling cpufeatures v0.2.17
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis_red)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.18s
     Running tests/l0_n3_line_canonical.rs (target/debug/deps/l0_n3_line_canonical-b87af8b5362d0725)

running 10 tests
test rifiuta_meno_zero_su_seq ... ok
test rifiuta_meno_zero_su_tick ... ok
test contratto_newline_finale_invariato ... ok
test riga_canonica_con_payload_byte_esatta ... ok
test rifiuta_meno_zero_su_valore_payload ... FAILED
test rifiuta_permutazione_chiavi_payload ... FAILED
test rifiuta_permutazione_membri_top_level ... FAILED
test riga_canonica_payload_vuoto_byte_esatta ... ok
test flight_log_via_api_round_trip_byte_per_byte ... ok
test riga_genesi_1042_accetta_e_byte_esatta ... ok

failures:

---- rifiuta_meno_zero_su_valore_payload stdout ----

thread 'rifiuta_meno_zero_su_valore_payload' (1301732) panicked at tests/l0_n3_line_canonical.rs:144:5:
-0 nel payload → rifiuto atteso, trovato Ok("{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{\"gain\":0},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- rifiuta_permutazione_chiavi_payload stdout ----

thread 'rifiuta_permutazione_chiavi_payload' (1301733) panicked at tests/l0_n3_line_canonical.rs:64:9:
ordine chiavi payload non canonico ({"stimolo":42,"gain":6}) → rifiuto atteso, trovato Ok("{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{\"gain\":6,\"stimolo\":42},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n")

---- rifiuta_permutazione_membri_top_level stdout ----

thread 'rifiuta_permutazione_membri_top_level' (1301734) panicked at tests/l0_n3_line_canonical.rs:100:9:
ordine membri top-level non canonico ({"tick":7,"seq":1,"type":"STATE","payload":{"gain":6},"prev_hash":"0000000000000000000000000000000000000000000000000000000000000000","hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}) → rifiuto atteso, trovato Ok("{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{\"gain\":6},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n")

failures:
    rifiuta_meno_zero_su_valore_payload
    rifiuta_permutazione_chiavi_payload
    rifiuta_permutazione_membri_top_level

test result: FAILED. 7 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `--test l0_n3_line_canonical`

Exited with code 101
```

**Primo errore rosso, verbatim** (`tests/l0_n3_line_canonical.rs:144:5`):

```
thread 'rifiuta_meno_zero_su_valore_payload' (1301732) panicked at tests/l0_n3_line_canonical.rs:144:5:
-0 nel payload → rifiuto atteso, trovato Ok("{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{\"gain\":0},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n")
```

> L'ordine di esecuzione dei 10 test è quello restituito dal tool; il «primo errore» riportato è il primo blocco `---- … stdout ----` del resoconto dei fallimenti, cioè `rifiuta_meno_zero_su_valore_payload`. Gli altri due fallimenti sono in §12.3 sopra, integrali.
> Nota di forma: la riga incriminata è `{\"seq\":1,…,payload\":{\"gain\":-0}…}` in ingresso e il valore restituito è `…\"payload\":{\"gain\":0}…`: `parse_i64` accetta `-0`, il serializzatore emette `0`. Il rosso è esattamente il difetto (3) dichiarato dal producer.

**Ritorno a `a02b66a`**: il worktree principale del gate (`l0n3bis`) **non è mai stato spostato** (`git checkout` negato, quindi nessuno stato da ripristinare); per il rosso ho usato il worktree separato `l0n3bis_red`. Il worktree `l0n3bis` resta quindi a `a02b66a` per tutta la durata del gate.

---

## 12.4 Passo 4 — test-first

Cosa contiene il primo commit del task, `edcca4a`:

```
$ git show --stat edcca4a
commit edcca4ae7ef2dfac2b2c27780dbb21e08fa63309
Author: Alessandro Spinabelli <Alessandro.spinabelli@studenti.unipr.it>
Date:   Thu Oct 22 19:39:39 2026 +0200

    L0 N3-bis: test ROSSI per la canonicità di RIGA — …(messaggio integrale sopra)…

 tests/l0_n3_line_canonical.rs | 213 ++++++++++++++++++++++++++++++++++++++++++
 1 file changed, 213 insertions(+)
```

**Un solo file, sotto `tests/`, 213 inserzioni, 0 rimozioni**: nessun file di `src/`, `Cargo.toml` o altro. **VERDE**.

Che i test in quel commit fossero **rossi** è provato al §12.3 (3 `FAILED`, exit 101 eseguito sul worktree a `edcca4a`).

Che il codice di produzione non sia stato toccato *dopo* l'ultimo commit dei test:

```
$ git diff --stat edcca4a a02b66a -- tests/
(no output)
```

**Diff vuoto**: fra il commit rosso e l'implementazione **nessun file di `tests/` è cambiato** — i test verificati verdi a `a02b66a` sono esattamente quelli rossi a `edcca4a`. **VERDE**.

L'implementazione, per contro, tocca solo `src/`:

```
$ git show --stat a02b66a
commit a02b66aa2e5c85ad0d51a0806887747ad49f275d
    L0 N3-bis: canonicità di RIGA — …
 src/log.rs | 40 ++++++++++++++++++++++++++++++++++++----
 1 file changed, 36 insertions(+), 4 deletions(-)
```

Ordine dei commit (`git log --oneline -4 a02b66a`, messaggi troncati):

```
a02b66a L0 N3-bis: canonicità di RIGA — from_jsonl ammette solo l'encoding completo …
edcca4a L0 N3-bis: test ROSSI per la canonicità di RIGA — serialize(parse(s)) == s …
fba6565 L0 N3-fix: import EventType inutilizzato rimosso dal test N3 …
1b453f9 L0 N4: test maintenance — fixture genesi portata alla forma canonica attuale …
```

`edcca4a` è il **padre diretto** di `a02b66a`. Il diff completo di `a02b66a` su `src/log.rs` (`git show a02b66a -- src/log.rs`) mostra 4 hunk: 2 di documentazione (`Error::Schema`, mod-doc di `from_jsonl`), 1 sostituzione del punto di chiamata (`events.push(canonical_line_or_err(line, parse_event(line)?)?)` in luogo di `events.push(parse_event(line)?)`) e 1 funzione nuova `canonical_line_or_err`. **La guardia `if input.contains('\r')` compare come riga di CONTESTO, non come riga aggiunta** (vedi §12.6.2).

---

## 12.5 Mnemosyne (richiamata e usata, non decorativa)

- `memory_open_session(projectId: "opencode:Ariadne", taskTitle: "L0 gate N3-bis")` → `sessionId = ses_1bf27cb8-9183-4fb5-9006-8c268143fcf1`, `corpusRevision = 2671e896-8c0d-4abf-922c-ea4538ab0294:5354`.
- `memory_context(sessionId, "L0 parser canonico")` → `requiredContextComplete: true`, `degradations: []`, `conflicts: []`, 8 ricordi. Id richiamati:
  - `mem_851b8059-a7f0-4890-873d-31ee18021c94` — payload canonico: chiavi in ordine alfabetico (BTreeMap), compatto, escape standard ✓ confermato dal caso 1c/1d e dal caso 5.
  - `mem_6cadd392-a84e-464e-a301-317f3678f90c` — ordine membri top-level `{seq, tick, type, payload, prev_hash, hash}` come da SPEC §2; parser rigoroso ✓ è esattamente ciò che N3-bis ora fa rispettare anche in *lettura*.
  - `mem_3adbd62d-d5b0-4aad-b13a-1d8661222ea9` — preimage con 0x1F ✓ rilevante per il caso 4 (hash).
  - `mem_08516980-3666-4abf-a8dc-e0f97f96eecc`, `mem_c804d06f-7d08-4542-a8cd-c120519fc302` — forma della riga genesi (state stringa decimale) ✓ usata nel caso 4c/4d e nel caso 5.
  - `mem_58656f04-b0c6-402d-a362-d59232d861aa`, `mem_7b37a78d-7145-499c-a139-d0572e393178`, `mem_076c3ace-4b2a-4759-bcd1-ea94b5ba9698`, `mem_4e59a7ca-ff5c-491b-98a1-808b6fbecc2c`, `mem_97b6089c-270f-450f-9082-bc606df15d5e`, `mem_e5d32c28-0108-406e-85bd-81504a88f85c`.
- `memory_search(sessionId, "L0 violazione")` → 11 item; **nessun pattern di violazione specifico di L0** fra i risultati: gli id restituiti sono le stesse decisioni (`mem_851b8059`, `mem_3adbd62d`, `mem_7b37a78d`, `mem_6cadd392`, `mem_97b6089c`, `mem_e5d32c28`, `mem_58656f04`, `mem_4e59a7ca`, `mem_08516980`, `mem_c804d06f`, `mem_076c3ace`). Non c'era quindi un pattern noto da riconfermare; il pattern nuovo è proposto a fine gate (§12.9).

---

## 12.6 La caccia al buco — i 5 casi limite, con esito OSSERVATO

Metodo: nessun file di test può essere aggiunto al repository né al worktree di gate. Ho quindi usato la **istanza di scratch** autorizzata, creando un **terzo worktree usa-e-getta** in scratch e aggiungendovi **un solo file nuovo, mai esistente nel repository**, dichiarato qui:

```
$ git worktree add /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis_probe a02b66a
Preparing worktree (detached HEAD a02b66a)
HEAD is now at a02b66a L0 N3-bis: canonicità di RIGA — …
```

File aggiunto (unico cambiamento, dentro il worktree di scratch; **nessun file di produzione o di test del repository è stato toccato**):
`tests/zz_probe_n3bis_scratch.rs` — un probe **che non asserisce nulla**, si limita a stampare `ACCETTATO -> <riesilizzazione>` / `RIFIUTATO -> <errore>` per ogni input. Il worktree è stato poi distrutto (§12.9); il file non esiste in alcun commit.

Comando e output **integrale**:

```
$ cargo test --test zz_probe_n3bis_scratch -- --nocapture
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling libc v0.2.191
   Compiling cfg-if v1.0.5
   Compiling generic-array v0.14.7
   Compiling cpufeatures v0.2.17
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling digest v0.10.7
   Compiling sha2 v0.10.9
   Compiling ariadne v0.1.0 (/private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis_probe)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.32s
     Running tests/zz_probe_n3bis_scratch.rs (target/debug/deps/zz_probe_n3bis_scratch-84fb54e6ed54a09c)

running 1 test
=== 1. CHIAVI DUPLICATE ===
1a seq duplicato (seq:1,seq:2): RIFIUTATO -> Schema("membro duplicato: seq")
1b hash duplicato: RIFIUTATO -> Schema("membro duplicato: hash")
1c chiave payload duplicata (gain:1,gain:2): RIFIUTATO -> Schema("chiave payload duplicata: gain")
1d chiave payload duplicata NON adiacente (a,b,a): RIFIUTATO -> Schema("chiave payload duplicata: a")
=== 2. CRLF ===
2a riga canonica + CRLF: RIFIUTATO -> Schema("CR vietato: nel log canonico solo LF")
2b riga canonica + CRLF + LF: RIFIUTATO -> Schema("CR vietato: nel log canonico solo LF")
2c riga canonica SENZA LF ma con CR finale: RIFIUTATO -> Schema("CR vietato: nel log canonico solo LF")
2d CR dentro una stringa payload (escape \r canonico, non CR grezzo): ACCETTATO -> "{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{\"s\":\"a\\rb\"},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n"
=== 3. NEWLINE FINALE ASSENTE ===
3a canonica senza LF finale: ACCETTATO -> "{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n"
3b canonica con LF finale: ACCETTATO -> "{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n"
3c due righe, LF solo sull'ultima: ACCETTATO -> "{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n{\"seq\":2,\"tick\":8,\"type\":\"STATE\",\"payload\":{},\"prev_hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n"
3d due righe, LF finale assente: ACCETTATO -> "{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n{\"seq\":2,\"tick\":8,\"type\":\"STATE\",\"payload\":{},\"prev_hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n"
=== 4. HASH INCOERENTE SU RIGA CANONICA ===
4a from_jsonl: ACCETTATO -> "{\"seq\":1,\"tick\":7,\"type\":\"STATE\",\"payload\":{\"gain\":6},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n"
4b verify() sul log accettato -> Err(Chain("seq non monotono da 0 senza gap: posizione 0, seq 1"))
4c genesi canonica corretta: ACCETTATA, verify -> Ok(())
4d genesi canonica con hash ERRATO: from_jsonl ACCETTA -> "{\"seq\":0,\"tick\":0,\"type\":\"STATE\",\"payload\":{\"prng\":\"PCG64\",\"seed\":1042,\"state\":\"43149974864253806286336044685702662359\"},\"prev_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"hash\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}\n"
4e genesi canonica con hash ERRATO: verify() -> Err(Chain("hash incoerente a seq 0: atteso 4649d1f4889db0378e963bd51f9d787eea98dc42886e66e51973f28e593526b7, registrato aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"))
=== 5. UNA RIGA CANONICA PER OGNI TIPO DI EVENTO ===
STATE payload vuoto: ACCETTATO, round-trip byte-identico = true
STATE payload pieno: ACCETTATO, round-trip byte-identico = true
SENSE payload vuoto: ACCETTATO, round-trip byte-identico = true
SENSE payload pieno: ACCETTATO, round-trip byte-identico = true
ACTION payload vuoto: ACCETTATO, round-trip byte-identico = true
ACTION payload pieno: ACCETTATO, round-trip byte-identico = true
STATE payload chiavi miste: ACCETTATO, round-trip byte-identico = true
5h type fuori dominio: RIFIUTATO -> Schema("type fuori dominio: FOO")
test probe_casi_limite ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### 12.6.1 Caso 1 — chiavi duplicate nella stessa riga → **VERDE**

Input esatti:
- 1a `{"seq":1,"seq":2,"tick":7,"type":"STATE","payload":{},"prev_hash":"0"×64,"hash":"a"×64}` + `\n`
- 1b `"hash"` due volte; 1c/1d `"gain"`/`"a"` due volte nel payload (adiacenti e non adiacenti).

Esito osservato: **tutte RIFIUTATE**, `Error::Schema`. Meccanismo, citato: `src/log.rs:305-310` — la macro `set_slot!` rifiuta il membro top-level duplicato (`"membro duplicato: {}"`), invocata per tutti e sei i membri (`:340`, `:341`, `:346`, `:348`, `:352`, `:357`); per il payload `src/log.rs:407-409` — `if pairs.iter().any(|(k, _)| *k == key)` → `"chiave payload duplicata: {key}"` (1d prova che il controllo è su tutte le coppie, non solo sulla precedente). **Domanda «quale vince» priva di oggetto: nessuna vince, la riga non entra.** Coperto anche dal test preesistente `tests/l0_event_log.rs:236,240` (`{"gain":5,"gain":5}` → rifiutato).

### 12.6.2 Caso 2 — CRLF → **VERDE (rifiuto), con NOTA di contratto**

Input: riga canonica + `"\r\n"` (2a), + `"\r\n\n"` (2b), + solo `"\r"` (2c), e — controprova — payload con `\r` **escapato** (2d). Esito osservato: **2a/2b/2c RIFIUTATE** con `Schema("CR vietato: nel log canonico solo LF")`; **2d ACCETTATA** e riserializzata identica.

Il `\r` **non resta in nessun token**: il rifiuto è un `contains('\r')` sull'*intero input*, prima di qualunque split (`src/log.rs:207-209`), quindi il `\r` non arriva mai a `parse_event`. In 2d il `\r` è dentro una stringa JSON come sequenza di due byte `\`+`r`, non come CR grezzo: correttamente ammesso (escape canonico, `mem_851b8059`).

**Cambio di contratto? NO.** Nel diff `a02b66a -- src/log.rs` la riga `if input.contains('\r') {` e la successiva compaiono **precedute da spazio (righe di contesto)**, non da `+`: la guardia CR esisteva già prima di N3-bis. La stessa formulazione è nella mod-doc di `from_jsonl` in entrambe le versioni (prima: *«Accetta solo linee canoniche con '\n' finale; CR e righe vuote interne sono rifiutate»*, `git show a02b66a -- src/log.rs`, hunk 2, riga rimossa; dopo: `src/log.rs:199-205`). **Nessun cambio non dichiarato.**

### 12.6.3 Caso 3 — newline finale assente → **VERDE (contratto invariato, verificato su chiamanti e su entrambe le forme)**

Esiti osservati: `3a` (senza LF) **ACCETTATA**, `3b` (con LF) **ACCETTATA**, `3c`/`3d` (due righe, con e senza LF finale) **ACCETTATE**; in tutti i casi `to_jsonl()` **ri-aggiunge** il `\n` finale. Proprietà esatta, dichiarata in `src/log.rs:204-205`: *«L'input senza newline finale resta accettato: l'ultima riga è comunque un evento, il contratto non cambia»*; il meccanismo è `src/log.rs:212-219` (`is_last` → `break` sulla riga vuota finale).

Chiamanti, letto tutto il repository (`grep from_jsonl`):
- `src/`: **nessun chiamante di produzione**. `from_jsonl` è chiamata solo da test e da doc (`src/log.rs:206` definizione; `src/log.rs:581` unit test). Il binario `src/bin/l0_replay.rs` **non la chiama affatto** (nessun match in quel file): l'harness diagnostico emette soltanto `to_jsonl()`.
- Test: `tests/l0_replay_process.rs:86,109` (ricarica lo stdout del binario, che ha LF finale), `tests/l0_cycle.rs:116,162,381`, `tests/l0_event_log.rs:276,280,297,304,310,313,331`, `tests/l0_state.rs:89,248,263,274`, e il nuovo `tests/l0_n3_line_canonical.rs:43,99,120,133,169,177,187,210`. Il caso «senza LF» è pinnato dal test nuovo `contratto_newline_finale_invariato` (`tests/l0_n3_line_canonical.rs:185-189`), **verde** nella suite di §12.1.

**NOTA (asimmetria dichiarata, non un buco)**: la proprietà `serialize(parse(s)) == s` vale **riga per riga**; sull'*intero input* vale solo se l'input termina con `\n` (in `3a`/`3d` `to_jsonl(from_jsonl(s)) != s` per il solo `\n` aggiunto). È la forma del contratto dichiarato dal producer nel messaggio di `a02b66a` («il contratto newline-finale opzionale resta invariato e ora è pinnato da test») e nella mod-doc: **dichiarato, quindi non ROSSO.** Chi volesse la canonicità *anche* dell'intero input senza LF dovrebbe cambiare il contratto — decisione non mia.

### 12.6.4 Caso 4 — riga canonica con `hash` incoerente → **VERDE (accettata dal parse, rifiutata da `verify`)**

Esiti osservati: `4d` genesi canonica con hash sostituito da `"a"×64` → `from_jsonl` **ACCETTA**; `4e` `verify()` → `Err(Chain("hash incoerente a seq 0: atteso 4649d1f4…26b7, registrato aaaa…aaaa"))`. `4c` genesi corretta → `verify() Ok(())`. `4a/4b` riga singola `seq:1` con hash inventato → `from_jsonl` accetta, `verify()` fallisce con `Chain("seq non monotono da 0 senza gap: posizione 0, seq 1")`.

**Argomento (canonicità ≠ semantica): è corretto, non è un buco.** La canonicità è una proprietà dell'*encoding* (bytes → struttura): `canonical_line_or_err` (`src/log.rs:254-265`) confronta la riga con la riserializzazione dello *stesso* evento, quindi decide su byte di forma, non su valori; l'hash memorizzato è un *campo*, e la riga di 4d **è** esattamente ciò che `event_line` emette per quell'evento. La verifica della catena è un'altra funzione, con un altro esito, dichiarata: mod-doc `src/log.rs:21` (*«`from_jsonl` è validazione di SCHEMA SOLA: non verifica la catena; la verifica … `verify`»*), `src/log.rs:198` e `:199`. Se il parse rifiutasse gli hash incoerenti, il controllo della catena sarebbe accorpato allo schema e — nota — `verify()` non potrebbe più distinguere i tre guasti che oggi distingue (payload, hash, prev_hash: `tests/l0_event_log.rs:295-310`). La separazione è già provata dai test esistenti (`tests/l0_event_log.rs:297,304,310` accettano con `.unwrap()` e poi pretendono `verify().is_err()`).

### 12.6.5 Caso 5 — una riga canonica per ogni tipo di evento → **VERDE**

Esiti osservati: `STATE` (vuoto, pieno, chiavi miste `{"alpha":"s","zeta":-3}`), `SENSE` (vuoto, pieno), `ACTION` (vuoto, pieno) → **tutti ACCETTATI con `round-trip byte-identico = true`**; `"type":"FOO"` → **RIFIUTATO** `Schema("type fuori dominio: FOO")`. La SPEC ammette solo `SENSE|ACTION|STATE` (`src/log.rs:344-345`, `EventType::parse`), quindi il perimetro di questo caso è coperto per intero, tipo fuori dominio compreso. Coperto anche dai test nuovi `riga_canonica_payload_vuoto_byte_esatta` (SENSE/ACTION, `tests/l0_n3_line_canonical.rs:163-173`), `riga_canonica_con_payload_byte_esatta` (`:156-161`), `riga_genesi_1042_accetta_e_byte_esatta` (`:175-180`), `flight_log_via_api_round_trip_byte_per_byte` (`:193-213`) — tutti **verdi** in §12.1.

### 12.6.6 Direzione «accettato e non canonico» — nessun esito trovato, e impossibile per costruzione

Nessuno dei casi osservati è «accettato e non canonico». Non è un caso: `from_jsonl` accetta la riga **solo** se essa è byte-uguale a `event_line(parse_event(line))` (`src/log.rs:220` + `:254-265`), e `to_jsonl` usa la **stessa** `event_line` (`src/log.rs:189-196` → `:231-243`). Quindi per ogni riga accettata `serialize(parse(s)) == s` vale *identicamente*: il buco nella direzione indicata non può esistere finché la funzione di confronto e quella di emissione restano la stessa. L'unica asimmetria residua è quella — dichiarata — del `\n` finale (§12.6.3): riga per riga, mai.

---

## 12.7 Isolamento e test che il producer dichiara (regola 6-7 del mandato)

### 12.7.1 Isolamento — nessun accesso a input non dichiarati

`grep` su `src/` per `std::fs|std::net|SystemTime|Instant|std::env|File::|TcpStream|include_str|include_bytes|env!|option_env` → **un solo match**:

```
/private/var/.../l0n3bis/src/bin/l0_replay.rs:
  Line 79:     let args: Vec<String> = std::env::args().skip(1).collect();
```

cioè l'harness diagnostico dichiarato (SPEC §5), invariato. **Nessuna violazione introdotta da N3-bis**: il diff di `a02b66a` tocca solo `src/log.rs` e non aggiunge alcun accesso esterno (le 36 righe aggiunte sono doc + `canonical_line_or_err`, `git show a02b66a -- src/log.rs`); `Cargo.toml` non è nel diff (`git show --stat a02b66a` elenca **solo** `src/log.rs`). Il test nuovo è puro calcolo in memoria: costruisce le righe con `format!` da costanti (`tests/l0_n3_line_canonical.rs:22-39`), nessun file, nessun ambiente.

### 12.7.2 I tre test che il mandato pretende esistano

| Test preteso | Esiste? | Prova |
|---|---|---|
| replay del prefisso storico byte-per-byte | **SÌ** | `tests/l0_event_log.rs:316-338` (`test_replay_prefix_byte_per_byte`): costruisce due volte lo stesso log (`:319-327`), asserisce `jsonl_a == jsonl_b` byte per byte (`:328`), poi `from_jsonl(&jsonl_a)` e `replayed.to_jsonl() == jsonl_a` byte per byte (`:331-332`), `verify()` ok (`:333`). **Verde** in §12.1. |
| test di immutabilità che fallisce se un evento vecchio cambia | **SÌ** | `tests/l0_event_log.rs:287-314` (`test_log_is_append_only_no_mutation_truncation_or_reorder`): (1) cambia un payload vecchio → `verify().is_err()` (`:295-298`); (2) cambia l'`hash` memorizzato di un evento vecchio → `verify().is_err()` (`:300-304`); (3) cambia il `prev_hash` memorizzato → `verify().is_err()` (`:306-310`); (4) il log non toccato resta valido (`:312-313`). **Verde** in §12.1. |
| test che vieta un evento senza `seq` di origine | **SÌ** | `tests/l0_event_log.rs:258-266` (`test_event_without_seq_is_rejected`): rimuove `"seq":0,` dalla riga genesi → `is_err()` (`:261-262`); la variante `"seq":,` (membro presente ma senza valore) → `is_err()` (`:264-265`). **Verde** in §12.1. Il rifiuto ha anche la sua causa nel codice: `set_slot!` + il match finale `_ => Err(Error::Schema("membro mancante: seq|tick|type|payload|prev_hash|hash"))` (`src/log.rs:366-378`). |

**Nessuna assenza di prova**: le tre voci sono VERDI con citazione e i tre file sono eseguiti verdi nella suite di §12.1 (binari `l0_event_log`, 18/18).

---

## 12.8 Riepilogo §12 (N3-bis)

| # | Voce | Esito | Prova |
|---|---|---|---|
| 1 | worktree pulito + `cargo test` a `a02b66a`, 80 test attesi | **VERDE** | §12.1: `git status --short` vuoto; 80 passed / 0 failed / 0 ignored, conteggio per binario; `l0_n3_line_canonical` 10/10 |
| 2 | `cargo test --release` a `a02b66a` | **VERDE** | §12.1: 80 passed / 0 failed, exit 0 |
| 3 | replay bit-perfetto a due processi | **VERDE** | §12.2: `cargo test --test l0_replay_process -- --nocapture` → 6/6 ok; il test confronta due `Command::new` distinti byte per byte (`tests/l0_replay_process.rs:34-39,49-63`) e riprova il round-trip intero (`:82-95`). Nessun artefatto prodotto dal test → nessun digest da riportare (NOTA di perimetro, esecuzione diretta del binario negata) |
| 4 | rosso provato su `edcca4a` | **VERDE** | §12.3: `cargo test --test l0_n3_line_canonical` → **7 passed; 3 failed**, exit **101**; primo errore verbatim riportato. Metodo: secondo worktree (il `git checkout` nel worktree è negato dai permessi — NOTA, non ROSSO) |
| 5 | primo commit del task = solo test, e rossi | **VERDE** | §12.4: `edcca4a` = 1 file, `tests/l0_n3_line_canonical.rs`, 213 `+`, 0 `-`; diff vuoto `edcca4a a02b66a -- tests/` |
| 6 | solo test tra commit rosso e implementazione | **VERDE** | §12.4: `git diff --stat edcca4a a02b66a -- tests/` → nessun output; l'impl tocca solo `src/log.rs` (36 `+`, 4 `-`) |
| 7 | criterio «accettato e non canonico» | **VERDE** (nessun controesempio) | §12.6: 5 casi osservati via probe in scratch; §12.6.6: impossibilità per costruzione (stessa `event_line`) |
| 8 | caso 1, chiavi duplicate | **VERDE** (rifiutate) | §12.6.1: 4/4 RIFIUTATE, `src/log.rs:305-310,407-409` |
| 9 | caso 2, CRLF | **VERDE** (rifiutate) + NOTA | §12.6.2: 2a/2b/2c rifiutate, 2d accettata; guardia `\r` **preesistente** (riga di contesto nel diff) → nessun cambio di contratto |
| 10 | caso 3, newline finale assente | **VERDE** + NOTA | §12.6.3: accettato con e senza `\n`, `to_jsonl` ri-aggiunge; contratto dichiarato `src/log.rs:204-205,212-219`; nessun chiamante di produzione di `from_jsonl`; asimmetria sull'intero input dichiarata |
| 11 | caso 4, hash incoerente su riga canonica | **VERDE** | §12.6.4: `from_jsonl` accetta (`4d`), `verify()` rifiuta `Chain` (`4e`); schema ≠ catena, `src/log.rs:21,198` |
| 12 | caso 5, una riga canonica per ogni tipo | **VERDE** | §12.6.5: STATE/SENSE/ACTION, vuoto e pieno, round-trip byte-identico = true; `FOO` rifiutato |
| 13 | isolamento | **VERDE** | §12.7.1: un solo accesso esterno in `src/`, `src/bin/l0_replay.rs:79`, invariato e dichiarato |
| 14 | test dichiarati (prefisso storico / immutabilità / evento senza `seq`) | **VERDE** | §12.7.2: `tests/l0_event_log.rs:316-338`, `:287-314`, `:258-266`, tutti eseguiti verdi |

**Esito complessivo del gate §12: NON dichiarato** (non spetta al tester). Nessuna voce ROSSA; **3 NOTE**: (a) esecuzione diretta del binario negata dai permessi (§12.2), (b) `git checkout` nel worktree negato → usato un secondo worktree (§12.3), (c) asimmetria del `\n` finale sull'intero input, dichiarata dal producer e pinnata da test (§12.6.3).

---

## 12.9 Pulizia e memoria

Worktree di questo gate, tutti creati da me e tutti rimossi:

```
$ git worktree remove /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis_red --force
$ git worktree remove /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis_probe --force
$ git worktree remove /private/var/folders/m4/4kk50jhs1qq0b7wn6csgfjrr0000gn/T/opencode/l0n3bis --force
```

(`--force` necessario per i due che contengono artefatti non tracciati: `target/` e, per il probe, il file di probe dichiarato in §12.6. Il worktree principale del repository non è stato toccato.)

Mnemosyne:
- sessione aperta e usata: `ses_1bf27cb8-9183-4fb5-9006-8c268143fcf1` (§12.5).
- memoria richiamata: `memory_context` + `memory_search` → id elencati in §12.5; **nessun pattern di violazione preesistente su L0**.
- memoria proposta: `memory_propose(kind: "convention", scope: {project: "opencode:Ariadne"}, epistemicBasis: "verified")` → **`mem_9a97634e-7f7e-4627-a9a8-0e89b59e6e25`** (stato `pending_approval`), contenuto: canonicità di riga = confronto byte-per-byte con la stessa `event_line` di `to_jsonl`, quindi «accettato e non canonico» impossibile per costruzione; le due asimmetrie dichiarate (`\n` finale opzionale, CR grezzo rifiutato globalmente e preesistente); chiavi duplicate = rifiuto strutturale; hash incoerente = accettato dal parse e rifiutato da `verify`.
- sessione chiusa: `memory_close_session(ses_1bf27cb8-9183-4fb5-9006-8c268143fcf1)`.

**Report**: `/Users/spina/Ariadne/reports/L0_n3_n4_test_report.md` (le §0–§11 restano intatte; §12 è la sezione aggiunta).

