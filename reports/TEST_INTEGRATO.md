# Test integrato — 5 novembre 2026

**Commit testato:** `b50a82e618625d2dc5a1e0b4dde8b399634abde1`
**Esecuzione:** 2026-11-05T15:48Z
**Esito complessivo:** **SUPERATO** per le prove eseguite, con i limiti
dichiarati in fondo. Non è una certificazione di produzione né una prova di
verità o di efficacia causale.

## Ambiente

| Voce | Valore |
| --- | --- |
| Host | macOS 26.6.2, arm64 |
| Rust | 1.99.1 |
| Python host | 3.9.6 |
| Python container | 3.11.2 |
| Docker | 29.6.1 |
| Docker Compose | 5.5.0 |
| Container | `ariadne-dashboard`, healthy |
| Dashboard | `http://127.0.0.1:8090` |
| MCP | `http://127.0.0.1:8080/mcp` |

## Esiti

| Prova | Comando | Esito |
| --- | --- | --- |
| Test Rust | `cargo test --locked` | **204 passati, 0 falliti** |
| Test Python | `python3 -m unittest dashboard.test_backup dashboard.test_dashboard mcp.test_ariadne_server` | **53 passati, 0 falliti** |
| Oracolo indipendente | `python3 preparazione_stress/aux_v2_oracle.py verify` | **ok**, 121 eventi, SHA-256 `690831b3…` |
| Stress L0–L6 | `python3 preparazione_stress/esegui_stress_l0_l6.py` | **ok**, 847 eventi, SHA-256 `43accb3a…` |
| Handshake MCP | initialize + tools/list + list_logs | **ok**, 14 tool, protocollo `2025-03-26` |
| Dashboard | `/api/diagnostics` | archivio e MCP raggiungibili; conformità `not_verified` |
| Backup | export + verify nel container | **ok**, 2 registri, 54 eventi, digest `e6cdc121…` |
| Container | `docker inspect` | **healthy** |

## Stress test L0–L6

**Manifest:** `preparazione_stress/MANIFEST_STRESS_L0_L6_V1.md`
**Report:** `reports/stress_L0_L6_v1.md`

- Eventi L0–L5: **847**
- Hash di testa:
  `8d114b99ddacea0eb6960e27a7824c1269d4848aac9e8d83699d82084f5ea8c4`
- SHA-256 del JSONL:
  `43accb3a73fc7d11e22e66db895785e2af2a7c61556f93c7174aac6e39f60168`
- Fixture L5: 4
- L2 summary: 5; espulsione di `gamma` al tick 2
- L3: promote alpha/beta, archive alpha/gamma/delta, recall alpha
- Stato finale: alpha `circuito`, beta `memoria_lunga`; 4 nodi, 1 arco
- L6: 100 coppie, 0 violazioni, 200/200 replay, `W=5050.0`, `n=100`,
  `p=0.000000`, `Δ̄=0.6647` → verdetto previsto **SUCCESSO**

Verifiche indipendenti: catena hash e canonicità con implementazione Python
separata; digest e associazione delle fixture; posizioni L5, predict e
`model_update`; ordine `archive` prima di `promote`; proiezioni finali
ricostruite dal backend.

## Servizio live

- `initialize` ha restituito `serverInfo` `ariadne 0.5.0` e una sessione
  `Mcp-Session-Id`.
- `tools/list` espone **14** tool.
- `ariadne_list_logs` legge i registri `kurisu-test-10-nodi` (37 eventi) e
  `verifica-wp1-20261105` (17 eventi).
- `/healthz` risponde `ok` con le versioni `2024-11-05` e `2025-03-26`.
- La dashboard vede 2 registri e 54 eventi.
- `conformity.status` resta `not_verified`: la raggiungibilità non è una
  dichiarazione di conformità.

## Limiti

- Lo stress test L0–L5 è lo scenario sintetico del **manifest v1**, non il
  vecchio `manifest_stress_L0_L6_bozza.md`, che resta invariato e non
  eseguito.
- La semantica L1–L4 dell'intero log non è rigiocata da una seconda
  implementazione completa: l'indipendenza copre catena e fixture; la
  semantica è coperta dall'oracolo `aux_v2_small` su un altro scenario.
- L6 è un esperimento sintetico separato: non dimostra l'efficacia causale
  di L1–L5 e non autentica fatti o persone.
- La suite Python è stata eseguita sull'host con Python 3.9.6; il container
  usa Python 3.11.2. La suite è verde in entrambi gli ambienti usati, ma non
  è una matrice di versioni.
- Dashboard e MCP sono esposti solo su loopback, senza autenticazione client
  e con CORS `*`: ambiente di test locale, non produzione.
- I file in `runs/` sono evidenze locali non committate; i report sì.

## Riproduzione

```sh
cargo test --locked
python3 -m unittest dashboard.test_backup dashboard.test_dashboard mcp.test_ariadne_server
python3 preparazione_stress/aux_v2_oracle.py verify
python3 preparazione_stress/esegui_stress_l0_l6.py
```
