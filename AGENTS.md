# AGENTS.md — Working guide for agents

> Operational guide for agents working on this project. **Business rules, data model, functional scope, and any domain question live in `PLAN.md`; they are not duplicated here.** If there is a conflict, `PLAN.md` wins.

---

## 1. Sources of truth

| File | Location | Purpose |
|------|----------|---------|
| `PLAN.md` | Repo root (`G:\Documents\Programming\Tauri\housekeeping-app\PLAN.md`) | **Source of truth:** scope, functional specs, data model, business rules, architecture, and roadmap. |
| `AGENTS.md` | Repo root | Operational and technical guide for agents (this file). |
| `CHANGELOG.md` | Repo root | Record of **post-MVP** changes, in English. |
| `README.md` | Repo root | General project information. |

**Working rules:**
1. Read `PLAN.md` before implementing. For domain, see especially `PLAN.md` §3–§5.
2. Do not deviate from the plan without updating it. If a rule changes, update `PLAN.md` and `AGENTS.md` in the same change.
3. Do not duplicate domain rules in this file: add a reference to `PLAN.md`.
4. Do not invent requirements outside scope without asking.

---

## 2. Current state

- Tauri v2 + SvelteKit 3 project migrated. **Phase 0 (foundation) is complete** on the `feature` branch.
- **MVP not implemented yet**; next step is **Phase 1 — Personal** (`PLAN.md` §8).
- `MIGRATION_TASKS.md` was deleted: the SvelteKit 3 migration is closed.
- Repository initialized with `1dc2a51 initial_setup`; Phase 0 is committed separately.
- Current backend: `db.rs`, `state.rs`, `error.rs`, `settings.rs`, migration `0001_initial.sql`; commands `settings_get`, `settings_update`, `take_startup_notice`.
- Current frontend: AppShell/Sidebar with light/dark theme, daily-board placeholder, Ajustes form backed by `settingsService`; `labels.ts`, `errors.ts`, `types.ts`, `format.ts` and design tokens in place.
- Data lives in `app_data_dir()/housekeeping.db` with photos under `app_data_dir()/photos/`.

---

## 3. Tech stack

| Layer | Technology | Version / note |
|-------|------------|----------------|
| Runtime / package manager | **Bun** | `1.4.3` |
| SvelteKit 3 requirement | Node | `>= 22.17` (the project runs on Bun) |
| Frontend | **Svelte 5** | `5.57.2` |
| Framework | **SvelteKit 3** | `3.1.0`, static adapter (SPA) |
| Bundler | **Vite 8** | `8.3.5` |
| Svelte plugin | `@sveltejs/vite-plugin-svelte` | `7.3.1` |
| Adapter | `@sveltejs/adapter-static` | `4.0.0`, `fallback: "index.html"` |
| Frontend language | TypeScript | `6.0.3` |
| Backend | **Rust / Tauri v2** | edition 2024 |
| Database | **SQLite + SQLx 0.8** | `runtime-tokio`, `sqlite`, `migrate`, `chrono` |
| Styling | Native CSS | CSS variables / design tokens |
| Dialogs | `tauri-plugin-dialog` | v2 |
| Files | `tauri-plugin-fs` | v2 |
| Single instance | `tauri-plugin-single-instance` | v2 |
| Open links | `tauri-plugin-opener` | v2 (already installed) |
| CSV | `csv` | `1.4` |
| XLSX | `rust_xlsxwriter` | `0.101` |
| PDF | `printpdf` | `0.12` (`DejaVuSans` font embedded) |
| Backup ZIP | `zip` | `9` |
| Photo traversal | `walkdir` | `2.5` |
| Backend tests | `tempfile` + `tokio` | dev-dependencies |

### 3.1 SvelteKit 3 configuration

- **There is no `svelte.config.js`.** Configuration lives in `vite.config.js`, inside `sveltekit({ ... })`.
- `adapter-static` with `fallback: "index.html"` for SPA mode.
- `src/routes/+layout.ts` exports `ssr = false`.
- `#lib` alias declared in `package.json` (`imports`), not `$lib`.
- When importing from `#lib`, use the `.js` extension (e.g. `#lib/api/types.js`).
- Modern Svelte 5: runes, snippets, and attachments.

---

## 4. Repository structure

### 4.1 Root

```
housekeeping-app/
  AGENTS.md
  CHANGELOG.md
  PLAN.md
  README.md
  package.json
  tsconfig.json
  vite.config.js
  src/
  src-tauri/
  static/
```

### 4.2 Planned backend (`src-tauri/`)

