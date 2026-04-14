# log_analizor

Petit outil Rust pour analyser des logs applicatifs avec un agent Rig en local via Ollama.

Formats pris en charge par les outils metier:
- JSON applicatif (structure `level/service/message/timestamp/...`)
- Ligne CloudFront (format access log)
- Syslog classique
- Texte libre (fallback heuristique)

## Prerequis

- Rust (toolchain stable)
- Ollama installe et lance localement
- Un modele Ollama present (ex: `llama3.2:latest`)

## Configuration

1. Copier le fichier d'exemple:

```bash
cp .env.example .env
```

2. Verifier les variables obligatoires dans `.env`:

```env
OLLAMA_MODEL=llama3.2:latest
OLLAMA_HOST=http://localhost:11434
CONTEXT7_ENABLED=true
CONTEXT7_API_KEY=<token_context7>
CONTEXT7_DEBUG=false
```

`CONTEXT7_ENABLED=true` active les appels Context7. Si false, aucune donnee n'est envoyee a Context7.
`CONTEXT7_API_KEY` est optionnelle. Sans cette cle, `suggest_fix` reste disponible mais indique explicitement que Context7 n'a pas ete appele.
`CONTEXT7_DEBUG=true` active l'affichage des candidats Context7 testes (`candidates_tested`).

## Lancement rapide

```bash
ollama serve
make run
```

Ou en direct:

```bash
cargo run
```

Par defaut, `main` choisit aleatoirement un scenario de log dans `src/sample_logs.rs` a chaque execution pour tester des cas differents.

## Commandes utiles

- `make check` : verification rapide compilation
- `make fmt` : formatage Rust
- `make clippy` : lint
- `make test` : tests
- `make test-one TEST=nom_test` : test unique exact

## Comportement Context7

- `suggest_fix` est disponible comme outil de l'agent. Il est appele seulement si le modele le juge necessaire pendant le flux multi-turn.
- Le choix de la librairie Context7 est dynamique: recherche `/api/v2/libs/search`, scoring local des candidats, puis fallback sur les 3 meilleurs IDs.
- La sortie de `suggest_fix` inclut toujours un bloc `Context7` explicite:
  - `called: yes` quand l'appel API a ete tente
  - `called: no` quand l'appel n'est pas tente (ex: `CONTEXT7_API_KEY` absente ou log non mappe)
- En cas d'appel, le bloc contient soit des `snippets`, soit un `error`.

Le binaire principal (`main`) affiche la reponse en mode streaming avec des evenements (`thinking`, `tool-call`, `tool-result`) puis un bloc final avec l'usage tokens.

Le prompt principal est adapte automatiquement au format detecte (JSON, CloudFront, syslog, texte libre) au lieu d'envoyer une consigne generique.

Exemple de bloc:

```text
Context7:
- called: no
- reason: missing CONTEXT7_API_KEY
```

## Exemples de logs

JSON:

```text
{"level":"ERROR","service":"billing","message":"Database timeout","timestamp":"2026-04-08T10:00:00Z","error_code":"DB_TIMEOUT","response_time_ms":3200}
```

CloudFront:

```text
2026-04-08 09:10:11 CDG3 123 1.2.3.4 GET d111111abcdef8.cloudfront.net /api 502 - Mozilla/5.0 - - Error abc 0.123
```

Syslog:

```text
Apr 08 12:34:56 prod-host sshd[1234]: Failed password for invalid user admin from 10.0.0.1
```

## Troubleshooting

- Erreur `model '...' not found`:
  - verifier le nom du modele dans `.env`
  - lister les modeles installes avec `ollama list`
  - telecharger un modele manquant avec `ollama pull <nom_modele>`
- Erreur `Missing OLLAMA_MODEL` ou `Missing OLLAMA_HOST`:
  - verifier que `.env` existe et contient les variables
- `Context7:` avec `called: no`:
  - verifier `CONTEXT7_API_KEY` dans `.env`
  - verifier que l'API key est valide
  - verifier que le log matche un `error_code` mappe (`DB_TIMEOUT`, `AUTH_INVALID_TOKEN`, `UPSTREAM_502`)

## Architecture (actuelle)

- `src/main.rs`: bootstrap runtime (chargement config, creation agent, invocation)
- `src/lib.rs`: point d'entree des modules internes
- `src/config.rs`: chargement `.env` + validation des variables requises
- `src/domain/mod.rs`: logique metier (parse/classify/suggest + mapping Context7)
- `src/domain/normalize.rs`: normalisation multi-format (JSON/CloudFront/syslog/texte)
- `src/sample_logs.rs`: jeu de logs de test multi-format choisi aleatoirement par `main`
- `src/tools/mod.rs`: point d'entree des outils + re-exports
- `src/tools/parse_log.rs`: outil `ParseLogTool` via `#[rig::tool_macro]`
- `src/tools/classify_incident.rs`: outil `ClassifyIncidentTool` via `#[rig::tool_macro]`
- `src/tools/suggest_fix.rs`: outil `SuggestFixTool` via implementation `rig::tool::Tool`
- `src/tools/context7_enrichment.rs`: resolution/scoring/fallback Context7
- `src/tools/args.rs`: args d'outils partages (`raw_log`)
- Helpers metier (dans `domain`):
  - `parse_log`
  - `classify_incident`
  - `suggest_fix`
  - `infer_cause`

## Roadmap

- [ ] Ajouter une documentation d'utilisation avec Context7 (installation, configuration, exemples de requetes).
  - [ ] Recuperer les Lybrary dispo sur Context7
  - [ ] Analizer type de library la plus adaper et requeter Context7 en fonction
- [ ] Integrer une interface front avec Dioxus pour l'analyse des logs.
  - [ ] Permettre le glisser-deposer d'un fichier de log.
  - [ ] Permettre le copier-coller de texte de log brut.
