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

// ─── Db impl ──────────────────────────────────────────────────────────────────

impl Db {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let db = Self { conn };
        db.init_schema()?;
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

    pub fn load_habits(&self) -> Result<Vec<HabitRecord>> {
        let mut stmt = self.conn.prepare(
            "
            SELECT h.id, h.type, h.label, h.target, h.sets, h.target_seconds,
                   h.rounds, h.seconds_per_round,
                   l.done, l.count, l.completed_sets, l.seconds_elapsed,
                   l.current_round, l.round_seconds_elapsed
            FROM habits h
            LEFT JOIN habit_logs l
                ON l.habit_id = h.id AND l.date = date('now', 'localtime')
            ORDER BY h.sort_order, h.id
        ",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(HabitRecord {
                id: row.get(0)?,
                habit_type: row.get(1)?,
                label: row.get(2)?,
                target: row.get(3)?,
                sets: row.get(4)?,
                target_seconds: row.get(5)?,
                rounds: row.get(6)?,
                seconds_per_round: row.get(7)?,
                done: row.get::<_, Option<i64>>(8)?.map(|v| v != 0),
                count: row.get(9)?,
                completed_sets: row.get(10)?,
                seconds_elapsed: row.get(11)?,
                current_round: row.get(12)?,
                round_seconds_elapsed: row.get(13)?,
            })
        })?;

        rows.collect()
    }

    pub fn add_habit(&self, data: HabitData) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO habits (type, label, sort_order, target, sets, target_seconds, rounds, seconds_per_round)
             VALUES (?1, ?2, (SELECT COALESCE(MAX(sort_order) + 1, 0) FROM habits), ?3, ?4, ?5, ?6, ?7)",
            params![
                data.habit_type, data.label,
                data.target, data.sets, data.target_seconds,
                data.rounds, data.seconds_per_round,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_habit(&self, id: i64, data: HabitData) -> Result<()> {
        self.conn.execute(
            "UPDATE habits SET type = ?1, label = ?2, target = ?3, sets = ?4,
             target_seconds = ?5, rounds = ?6, seconds_per_round = ?7 WHERE id = ?8",
            params![
                data.habit_type,
                data.label,
                data.target,
                data.sets,
                data.target_seconds,
                data.rounds,
                data.seconds_per_round,
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
