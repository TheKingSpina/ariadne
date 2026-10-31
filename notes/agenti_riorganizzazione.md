# Ristrutturazione agentica — proposta del 31/10/2026

**Stato corrente: ATTIVO localmente — policy v1 del 31/10/2026.**
Predisposto da `openai/gpt-6.1-sol`. Nessun commit o push eseguito.
L'atto di attivazione sotto sostituisce lo stato BOZZA delle sezioni storiche.

## Atto di attivazione locale

Autorizzazione umana: «direi di attivarlo (ma fai un backup del sistema attuale,
non si sa mai)». Prima delle modifiche è stato creato e verificato il backup
`/Users/spina/Ariadne-agent-backups/pre-routing-20261031-195101`:
13 file, manifest SHA-256, snapshot del working tree e baseline Git `a1d88eb`.

- Orchestratore GLM → Router MiMo; rimossi i permessi di dispatch diretto
  producer/tester/reviewer dall'orchestratore.
- Rimossi i tre pin operativi: il Router passa un modello esplicito per
  ciascun mandato secondo le categorie e l'allowlist autorizzate.
- Policy `.opencode/routing.md` attiva come v1 locale. Registro per mandato
  in report distinti; nessun commit automatico.
- Per non inventare decisioni aperte: seriale per default, parallelismo solo
  con mandato esplicito che definisca isolamento e limite; nessun fallback
  automatico. Consenso prima di abbonamento o OpenRouter a pagamento.
- Span non identificato e selezione downstream di Free Models Router restano
  limitazioni circoscritte: non si usano senza chiarimenti, ma non bloccano
  gli altri modelli. Non imposta diversità di famiglia fra i valutatori;
  sessioni e responsabilità restano separate.
- Nessun task, test Rust o modello lanciato per questa attivazione.
  I controlli statici/caricamento si riportano separatamente; non equivalgono
  a un collaudo end-to-end della delega annidata.

## Storia della predisposizione (proposte non più normative)

Successiva autorizzazione umana: «sistemiamo comunque tutti gli agenti per
tenere tutto coerente». Le modifiche di coerenza sono applicate nel working
tree, mantenendo pin e percorso diretto durante la BOZZA. Le decisioni su
categorie, modelli e attivazione restano aperte: questa richiesta non le firma.

## Architettura proposta

### Decisioni umane successive sulle categorie (31/10/2026)

- `light`: modelli free in generale, previa verifica di disponibilità, gratuità
  e adeguatezza al task; nessuna lista di ID inventata.
- `medium`: `openai/gpt-6-luna` e `openai/gpt-6.1-terra`.
- `heavy`: `openai/gpt-6.1-sol`.
- Canali: free → abbonamento → OpenRouter a pagamento come ultima opzione.
  Notifica e discussione con l'umano prima del passaggio, non escalation muta.
- Nel percorso finale niente modelli fissi per ruolo: scelta esplicita del
  Router per mandato. I default legacy restano solo nella transizione BOZZA;
  la loro rimozione va completata con l'attivazione del Router.
- Router: non SOL; candidato «JEV 1.14» da identificare. Nessun riscontro
  nel catalogo consultato, nessun ID assegnato per supposizione. Rimosso il
  pin SOL provvisorio dal Router: senza assegnazione resta non operativo,
  l'ereditarietà del modello della sessione non autorizza a usarlo.

Queste decisioni aggiornano le proposte a/c/e/g sotto senza firmare gli altri
punti. Nessun commit, push o dispatch eseguito per applicare queste scelte.

### Allowlist OpenRouter e nuovo candidato Router

Successiva indicazione umana: autorizzati per quando sarà il momento GLM 5.3
Flash, GPT-6 Luna, Qwen3.8 Flash, Qwen3.7 Flash, DeepSeek V4.1 Flash 1017,
Mercury 3, Nemotron 3.5 Super VL (free), Nemotron 3 Ultra (free), Gemma 4
26B A4B (free), Gemma 4 31B (free), Free Models Router e Span-01 Lite.
La tabella `.opencode/routing.md` contiene gli undici ID verificati e la
riga Span-01 Lite ancora senza ID: nessuna chiamata eseguita, nessuna
autorizzazione implicita di spesa. Il consenso sui canali resta invariato.

