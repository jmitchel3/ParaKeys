# ParaKeys

> **Like Apple Passwords, but for dotenv.**

Encrypted project environments, multi-device sync without a required backend, and agent grants so reading `.env` does not vacuum secret values into chat.

## Status

MVP vertical slice works: `init` → `import` → `run`. Remaining: set/unset/list/doctor polish.

## Quick start

```sh
cargo build --release
./target/release/parakeys init
# save the recovery code offline

# put real secrets in .env once, then:
./target/release/parakeys import .env
# .env now has <set in parakeys> placeholders

./target/release/parakeys run -- your-command
```

Second machine: pull repo, then `parakeys init --recover '<code>'`.

## Documentation

Design vault (Obsidian-friendly wiki): **[docs/Home.md](docs/Home.md)**

Open the `docs/` folder as an Obsidian vault. Notes use `[[wikilinks]]`.

## Coordination

[Kanbanlan](docs/workflow/kanbanlan.md) + [GitHub Project 12](https://github.com/users/jmitchel3/projects/12)

## License

TBD.
