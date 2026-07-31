# Architecture Overview

#architecture

## High level

```text
┌──────────────────────────────────────────────────────────┐
│  ParaKeys core                                            │
│  vault format · policy · merge · run · grants             │
└────────────┬─────────────────────────────┬────────────────┘
             │                             │
    ┌────────▼────────┐           ┌────────▼────────┐
    │ Key wallet      │           │ Transports        │
    │ (platform)      │           │ git · file · chat │
    │ Keychain / etc. │           │ (ciphertext only) │
    └─────────────────┘           └───────────────────┘
```

## Components (target)

| Component | Role |
|-----------|------|
| **CLI** | `import`, `set`, `keys`, `run`, `agent-*`, export/apply grant |
| **Local vault cache** | Decrypted-at-rest ciphertext; values only after unlock |
| **Key wallet** | Master key / device key / agent private key in OS store |
| **Manifest writer** | Maintains placeholder `.env` ([[Env Placeholder Design]]) |
| **Optional GUI** | Passwords-like list (Mac later; not required for MVP) |
| **Optional agent** | LaunchAgent later; MVP can be CLI-only |

## Data flow: human run

```text
1. Resolve binding (cwd → environment id / git remote)
2. Unlock key wallet (Touch ID / keyring) if needed
3. Load vault ciphertext → decrypt environment
4. Read workspace .env placeholders + plaintext non-secrets
5. Merge: placeholders → values from vault; pass through safe literals
6. Exec child with merged env (prefer not writing secrets back to disk)
```

## Data flow: multi-device

```text
One-time:  master/device key → other Mac (recovery / AirDrop / iCloud Keychain)
Ongoing:   vault ciphertext → other Mac (git pull)
Manifest:  placeholder .env → other Mac (git pull)
```

See [[Sync Strategy]], [[Git as Transport]], [[AirDrop and Bootstrap]].

## Data flow: agent

```text
1. Agent has agent keypair (private key on Linux only)
2. Human encrypts grant (allowlisted keys) to agent public key
3. Grant arrives via git or chat drop
4. agent-run decrypts grant → injects into child only
5. Workspace .env still shows placeholders / status, not values
```

See [[Agent Model]], [[Agent Grants]], [[Chat Drop Flow]].

## Design constraints

1. Vault format is **OS-agnostic** (bytes + version).
2. Keys referenced by **id**, not "the Keychain item."
3. Platform trait: `store_key` / `load_key` / `unlock` ([[Key Wallet]]).
4. Bootstrap always works with **recovery code** (no cloud required).
5. Server is optional forever for the personal path ([[No Infra Path]]).

## Related

- [[Vault Format]]
- [[Runtime Inject]]
- [[Platform Support]]
