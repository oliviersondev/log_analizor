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
```

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

## Troubleshooting

- Erreur `model '...' not found`:
  - verifier le nom du modele dans `.env`
  - lister les modeles installes avec `ollama list`
  - telecharger un modele manquant avec `ollama pull <nom_modele>`
- Erreur `Missing OLLAMA_MODEL` ou `Missing OLLAMA_HOST`:
  - verifier que `.env` existe et contient les variables

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
