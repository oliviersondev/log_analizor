# log_analizor

Petit outil Rust pour analyser des logs applicatifs avec un agent Strands en local via Ollama.

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
CONTEXT7_API_KEY=<token_context7>
```

`CONTEXT7_API_KEY` est optionnelle. Sans cette cle, `suggest_fix` reste disponible mais indique explicitement que Context7 n'a pas ete appele.

## Lancement rapide

```bash
ollama serve
make run
```

Ou en direct:

```bash
cargo run
```

## Commandes utiles

- `make check` : verification rapide compilation
- `make fmt` : formatage Rust
- `make clippy` : lint
- `make test` : tests
- `make test-one TEST=nom_test` : test unique exact

## Comportement Context7

- `suggest_fix` est force au moins une fois dans le flux principal (`main`) via un appel direct de tool.
- La sortie de `suggest_fix` inclut toujours un bloc `Context7` explicite:
  - `called: yes` quand l'appel API a ete tente
  - `called: no` quand l'appel n'est pas tente (ex: `CONTEXT7_API_KEY` absente ou log non mappe)
- En cas d'appel, le bloc contient soit des `snippets`, soit un `error`.

Exemple de bloc:

```text
Context7:
- called: no
- reason: missing CONTEXT7_API_KEY
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
- `src/domain.rs`: logique metier de parsing/classification/suggestion
- `src/tools.rs`: wrappers `AgentTool` qui deleguent a `domain`
- Outils agent implementes manuellement via `AgentTool`:
  - `ParseLogTool`
  - `ClassifyIncidentTool`
  - `SuggestFixTool`
- Helpers metier (dans `domain`):
  - `parse_log`
  - `classify_incident`
  - `suggest_fix`
  - `infer_cause`

## Roadmap

- [ ] Ajouter une documentation d'utilisation avec Context7 (installation, configuration, exemples de requetes).
- [ ] Integrer une interface front avec Dioxus pour l'analyse des logs.
  - [ ] Permettre le glisser-deposer d'un fichier de log.
  - [ ] Permettre le copier-coller de texte de log brut.
