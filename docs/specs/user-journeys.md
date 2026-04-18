# User Journeys

## Journey 1: League Operator -- Full Lifecycle

**Persona:** League Operator (non-technical, web UI only)

**Requirements exercised:** UR-AUTH-001, UR-AUTH-002, UR-LM-001, UR-LM-002, UR-PM-001, UR-PM-002, UR-ME-001, UR-ME-002, UR-LB-001, UR-RH-001, UR-ADM-001

### Steps

1. **Log in.** The operator navigates to the platform and logs in with their credentials. They are authenticated and their session is established. The system recognizes them as a League Operator with access to their assigned leagues. *(UR-AUTH-001, UR-AUTH-002, SR-AUTH-001, SR-AUTH-003)*

2. **Create a league.** The operator navigates to the league creation page, enters a league name and description, and selects TrueSkill as the rating algorithm. The UI pre-fills TrueSkill default parameters (mu, sigma, beta, tau, draw_probability). The operator adjusts draw_probability to 0.1 and submits. The system creates the league and its first season. *(UR-LM-001, SR-ALG-001, SR-ALG-002)*

3. **Add players.** The operator navigates to the player management view within the new league. They add six players: four humans and two non-human agents. For two of the humans, they select existing global player records; the rest are created new. Each player is initialized with TrueSkill default ratings (mu=25, sigma=8.333). *(UR-PM-001, SR-PER-001)*

4. **Record individual matches.** The operator records three 1v1 matches using the match entry form. For each match, they select two players and a winner. After each submission, the leaderboard is displayed with updated ratings. The draw option is visible because draw_probability > 0. *(UR-ME-001, SR-PER-002, SR-PER-008)*

5. **View leaderboard.** The operator views the leaderboard. Players are ranked by TrueSkill conservative rating (mu - 3*sigma). Columns show rank, name, rating (mu), uncertainty (sigma), and match count. The operator sorts by match count to see who has played least. *(UR-LB-001, SR-ALG-005, SR-API-001, SR-API-003)*

6. **View rating history.** The operator clicks a player's name to view their profile. The per-season detail chart shows rating progression across the three matches. Rating uncertainty narrows after each match. The profile also shows other leagues the player belongs to. *(UR-RH-001, UR-PM-001)*

7. **Batch entry.** The operator recorded several 1v1 matches offline and now enters them. Using the batch entry workflow, they enter six 1v1 matches. The UI validates each entry interactively. They review and confirm the batch. All matches are processed sequentially. (N-player ranked events cannot be batch-entered; those must use the standard match entry form.) *(UR-ME-002, SR-PER-002, SR-PER-008)*

8. **Correct a mistake.** The operator realizes they recorded the wrong winner on match #2. Since corrections require Admin privileges, they contact the platform admin. The admin logs in, navigates to the match correction interface, and changes the outcome. The system queues an asynchronous recalculation. The admin sees "recalculation in progress." Once complete, all ratings from match #2 forward are updated. An audit log entry records the correction with the admin's verified identity. *(UR-ADM-001, UR-AUTH-002, SR-ADM-001, SR-PER-009)*

9. **Alias two players.** The operator discovers that "PlayerA" and "Player_A" are the same person. They link the two records as aliases. The system queues an asynchronous recalculation for the season. Both records persist; the leaderboard shows a single combined entry once recalculation completes. *(UR-PM-002, SR-PER-007, SR-PER-009)*

10. **Season change.** After several months, the operator decides to switch from TrueSkill to Glicko-2. On the league settings page, they initiate an algorithm type change (a separate action from editing league metadata). The system prompts: reset to defaults or seed from rankings? The operator chooses seeding. A new season is created. Players receive initial Glicko-2 ratings based on their ordinal TrueSkill ranking with spread. *(UR-LM-001, UR-LM-002, SR-ALG-003, SR-ALG-004)*

11. **View cross-season history.** The operator views a player's rating history. The season overview shows final TrueSkill rating for season 1 and current Glicko-2 rating for season 2 in a table. No combined chart is shown. The season picker allows switching between per-season detail views. *(UR-RH-001)*

### Error and Edge Cases

- **Unauthorized access:** The operator tries to access a league they are not assigned to. The system returns 403 Forbidden. *(UR-AUTH-002, SR-AUTH-003)*
- **Duplicate match:** The operator accidentally submits the same match twice (same players, outcome, timestamp). The system rejects the second submission with a clear error. *(SR-PER-004)*
- **Closed season match:** The operator tries to record a match in season 1 after season 2 has started. The system rejects the attempt. *(SR-PER-005)*
- **Guardrail violation:** The operator tries to set K-factor to -5 on an Elo league. The system rejects the value and shows the valid range. *(SR-ALG-002, SR-API-002)*
- **Archived league:** The operator archives a league. Attempts to record matches or add players return errors. The league data remains visible and searchable. *(UR-LM-001)*
- **Soft-delete:** The operator removes a player. The player disappears from the leaderboard but their matches remain in history. New matches cannot reference the removed player. *(UR-PM-001, SR-PER-003)*

