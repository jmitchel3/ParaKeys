# MVP: manifest sync from vault to .env
- Kanbanlan: KBL-466DACH6WNDCJOFHK7T22246X4
- [#14](https://github.com/jmitchel3/ParaKeys/issues/14)
## Acceptance
- [x] manifest sync updates .env placeholders from vault
- [x] no secret values on disk
- [x] unit test sync_writes_placeholders_only
## Verification
cargo test including commands::manifest::tests::sync_writes_placeholders_only
