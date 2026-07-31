# MVP: parakeys run process inject

- Kanbanlan: `KBL-MTZW57SXF5C5XCRJP2PSHVEOIM`
- Canonical home: `github`
- Canonical request: [#5](https://github.com/jmitchel3/ParaKeys/issues/5)

## Request

## Outcome
`parakeys run -- <cmd>` injects vault secrets into child process env without writing plaintext .env.

## Acceptance
- [x] Merge placeholders with vault values
- [x] Child sees real values; file on disk still placeholders
- [x] Missing required secret fails clearly

## Decisions

- Inject all vault keys into child env, then apply `.env` placeholder resolution (missing set-placeholder → hard error).
- Non-placeholder values still on disk pass through (override vault) for gradual migration.
- Do not rewrite `.env` during run; secrets stay process-local.
- Exit with the child process status code.

## Verification

- `cargo test` 11 tests
- Manual: init → import → `run -- sh -c 'echo $DATABASE_URL'` prints secret; `.env` still placeholders

## Delivered result

Core vertical slice complete: init → import → run. Remaining MVP polish: set/unset/list/doctor (#7).
