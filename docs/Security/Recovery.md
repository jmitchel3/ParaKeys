# Recovery

#security #sync

## Problem

E2E and local-only keys mean: **lose all keys and recovery material ⇒ lose the vault** (or wait for ciphertext that can never open).

Be honest in product copy. Apple Passwords leans on iCloud account recovery; a third-party app must offer an explicit recovery kit.

## Recovery code

- Generated once at vault creation
- Shown once; user stores in Apple Passwords / printout / offline
- Sufficient to unwrap master key on a new device
- **Never** put in agent chat, git, or grant files

## Device add

1. Install app / CLI
2. Enter recovery code **or** approve via existing device (AirDrop / optional iCloud Keychain)
3. Store device-local wrapped key in [[Key Wallet]]
4. Pull ciphertext from git

## Device lose / revoke (no infra)

- Generate new master key or rotate vault key
- Re-encrypt vault
- Push new ciphertext
- Update remaining devices
- Old offline copies of ciphertext become useless only after rotation (old keys still open old blobs until rotated)

With infra later: device list + epoch flags. See [[No Infra Path]].

## Agent compromise

- Do not rotate human master key if only agent grant key leaked (ideal)
- Rotate **agent** keypair; issue new grant; remove old grant files
- Re-encrypt any grants that used the old agent pubkey

## Related

- [[AirDrop and Bootstrap]]
- [[Key Wallet]]
- [[Threat Model]]
