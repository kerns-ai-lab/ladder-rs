# SR-AUTH-007: Library API Key Management

**Status:** Draft
**Parent:** UR-SW-001, UR-AUTH-002
**Priority:** Must-have

## Description

The system provides Admin-managed API keys for authenticating swarm operator processes that access `ladder-rs-persistence` directly (via the library path, bypassing HTTP). Each API key is associated with a user account that holds operator role and league assignments. The persistence crate validates the key at process startup and constructs an authorization context (`SwarmContext`) that scopes all write operations to leagues assigned to that user.

## Rationale

The `ladder-rs-persistence` library can be consumed by multiple independent swarm operator processes, each potentially managing different leagues. Without per-process authentication, any process with a valid `DATABASE_URL` can write to any league. Library API keys provide the same per-operator isolation enforced by the HTTP layer's RBAC middleware, applied at the library level. See ADR-0009.

## Acceptance Criteria

- [ ] An Admin-only endpoint exists to generate a new library API key: `POST /api/admin/api-keys`. The request body includes a `user_id` (must have `operator` role) and a human-readable `description`
- [ ] The key generation response returns the plaintext API key exactly once; subsequent requests return only the key ID and metadata (not the key value)
- [ ] The plaintext key is a cryptographically random 32-byte value; only its SHA-256 hash is stored in the database
- [ ] An Admin-only endpoint exists to list all API keys: `GET /api/admin/api-keys`. Response includes key ID, description, associated user, and `created_at`; the plaintext key and hash are never returned
- [ ] An Admin-only endpoint exists to revoke a key: `DELETE /api/admin/api-keys/:id`. After revocation, the key is rejected on next process initialization
- [ ] The `ladder-rs-persistence` crate exposes an `init_swarm_context(pool, api_key: &str) -> Result<SwarmContext>` function that validates the key against the `api_keys` table and returns a `SwarmContext { user_id }`
- [ ] All persistence write functions that touch league-scoped data accept a `&SwarmContext` parameter and verify that `ctx.user_id` is assigned to the target league via the League Repository's `is_operator` check
- [ ] A swarm operator process that calls a write function for a league they are not assigned to receives `PersistenceError::Unauthorized`
- [ ] Revoking an API key does not immediately terminate running processes; processes continue until their next startup attempt, at which point initialization fails and the process must exit
- [ ] `ladder-rs-server` never constructs a `SwarmContext`; it uses `AuthContext` from the HTTP middleware instead
- [ ] API keys have no automatic expiry in v1; revocation is explicit and admin-only

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Library API Key Management

  Background:
    Given the system is running
    And an Admin user "admin" is authenticated
    And a user "swarm-user" exists with role "operator" assigned to "League A"

  Scenario: Admin generates an API key for a swarm operator
    When admin POSTs to /api/admin/api-keys with user_id for "swarm-user" and description "Swarm process 1"
    Then the HTTP response status is 201
    And the response body contains a plaintext API key
    And the response body contains a key_id
    And the key is stored in the database as a SHA-256 hash

  Scenario: API key plaintext is returned exactly once
    Given admin has generated API key K for "swarm-user"
    When admin GETs /api/admin/api-keys
    Then the response includes key K's metadata (id, description, user, created_at)
    And the response does not include the plaintext key or its hash

  Scenario: Swarm process initializes successfully with a valid API key
    Given admin has generated API key K for "swarm-user"
    When a swarm process calls init_swarm_context(pool, K)
    Then the result is Ok(SwarmContext { user_id: swarm-user.id })

  Scenario: Swarm process fails to initialize with an invalid API key
    When a swarm process calls init_swarm_context(pool, "invalid-key")
    Then the result is Err(PersistenceError::InvalidToken)

  Scenario: Swarm operator can write to an assigned league
    Given a swarm process holds SwarmContext for "swarm-user" (assigned to "League A")
    When the process calls record_match(pool, season_in_league_a, participants, None, &ctx)
    Then the result is Ok

  Scenario: Swarm operator cannot write to an unassigned league
    Given a swarm process holds SwarmContext for "swarm-user" (assigned to "League A" only)
    When the process calls record_match(pool, season_in_league_b, participants, None, &ctx)
    Then the result is Err(PersistenceError::Unauthorized)

  Scenario: Admin revokes an API key
    Given admin has generated API key K for "swarm-user"
    When admin DELETEs /api/admin/api-keys/:key_id
    Then the HTTP response status is 200
    And a new call to init_swarm_context(pool, K) returns Err(PersistenceError::InvalidToken)

  Scenario: Non-admin cannot access API key endpoints
    Given a League Operator is authenticated
    When the operator POSTs to /api/admin/api-keys
    Then the HTTP response status is 403
```