---

## Journey 2: Swarm Operator -- Programmatic Integration

**Persona:** Swarm Operator (technical, uses library crate directly)

**Requirements exercised:** UR-AUTH-001, UR-SW-001, SR-PER-001, SR-PER-006, SR-PER-002, SR-PER-008

### Steps

1. **Link the library.** The swarm operator adds `ladder-rs` as a dependency in their Rust project's `Cargo.toml`. They configure the persistence layer by providing a SQLite database path. The library initializes the database schema on first use. *(SR-PER-001)*

2. **Create a league programmatically.** Using the library API, the operator calls `create_league()` with a name, description, and Elo algorithm with default parameters. The library creates the league and first season. *(SR-PER-001, SR-ALG-001)*

3. **Run agent matches.** The swarm operator's agent framework runs 10,000 matches between 500 agents. For each match, the framework calls `record_match()` with two agent identifiers and the outcome. Agents that do not yet exist are auto-created on first reference with default Elo ratings and non-human type. Each match recording is atomic. *(SR-PER-006, SR-PER-002, SR-PER-008)*

4. **Log in to the dashboard.** The swarm operator opens the web UI, logs in with their credentials, and navigates to the swarm dashboard for their league. *(UR-AUTH-001, SR-AUTH-001)*

5. **Monitor via dashboard.** They see:
   - A rating distribution histogram showing the bell curve of agent ratings
   - Match volume chart showing matches per hour over the past day
   - Top 10 and bottom 10 agents by rating
   - Rating velocity showing which agents are improving fastest
   *(UR-SW-001)*

6. **Filter active agents.** The operator uses the "active agent" filter to show only agents with matches in the last 24 hours. Agents that stopped competing are hidden. *(UR-SW-001)*

7. **Review win rates.** The operator views win rate by rating bucket to assess whether the rating system is calibrating correctly (higher-rated agents should win more). *(UR-SW-001)*

### Error and Edge Cases

- **Concurrent writes:** The swarm operator's agent framework and the web server both access the database through the library. SQLite WAL mode and busy_timeout handle contention. No data corruption occurs. *(NFR-SCALE-001, NFR-REL-001)*
- **Crash recovery:** The swarm operator's process crashes mid-batch. On restart, committed matches are present; in-flight transactions were rolled back. No partial state exists. *(NFR-REL-001)*
- **Duplicate submission:** The framework retries a match on a transient error. The duplicate is rejected cleanly, providing idempotency. *(SR-PER-004)*

---

## Journey 3: Player/Viewer -- Read-Only Experience

**Persona:** Player/Viewer (participant or spectator, read-only web UI access)

**Requirements exercised:** UR-AUTH-001, UR-AUTH-002, UR-LB-001, UR-RH-001

### Steps

1. **Log in.** The player navigates to the platform and logs in with their credentials. They are authenticated and their session is established. The system recognizes them as a Player/Viewer. *(UR-AUTH-001, SR-AUTH-001)*

2. **Browse leagues.** The player views the list of leagues. Their list includes all public leagues plus any private leagues where their linked player record is a member. If their account is not linked to a player record, they see only public leagues. Private leagues they have no membership in are not visible (the server returns 404 for direct access attempts to such leagues). They select a public league they participate in. *(UR-LM-001, UR-LB-001, SR-AUTH-003, SR-AUTH-006)*

3. **View leaderboard.** The player views the leaderboard for the current season. They see their own position ranked by the conservative estimate for the league's algorithm. They sort by different columns to explore the standings. *(UR-LB-001, SR-ALG-005, SR-API-001)*

4. **Check own rating history.** The player clicks their own name (linked to their user account's player record) to view their profile. They see their current rating, match count, and a per-season chart of rating progression. The season overview shows their final rating in each completed season. *(UR-RH-001, UR-AUTH-001)*

5. **View other leagues.** The player navigates to another league they belong to and views their rating there. Since players are global, their profile aggregates all leagues they participate in. *(UR-PM-001, UR-LB-001)*

### Error and Edge Cases

- **Attempt write operation.** The player tries to record a match (e.g., by crafting an API request). The system returns 403 Forbidden because Player/Viewer users have no write access. *(UR-AUTH-002, SR-AUTH-002)*
- **Unauthenticated access.** The player's session expires. Subsequent requests return 401 Unauthorized, and they are redirected to the login page. *(SR-AUTH-001)*
- **View unlinked profile.** A Player/Viewer whose account is not linked to a player record can still browse leaderboards and league standings for public leagues, but does not have a "my ratings" shortcut. They cannot see any private leagues, regardless of whether they competed in them before their account was created. *(UR-AUTH-001, SR-AUTH-006)*
