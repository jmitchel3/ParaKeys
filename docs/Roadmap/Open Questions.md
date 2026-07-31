# Open Questions

#mvp #product

Soft decisions. Capture answers in [[Decision Log]] when frozen.

## Format

1. Own vault envelope vs dotenvx-compatible vs SOPS/age first?
2. One vault file per repo vs one global vault with many projects?
3. Exact on-disk paths (`.parakeys/`, filename extensions)?

## Manifest

1. Commit `.env` with placeholders, or commit `.env.example` and gitignore `.env`?
2. Allow plaintext non-secrets mixed in `.env` (recommended yes)?
3. Placeholder exact strings frozen for v1 parsers?

## Runtime

1. If key is in vault but missing from `.env`, inject anyway or require manifest entry?
2. For agents, is `.env` the allowlist even beyond grant? (Grant should win.)

## Sync

1. Is git the documented primary multi-device path for v1? (Lean yes.)
2. Support global user vault outside repos for non-git projects?

## Agents

1. Grant format: separate files vs multi-recipient vault encryption (age-style recipients)?
2. TTL on grants without infra: honor `expires_at` only on agent CLI clock?
3. Should agents ever get a `get_secret` tool, or only `run` inject?

## Product

1. Name "ParaKeys" final?
2. Open source from day one? (Helps trust.)
3. Mac GUI before or after agent grants?

## Security

1. Minimum key length / KDF choices?
2. How aggressive is `doctor` on false positives?

## Related

- [[Decision Log]]
- [[MVP Scope]]
