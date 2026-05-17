//! Player repository for global player CRUD and league membership
//!
//! Manages the `players` table, `league_players` join table, name-prefix search,
//! soft-delete, and player auto-creation for the swarm operator path.

use crate::{PersistenceError, Player, Result};
use chrono::Utc;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

/// Filter options for listing players
#[derive(Debug, Clone)]
pub struct PlayerFilter {
    /// Filter by player type (e.g., "human", "non-human")
    pub player_type: Option<String>,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Pagination offset
    pub offset: Option<usize>,
}

impl Default for PlayerFilter {
    fn default() -> Self {
        Self {
            player_type: None,
            is_active: Some(true),
            limit: Some(20),
            offset: Some(0),
        }
    }
}

/// Patch for updating a player
#[derive(Debug, Clone)]
pub struct PlayerPatch {
    pub name: Option<String>,
    pub nickname: Option<String>,
    pub player_type: Option<String>,
    pub is_active: Option<bool>,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Parse an rfc3339 string from the database into a DateTime<Utc>.
fn parse_rfc3339(s: &str) -> std::result::Result<chrono::DateTime<Utc>, String> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| format!("Failed to parse timestamp '{}': {}", s, e))
}

/// Convert an `sqlx::Error` into a `PersistenceError`.
fn map_sqlx_error(e: sqlx::Error) -> PersistenceError {
    if let Some(db_err) = e.as_database_error() {
        if db_err.is_unique_violation() {
            return PersistenceError::Conflict(db_err.message().to_string());
        }
        if db_err.is_foreign_key_violation() {
            return PersistenceError::InvalidInput(format!(
                "Referenced entity does not exist: {}",
                db_err.message()
            ));
        }
    }
    PersistenceError::DatabaseError(e.to_string())
}

/// Escape special LIKE characters (`%`, `_`, `\`) in a search string.
fn escape_like_pattern(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '%' | '_' | '\\' => {
                escaped.push('\\');
                escaped.push(ch);
            }
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Build a `Player` from a database row (global context: no league_id).
fn row_to_player(row: &SqliteRow) -> std::result::Result<Player, sqlx::Error> {
    let created_at_str: String = row.try_get("created_at")?;
    let is_active_int: i32 = row.try_get("is_active")?;

    Ok(Player {
        id: row.try_get("id")?,
        league_id: None,
        name: row.try_get("name")?,
        nickname: row.try_get("nickname")?,
        is_active: is_active_int != 0,
        player_type: row.try_get("player_type")?,
        created_at: parse_rfc3339(&created_at_str).map_err(|msg| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                msg,
            )))
        })?,
    })
}

/// Build a `Player` from a join row (league context).
/// Expects aliased columns: `lp_is_active` for league-membership active status
/// and `lp_league_id` for the league context.
fn row_to_player_with_league(row: &SqliteRow) -> std::result::Result<Player, sqlx::Error> {
    let created_at_str: String = row.try_get("created_at")?;
    let lp_is_active_int: i32 = row.try_get("lp_is_active")?;
    let league_id: String = row.try_get("lp_league_id")?;

    Ok(Player {
        id: row.try_get("id")?,
        league_id: Some(league_id),
        name: row.try_get("name")?,
        nickname: row.try_get("nickname")?,
        is_active: lp_is_active_int != 0,
        player_type: row.try_get("player_type")?,
        created_at: parse_rfc3339(&created_at_str).map_err(|msg| {
            sqlx::Error::Decode(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                msg,
            )))
        })?,
    })
}

/// Repository for player operations
pub struct PlayerRepository;

