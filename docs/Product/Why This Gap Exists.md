# Why This Gap Exists

#product

People have done **pieces** of ParaKeys. Almost nobody has shipped the exact "Passwords.app for dotenv + agent grants" package.

## Adjacent products

| What exists | Gap vs ParaKeys |
|-------------|-----------------|
| 1Password Environments / `op run` | Suite-heavy; not tiny Mac-native Passwords for dotenv |
| Doppler / Infisical / Vault | Team cloud, ops weight |
| dotenvx / SOPS / age | Encrypt-in-git; not Keychain + Passwords UX |
| direnv | Loads files; not the secret store/sync |
| DIY Keychain scripts | No product surface |

## Structural reasons the niche is open

1. **Pain is real but intermittent** (new machine, rare leak) vs 200 website logins.
2. **Power users already bought a suite** (1Password, Doppler).
3. **Apple-shaped shipping is tedious** (Keychain, notarization, entitlements).
4. **Sync is the cliff** where side projects die.
5. **Trust tax** for "put all API keys here."
6. **Env vars are messy** (local/staging/prod, monorepos, Docker, agents).
7. **Money goes to team cloud SaaS** or **open crypto formats**, not personal Passwords-for-dotenv.
8. **Apple could build a slice** of the Mac story; agents on Linux are less their priority.

## Implication

The opportunity is **packaging and taste** (and agent blast-radius design), not inventing secret storage.

## Related

- [[Positioning vs 1Password]]
- [[Product Vision]]
