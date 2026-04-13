use chrono::Datelike;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct Db {
    conn: Connection,
}

// ─── Wire types (flat, matches SQL schema) ────────────────────────────────────

/// Returned by load_habits — flat row with nullable fields per type.
#[derive(Serialize)]
pub struct HabitRecord {
    pub id: i64,
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

/// Sent by frontend after every progress mutation.
#[derive(Deserialize)]
pub struct LogData {
    pub habit_id: i64,
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
    use chrono::{Local, NaiveDate};

    let today = Local::now().date_naive();

    let start = match start_date {
        None => return false, // draft / idea
        Some(s) => match NaiveDate::parse_from_str(s, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return false,
        },
    };

    if today < start {
        return false; // hasn't started yet
    }

    match repeat_type {
        "daily" => true,
        "weekly" => {
            // repeat_days: JSON array of 0–6 where 0 = Sunday (matches JS Date.getDay())
            let weekday_num = today.weekday().num_days_from_sunday();
            let days: Vec<u32> = repeat_days
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            days.contains(&weekday_num)
        }
        "monthly" => {
            // repeat_days: JSON array of day-of-month numbers 1–31
            let day = today.day();
            let days: Vec<u32> = repeat_days
                .as_ref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default();
            days.contains(&day)
        }
        "interval" => {
            // every N days from start_date
            let every = repeat_every.unwrap_or(1).max(1);
            let diff = (today - start).num_days();
            diff % every == 0
        }
        _ => true,
    }
}

// ─── Db impl ──────────────────────────────────────────────────────────────────

impl Db {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self { conn };
        db.init_schema()?;
        db.migrate()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS habits (
                id                INTEGER PRIMARY KEY AUTOINCREMENT,
                type              TEXT NOT NULL,
                label             TEXT NOT NULL,
                sort_order        INTEGER NOT NULL DEFAULT 0,
                target            INTEGER,
                sets              INTEGER,
                target_seconds    INTEGER,
                rounds            INTEGER,
                seconds_per_round INTEGER
            );

            CREATE TABLE IF NOT EXISTS habit_logs (
                habit_id              INTEGER NOT NULL REFERENCES habits(id) ON DELETE CASCADE,
                date                  TEXT NOT NULL,
                done                  INTEGER,
                count                 INTEGER,
                completed_sets        INTEGER,
                seconds_elapsed       INTEGER,
                current_round         INTEGER,
                round_seconds_elapsed INTEGER,
                PRIMARY KEY (habit_id, date)
            );
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

        Ok(())
    }

    pub fn load_habits(&self) -> Result<Vec<HabitRecord>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT h.id, h.type, h.label, h.target, h.sets, h.target_seconds,
                   h.rounds, h.seconds_per_round,
                   h.start_date, h.repeat_type, h.repeat_days, h.repeat_every,
                   l.done, l.count, l.completed_sets, l.seconds_elapsed,
                   l.current_round, l.round_seconds_elapsed
            FROM habits h
            LEFT JOIN habit_logs l
                ON l.habit_id = h.id AND l.date = date('now', 'localtime')
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
                done: row.get::<_, Option<i64>>(12)?.map(|v| v != 0),
                count: row.get(13)?,
                completed_sets: row.get(14)?,
                seconds_elapsed: row.get(15)?,
                current_round: row.get(16)?,
                round_seconds_elapsed: row.get(17)?,
            })
        })?;

        rows.collect()
    }

    pub fn add_habit(&self, data: HabitData) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO habits
                (type, label, sort_order, target, sets, target_seconds, rounds, seconds_per_round,
                 start_date, repeat_type, repeat_days, repeat_every)
             VALUES (?1, ?2, (SELECT COALESCE(MAX(sort_order) + 1, 0) FROM habits),
                     ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
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
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_habit(&self, id: i64, data: HabitData) -> Result<()> {
        self.conn.execute(
            "UPDATE habits SET
                type = ?1, label = ?2, target = ?3, sets = ?4,
                target_seconds = ?5, rounds = ?6, seconds_per_round = ?7,
                start_date = ?8, repeat_type = ?9, repeat_days = ?10, repeat_every = ?11
             WHERE id = ?12",
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
                id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_habit(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM habits WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn save_log(&self, log: LogData) -> Result<()> {
        self.conn.execute(
            "INSERT INTO habit_logs (habit_id, date, done, count, completed_sets, seconds_elapsed, current_round, round_seconds_elapsed)
             VALUES (?1, date('now', 'localtime'), ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT (habit_id, date) DO UPDATE SET
                 done = excluded.done,
                 count = excluded.count,
                 completed_sets = excluded.completed_sets,
                 seconds_elapsed = excluded.seconds_elapsed,
                 current_round = excluded.current_round,
                 round_seconds_elapsed = excluded.round_seconds_elapsed",
            params![
                log.habit_id,
                log.done.map(|b| if b { 1i64 } else { 0 }),
                log.count,
                log.completed_sets,
                log.seconds_elapsed,
                log.current_round,
                log.round_seconds_elapsed,
            ],
        )?;
        Ok(())
    }
}