Span-01 Lite sostituisce JEV come candidato modello del Router. La proposta
è registrata, ma il nome non ha corrispondenza verificata nel catalogo
OpenCode o nella ricerca sul catalogo pubblico OpenRouter; occorre link/ID.
La scelta delle categorie per i candidati OpenRouter a pagamento e delle
varianti resta aperta. Per Free Models Router occorre chiarire se sia ammessa
una selezione downstream fuori dall'elenco nominativo, e come attestare il
modello effettivo nei report: non è il Router agentico di Ariadne.

### Scelta finale del modello Router

L'umano ha escluso un modello decisionale dedicato e scelto **MiMo free**,
tenendo GLM per l'orchestratore. ID verificato nel catalogo:
`opencode/mimo-v2.6-flash-free`. Impostato nel frontmatter del Router;
sostituisce SOL provvisorio e i candidati JEV/Span, che restano qui solo
come storia della proposta. La voce Span nell'allowlist OpenRouter resta
da identificare e non è stata usata. Nessuna chiamata modello o spesa
eseguita per verificare MiMo; policy ancora BOZZA, nessun commit o push.

```text
umano ↔ orchestratore (governance e stato del ciclo)
                 ↓ mandato di fase
              Router (unico punto di dispatch dei ruoli)
                 ↓ classe → modello autorizzato
          producer → tester → reviewer → umano
```

Il Router sceglie come eseguire un mandato, non cosa debba fare il livello.
Non è un secondo giudice né un ulteriore gate. Il frazionamento non cambia
la legge e non consente di saltare l'ordine dei gate.
Più unità indipendenti possono essere proposte; l'integrazione è assegnata
a un producer nominato, non al Router. Gate e audit finali riguardano la
revisione integrata, non una somma di approvazioni locali.

I tre file predisposti sono una bozza comportamentale/configurativa: nessun
plugin, selettore automatico di prezzi o controllo runtime dei budget è stato
implementato. `.opencode/routing.md` non viene interpretato nativamente come
alias dei modelli. Il Router deve leggerlo e selezionare esplicitamente il
modello nello strumento di dispatch; gli ID restano da assegnare dall'umano.

## Permessi del nuovo Router e motivazione

- Edit negato salvo `reports/**`: permette tracce operative senza accesso a
  codice, legge o tabella vincolante. Non è concessa la scrittura opzionale
  su `.opencode/routing.md`: gli scostamenti vanno nei report, non nella policy.
- Shell negata salvo `git log*`, `git show*`, `git diff*`, come il reviewer:
  solo letture, comandi singoli; non è permesso aggirare una negazione tramite
  altri strumenti o opzioni. Le wildcard non sono una sandbox semantica di Git.
- Subagent negato per default e ammesso solo per producer/tester/reviewer:
  consente il ruolo richiesto senza deleghe ricorsive o ruoli generalisti.
- Nessuna nuova delega di governance Mnemosyne: il Router restituisce le prove
  all'orchestratore, che conserva la delega esistente. I permessi globali
  eventualmente ereditati non sono stati modificati da questa bozza.
- `mode: all`, pin SOL provvisorio e `steps: 25` come da mandato. OpenCode può
  scoprire il file prima della firma: il corpo impone un blocco esplicito dei
  dispatch finché la policy non è ATTIVA; non è una disabilitazione tecnica.

## Diff di progetto — riferimento alla prima proposta

I blocchi sotto conservano il disegno della prima proposta, non sono patch
da applicare automaticamente: l'implementazione di transizione è descritta
nella sezione successiva. In particolare la rimozione dei permessi di
dispatch diretto è rinviata: altrimenti la BOZZA bloccherebbe anche il ciclo
vigente. I pin `model:` esistenti restano tutti intatti. Le classi non possono
sostituire un ID nel frontmatter; i default tecnici e la policy di override
si approvano separatamente.

### 1. `.opencode/agents/orchestrator.md`

