//! Alias repository for player alias management
//!
//! Manages the `player_aliases` table. Creating or removing an alias triggers
//! recalculation job insertion for all affected seasons.

use crate::{PersistenceError, Result};
use sqlx::SqlitePool;

/// A player alias relationship
#[derive(Debug, Clone)]
pub struct PlayerAlias {
    pub id: String,
    pub primary_player_id: String,
    pub alias_player_id: String,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Repository for player alias operations
pub struct AliasRepository;

impl AliasRepository {
    /// Create an alias link between two players.
    /// Returns the job IDs for recalculation jobs inserted for affected seasons.
    pub async fn create_alias(
        _pool: &SqlitePool,
        _primary_player_id: &str,
        _alias_player_id: &str,
        _created_by: &str,
    ) -> Result<Vec<String>> {
        Err(PersistenceError::Unknown(
            "create_alias not yet implemented".into(),
        ))
    }

    /// Remove an alias link between two players.
    /// Returns the job IDs for recalculation jobs inserted for affected seasons.
    pub async fn remove_alias(
        _pool: &SqlitePool,
        _primary_player_id: &str,
        _alias_player_id: &str,
    ) -> Result<Vec<String>> {
        Err(PersistenceError::Unknown(
            "remove_alias not yet implemented".into(),
        ))
    }

    /// Get all aliases for a player
    pub async fn get_aliases(_pool: &SqlitePool, _player_id: &str) -> Result<Vec<PlayerAlias>> {
        Err(PersistenceError::Unknown(
            "get_aliases not yet implemented".into(),
        ))
    }

    /// Resolve the full alias group for a player (all linked player IDs)
    pub async fn resolve_alias_group(_pool: &SqlitePool, _player_id: &str) -> Result<Vec<String>> {
        Err(PersistenceError::Unknown(
            "resolve_alias_group not yet implemented".into(),
        ))
    }
}
