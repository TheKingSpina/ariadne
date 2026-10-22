- **Evento**: { seq: uint64 monotono, tick: logico (mai wall-clock!), type: SENSE | ACTION | STATE, payload: JSON canonico — chiavi ordinate, UTF-8, numeri a formato fisso —, prev_hash, hash = SHA-256(seq ‖ prev_hash ‖ payload) }
- **Catena**: ogni evento punta all'hash del precedente → append-only totale, niente update né delete; gli errori si correggono con eventi compensativi, mai riscrivendo.
- **Determinismo**: PRNG seedato una sola volta all'evento genesi, stato del PRNG incluso nel replay; zero timestamp reali, zero I/O esterna, zero float non normalizzati.
- **Replay**: dal genesi, applichi in ordine di seq, lo stato è la fold del log — se l'output non è bit-perfetto, il ciclo L0 è rosso.

