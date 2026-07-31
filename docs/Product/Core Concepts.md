# Core Concepts

#product #architecture

## Vocabulary

| Term | Meaning |
|------|---------|
| **Environment** | Named set of keys for a project context (e.g. `acme · local`, `acme · staging`) |
| **Vault** | Encrypted store of environments and values (source of truth for secrets) |
| **Manifest** | Non-secret view of key **names** and **status** (often the workspace `.env` with placeholders) |
| **Grant** | Scoped, encrypted package an agent (or other recipient) may decrypt: allowlisted keys only |
| **Binding** | Link between a local folder / git remote and an Environment |
| **Key wallet** | OS-backed store for private keys (Keychain, libsecret, etc.), not the vault contents |
| **Recipient** | Human device or agent identity that can unwrap some ciphertext |
| **Master key** | Human vault key; never given to agents |
| **Agent key** | Separate keypair; only unlocks grants encrypted to that agent |

## Mental model

```text
Human devices  ── full vault decrypt (Touch ID / Keychain)
Agent Linux    ── grant decrypt (subset only)
Chat drop      ── sealed grant update (ciphertext)
Git            ── ciphertext vault + placeholder manifest
```

- **Vault** = source of truth (sync pain A)
- **Grant** = what this agent may materialize (pain B)
- **Manifest** = what exists / is required without values

## Project identity

Do **not** key environments only by absolute path (`/Users/you/...` differs per machine).

Prefer:

1. Stable environment id
2. Optional git remote URL
3. Local binding: "this folder is environment X" (machine-specific)

See [[Architecture Overview]].

## Related

- [[Vault Format]]
- [[Env Placeholder Design]]
- [[Agent Grants]]
- [[Key Wallet]]
