# ParaKeys

> **Like Apple Passwords, but for dotenv.**

Encrypted project environments, multi-device sync without a required backend, and agent grants so reading `.env` does not vacuum secret values into chat.

## Status

Human MVP + Mac polish: CLI, **macOS Keychain / Touch ID unlock** (with file fallback), and a **desktop GUI**.

## CLI quick start

```sh
cargo build --release
./target/release/parakeys init
# save the recovery code offline

# put real secrets in .env once, then:
./target/release/parakeys import .env
# .env now has <set in parakeys> placeholders

./target/release/parakeys run -- your-command
./target/release/parakeys list
./target/release/parakeys doctor
```

Second machine: pull repo, then `parakeys init --recover '<code>'`.

### Unlock wallet (macOS)

| Backend | When |
|---------|------|
| **Keychain + user presence** | Preferred when the binary has Keychain entitlements (Touch ID / passcode). |
| **Keychain** (plain) | Default for typical unsigned CLI builds after presence ACL fails (`errSecMissingEntitlement`). Still Keychain, not a file. |
| **File** (fallback) | If Keychain store fails entirely, or `PARAKEYS_FORCE_FILE_WALLET=1`. Path: `.parakeys/local.key` (never commit). |

`init` prints notes when it degrades from presence → plain Keychain → file. Set `PARAKEYS_KEYCHAIN_NO_PRESENCE=1` only for non-interactive tests that skip the presence attempt.

## GUI (macOS)

```sh
cargo build --release --features gui
./target/release/parakeys-gui
```

The GUI is a Passwords-like shell over the same vault/wallet code as the CLI:

- Choose a project folder
- List keys by **status** (secrets hidden unless Reveal)
- **Init vault**, **Import .env**, **parakeys run** (via sibling CLI)

Requires the `gui` feature (default). Builds `parakeys-gui` next to `parakeys`.

## Documentation

Design vault (Obsidian-friendly wiki): **[docs/Home.md](docs/Home.md)**

Open the `docs/` folder as an Obsidian vault. Notes use `[[wikilinks]]`.

## Coordination

[Kanbanlan](docs/workflow/kanbanlan.md) + [GitHub Project 12](https://github.com/users/jmitchel3/projects/12)

## License

TBD.

## Git transport layout

Commit ciphertext and placeholders; never commit unlock keys.

```text
project/
  .env                      # placeholders: KEY=<set in parakeys> (safe to commit)
  .parakeys/
    vault.enc               # encrypted vault (safe to commit)
    config.toml             # optional non-secret metadata (safe to commit)
    local.key               # NEVER commit (file wallet fallback; gitignored)
  .parakeys-agent/          # agent-only material (gitignored)
```

Suggested project `.gitignore` snippet:

```gitignore
.parakeys/local.key
**/.parakeys/local.key
.parakeys-agent/
```

Multi-device: `git pull` gets `vault.enc` + `.env`; restore unlock with `parakeys init --recover '<code>'` (stores to Keychain when available).

## GUI design system

The desktop UI lives under `src/bin/parakeys_gui/`:

- `ds.rs` — design system: scale tokens, semantic color, elevation/shadows, motion, and shared widgets (buttons, tiles, rows, cards, search, empty states)
- `app.rs` — three-pane shell and product behavior built from those widgets
- `main.rs` — entry

**Tokens:** `Space`, `Radius`, `Type`, `Color`, `Layout`, `Motion`, `Elevation`.

**Fluid interactions:** hover/selection animate via `anim` / `anim_fast` (cubic-out), soft fill blends, and raised shadows on interactive chrome.

Edit `ds.rs` first when changing look or feel. Prefer design-system helpers over one-off colors, padding, or paint code in `app.rs`.
