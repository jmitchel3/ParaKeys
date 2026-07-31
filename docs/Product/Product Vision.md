# Product Vision

#product

## Pitch

> **Like Apple Passwords, but for dotenv.**

The unit of storage is a project's env, not a website login.

| Apple Passwords | ParaKeys |
|-----------------|----------|
| Login: site + username + password | Project: path / git remote + keys + values |
| iCloud Keychain sync | Encrypted vault via git (and optional later sync) |
| Face ID / Touch ID unlock | Same: Keychain + biometrics |
| Autofill into Safari | Inject into a process (`parakeys run`) |
| Never store passwords in Notes | Never leave plaintext secrets in the repo |

## Wedge

The underserved intersection:

```text
Passwords-grade local unlock
  × dotenv / project as primary object
  × path or repo scoped run
  × personal, light, no team cloud required (v1)
  × agent grants (list keys, not values; scoped materialize)
```

Not "another Vault Enterprise." Not "replace 1Password for logins."

## Primary jobs

1. **Human multi-device:** same project secrets on every Mac without scp'ing `.env`.
2. **Agent-safe workspace:** reading `.env` answers "what's configured?" not "here are the secrets."

## Non-goals (early)

- Browser autofill for websites
- Full password manager (cards, identity docs)
- Enterprise SSO / admin console (later maybe)
- Perfect protection against a compromised agent process that already has live env vars
- Always-on cloud sync on day one ([[No Infra Path]])

## Product thesis (short)

> dotenvx-style transport (encrypted secrets can live with the project) + Apple Passwords feeling (local unlock) + first-class agent grants.

## Related

- [[Problem]]
- [[Positioning vs 1Password]]
- [[Why This Gap Exists]]
- [[Roadmap]]
