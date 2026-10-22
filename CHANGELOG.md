# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Places and cleaning screens now reload date-aware recommendations for the selected date instead of reusing the last daily-board date.
- Cleaning service forms now show inactive assigned staff so they can be removed or reassigned.
- Cancelled services can now be reopened from the edit form with explicit confirmation.
- Staff assignment warnings now only appear for overlapping services or when times are missing, instead of for every same-day assignment.
- Completed services can no longer be cancelled; they can only be reopened to a planned or in-progress state with explicit confirmation.
- CSV exports now use semicolon separators and comma decimals so they open correctly in Excel with Spanish regional settings.
- Editing a staff member no longer drops main-place associations to deactivated apartments; they are restored when the apartment is reactivated.
- Daily-sheet exports no longer count cancelled services in the service, completed, or minutes totals.
- Invalid ISO dates such as `2026-02-30` are no longer rolled over into the next month.
- Duration formatting no longer renders `60 min` for values close to one hour.
- Time formatting and duration calculations now reject out-of-range hours and minutes.
- `shiftIsoDate` keeps invalid input unchanged instead of silently jumping to today.

## [0.1.0] - 2026-10-22

### Added

- Local-first desktop foundation on Tauri v2 + Svelte 5 + SvelteKit 3 with SQLite WAL persistence, automatic startup recovery/quarantine notice, and single-instance window focus.
- Staff management (`/staff`): CRUD with soft deactivation/reactivation, Uruguayan national ID validation, roles (`maid`, `window_cleaner`, `common_areas`), gender-based default avatars, local photo upload, seasonal contracts (`is_seasonal`, start/end dates), weekly rest days, absences (`medical_certificate`, `vacation`, `unplanned`) with overlap prevention, and primary apartment assignments (`staff_place`).
- Places management (`/places`): CRUD for apartments and common areas, soft deactivation/reactivation, optional stay dates (`check_in_date`, `check_out_date`), manual status override including skipped turn (`cancelled`), automatic effective status calculation, and date-aware recommended cleaning task (`post_checkout`, `pre_checkin`, `daily_cleaning`, `touch_up`).
- Cleaning services (`/cleaning`): daily service planning and tracking (`planned`, `in_progress`, `completed`, `cancelled`), mandatory staff assignment (`>= 1` member, at most one `is_lead`) enforced in service and SQLite triggers, assisted staff assignment ranking (availability, main apartment, role match, daily workload), duration validation on completion, and confirmation when reopening completed services.
- Daily operational board (`/`): unified daily command center with date navigation, pending recommended apartment tasks with 1-click prefill into service creation, service status actions, staff availability overview, and daily open/resolved incidents.
- Incidents tracking (`/incidents`): CRUD and status transitions (`open`, `resolved`) for damages, lost items, and supply shortages linked to cleaning services, optional photo evidence with native modal preview, and filters by status, type, date, and service.
- Operational reports and exports (`/reports`): services and hours by staff member, average completion duration by service type, monthly absence days (split accurately across month boundaries), and daily operational sheet; native Rust exports to UTF-8 BOM CSV (`csv`), print-ready XLSX spreadsheets (`rust_xlsxwriter`), and paginated landscape A4 PDFs (`printpdf` with embedded `DejaVuSans` font).
- Global settings and ZIP backup/restore (`/settings`): configurable recommendation windows (`recent_vacated_days`, `upcoming_occupancy_days`) via `settingsService`, atomic ZIP backup (`VACUUM INTO` database snapshot + `photos/` + `manifest.json`), and verified ZIP restore with SQLite integrity/foreign-key checks, safety backup rollback, and live connection pool swap.
- Light and dark theme support with zero-FOUC initialization, `es-UY` Spanish interface and formatting, keyboard navigation, and WCAG 2.2 AA accessibility polish.

<!--
Post-MVP workflow:
1. Add every user-facing change under [Unreleased].
2. Use the Added / Changed / Deprecated / Removed / Fixed / Security sections.
3. On release, rename [Unreleased] to [x.y.z] - YYYY-MM-DD and create a new empty [Unreleased].
4. Add comparison links here once the repository remote is configured.
-->

<!--
Release template:

## [x.y.z] - YYYY-MM-DD

### Added
- ...

### Changed
- ...

### Deprecated
- ...

### Fixed
- ...

### Removed
- ...

### Security
- ...
-->
