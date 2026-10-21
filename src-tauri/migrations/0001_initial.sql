-- Initial schema for the housekeeping app.
-- Business rules and field semantics live in PLAN.md §3–§5.

CREATE TABLE staff (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    birth_date TEXT,
    phone TEXT,
    national_id TEXT NOT NULL UNIQUE,
    address TEXT,
    photo_path TEXT,
    gender TEXT NOT NULL CHECK (gender IN ('male', 'female')),
    role TEXT NOT NULL CHECK (role IN ('maid', 'window_cleaner', 'common_areas')),
    is_seasonal INTEGER NOT NULL DEFAULT 0 CHECK (is_seasonal IN (0, 1)),
    season_start_date TEXT,
    season_end_date TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (
        (is_seasonal = 0 AND season_start_date IS NULL AND season_end_date IS NULL)
        OR (
            is_seasonal = 1
            AND (
                (season_start_date IS NULL AND season_end_date IS NULL)
                OR (
                    season_start_date IS NOT NULL
                    AND season_end_date IS NOT NULL
                    AND season_start_date <= season_end_date
                )
            )
        )
    )
);

CREATE TABLE staff_rest_day (
    staff_id INTEGER NOT NULL REFERENCES staff(id) ON DELETE CASCADE,
    weekday INTEGER NOT NULL CHECK (weekday BETWEEN 0 AND 6),
    PRIMARY KEY (staff_id, weekday)
);

CREATE TABLE absence (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    staff_id INTEGER NOT NULL REFERENCES staff(id) ON DELETE CASCADE,
    type TEXT NOT NULL CHECK (type IN ('medical_certificate', 'leave', 'absence')),
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (start_date <= end_date)
);

CREATE INDEX idx_absence_staff_dates ON absence (staff_id, start_date, end_date);

CREATE TABLE place (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('apartment', 'common_area')),
    status TEXT CHECK (
        status IS NULL
        OR status IN ('occupied', 'upcoming_occupancy', 'vacant', 'recently_vacated', 'cancelled')
    ),
    location TEXT,
    check_in_date TEXT,
    check_out_date TEXT,
    is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0, 1)),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (
        check_in_date IS NULL
        OR check_out_date IS NULL
        OR check_in_date <= check_out_date
    ),
    CHECK (
        type = 'apartment'
        OR (status IS NULL AND check_in_date IS NULL AND check_out_date IS NULL)
    )
);

CREATE UNIQUE INDEX idx_place_name_unique ON place (name COLLATE NOCASE);

CREATE TABLE staff_place (
    staff_id INTEGER NOT NULL REFERENCES staff(id) ON DELETE CASCADE,
    place_id INTEGER NOT NULL REFERENCES place(id) ON DELETE CASCADE,
    PRIMARY KEY (staff_id, place_id)
);

CREATE INDEX idx_staff_place_place ON staff_place (place_id);

CREATE TABLE cleaning_service (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    place_id INTEGER NOT NULL REFERENCES place(id) ON DELETE RESTRICT,
    date TEXT NOT NULL,
    start_time TEXT,
    end_time TEXT,
    service_type TEXT NOT NULL CHECK (
        service_type IN ('glass', 'daily_cleaning', 'deep_cleaning', 'touch_up')
    ),
    status TEXT NOT NULL DEFAULT 'planned' CHECK (
        status IN ('planned', 'in_progress', 'completed', 'cancelled')
    ),
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    CHECK (start_time IS NULL OR end_time IS NULL OR start_time < end_time)
);

CREATE INDEX idx_cleaning_service_date ON cleaning_service (date);
CREATE INDEX idx_cleaning_service_place_date ON cleaning_service (place_id, date);

CREATE TABLE service_staff (
    service_id INTEGER NOT NULL REFERENCES cleaning_service(id) ON DELETE CASCADE,
    staff_id INTEGER NOT NULL REFERENCES staff(id) ON DELETE RESTRICT,
    is_lead INTEGER NOT NULL DEFAULT 0 CHECK (is_lead IN (0, 1)),
    PRIMARY KEY (service_id, staff_id)
);

CREATE UNIQUE INDEX idx_service_staff_one_lead
    ON service_staff (service_id)
    WHERE is_lead = 1;

CREATE INDEX idx_service_staff_staff ON service_staff (staff_id);

CREATE TRIGGER trg_service_staff_keep_last
BEFORE DELETE ON service_staff
WHEN (SELECT COUNT(*) FROM service_staff WHERE service_id = OLD.service_id) <= 1
BEGIN
    SELECT RAISE(ABORT, 'Un servicio debe tener al menos un funcionario asignado.');
END;

CREATE TABLE incident (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id INTEGER NOT NULL REFERENCES cleaning_service(id) ON DELETE RESTRICT,
    type TEXT NOT NULL CHECK (type IN ('damage', 'lost_item', 'supply_shortage')),
    description TEXT NOT NULL,
    photo_path TEXT,
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'resolved')),
    date TEXT NOT NULL,
    resolved_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_incident_service ON incident (service_id);
CREATE INDEX idx_incident_status ON incident (status);

CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO settings (key, value) VALUES
    ('recent_vacated_days', '2'),
    ('upcoming_occupancy_days', '2');
