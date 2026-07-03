use chrono::{Datelike, Local, NaiveDate};
#[cfg(debug_assertions)]
use chrono::Duration;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use uuid::Uuid;

pub struct Db {
    conn: Connection,
    device_id: String,
}

// ─── Wire types (flat, matches SQL schema) ────────────────────────────────────

/// Returned by load_habits — flat row with nullable fields per type.
#[derive(Serialize)]
pub struct HabitRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub habit_type: String,
    pub label: String,
    // counter
    pub target: Option<i64>,
    pub sets: Option<i64>,
    // timer
    pub target_seconds: Option<i64>,
    // counter-timer
    pub rounds: Option<i64>,
    pub seconds_per_round: Option<i64>,
    // scheduling
    pub start_date: Option<String>,
    pub repeat_type: String,
    pub repeat_days: Option<String>,
    pub repeat_every: Option<i64>,
    pub is_active_today: bool,
    // sync metadata — see tasks/14-multi-device-sync-schema.md in enhabitz-mobile
    pub updated_at: i64,
    pub device_id: String,
    // today's log — all nullable (no log yet = defaults on frontend)
    pub done: Option<bool>,
    pub count: Option<i64>,
    pub completed_sets: Option<i64>,
    pub seconds_elapsed: Option<i64>,
    pub current_round: Option<i64>,
    pub round_seconds_elapsed: Option<i64>,
}

/// Sent by frontend when creating or updating a habit definition.
#[derive(Deserialize)]
pub struct HabitData {
    #[serde(rename = "type")]
    pub habit_type: String,
    pub label: String,
    pub target: Option<i64>,
    pub sets: Option<i64>,
    pub target_seconds: Option<i64>,
    pub rounds: Option<i64>,
    pub seconds_per_round: Option<i64>,
    // scheduling
    pub start_date: Option<String>,
    pub repeat_type: String,
    pub repeat_days: Option<String>,
    pub repeat_every: Option<i64>,
}

/// One row returned by load_log_history — one completed habit per day.
#[derive(Serialize)]
pub struct DayEntry {
    pub date: String,
    pub label: String,
}

/// Sent by frontend after every progress mutation.
#[derive(Deserialize)]
pub struct LogData {
    pub habit_id: String,
    pub done: Option<bool>,
    pub count: Option<i64>,
    pub completed_sets: Option<i64>,
    pub seconds_elapsed: Option<i64>,
    pub current_round: Option<i64>,
    pub round_seconds_elapsed: Option<i64>,
}

// ─── Scheduling ───────────────────────────────────────────────────────────────

fn is_active_today(
    start_date: &Option<String>,
    repeat_type: &str,
    repeat_days: &Option<String>,
    repeat_every: &Option<i64>,
) -> bool {
    is_active_on(
        start_date,
        repeat_type,
        repeat_days,
        repeat_every,
        Local::now().date_naive(),
    )
}

/// Generalizes `is_active_today` to an arbitrary date — the seed data
/// generator needs to know which past days a habit was scheduled on, not
/// just today.
fn is_active_on(
    start_date: &Option<String>,
    repeat_type: &str,
    repeat_days: &Option<String>,
    repeat_every: &Option<i64>,
    date: NaiveDate,
) -> bool {
    let start = match start_date {
        None => return false, // draft / idea
        Some(s) => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return false,
        },
    };

    if date < start {
        return false; // hasn't started yet
    }

    match repeat_type {
        "daily" => true,
        "weekly" => {
            // repeat_days: JSON array of 0–6 where 0 = Sunday (matches JS Date.getDay())
            let weekday_num = date.weekday().num_days_from_sunday();
            let days: Vec<u32> = repeat_days
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            days.contains(&weekday_num)
        }
        "monthly" => {
            // repeat_days: JSON array of day-of-month numbers 1–31
            let day = date.day();
            let days: Vec<u32> = repeat_days
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            days.contains(&day)
        }
        "interval" => {
            // every N days from start_date
            let every = repeat_every.unwrap_or(1).max(1);
            let diff = (date - start).num_days();
            diff % every == 0
        }
        _ => true,
    }
}

fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// Reads `device_id.txt` from the app data dir, creating a fresh UUID v4 the
/// first time. Mirrors `DeviceIdentity` in enhabitz-mobile — a stable per-install
/// id written into every row this install touches (see
/// tasks/14-multi-device-sync-schema.md in enhabitz-mobile).
pub fn get_or_create_device_id(data_dir: &Path) -> std::io::Result<String> {
    let path = data_dir.join("device_id.txt");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }
    }
    let id = Uuid::new_v4().to_string();
    std::fs::write(&path, &id)?;
    Ok(id)
}

// ─── Db impl ──────────────────────────────────────────────────────────────────

/// Schema version produced by `init_schema()` — kept in sync with the last
/// `PRAGMA user_version` that `migrate()` sets.
const LATEST_SCHEMA_VERSION: i64 = 2;

impl Db {
    pub fn new(path: &Path, device_id: String) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        // Must be checked before init_schema() runs, since CREATE TABLE IF NOT
        // EXISTS would otherwise make every database look pre-existing.
        let is_fresh = !Self::table_exists(&conn, "habits")?;
        let db = Self { conn, device_id };
        db.init_schema()?;
        if is_fresh {
            // init_schema() already creates the latest schema (UUID ids, sync
            // columns, all of it), so a brand-new database has nothing to
            // migrate. Running migrate() anyway would hit `ALTER TABLE ADD
            // COLUMN start_date` on a column init_schema already created,
            // aborting the whole process on first launch.
            db.conn.execute_batch(&format!(
                "PRAGMA user_version = {LATEST_SCHEMA_VERSION};"
            ))?;
        } else {
            db.migrate()?;
        }
        // Indexes are created last, not inside init_schema/migrate_to_uuid_ids: on an
        // upgrade from a pre-sync database, `habits`/`habit_logs` only gain the
        // `updated_at` column once migrate() has run, so indexing it any earlier would
        // fail with "no such column" on every install except a brand new one.
        db.ensure_indexes()?;
        Ok(db)
    }

    fn table_exists(conn: &Connection, name: &str) -> Result<bool> {
        conn.query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [name],
            |_| Ok(()),
        )
        .optional()
        .map(|row| row.is_some())
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS habits (
                id                TEXT PRIMARY KEY,
                type              TEXT NOT NULL,
                label             TEXT NOT NULL,
                sort_order        INTEGER NOT NULL DEFAULT 0,
                target            INTEGER,
                sets              INTEGER,
                target_seconds    INTEGER,
                rounds            INTEGER,
                seconds_per_round INTEGER,
                start_date        TEXT,
                repeat_type       TEXT NOT NULL DEFAULT 'daily',
                repeat_days       TEXT,
                repeat_every      INTEGER,
                updated_at        INTEGER NOT NULL,
                deleted_at        INTEGER,
                device_id         TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS habit_logs (
                habit_id              TEXT NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
                date                  TEXT NOT NULL,
                done                  INTEGER,
                count                 INTEGER,
                completed_sets        INTEGER,
                seconds_elapsed       INTEGER,
                current_round         INTEGER,
                round_seconds_elapsed INTEGER,
                updated_at            INTEGER NOT NULL,
                device_id             TEXT NOT NULL,
                PRIMARY KEY (habit_id, date)
            );
        ",
        )
    }

    fn ensure_indexes(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE INDEX IF NOT EXISTS idx_habits_updated_at ON habits(updated_at);
            CREATE INDEX IF NOT EXISTS idx_habit_logs_updated_at ON habit_logs(updated_at);
            ",
        )
    }

    fn migrate(&self) -> Result<()> {
        let version: i64 = self
            .conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))?;

        if version < 1 {
            self.conn.execute_batch(
                "
                ALTER TABLE habits ADD COLUMN start_date   TEXT;
                ALTER TABLE habits ADD COLUMN repeat_type  TEXT NOT NULL DEFAULT 'daily';
                ALTER TABLE habits ADD COLUMN repeat_days  TEXT;
                ALTER TABLE habits ADD COLUMN repeat_every INTEGER;
                -- Give existing habits today's date so they stay active
                UPDATE habits SET start_date = date('now', 'localtime') WHERE start_date IS NULL;
                PRAGMA user_version = 1;
                ",
            )?;
        }

        if version < 2 {
            self.migrate_to_uuid_ids()?;
            self.conn.execute_batch("PRAGMA user_version = 2;")?;
        }

        Ok(())
    }

    /// Rebuilds `habits`/`habit_logs` with TEXT (UUID) ids plus the sync
    /// metadata columns (`updated_at`, `deleted_at`, `device_id`), matching
    /// enhabitz-mobile's schema column-for-column — see
    /// tasks/14-multi-device-sync-schema.md in that repo. `INTEGER PRIMARY KEY
    /// AUTOINCREMENT` ids are local to one SQLite file, so two devices would
    /// independently produce colliding ids; a sync engine needs globally
    /// unique ids before it can merge two devices' data at all. SQLite can't
    /// ALTER a column's type or generate UUIDs in SQL, so the rename/rebuild
    /// happens here in Rust, one row at a time, with an explicit
    /// old-id → new-id map used to re-point `habit_logs.habit_id`.
    fn migrate_to_uuid_ids(&self) -> Result<()> {
        let now = now_millis();

        self.conn.execute_batch(
            "
            ALTER TABLE habits RENAME TO habits_v1;
            ALTER TABLE habit_logs RENAME TO habit_logs_v1;

            CREATE TABLE habits (
                id                TEXT PRIMARY KEY,
                type              TEXT NOT NULL,
                label             TEXT NOT NULL,
                sort_order        INTEGER NOT NULL DEFAULT 0,
                target            INTEGER,
                sets              INTEGER,
                target_seconds    INTEGER,
                rounds            INTEGER,
                seconds_per_round INTEGER,
                start_date        TEXT,
                repeat_type       TEXT NOT NULL DEFAULT 'daily',
                repeat_days       TEXT,
                repeat_every      INTEGER,
                updated_at        INTEGER NOT NULL,
                deleted_at        INTEGER,
                device_id         TEXT NOT NULL
            );

            CREATE TABLE habit_logs (
                habit_id              TEXT NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
                date                  TEXT NOT NULL,
                done                  INTEGER,
                count                 INTEGER,
                completed_sets        INTEGER,
                seconds_elapsed       INTEGER,
                current_round         INTEGER,
                round_seconds_elapsed INTEGER,
                updated_at            INTEGER NOT NULL,
                device_id             TEXT NOT NULL,
                PRIMARY KEY (habit_id, date)
            );
            ",
        )?;

        let mut id_map: HashMap<i64, String> = HashMap::new();
        {
            let mut select = self.conn.prepare(
                "SELECT id, type, label, sort_order, target, sets, target_seconds, rounds,
                        seconds_per_round, start_date, repeat_type, repeat_days, repeat_every
                 FROM habits_v1",
            )?;
            let mut insert = self.conn.prepare(
                "INSERT INTO habits
                    (id, type, label, sort_order, target, sets, target_seconds, rounds,
                     seconds_per_round, start_date, repeat_type, repeat_days, repeat_every,
                     updated_at, deleted_at, device_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, NULL, ?15)",
            )?;

            let mut rows = select.query([])?;
            while let Some(row) = rows.next()? {
                let old_id: i64 = row.get(0)?;
                let new_id = Uuid::new_v4().to_string();
                id_map.insert(old_id, new_id.clone());

                insert.execute(params![
                    new_id,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    row.get::<_, Option<i64>>(8)?,
                    row.get::<_, Option<String>>(9)?,
                    row.get::<_, String>(10)?,
                    row.get::<_, Option<String>>(11)?,
                    row.get::<_, Option<i64>>(12)?,
                    now,
                    self.device_id,
                ])?;
            }
        }

        {
            let mut select = self.conn.prepare(
                "SELECT habit_id, date, done, count, completed_sets, seconds_elapsed,
                        current_round, round_seconds_elapsed
                 FROM habit_logs_v1",
            )?;
            let mut insert = self.conn.prepare(
                "INSERT INTO habit_logs
                    (habit_id, date, done, count, completed_sets, seconds_elapsed,
                     current_round, round_seconds_elapsed, updated_at, device_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            )?;

            let mut rows = select.query([])?;
            while let Some(row) = rows.next()? {
                let old_habit_id: i64 = row.get(0)?;
                let Some(new_habit_id) = id_map.get(&old_habit_id) else {
                    continue; // orphaned log row with no matching habit — drop it
                };

                insert.execute(params![
                    new_habit_id,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<i64>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<i64>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                    row.get::<_, Option<i64>>(6)?,
                    row.get::<_, Option<i64>>(7)?,
                    now,
                    self.device_id,
                ])?;
            }
        }

        self.conn
            .execute_batch("DROP TABLE habits_v1; DROP TABLE habit_logs_v1;")?;

        Ok(())
    }

    pub fn load_habits(&self) -> Result<Vec<HabitRecord>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT h.id, h.type, h.label, h.target, h.sets, h.target_seconds,
                   h.rounds, h.seconds_per_round,
                   h.start_date, h.repeat_type, h.repeat_days, h.repeat_every,
                   h.updated_at, h.device_id,
                   l.done, l.count, l.completed_sets, l.seconds_elapsed,
                   l.current_round, l.round_seconds_elapsed
            FROM habits h
            LEFT JOIN habit_logs l
                ON l.habit_id = h.id AND l.date = date('now', 'localtime')
            WHERE h.deleted_at IS NULL
            ORDER BY h.sort_order, h.id
        ",
        )?;

        let rows = stmt.query_map([], |row| {
            let start_date: Option<String> = row.get(8)?;
            let repeat_type: String = row.get(9)?;
            let repeat_days: Option<String> = row.get(10)?;
            let repeat_every: Option<i64> = row.get(11)?;
            let active = is_active_today(&start_date, &repeat_type, &repeat_days, &repeat_every);

            Ok(HabitRecord {
                id: row.get(0)?,
                habit_type: row.get(1)?,
                label: row.get(2)?,
                target: row.get(3)?,
                sets: row.get(4)?,
                target_seconds: row.get(5)?,
                rounds: row.get(6)?,
                seconds_per_round: row.get(7)?,
                start_date,
                repeat_type,
                repeat_days,
                repeat_every,
                is_active_today: active,
                updated_at: row.get(12)?,
                device_id: row.get(13)?,
                done: row.get::<_, Option<i64>>(14)?.map(|v| v != 0),
                count: row.get(15)?,
                completed_sets: row.get(16)?,
                seconds_elapsed: row.get(17)?,
                current_round: row.get(18)?,
                round_seconds_elapsed: row.get(19)?,
            })
        })?;

        rows.collect()
    }

    pub fn add_habit(&self, data: HabitData) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = now_millis();
        self.conn.execute(
            "INSERT INTO habits
                (id, type, label, sort_order, target, sets, target_seconds, rounds, seconds_per_round,
                 start_date, repeat_type, repeat_days, repeat_every, updated_at, device_id)
             VALUES (?1, ?2, ?3, (SELECT COALESCE(MAX(sort_order) + 1, 0) FROM habits),
                     ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                id,
                data.habit_type,
                data.label,
                data.target,
                data.sets,
                data.target_seconds,
                data.rounds,
                data.seconds_per_round,
                data.start_date,
                data.repeat_type,
                data.repeat_days,
                data.repeat_every,
                now,
                self.device_id,
            ],
        )?;
        Ok(id)
    }

    pub fn update_habit(&self, id: &str, data: HabitData) -> Result<()> {
        let now = now_millis();
        self.conn.execute(
            "UPDATE habits SET
                type = ?1, label = ?2, target = ?3, sets = ?4,
                target_seconds = ?5, rounds = ?6, seconds_per_round = ?7,
                start_date = ?8, repeat_type = ?9, repeat_days = ?10, repeat_every = ?11,
                updated_at = ?12, device_id = ?13
             WHERE id = ?14",
            params![
                data.habit_type,
                data.label,
                data.target,
                data.sets,
                data.target_seconds,
                data.rounds,
                data.seconds_per_round,
                data.start_date,
                data.repeat_type,
                data.repeat_days,
                data.repeat_every,
                now,
                self.device_id,
                id,
            ],
        )?;
        Ok(())
    }

    pub fn load_log_history(&self) -> Result<Vec<DayEntry>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT l.date, h.label
            FROM habit_logs l
            JOIN habits h ON h.id = l.habit_id
            WHERE h.deleted_at IS NULL AND (
                (h.type = 'todo'          AND l.done = 1) OR
                (h.type = 'counter'       AND h.sets IS NULL     AND l.count >= h.target) OR
                (h.type = 'counter'       AND h.sets IS NOT NULL AND l.completed_sets >= h.sets) OR
                (h.type = 'timer'         AND l.seconds_elapsed >= h.target_seconds) OR
                (h.type = 'counter-timer' AND l.current_round >= h.rounds)
            )
            ORDER BY l.date, h.label
            ",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DayEntry {
                date: row.get(0)?,
                label: row.get(1)?,
            })
        })?;
        rows.collect()
    }

    /// Soft-delete — NOT a real `DELETE`, so the removal can propagate to other
    /// devices once a sync engine exists (see tasks/14-multi-device-sync-schema.md
    /// in enhabitz-mobile). A hard `DELETE` is invisible to a device that hasn't
    /// synced yet — there's no row left to tell it "this was removed".
    pub fn delete_habit(&self, id: &str) -> Result<()> {
        let now = now_millis();
        self.conn.execute(
            "UPDATE habits SET deleted_at = ?1, updated_at = ?1, device_id = ?2 WHERE id = ?3",
            params![now, self.device_id, id],
        )?;
        Ok(())
    }

    pub fn save_log(&self, log: LogData) -> Result<()> {
        let now = now_millis();
        self.conn.execute(
            "INSERT INTO habit_logs
                (habit_id, date, done, count, completed_sets, seconds_elapsed,
                 current_round, round_seconds_elapsed, updated_at, device_id)
             VALUES (?1, date('now', 'localtime'), ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT (habit_id, date) DO UPDATE SET
                 done = excluded.done,
                 count = excluded.count,
                 completed_sets = excluded.completed_sets,
                 seconds_elapsed = excluded.seconds_elapsed,
                 current_round = excluded.current_round,
                 round_seconds_elapsed = excluded.round_seconds_elapsed,
                 updated_at = excluded.updated_at,
                 device_id = excluded.device_id",
            params![
                log.habit_id,
                log.done.map(|b| if b { 1i64 } else { 0 }),
                log.count,
                log.completed_sets,
                log.seconds_elapsed,
                log.current_round,
                log.round_seconds_elapsed,
                now,
                self.device_id,
            ],
        )?;
        Ok(())
    }

    /// Populates a handful of habits plus weeks of history so the Stats page
    /// has something to show during development. Dev-tooling only — compiled
    /// out of release builds entirely, same as the rest of this file follows
    /// the "don't ship what you don't need" rule for Android.
    #[cfg(debug_assertions)]
    pub fn seed_demo_data(&self) -> Result<()> {
        let today = Local::now().date_naive();

        struct SeedHabit {
            habit_type: &'static str,
            label: &'static str,
            target: Option<i64>,
            sets: Option<i64>,
            target_seconds: Option<i64>,
            rounds: Option<i64>,
            seconds_per_round: Option<i64>,
            start_days_ago: i64,
            repeat_type: &'static str,
            repeat_days: Option<&'static str>,
            repeat_every: Option<i64>,
            // Given the index of a *scheduled* day (0 = oldest), returns
            // whether it was completed.
            done: fn(usize) -> bool,
        }

        let habits = [
            // Steady habit with one gap in the middle: shows a best streak
            // longer than the current one.
            SeedHabit {
                habit_type: "todo",
                label: "Drink water",
                target: None,
                sets: None,
                target_seconds: None,
                rounds: None,
                seconds_per_round: None,
                start_days_ago: 45,
                repeat_type: "daily",
                repeat_days: None,
                repeat_every: None,
                done: |i| !(24..=26).contains(&i),
            },
            // Irregular completion — modest streak, ~80% rate.
            SeedHabit {
                habit_type: "counter",
                label: "Push-ups",
                target: Some(30),
                sets: Some(3),
                target_seconds: None,
                rounds: None,
                seconds_per_round: None,
                start_days_ago: 35,
                repeat_type: "daily",
                repeat_days: None,
                repeat_every: None,
                done: |i| i % 5 != 1,
            },
            // Weekly habit, nearly perfect once it got going.
            SeedHabit {
                habit_type: "timer",
                label: "Read",
                target: None,
                sets: None,
                target_seconds: Some(1200),
                rounds: None,
                seconds_per_round: None,
                start_days_ago: 70,
                repeat_type: "weekly",
                repeat_days: Some("[1,3,5]"), // Mon/Wed/Fri
                repeat_every: None,
                done: |i| i >= 2,
            },
            // Just started, perfect so far.
            SeedHabit {
                habit_type: "counter-timer",
                label: "Stretch",
                target: None,
                sets: None,
                target_seconds: None,
                rounds: Some(3),
                seconds_per_round: Some(60),
                start_days_ago: 20,
                repeat_type: "daily",
                repeat_days: None,
                repeat_every: None,
                done: |_| true,
            },
            // Draft — no start date, so it's excluded from stats entirely.
            SeedHabit {
                habit_type: "todo",
                label: "Learn guitar",
                target: None,
                sets: None,
                target_seconds: None,
                rounds: None,
                seconds_per_round: None,
                start_days_ago: -1, // sentinel: no start_date
                repeat_type: "daily",
                repeat_days: None,
                repeat_every: None,
                done: |_| false,
            },
        ];

        for h in habits {
            let start_date = if h.start_days_ago < 0 {
                None
            } else {
                Some((today - Duration::days(h.start_days_ago)).format("%Y-%m-%d").to_string())
            };
            let repeat_days = h.repeat_days.map(|s| s.to_string());

            let id = self.add_habit(HabitData {
                habit_type: h.habit_type.to_string(),
                label: h.label.to_string(),
                target: h.target,
                sets: h.sets,
                target_seconds: h.target_seconds,
                rounds: h.rounds,
                seconds_per_round: h.seconds_per_round,
                start_date: start_date.clone(),
                repeat_type: h.repeat_type.to_string(),
                repeat_days,
                repeat_every: h.repeat_every,
            })?;

            let Some(start_date) = start_date else { continue };
            let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
            let repeat_days = h.repeat_days.map(|s| s.to_string());

            let mut scheduled_index = 0usize;
            let mut date = start;
            while date <= today {
                if is_active_on(&Some(start_date.clone()), h.repeat_type, &repeat_days, &h.repeat_every, date) {
                    if (h.done)(scheduled_index) {
                        self.seed_log_for(&id, h.habit_type, date, h.target, h.sets, h.target_seconds, h.rounds)?;
                    }
                    scheduled_index += 1;
                }
                date += Duration::days(1);
            }
        }

        Ok(())
    }

    /// Writes a habit_logs row for `date` with values that satisfy
    /// load_log_history's "done" criteria for the given habit type.
    #[cfg(debug_assertions)]
    fn seed_log_for(
        &self,
        habit_id: &str,
        habit_type: &str,
        date: NaiveDate,
        target: Option<i64>,
        sets: Option<i64>,
        target_seconds: Option<i64>,
        rounds: Option<i64>,
    ) -> Result<()> {
        let (done, count, completed_sets, seconds_elapsed, current_round) = match habit_type {
            "todo" => (Some(1i64), None, None, None, None),
            "counter" => (None, target, sets, None, None),
            "timer" => (None, None, None, target_seconds, None),
            "counter-timer" => (None, None, None, None, rounds),
            _ => (None, None, None, None, None),
        };

        self.conn.execute(
            "INSERT INTO habit_logs
                (habit_id, date, done, count, completed_sets, seconds_elapsed,
                 current_round, round_seconds_elapsed, updated_at, device_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT (habit_id, date) DO NOTHING",
            params![
                habit_id,
                date.format("%Y-%m-%d").to_string(),
                done,
                count,
                completed_sets,
                seconds_elapsed,
                current_round,
                None::<i64>, // round_seconds_elapsed: irrelevant to the "done" check
                now_millis(),
                self.device_id,
            ],
        )?;
        Ok(())
    }
}
