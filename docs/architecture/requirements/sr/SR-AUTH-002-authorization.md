# SR-AUTH-002: Authorization

**Status:** Draft
**Parent:** UR-AUTH-002
**Priority:** Must-have

## Description

The system enforces role-based authorization on all API endpoints. Every request is checked against the authenticated user's role before processing. The authorization layer maps each endpoint to the minimum required role and rejects requests from users with insufficient privileges.

## Rationale

Authorization enforcement ensures that RBAC policies are applied consistently across the entire API surface. Centralizing authorization logic prevents individual endpoints from having inconsistent access rules.

## Acceptance Criteria

- [ ] Every API endpoint is annotated with a minimum required role (Admin, League Operator, or Player/Viewer)
- [ ] Read-only endpoints (leaderboard, rating history, league listing) are accessible to all authenticated users (Player/Viewer and above)
- [ ] Write endpoints for league management (create/edit/archive league, manage players, record matches, manage seasons) require League Operator or Admin role
- [ ] Match correction endpoints require Admin role
- [ ] User management endpoints (create/edit/delete users, assign roles) require Admin role
- [ ] A request from a user with insufficient role returns 403 Forbidden with a structured error response indicating the required role
- [ ] Authorization checks occur after authentication (a 401 is returned before a 403 if the user is not authenticated)
- [ ] The authorization layer is applied uniformly; no endpoint bypasses role checking (except login and registration)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Role-Based Authorization

  Background:
    Given the system is running
    And the following user accounts exist:
      | Username | Role            |
      | admin1   | Admin           |
      | op1      | League Operator |
      | viewer1  | Player/Viewer   |
    And league "Test League" exists

  Scenario: Player/Viewer can read the leaderboard
    Given viewer1 is authenticated
    When viewer1 requests GET /leagues/test-league/leaderboard
    Then the HTTP response status is 200

  Scenario: Player/Viewer can read the league listing
    Given viewer1 is authenticated
    When viewer1 requests GET /leagues
    Then the HTTP response status is 200

  Scenario: Player/Viewer cannot record a match
    Given viewer1 is authenticated
    When viewer1 POSTs to /leagues/test-league/matches with valid match data
    Then the HTTP response status is 403
    And the response body indicates insufficient permissions

  Scenario: Player/Viewer cannot create a league
    Given viewer1 is authenticated
    When viewer1 POSTs to /leagues with valid league data
    Then the HTTP response status is 403

  Scenario: League Operator can record a match in their assigned league
    Given op1 is authenticated and assigned to "Test League"
    When op1 POSTs to /leagues/test-league/matches with valid match data
    Then the HTTP response status is 201

  Scenario: League Operator cannot correct a match (Admin-only)
    Given op1 is authenticated and assigned to "Test League"
    When op1 PATCHes /leagues/test-league/matches/match-001 with a corrected outcome
    Then the HTTP response status is 403
    And the error message indicates the Admin role is required

  Scenario: League Operator cannot create or edit user accounts
    Given op1 is authenticated
    When op1 POSTs to /admin/users with user creation data
    Then the HTTP response status is 403

  Scenario: Admin can correct a match
    Given admin1 is authenticated
    When admin1 PATCHes /leagues/test-league/matches/match-001 with a corrected outcome
    Then the HTTP response status is 202 (async correction accepted)

  Scenario: Admin can create user accounts
    Given admin1 is authenticated
    When admin1 POSTs to /admin/users with valid user data
    Then the HTTP response status is 201

  Scenario: Unauthenticated request returns 401 before authorization check
    Given no session cookie is present
    When a client POSTs to /leagues/test-league/matches
    Then the HTTP response status is 401
    And the HTTP response status is not 403

  Scenario: 403 response includes structured error indicating the required role
    Given viewer1 is authenticated
    When viewer1 POSTs to /leagues with valid league data
    Then the HTTP response status is 403
    And the response body contains "error_code" and "message"
    And the message indicates which role is required (League Operator or Admin)

  Scenario: Authorization is applied uniformly — no endpoint bypasses the check
    Given viewer1 is authenticated
    When viewer1 attempts write operations on any of: /leagues, /leagues/test-league/players, /leagues/test-league/matches
    Then every write attempt returns 403
    And none of the write endpoints accept the request without checking the role

  Scenario: Login and registration endpoints are accessible without authentication
    When an unauthenticated client POSTs to /auth/login
    Then the HTTP response status is not 401 (login endpoint itself does not require authentication)
    When an unauthenticated client POSTs to /auth/register
    Then the HTTP response status is not 401 (registration endpoint does not require authentication)

  Scenario: Deactivated user with valid session receives 401 and cookie is cleared
    Given alice is authenticated with a valid session cookie
    And an Admin deactivates alice's account (is_active = 0)
    When alice makes a request to GET /leagues
    Then the HTTP response status is 401
    And the response clears the session cookie with Max-Age=0 or equivalent

  Scenario: Deactivated user cannot log in
    Given alice's account has is_active = 0
    When alice POSTs to /auth/login with correct credentials
    Then the HTTP response status is 401
    And the response body contains a generic error message
    And the message does not specifically reveal that the account is deactivated
```
