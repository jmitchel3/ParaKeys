# MVP: scaffold Rust CLI crate

- Kanbanlan: `KBL-KL26MYAI3NEZNNDF5OGAPMBEFE`
- Canonical home: `github`
- Canonical request: [#1](https://github.com/jmitchel3/ParaKeys/issues/1)

## Request

## Outcome
Rust binary `parakeys` builds and exposes a clap CLI skeleton with subcommands stubs: init, import, set, unset, list, run, doctor.

## Acceptance
- [x] `cargo build` succeeds on macOS
- [x] `parakeys --help` lists planned MVP commands
- [x] Crate layout ready for vault module

## Notes
See docs/Roadmap/MVP Scope.md

## Decisions

- Binary crate with clap derive; edition 2021 for broad toolchain comfort.
- Module layout: `cli`, `commands/*`, `vault`, `keywallet`, `envfile`, `error` so later MVP cards drop into place without reshaping.
- Subcommands currently return clear "not implemented" errors pointing at the owning MVP issue.
- File-based key wallet and vault crypto deferred to #2 / #3 (not this card).

## Verification

- `cargo build` succeeds (macOS, rustc 1.92).
- `cargo run -- --help` lists: init, import, set, unset, list, run, doctor.
- `cargo run -- init` exits with not-implemented message (expected until #3).

## Delivered result

Rust CLI scaffold on branch `work/kbl-kl26myai3neznndf5ogapmbefe-mvp-scaffold-rust-cli-crate`. Follow-up: vault format (#2), init/recovery (#3), import (#4), run (#5), set/list/doctor (#7).
