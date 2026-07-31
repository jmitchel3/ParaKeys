# MVP: vault format v0 encrypt/decrypt

- Kanbanlan: `KBL-S3ATJE2RUJDHHCCLEGL3GFNGKY`
- Canonical home: `github`
- Canonical request: [#2](https://github.com/jmitchel3/ParaKeys/issues/2)

## Request

## Outcome
Versioned encrypted vault file format with encrypt/decrypt round trip.

## Acceptance
- [x] Create vault envelope (version + ciphertext)
- [x] Encrypt/decrypt key-value environment data
- [x] Unit tests for round trip and bad key failure

## Notes
docs/Architecture/Vault Format.md — own simple versioned envelope for v0.

## Decisions

- Own JSON envelope (`format: parakeys-vault`, `version: 0`) with base64 nonce + ciphertext so the file is git-friendly and human-inspectable as opaque ciphertext.
- AEAD: ChaCha20-Poly1305; 32-byte vault key; 12-byte random nonce per encrypt.
- Plaintext payload is JSON `{"keys": { "NAME": "value", ... }}` (single environment map for v0; multi-env later).
- Default path: `.parakeys/vault.enc` under the project root.
- Key wallet / recovery wrapping deferred to #3; this card only defines crypto + file IO APIs.

## Verification

- `cargo test`: 5 vault unit tests passed
  - encrypt_decrypt_round_trip
  - wrong_key_fails
  - envelope_json_round_trip
  - reject_unknown_format_version
  - save_and_load_file (asserts plaintext absent from file)

## Delivered result

`src/vault/mod.rs` with `VaultData`, `VaultKey`, `VaultEnvelope`, encrypt/decrypt/load/save helpers. Next: init + recovery (#3) stores the vault key and creates an empty vault file.
