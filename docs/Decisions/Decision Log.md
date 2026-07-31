# Decision Log

#product #architecture

Record decisions when they stick. Format: date, decision, rationale, alternatives.

---

## 2026-07-30: Product wedge

**Decision:** Personal project/env secrets + multi-device sync + agent blast-radius control. Not a general password manager.

**Rationale:** Daily pain is device sync and agents reading `.env`, not website logins.

**Tagline:** Like Apple Passwords, but for dotenv.

**Alternatives:** Full 1Password competitor; team cloud secret manager only.

---

## 2026-07-30: No infra for v1

**Decision:** Start without ParaKeys accounts or servers. Git + recovery/AirDrop for multi-device.

**Rationale:** Unblocks shipping; vault format stays sync-ready.

**Alternatives:** CloudKit or custom API first.

---

## 2026-07-30: Git as primary ongoing transport

**Decision:** Encrypted vault (and placeholder manifest) can live in the repo; keys never do.

**Rationale:** dotenvx-style; no servers; works for agents and second Macs.

**Alternatives:** iCloud Drive only; always-on API only.

---

## 2026-07-30: Placeholder `.env` as agent-safe manifest

**Decision:** Workspace `.env` uses `<set in parakeys>` / `<not set in parakeys>` (or equivalent) instead of secret values.

**Rationale:** Agents and humans see status; `read_file` does not yield material.

**Alternatives:** Only proprietary manifest path; refs like `pk://...` only.

---

## 2026-07-30: Humans get vault; agents get grants

**Decision:** Separate agent recipient keys and allowlisted grants; never give agents the human master key.

**Rationale:** Limit spread and rotate agent compromise independently.

**Alternatives:** Shared project private key on every machine including agents.

---

## 2026-07-30: Process inject over rewriting plaintext `.env`

**Decision:** Default materialization is `parakeys run` / `agent-run` into child env.

**Rationale:** Avoid reintroducing secret files into the tree.

**Alternatives:** Write gitignored `.env.local` on each run (optional later, not default).

---

## Pending

See [[Open Questions]].
