# Platform Support

#architecture

## Principle

**Core is portable. Key wallet and polish are per-OS.**

| Piece | Cross-platform? |
|-------|-----------------|
| Encrypted vault format | Yes |
| CLI: import, set, list, run, agent-* | Yes |
| Project binding (git remote, relative) | Yes |
| Crypto | Yes |
| Recovery codes | Yes |
| Git / file / chat transports | Yes |
| Keychain / biometrics UX | Per OS |
| Passwords-like GUI | Mac first, optional elsewhere |
| iCloud Keychain key sync | Mac only |

## Target matrix

| Feature | macOS | Linux | Windows |
|---------|-------|-------|---------|
| CLI + git vault | Full | Full | Full |
| Key in OS store | Keychain | libsecret | CredManager |
| Biometric unlock | Strong | Variable | Hello when available |
| Passwords-like GUI | First | Later / optional | Later / optional |
| Auto key via iCloud | Optional | No | No |
| Agent CLI | Yes | **Primary agent home** | Yes |

## Implementation sketch (Rust-friendly)

Single CLI with platform modules:

- `macos` → security-framework / Keychain
- `linux` → secret-service + headless file fallback
- `windows` → Credential Manager / DPAPI

GUI can be separate and Mac-first forever without blocking Linux agents.

## Shipping notes

- Notarized Mac app eventually for GUI / Keychain entitlements comfort
- Apple Developer Program needed for CloudKit and polished distribution; **not** required for pure local + git experiments
- Linux agents often lack a GUI keyring: design [[Headless Linux]] from the start

## Related

- [[Key Wallet]]
- [[Roadmap]]
- [[No Infra Path]]
