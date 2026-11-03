# Fonti e governance

## Documenti di riferimento

| Livello | SPEC | Leggi e precisazioni |
| --- | --- | --- |
| L0 | `spec/L0_SPEC_v0.4.md` | `notes/L0_accettazione.md` |
| L1 | `spec/L1_SPEC_v0.3.md` | addendum F1 incluso nel percorso normativo L1 |
| L2 | `spec/L2_SPEC_v0.3.md` | `notes/L2_legge_selezione.md`, inclusa L2.27 |
| L3 | `spec/L3_SPEC_v0.1.md` | `notes/L3_legge_memoria.md`, `notes/L3_provenienza.md` |
| L4 | `spec/L4_SPEC_v0.1.md` | `notes/L4_legge_identita.md` |
| L5 | `spec/L5_SPEC_v0.1.md` | `notes/L5_legge_sociale.md` |
| L6 | `spec/L6_SPEC_v0.2.md` | `notes/L6_legge_valutazione.md` |

I documenti `notes/*_domande_*.md` sono triage e contesto delle scelte;
le versioni precedenti delle SPEC sono reperti storici. In caso di tensione
tra documenti, non risolverla silenziosamente: verificare la legge e gli
emendamenti applicabili e sottoporre la questione alla governance.

## Report e provenienza

- I report di implementazione, gate e audit sono in `reports/`. Riflettono
  l'esito e il perimetro dichiarati al momento del rispettivo ciclo; non
  sostituiscono una nuova verifica.
- Per L6, `reports/gate_l6_impl.md`, `reports/audit_l6_impl.md` e il commit
  di rettifica `705e02f` documentano la discrepanza fra modelli richiesti e
  modelli attestati. La sola attestazione di modello non dimostra da sé il
  costo effettivo o l'assenza di addebiti.
- La verifica della suite riportata in questa cartella è successiva ai
  report storici; non riscrive i loro risultati.

## Flusso di lavoro

La policy di routing è in `.opencode/routing.md`; i ruoli sono configurati
in `.opencode/agents/`. Per lavoro futuro applicare il mandato e le
autorizzazioni effettive, non dedurre un'autorizzazione permanente da una
scelta passata. Le decisioni normative e l'accettazione seguono il percorso
di governance del progetto.

Mnemosyne è un registro separato dal repository Git: proposte di memoria e
collegamenti possono richiedere approvazione e non diventano automaticamente
parte della documentazione versionata.
