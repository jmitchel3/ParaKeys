# Downsides

#security #product

Honest costs of this design. None kill the idea; all shape scope and messaging.

## Product / market

- Overlaps 1Password / Doppler / dotenvx ("I already have that")
- Trust barrier for a new secrets app
- Plaintext `.env` habit is sticky
- Apple could nibble the Mac-only side later
- Agent workflows evolve; keep plain `npm start` useful too

## Git transport

- Not instant (push/pull)
- Ciphertext merge conflicts are painful
- Key distribution still out-of-band
- Some orgs forbid even encrypted secrets in git
- Revoke is slow without infra
- Repo ACL + key possession = access model

## Agents

- Live process env can still be printed or sent out
- Over-broad grants recreate god-mode keys on Linux
- Long-lived agent keys on a VPS are standing credentials
- False sense of safety if users still paste plaintext into chat
- Weak audit offline (who used which key when)
- One key per agent is correct and annoying; people will reuse keys

## Security limits

- Unlocked machine / same-user malware not fully stoppable
- Recovery codes get mishandled (screenshots, Notes)
- No remote wipe without infra
- Metadata leaks (key names, project structure)

## Engineering

- Keychain / libsecret / DPAPI edge cases
- Headless Linux fights pretty keyring story
- Cross-OS UX will diverge (Mac GUI vs Linux CLI)
- Format and grant compatibility can scope-creep
- Support: "decrypt failed" = key mismatch class of bugs

## UX

- Extra wrapper (`parakeys run`) vs bare commands
- Onboarding is multi-step (vault + key + grant)
- Vocabulary (vault / grant / recovery / agent key) needs ruthless naming

## Design around vs accept

**Accept (personal / small-team):** git delay, no instant revoke, Mac-richer UX, format overlap with dotenvx.

**Design hard:** agent over-scoping, headless key storage, master key never on agent, clear human vs grant split.

**Needs infra later if required:** instant revoke, short-lived untrusted agent tokens, real audit, push-without-git.

## Headline risk

**The agent grant model is the value and the liability:** powerful when narrow and rotatable; dangerous if it becomes "full vault key on the Linux box."

## Related

- [[Threat Model]]
- [[Positioning vs 1Password]]
- [[Agent Grants]]