```diff
@@ permissions (dopo il deny globale subagent)
-  - action: subagent
-    resource: "producer"
-    effect: allow
-  - action: subagent
-    resource: "tester"
-    effect: allow
-  - action: subagent
-    resource: "reviewer"
+  - action: subagent
+    resource: "router"
     effect: allow
@@ Ciclo di lavoro
-2. Lancia il producer con il livello assegnato, il percorso SPEC e il requisito test-first (primo commit solo test rossi). Se il livello non è chiaro dalla SPEC o dal task, CHIEDI all'umano: non sceglierlo da solo.
+2. Affida al Router il mandato di fase producer, con livello, legge, revisione, scope e requisito test-first. Se il livello non è chiaro, chiedi all'umano. Non lanciare direttamente i ruoli.
-3. Al termine, lancia il tester: verifica indipendente contro la SPEC, con output in `reports/L<n>_test_report.md`.
+3. Al termine, richiedi al Router il tester sulla revisione integrata, con verifica indipendente e report in `reports/L<n>_test_report.md` (o percorso univoco indicato nel mandato).
@@ Dimensione del task e modelli
-- Usa il modello e la variante pinnati nel frontmatter. Nessun fallback o cambio provider è autorizzato: se un dispatch non parte per indisponibilità o quota, fermati e riferisci il blocco all'umano. Nessun override per tester o reviewer.
+- Il tuo modello resta il default del frontmatter. Per i figli il Router applica esclusivamente la policy ATTIVA in `.opencode/routing.md`; ogni scelta e subentrata deve essere autorizzata dalla policy e tracciata. Senza policy attiva non avviare il nuovo percorso di routing.
```

Al passo 4 del ciclo, sostituire «riporta gli errori al producer» con
«riaffida al Router un mandato producer con gli errori». Al passo 5,
sostituire «lancia il reviewer» con «richiedi al Router il reviewer»;
lasciare intatti tutti i vincoli su report verde, audit e accettazione umana.
Il Router non avvia autonomamente una fase successiva.

### 2. `.opencode/agents/producer.md`

