# No Infra Path

#sync #mvp

## Answer

**Yes: start user-less and infra-less.**  
Multi-device needs a *channel*, not necessarily *our* servers.

## Ladder

```text
0. Local only                 ← no infra
1. Recovery / AirDrop key + git vault
2. Optional user Drive folder for vault file
3. CloudKit private DB        ← Apple infra, not ours
4. Our sync API               ← our infra (only if needed)
```

## What "user-less" means

| Meaning | v1? |
|---------|-----|
| No ParaKeys account / email login | Yes |
| No servers we operate | Yes |
| No network at all | Single machine yes; multi-device needs some pipe (git/AirDrop/Drive) |

## When infra becomes worth it

- Always-on multi-device without git pull discipline
- Instant revoke / device list / remote wipe epoch
- Short-lived tokens for untrusted agents at scale
- Real audit logs
- Teams and sharing policies

Until then: **encrypted vault format as if sync exists**; transport is a plugin.

## CloudKit note

Requires paid [[Platform Support|Apple Developer Program]]. Not needed for MVP.

## Related

- [[Sync Strategy]]
- [[MVP Scope]]
- [[Roadmap]]
