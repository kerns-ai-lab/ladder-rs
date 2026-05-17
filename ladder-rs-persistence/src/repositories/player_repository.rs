//! Player repository for global player CRUD and league membership
//!
//! Manages the `players` table, `league_players` join table, name-prefix search,
//! soft-delete, and player auto-creation for the swarm operator path.

use crate::{PersistenceError, Player, Result};
use sqlx::SqlitePool;

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

/// Repository for player operations
pub struct PlayerRepository;

impl PlayerRepository {
    /// Create a new player
    pub async fn create_player(
        _pool: &SqlitePool,
        _name: &str,
        _player_type: &str,
    ) -> Result<Player> {
        Err(PersistenceError::Unknown(
            "create_player not yet implemented".into(),
        ))
    }

    /// Get a player by ID
    pub async fn get_player(_pool: &SqlitePool, _id: &str) -> Result<Option<Player>> {
        Err(PersistenceError::Unknown(
            "get_player not yet implemented".into(),
        ))
    }

    /// Get or create a player by name (for swarm auto-creation).
    /// Returns (player, created) — created is true if the player was just created.
    pub async fn get_or_create_player(
        _pool: &SqlitePool,
        _name: &str,
        _player_type: &str,
    ) -> Result<(Player, bool)> {
        Err(PersistenceError::Unknown(
            "get_or_create_player not yet implemented".into(),
        ))
    }

    /// List players in a league with optional filtering
    pub async fn list_players(
        _pool: &SqlitePool,
        _league_id: &str,
        _filter: &PlayerFilter,
    ) -> Result<Vec<Player>> {
        Err(PersistenceError::Unknown(
            "list_players not yet implemented".into(),
        ))
    }

    /// Update a player's fields
    pub async fn update_player(
        _pool: &SqlitePool,
        _id: &str,
        _patch: &PlayerPatch,
    ) -> Result<Player> {
        Err(PersistenceError::Unknown(
            "update_player not yet implemented".into(),
        ))
    }

    /// Soft-delete a player from a league (sets is_active = false on league_players)
    pub async fn soft_delete_from_league(
        _pool: &SqlitePool,
        _league_id: &str,
        _player_id: &str,
    ) -> Result<()> {
        Err(PersistenceError::Unknown(
            "soft_delete_from_league not yet implemented".into(),
        ))
    }

    /// Add a player to a league
    pub async fn add_to_league(
        _pool: &SqlitePool,
        _league_id: &str,
        _player_id: &str,
    ) -> Result<()> {
        Err(PersistenceError::Unknown(
            "add_to_league not yet implemented".into(),
        ))
    }

    /// Search for players by name prefix
    pub async fn search_by_prefix(
        _pool: &SqlitePool,
        _q: &str,
        _limit: usize,
    ) -> Result<Vec<Player>> {
        Err(PersistenceError::Unknown(
            "search_by_prefix not yet implemented".into(),
        ))
    }

    /// Link a player record to a user account
    pub async fn link_account(
        _pool: &SqlitePool,
        _player_id: &str,
        _user_id: &str,
        _created_by: &str,
    ) -> Result<()> {
        Err(PersistenceError::Unknown(
            "link_account not yet implemented".into(),
        ))
    }
}
