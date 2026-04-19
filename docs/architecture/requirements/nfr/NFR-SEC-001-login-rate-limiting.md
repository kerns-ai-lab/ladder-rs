# NFR-SEC-001: Login Rate Limiting

**Status:** Draft
**Parent:** UR-AUTH-001, RQ-R3-5
**Priority:** Must-have

## Description

The login endpoint enforces rate limiting on a per-account basis. After 10 consecutive failed login attempts for the same account, that account is locked out for 15 minutes. Lockout is time-based and lifts automatically. An Admin can unlock any account before the timeout expires.

## Rationale

Credential stuffing and brute-force password attacks are the most common vectors for account compromise. Rate limiting at the account level prevents automated tools from testing large numbers of passwords against a single account. Time-based automatic unlock avoids permanent denial-of-service while still imposing a practical cost on attackers.

## Acceptance Criteria

- [ ] The system tracks consecutive failed login attempts per account (by username/email); the counter resets to zero on any successful login — only unbroken consecutive failures since the last success count toward the lockout threshold
- [ ] After exactly 10 consecutive failed login attempts for an account, the account is placed in a locked state with a lockout timestamp
- [ ] While an account is locked, any login attempt (even with the correct credentials) returns a 429 Too Many Requests response with a message indicating the account is locked and the approximate remaining lockout duration
- [ ] The lockout automatically lifts exactly 15 minutes after the lockout timestamp; subsequent login attempts with correct credentials succeed without admin intervention
- [ ] An Admin-only endpoint exists to immediately unlock a locked account, resetting both the failed attempt counter and the lockout timestamp
- [ ] The failed attempt counter and lockout state are persisted to the database (not held only in memory) so that a server restart does not reset lockout state
- [ ] The lockout is keyed to the account identity (not the IP address); the same account locked from multiple IPs is still locked regardless of which IP makes the next attempt
- [ ] The response for a failed login attempt does not distinguish "wrong password" from "account locked" to avoid leaking lockout state to attackers (a single generic message covers both)

### Behavioral Acceptance Tests (BDD)

```gherkin
Feature: Login Rate Limiting

  Background:
    Given the system is running
    And a user account "alice@example.com" exists with password "correct-password"
    And the lockout threshold is 10 consecutive failed login attempts
    And the lockout duration is 15 minutes

  Scenario: Failed login attempts are tracked per account
    When 5 consecutive failed login attempts are made for "alice@example.com" with wrong passwords
    Then the failed attempt counter for "alice@example.com" is 5
    And the account is not yet locked

  Scenario: Account is locked after exactly 10 consecutive failed login attempts
    When 10 consecutive failed login attempts are made for "alice@example.com" with wrong passwords
    Then the account "alice@example.com" is placed in a locked state
    And the lockout timestamp is recorded in the database

  Scenario: Locked account rejects login even with correct credentials
    Given "alice@example.com" is locked after 10 failed attempts
    When a login attempt is made for "alice@example.com" with the correct password "correct-password"
    Then the HTTP response status is 429
    And the response message indicates the account is locked
    And the response includes an approximate remaining lockout duration

  Scenario: Locked account error response does not distinguish wrong password from lockout
    Given "alice@example.com" is locked
    When a login attempt is made with a wrong password
    Then the HTTP response status is 429
    And the error message is the same generic message used for failed login attempts
    And the message does not specifically say "account locked" in a way that reveals lockout to an attacker

  Scenario: Lockout lifts automatically after 15 minutes
    Given "alice@example.com" was locked at 2026-04-15T10:00:00Z
    And the current server time is 2026-04-15T10:15:01Z (15 minutes and 1 second after lockout)
    When a login attempt is made with the correct password "correct-password"
    Then the HTTP response status is 200
    And the session cookie is set

  Scenario: Lockout does not lift before 15 minutes have elapsed
    Given "alice@example.com" was locked at 2026-04-15T10:00:00Z
    And the current server time is 2026-04-15T10:14:59Z (14 minutes and 59 seconds after lockout)
    When a login attempt is made with the correct password
    Then the HTTP response status is 429

  Scenario: Successful login resets the failed attempt counter to zero
    Given 7 consecutive failed login attempts have been made for "alice@example.com"
    When a login attempt is made with the correct password "correct-password"
    Then the HTTP response status is 200
    And the failed attempt counter for "alice@example.com" is reset to 0

  Scenario: Admin can unlock a locked account before the timeout expires
    Given "alice@example.com" is locked and the Admin "admin1" is authenticated
    When admin1 POSTs to /admin/users/alice/unlock
    Then the HTTP response status is 200
    And the failed attempt counter for "alice@example.com" is reset to 0
    And the lockout timestamp is cleared
    And alice can immediately log in with correct credentials

  Scenario: Lockout state persists across server restarts
    Given "alice@example.com" is locked and the lockout is stored in the database
    When the application process is restarted
    And a login attempt is made for "alice@example.com" with correct credentials after restart
    Then the HTTP response status is 429
    And the lockout is still in effect (not reset by the restart)

  Scenario: Lockout is keyed to account identity, not IP address
    Given "alice@example.com" is locked due to 10 failures from IP 192.168.1.1
    When a login attempt is made for "alice@example.com" with correct credentials from IP 10.0.0.1
    Then the HTTP response status is 429
    And the account remains locked regardless of the request IP address
```
