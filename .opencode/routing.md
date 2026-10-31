# Routing modelli Ariadne — policy v1

**Stato: ATTIVO. Versione: v1, 31 ottobre 2026 (working tree, non committata).**
Autorizzazione umana: «direi di attivarlo (ma fai un backup del sistema attuale,
non si sa mai)». Backup verificato prima delle modifiche:
`/Users/spina/Ariadne-agent-backups/pre-routing-20261031-195101`.
Contiene snapshot locale e baseline committata (`a1d88eb`), con manifest SHA-256.
Il Router non può modificare questa policy. Le funzioni non ancora definite
restano bloccate per singolo mandato, senza bloccare il percorso seriale free.

## Classi assegnate dall'umano (31/10/2026)

| Classe | Modelli ammessi (provider/modello#variante) | Profilo indicativo, non assegnazione automatica | Limiti di costo/concorrenza |
| --- | --- | --- | --- |
| `light` | Modelli free disponibili, con gratuità e ID verificati prima dell'uso | Task per cui il modello gratuito scelto è adeguato; nessun legame fisso col ruolo | Seriale per default; nessuna spesa autorizzata |
| `medium` | `openai/gpt-6-luna`, `openai/gpt-6.1-terra` | Lavori intermedi secondo complessità e rischio; variante da dichiarare | Abbonamento solo entro consenso del mandato |
| `heavy` | `openai/gpt-6.1-sol` | Lavori complessi secondo complessità e rischio; variante da dichiarare | Abbonamento solo entro consenso del mandato |

Una classe misura l'adeguatezza al task, non attribuisce autorità. Un reviewer
non è automaticamente `light` perché legge: giudizio e rischio possono
richiedere capacità maggiori. I ruoli conservano tutti i loro divieti.
Non sono fissati prezzi, ordinamenti economici dei modelli o soglie inventate.
Gli ID OpenAI sopra sono presenti nel catalogo consultato il 31/10/2026;
le varianti non sono state scelte dall'umano in questa assegnazione. Il nome
`free` o un listino vuoto non bastano a provare gratuità, quota residua o
accesso in abbonamento. Non sono incluse automaticamente varianti `-fast`,
provider alternativi o omonimi OpenRouter a pagamento.

## Ordine dei canali di costo e consenso

Decisione umana: preferire i free; se esauriti, passare ai modelli coperti
da abbonamento e, solo come ultima opzione, a OpenRouter a pagamento.
**Prima del passaggio notificare l'umano e discuterne: nessuna escalation
automatica di canale di costo.** In attesa della risposta il mandato resta
bloccato, non si consuma il nuovo canale per verificarne la disponibilità.

- Il Router sceglie il modello adatto al task entro le categorie autorizzate;
  la disponibilità gratuita non è una garanzia di capacità né un gate ridotto.
- Per un incarico che richiede direttamente medium/heavy, il mandato deve
  chiarire l'autorizzazione a usare l'abbonamento; la classificazione non è
  una delega illimitata a consumarne la quota. Se manca, chiedi conferma.
- Per OpenRouter a pagamento richiedi modello/provider esatti e consenso
  specifico sui costi/budget. Non usare un modello a pagamento come fallback
  silenzioso. Anche i limiti/condizioni dei modelli OpenRouter free si verificano.
- Un'indisponibilità documentata non significa che tutti i free siano esauriti;
  non dichiararlo senza prove e non lanciare una serie illimitata di tentativi.

## Niente pin per ruolo nella configurazione finale

Decisione umana: producer/tester/reviewer (e altri incarichi delegati) non
avranno un modello fisso. Il Router selezionerà e passerà un ID esplicito per
ogni dispatch; i ruoli manterranno permessi e responsabilità, non pin.
I pin legacy sono rimossi da producer/tester/reviewer. Senza un `model:` OpenCode
eredita il modello del parent: questa ereditarietà NON sostituisce la scelta
esplicita del Router e non autorizza dispatch fuori policy.

Il modello di avvio del Router è **MiMo-V2.6-Flash Free**:
`opencode/mimo-v2.6-flash-free`, scelto dall'umano al posto dei candidati
precedenti. ID presente nel catalogo OpenCode con prezzi input/output zero;
disponibilità e quota non ancora provate con una chiamata. È un normale
modello chat con strumenti, non richiede una categoria di modelli decisionali.
GLM resta distinto come modello di avvio dell'orchestratore. Le versioni MiMo
OpenRouter a pagamento non sono autorizzate da questa scelta.

## Allowlist OpenRouter autorizzata per uso futuro

Elenco indicato dall'umano il 31/10/2026. L'inclusione autorizza il candidato,
**non attiva il canale né autorizza spesa immediata**: rimangono notifica e
discussione prima del passaggio concordato. Nessuna estensione automatica
ad altri modelli, alias latest, varianti Pro o versioni non-free degli omonimi.
Le categorie dei candidati a pagamento restano da definire: questa lista
non li trasforma automaticamente in `light`.

| Nome autorizzato | ID OpenCode verificato | Canale nel catalogo |
| --- | --- | --- |
| GLM 5.3 Flash | `openrouter/z-ai/glm-5.3-flash` | A pagamento |
| GPT-6 Luna | `openrouter/openai/gpt-6-luna` | A pagamento; distinto dall'accesso OpenAI in abbonamento |
| Qwen3.8 Flash | `openrouter/qwen/qwen3.8-flash` | A pagamento |
| Qwen3.7 Flash | `openrouter/qwen/qwen3.7-flash` | A pagamento |
| DeepSeek V4.1 Flash 1017 | `openrouter/deepseek/deepseek-v4.1-flash-1017` | A pagamento |
| Mercury 3 | `openrouter/inception/mercury-3` | A pagamento |
| Nemotron 3.5 Super VL (free) | `openrouter/nvidia/nemotron-3.5-super-vl:free` | Free, quote da verificare |
| Nemotron 3 Ultra (free) | `openrouter/nvidia/nemotron-3-ultra-550b-a55b:free` | Free, quote da verificare |
| Gemma 4 26B A4B (free) | `openrouter/google/gemma-4-26b-a4b-it:free` | Free, quote da verificare |
| Gemma 4 31B (free) | `openrouter/google/gemma-4-31b-it:free` | Free, quote da verificare |
| Free Models Router | `openrouter/openrouter/free` | Free; selezione downstream non fissata |
| Span-01 Lite | DA IDENTIFICARE — nessuna corrispondenza verificata | Costo e disponibilità da verificare |

Verifica degli ID/canali: catalogo OpenCode, 31/10/2026; non è una prova di
quota residua, accesso all'account, capacità di tool calling o qualità per
il task. Nessun modello della lista è stato eseguito per questa verifica.

**Free Models Router non è l'agente Router di Ariadne:** è un endpoint che
sceglie a sua volta il modello effettivo. L'autorizzazione dell'endpoint è
registrata; il suo uso resta subordinato al chiarimento sulla selezione
downstream: potrebbe scegliere un modello fuori dall'allowlist nominativa.
Non dichiarare noto il modello effettivo se non viene restituito; chiarire
con l'umano se è ammessa questa delega prima di usarlo per un gate o audit.

## Limiti operativi della v1

- **Concorrenza:** un figlio alla volta per default. Divide-et-impera seriale
  ammesso su scope separati; per esecuzione concorrente serve un mandato con
  numero massimo di figli, isolamento delle scritture e integratore nominato.
  Senza tali dati non avviare parallelismo e non creare worktree dal Router.
- **Fallback:** nessun retry o cambio modello automatico in caso di problema.
  Riporta l'errore all'orchestratore e richiedi conferma. La vecchia regola
  di subentrata Luna (`0defe20`) è sostituita per il percorso attivo da questa
  selezione per mandato e dai vincoli di consenso. Luna è un candidato medium.
  Il Router non rispondente richiede intervento dell'umano/supervisore.
- **Modello/variante:** ID esplicito in ogni chiamata subagent; se non è
  prescritta una variante, usa il default del catalogo senza inventarne una
  e dichiaralo nel report. Vietata l'ereditarietà implicita come routing.
- **Budget:** rispettare gli step dei ruoli, nessun rilancio identico illimitato.
  Se serve un budget monetario non ancora concordato, chiedilo; un listino
  assente non autorizza a considerare gratuito il modello. Per abbonamento
  e OpenRouter a pagamento attendere il consenso previsto sopra.
- **Registro:** un report separato per mandato in `reports/`, con versione v1,
  hash SHA-256 del file policy se disponibile, revisione Git, modello/classe,
  motivazione, figli e session ID. Non sovrascrivere record precedenti.
  Governance durevole a Mnemosyne a cura dell'orchestratore, non del Router.
  Nessun commit/push automatico dei log o della configurazione.
- **Indipendenza:** sessioni separate per producer, tester e reviewer; nessun
  agente può auto-approvarsi. Non è imposta diversità di famiglia del modello;
  se un mandato la richiede rispettarla, altrimenti dichiarare il modello
  realmente usato da ogni valutatore, senza promettere indipendenza assoluta.
- **Endpoint opachi:** Free Models Router e Span-01 Lite rimangono candidati
  bloccati dalle verifiche mancanti descritte sopra; non impediscono l'uso
  degli altri candidati identificati e autorizzati.

## Regole d'uso e modifica

1. Il Router legge questa tabella prima di ogni mandato; l'orchestratore
   verifica precondizioni e stato del ciclo, i figli ricevono la scelta nel
   mandato e la confrontano con le istruzioni del proprio ruolo.
2. L'orchestratore può lanciare solo il Router; il Router può lanciare solo
   producer/tester/reviewer. I figli non possono delegare. Non avviare la
   fase successiva del ciclo senza nuovo mandato dell'orchestratore.
3. Attivazione locale autorizzata il 31/10/2026, senza commit o push in questo
   passaggio. Per revisioni future conservare evidenza della firma e versione;
   se sospesa la policy, fermare i nuovi dispatch, non ripristinare implicitamente
   pin o permessi legacy. Il ripristino richiede una richiesta umana.
4. Le revisioni della policy entrano con nuovi commit: mai amend/rebase della
   storia per simulare una decisione precedente. I dispatch citano la revisione
   della policy realmente utilizzata. Git versiona il file; non lo rende
   tecnicamente immutabile in assenza di controlli aggiuntivi.
5. Non inserire nomi di classe nel campo `model:`: OpenCode richiede un ID di
   modello. Le classi sono policy del progetto e il Router deve risolverle
   esplicitamente nel dispatch, non una funzione nativa attivata da questo file.
6. I figli non hanno pin di ruolo. Il modello effettivo non cambia perché un
   prompt lo dichiara: serve il parametro `model` dello strumento subagent.
   Se il runtime non offre il parametro o applica un modello diverso, fermati
   e segnala il limite: non dichiarare riuscito il routing.
7. Costi/quote non disponibili si dichiarano sconosciuti. Budget sconosciuto,
   modello non assegnato o conflitto di policy → richiesta di decisione,
   non scelta autonoma fuori tabella.

## Verifica e ripristino

Attivazione dei file e controlli statici non sono un collaudo end-to-end dei
modelli o della delega annidata. Nessuna chiamata ai modelli è necessaria per
salvare la policy; un eventuale collaudo verrà dichiarato separatamente.
Per ripristinare, usare il backup indicato nell'intestazione dopo richiesta
umana: `working-tree/` conserva lo stato immediatamente precedente,
`committed-baseline/` i vecchi quattro agenti e `opencode.jsonc` a `a1d88eb`.
Verificare prima `manifest.json`, salvare i cambi successivi e non ripristinare
con reset/checkout distruttivi. Non copiare il backup dentro `.opencode/`,
altrimenti OpenCode potrebbe scoprirne le vecchie definizioni.
