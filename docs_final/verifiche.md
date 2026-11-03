# Verifiche, evidenze e limiti

## Suite corrente

Nell'ultima verifica svolta in questa sessione, sulla revisione `94e6eb6`:

| Comando | Esito |
| --- | --- |
| `cargo test` | 182 superati, 0 falliti |
| `cargo test --release` | 182 superati, 0 falliti |

È presente il warning non bloccante `unused_mut` in
`tests/l6_protocol.rs:119`. I due comandi eseguono la suite esistente, non
costituiscono un nuovo audit di conformità.

### Copertura di integrazione

`tests/l0_l1_l2_pipeline.rs` contiene una pipeline esplicita L0→L1→L2.
Altri test verificano singoli livelli e interazioni mirate, tra cui la
chiusura della finestra L2 da eventi L4 e L5. Non è stato individuato un
singolo test che attraversi tutti i livelli L0→L6 sullo stesso scenario.
La suite verde non va quindi descritta come prova di tale percorso completo.

## Esperimento L6 registrato

Il report storico `reports/l6_esperimento.md` registra 100 coppie sui seed
1000–1099, con 120 task ciascuna, 80 di apprendimento e 40 di test. Riporta
W=5050, p formattato a sei decimali come `0.000000`, Δ̄=0.6647 e nessuna
violazione registrata. Il valore stampato di p **non significa p=0**.
L'esecuzione completa non è stata ripetuta nell'ultima verifica della suite.

### Interpretazione corretta

- Il confronto implementato è tra una policy che conta gli esiti precedenti
  nel **proprio log** e un controllo che sceglie sempre 0. La regola e i
  task sono sintetici. Il risultato riguarda tale disegno e non dimostra
  che le viste L1–L5 siano state utilizzate o abbiano causato il vantaggio.
- La SPEC L6 riporta un benchmark casuale di 0,50; la legge dell'esperimento
  e il report adottano tre classi, per le quali il benchmark teorico
  uniforme è circa 1/3. È una differenza documentata, non un'identità tra
  le due formulazioni.
- Il report dice «200/200» per i log delle 100 coppie. In `run_pair`,
  `src/valuation.rs`, il controllo ricostruisce il JSONL, verifica la
  catena, confronta la riserializzazione e ricalcola i reward con
  `fold_run`. Questo **non dimostra indipendentemente il ricalcolo di ogni
  decisione** del braccio come richiesto dalla formulazione di L6.9.
- Il timeout è misurato nell'harness dopo il ritorno della coppia, non è un
  meccanismo che interrompe forzatamente un'esecuzione bloccata.
- Il Wilcoxon usa la formula specificata dalla legge L6, inclusi i ranghi
  medi per i pareggi; l'interpretazione statistica e l'adeguatezza della
  varianza in presenza di pareggi meritano una verifica indipendente prima
  di usare il risultato come evidenza generale.

## Ripetibilità

L'esperimento completo può riscrivere report e log esistenti; vedere
[uso.md](uso.md). I risultati registrati, la suite unitaria e un eventuale
nuovo test integrato sono evidenze distinte e non vanno confuse.