```diff
@@ Provenienza della risposta
-- Usa il pin `openai/gpt-6-luna#medium` dal frontmatter; nessun override o cambio provider è autorizzato. Se il dispatch non parte per indisponibilità o quota, fermati e riferisci il blocco.
+- Il frontmatter è il default tecnico. Nel percorso Router attivo usa solo il modello selezionato esplicitamente dal dispatch secondo `.opencode/routing.md`, con classe e revisione della policy. Dichiara l'attestazione del modello e le eventuali differenze osservabili; in caso di conflitto fermati. Non scegli autonomamente modelli o subentrate.
@@ Divieti
-- Non lanciare tester né reviewer: i ruoli li gestisce l'orchestratore.
+- Non lanciare altri agenti: l'orchestratore governa le fasi, il Router esegue i dispatch. Se sei producer integratore, integra solo le unità e i file assegnati, poi richiedi il gate sulla revisione risultante.
```

Il riferimento a Luna nel corpo è già incoerente col frontmatter Terra:
il diff proposto lo risolve senza cambiare il pin tecnico in questa bozza.
Tutti i vincoli di test-first, codice e divieto di auto-approvazione restano.

### 3. `.opencode/agents/tester.md`

```diff
@@ Memoria / modello
-- Usa esclusivamente il modello pinnato qui; niente override o cambio provider. Se il dispatch è indisponibile, segnala il blocco senza tentare sostituzioni.
+- Il frontmatter è il default tecnico. Nel percorso Router attivo il modello è selezionato esplicitamente secondo `.opencode/routing.md`; dichiara classe, revisione della policy e modello attestato. Non scegli sostituzioni e non aggiri permessi negati.
@@ Ciclo di verifica (aggiunta)
+- Per un mandato suddiviso dichiara il tuo scope e la revisione verificata. Un esito parziale non chiude il gate globale: serve una verifica finale sulla revisione integrata, indipendente dal producer.
```

Restano invariati i permessi, l'esecuzione indipendente dei test, i report
verbatim e il divieto di correggere produzione/test. L'indipendenza richiesta
fra modelli in ruoli diversi è una decisione aperta, non un vincolo inventato.

### 4. `.opencode/agents/reviewer.md`

```diff
@@ Modello (ultime quattro righe)
-- Usa esclusivamente il modello pinnato in questo file. Se il dispatch non
-  parte con il pin (stallo, modello non raggiungibile), subentra la sola
-  `openai/gpt-6-luna#medium` e dichiaralo in testa al report; nessun altro
-  override o cambio provider.
+- Il frontmatter è il default tecnico. Nel percorso Router attivo il modello è selezionato esplicitamente secondo `.opencode/routing.md`; dichiara classe, revisione della policy e modello attestato. Non scegli sostituzioni: le gestisce il Router entro l'autorizzazione firmata.
@@ Divieti (chiarimento per coerenza col frontmatter)
-- Nessuna edit, nessun comando oltre alla lettura del repository via git.
+- Nessuna modifica salvo il report assegnato in `reports/`, già ammesso dai permessi. Leggi file e prove con gli strumenti consentiti; usa la shell solo per letture Git in allowlist, mai per eseguire test. Un permesso negato resta un limite metodologico esplicito.
```

Il chiarimento evita il conflitto già emerso fra permesso di scrivere report
e divieto testuale assoluto. Non allarga l'allowlist Git e non elimina il
gate tester verde come precondizione dell'audit finale.

### Riferimento comune alla subentrata (quattro agenti)

Proposta, da applicare solo dopo firma/attivazione: sostituire i commenti
frontmatter sulla subentrata con un riferimento alla versione firmata di
`.opencode/routing.md`, mantenendo la regola vigente finché la migrazione non
è completata. Non copiare due policy potenzialmente divergenti in ogni ruolo.
Il fallback dell'orchestratore/Router non può essere gestito da un agente
che non risponde: serve un supervisore umano o una funzione runtime distinta.

## Allineamento applicato nel working tree su richiesta umana

- **Orchestratore:** legge lo stato della policy; in BOZZA conserva il dispatch
  diretto, con policy ATTIVA usa esclusivamente il Router. Aggiunta l'allowlist
  `subagent: router`, senza rimuovere ancora i tre permessi diretti. Il ciclo
  conserva tutti i gate; la legge firmata prevale sulla semplice numerazione
  più recente delle SPEC. Chiarito che le letture Git consentite non sono
  vietate dalla frase precedente «nessuna shell».
- **Producer:** eliminato il riferimento testuale obsoleto al pin Luna
  (frontmatter Terra invariato), aggiunti mandato Router condizionale e
  regole di integrazione senza writer concorrenti sugli stessi file/indice.
- **Tester:** mandato e report per revisione/scope, nessuna auto-sostituzione,
  gate finale sulla revisione integrata. Rimozione dell'obbligo di pulizia
  worktree con `--force`: non scartare modifiche senza autorizzazione.
  Chiarita l'eccezione scratch già prevista, senza ampliare i permessi.
- **Reviewer:** eliminata la contraddizione fra report ammesso dai permessi
  e divieto assoluto di edit; confermato che non esegue test, che cita limiti
  metodologici e che un gate su altra revisione non si estende implicitamente.
- **Regola test-first comune ai ruoli:** solo test rossi a runtime per default;
  stub/export minimi sono eccezioni solo in presenza di esplicita autorizzazione
  legislativa o mandato umano firmato, citata insieme ai file nel commit.
  Nessuna nuova autorizzazione generale agli stub e nessuna riscrittura dei
  giudizi o dei commit storici L3.
- **Tutti i pin:** invariati; riferimento unico alla policy per la subentrata.
  Il Router resta bloccato in BOZZA. Nessun costo, classe/modello o nuovo
  fallback è stato scelto al posto dell'umano.

## Decisioni aperte per la firma dell'umano

Le caselle restano **non selezionate**: `[x]` attesterebbe una firma non data.
La firma deve indicare la scelta, non soltanto approvare genericamente il file.

1. **(a) Modelli nelle classi.** Opzioni: `light/medium/heavy`, categorie per
   capacità specifica, oppure entrambe. Raccomandazione: partire dalle tre
   classi e indicare per ogni modello i task ammessi; l'umano fornisce gli ID,
   da verificare prima dell'attivazione. Nessuna assegnazione proposta ora.
   - [ ] umano — ___; assegnazioni: ___
2. **(b) Subentrata.** Opzioni: confermare Luna, sostituirla, fallback per
   classe/ruolo, nessun fallback. Raccomandazione: conservare inizialmente
   `openai/gpt-6-luna#medium` e rendere ogni uso visibile; non usarlo per
   superare un test rosso o un audit negativo.
   - [ ] umano — ___; scelta: ___
