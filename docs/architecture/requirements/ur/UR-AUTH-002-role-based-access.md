# UR-AUTH-002: Role-Based Access Control

**Status:** Draft
**Parent:** Spec Section 7 (scope reversal per RQ-R2-1), RQ-R2-1 + RQ-R2-1a decisions
**Priority:** Must-have

## Description

The platform enforces role-based access control with three roles: Admin, League Operator, and Player/Viewer. Roles are league-scoped for League Operators, meaning an operator must be explicitly granted access to each league they manage. Admins have global access. Player/Viewers have read-only access.

### Role Definitions

1. **Admin** -- Global access. Can manage user accounts, make match corrections, and perform all league operations across all leagues.
2. **League Operator** -- Scoped to assigned leagues. Can manage players, matches, and seasons within their assigned leagues. Cannot make match corrections or manage user accounts.
3. **Player/Viewer** -- Read-only. Can view leaderboards, their own rating history, and league standings. No write access to any resource.

## Rationale

RBAC provides appropriate access boundaries for a multi-user platform. League-scoped operator assignment allows delegation of management responsibilities without granting system-wide access. The three-tier model covers the identified personas: platform administrators, league managers, and participants/spectators.

## Acceptance Criteria

- [ ] Every authenticated user has exactly one role: Admin, League Operator, or Player/Viewer
- [ ] An Admin can assign roles to other users
- [ ] An Admin can grant or revoke League Operator access to specific leagues
- [ ] A League Operator can only perform write operations (manage players, record matches, manage seasons) in leagues they have been explicitly assigned to
- [ ] A League Operator cannot make match corrections (this is an Admin-only action)
- [ ] A League Operator cannot manage user accounts or roles
- [ ] A Player/Viewer cannot perform any write operations (no creating leagues, players, matches, or seasons)
- [ ] A Player/Viewer can view leaderboards, league standings, and their own linked player's rating history
- [ ] An Admin can perform all operations a League Operator can, in any league, without needing explicit assignment
- [ ] Attempting an operation beyond a user's role returns a 403 Forbidden response with a clear error message
- [ ] League-scoped access is checked on every request: an operator assigned to League A cannot access League B's management endpoints

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Role-Based Access Control

  Background:
    Given the platform is running and the database is initialized
    And a user "admin" with role "admin" exists and is authenticated
    And a user "alice_op" with role "operator" exists, assigned to league 42
    And a user "viewer_user" with role "viewer" exists and is authenticated
    And league 42 "Alpha League" is active with an open Elo season (id 7)
    And league 99 "Other League" is active (alice_op is NOT assigned)

  Scenario: Every authenticated user has exactly one role
    When "admin" sends GET /api/auth/me
    Then the response body contains role "admin"
    When "alice_op" sends GET /api/auth/me
    Then the response body contains role "operator"
    When "viewer_user" sends GET /api/auth/me
    Then the response body contains role "viewer"

  Scenario: Admin can assign the operator role to a user
    Given user "carol" exists with role "viewer"
    And "admin" is authenticated
    When "admin" sends PATCH /api/admin/users/carol with role "operator"
    Then the response status is 200 OK
    And user "carol" now has role "operator"

  Scenario: Admin can grant League Operator access to a specific league
    Given user "carol" has role "operator"
    And "admin" is authenticated
    When "admin" sends POST /api/leagues/42/operators with user_id of "carol"
    Then the response status is 201 Created
    And "carol" is listed as an operator for league 42

  Scenario: Admin can revoke League Operator access to a specific league
    Given "alice_op" is assigned to league 42
    And "admin" is authenticated
    When "admin" sends DELETE /api/leagues/42/operators/alice_op
    Then the response status is 200 OK
    And "alice_op" is no longer listed as an operator for league 42
    And subsequent write requests from "alice_op" to league 42 return 403

  Scenario: League Operator can perform writes in their assigned league
    Given "alice_op" is authenticated and assigned to league 42
    When "alice_op" sends POST /api/leagues/42/players with name "NewPlayer" and type "human"
    Then the response status is 201 Created

  Scenario: League Operator cannot perform writes in an unassigned league
    Given "alice_op" is authenticated and NOT assigned to league 99
    When "alice_op" sends POST /api/leagues/99/players with name "Sneaky" and type "human"
    Then the response status is 403 Forbidden
    And the response body contains an error indicating insufficient role or assignment

  Scenario: League Operator cannot make a match correction
    Given "alice_op" is authenticated
    And match 100 exists in season 7 (league 42)
    When "alice_op" sends PATCH /api/matches/100 with corrected participants
    Then the response status is 403 Forbidden

  Scenario: League Operator cannot manage user accounts or roles
    Given "alice_op" is authenticated
    When "alice_op" sends PATCH /api/admin/users/viewer_user with role "admin"
    Then the response status is 403 Forbidden
    When "alice_op" sends POST /api/admin/users with username "hax" and role "admin"
    Then the response status is 403 Forbidden

  Scenario: Player/Viewer can view public leaderboards and league listings
    Given "viewer_user" is authenticated
    When "viewer_user" sends GET /api/leagues
    Then the response status is 200 OK
    When "viewer_user" sends GET /api/seasons/7/leaderboard
    Then the response status is 200 OK

  Scenario: Player/Viewer cannot create a league
    Given "viewer_user" is authenticated
    When "viewer_user" sends POST /api/leagues with name "ViewerLeague" and algorithm "elo"
    Then the response status is 403 Forbidden

  Scenario: Player/Viewer cannot record a match
    Given "viewer_user" is authenticated
    When "viewer_user" sends POST /api/seasons/7/matches with valid participants
    Then the response status is 403 Forbidden

  Scenario: Player/Viewer cannot add or remove players
    Given "viewer_user" is authenticated
    When "viewer_user" sends POST /api/leagues/42/players with name "ViewerPlayer" and type "human"
    Then the response status is 403 Forbidden

  Scenario: Admin can perform all operations an operator can in any league without explicit assignment
    Given "admin" is authenticated and has NO explicit assignment to league 42
    When "admin" sends POST /api/leagues/42/players with name "AdminPlayer" and type "human"
    Then the response status is 201 Created

  Scenario: Operations beyond user role return 403 with clear error
    Given "viewer_user" is authenticated
    When "viewer_user" sends DELETE /api/leagues/42/players/1
    Then the response status is 403 Forbidden
    And the response body contains error_code "FORBIDDEN"
    And the response body contains a message indicating the required role

  Scenario: League-scoped access is checked on every request
    Given "alice_op" is authenticated and assigned to league 42 but NOT league 99
    When "alice_op" sends PATCH /api/leagues/99/seasons/12 with algorithm_params k_factor 16
    Then the response status is 403 Forbidden

  Scenario: Unauthenticated request returns 401 before role check returns 403
    When an unauthenticated client sends DELETE /api/leagues/42/players/1
    Then the response status is 401 Unauthorized
    And the response body does not contain role-related error information
```