impl PlayerRepository {
    /// Create a new player
    pub async fn create_player(pool: &SqlitePool, name: &str, player_type: &str) -> Result<Player> {
        // Validate inputs
        if name.trim().is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Player name cannot be empty".into(),
            ));
        }
        if player_type.trim().is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Player type cannot be empty".into(),
            ));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_rfc3339 = now.to_rfc3339();

        sqlx::query(
            "INSERT INTO players (id, name, player_type, is_active, created_at, updated_at) \
             VALUES (?, ?, ?, 1, ?, ?)",
        )
        .bind(&id)
        .bind(name)
        .bind(player_type)
        .bind(&now_rfc3339)
        .bind(&now_rfc3339)
        .execute(pool)
        .await
        .map_err(map_sqlx_error)?;

        Ok(Player {
            id,
            league_id: None,
            name: name.to_string(),
            nickname: None,
            is_active: true,
            player_type: player_type.to_string(),
            created_at: now,
        })
    }

    /// Get a player by ID
    pub async fn get_player(pool: &SqlitePool, id: &str) -> Result<Option<Player>> {
        let row = sqlx::query(
            "SELECT id, name, nickname, player_type, is_active, created_at \
             FROM players WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(r) => {
                let player = row_to_player(&r)
                    .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
                Ok(Some(player))
            }
            None => Ok(None),
        }
    }

    /// Get or create a player by name (for swarm auto-creation).
    /// Returns (player, created) — created is true if the player was just created.
    pub async fn get_or_create_player(
        pool: &SqlitePool,
        name: &str,
        player_type: &str,
    ) -> Result<(Player, bool)> {
        // Try to find existing player by name
        let row = sqlx::query(
            "SELECT id, name, nickname, player_type, is_active, created_at \
             FROM players WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_error)?;

        if let Some(r) = row {
            let player =
                row_to_player(&r).map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
            return Ok((player, false));
        }

        // Not found — create
        let player = Self::create_player(pool, name, player_type).await?;
        Ok((player, true))
    }

    /// List players in a league with optional filtering
    pub async fn list_players(
        pool: &SqlitePool,
        league_id: &str,
        filter: &PlayerFilter,
    ) -> Result<Vec<Player>> {
        use sqlx::QueryBuilder;

        let mut builder = QueryBuilder::new(
            "SELECT p.id, p.name, p.nickname, p.player_type, p.created_at, \
             lp.is_active AS lp_is_active, lp.league_id AS lp_league_id \
             FROM players p \
             INNER JOIN league_players lp ON p.id = lp.player_id \
             WHERE lp.league_id = ",
        );
        builder.push_bind(league_id);

        if let Some(ref player_type) = filter.player_type {
            builder.push(" AND p.player_type = ");
            builder.push_bind(player_type);
        }
        if let Some(is_active) = filter.is_active {
            builder.push(" AND lp.is_active = ");
            builder.push_bind(is_active as i32);
        }

        builder.push(" ORDER BY p.name ASC");

        if let Some(limit) = filter.limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }
        if let Some(offset) = filter.offset {
            builder.push(" OFFSET ");
            builder.push_bind(offset as i64);
        }

        let rows = builder
            .build()
            .fetch_all(pool)
            .await
            .map_err(map_sqlx_error)?;

        let mut players = Vec::with_capacity(rows.len());
        for row in &rows {
            let player = row_to_player_with_league(row)
                .map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
            players.push(player);
        }
        Ok(players)
    }

    /// Update a player's fields
    pub async fn update_player(pool: &SqlitePool, id: &str, patch: &PlayerPatch) -> Result<Player> {
        if id.is_empty() {
            return Err(PersistenceError::InvalidInput(
                "Player ID cannot be empty".into(),
            ));
        }

        let now = Utc::now();
        let now_rfc3339 = now.to_rfc3339();

        // Build dynamic UPDATE SET clause (always includes updated_at)
        let mut set_clauses: Vec<String> = vec!["updated_at = ?".to_string()];

        if patch.name.is_some() {
            set_clauses.push("name = ?".to_string());
        }
        if patch.nickname.is_some() {
            set_clauses.push("nickname = ?".to_string());
        }
        if patch.player_type.is_some() {
            set_clauses.push("player_type = ?".to_string());
        }
        if patch.is_active.is_some() {
            set_clauses.push("is_active = ?".to_string());
        }

        // Build and execute the UPDATE
        let set_sql = set_clauses.join(", ");
        let sql = format!("UPDATE players SET {} WHERE id = ?", set_sql);

        let mut query = sqlx::query(&sql);

        // Bind updated_at
        query = query.bind(&now_rfc3339);

        // Bind optional fields
        if let Some(ref name) = patch.name {
            query = query.bind(name);
        }
        if let Some(ref nickname) = patch.nickname {
            query = query.bind(nickname);
        }
        if let Some(ref player_type) = patch.player_type {
            query = query.bind(player_type);
        }
        if let Some(is_active) = patch.is_active {
            query = query.bind(is_active as i32);
        }

        // Bind the WHERE id
        query = query.bind(id);

        let result = query.execute(pool).await.map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(PersistenceError::NotFound {
                entity: "player".into(),
                id: id.to_string(),
            });
        }

        // Fetch and return the updated row
        let row = sqlx::query(
            "SELECT id, name, nickname, player_type, is_active, created_at \
             FROM players WHERE id = ?",
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(map_sqlx_error)?;

        let player =
            row_to_player(&row).map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
        Ok(player)
    }

    /// Soft-delete a player from a league (sets is_active = false on league_players)
    pub async fn soft_delete_from_league(
        pool: &SqlitePool,
        league_id: &str,
        player_id: &str,
    ) -> Result<()> {
        let result = sqlx::query(
            "UPDATE league_players SET is_active = 0 WHERE league_id = ? AND player_id = ?",
        )
        .bind(league_id)
        .bind(player_id)
        .execute(pool)
        .await
        .map_err(map_sqlx_error)?;

        // Idempotent: if no rows matched, the player wasn't in the league or was already
        // soft-deleted — both are acceptable outcomes.
        let _ = result;
        Ok(())
    }

    /// Add a player to a league
    pub async fn add_to_league(pool: &SqlitePool, league_id: &str, player_id: &str) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_rfc3339 = now.to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO league_players (id, league_id, player_id, is_active, joined_at, created_at) \
             VALUES (?, ?, ?, 1, ?, ?)",
        )
        .bind(&id)
        .bind(league_id)
        .bind(player_id)
        .bind(&now_rfc3339)
        .bind(&now_rfc3339)
        .execute(pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(db_err) = e.as_database_error() {
                    if db_err.is_unique_violation() {
                        // Idempotent: player is already in the league
                        return Ok(());
                    }
                    if db_err.is_foreign_key_violation() {
                        return Err(PersistenceError::InvalidInput(format!(
                            "Referenced entity does not exist: {}",
                            db_err.message()
                        )));
                    }
                }
                Err(PersistenceError::DatabaseError(e.to_string()))
            }
        }
    }

    /// Search for players by name prefix
    pub async fn search_by_prefix(pool: &SqlitePool, q: &str, limit: usize) -> Result<Vec<Player>> {
        // Escape special LIKE characters in the query, then append '%' for prefix matching
        let pattern = format!("{}%", escape_like_pattern(q));

        let rows = sqlx::query(
            "SELECT id, name, nickname, player_type, is_active, created_at \
             FROM players WHERE name LIKE ? ESCAPE '\\' \
             ORDER BY name ASC LIMIT ?",
        )
        .bind(&pattern)
        .bind(limit as i64)
        .fetch_all(pool)
        .await
        .map_err(map_sqlx_error)?;

        let mut players = Vec::with_capacity(rows.len());
        for row in &rows {
            let player =
                row_to_player(row).map_err(|e| PersistenceError::DatabaseError(e.to_string()))?;
            players.push(player);
        }
        Ok(players)
    }

    /// Link a player record to a user account
    pub async fn link_account(
        pool: &SqlitePool,
        player_id: &str,
        user_id: &str,
        _created_by: &str,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let now_rfc3339 = now.to_rfc3339();

        let result = sqlx::query(
            "INSERT INTO player_account_links (id, player_id, user_id, created_at) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(player_id)
        .bind(user_id)
        .bind(&now_rfc3339)
        .execute(pool)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(db_err) = e.as_database_error() {
                    if db_err.is_unique_violation() {
                        return Err(PersistenceError::Conflict(db_err.message().to_string()));
                    }
                    if db_err.is_foreign_key_violation() {
                        return Err(PersistenceError::InvalidInput(format!(
                            "Referenced entity does not exist: {}",
                            db_err.message()
                        )));
                    }
                }
                Err(PersistenceError::DatabaseError(e.to_string()))
            }
        }
    }
}