```
src-tauri/
  assets/
    fonts/DejaVuSans.ttf      # PDF with accents/ñ
  migrations/
    0001_initial.sql
  src/
    main.rs
    lib.rs                    # Builder, plugins, setup, commands
    error.rs                  # CommandError + SQLx mapping
    db.rs                     # connect, open_with_recovery, db_path, quarantine
    state.rs                  # DbState(RwLock<Pool<Sqlite>>), StartupNotice
    settings.rs               # get_settings / save_settings (typed)
    backup.rs                 # ZIP + validation + restore + pool swap
    domain/
      staff/                  # model.rs, repo.rs, service.rs, commands.rs
      places/
      cleaning/
      incidents/
      reports/                # queries.rs, export_csv.rs, export_xlsx.rs, export_pdf.rs
    util/
      dates.rs
      availability.rs
      files.rs
```

### 4.3 Planned frontend (`src/`)

```
src/
  app.html                    # lang="es", theme script (FOUC)
  routes/
    +layout.svelte            # AppShell, navigation, theme toggle
    +layout.ts
    +page.svelte              # Daily board
    staff/
    places/
    cleaning/
    incidents/
    reports/
    settings/
  lib/
    api/
      types.ts                # TS DTOs (mirror of Rust, in English)
      invoke.ts
      errors.ts               # CommandError -> Spanish message
      labels.ts               # enum codes -> Spanish labels
    utils/
      format.ts               # dates/numbers in es-UY
    services/                 # classes, logic separated from UI
      staffService.svelte.ts
      placesService.svelte.ts
      cleaningService.svelte.ts
      incidentsService.svelte.ts
      reportsService.svelte.ts
      exportService.svelte.ts
      backupService.svelte.ts
      settingsService.svelte.ts
      themeService.svelte.ts
      dateService.svelte.ts
    components/
      AppShell.svelte
      Sidebar.svelte
      ThemeToggle.svelte
      Avatar.svelte
      StaffForm.svelte
      PlaceForm.svelte
      ServiceForm.svelte
      DailyBoard.svelte
      AvailabilityList.svelte
      IncidentForm.svelte
      ConfirmDialog.svelte
      Toast.svelte
      EmptyState.svelte
      FormField.svelte
    styles/tokens.css
static/
  avatars/default-male.svg
  avatars/default-female.svg
```

---

## 5. Technical architecture

### 5.1 Backend

- **Domain separation:** each domain has `model.rs` (structs/serde), `repo.rs` (SQL), `service.rs` (rules/validation), and `commands.rs` (`#[tauri::command]`).
- **Shared infrastructure:** `db.rs`, `error.rs`, `state.rs`, `settings.rs`, `backup.rs`, `util/`.
- **Do not write SQL in commands.** Commands delegate to `service`, which uses `repo`.
- **Transactions** for operations spanning multiple tables.
- **Patterns adopted from `tauri-inventory-manager/src-tauri/src/inventory.rs`:**
  - `DbState(RwLock<SqlitePool>)` with `pool()` so it can be closed/reopened on restore.
  - `connect(path)` creates the parent directory; `SqlitePoolOptions::new().max_connections(1)`; `SqliteConnectOptions` with `create_if_missing`, `foreign_keys(true)`, `busy_timeout(10s)`, `journal_mode(Wal)`, `synchronous(Normal)`, `temp_store=2`, `cache_size=-8000`.
  - `CommandError { code: &'static str, message: String }`, serializable; messages in Spanish.
  - `open_with_recovery` + `quarantine_database` + `StartupNotice` + `take_startup_notice`.
  - Backup with `VACUUM INTO`, validation, safety copy, and pool swap.
  - `settings` key/value with typed `get_settings` and `save_settings` using upsert.
  - `tauri-plugin-single-instance`.
- **Data path:** DB at `app_data_dir()/housekeeping.db`; photos at `app_data_dir()/photos/`.
- **Migrations:** `sqlx::migrate!("./migrations").run(&pool)` in `setup`.
- **Commands:** `{domain}_{action}` in English (snake_case).

> **Data model, entities, fields, constraints, and business rules:** `PLAN.md` §4–§5. Do not duplicate here.

### 5.2 Frontend

- **Logic outside components:** each domain has a **class** in `{name}Service.svelte.ts` with runes state (`$state`, `$derived`) and methods that call `invoke`.
- Components only render and call service methods.
- **Do not create giant files:** split into small components and per-domain services.
- `labels.ts` centralizes enum labels in Spanish; `errors.ts` translates `CommandError`.
- `themeService` handles light/dark theme; `settingsService` is the only service that interacts with the `settings` table.
- `exportService` and `backupService` open dialogs and call Rust.
- `dateService` holds the board's active date.
- Date formatting: `Intl.DateTimeFormat('es-UY', ...)`.
- Theme: `[data-theme="light"]` / `[data-theme="dark"]` on `<html>`, persisted in `localStorage` (`housekeeping-theme`).
- Minimal script in `app.html` to avoid FOUC.

