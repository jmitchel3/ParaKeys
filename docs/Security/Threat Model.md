# Threat Model

#security

## In scope goals

1. Secrets not sitting as plaintext in git or casually syncable folders
2. Second device access requires key bootstrap, not just repo clone
3. Agents cannot obtain full secret material by reading workspace `.env`
4. Agents receive **least privilege** via grants
5. Our (optional) servers never need plaintext values (E2E)

## Assets

| Asset | Sensitivity |
|-------|-------------|
| Env values (API keys, DB URLs) | High |
| Human master key / recovery code | Critical |
| Agent private key | High for grant scope |
| Placeholder `.env` / key names | Low–medium (metadata) |
| Grant ciphertext | High if agent key also stolen |

## Adversaries / scenarios

| Scenario | Expected outcome |
|----------|------------------|
| Clone public/private repo only | Ciphertext + placeholders; no values without key |
| Agent `read_file(".env")` | Placeholders / status only |
| Prompt injection on agent without grant | No values in tree; limited harm from file reads |
| Prompt injection on agent **with** live injected env | **Can still exfiltrate** process env; mitigate with narrow grants + short life |
| Stolen locked laptop | Keychain/biometric gates help; not perfect |
| Stolen unlocked session | Treat as full user compromise |
| Malware as same user on agent host | Can read agent key file / memory; grants limit blast radius vs master key |
| Malicious sync server (if any) | Sees ciphertext only if E2E held |

## Non-goals

- Protect secrets from root on a machine that already unlocked them
- Stop every exfil path from a tool-enabled agent that was given live credentials
- Replace enterprise DLP / HSM / cloud secret managers for regulated prod

## Default security policies

1. Never commit resolved secrets
2. Never give agents the human master key
3. Deny-by-default grants (allowlist)
4. Prefer `run` / `agent-run` over long-lived exported shell env
5. doctor/pre-commit against re-plaintexting `.env`
6. Recovery codes offline; not in agent chat

## Related

- [[Downsides]]
- [[Agent Grants]]
- [[Env Placeholder Design]]
- [[Recovery]]
