# Chat Drop Flow

#agents #sync

## Idea

Drop something in the agent chat so it can **replace a local grant file** and run. Chat is a delivery truck for **ciphertext**, not for raw keys.

## Good drop

- Sealed grant blob (`grant.pk` / `pkgrant1.…`)
- Short instruction: run `parakeys agent apply` then `agent-run`

## Bad drop

| Drop | Why bad |
|------|---------|
| Plaintext `.env` or raw API keys | Chat logs, traces, provider history |
| Human recovery / master key | Agent becomes the vault |
| Ciphertext + private key in one message | Same as giving away the grant forever to the log |

## Prerequisite

Agent machine already has **agent private key** from `parakeys agent keygen` (or equivalent). The chat message alone should not be the only secret forever without a recipient key model.

## Flow

```text
You (Mac)  →  copy sealed grant  →  paste in agent chat
Agent      →  writes local grant file  →  agent-run
```

Example user message:

> Apply this ParaKeys grant and run the tests.
>
> ```
> pkgrant1.BASE64ORFILE...
> ```

Agent (or human-approved command):

```bash
parakeys agent apply ./grant.pk   # or stdin
parakeys agent-run -- npm test
```

## Compared to git

| | Git | Chat drop |
|--|-----|-----------|
| Ongoing updates | Strong | Manual each time |
| Audit | Repo history | Chat history (messy) |
| UX for one agent now | Clunky | Excellent |
| Risk of plaintext habit | Lower | Higher if lazy |

**Use both:** git for default project ciphertext; chat for immediate grant push to this agent.

## Related

- [[Agent Grants]]
- [[Sync Strategy]]
- [[Env Placeholder Design]]
