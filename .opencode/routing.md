# Routing modelli Ariadne — policy v2

**Stato: ATTIVO dal 5 novembre 2026 (working tree, non committata).**
Sostituisce la policy v1 con Router. Il testo integrale della v1 è conservato
in `.opencode/routing-v1-archivio.md` e nella storia Git.

La v1 era stata autorizzata il 31 ottobre 2026 con backup in
`/Users/spina/Ariadne-agent-backups/pre-routing-20261031-195101`. La v2 non
ripristina il Router: i modelli sono dichiarati esplicitamente nei file degli
agenti e nessun agente li seleziona a runtime.

## Agenti e modelli

| Agente | Ruolo | Modello | Costo dichiarato dal catalogo |
| --- | --- | --- | --- |
| `orchestrator` | primary: coordina, non modifica file | `openrouter/deepseek/deepseek-v4.1-flash-1017` | $0,15 / $0,60 per milione (input/output) |
| `producer` | subagent: implementa e testa | `openrouter/z-ai/glm-5.3-flash` | $0,15 / $0,50 per milione |
| `producer-fallback` | subagent: solo se il producer OpenRouter non è disponibile | `opencode/glm-5.3-flash-free` | gratuito nel catalogo OpenCode |
| `tester` | subagent: verifica indipendente | `openai/gpt-6-luna` | non indicato dal catalogo |
| `tester-fallback` | subagent: solo se la quota Luna è esaurita | `openrouter/nvidia/nemotron-3.5-super-vl:free` | gratuito nel catalogo OpenRouter |

I prezzi sono quelli osservati nel catalogo il 5 novembre 2026 e non sono una
garanzia di quota, disponibilità o costo effettivo. Il nome `free` non prova
che la quota sia residua.

## Regole

1. Nessun Router e nessuna eredità implicita: il modello usato è quello
   scritto nel file dell'agente.
2. `orchestrator` non scrive codice di produzione, non modifica leggi o SPEC e
   non firma l'accettazione umana.
3. `producer` e `tester` lavorano in sessioni distinte. Il tester non modifica
   il codice e non dichiara indipendente un oracolo ricavato dalla stessa
   implementazione.
4. Il passaggio a `producer-fallback` o `tester-fallback` avviene solo dopo un
   errore del provider che confermi l'indisponibilità o l'esaurimento della
   quota. Va comunicato all'utente; i risultati parziali vanno distinti.
5. Se anche il fallback non è disponibile, l'orchestratore si ferma e chiede.
   Non sceglie altri modelli, provider o varianti a pagamento.
6. Le classi `light`/`medium`/`heavy` della v1 non sono più usate per
   assegnare modelli. Restano come riferimento storico nel documento archiviato.
7. Nessuna spesa oltre i modelli dichiarati è autorizzata da questa policy.
8. L'eventuale cambio di modello richiede una modifica esplicita del file
   agente e una nuova decisione umana.

## Limiti

Questa policy descrive la configurazione di OpenCode, non il comportamento del
runtime: il modello della sessione corrente non cambia retroattivamente. Le
modifiche sono locali finché non vengono committate. L'accettazione delle
verifiche resta umana e distinta dal giudizio del tester.
