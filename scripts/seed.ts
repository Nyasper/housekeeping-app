#!/usr/bin/env bun
/**
 * Seeds the local Housekeeping database with the building layout and a demo
 * team. The app stores its data in Tauri's `app_data_dir()`:
 *
 *   Windows: %APPDATA%\housekeeping-app\housekeeping.db
 *   macOS:   ~/Library/Application Support/housekeeping-app/housekeeping.db
 *   Linux:   $XDG_DATA_HOME/housekeeping-app/housekeeping.db
 *
 * Usage:
 *   bun run seed
 *   bun run seed -- --db "C:\path\to\housekeeping.db"
 *   HOUSEKEEPING_DB_PATH=... bun run seed
 *
 * The script is idempotent: places are matched by their unique name and staff
 * by their unique national id, so running it again never duplicates rows or
 * overwrites data edited in the app.
 */

import { Database } from 'bun:sqlite';
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';

const APP_IDENTIFIER = 'housekeeping-app';
const DATABASE_FILE_NAME = 'housekeeping.db';
const MIGRATIONS_DIR = join(import.meta.dir, '..', 'src-tauri', 'migrations');

const FLOOR_COUNT = 7;
const TOWERS = [
  { code: 'A', blocks: 3 },
  { code: 'B', blocks: 2 },
  { code: 'C', blocks: 3 }
] as const;

const COMMON_AREAS = [
  'Cine',
  'Gimnasio',
  'Sauna',
  'Sala de masajes',
  'Piscina interior',
  'Hall A',
  'Hall B',
  'Hall C'
] as const;

type Gender = 'male' | 'female';
type StaffRole = 'maid' | 'window_cleaner' | 'common_areas';

interface ApartmentSeed {
  name: string;
  location: string;
}

interface StaySeed {
  checkInOffset: number;
  checkOutOffset: number;
}

interface StaffSeed {
  firstName: string;
  lastName: string;
  birthDate: string;
  phone: string;
  nationalId: string;
  address: string;
  gender: Gender;
  role: StaffRole;
  isSeasonal: boolean;
  seasonStartDate: string | null;
  seasonEndDate: string | null;
  notes: string | null;
  restDays: number[];
  mainPlaces: string[];
}

/**
 * A few apartments get demo stay dates so the daily board shows every
 * recommendation branch (occupied, recently vacated and upcoming occupancy).
 * Offsets are relative to the day the script runs. Only applied on first seed.
 */
const DEMO_STAYS: Record<string, StaySeed> = {
  A201: { checkInOffset: -3, checkOutOffset: 2 },
  B402: { checkInOffset: -1, checkOutOffset: 5 },
  C603: { checkInOffset: -2, checkOutOffset: 1 },
  A203: { checkInOffset: -6, checkOutOffset: -1 },
  B302: { checkInOffset: -4, checkOutOffset: -1 },
  C502: { checkInOffset: -5, checkOutOffset: -1 },
  A702: { checkInOffset: 2, checkOutOffset: 6 },
  B102: { checkInOffset: 1, checkOutOffset: 4 },
  C303: { checkInOffset: 2, checkOutOffset: 5 }
};

