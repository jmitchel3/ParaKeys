# Vault Format

#architecture

Status: **design sketch** (not frozen). Implement MVP with a versioned envelope so format can evolve.

## Goals

- Ciphertext safe to store in git or AirDrop
- Multiple environments per vault or per-project vault file
- Per-environment or per-key encryption possible later
- Recipients: human master key; later agent grants as separate blobs

## Suggested layout (project-local)

```text
project/
  .env                          # placeholders + safe plaintext (committable)
  .parakeys/
    vault.enc                   # encrypted environments (committable)
    config.toml                 # non-secret: env id, recipients metadata
  .parakeys-agent/              # optional local only, gitignored
    grant.enc                   # current agent grant
    agent.key                   # or use OS keyring
```

Exact names TBD; keep secrets out of paths agents casually dump into prompts when possible (still: encrypt).

## Logical model

```text
Vault
  version
  environments[]
    id
    name                    # "acme · local"
    keys[]
      name                  # DATABASE_URL
      ciphertext            # value encrypted to vault key
      updated_at
  recipients[]              # optional metadata: device/agent pubkeys
  revision / updated_at
```

## Encryption principles

- AEAD (e.g. age, libsodium secretbox, or equivalent)
- Server/git never sees plaintext values
- Human master key wraps vault key; stored in [[Key Wallet]]
- Grants are **separate** ciphertext packages encrypted to agent pubkey, not "agent gets master key"

## What is never in the vault file

- Human master key in plaintext
- Agent private keys
- Absolute machine-local paths as sole identity

## Compatibility options

| Approach | Pros | Cons |
|----------|------|------|
| **A. Own format** | Control, grants metadata | Another standard |
| **B. dotenvx-compatible** | Interop | Tied to their evolution |
| **C. SOPS/age file** | Known tools | UX and grants layer still custom |

Early recommendation: **own simple versioned envelope**, optionally import/export dotenvx/SOPS later. See [[Decision Log]].

## Related

- [[Git as Transport]]
- [[Agent Grants]]
- [[Recovery]]
