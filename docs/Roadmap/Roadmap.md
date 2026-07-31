# Roadmap

#mvp #product

## North star

Passwords-like comfort for project secrets + multi-device sync + agent grants that stop casual `.env` vacuuming.

## Phases

### Phase 0: Local human MVP

- Encrypted vault on disk
- Key in Keychain (or password-wrapped file for dev)
- `import` plaintext `.env` → vault + rewrite placeholders
- `run` injects into child
- Project binding (cwd / config)
- `doctor` basic checks

See [[MVP Scope]].

### Phase 1: Git transport

- Documented layout (`.parakeys/`, placeholder `.env`)
- Commit ciphertext safely
- Second machine: recovery code bootstrap + pull
- `manifest sync`

### Phase 2: Agents

- `agent keygen`
- `grant create` allowlist
- `agent apply` / `agent-run`
- Chat drop instructions (sealed blob)
- Headless key file support

### Phase 3: Mac polish

- Passwords-like GUI (optional)
- Touch ID UX
- Optional iCloud Keychain for human key sync

### Phase 4: Wider platforms

- Linux / Windows key wallet backends for **humans**
- Agent path already Linux-first in phase 2

### Phase 5: Optional infra (only if needed)

- Always-on sync API or CloudKit
- Device revoke, audit, short-lived grants

## Explicit non-phase-0

- Team admin, SSO
- Browser autofill
- Full 1Password replacement
- Perfect agent sandboxing

## Related

- [[MVP Scope]]
- [[Open Questions]]
- [[No Infra Path]]