const STAFF: StaffSeed[] = [
  {
    firstName: 'Lucía',
    lastName: 'Fernández',
    birthDate: '1990-05-14',
    phone: '099 123 456',
    nationalId: '41234567',
    address: 'Av. 18 de Julio 1234',
    gender: 'female',
    role: 'maid',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Referente de Torre A.',
    restDays: [6],
    mainPlaces: ['A101', 'A102']
  },
  {
    firstName: 'Marta',
    lastName: 'Rodríguez',
    birthDate: '1985-11-02',
    phone: '098 234 567',
    nationalId: '39876542',
    address: 'Bulevar España 2345',
    gender: 'female',
    role: 'maid',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Equipo de Torre A.',
    restDays: [0, 6],
    mainPlaces: ['A201', 'A202']
  },
  {
    firstName: 'Sofía',
    lastName: 'Martínez',
    birthDate: '1995-03-21',
    phone: '097 345 678',
    nationalId: '45678912',
    address: 'Miguelete 1567',
    gender: 'female',
    role: 'maid',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Equipo de Torre A.',
    restDays: [3],
    mainPlaces: ['A301', 'A302', 'A303']
  },
  {
    firstName: 'Valentina',
    lastName: 'López',
    birthDate: '1988-07-09',
    phone: '096 456 789',
    nationalId: '38765432',
    address: 'Rivera 2890',
    gender: 'female',
    role: 'maid',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Equipo de Torre B.',
    restDays: [1],
    mainPlaces: ['B101', 'B102', 'B201']
  },
  {
    firstName: 'Camila',
    lastName: 'Pereira',
    birthDate: '1993-01-30',
    phone: '095 567 890',
    nationalId: '42345678',
    address: 'Colonia 1122',
    gender: 'female',
    role: 'maid',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Equipo de Torre B.',
    restDays: [2],
    mainPlaces: ['B202', 'B301', 'B302']
  },
  {
    firstName: 'Daniela',
    lastName: 'Silva',
    birthDate: '1992-09-17',
    phone: '094 678 901',
    nationalId: '40123456',
    address: 'Canelones 1980',
    gender: 'female',
    role: 'maid',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Equipo de Torre C.',
    restDays: [4],
    mainPlaces: ['C101', 'C102', 'C103']
  },
  {
    firstName: 'Gabriela',
    lastName: 'Núñez',
    birthDate: '1997-12-05',
    phone: '093 789 012',
    nationalId: '44567890',
    address: 'San José 1456',
    gender: 'female',
    role: 'maid',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Equipo de Torre C.',
    restDays: [5],
    mainPlaces: ['C201', 'C202', 'C203']
  },
  {
    firstName: 'Florencia',
    lastName: 'Acosta',
    birthDate: '1983-04-11',
    phone: '092 890 123',
    nationalId: '37890123',
    address: 'Soriano 1234',
    gender: 'female',
    role: 'maid',
    isSeasonal: true,
    seasonStartDate: '2026-11-01',
    seasonEndDate: '2027-03-31',
    notes: 'Refuerzo de temporada.',
    restDays: [6],
    mainPlaces: ['A401', 'A402']
  },
  {
    firstName: 'Nicolás',
    lastName: 'Cabrera',
    birthDate: '1986-08-23',
    phone: '091 901 234',
    nationalId: '46789012',
    address: 'Av. Italia 3456',
    gender: 'male',
    role: 'window_cleaner',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Vidrios de torres y áreas comunes.',
    restDays: [6],
    mainPlaces: ['A501', 'B401']
  },
  {
    firstName: 'Sebastián',
    lastName: 'Viera',
    birthDate: '1991-02-14',
    phone: '099 012 345',
    nationalId: '43456789',
    address: 'Propios 2211',
    gender: 'male',
    role: 'window_cleaner',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Vidrios de torres y áreas comunes.',
    restDays: [3],
    mainPlaces: ['C301', 'C302']
  },
  {
    firstName: 'Patricia',
    lastName: 'Moreira',
    birthDate: '1979-06-19',
    phone: '098 112 233',
    nationalId: '41230987',
    address: 'Durazno 1678',
    gender: 'female',
    role: 'common_areas',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Mantenimiento de áreas comunes.',
    restDays: [0],
    mainPlaces: []
  },
  {
    firstName: 'Andrés',
    lastName: 'Suárez',
    birthDate: '1981-10-27',
    phone: '097 223 344',
    nationalId: '39870123',
    address: 'Yaguarón 1890',
    gender: 'male',
    role: 'common_areas',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Mantenimiento de áreas comunes.',
    restDays: [4],
    mainPlaces: []
  },
  {
    firstName: 'Romina',
    lastName: 'Cardozo',
    birthDate: '1999-08-08',
    phone: '096 334 455',
    nationalId: '44561234',
    address: 'Tacuarembó 1456',
    gender: 'female',
    role: 'maid',
    isSeasonal: true,
    seasonStartDate: '2026-12-01',
    seasonEndDate: '2027-02-28',
    notes: 'Refuerzo de temporada.',
    restDays: [2],
    mainPlaces: ['B501', 'B502']
  },
  {
    firstName: 'Alejandra',
    lastName: 'Bentancor',
    birthDate: '1977-05-03',
    phone: '095 445 566',
    nationalId: '40987654',
    address: 'Gaboto 1234',
    gender: 'female',
    role: 'common_areas',
    isSeasonal: false,
    seasonStartDate: null,
    seasonEndDate: null,
    notes: 'Mantenimiento de áreas comunes.',
    restDays: [5],
    mainPlaces: []
  }
];

function defaultDatabasePath(): string {
  const override = process.env.HOUSEKEEPING_DB_PATH?.trim();
  if (override) {
    return resolve(override);
  }

  const home = homedir();
  let dataDirectory: string;
  if (process.platform === 'win32') {
    dataDirectory = process.env.APPDATA ?? join(home, 'AppData', 'Roaming');
  } else if (process.platform === 'darwin') {
    dataDirectory = join(home, 'Library', 'Application Support');
  } else {
    dataDirectory = process.env.XDG_DATA_HOME ?? join(home, '.local', 'share');
  }

  return join(dataDirectory, APP_IDENTIFIER, DATABASE_FILE_NAME);
}

