# Sync Strategy

#sync

## Principle

Split payloads:

| Payload | How it moves | Notes |
|---------|----------------|-------|
| **Vault ciphertext** | Git (primary), file, AirDrop | Safe if encrypted |
| **Placeholder manifest** | Git | `.env` with `<set in parakeys>` etc. |
| **Human master key** | Recovery code, AirDrop once, optional iCloud Keychain | **Never** git |
| **Agent grants** | Git and/or chat drop | Encrypted to agent pubkey |
| **Agent private key** | Generated on agent machine | **Never** chat with vault master |

```text
One-time:  key  →  other device (out of band)
Ongoing:   vault → other device (git pull)
```

## Multi-device without *our* servers

Yes. Sync needs a **channel**, not necessarily ParaKeys infra.

| Channel | Our servers? | Always-on? |
|---------|--------------|------------|
| Git push/pull | No | On pull |
| AirDrop / export file | No | Manual |
| User iCloud Drive / Syncthing folder | No | Mostly |
| Chat drop (sealed grant) | No | Manual, great for agents |
| CloudKit | Apple's | Yes on Apple |
| Our API | Yes | Yes |

See [[No Infra Path]], [[Git as Transport]], [[AirDrop and Bootstrap]], [[Chat Drop Flow]].

## Conflict handling (personal)

Keep simple:

- Last-write-wins per environment or per key when possible
- Whole encrypted file conflicts in git are ugly: prefer single-writer habits or re-encrypt from one machine
- Not collaborative Google Docs

## What not to sync

- Absolute local paths as the only identity
- Plaintext `.env` values
- Master key next to vault ciphertext in the same unencrypted package

## Related

- [[Architecture Overview]]
- [[Recovery]]
