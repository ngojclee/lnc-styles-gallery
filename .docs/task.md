# Task Tracker — `lnc-styles-gallery` Rust Migration (`v1.0.0.0`)

- [x] `ST01` Read legacy Python backend (`app/*.py`, `Dockerfile`, `infra/docker-compose.yml`, React `ui/src/*`)
- [x] `ST02` Create/update `.agents/skills.json`, `.docs/CONSTRAINTS.md`, `.docs/plan.md`, `.docs/task.md`
- [x] `ST03` Wave 0: `server-rs` + `crypto.rs` + `config.rs` (migrate `keys.json`, scrub plaintext `raw_key`)
- [x] `ST04` Wave 0: `store.rs` (Refero styles, collections, templates, components, search, reindex parity)
- [x] `ST05` Wave 1A: `oidc.rs`, `mcp.rs` (8 tools), `server.rs`, `main.rs`, `Dockerfile*`, `infra/docker-compose.yml`
- [x] `ST06` Wave 1B: React `ui/src/*` OIDC + Zero-F12-Leak updates & `corepack pnpm build` + `tests/e2e/e2e_scorecard.py`
- [x] `ST07` Verification: `cargo clippy --all-targets -- -D warnings` (0 warnings), `cargo test` (11/11 pass), `cargo build --release` (`6,929,920` bytes), `e2e_scorecard.py` (**`100.0/100`**, Hard-Fail `CLEAR`), `.docs/PROJECT_STATUS.md`
