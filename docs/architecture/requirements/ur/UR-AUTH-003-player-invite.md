# UR-AUTH-003: Player-to-Account Linking via Invite

**Status:** Draft
**Parent:** Spec Section 7 (scope addition per RQ-R3-3), RQ-R3-3 decision
**Priority:** Must-have

## Description

An Admin or League Operator generates an invite link tied to a specific player record. The invitee follows the link to register a new account or log into an existing account, at which point the system automatically links that account to the player record. Each player record can be linked to at most one user account. An Admin can reassign a player record's link to a different account if needed.

## Rationale

Player/Viewers need their user accounts connected to their player records so the system can surface their own rating history and leaderboard position. A manual admin-linking workflow creates a bottleneck at scale. Invite links allow operators to delegate linking to players themselves while ensuring the correct player record is bound to the correct account. Uniqueness enforcement prevents accidental or malicious double-linking.

## Acceptance Criteria

- [ ] An Admin or League Operator can generate an invite link tied to a specific player record
- [ ] The invite link, when visited, prompts the recipient to either register a new account or log into an existing account
- [ ] On successful registration or login via an invite link, the system automatically links the authenticated account to the specified player record
- [ ] If the invitee is already logged in when they visit the invite link, the system links their current session's account to the player record without re-authentication
- [ ] Attempting to link a player record that already has a linked account returns a clear error to the invitee (the link is already claimed)
- [ ] Attempting to link a user account that already has a linked player record to a second player record returns a clear error (one account to one player)
- [ ] A Player/Viewer can see their linked player record in their account profile
- [ ] An Admin can view the current player-to-account mapping for any player record
- [ ] An Admin can reassign a player record's link to a different user account (breaking the prior link)
- [ ] Invite links expire after a configurable period (default: 7 days) and cannot be claimed after expiry

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Player-to-Account Linking via Invite

  Background:
    Given the platform is running and the database is initialized
    And a user "admin" with role "admin" exists and is authenticated
    And a user "alice_op" with role "operator" is assigned to league 42
    And player "Charlie" (id 7) exists in league 42 with no linked account
    And player "Dave" (id 8) exists in league 42 with no linked account

  Scenario: Admin generates an invite link for a player record
    Given "admin" is authenticated
    When "admin" sends POST /api/players/7/invite
    Then the response status is 201 Created
    And the response body contains a plaintext invite token (one-time, opaque)
    And the response body contains a full invite URL containing the token
    And the token's expiry is set to 7 days from now in the database
    And the token is not logged in application logs

  Scenario: League Operator generates an invite link for a player in their league
    Given "alice_op" is authenticated
    When "alice_op" sends POST /api/players/7/invite
    Then the response status is 201 Created
    And the response body contains a plaintext invite token

  Scenario: Unauthenticated user visits invite link and registers a new account - link is claimed
    Given a valid invite token "TOKEN-ABC" exists for player 7 with 6 days until expiry
    And the invite URL is visited by an unauthenticated user
    When the user registers a new account with username "charlie_user" and password "Pass123!" via the invite flow
    Then the response status is 200 OK
    And a user account "charlie_user" is created
    And a player_account_link exists between player 7 and "charlie_user"
    And the invite token "TOKEN-ABC" is marked as claimed (claimed_at is not null)

  Scenario: Authenticated user visits invite link - link is claimed without re-authentication
    Given a valid invite token "TOKEN-DEF" exists for player 8
    And user "dave_user" is authenticated with a valid session
    When "dave_user" sends POST /api/invite/claim with token "TOKEN-DEF"
    Then the response status is 200 OK
    And a player_account_link exists between player 8 and "dave_user"
    And the token "TOKEN-DEF" is marked as claimed

  Scenario: Claiming an already-claimed token returns 400
    Given invite token "TOKEN-GHI" for player 7 has already been claimed by "charlie_user"
    And user "new_user" is authenticated
    When "new_user" sends POST /api/invite/claim with token "TOKEN-GHI"
    Then the response status is 400 Bad Request
    And the response body indicates the link has already been claimed

  Scenario: Linking a player record that already has a linked account returns 409
    Given player 7 is already linked to user "charlie_user"
    And a new invite token "TOKEN-JKL" exists for player 7
    And user "another_user" is authenticated
    When "another_user" sends POST /api/invite/claim with token "TOKEN-JKL"
    Then the response status is 409 Conflict
    And the response body indicates the player record is already linked to an account

  Scenario: Linking to a second player when account already has a link returns 409
    Given user "charlie_user" is already linked to player 7
    And invite token "TOKEN-MNO" exists for player 8
    And "charlie_user" is authenticated
    When "charlie_user" sends POST /api/invite/claim with token "TOKEN-MNO"
    Then the response status is 409 Conflict
    And the response body indicates the account is already linked to a player record

  Scenario: Claiming an expired invite token returns 400
    Given invite token "TOKEN-PQR" for player 7 was created 8 days ago (expired)
    And user "late_user" is authenticated
    When "late_user" sends POST /api/invite/claim with token "TOKEN-PQR"
    Then the response status is 400 Bad Request
    And the response body indicates the invite link has expired

  Scenario: Viewer can see their linked player record in their account profile
    Given user "charlie_user" is linked to player 7
    And "charlie_user" is authenticated
    When "charlie_user" sends GET /api/auth/me
    Then the response body contains linked_player_id 7

  Scenario: Admin can view the player-to-account mapping for any player
    Given player 7 is linked to user "charlie_user"
    And "admin" is authenticated
    When "admin" sends GET /api/players/7/account-link
    Then the response status is 200 OK
    And the response body contains user_id of "charlie_user"

  Scenario: Admin reassigns a player record link to a different account
    Given player 7 is currently linked to user "charlie_user"
    And user "charlie_v2" exists
    And "admin" is authenticated
    When "admin" sends POST /api/players/7/reassign-link with new_user_id of "charlie_v2"
    Then the response status is 200 OK
    And player 7 is now linked to "charlie_v2"
    And the prior link between player 7 and "charlie_user" no longer exists

  Scenario: Player/Viewer cannot generate an invite link
    Given a user "viewer" with role "viewer" is authenticated
    When "viewer" sends POST /api/players/7/invite
    Then the response status is 403 Forbidden

  Scenario: Invite token for a non-existent player returns 404
    Given "admin" is authenticated
    When "admin" sends POST /api/players/9999/invite
    Then the response status is 404 Not Found
```

### Mapped System Requirements

- SR-AUTH-005: Player Invite Linking