> **API contracts, commands, and UI/domain rules:** `PLAN.md` §6.4 and §3. Do not duplicate here.

### 5.3 Tauri

- Plugins: `single-instance`, `dialog`, `fs`, `opener`.
- Capabilities: `core:default`, `opener:default`, `dialog:default`, `fs:default`.
- `assetProtocol` enabled with scope `["$APPDATA/photos/**"]`.
- DB and photos under `app_data_dir()`.
- `app.html` with `lang="es"` and theme script.
- Do not commit `target/`, `build/`, `.svelte-kit/`, `node_modules/`.

### 5.4 Quality

- **Rust:** `cargo fmt` and `cargo clippy -- -D warnings`; tests with `cargo test`.
- **Frontend:** `bun run check` (svelte-check); `prettier` + `prettier-plugin-svelte`; ESLint with `eslint-plugin-svelte` (or Biome).
- **Scripts:** `check`, `lint`, `format`, `test` in `package.json`.
- **Git:** automatic commit for each feature/big change with an English message; post-MVP `CHANGELOG.md` (section 9).

---

## 6. Technical and code rules

### 6.1 Language and naming

- **English identifiers:** tables, columns, types, properties, functions, files, routes, modules.
- **Spanish UI.**
- Command names: `{domain}_{action}` in snake_case.
- No cryptic abbreviations; use descriptive names.

### 6.2 Rust

- Run `cargo fmt` and `cargo clippy -- -D warnings` before committing.
- Use `CommandError` for command errors; messages in Spanish; stable codes.
- Do not use `unwrap()`/`expect()` in production code except for controlled startup.
- Validate input in `service.rs`; do not trust the frontend.
- Use transactions for multi-table writes.
- Do not write SQL in `commands.rs`.
- Versioned migrations in `migrations/`; do not modify migrations that have already been applied.
- Unit and integration tests; cover edge cases.

### 6.3 Frontend

- Modern Svelte 5: `$state`, `$derived`, `$effect`, `$props`, `$bindable`, `{#snippet}`, `{@render}`, `{@attach}`.
- Use `.svelte.ts` for services with runes; **classes**, not closures.
- Do not put business logic in components; put it in services.
- Do not create giant files; split components/services.
- Do not use `$lib`; use `#lib` with the `.js` extension.
- Do not create `svelte.config.js`; configuration belongs in `vite.config.js`.
- Do not use `window.print()` or print CSS for PDF.
- UI must be modern, fast, and intuitive; see `PLAN.md` §3.11. Use the `frontend-design` and `accessibility` skills for UI work.
- Keep `ssr = false` and the static adapter.
- Basic accessibility: labels, focus, keyboard, contrast.

### 6.4 Database

- SQLite with WAL, foreign keys, and `max_connections(1)`.
- Versioned migrations; do not modify applied migrations.
- Do not copy the `.db` manually while WAL is active; use `VACUUM INTO`.
- Schema, indexes, constraints, and persistence rules: **`PLAN.md` §4**.

### 6.5 Domain rules

- **Do not duplicate them in this file.** See `PLAN.md`:
  - §3: functional specifications.
  - §4: data model.
  - §5: business rules.
  - §6.4: commands and contracts.

---

## 7. Resources and tools

### 7.1 Official documentation

- **Svelte 5:** https://svelte.dev/docs/svelte
- **SvelteKit 3:** https://svelte.dev/docs/kit
- **Migrating to SvelteKit 3:** https://svelte.dev/docs/kit/migrating-to-sveltekit-3
- **Tauri v2:** https://v2.tauri.app/
- **Tauri plugins:** https://v2.tauri.app/plugin/
- **SQLx:** https://docs.rs/sqlx
- **rust_xlsxwriter:** https://docs.rs/rust_xlsxwriter
- **printpdf:** https://docs.rs/printpdf
- **csv:** https://docs.rs/csv
- **zip:** https://docs.rs/zip
- **Bun:** https://bun.sh/docs
- **Vite:** https://vite.dev/
- **Keep a Changelog:** https://keepachangelog.com/en/1.1.0/
- **Semantic Versioning:** https://semver.org/
- **Conventional Commits:** https://www.conventionalcommits.org/

### 7.2 Available skills

Load the relevant skill before working in that area:

