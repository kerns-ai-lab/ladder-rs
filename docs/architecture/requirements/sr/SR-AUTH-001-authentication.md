# SR-AUTH-001: Authentication

**Status:** Draft
**Parent:** UR-AUTH-001
**Priority:** Must-have

## Description

The system provides user authentication via login and session management. Users authenticate with credentials (username/email and password). The system issues session tokens upon successful authentication and validates them on every subsequent request. Passwords are securely hashed before storage.

## Rationale

Authentication is the prerequisite for all authorization decisions. Session-based authentication allows the system to identify the acting user on every request, which is required for RBAC enforcement, audit logging, and user-scoped data access.

## Acceptance Criteria

- [ ] The system exposes a login endpoint that accepts username/email and password and returns a session token on success
- [ ] The system exposes a registration endpoint that creates a new user account with a hashed password
- [ ] Passwords are hashed using a computationally expensive algorithm (bcrypt, argon2, or scrypt) with a unique salt per user
- [ ] Session tokens are cryptographically random and sufficiently long to resist brute-force attacks (minimum 128 bits of entropy)
- [ ] Session tokens have a configurable expiration time
- [ ] The system validates the session token on every API request and rejects expired or invalid tokens with a 401 response
- [ ] The system exposes a logout endpoint that invalidates the current session token
- [ ] Failed login attempts return a generic error message that does not reveal whether the username or password was incorrect
- [ ] The registration endpoint rejects duplicate usernames and duplicate emails with a clear error
- [ ] The login endpoint integrates with the login rate limiting mechanism (NFR-SEC-001): after 10 consecutive failed attempts the account is locked and the endpoint returns 429 for subsequent attempts until the lockout clears
- [ ] Session tokens are issued exclusively as cookies with `HttpOnly`, `Secure`, and `SameSite=Strict` attributes (NFR-SEC-002); the token value is not present in the response body
- [ ] An Admin-only endpoint exists to set a temporary password for any user account; on success, the target account is flagged to require a password change on next login
- [ ] An authenticated user whose account is flagged for a required password change can only access the password change endpoint; all other authenticated endpoints return 403 until the password is changed
- [ ] The password change endpoint clears the forced-change flag on success

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Authentication

  Background:
    Given the system is running
    And a user account "alice@example.com" exists with password "correct-password"

  Scenario: Successful login returns session token as HttpOnly cookie
    When a client POSTs to /auth/login with email "alice@example.com" and password "correct-password"
    Then the HTTP response status is 200
    And the response sets a session cookie with attribute "HttpOnly"
    And the response sets a session cookie with attribute "Secure"
    And the response sets a session cookie with attribute "SameSite=Strict"
    And the response body does not contain the session token value

  Scenario: Failed login with wrong password returns generic error
    When a client POSTs to /auth/login with email "alice@example.com" and password "wrong-password"
    Then the HTTP response status is 401
    And the response body contains a generic error message
    And the response body does not indicate whether the username or password was incorrect

  Scenario: Failed login with non-existent username returns the same generic error
    When a client POSTs to /auth/login with email "nobody@example.com" and password "any-password"
    Then the HTTP response status is 401
    And the error message is identical to the one returned for a wrong password

  Scenario: Successful registration creates a new account with hashed password
    When a client POSTs to /auth/register with username "newuser", email "new@example.com", and password "SecurePass1!"
    Then the HTTP response status is 201
    And the user account "new@example.com" exists in the database
    And the stored password is a hash, not the plaintext "SecurePass1!"

  Scenario: Registration rejects duplicate email
    Given user account "alice@example.com" already exists
    When a client POSTs to /auth/register with email "alice@example.com"
    Then the HTTP response status is 409
    And the error message indicates the email is already registered

  Scenario: Registration rejects duplicate username
    Given a user with username "alice" already exists
    When a client POSTs to /auth/register with username "alice" and a different email
    Then the HTTP response status is 409
    And the error message indicates the username is already taken

  Scenario: API request with valid session token is accepted
    Given the client has a valid session cookie from a successful login
    When the client requests a protected endpoint GET /leagues
    Then the HTTP response status is 200

  Scenario: API request with expired session token returns 401
    Given the client has a session cookie that expired 1 hour ago
    When the client requests a protected endpoint GET /leagues
    Then the HTTP response status is 401

  Scenario: API request with invalid session token returns 401
    Given the client presents a session cookie with a forged token value
    When the client requests a protected endpoint GET /leagues
    Then the HTTP response status is 401

  Scenario: Logout invalidates the session token
    Given the client is logged in with a valid session cookie
    When the client POSTs to /auth/logout
    Then the HTTP response status is 200
    And the response clears the session cookie with Max-Age=0 or equivalent
    And subsequent requests using the same token return 401

  Scenario: Account locked after 10 consecutive failed login attempts returns 429
    Given 10 consecutive failed login attempts have been made for "alice@example.com"
    When an 11th login attempt is made for "alice@example.com" with the correct password
    Then the HTTP response status is 429
    And the response message indicates the account is locked

  Scenario: Admin sets a temporary password, target account is flagged for forced change
    Given an Admin is authenticated
    When the Admin POSTs to /admin/users/alice/password with a new temporary password
    Then the HTTP response status is 200
    And alice's account has the "force_password_change" flag set to true

  Scenario: Forced password change user can only access the password change endpoint
    Given alice's account has "force_password_change" = true
    And alice is logged in with a valid session
    When alice requests GET /leagues
    Then the HTTP response status is 403
    And the error message indicates a password change is required

  Scenario: Forced password change user can access the password change endpoint
    Given alice's account has "force_password_change" = true
    And alice is logged in with a valid session
    When alice POSTs to /auth/change-password with a new password
    Then the HTTP response status is 200

  Scenario: Completing forced password change clears the flag
    Given alice's account has "force_password_change" = true
    And alice is logged in with a valid session
    When alice POSTs to /auth/change-password with a valid new password
    Then alice's "force_password_change" flag is set to false
    And alice can now access GET /leagues with HTTP 200

  Scenario: Force-change user can log out
    Given alice's account has "force_password_change" = true
    And alice is logged in with a valid session
    When alice POSTs to /auth/logout
    Then the HTTP response status is 200
    And the session cookie is cleared with Max-Age=0 or equivalent

  Scenario: Force-change guard re-applies after logout and re-login
    Given alice's account has "force_password_change" = true
    And alice has logged out successfully
    When alice POSTs to /auth/login with correct credentials
    And alice requests GET /leagues
    Then the HTTP response status is 403
    And the error message indicates a password change is required

  Scenario: Health check endpoint is accessible without authentication
    Given no session cookie is present
    When a client requests GET /health
    Then the HTTP response status is 200
    And the response body is {"status":"ok"}

  Scenario: Database unavailable during session validation returns 500
    Given alice has a valid session cookie
    And the database connection pool returns an error (simulated failure)
    When alice requests GET /leagues
    Then the HTTP response status is 500
    And the response body contains {"error_code":"INTERNAL_ERROR"}
    And the response body does not contain any internal database error details
```
