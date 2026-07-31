# Agent Grants

#agents #security

## What a grant is

A **scoped encrypted package**:

- Which environment / project
- Which key **names** are included (allowlist)
- Ciphertext of those values only
- Encrypted to an **agent public key** (recipient)
- Optional: issued_at, expires_at, grant id

Not the whole vault. Not the human master key.

## Creation (human)

Sketch:

```text
parakeys grant create \
  --to agent-ci.pub \
  --keys OPENAI_API_KEY,GITHUB_TOKEN \
  --out grant.pk
```

Or GUI: Share → Agent → pick keys → export / copy sealed blob.

## Use (agent)

```bash
parakeys agent apply grant.pk    # store local grant ciphertext
parakeys agent-run -- ./do-work.sh
```

Apply replaces the local grant file; run injects allowlisted values into the child only.

## Delivery channels

| Channel | Use |
|---------|-----|
| Git (committed grant or recipients list + build) | Ongoing project setup |
| [[Chat Drop Flow]] | "Right now in this session" |
| scp / AirDrop of grant file | Manual |

## Rules of thumb

1. Prefer **narrow allowlists** (not "entire env")
2. One agent identity per machine or purpose (`vps-1`, `ci`, `codex`)
3. Rotate agent keys when a box is burned
4. Never put human recovery codes in agent grants
5. Default deny: keys not in grant do not materialize even if placeholder says `<set in parakeys>`

## Related

- [[Agent Model]]
- [[Chat Drop Flow]]
- [[Downsides]]
