# MVP: set/unset/list + doctor

- Kanbanlan: `KBL-GKMYJJCQNRDHPA3QDNMHCHBWSQ`
- Canonical home: `github`
- Canonical request: [#7](https://github.com/jmitchel3/ParaKeys/issues/7)

## Request

## Outcome
Basic vault editing and inspection without dumping secrets by default.

## Acceptance
- [x] set / unset keys
- [x] list shows names (+ set/missing), reveal is explicit
- [x] doctor warns if .env looks like it contains real secrets again

## Decisions

- `set KEY=value` or `set KEY --value`; updates vault and upserts `.env` placeholder when `.env` exists.
- `unset` removes from vault and marks `.env` as `<not set in parakeys>` when present.
- `list` hides values; `--reveal` prints them.
- `doctor` checks vault, local key, and heuristic secret leak detection in `.env`.

## Verification

- `cargo test` (15 tests including set parse + doctor heuristics)
- CLI: set, list, list --reveal, unset, doctor fail on leak, doctor pass after import

## Delivered result

set/unset/list/doctor implemented on main via this PR.
