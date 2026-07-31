# ParaKeys

> **Like Apple Passwords, but for dotenv.**

Mac-first tool for project env secrets: sync across personal devices, unlock with Keychain / biometrics, and give Linux agents narrow grants without a world-readable `.env` full of values.

## Daily pain

1. **Sync** project keys across devices without hand-copying `.env` files.
2. **Limit spread** when an agent reads the workspace: it should see *what is configured*, not vacuum every secret into chat/logs/tools.

## One-sentence product

Sync an encrypted project vault across your machines; give agents a narrow grant and a key list, never a world-readable `.env` full of values.

## Start here

| Note | What it covers |
|------|----------------|
| [[Product Vision]] | Analogy, wedge, non-goals |
| [[Problem]] | Why plaintext `.env` fails |
| [[Core Concepts]] | Vault, grant, manifest, placeholders |
| [[Architecture Overview]] | Layers and data flow |
| [[Roadmap]] | What to build first |
| [[Open Questions]] | Decisions still soft |

## Map

### Product
- [[Product Vision]]
- [[Problem]]
- [[Core Concepts]]
- [[Positioning vs 1Password]]
- [[Why This Gap Exists]]

### Architecture
- [[Architecture Overview]]
- [[Vault Format]]
- [[Env Placeholder Design]]
- [[Runtime Inject]]
- [[Key Wallet]]
- [[Platform Support]]

### Sync
- [[Sync Strategy]]
- [[Git as Transport]]
- [[AirDrop and Bootstrap]]
- [[No Infra Path]]

### Agents
- [[Agent Model]]
- [[Agent Grants]]
- [[Chat Drop Flow]]
- [[Headless Linux]]

### Security
- [[Threat Model]]
- [[Downsides]]
- [[Recovery]]

### Roadmap
- [[Roadmap]]
- [[MVP Scope]]
- [[Open Questions]]

### Decisions
- [[Decision Log]]

## Tags

Use Obsidian tags freely: `#product` `#architecture` `#sync` `#agents` `#security` `#mvp`
