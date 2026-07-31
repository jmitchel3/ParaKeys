# Positioning vs 1Password

#product

## Bottom line

| Question | Answer |
|----------|--------|
| Better than 1Password overall? | **No** |
| Better password manager? | **No** |
| Can be better for project/agent env secrets with light git-native flow? | **Yes, if UX is ruthless** |

Do not market a "1Password killer." Market a **narrow tool** for project runtime secrets and agent grants.

## Possible framing

> 1Password is for your identity. ParaKeys is for your project and agent runtime secrets.

Or:

> dotenvx's transport + agent grants + Passwords-like local unlock.

People can use both.

## Where ParaKeys can win

| Job | Why |
|-----|-----|
| Dotenv / project as primary object | Not a side feature of a vault item |
| Linux coding agents | First-class grants, chat-drop sealed blob, `agent-run` |
| Git-native | Ciphertext with the repo |
| Minimal personal setup | No suite account required for core loop |
| Agent-safe workspace | Placeholder `.env`; values not in tree |
| Scoped inject | `run` / path / repo mental model |

## Where 1Password wins

| Job | Why |
|-----|-----|
| Logins, cards, docs, autofill | Full product |
| Trust / brand / compliance | Years of scrutiny |
| Teams, admin, SSO | Mature |
| Cross-platform polish | Apps everywhere |
| Sync and revoke | Real infra |
| Already installed | `op run` is good enough for many |
| Environments product direction | They are moving into this space |

## Feature sketch

| Capability | 1Password | ParaKeys (target) |
|------------|-----------|-------------------|
| Project env vars | Environments, items, `op://` | Core |
| Run with secrets | `op run` | `parakeys run` / `agent-run` |
| Avoid plaintext `.env` | Yes | Yes + placeholder status file |
| Git as primary transport | No | Yes |
| Chat sealed grant | Possible, not the pitch | Natural pitch |
| Agent as recipient | Service accounts (heavier) | Designed in |
| No account / no infra v1 | No | Yes |
| Full password manager | Yes | No |

## Competitive risk

Main enemy is **good enough**, not crypto impossibility: plaintext `.env`, or `op run` for people who already pay for 1Password.

## Related

- [[Product Vision]]
- [[Why This Gap Exists]]
- [[Downsides]]
