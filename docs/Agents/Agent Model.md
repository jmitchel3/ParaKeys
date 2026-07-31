# Agent Model

#agents #product

## Goal

Simple way to share project keys with **agents that live on Linux** (and any headless runner), without:

- Pasting plaintext keys into chat
- Leaving a full plaintext `.env` in the tree for `read_file`
- Giving the agent the human vault master key

## Principle

**Humans get the vault. Agents get a grant.**

| Actor | Gets |
|-------|------|
| Human Mac | Full environment (after unlock) |
| Agent | Allowlisted keys only, via grant + `agent-run` |
| Agent chat / tools | Placeholder `.env` + maybe key list; not values by default |

## Workspace the agent sees

```bash
DATABASE_URL=<set in parakeys>
OPENAI_API_KEY=<set in parakeys>
AWS_SECRET_ACCESS_KEY=<set in parakeys; not granted>   # later
DEBUG=true
```

Reading the file answers configuration shape, not secret material. See [[Env Placeholder Design]].

## Lifecycle

```text
1. agent keygen on Linux
2. human approves recipient + key allowlist
3. grant delivered (git or chat drop)
4. agent-run materializes env into child process
5. rotate: new grant, revoke old agent key as needed
```

## What this does not fully stop

An agent **process** that already has env vars can still `printenv` or exfiltrate. ParaKeys reduces:

- Secrets sitting in the repo tree
- Full vault on the agent box
- Accidental load of all values into context via `read_file(".env")`

It does not make a malicious or injected agent with live credentials harmless. See [[Threat Model]].

## Related

- [[Agent Grants]]
- [[Chat Drop Flow]]
- [[Headless Linux]]
