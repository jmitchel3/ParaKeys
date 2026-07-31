# AirDrop and Bootstrap

#sync #security

## Role

Out-of-band path for **keys** and optional one-shot vault transfer. Not the long-term primary channel for secret *updates* (prefer [[Git as Transport]]).

## Bootstrap second human device

**Preferred ongoing:** git has vault; only the key must move once.

1. Mac A: vault key in Keychain; recovery code shown once
2. Mac B: install ParaKeys; enter recovery code **or** receive key via AirDrop package designed for key-only
3. Mac B: pull repo; unlock; run

**Alternative:** AirDrop entire encrypted vault file if git is not in play yet; key still separate or in recovery.

## Optional: Apple-synced encryption key

Master key in iCloud Keychain (`synchronizable` item):

- Same Apple ID, iCloud Keychain on
- Both installs of signed app with same access group
- Vault blob via git/AirDrop still separate

Fallback when iCloud Keychain off: recovery code. See [[Key Wallet]].

## AirDrop rules

| Do | Don't |
|----|--------|
| AirDrop ciphertext vault | AirDrop plaintext `.env` |
| AirDrop key-only bootstrap carefully | Put master key in the same clear package as nothing else protective |
| Prefer recovery code user stores in Apple Passwords | Put recovery code in the agent chat |

## Related

- [[Sync Strategy]]
- [[Recovery]]
- [[No Infra Path]]
