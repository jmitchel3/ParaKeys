# ParaKeys

> **Like Apple Passwords, but for dotenv.**

Encrypted project environments, multi-device sync without a required backend, and agent grants so reading `.env` does not vacuum secret values into chat.

## Status

MVP in progress. CLI skeleton builds; vault/import/run land on subsequent cards.

## CLI (scaffold)

```sh
cargo build
cargo run -- --help
```

Planned commands: `init`, `import`, `set`, `unset`, `list`, `run`, `doctor`.

## Documentation

Design vault (Obsidian-friendly wiki): **[docs/Home.md](docs/Home.md)**

Open the `docs/` folder as an Obsidian vault. Notes use `[[wikilinks]]`.

## Coordination

[Kanbanlan](docs/workflow/kanbanlan.md) + [GitHub Project 12](https://github.com/users/jmitchel3/projects/12)

## License

TBD.
