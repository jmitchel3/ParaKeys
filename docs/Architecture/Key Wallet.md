# Key Wallet

#architecture #security

## Role

The **key wallet** holds cryptographic keys that unwrap vaults and grants. It does **not** hold the dotenv values themselves (those live in the encrypted vault / grant blobs).

## Human keys

| Key | Purpose |
|-----|---------|
| Vault master key | Decrypts personal vault ciphertext |
| Device-wrapped form | Master key sealed for this machine's OS store |
| Recovery code | Bootstrap a new device without the old machine |

## Agent keys

| Key | Purpose |
|-----|---------|
| Agent private key | Decrypts grants encrypted to this agent |
| Agent public key | Shared with human app for encrypt-to-agent |

**Never** put the human master key or recovery code on the agent. Agents get **grants**, not the vault key. See [[Agent Model]].

## Platform backends

| OS | Store |
|----|--------|
| macOS | Keychain (+ optional Touch ID / SecAccessControl) |
| Linux | libsecret / Secret Service when available; file fallback for headless |
| Windows | Credential Manager / DPAPI |

Abstract early:

```text
store_key(id, bytes)
load_key(id) -> bytes
unlock()     # biometric / keyring unlock if applicable
```

See [[Platform Support]], [[Headless Linux]].

## iCloud Keychain (optional Mac nicety)

Synchronizable Keychain items can move a **small master key** across Macs signed into the same Apple ID.

- Nice for multi-Mac human unlock
- Not available the same way on Linux agents
- Never the only bootstrap path; always support recovery code

See [[AirDrop and Bootstrap]], [[Sync Strategy]].

## Related

- [[Recovery]]
- [[Threat Model]]
- [[Vault Format]]
