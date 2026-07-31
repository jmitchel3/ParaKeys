# MVP: import .env and placeholder rewrite

- Kanbanlan: `KBL-QX7PQVAQWZFQRLFC3Q7XGLIMM4`
- Canonical home: `github`
- Canonical request: [#4](https://github.com/jmitchel3/ParaKeys/issues/4)

## Request

## Outcome
Import a plaintext .env into the vault and rewrite it to placeholders.

## Acceptance
- [x] `parakeys import .env` stores values in vault
- [x] Working tree .env shows only placeholders for secrets
- [x] Non-secret plain values can remain (optional v0: treat all as secrets)

## Decisions

- v0 treats every non-empty, non-placeholder assignment as a secret on import (including DEBUG=true). Safer default; selective plaintext can come later.
- Preserve comments/blank lines; rewrite only imported keys to `<set in parakeys>`.
- Skip values already placeholders so re-import is a no-op when clean.

## Verification

- `cargo test` 11 tests
- Manual: init → write .env with secrets → import → .env is placeholders; vault.enc has no plaintext

## Delivered result

`parakeys import` works. Next: `parakeys run` (#5) resolves placeholders into process env.
