# Headless Linux

#agents #architecture

## Reality

Coding agents and VPS workers often run **without** a GUI keyring unlock session. libsecret may be unavailable or unlocked only after login you never do.

## Design implications

1. **Agent private key file** with `0600` (or equivalent) is a valid backend, not only "OS keyring."
2. Document threat: root or same-user malware can read the agent key file.
3. Prefer **grants** (subset) so stolen agent key ≠ full human vault.
4. Optional: load grant decrypt key from a secrets mount / tmpfs provided by the orchestrator.
5. `agent-run` should work in CI-like environments with explicit key path flags.

## Sketch

```bash
parakeys agent keygen --out ~/.config/parakeys/agent.key
parakeys agent apply grant.pk
parakeys agent-run --env-file .env -- ./run.sh
```

## Related

- [[Key Wallet]]
- [[Platform Support]]
- [[Agent Grants]]
