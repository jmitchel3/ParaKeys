# MVP Scope

#mvp

## Goal

Smallest thing that is daily-usable for **one human on one Mac** (or one machine), with a clear path to git multi-device and agents.

## In

| Item | Notes |
|------|-------|
| CLI in Rust (preferred) | Portable core |
| Create vault + recovery code | Once |
| Import `.env` | Values → vault; file → placeholders |
| Set / unset / list keys | List shows names; reveal is explicit |
| Placeholder `.env` write | `<set in parakeys>` / `<not set in parakeys>` |
| `parakeys run -- <cmd>` | Process env inject |
| Local key storage | Keychain on Mac if feasible; file OK for spike |
| Versioned vault envelope | So format can move |

## Out (MVP)

| Item | Why later |
|------|-----------|
| GUI | Polish |
| Git automation beyond "files you can commit" | Docs + convention enough |
| Agent grants | Phase 2 |
| iCloud Keychain sync | Convenience |
| CloudKit / our API | Infra |
| Windows/Linux human polish | After format stable |
| Team sharing | Out of wedge |

## Definition of done (MVP)

1. Import a real project `.env` without leaving secrets in the file
2. `parakeys run` starts the app with working secrets
3. Agent or human reading `.env` sees placeholders only
4. Recovery code can open the vault on a fresh profile (tested once)
5. Vault file is safe to inspect as opaque ciphertext

## Spike order

1. Encrypt/decrypt round trip + file format v0
2. Placeholder rewrite + merge on run
3. Keychain (or dev file key) unlock
4. Recovery code bootstrap
5. Manual second-folder "device" test

## Related

- [[Roadmap]]
- [[Env Placeholder Design]]
- [[Runtime Inject]]