function parseDatabasePath(argv: string[]): string {
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--db') {
      const value = argv[index + 1];
      if (!value) {
        throw new Error('Falta la ruta después de --db.');
      }
      return resolve(value);
    }
    if (argument.startsWith('--db=')) {
      const value = argument.slice('--db='.length).trim();
      if (!value) {
        throw new Error('Falta la ruta después de --db=.');
      }
      return resolve(value);
    }
    if (argument === '--help' || argument === '-h') {
      return '';
    }
    throw new Error(`Argumento desconocido: ${argument}`);
  }

  return defaultDatabasePath();
}

function printUsage(): void {
  console.log(`Uso: bun run seed [--db <ruta>]

Sin argumentos usa la base de datos local de la app:
  ${defaultDatabasePath()}

También podés definir HOUSEKEEPING_DB_PATH.
`);
}

function isoDateFromToday(offsetDays: number): string {
  const date = new Date();
  date.setHours(12, 0, 0, 0);
  date.setDate(date.getDate() + offsetDays);

  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/** `{tower}{floor}0{block}`, e.g. B601 for tower B, floor 6, block 1. */
export function apartmentName(tower: string, floor: number, block: number): string {
  return `${tower}${floor}0${block}`;
}

export function buildApartments(): ApartmentSeed[] {
  const apartments: ApartmentSeed[] = [];
  for (const tower of TOWERS) {
    for (let floor = 1; floor <= FLOOR_COUNT; floor += 1) {
      for (let block = 1; block <= tower.blocks; block += 1) {
        apartments.push({
          name: apartmentName(tower.code, floor, block),
          location: `Piso ${floor}`
        });
      }
    }
  }
  return apartments;
}

interface MigrationFile {
  version: number;
  description: string;
  path: string;
}

function listMigrations(): MigrationFile[] {
  return readdirSync(MIGRATIONS_DIR)
    .filter((file) => file.endsWith('.sql'))
    .map((file) => {
      const match = /^(\d+)_(.+)\.sql$/.exec(file);
      if (!match) {
        throw new Error(`Nombre de migración inválido: ${file}`);
      }
      return {
        version: Number(match[1]),
        description: match[2],
        path: join(MIGRATIONS_DIR, file)
      };
    })
    .sort((left, right) => left.version - right.version);
}

/**
 * Creates the schema when the database is empty. SQLx stores a SHA-384
 * checksum for every applied migration, so the script registers the same
 * version/checksum rows; the app can then open the database normally.
 */
function ensureSchema(database: Database): { created: boolean; applied: number } {
  const tableNames = database
    .query<{ name: string }, []>(
      "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
    )
    .all()
    .map((row) => row.name);

  const applicationTables = tableNames.filter((name) => name !== '_sqlx_migrations');
  if (applicationTables.length > 0) {
    return { created: false, applied: 0 };
  }

  database.exec(`
    CREATE TABLE IF NOT EXISTS _sqlx_migrations (
      version BIGINT PRIMARY KEY,
      description TEXT NOT NULL,
      installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
      success BOOLEAN NOT NULL,
      checksum BLOB NOT NULL,
      execution_time BIGINT NOT NULL
    );
  `);

  const appliedVersions = new Set(
    database
      .query<{ version: number }, []>(
        'SELECT version FROM _sqlx_migrations WHERE success = 1'
      )
      .all()
      .map((row) => Number(row.version))
  );

  let applied = 0;
  for (const migration of listMigrations()) {
    if (appliedVersions.has(migration.version)) {
      continue;
    }

    const bytes = readFileSync(migration.path);
    if (bytes.includes(0x0d)) {
      throw new Error(
        `La migración ${migration.path} tiene finales de línea CRLF. Ejecutá ` +
          '`git checkout -- src-tauri/migrations` antes de sembrar.'
      );
    }

    const checksum = createHash('sha384').update(bytes).digest();
    database.transaction(() => {
      database.exec(bytes.toString('utf8'));
      database
        .query(
          `INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
           VALUES (?, ?, 1, ?, 0)`
        )
        .run(migration.version, migration.description, new Uint8Array(checksum));
    })();
    applied += 1;
  }

  return { created: true, applied };
}

interface SeedReport {
  newPlaces: number;
  newStaff: number;
  newMainPlaces: number;
  newRestDays: number;
}

function seedData(database: Database): SeedReport {
  const apartments = buildApartments();

  const insertApartment = database.query(
    `INSERT OR IGNORE INTO place
       (name, type, status, location, check_in_date, check_out_date, is_active)
     VALUES (?, 'apartment', NULL, ?, ?, ?, 1)`
  );
  const insertCommonArea = database.query(
    `INSERT OR IGNORE INTO place (name, type, is_active)
     VALUES (?, 'common_area', 1)`
  );
  const insertStaff = database.query(
    `INSERT OR IGNORE INTO staff
       (first_name, last_name, birth_date, phone, national_id, address, gender, role,
        is_seasonal, season_start_date, season_end_date, is_active, notes)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?)`
  );
  const findPlace = database.query<{ id: number }, [string]>(
    `SELECT id FROM place
     WHERE name = ? COLLATE NOCASE AND type = 'apartment' AND is_active = 1`
  );
  const findStaff = database.query<{ id: number }, [string]>(
    'SELECT id FROM staff WHERE national_id = ?'
  );
  const insertMainPlace = database.query(
    'INSERT OR IGNORE INTO staff_place (staff_id, place_id) VALUES (?, ?)'
  );
  const insertRestDay = database.query(
    'INSERT OR IGNORE INTO staff_rest_day (staff_id, weekday) VALUES (?, ?)'
  );

  const report: SeedReport = {
    newPlaces: 0,
    newStaff: 0,
    newMainPlaces: 0,
    newRestDays: 0
  };

  const run = database.transaction(() => {
    for (const apartment of apartments) {
      const stay = DEMO_STAYS[apartment.name];
      const result = stay
        ? insertApartment.run(
            apartment.name,
            apartment.location,
            isoDateFromToday(stay.checkInOffset),
            isoDateFromToday(stay.checkOutOffset)
          )
        : insertApartment.run(apartment.name, apartment.location, null, null);
      report.newPlaces += result.changes;
    }

    for (const name of COMMON_AREAS) {
      report.newPlaces += insertCommonArea.run(name).changes;
    }

    for (const staff of STAFF) {
      const result = insertStaff.run(
        staff.firstName,
        staff.lastName,
        staff.birthDate,
        staff.phone,
        staff.nationalId,
        staff.address,
        staff.gender,
        staff.role,
        staff.isSeasonal ? 1 : 0,
        staff.seasonStartDate,
        staff.seasonEndDate,
        staff.notes
      );

      if (result.changes === 0) {
        continue;
      }

      report.newStaff += 1;
      const staffRow = findStaff.get(staff.nationalId);
      if (!staffRow) {
        continue;
      }

      for (const placeName of staff.mainPlaces) {
        const placeRow = findPlace.get(placeName);
        if (!placeRow) {
          console.warn(
            `Aviso: ${placeName} no está disponible como apartamento activo; ` +
              `se omite para ${staff.firstName} ${staff.lastName}.`
          );
          continue;
        }
        report.newMainPlaces += insertMainPlace.run(staffRow.id, placeRow.id).changes;
      }

      for (const weekday of staff.restDays) {
        report.newRestDays += insertRestDay.run(staffRow.id, weekday).changes;
      }
    }
  });

  run();
  return report;
}

function main(): void {
  const argv = process.argv.slice(2);
  const databasePath = parseDatabasePath(argv);
  if (!databasePath) {
    printUsage();
    return;
  }

  mkdirSync(dirname(databasePath), { recursive: true });

  const database = new Database(databasePath, { create: true, strict: true });
  try {
    database.exec('PRAGMA busy_timeout = 10000;');
    database.exec('PRAGMA foreign_keys = ON;');
    database.exec('PRAGMA journal_mode = WAL;');

    const schema = ensureSchema(database);
    const report = seedData(database);

    console.log(`Base de datos: ${databasePath}`);
    if (schema.created) {
      console.log(`Esquema: creado (${schema.applied} migración(es) aplicada(s)).`);
    } else {
      console.log('Esquema: existente.');
    }
    console.log(
      `Lugares: +${report.newPlaces} (${buildApartments().length} apartamentos, ` +
        `${COMMON_AREAS.length} áreas comunes).`
    );
    console.log(`Personal: +${report.newStaff} (${STAFF.length} registros de ejemplo).`);
    console.log(
      `Apartamentos principales: +${report.newMainPlaces}. ` +
        `Días de descanso: +${report.newRestDays}.`
    );
    console.log('Listo. Si la app está abierta, recargá la pantalla para ver los cambios.');
  } finally {
    database.close();
  }
}

if (import.meta.main) {
  try {
    main();
  } catch (error) {
    console.error(`Error al sembrar la base de datos: ${error instanceof Error ? error.message : String(error)}`);
    process.exitCode = 1;
  }
}
