# MVP: init + recovery code + key wallet

- Kanbanlan: `KBL-UKVJJGH35NF4PD22VZUBMO6LCU`
- Canonical home: `github`
- Canonical request: [#3](https://github.com/jmitchel3/ParaKeys/issues/3)

## Request

## Outcome
Human can bootstrap a vault with a recovery code and unlock on a fresh profile.

## Acceptance
- [x] `parakeys init` generates vault + recovery code (shown once)
- [x] Recovery code can re-open vault after clearing local key store
- [x] Local key stored in file-based key wallet for v0 (Keychain later OK)

## Decisions

- File wallet at `.parakeys/local.key` (mode 0600), gitignored.
- Recovery code = Crockford-style base32 (RFC 4648 base32) of the raw 32-byte vault key, grouped with hyphens; case/space insensitive on decode.
- `parakeys init --recover CODE` restores local key after verifying the code decrypts the existing vault.
- `--force` overwrites existing vault/key when creating or recovering.
- Best-effort append of `.parakeys/local.key` to project `.gitignore` on init.

## Verification

- `cargo test`: 8 tests (vault + keywallet) passed
- Manual: init → delete local.key → init --recover → vault unlocks

## Delivered result

Working `parakeys init` / `init --recover`. Next: import (#4) and run (#5) load via `load_local_key`.
