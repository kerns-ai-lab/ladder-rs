# UR-AUTH-001: User Accounts

**Status:** Draft
**Parent:** Spec Section 7 (scope reversal per RQ-R2-1), RQ-R2-1 decision
**Priority:** Must-have

## Description

The platform supports user accounts with registration, login, and profile management. Every user who interacts with the web UI must authenticate. User accounts are the foundation for role-based access control (see UR-AUTH-002). Each account has a unique identity used for audit trails, access control decisions, and optional linking to a player record.

## Rationale

User accounts are required to support RBAC, which was brought into v1 scope during round 2 requirements elicitation. Without authenticated identities, the system cannot enforce role-based permissions, attribute admin corrections to specific users, or scope operator access to specific leagues. This reverses the previous "no user accounts" out-of-scope decision.

## Acceptance Criteria

- [ ] A new user can register an account with a unique username, email, and password
- [ ] A registered user can log in with their credentials and receive an authenticated session
- [ ] A logged-in user can view and edit their own profile (display name, email)
- [ ] A logged-in user can change their password
- [ ] A logged-in user can log out, invalidating their session
- [ ] Unauthenticated requests to any API endpoint (except login and registration) are rejected with a 401 status
- [ ] A Player/Viewer user account can be linked to a player record, allowing them to see "their" rating history across leagues
- [ ] Passwords are never stored in plaintext; the system uses a secure hashing algorithm (e.g., bcrypt, argon2)
- [ ] An Admin can set a temporary password for any user account; that account is required to change the temporary password on next login before accessing any other functionality
- [ ] After an Admin sets a temporary password, the affected user can log in with the temporary password and is immediately prompted to choose a new password

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: User Accounts

  Background:
    Given the platform is running with at least one existing admin account
    And the admin account "admin" has username "admin", email "admin@example.com", and a known password

  Scenario: New user registers with unique username, email, and password
    When an unauthenticated client sends POST /api/auth/register with username "newuser", email "new@example.com", and password "SecurePass123!"
    Then the response status is 201 Created
    And a user account exists with username "newuser" and email "new@example.com"
    And the password is NOT stored in plaintext (password_hash column contains a hash)

  Scenario: Registered user logs in and receives an authenticated session
    Given user "testuser" exists with password "CorrectPassword!"
    When "testuser" sends POST /api/auth/login with username "testuser" and password "CorrectPassword!"
    Then the response status is 200 OK
    And the response sets a session cookie with HttpOnly, Secure, and SameSite=Strict attributes
    And the session token is NOT present in the response body

  Scenario: Logged-in user can view and edit their own profile
    Given "testuser" is authenticated with a valid session
    When "testuser" sends GET /api/auth/me
    Then the response status is 200 OK
    And the response body contains username "testuser"
    When "testuser" sends PATCH /api/auth/me with email "updated@example.com"
    Then the response status is 200 OK
    And the user's email is now "updated@example.com"

  Scenario: Logged-in user can change their password
    Given "testuser" is authenticated with a valid session
    When "testuser" sends POST /api/auth/change-password with current_password "CorrectPassword!" and new_password "NewPassword456!"
    Then the response status is 200 OK
    And subsequent login with "NewPassword456!" succeeds
    And subsequent login with "CorrectPassword!" fails

  Scenario: Logged-in user can log out, invalidating their session
    Given "testuser" is authenticated with a valid session (cookie value = "sess-abc123")
    When "testuser" sends POST /api/auth/logout
    Then the response status is 200 OK
    And the response sets the session cookie with Max-Age=0 (cookie cleared)
    When "testuser" sends GET /api/leagues using the old session cookie "sess-abc123"
    Then the response status is 401 Unauthorized

  Scenario: Unauthenticated request to a protected endpoint is rejected with 401
    When an unauthenticated client sends GET /api/leagues
    Then the response status is 401 Unauthorized
    When an unauthenticated client sends GET /api/seasons/7/leaderboard
    Then the response status is 401 Unauthorized

  Scenario: Registration with duplicate username is rejected
    Given user "existinguser" already exists
    When a client sends POST /api/auth/register with username "existinguser" and a new email "other@example.com"
    Then the response status is 409 Conflict
    And the response body contains error_code "DUPLICATE_USER"

  Scenario: Registration with duplicate email is rejected
    Given user with email "taken@example.com" already exists
    When a client sends POST /api/auth/register with a new username "anotheruser" and email "taken@example.com"
    Then the response status is 409 Conflict
    And the response body contains error_code "DUPLICATE_USER"

  Scenario: Failed login attempt returns a generic error without revealing whether username is wrong
    When a client sends POST /api/auth/login with username "noexist" and password "anything"
    Then the response status is 401 Unauthorized
    And the response body contains a generic message without indicating whether the username or password is wrong

  Scenario: Admin sets a temporary password for a user account
    Given "admin" is authenticated
    And user "target_user" exists with a current password
    When "admin" sends POST /api/admin/users/target_user/set-password with temporary_password "TempPass789!"
    Then the response status is 200 OK
    And "target_user"'s force_password_change flag is set to 1

  Scenario: User with force_password_change flag can log in but is blocked from other endpoints
    Given "target_user"'s force_password_change flag is 1
    When "target_user" sends POST /api/auth/login with username "target_user" and temporary password "TempPass789!"
    Then the response status is 200 OK and a session is established
    When "target_user" sends GET /api/leagues using that session
    Then the response status is 403 Forbidden
    And the response body indicates a password change is required

  Scenario: User with force_password_change flag completes password change and gains full access
    Given "target_user" is authenticated with force_password_change flag 1
    When "target_user" sends POST /api/auth/change-password with temporary password and new_password "FinalPass000!"
    Then the response status is 200 OK
    And "target_user"'s force_password_change flag is now 0
    When "target_user" sends GET /api/leagues using the same session
    Then the response status is 200 OK

  Scenario: Viewer can link their account to a player record
    Given user "viewer_user" with role "viewer" is authenticated
    And player "Alice" (id 1) exists with no linked account
    And a valid invite token for player 1 was generated by an operator
    When "viewer_user" sends POST /api/invite/claim with the token
    Then the response status is 200 OK
    And a player_account_link exists between player 1 and "viewer_user"
```
