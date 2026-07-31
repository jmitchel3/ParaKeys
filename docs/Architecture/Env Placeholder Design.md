# Env Placeholder Design

#architecture #product #agents

## Idea

Keep a familiar `.env` in the project, but make it a **status manifest**, not a secret store.

```bash
# managed by parakeys - values are not stored here
DATABASE_URL=<set in parakeys>
OPENAI_API_KEY=<set in parakeys>
STRIPE_SECRET_KEY=<not set in parakeys>
DEBUG=true
```

- **Secrets** → placeholders only
- **Non-secrets** (flags, public URLs) → real values allowed in-file
- Agent `read_file(".env")` learns **names + presence**, not material

## Why

| Goal | How this helps |
|------|----------------|
| Clear what's up | Humans and agents see the same checklist |
| Limit spread | No values to vacuum into chat |
| Familiar | Still "the `.env` file" |
| Sync | Placeholders commit with the repo; values never do |
| Drift | `<not set in parakeys>` = incomplete setup |

## Placeholder vocabulary (v1)

Stable and greppable:

| Placeholder | Meaning |
|-------------|---------|
| `<set in parakeys>` | Value exists in vault (for recipients who can unlock) |
| `<not set in parakeys>` | Key declared; no value stored yet |

## Later states (optional)

| Placeholder | Meaning |
|-------------|---------|
| `<set in parakeys; not granted>` | In vault for humans; this agent grant cannot materialize |
| `<parakeys:set>` / `<parakeys:missing>` | Machine-parseable short form |

Start with the two human-readable forms unless tooling needs shorter tokens.

## Runtime merge ([[Runtime Inject]])

1. If value is a parakeys placeholder → resolve from vault/grant (or fail if missing / not granted)
2. If value is normal plaintext → use as-is
3. Policy choice: inject vault-only keys not listed in `.env`, **or** treat `.env` as allowlist (prefer **allowlist for agents**)

## Commands (sketch)

| Command | Behavior |
|---------|----------|
| `parakeys import .env` | Read real values once; write vault; rewrite file to placeholders |
| `parakeys manifest sync` | Refresh placeholder file from vault key names + set/missing |
| `parakeys doctor` | Fail if `.env` looks like it contains real secrets again |
| `parakeys run -- <cmd>` | Resolve placeholders into child env |

## Commit policy

- **Do** commit placeholder `.env` (or generate committed `.env.example` from it; prefer one clear story)
- **Do not** commit resolved values
- Pre-commit / doctor: detect high-entropy values where placeholders should be

## Caveats

| Issue | Mitigation |
|-------|------------|
| App reads `.env` without `parakeys run` | Gets placeholders; document runner, or optional private gitignored override |
| Strict parsers break on placeholder strings | Prefer resolve-into-process-env; treat placeholders as empty for naive loaders if needed |
| Duplicate source of truth | Manifest sync from vault; `.env` remains declared set of keys |
| Habit of pasting secrets back | doctor + education |

## Related

- [[Problem]]
- [[Runtime Inject]]
- [[Agent Model]]
