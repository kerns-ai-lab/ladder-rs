# SR-AUTH-005: Player Invite Linking

**Status:** Draft
**Parent:** UR-AUTH-003, RQ-R3-3
**Priority:** Must-have

## Description

The system generates cryptographically signed, time-limited invite links that bind a user account to a specific player record. The invite link flow handles new registration, existing account login, and already-authenticated sessions. Uniqueness constraints enforce the one-player-to-one-account invariant. Admins can break and reassign links.

## Rationale

Invite-based linking delegates the identity-to-player binding to the player themselves, removing the admin bottleneck while maintaining correctness. The server-side generation and expiry of tokens prevents forgery. Uniqueness enforcement at the database level ensures data integrity even under concurrent claim attempts.

## Acceptance Criteria

- [ ] The system exposes an endpoint (Admin or League Operator only) that accepts a player record ID and returns a signed invite token and full invite URL
- [ ] Invite tokens are cryptographically random, at least 128 bits of entropy, and opaque to the client
- [ ] Each invite token has an expiry timestamp stored server-side; the default expiry is 7 days from generation
- [ ] The system exposes an invite claim endpoint that accepts a token; requests must be authenticated (logged-in session required)
- [ ] On a valid claim, the system writes a unique player-to-account mapping: `player_id` and `user_id` with a unique constraint on both columns individually (one player → one account, one account → one player)
- [ ] If the token has expired, the claim endpoint returns a 400 error with a message indicating expiry
- [ ] If the player record is already linked to a different account, the claim returns a 409 Conflict response
- [ ] If the authenticated user's account is already linked to a different player record, the claim returns a 409 Conflict response
- [ ] Claiming a token marks it as used; re-presenting a used token returns a 400 error
- [ ] The system exposes an Admin-only endpoint to delete an existing player-to-account link and optionally create a new one in a single atomic operation
- [ ] The system exposes an Admin-only endpoint to list all player-to-account links (with pagination)
- [ ] Invite tokens are invalidated (unusable) when their associated player record is hard-deleted

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Player Invite Linking

  Background:
    Given the system is running
    And a player record "player-001" (Alice) exists without any linked user account
    And user accounts "user-alice" and "user-bob" exist
    And an Admin "admin1" is authenticated

  Scenario: Admin generates an invite token for a player record
    When admin1 POSTs to /admin/players/player-001/invite
    Then the HTTP response status is 201
    And the response includes a "token" field (opaque, non-empty string)
    And the response includes a "invite_url" field containing the full invite URL
    And the token has at least 128 bits of entropy

  Scenario: League Operator can also generate an invite token
    Given a League Operator "op1" is authenticated and assigned to the relevant league
    When op1 POSTs to /admin/players/player-001/invite
    Then the HTTP response status is 201
    And the response includes a valid invite token and URL

  Scenario: Invite token has a 7-day expiry stored server-side
    When admin1 generates an invite token for player-001
    Then the token's expiry timestamp is stored in the database
    And the expiry is exactly 7 days from the generation timestamp

  Scenario: Valid token claim links the authenticated user to the player record
    Given admin1 generated an invite token for player-001
    And user-alice is authenticated
    When user-alice POSTs to /invite/claim with the valid token
    Then the HTTP response status is 200
    And a player-to-account mapping exists with player_id = "player-001" and user_id = "user-alice"

  Scenario: Expired token claim returns 400 with expiry message
    Given a token for player-001 was generated 8 days ago and has not been claimed
    And user-alice is authenticated
    When user-alice POSTs to /invite/claim with the expired token
    Then the HTTP response status is 400
    And the response message indicates the invite token has expired

  Scenario: Claiming an already-claimed token returns 400
    Given user-alice successfully claimed the invite token for player-001
    And user-bob is authenticated
    When user-bob POSTs to /invite/claim with the same (now used) token
    Then the HTTP response status is 400
    And the response message indicates the token has already been used

  Scenario: Player already linked to another account — claim returns 409 Conflict
    Given player-001 is already linked to user-alice's account
    And user-bob is authenticated
    And admin1 generates a new invite token for player-001
    When user-bob claims the new token
    Then the HTTP response status is 409
    And the response message indicates the player record is already linked to another account

  Scenario: User account already linked to another player — claim returns 409 Conflict
    Given user-alice is already linked to a different player record "player-002"
    And an invite token exists for player-001
    When user-alice POSTs to /invite/claim with the token
    Then the HTTP response status is 409
    And the response message indicates the user account is already linked to a different player

  Scenario: Unauthenticated claim attempt returns 401
    Given an invite token exists for player-001
    When an unauthenticated client POSTs to /invite/claim with the token
    Then the HTTP response status is 401

  Scenario: Admin can delete a player-to-account link atomically
    Given player-001 is linked to user-alice
    When admin1 DELETEs /admin/player-links/player-001
    Then the HTTP response status is 200
    And the player-to-account mapping for player-001 is removed
    And user-alice is no longer linked to player-001

  Scenario: Admin can delete and reassign a player link in one atomic operation
    Given player-001 is linked to user-alice
    When admin1 POSTs to /admin/player-links with player_id = "player-001" and new user_id = "user-bob" and delete_existing = true
    Then the HTTP response status is 200
    And player-001 is now linked to user-bob
    And user-alice is no longer linked to player-001
    And the operation was atomic (either fully completed or fully rolled back)

  Scenario: Admin can list all player-to-account links with pagination
    Given 50 player-to-account links exist
    When admin1 requests GET /admin/player-links?limit=25
    Then the HTTP response status is 200
    And the response contains 25 entries with pagination fields

  Scenario: Invite token is invalidated when the associated player record is hard-deleted
    Given an invite token exists for player-001
    When admin1 hard-deletes player-001
    And user-alice POSTs to /invite/claim with the previously valid token
    Then the HTTP response status is 400
    And the response message indicates the invite is no longer valid

  Scenario: Schema enforces UNIQUE(player_id) and UNIQUE(user_id) on player_account_links
    Given the database schema is fully migrated
    When the indexes on player_account_links are inspected
    Then a UNIQUE constraint exists on player_id
    And a UNIQUE constraint exists on user_id
    And no partial index condition exists (constraints apply to all rows regardless of soft-delete status)

  Scenario: Concurrent invite claims for the same user are resolved by UNIQUE constraint
    Given an invite token exists for player-001
    And user-alice is not yet linked to any player
    When two concurrent claim attempts by user-alice execute simultaneously
    Then exactly one succeeds with HTTP 200
    And the other fails with a UNIQUE constraint violation (translated to 409 Conflict)
    And exactly one player_account_links row exists for user-alice
```
