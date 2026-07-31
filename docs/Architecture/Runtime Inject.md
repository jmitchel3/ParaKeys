# Runtime Inject

#architecture

## Goal

Materialize secrets into a **child process** (or explicit export), not into a committed file and not into agent chat by default.

## Primary interface

```bash
parakeys run -- <command> [args...]
# example
parakeys run -- npm start
parakeys run -- ./scripts/migrate.sh
```

Behavior:

1. Unlock key wallet if needed
2. Resolve environment for cwd / flags
3. Merge placeholder `.env` + vault values ([[Env Placeholder Design]])
4. `exec` child with merged environment
5. Prefer **never** writing plaintext secrets back to `.env` on disk

## Why not write a real `.env` by default

- Agents and backups re-read the tree
- Accidental commit risk returns
- Sync of plaintext across cloud folders returns

Exception (explicit, advanced): temporary file in secure temp with strict permissions, deleted after run. Default should be process env only.

## Agent variant

```bash
parakeys agent-run -- <command>
```

Same inject path, but decrypt **grant** only; enforce allowlist. See [[Agent Grants]].

## Shell export (secondary)

```bash
eval "$(parakeys export --format shell)"
```

Useful; easier to leak via shell history and parent environment. Document as power-user and less agent-safe than `run`.

## Binding resolution

Order sketch:

1. `--env <id>` flag
2. `.parakeys/config` in cwd / parents
3. Git remote match to known environment
4. Prompt / error if ambiguous

Paths alone are insufficient across machines ([[Core Concepts]]).

## Related

- [[Architecture Overview]]
- [[Env Placeholder Design]]
- [[Agent Model]]
