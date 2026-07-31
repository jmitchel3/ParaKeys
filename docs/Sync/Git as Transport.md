# Git as Transport

#sync

## Idea

Copy dotenvx / SOPS insight: **git moves ciphertext; the key never lives in the repo.**

```text
Git  ── moves ──►  encrypted vault + placeholder .env
Key wallet ── holds ──►  decryption key
CLI  ── does ──►  unlock, edit, inject
```

Multi-device for anyone who already clones the project. No ParaKeys account required.

## What is committed

| Path | Commit? | Content |
|------|---------|---------|
| `.env` placeholders | Yes | Names + set/not-set |
| `.parakeys/vault.enc` | Yes | Ciphertext |
| Agent private key | **No** | Local / keyring |
| Human recovery code | **No** | Offline / Passwords app |
| Resolved plaintext `.env` | **No** | Antipattern |

## Flow: second Mac

1. Clone / pull (ciphertext + placeholders)
2. Bootstrap key (recovery code or AirDrop / iCloud Keychain)
3. `parakeys run` works
4. Later: pull gets updated secrets

## Flow: agent machine

1. `parakeys agent keygen` (local private key)
2. Human encrypts grant to agent pubkey (or project recipients list)
3. Grant file in repo **or** chat drop
4. `parakeys agent-run`

Git can carry grants; chat is optional UX. See [[Agent Grants]].

## Pros

- No infra for multi-device
- Devs already live in git
- History of encrypted updates (limited reviewability)
- New laptop: clone + restore key

## Cons

| Issue | Reality |
|-------|---------|
| Not instant | Sync cadence is push/pull |
| Ciphertext merge conflicts | Ugly; avoid concurrent vault edits |
| Key still out-of-band | Bootstrap problem remains |
| Some orgs forbid secrets in git even encrypted | Policy |
| Revoke is slow | Re-encrypt + push + rotate agent keys |
| Repo + private key = access | Git ACL is part of threat model |

## Related

- [[Sync Strategy]]
- [[Vault Format]]
- [[Downsides]]