| Skill | When to use it |
|-------|----------------|
| `svelte-core-bestpractices` | Any Svelte task: writing/editing/analyzing components or modules. |
| `svelte5-best-practices` | Runes, snippets, SvelteKit, TypeScript, Svelte 4→5 migration, performance. |
| `tauri-v2` | Tauri configuration, Rust commands, IPC, capabilities, permissions, builds. |
| `bun` | Scripts, dependencies, bundling, tests. |
| `typescript-advanced-types` | Complex types, generics, type-safe utilities. |
| `vite` | Vite configuration, plugins, SSR. |
| `accessibility` | WCAG accessibility review/improvements. |
| `frontend-design` | Visual direction, typography, non-generic UI. |
| `web-perf` | Performance audits and Core Web Vitals. |

There is also the **`svelte-file-editor`** subagent: use it proactively to create/edit/review `.svelte` files, always with the Svelte MCP tools.

### 7.3 Svelte MCP

MCP server: **`svelte`**. Tools:

- `list-sections`: lists available documentation sections.
- `get-documentation`: retrieves official documentation by title or path.
- `svelte-autofixer`: analyzes and fixes Svelte components/modules.
- `playground-link`: generates a playground link with the code.

**Usage rules:**
1. Before editing a `.svelte` or `.svelte.ts` file, consult the relevant documentation via the MCP.
2. After editing, run `svelte-autofixer`.
3. Fix the issues and run it again until clean.
4. Pass the Svelte version from `package.json` (`5.57.2`).
5. Use `async: true` only if `compilerOptions.experimental.async` is enabled (it is not today).

---

## 8. Workflow

1. **Read** `PLAN.md` (domain) and the relevant roadmap phase.
2. **Choose** the next unit of work (feature/phase).
3. **Implement** modularly, following the architecture.
4. **Svelte:** use skills + MCP; run the autofixer after each edit.
5. **Verify:** `cargo test`, `cargo clippy`, `bun run check`, `bun run build` as applicable.
6. **Changelog:** post-MVP, add the entry to `CHANGELOG.md`.
7. **Commit:** automatically after each feature/big change, with an English message.
8. **Update** `PLAN.md`/`AGENTS.md` if a rule or decision changed.

### 8.1 Definition of Done

1. Complete, modular implementation.
2. Applicable tests/checks are green.
3. `CHANGELOG.md` updated (post-MVP only).
4. Commit in English with a descriptive message.

### 8.2 Quick commands

```bash
# Frontend
bun install
bun run dev
bun run build
bun run check
bun run tauri dev

# Backend
cd src-tauri
cargo test
cargo fmt
cargo clippy -- -D warnings
```

---

## 9. Commits and changelog

### 9.1 Commits

- **Initial commit** at the start of Phase 0: `chore: initialize SvelteKit 3 project and implementation plan`.
- After each **completed feature or big change**, commit automatically.
- Messages must be **short, descriptive, in English, imperative, and follow Conventional Commits**:
  - `feat: add database migrations and settings`
  - `feat: implement CRUD screens and forms`
  - `feat: add CSV, XLSX and PDF exports`
  - `feat: add daily board and theme toggle`
  - `fix: prevent invalid state transitions`
  - `chore: configure Tauri plugins and permissions`
  - `docs: update changelog for 0.1.1`
- One commit per logical unit; do not mix unrelated changes.
- Run checks before committing; do not commit artifacts.

### 9.2 Post-MVP changelog

- `CHANGELOG.md` is in English and follows **Keep a Changelog 1.1.0** + **SemVer**.
- Every user-facing change is recorded under `[Unreleased]` in `Added` / `Changed` / `Deprecated` / `Removed` / `Fixed` / `Security`.
- On release: move entries to `## [x.y.z] - YYYY-MM-DD`, create a new `[Unreleased]`, and update the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- The MVP is recorded as **`0.1.0`** when Phase 7 is complete.
- Internal changes (refactors/tests/tooling) may skip the changelog but are still committed.

---

## 10. Quick "do not" list (technical)

- Do not duplicate domain rules in `AGENTS.md`; reference `PLAN.md`.
- Do not use `$lib`; use `#lib` with `.js`.
- Do not create `svelte.config.js`.
- Do not copy the `.db` manually while WAL is active.
- Do not commit `target/`, `build/`, `.svelte-kit/`, `node_modules/`.
- Do not use `unwrap()`/`expect()` in production.
- Do not write SQL in `commands.rs`.
- Do not create giant files or put business logic in components.
- Do not change plan rules without updating `PLAN.md` and `AGENTS.md`.
