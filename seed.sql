-- Enhabitz demo seed
-- Run with: sqlite3 ~/.local/share/com.mkrbz.enhabitz/enhabitz.db < seed.sql

BEGIN;

-- Demo habits
INSERT OR IGNORE INTO habits (type, label, sort_order, target, start_date, repeat_type)
VALUES
    ('todo',    'Morning stretch', 10, NULL,  date('now', 'localtime'), 'daily'),
    ('counter', 'Push-ups',        11, 10,    date('now', 'localtime'), 'daily'),
    ('counter', 'Read pages',      12, 20,    date('now', 'localtime'), 'daily'),
    ('timer',   'Meditation',      13, NULL,  date('now', 'localtime'), 'daily');

INSERT OR IGNORE INTO habits (type, label, sort_order, target_seconds, start_date, repeat_type)
VALUES
    ('timer', 'Meditation', 13, 600, date('now', 'localtime'), 'daily');

-- Grab their IDs into temp table
CREATE TEMP TABLE seed_ids AS
SELECT id, type, label FROM habits
WHERE label IN ('Morning stretch', 'Push-ups', 'Read pages', 'Meditation');

-- 90 days of logs (~70% completion, varying values)
-- Morning stretch (todo)
INSERT OR IGNORE INTO habit_logs (habit_id, date, done)
SELECT id, date('now', 'localtime', '-' || n || ' days'), 1
FROM seed_ids, (
    WITH RECURSIVE cnt(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM cnt WHERE n < 90)
    SELECT n FROM cnt WHERE (n * 7) % 10 >= 3
)
WHERE label = 'Morning stretch';

-- Push-ups (counter, target 10 — log 10-15)
INSERT OR IGNORE INTO habit_logs (habit_id, date, count)
SELECT id, date('now', 'localtime', '-' || n || ' days'), 10 + (n * 3) % 6
FROM seed_ids, (
    WITH RECURSIVE cnt(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM cnt WHERE n < 90)
    SELECT n FROM cnt WHERE (n * 11) % 10 >= 3
)
WHERE label = 'Push-ups';

-- Read pages (counter, target 20 — log 20-34)
INSERT OR IGNORE INTO habit_logs (habit_id, date, count)
SELECT id, date('now', 'localtime', '-' || n || ' days'), 20 + (n * 5) % 15
FROM seed_ids, (
    WITH RECURSIVE cnt(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM cnt WHERE n < 90)
    SELECT n FROM cnt WHERE (n * 13) % 10 >= 3
)
WHERE label = 'Read pages';

-- Meditation (timer, target 600s — log 600-719s)
INSERT OR IGNORE INTO habit_logs (habit_id, date, seconds_elapsed)
SELECT id, date('now', 'localtime', '-' || n || ' days'), 600 + (n * 17) % 120
FROM seed_ids, (
    WITH RECURSIVE cnt(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM cnt WHERE n < 90)
    SELECT n FROM cnt WHERE (n * 9) % 10 >= 3
)
WHERE label = 'Meditation';

COMMIT;
