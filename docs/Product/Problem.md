# Problem

#product

## Pain A: sync across devices

Developers keep project secrets in plaintext `.env` files.

- New Mac: hunt for the old file, Slack yourself keys, or miss half of them
- Two machines drift (staging key updated on laptop only)
- Temptation to commit secrets, put them in iCloud Drive unencrypted, or paste into chat

Password managers help, but the unit of work is still "project env," not "login for github.com."

## Pain B: agents and blast radius

Coding agents and tools that can `read_file` the workspace will happily open `.env`.

```text
Plain .env on disk
  → agent tool reads it
  → full secret material in context / logs / traces
  → prompt injection or careless tool use → exfil
```

People want agents to:

- Know **which** keys a project needs
- Know whether they are **set**
- Run commands that need secrets

They do **not** want the agent to load every value into the transcript just because a file exists.

## Why plaintext `.env` cannot solve B

A file of `KEY=value` cannot be "readable for names only." Anything that opens the file gets values.

So the fix is not a smarter plaintext `.env`. It is:

- **Manifest / placeholder `.env`** in the workspace ([[Env Placeholder Design]])
- **Real values** only in an encrypted vault / grant
- **Materialize** only at process boundary ([[Runtime Inject]])

## Success criteria

| Pain | Success looks like |
|------|--------------------|
| A | Second device: unlock once, `parakeys run` works; no hand-copied secrets |
| B | Agent can list keys and status; `read_file(".env")` does not yield secret material |
| Both | One vault model; humans get full unlock; agents get scoped grants |

## Related

- [[Product Vision]]
- [[Env Placeholder Design]]
- [[Agent Model]]
- [[Threat Model]]