3. **(c) Pin del Router.** Opzioni: SOL definitivo, altro modello deciso
   dall'umano, default senza pin. Raccomandazione: un pin esplicito per il
   Router, da scegliere su qualità/costo misurati; SOL resta provvisorio.
   - [ ] umano — ___; modello: ___
4. **(d) Parallelismo e stitching.** Opzioni: seriale, parallelo stesso ruolo,
   parallelo solo letture. Raccomandazione: consentire unità indipendenti con
   dipendenze esplicite, owner/file separati e un integratore producer; partire
   con un massimo di due figli concorrenti e serializzare i gate finali.
   Worktree separati richiedono predisposizione autorizzata, non nuove shell
   per il Router. La soglia due è una proposta, non un limite già in vigore.
   - [ ] umano — ___; concorrenza/isolamento/integrazione: ___
5. **(e) Autonomia di fallback.** Opzioni: firma per ogni uso, delega entro
   policy, fallback solo manuale. Raccomandazione: una sola sostituzione per
   mandato su indisponibilità documentata, dopo chiusura/annullamento del
   precedente incarico; nessun retry illimitato. Stabilire chi rileva lo
   stallo e quali prove lo dimostrano: il semplice background non lo prova.
   - [ ] umano — ___; trigger/autorità/limiti: ___
6. **(f) Registro.** Opzioni: report operativi, commit per ogni dispatch,
   Mnemosyne, combinazione. Raccomandazione: record separati in `reports/`
   per dispatch, commit firmati per cambi di policy, Mnemosyne per decisioni
   durevoli tramite orchestratore. Non committare necessariamente ogni
   chiamata e non duplicare integralmente i log tecnici nelle memorie.
   - [ ] umano — ___; destinazione e conservazione: ___
7. **(g) Costi.** Opzioni: nessun campo, fasce indicative, budget misurati.
   Raccomandazione: budget per mandato e concorrenza, prezzi solo con fonte
   e data. Se il consumo non è disponibile, dichiararlo; non promettere un
   tetto monetario tecnicamente non applicato. Limiti runtime richiedono un
   lavoro distinto (plugin/SDK o supervisione), non questo skeleton.
   - [ ] umano — ___; budget e fonti: ___

## Questioni parcheggiate — nessuna decisione implicita

- **Un solo agente principale o due?** Questa bozza conserva orchestratore
  come governance e aggiunge Router come dispatcher, come nel mandato. Non
  fonde i ruoli. Se l'obiettivo è un unico agente primario anche di governance,
  occorre una decisione esplicita prima della migrazione.
- **Indipendenza dei valutatori:** sessioni diverse sono necessarie per ruoli
  distinti, ma non garantiscono indipendenza dei modelli. Decidere se un
  fallback Luna può verificare lavoro prodotto da Luna o se serve un modello
  diverso; nessun task può auto-approvarsi cambiando solo etichetta di ruolo.
- **Permessi negati nella storia L3:** i pattern Git esistenti non garantiscono
  l'esecuzione di comandi concatenati. Non allargare indiscriminatamente la
  shell; provare comandi singoli in un futuro collaudo autorizzato del routing.
- **Test-first e stub:** i prompt ora distinguono regola ordinaria ed eccezione
  esplicitamente autorizzata. Se si desidera un'autorizzazione generale agli
  stub per ogni livello, serve ancora una decisione umana: non è stata concessa
  dalla ristrutturazione e non modifica retroattivamente la governance di L3.
- **Hot reload:** OpenCode V2 ricarica le definizioni salvate; non serve un
  riavvio per queste bozze. Prima dell'attivazione, il Router deve comunque
  fermarsi. L'imposizione tecnica di una sospensione totale è fuori scope.

## Verifica della predisposizione

Fonte tecnica letta: https://opencode.ai/v2/docs/agents (31/10/2026).
Campi V2 usati: `description`, `mode`, `model`, `steps`, `permissions`;
regole ordinate con deny generale prima delle eccezioni. I pin esistenti non
sono stati alterati; i prompt e il permesso di dispatch Router dell'orchestratore
sono allineati nel working tree. Nessun cargo, plugin, subagent, commit o push
eseguito per questo lavoro. Il collaudo operativo resta sospeso fino alla firma.
